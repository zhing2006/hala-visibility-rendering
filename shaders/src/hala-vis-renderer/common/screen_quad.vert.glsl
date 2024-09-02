#version 460 core

#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_debug_printf : enable
#extension GL_ARB_gpu_shader_int64 : enable

#include "defines.glsl"

layout(location = 0) out float2 out_uv;

void main() {
  #define IN_VERTEX_ID gl_VertexIndex

  #define OUT_POSITION gl_Position
  #define OUT_UV out_uv

  const float2 uv = float2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2) * 0.5;
  gl_Position = float4(uv * 2.0 - 1.0, 0.0, 1.0);
  out_uv = uv * float2(1, -1) + float2(0, 1);
}