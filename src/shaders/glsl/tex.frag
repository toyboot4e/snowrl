#version 330

uniform sampler2D tex1;

in vec4 color;
in vec2 uv;

out vec4 frag_color;

void main() {
    frag_color = texture(tex1, uv) * color;
}
