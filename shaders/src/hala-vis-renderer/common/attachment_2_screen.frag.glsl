#version 460 core
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_debug_printf : enable
#extension GL_ARB_gpu_shader_int64 : enable

#include "defines.glsl"

layout(push_constant) uniform AttachmentToScreenPushConstants {
  vec4 scale;
} g_push_constants;

layout(input_attachment_index = 0, binding = 0, set = 3) uniform subpassInput in_input_image;

layout(location = 0) out vec4 out_color;

void main() {
  out_color = subpassLoad(in_input_image) * g_push_constants.scale;
}
