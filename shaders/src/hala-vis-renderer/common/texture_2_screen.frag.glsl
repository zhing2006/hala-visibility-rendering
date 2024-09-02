#version 460 core
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_debug_printf : enable
#extension GL_ARB_gpu_shader_int64 : enable

#include "defines.glsl"

layout(set = 3, binding = 0) uniform sampler2D in_image;

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

void main() {
  out_color = texture(in_image, in_uv);
}
