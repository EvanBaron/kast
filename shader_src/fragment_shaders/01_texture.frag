#version 460
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 tex_coord;
layout(location = 1) flat in uint texture_id;

layout(set = 0, binding = 2) uniform sampler2D tex_sampler[16];

layout(location = 0) out vec4 color;

void main() {
    color = texture(tex_sampler[texture_id], tex_coord);
}
