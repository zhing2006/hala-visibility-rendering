#define HALA_NO_GLOBAL_PUSH_CONSTANT

#ifdef HALA_HLSL

  #include "scene.hlsl"

  struct HiZPushContants {
    float scale;
  };

  [[vk::push_constant]]
  HiZPushContants g_push_constants;

  [[vk::binding(0, 3)]]
  Texture2D<float> in_depth_image;

  [[vk::binding(1, 3)]]
  SamplerState in_depth_sampler;

  struct ToFragment {
    [[vk::location(0)]] float2 uv : TEXCOORD0;
  };

  struct FragmentOutput {
    float4 color : SV_Target0;
  };

  FragmentOutput main(ToFragment input) {
    FragmentOutput output;
    #define IN_UV input.uv
    #define OUT_COLOR output.color

#else

  #include "scene.glsl"

  layout(push_constant) uniform HiZPushContants {
    float scale;
  } g_push_constants;

  layout(set = 3, binding = 0) uniform texture2D in_depth_image;
  layout(set = 3, binding = 1) uniform sampler in_depth_sampler;

  layout(location = 0) in float2 in_uv;
  layout(location = 0) out float4 out_color;

  void main() {
    #define IN_UV in_uv
    #define OUT_COLOR out_color

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.

  const float2 uv = float2(IN_UV.x, 1.0 - IN_UV.y);
  OUT_COLOR = float4(SAMPLE_TEXTURE(in_depth_image, in_depth_sampler, uv).rrr * g_push_constants.scale, 1.0);

  // End Function Code.
  //////////////////////////////////////////////////////////////////////////

#ifdef HALA_HLSL
  return output;
#endif
}