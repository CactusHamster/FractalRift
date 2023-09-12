#[allow(dead_code)]

use std::mem::size_of_val;
use gl::{self, types::{GLenum, GLint, GLuint}, COMPILE_STATUS, INFO_LOG_LENGTH, GetShaderInfoLog, AttachShader, GetProgramiv, LINK_STATUS};
use glfw;

pub fn get_glstring<'a> (name: GLenum) -> &'a str {
    let v = unsafe { gl::GetString(name) };
    let v: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(v as *const i8) };
    v.to_str().unwrap()
}

pub fn loadgl (window: &mut glfw::Window) {
    gl::load_with(|ptr| window.get_proc_address(ptr));
}
pub fn viewport (width: i32, height: i32) {
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}
pub fn clearcolor (r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        gl::ClearColor(r, g, b, a);
    }
}
pub fn clear () {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

// Vertex Buffer Object management.
pub enum BufferType {
    Array = gl::ARRAY_BUFFER as isize,
    ElementArray = gl::ELEMENT_ARRAY_BUFFER as isize,
}
pub fn makebuffer () -> u32 {
    let mut ptr: u32 = 0;
    unsafe { gl::GenBuffers(1, &mut ptr); }
    return ptr;
}
pub fn bind_buffer (btype: BufferType, buffer: u32) {
    unsafe {
        gl::BindBuffer(btype as GLenum, buffer);
    }
}
pub fn unbind_buffer(btype: BufferType) {
    unsafe {
        gl::BindBuffer(btype as GLenum, 0);
    }
}

pub fn buffer_data (btype: BufferType, data: &[u8], usage: GLenum) {
    unsafe {
        gl::BufferData(
            btype as u32,
            size_of_val(data).try_into().unwrap(),
            data.as_ptr().cast(),
            usage
        )
    }
}

// Vertex Object Array management.
pub fn make_vertex_object_array () -> u32 {
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }
    return vao;
}
pub fn bind_vertex_object_array (voa: u32) {
    unsafe {
        gl::BindVertexArray(voa);
    }
}
pub fn unbind_vertex_object_array () {
    unsafe {
        gl::BindVertexArray(0);
    }
}

// Shader management.
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize
}
pub struct Shader (GLuint);
impl Shader {
    pub fn new (shader_type: ShaderType) -> Option<Self> {
        let shader = unsafe { gl::CreateShader(shader_type as u32) };
        if shader == 0 { None }
        else { Some(Self(shader)) }
    }
    pub fn source (&self, source_string: &str) {
        unsafe {
            gl::ShaderSource(
                self.0,
                1,
                &(source_string.as_bytes().as_ptr().cast()),
                &(source_string.len().try_into().unwrap())
            )
        }
    }
    pub fn get_log (&self) -> String {
        let mut loglength: i32 = 0;
        unsafe { gl::GetShaderiv(self.0, INFO_LOG_LENGTH, &mut loglength); }
        let mut logv: Vec<u8> = Vec::with_capacity(loglength.try_into().unwrap());
        let mut written: i32 = 0;
        unsafe {
            GetShaderInfoLog(self.0, loglength, &mut written, logv.as_mut_ptr().cast());
            logv.set_len(written.try_into().unwrap());
        }
        return String::from_utf8_lossy(&logv).into_owned();
    }
    pub fn compile (&self) -> bool {
        unsafe { gl::CompileShader(self.0); };
        let mut success: i32 = 0;
        unsafe { gl::GetShaderiv(self.0, COMPILE_STATUS, &mut success); };
        return success as u8 == gl::TRUE;
    }
    pub fn delete (&self) {
        unsafe { gl::DeleteShader(self.0); }
    }
    pub fn from_str (shader_type: ShaderType, source_string: &str) -> Result<Shader, String> {
        let shader = Self::new(shader_type).unwrap();
        shader.source(source_string);
        let success = shader.compile();
        if success == false { Err(shader.get_log()) }
        else { Ok(shader) }
    }
}

pub struct ProgramFromSourceResult {
    pub program: Program,
    pub vertex_shader: Shader,
    pub fragment_shader: Shader
}

pub struct Program (pub u32);
impl Program {
    pub fn new () -> Option<Self> {
        let program: u32 = unsafe { gl::CreateProgram() };
        if program == 0 { None }
        else { Some(Self(program)) }
    }
    pub fn attach (&self, shader: &Shader) {
        unsafe { AttachShader(self.0, shader.0); }
    }
    pub fn get_log (&self) -> String {
        let mut loglength: i32 = 0;
        unsafe { GetProgramiv(self.0, INFO_LOG_LENGTH, &mut loglength); };
        let mut logvector: Vec<u8> = Vec::with_capacity(loglength.try_into().unwrap());
        let mut written: i32 = 0;
        unsafe {
            logvector.set_len(written.try_into().unwrap());
            gl::GetProgramInfoLog(self.0, loglength, &mut written, logvector.as_mut_ptr().cast());
        };
        return String::from_utf8_lossy(&logvector).into_owned();
    }
    pub fn link (&self) -> bool {
        let mut success = 0;
        unsafe {
            gl::LinkProgram(self.0);
            GetProgramiv(self.0, LINK_STATUS, &mut success);
        };
        return success as u8 == gl::TRUE;
    }
    pub fn use_program (&self) {
        unsafe { gl::UseProgram(self.0); };
    }
    pub fn delete (&self) {
        unsafe { gl::DeleteProgram(self.0); }
    }
    pub fn get_uniform_location (&self, name: &str) -> Option<GLint> {
        let location = unsafe { gl::GetUniformLocation(self.0, name.as_ptr().cast()) };
        if location == -1 { None }
        else { Some(location) }
    }
    pub fn from_shader_sources (vertex_source: &str, fragment_source: &str) -> Result<ProgramFromSourceResult, String> {
        let program = Self::new()
            .expect("Failed to create program.");
        let vert = Shader::from_str(ShaderType::Vertex, &vertex_source)
            .expect("Failed to create vertex shader.");
        let frag = Shader::from_str(ShaderType::Fragment, &fragment_source)
            .expect("Failed to create fragment shader.");
        program.attach(&vert);
        program.attach(&frag);
        vert.delete();
        frag.delete();
        let success = program.link();
        if success {
            let result = ProgramFromSourceResult {
                program: program,
                vertex_shader: vert,
                fragment_shader: frag
            };
            Ok(result)
        }
        else {
            let log = program.get_log();
            program.delete();
            Err(log)
        }
    }
}