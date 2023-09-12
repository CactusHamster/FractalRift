use bytemuck;
use std::time::Duration;
mod birbgl;
use birbgl::{clearcolor, clear, viewport, BufferType};
use gl::{STATIC_DRAW, DrawElements};
use glfw::{Action, Context, Key, WindowEvent};

use crate::birbgl::{get_glstring, ProgramFromSourceResult};

type Vertex = [f32; 3];
type TriIndexes = [u32; 3];
const VERTICES: [Vertex; 4] = [
    [1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [-1.0, 1.0, 0.0]
];
// Define which vertices make triangles.
const INDICES: [TriIndexes; 2] = [
    [0, 1, 3],
    [1, 2, 3]
];

const VERT_SHADER_SOURCE: &str = include_str!("vert.glsl");
const FRAG_SHADER_SOURCE: &str = include_str!("frag.glsl");

const TRANSLATE_SPEED: f32 = 0.3;
const ZOOM_SPEED: f32 = 1.05;

struct Camera {
    x: f32,
    y: f32,
    zoom_level: f32
}
#[allow(dead_code)]
impl Camera {
    pub fn new () -> Self {
        return Camera {
            x: 0.0,
            y: 0.0,
            zoom_level: 1.0
        }
    }
    pub fn translatex (&mut self, x: f32) {
        self.x += x * self.zoom_level;
    }
    pub fn translatey (&mut self, y: f32) {
        self.y += y * self.zoom_level;
    }
    pub fn translate (&mut self, x: f32, y: f32) {
        self.translatex(x);
        self.translatey(y);
    }
    pub fn zoom (&mut self, z: f32) {
        self.zoom_level *= z;
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::Resizable(true));
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    let (mut window, events) = glfw.create_window(600, 800, "STAR", glfw::WindowMode::Windowed)
        .expect("Failed to create window.");
    window.make_current();
    window.maximize();
    window.set_key_polling(true);
    window.set_size_polling(true);
    birbgl::loadgl(&mut window);

    let gl_version = get_glstring(gl::VERSION);
    let glsl_version = get_glstring(gl::SHADING_LANGUAGE_VERSION);
    println!("Using OpenGL {}.", gl_version);
    println!("Using GLSL {}.", glsl_version);

    let (screen_width, screen_height) = window.get_framebuffer_size();
    viewport(screen_width, screen_height);
    clearcolor(0.0, 0.0, 0.0, 1.0);

    // Generate Vertex/Element buffers
    let vao = birbgl::make_vertex_object_array();
    let vbo = birbgl::makebuffer();
    let ebo = birbgl::makebuffer();

    birbgl::bind_vertex_object_array(vao);
    birbgl::bind_buffer(BufferType::Array, vbo);
    birbgl::buffer_data(BufferType::Array, bytemuck::cast_slice(&VERTICES), STATIC_DRAW);
    birbgl::bind_buffer(birbgl::BufferType::ElementArray, ebo);
    birbgl::buffer_data(BufferType::ElementArray, bytemuck::cast_slice(&INDICES), STATIC_DRAW);
    unsafe {
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE as u8,
            3 * std::mem::size_of::<f32>() as i32,
            0 as *const _,
        );
    }
    birbgl::unbind_buffer(BufferType::Array);

    let ProgramFromSourceResult {
        program: shader_program,
        vertex_shader: _,
        fragment_shader: _
    } = birbgl::Program::from_shader_sources(VERT_SHADER_SOURCE, FRAG_SHADER_SOURCE)
    .unwrap_or_else(|e| panic!("Failure: {}", e));
    shader_program.use_program();
    let aspect_ratio_uniform = shader_program.get_uniform_location("aspect_ratio\0")
        .expect("Failed to get aspect_ratio uniform.");
    let camera_uniform = shader_program.get_uniform_location("camera\0")
        .expect("Faled to get camera uniform.");
    let time_uniform = shader_program.get_uniform_location("time\0")
        .expect("Failed to get time uniform.");
    let iterations_uniform = shader_program.get_uniform_location("iterations\0")
        .expect("Failed to get iterations uniform.");
    let mut camera: Camera = Camera::new();
    let mut frame: f32 = 0.0;
    let mut paused = false;
    let mut iterations = 30;
    let mut rerender = true;
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Size(width, height) =>  { viewport(width, height); },
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => { window.set_should_close(true); },
                WindowEvent::Key(Key::Space, _, Action::Press, _) => { paused = !paused; },
                _ => {}
            }
        }
        unsafe {
            if window.get_key(Key::W) == Action::Press { camera.translatey(TRANSLATE_SPEED); rerender = true; }
            else if window.get_key(Key::S) == Action::Press { camera.translatey(-TRANSLATE_SPEED); rerender = true; }
            if window.get_key(Key::A) == Action::Press { camera.translatex(-TRANSLATE_SPEED); rerender = true; }
            else if window.get_key(Key::D) == Action::Press { camera.translatex(TRANSLATE_SPEED); rerender = true; }
            if window.get_key(Key::Equal) == Action::Press { camera.zoom(1.0 / ZOOM_SPEED); rerender = true; }
            else if window.get_key(Key::Minus) == Action::Press { camera.zoom(ZOOM_SPEED); rerender = true; }
            if window.get_key(Key::Period) == Action::Press { iterations += 1; rerender = true; }
            else if window.get_key(Key::Comma) == Action::Press { iterations -= 1; rerender = true; }
            if !paused || rerender {
                if !paused { frame += 0.1 };
                let (width, height) = window.get_framebuffer_size();
                gl::Uniform1f(aspect_ratio_uniform, width as f32 / height as f32);
                gl::Uniform3f(camera_uniform, camera.x, camera.y, camera.zoom_level);
                gl::Uniform1f(time_uniform, frame);
                gl::Uniform1i(iterations_uniform, iterations);
                clear();
                DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
                window.swap_buffers();
                rerender = false;
            }
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
