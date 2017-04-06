#version 400 core

in vec2 tex_uv;

uniform sampler2D u_font_bitmap;
uniform float tex_u;
uniform float tex_v;

out vec4 out_color;

void main() {
    vec2 my_tex_uv = vec2(tex_uv.x / 16.0, tex_uv.y / 6.0);
    my_tex_uv += vec2(tex_u, tex_v);
    out_color = texture(u_font_bitmap, my_tex_uv);
}
