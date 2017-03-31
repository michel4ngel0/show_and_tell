#version 150 core

in vec2 pos2d;
in vec2 tex_uv;

uniform Locals {
	mat4 u_Model;
	mat4 u_CameraProjection;
};

out vec2 tex_coord;

void main() {
    tex_coord = tex_uv;
    gl_Position = u_CameraProjection * u_Model * vec4(pos2d, 0.0, 1.0);
}
