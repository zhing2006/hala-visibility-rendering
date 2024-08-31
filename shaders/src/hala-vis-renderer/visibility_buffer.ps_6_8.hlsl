#define USE_MESH_SHADER

#ifdef HALA_GLSL

#include "scene.glsl"
// #include "hala-vis-renderer/visibility.glsl"

layout(location = 0) out uint out_color;

void main() {
  #define IN_PRIMITIVE_ID gl_PrimitiveID
  #define OUT_COLOR out_color

#else

#include "scene.hlsl"
// #include "visibility.hlsl"

struct FragmentOutput {
  [[vk::location(0)]] uint color: SV_Target0;
};

FragmentOutput main(uint primitive_id: SV_PrimitiveID) {
  #define IN_PRIMITIVE_ID primitive_id
  #define OUT_COLOR output.color

  FragmentOutput output = (FragmentOutput)0;

#endif

  OUT_COLOR = IN_PRIMITIVE_ID;
  // uint meshlet_index, triangle_id;
  // unpack_meshlet_triangle_index(uint(IN_PRIMITIVE_ID), meshlet_index, triangle_id);
  // if (meshlet_index == 1) {
  //   printf("meshlet_index: %d, triangle_id: %d\n", meshlet_index, triangle_id);
  // }

#ifdef HALA_HLSL
  return output;
#endif
}