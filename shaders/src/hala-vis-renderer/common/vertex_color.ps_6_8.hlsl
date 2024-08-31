#ifdef HALA_GLSL

  #include "scene.glsl"

  layout(location = 0) in float3 in_color;
  layout(location = 1) in flat uint in_material_index;

  layout(location = 0) out float4 out_color;

  void main() {

#else

  #include "scene.hlsl"

  struct ToFragment {
    [[vk::location(0)]] float3 color: COLOR0;
    [[vk::location(1)]] nointerpolation uint material_index: MATERIAL_INDEX;
  };

  struct FragmentOutput {
    [[vk::location(0)]] float4 color: SV_Target0;
  };

  FragmentOutput main(ToFragment input) {
    #define in_color input.color
    #define in_material_index input.material_index
    FragmentOutput output = (FragmentOutput)0;

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.

  float4 color = float4(in_color, 1.0);

  // End Function Code.
  //////////////////////////////////////////////////////////////////////////

#ifdef HALA_GLSL
  out_color = color;
#else
  output.color = color;
  return output;
#endif
}