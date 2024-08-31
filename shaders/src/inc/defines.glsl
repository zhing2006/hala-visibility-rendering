#ifndef _DEFINES_GLSL_
#define _DEFINES_GLSL_

#define float2 vec2
#define float3 vec3
#define float4 vec4
#define float4x4 mat4
#define float3x3 mat3
#define float2x2 mat2
#define int2 ivec2
#define int3 ivec3
#define int4 ivec4
#define uint2 uvec2
#define uint3 uvec3
#define uint4 uvec4

#ifdef HALA_GLSL

#define printf debugPrintfEXT
#define SetMeshOutputCounts SetMeshOutputsEXT
#define WavePrefixCountBits(x) subgroupBallotExclusiveBitCount(subgroupBallot(x))
#define WaveActiveCountBits(x) subgroupBallotBitCount(subgroupBallot(x))
#define GroupMemoryBarrierWithGroupSync() groupMemoryBarrier(); \
  barrier()
#define GroupMemoryBarrier() groupMemoryBarrier()
#define DeviceMemoryBarrierWithGroupSync() memoryBarrier(); \
  memoryBarrierBuffer(); \
  memoryBarrierImage(); \
  barrier()
#define DeviceMemoryBarrier() memoryBarrier(); \
  memoryBarrierBuffer(); \
  memoryBarrierImage()
#define AllMemoryBarrierWithGroupSync() groupMemoryBarrier(); \
  memoryBarrier(); \
  memoryBarrierBuffer(); \
  memoryBarrierImage(); \
  barrier()
#define AllMemoryBarrier() groupMemoryBarrier(); \
  memoryBarrier(); \
  memoryBarrierBuffer(); \
  memoryBarrierImage()
#define inline

#define mul(a, b) ((a) * (b))
#define saturate(a) clamp(a, 0.0, 1.0)
#define lerp(a, b, t) mix(a, b, t)
#define asuint(a) floatBitsToUint(a)
#define asfloat(a) uintBitsToFloat(a)
#define firstbitlow(a) findLSB(a)
#define firstbithigh(a) findMSB(a)
#define rcp(a) (1.0 / a)
#define rsqrt(a) inversesqrt(a)
#define frac(a) fract(a)

#define DISPATCH_MESH(x, y, z, payload) EmitMeshTasksEXT(x, y, z)

#define BEGIN_PUSH_CONSTANTS(name) layout(push_constant) uniform name {
#define END_PUSH_CONSTANTS(name, var) } var;

#define BEGIN_UNIFORM_BUFFER(_set, _binding, name) layout(set = _set, binding = _binding) uniform name {
#define END_UNIFORM_BUFFER(_set, _binding, name, var) } var;

#define BEGIN_UNIFORM_BUFFER_BINDLESS(_set, _binding, name) layout(set = _set, binding = _binding) uniform name {
#define END_UNIFORM_BUFFER_BINDLESS(_set, _binding, name, var) } var[];

#define BEGIN_BUFFER(_set, _binding, name) layout(set = _set, binding = _binding) buffer name##Buffer##_set##_binding { \
  name data[];
#define END_BUFFER(_set, _binding, name, var) } var;

#define BEGIN_RWBUFFER(_set, _binding, name) layout(set = _set, binding = _binding) buffer name##Buffer##_set##_binding { \
  name data[];
#define END_RWBUFFER(_set, _binding, name, var) } var;

#define BEGIN_BUFFER_BINDLESS(_set, _binding, name) layout(set = _set, binding = _binding) buffer name##Buffer##_set##_binding { \
  name data[];
#define END_BUFFER_BINDLESS(_set, _binding, name, var) } var[];

#define TEXTURE2D(_set, _binding, var) layout(set = _set, binding = _binding) uniform texture2D var;

#define TEXTURE2D_BINDLESS(_set, _binding, var) layout(set = _set, binding = _binding) uniform texture2D var[];

#define SAMPLER(_set, _binding, var) layout(set = _set, binding = _binding) uniform sampler var;

#define SAMPLER_BINDLESS(_set, _binding, var) layout(set = _set, binding = _binding) uniform sampler var[];

#define SAMPLE_TEXTURE(tex, sampler, uv) (texture(sampler2D(tex, sampler), uv))
#define SAMPLE_TEXTURE_GRAD(tex, sampler, uv, dPdx, dPdy) (textureGrad(sampler2D(tex, sampler), uv, dPdx, dPdy))
#define SAMPLE_TEXTURE_LEVEL(tex, sampler, uv, level) (textureLod(sampler2D(tex, sampler), uv, level))

#define LOAD_SAMPLE(sampler, uv, level) (texelFetch(sampler, ivec2(uv), int(level)))

#define LOAD_SUBPASS_INPUT(input) (subpassLoad(input))

#define INTERLOCKED_OR(ptr, value, out) (out = atomicOr(ptr, value))

#define INTERLOCKED_ADD_RWBUFFER(ptr, addr, value, out) (out = atomicAdd(ptr[(addr) / 4], value))

#define LOAD_BUFFER(ptr, addr) (ptr[(addr) / 4])
#define STORE_RWBUFFER(ptr, addr, value) (ptr[(addr) / 4] = value)

#define BEGIN_CONST(type, name) const type name = type(
#define END_CONST() );

#define ANNOTATION_BRANCH
#define ANNOTATION_UNROLL

#endif

#include "defines.hlsl"

#endif  // _DEFINES_GLSL_