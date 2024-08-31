#ifdef HALA_HLSL

#include "scene.hlsl"

struct VertexInput {
  [[vk::location(0)]] float4 position: POSITION;
};

struct ToFragment {
  float4 position: SV_Position;
};

ToFragment main(VertexInput input) {
  #define IN_POSITION input.position.xyz

  #define OUT_POSITION output.position

  ToFragment output = (ToFragment)0;

  const ObjectUniform per_object_data = g_per_object_uniforms[g_push_constants.object_index];

#else

#include "scene.glsl"

layout(location = 0) in float3 in_position;

void main() {
  #define IN_POSITION in_position

  #define OUT_POSITION gl_Position

  #define per_object_data (g_per_object_uniforms[g_push_constants.object_index])

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.
  OUT_POSITION = mul(per_object_data.mvp_mtx, float4(IN_POSITION, 1.0));
  // End Function Code.
  //////////////////////////////////////////////////////////////////////////

#ifdef HALA_HLSL
  return output;
#endif
}