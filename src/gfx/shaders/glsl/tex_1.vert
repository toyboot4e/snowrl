#version 330

uniform mat4 transform;

layout(location=0) in vec2 vert_pos;
layout(location=1) in vec4 vert_color;
layout(location=2) in vec2 vert_uv;

out vec4 color;
out vec2 uv;

void main() {
	gl_Position = transform * vec4(vert_pos, 0.0, 1.0);
    color = vert_color;
    uv = vert_uv;
}
