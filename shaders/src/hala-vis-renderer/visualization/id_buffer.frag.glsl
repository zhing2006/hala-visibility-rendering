#version 460 core

#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_debug_printf : enable
#extension GL_ARB_gpu_shader_int64 : enable

#include "defines.glsl"
#include "visualization.glsl"

layout(input_attachment_index = 0, binding = 0, set = 3) uniform usubpassInput in_input_image;

layout(location = 0) out vec4 out_color;

void main() {
  const uint id = subpassLoad(in_input_image).x;
  out_color = float4(int_to_color(id), 1.0);
}
