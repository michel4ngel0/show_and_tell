#version 150 core

in vec3 v_pos_xyz;
in vec2 v_tex_uv;

uniform mat4 u_model;
uniform mat4 u_camera_projection;
uniform uint u_id;

out vec2 tex_uv;
flat out uint id;

void main() {
    tex_uv = v_tex_uv;
    id = u_id;
    gl_Position = u_camera_projection * u_model * vec4(v_pos_xyz, 1.0);
}
