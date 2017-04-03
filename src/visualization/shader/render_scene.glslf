#version 400 core

in vec2 tex_uv;

//uniform sampler2D t_Texture;
uniform uint u_id;

out vec4 out_color;
out uint out_index;

void main() {
    float blue = 1.0 - (tex_uv[0] + tex_uv[1]) / 2.0;
    out_color = vec4(tex_uv, blue, 1.0);       //texture(t_Texture, tex_uv);

    out_index = u_id;
}
