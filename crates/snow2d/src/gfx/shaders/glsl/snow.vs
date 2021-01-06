#version 330

uniform mat4 transform;

layout(location=0) in vec2 vert_pos;

out vec2 fragCoord;

void main() {
    // gl_Position = vec4(vert_pos / 2.0 - vec2(0.0, 1.0), 0.0, 1.0);
    gl_Position = transform * vec4(vert_pos, 0.0, 1.0);

    // send coordinates in pixels
    fragCoord = vec2(vert_pos.x, vert_pos.y);
}
