#define HALA_NO_GLOBAL_PUSH_CONSTANT

#ifdef HALA_HLSL

  #include "scene.hlsl"

  [[vk::input_attachment_index(0)]]
  [[vk::binding(0, 3)]]
  SubpassInput<float3> in_albedo_image;

  [[vk::input_attachment_index(1)]]
  [[vk::binding(1, 3)]]
  SubpassInput<float3> in_normal_image;

  [[vk::input_attachment_index(2)]]
  [[vk::binding(2, 3)]]
  SubpassInput<float> in_depth_image;

  struct ToFragment {
    float4 position: SV_Position;
    [[vk::location(0)]] float2 uv: TEXCOORD0;
  };

  struct FragmentOutput {
    [[vk::location(0)]] float4 color: SV_Target0;
  };

  FragmentOutput main(ToFragment input) {
    FragmentOutput output = (FragmentOutput)0;
    #define IN_UV input.uv
    #define OUT_COLOR output.color

#else

  #include "scene.glsl"

  layout(input_attachment_index = 0, binding = 0, set = 3) uniform subpassInput in_albedo_image;
  layout(input_attachment_index = 1, binding = 1, set = 3) uniform subpassInput in_normal_image;
  layout(input_attachment_index = 2, binding = 2, set = 3) uniform subpassInput in_depth_image;

  layout(location = 0) in float2 in_uv;

  layout(location = 0) out float4 out_color;

  void main() {
    #define IN_UV in_uv
    #define OUT_COLOR out_color

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.

  const float3 albedo = LOAD_SUBPASS_INPUT(in_albedo_image).rgb;
  const float3 normal = LOAD_SUBPASS_INPUT(in_normal_image).xyz * 2.0 - 1.0;
  const float depth = LOAD_SUBPASS_INPUT(in_depth_image).x;

  if (depth <= 0.0) {
    discard;
  }

  const float4 clip_pos = float4(IN_UV * 2.0 - 1.0, depth, 1.0);
  const float4 world_w = mul(g_global_uniform.i_vp_mtx, clip_pos);
  const float3 pos = world_w.xyz * rcp(world_w.w);

  const Light light = g_lights.data[0];
  const float3 light_2_surface = light.position - pos;
  const float light_distance_sq = dot(light_2_surface, light_2_surface);
  const float attenuation = rcp(light_distance_sq);
  const float3 light_dir = light_2_surface * rsqrt(light_distance_sq);

  const float intensity = max(dot(normal, light_dir) * attenuation, 0.0);

  OUT_COLOR = float4(light.intensity * intensity * albedo, 1.0);

  // End Function Code.
  //////////////////////////////////////////////////////////////////////////

#ifdef HALA_HLSL
  return output;
#endif
}