#version 150 core

uniform sampler2D t_Texture;
in vec2 tex_coord;
out vec4 Target0;

void main() {
    Target0 = texture(t_Texture, tex_coord);
}
