// Use OpenGL 3.3
#version 330 core
// Include the vector attribute at location 0. Save it as `pos`.
layout (location = 0) in vec3 pos;
out vec2 vertex_position;
void main () {
    vertex_position = pos.xy;
    gl_Position = vec4(pos.xyz, 1.0);
}