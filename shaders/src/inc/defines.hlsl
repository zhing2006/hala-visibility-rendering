#ifndef _DEFINES_HLSL_
#define _DEFINES_HLSL_

#ifdef HALA_HLSL

#define DISPATCH_MESH(x, y, z, payload) DispatchMesh(x, y, z, payload)

#define BEGIN_PUSH_CONSTANTS(name) struct name {
#define END_PUSH_CONSTANTS(name, var) }; \
  [[vk::push_constant]] \
  name var;

#define BEGIN_UNIFORM_BUFFER(_set, _binding, name) struct name {
#define END_UNIFORM_BUFFER(_set, _binding, name, var) }; \
  [[vk::binding(_binding, _set)]] \
  ConstantBuffer<name> var : register(b##_binding, space##_set);

#define BEGIN_UNIFORM_BUFFER_BINDLESS(_set, _binding, name) struct name {
#define END_UNIFORM_BUFFER_BINDLESS(_set, _binding, name, var) }; \
  [[vk::binding(_binding, _set)]] \
  ConstantBuffer<name> var[] : register(b##_binding, space##_set);

#define BEGIN_BUFFER(_set, _binding, name)
#define END_BUFFER(_set, _binding, name, var) \
  [[vk::binding(_binding, _set)]] \
  StructuredBuffer<name> var : register(t##_binding, space##_set);

#define BEGIN_RWBUFFER(_set, _binding, name)
#define END_RWBUFFER(_set, _binding, name, var) \
  [[vk::binding(_binding, _set)]] \
  RWStructuredBuffer<name> var : register(u##_binding, space##_set);

#define BEGIN_BUFFER_BINDLESS(_set, _binding, name)
#define END_BUFFER_BINDLESS(_set, _binding, name, var) \
  [[vk::binding(_binding, _set)]] \
  StructuredBuffer<name> var[] : register(t##_binding, space##_set);

#define TEXTURE2D(_set, _binding, var) \
  [[vk::binding(_binding, _set)]] \
  Texture2D<float4> var : register(t##_binding, space##_set);

#define TEXTURE2D_BINDLESS(_set, _binding, var) \
  [[vk::binding(_binding, _set)]] \
  Texture2D<float4> var[] : register(t##_binding, space##_set);

#define SAMPLER(_set, _binding, var) \
  [[vk::binding(_binding, _set)]] \
  SamplerState var : register(s##_binding, space##_set);

#define SAMPLER_BINDLESS(_set, _binding, var) \
  [[vk::binding(_binding, _set)]] \
  SamplerState var[] : register(s##_binding, space##_set);

#define SAMPLE_TEXTURE(tex, sampler, uv) (tex.Sample(sampler, uv))
#define SAMPLE_TEXTURE_GRAD(tex, sampler, uv, dx, dy) (tex.SampleGrad(sampler, uv, dx, dy))
#define SAMPLE_TEXTURE_LEVEL(tex, sampler, uv, level) (tex.SampleLevel(sampler, uv, level))

#define LOAD_SAMPLE(tex, uv, level) (tex.Load(uint3(uv, level)))

#define LOAD_SUBPASS_INPUT(input) (input.SubpassLoad())

#define INTERLOCKED_OR(ptr, value, out) (InterlockedOr(ptr, value, out))

#define INTERLOCKED_ADD_RWBUFFER(buffer, addr, value, out) (buffer.InterlockedAdd(addr, value, out))

#define LOAD_BUFFER(buffer, addr) (buffer.Load(addr))
#define STORE_RWBUFFER(buffer, addr, value) (buffer.Store(addr, value))

#define BEGIN_CONST(type, name) static const type name = {
#define END_CONST() };

#define ANNOTATION_BRANCH [branch]
#define ANNOTATION_UNROLL [unroll]

#endif

#define MAX_CAMERAS 8
#define MAX_LIGHTS 16
#define INVALID_INDEX 0xFFFFFFFF
#define DIV_UP(a, b) (((a) + (b) - 1) / (b))

#ifdef USE_MESH_SHADER
#define TASK_SHADER_GROUP_SIZE 32
#define MESH_SHADER_GROUP_SIZE 64
#endif

#define ERROR_COLOR float4(1, 0, 1, 1)

#endif // _DEFINES_HLSL_