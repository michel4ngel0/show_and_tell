#version 150 core

in vec2 tex_uv;

uniform sampler2D u_rendered_scene;

out vec4 out_color;

void main() {
    out_color = texture(u_rendered_scene, tex_uv);
}
