#include "defines.hlsl"
#include "visualization.hlsl"

[[vk::input_attachment_index(0)]]
[[vk::binding(0, 3)]]
SubpassInput<uint> in_input_image;

struct FragmentOutput {
  [[vk::location(0)]] float4 color: SV_Target0;
};

FragmentOutput main() {
  FragmentOutput output = (FragmentOutput)0;

  const uint id = in_input_image.SubpassLoad();
  output.color = float4(int_to_color(id), 1.0);

  return output;
}
