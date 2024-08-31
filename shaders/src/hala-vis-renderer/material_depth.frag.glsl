#version 460 core
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_debug_printf : enable
#extension GL_ARB_gpu_shader_int64 : enable

#include "defines.glsl"
#include "types.hlsl"
#include "scene.hlsl"
#include "visibility.hlsl"

layout(input_attachment_index = 0, binding = 0, set = 3) uniform usubpassInput in_vis_image;
layout(input_attachment_index = 1, binding = 1, set = 3) uniform subpassInput in_depth_image;

void main() {
  const float depth = subpassLoad(in_depth_image).x;
  if (depth <= 0.0) {
    gl_FragDepth = 1.0;
  } else {
    const uint id = subpassLoad(in_vis_image).x;
    uint meshlet_index, triangle_id;
    unpack_meshlet_triangle_index(id, meshlet_index, triangle_id);

    const Meshlet meshlet = g_global_meshlets.data[meshlet_index];
    const DrawData draw_data = g_draw_data.data[meshlet.draw_index];
    gl_FragDepth = float(draw_data.material_index) / float(CLASSIFY_DEPTH_RANGE);
  }
}
