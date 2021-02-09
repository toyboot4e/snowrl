#version 330

layout(location=0) in vec2 vs_pos;
layout(location=1) in vec2 vs_uv;

out vec2 fs_uv;

void main() {
    gl_Position = vec4(vs_pos, 0, 1);
    fs_uv = vs_uv;
}
