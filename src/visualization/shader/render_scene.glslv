#version 400 core

in vec3 v_pos_xyz;
in vec2 v_tex_uv;

uniform mat4 u_model;
uniform mat4 u_camera_projection;

out vec2 tex_uv;

void main() {
    tex_uv = vec2(v_tex_uv.x, 1.0 - v_tex_uv.y);
    gl_Position = u_camera_projection * u_model * vec4(v_pos_xyz, 1.0);
}
