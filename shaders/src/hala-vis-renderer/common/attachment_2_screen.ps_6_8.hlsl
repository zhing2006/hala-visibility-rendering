#include "defines.hlsl"

struct AttachmentToScreenPushConstants {
  float4 scale;
};

[[vk::push_constant]]
AttachmentToScreenPushConstants g_push_constants;

[[vk::input_attachment_index(0)]]
[[vk::binding(0, 3)]]
SubpassInput in_input_image;

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main() {
  FragmentOutput output = (FragmentOutput)0;

  output.color = in_input_image.SubpassLoad() * g_push_constants.scale;

  return output;
}
