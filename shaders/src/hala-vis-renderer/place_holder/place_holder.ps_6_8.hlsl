#define USE_MESH_SHADER

#ifdef HALA_GLSL

  #include "scene.glsl"

  layout(location = 0) in float2 in_uv;
  layout(location = 1) in flat uint in_material_index;

  layout(location = 0) out float4 out_color;

  void main() {

#else

  #include "scene.hlsl"

  struct ToFragment {
    [[vk::location(0)]] float2 uv: TEXCOORD0;
    [[vk::location(1)]] nointerpolation uint material_index: MATERIAL_INDEX;
  };

  struct FragmentOutput {
    [[vk::location(0)]] float4 color: SV_Target0;
  };

  FragmentOutput main(ToFragment input) {
    #define in_uv input.uv
    #define in_material_index input.material_index
    FragmentOutput output = (FragmentOutput)0;

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.
  const Material mtrl = g_materials[in_material_index].data;

  float4 color;
  if (mtrl.base_color_map_index != INVALID_INDEX) {
    float3 base_color = SAMPLE_TEXTURE(g_textures[mtrl.base_color_map_index], g_samplers[mtrl.base_color_map_index], in_uv).rgb;
    color = float4(base_color, 1.0);
  } else {
    color = float4(mtrl.base_color, 1.0);
  }
  // End Function Code.
  //////////////////////////////////////////////////////////////////////////

#ifdef HALA_GLSL
  out_color = color;
#else
  output.color = color;
  return output;
#endif
}