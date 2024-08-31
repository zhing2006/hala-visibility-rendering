#version 460 core
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_debug_printf : enable
#extension GL_ARB_gpu_shader_int64 : enable
#extension GL_EXT_samplerless_texture_functions : enable

#include "depth_reduction.ps_6_8.hlsl"