#define HALA_NO_GLOBAL_PUSH_CONSTANT

#ifdef HALA_HLSL

  #include "scene.hlsl"
  #include "visibility.hlsl"
  #include "material_tile.hlsl"

  [[vk::input_attachment_index(0)]]
  [[vk::binding(1, 3)]]
  SubpassInput<uint> in_vis_image;

  struct FragmentOutput {
    [[vk::location(0)]] float4 albedo: SV_Target0;
    [[vk::location(1)]] float4 normal: SV_Target1;
  };

  FragmentOutput main(ToFragment input) {
    #define IN_POSITION input.position
    FragmentOutput output = (FragmentOutput)0;
    #define OUT_ALBEDO output.albedo
    #define OUT_NORMAL output.normal

#else

  #include "scene.glsl"
  #include "hala-vis-renderer\visibility.glsl"
  #include "hala-vis-renderer\material_tile.hlsl"

  layout(input_attachment_index = 0, binding = 1, set = 3) uniform usubpassInput in_vis_image;

  layout(location = 0) out float4 out_albedo;
  layout(location = 1) out float4 out_normal;

  void main() {
    #define IN_POSITION gl_FragCoord
    #define OUT_ALBEDO out_albedo
    #define OUT_NORMAL out_normal

    #define g_global_meshlets (g_global_meshlets.data)
    #define g_draw_data (g_draw_data.data)

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.

  const float2 pixel_pos = IN_POSITION.xy;

  const uint id = LOAD_SUBPASS_INPUT(in_vis_image).x;
  uint meshlet_index, triangle_id;
  unpack_meshlet_triangle_index(id, meshlet_index, triangle_id);

  const Meshlet meshlet = g_global_meshlets[meshlet_index];
  const DrawData draw_data = g_draw_data[meshlet.draw_index];
  const Material mtrl = g_materials[draw_data.material_index].data;

  uint triangle_index = meshlet.offset_of_primitives + triangle_id;
  const uint3 tri = load_primitive_index(triangle_index, meshlet.draw_index);
  const VertexAttributes vertex_attributes = get_vertex_attributes(float2(g_push_constants.screen_size), pixel_pos, draw_data, tri, meshlet);

  if (mtrl.base_color_map_index != INVALID_INDEX) {
    float3 base_color = SAMPLE_TEXTURE_GRAD(
      g_textures[mtrl.base_color_map_index],
      g_samplers[mtrl.base_color_map_index],
      vertex_attributes.texcoord,
      vertex_attributes.texcoord_ddx,
      vertex_attributes.texcoord_ddy
    ).rgb;
    OUT_ALBEDO = float4(base_color, 1.0);
  } else {
    OUT_ALBEDO = float4(mtrl.base_color, 1.0);
  }

  OUT_NORMAL = float4(vertex_attributes.normal * 0.5 + 0.5, 1.0);

  // End Function Code.
  //////////////////////////////////////////////////////////////////////////

#ifdef HALA_HLSL
  return output;
#endif
}