// Use OpenGL 3.3.0
#version 330 core
precision lowp float;
uniform float aspect_ratio;
uniform vec3 camera;
uniform float time;
uniform int iterations;
in vec2 vertex_position;
out vec4 color;
vec2 add (vec2 a, vec2 b) { return vec2(a.x + b.x, a.y + b.y); }
vec2 mult (vec2 a, vec2 b) {
    return vec2(
        a.x * b.x - a.y * b.y,
        a.x * b.y + a.y * b.x
    );
}
float len (vec2 a) {
    return sqrt(a.x*a.x + a.y*a.y);
}
float prop(float n, float newmin, float newmax, float oldmin, float oldmax) {
    return ((n - oldmin) / (oldmax - oldmin)) * (newmax - newmin) + newmin;
}
vec3 hsv(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

int squishy (vec2 uv) {
    vec2 c = vec2( (uv.x * uv.y), (uv.x * uv.y) );
    float t = time / 10;
    vec2 z1 = vec2( cos(uv.x + t), sin(uv.y + t) );
    vec2 z2 = vec2( sin(uv.x + t), cos(uv.y + t) );
    int i = 0;
    do {
        z1 = add(mult(z1, z1), c);
        z2 = add(mult(z2, z2), c);
        i = i + 1;
    } while (i < iterations && (len(z1) <= 10.0 || len(z2) <= 10.0));
    return i;
}

void main () {
    mat2 bounds;
    float zoom = float(camera.z);
    bounds[0] = vec2(-4.0 * zoom, 4.0 * zoom);
    bounds[1] = vec2(-4.0 * zoom, 4.0 * zoom);
    vec2 uv = vec2(
        prop(vertex_position.x * aspect_ratio, bounds[0][0], bounds[0][1], -1.0, 1.0) + camera.x,
        prop(vertex_position.y * aspect_ratio, bounds[1][0], bounds[1][1], -1.0, 1.0) + camera.y
    );
    int i = squishy(uv);



    //float r = float(i) / float(iterations);
    //color = vec4(r, 1.0, 1.0, 1.0 );
    color = vec4(
        0.5 * (sin(2 * 3.14 * i * 0.006 - 1.53) + 1),
        0.5 * (sin(2 * 3.14 * i * 0.006 - 0.7) + 1),
        0.5 * (sin(2 * 3.14 * i * 0.006 - 0.0) + 1),
        1.0
    );
    //if (i == 1) color = vec4(0.0, 0.0, 0.0, 1.0);
    //else color = vec4(1.0, 0.0, 1.0, 1.0);
}