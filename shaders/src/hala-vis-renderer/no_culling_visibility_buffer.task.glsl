#version 460 core

#extension GL_EXT_mesh_shader : require
#extension GL_KHR_shader_subgroup_basic : require
#extension GL_KHR_shader_subgroup_ballot : require
#extension GL_KHR_shader_subgroup_vote : require
#extension GL_EXT_nonuniform_qualifier : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_debug_printf : enable
#extension GL_ARB_gpu_shader_int64 : enable
#extension GL_EXT_samplerless_texture_functions : enable

#include "no_culling_visibility_buffer.as_6_8.hlsl"