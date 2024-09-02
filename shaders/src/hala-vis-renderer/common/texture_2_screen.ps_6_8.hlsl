#include "defines.hlsl"

[[vk::combinedImageSampler]]
[[vk::binding(0, 3)]]
Texture2D<float4> in_image;
[[vk::combinedImageSampler]]
[[vk::binding(0, 3)]]
SamplerState in_sampler;

struct ToFragment {
  [[vk::location(0)]] float2 uv: TEXCOORD0;
};

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main(in ToFragment input) {
  FragmentOutput output = (FragmentOutput)0;

  output.color = in_image.Sample(in_sampler, input.uv);

  return output;
}
