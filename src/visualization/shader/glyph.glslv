#version 400 core

in vec2 v_pos_xy;
in vec2 v_tex_uv;

uniform mat3 u_transform;

out vec2 tex_uv;

void main() {
	tex_uv = vec2(v_tex_uv.x, 1.0 - v_tex_uv.y);
    vec3 new_pos = u_transform * vec3(v_pos_xy, 1.0);
    gl_Position = vec4(new_pos.xy, 0.0, 1.0);
}
