#version 330

uniform sampler2D tex1;
uniform float is_horizontal;

in vec4 pip_color; // FIXME: UNUSED
in vec2 pip_uv;

out vec4 frag_color;

const float weight[5] = float[]
    (0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

void main() {
    // size of single texel
    vec2 uv_unit = vec2(1.0, 1.0) / textureSize(tex1, 0);

    // blur direction ([1, 0] or [0, 1])
    vec2 dir = vec2(is_horizontal, 1.0 - is_horizontal);

    // vec4 result = texture(tex1, pip_uv).rgba * weight[0];
    float alpha = texture(tex1, pip_uv).a * weight[0];

    for(int i = 1; i < 5; ++i) {
        // result += texture(tex1, pip_uv + ((i * uv_unit) * dir)).rgba * weight[i];
        // result += texture(tex1, pip_uv - ((i * uv_unit) * dir)).rgba * weight[i];
        alpha += texture(tex1, pip_uv + ((i * uv_unit) * dir)).w * weight[i];
        alpha += texture(tex1, pip_uv - ((i * uv_unit) * dir)).w * weight[i];
    }

    // frag_color = result;
    frag_color = vec4(0, 0, 0, alpha);
}
