#version 150 core

in vec2 tex_uv;
flat in uint id;
//uniform sampler2D t_Texture;

out vec4 out_color;
out vec4 out_index;

void main() {
    float blue = 1.0 - (tex_uv[0] + tex_uv[1]) / 2.0;
    out_color = vec4(tex_uv, blue, 1.0);       //texture(t_Texture, tex_uv);
    float fff = (float(id) / 30.0) + 0.1;
    out_index = vec4(fff, 0, 0, 0);      //vec4(u_id & 255, (u_id >> 8) & 255, (u_id >> 16) & 255, (u_id >> 24) & 255);
}
