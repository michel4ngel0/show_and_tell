#version 150 core

in vec2 v_pos_xy;
in vec2 v_tex_uv;

out vec2 tex_uv;

void main() {
	tex_uv = v_tex_uv;
    gl_Position = vec4(2.0 * v_pos_xy, 0.0, 1.0);
}
