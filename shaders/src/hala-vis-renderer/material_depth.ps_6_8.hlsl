#include "scene.hlsl"
#include "visibility.hlsl"

[[vk::input_attachment_index(0)]]
[[vk::binding(0, 3)]]
SubpassInput<uint> in_vis_image;

[[vk::input_attachment_index(1)]]
[[vk::binding(1, 3)]]
SubpassInput<float> in_depth_image;

struct FragmentOutput {
  [[vk::location(0)]] float color: SV_Target0;
  [[vk::location(1)]] float depth: SV_Depth;
};

FragmentOutput main() {
  FragmentOutput output = (FragmentOutput)0;

  const float depth = in_depth_image.SubpassLoad();

  [branch]
  if (depth <= 0.0) {
    output.depth = 1.0;
  } else {
    const uint id = in_vis_image.SubpassLoad();
    uint meshlet_index, triangle_id;
    unpack_meshlet_triangle_index(id, meshlet_index, triangle_id);

    const Meshlet meshlet = g_global_meshlets[meshlet_index];
    const DrawData draw_data = g_draw_data[meshlet.draw_index];
    output.depth = (float)draw_data.material_index / (float)CLASSIFY_DEPTH_RANGE;
  }
  output.color = output.depth;

  return output;
}
