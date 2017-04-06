#version 400 core

in vec2 tex_uv;

uniform sampler2D u_texture;
uniform int u_texture_bound;
uniform vec3 u_color;
uniform uint u_id;
uniform vec3 u_selection_highlight;

out vec4 out_color;
out uint out_index;

void main() {
    out_color = (u_texture_bound != 0) ? texture(u_texture, tex_uv) : vec4(u_color, 1.0);
    out_color += 0.1 * vec4(u_selection_highlight, 0.0);
    out_index = u_id;
}
