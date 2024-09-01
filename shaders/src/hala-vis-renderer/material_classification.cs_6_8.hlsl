#define HALA_NO_GLOBAL_PUSH_CONSTANT

#ifdef HALA_HLSL

  #include "scene.hlsl"
  #include "visibility.hlsl"

  [[vk::combinedImageSampler]]
  [[vk::binding(0, 3)]]
  Texture2D<uint> in_vis_image;
  [[vk::combinedImageSampler]]
  [[vk::binding(0, 3)]]
  SamplerState in_vis_sampler;

  [[vk::combinedImageSampler]]
  [[vk::binding(1, 3)]]
  Texture2D<float> in_depth_image;
  [[vk::combinedImageSampler]]
  [[vk::binding(1, 3)]]
  SamplerState in_depth_sampler;

  [[vk::binding(2, 3)]]
  RWByteAddressBuffer out_indirect_draw_arguments;

  [[vk::binding(3, 3)]]
  RWByteAddressBuffer out_tile_index;

  groupshared uint gs_material_flag[CLASSIFY_NUM_OF_MATERIALS_PER_GROUP];

  #define lessThan(x, y) (x < y)

#else

  #include "scene.glsl"
  #include "hala-vis-renderer\visibility.glsl"

  layout(set = 3, binding = 0) uniform usampler2D in_vis_image;
  layout(set = 3, binding = 1) uniform sampler2D in_depth_image;

  layout(set = 3, binding = 2) buffer IndirectDrawArgumentsBuffer {
    uint out_indirect_draw_arguments[];
  };

  layout(set = 3, binding = 3) buffer TileIndexBuffer {
    uint out_tile_index[];
  };

  shared uint gs_material_flag[CLASSIFY_NUM_OF_MATERIALS_PER_GROUP];

  #define g_global_meshlets (g_global_meshlets.data)
  #define g_draw_data (g_draw_data.data)

#endif

BEGIN_PUSH_CONSTANTS(MaterialClassifyPushConstants)
  uint2 screen_size;
  uint x_size;
  uint num_of_tiles;
END_PUSH_CONSTANTS(MaterialClassifyPushConstants, g_push_constants)

void classify_pixel(in uint2 pos) {
  if (all(lessThan(pos, g_push_constants.screen_size))) {
    const float depth = LOAD_SAMPLE(in_depth_image, pos, 0).x;

    ANNOTATION_BRANCH
    if (depth > 0.0) {
      const uint id = LOAD_SAMPLE(in_vis_image, pos, 0).x;
      uint meshlet_index, triangle_id;
      unpack_meshlet_triangle_index(id, meshlet_index, triangle_id);

      const Meshlet meshlet = g_global_meshlets[meshlet_index];
      const DrawData draw_data = g_draw_data[meshlet.draw_index];
      const uint material_index = draw_data.material_index;
      const uint index = draw_data.material_index / 32;
      const uint bit = draw_data.material_index % 32;
      uint orig;
      INTERLOCKED_OR(gs_material_flag[index], 0x1u << bit, orig);
    }
  }
}

#ifdef HALA_HLSL

  [numthreads(CLASSIFY_THREAD_WIDTH, CLASSIFY_THREAD_WIDTH, 1)]
  void main(
    uint3 group_id : SV_GroupID,
    uint3 group_thread_id : SV_GroupThreadID,
    uint3 dispatch_thread_id : SV_DispatchThreadID)
  {

#else

  layout(local_size_x = CLASSIFY_THREAD_WIDTH, local_size_y = CLASSIFY_THREAD_WIDTH, local_size_z = 1) in;
  void main() {
    #define group_id gl_WorkGroupID
    #define group_thread_id gl_LocalInvocationID
    #define dispatch_thread_id gl_GlobalInvocationID

#endif

  const uint mat_chunk_index = group_thread_id.y * CLASSIFY_THREAD_WIDTH + group_thread_id.x;
  gs_material_flag[mat_chunk_index] = 0x0;

  GroupMemoryBarrierWithGroupSync();

  // Classify materials.
  const uint2 base_pos = group_id.xy * CLASSIFY_TILE_WIDTH + group_thread_id.xy;
  ANNOTATION_UNROLL
  for (uint x = 0; x < 4; x++) {
    ANNOTATION_UNROLL
    for (uint y = 0; y < 4; y++) {
      classify_pixel(base_pos + uint2(x, y) * CLASSIFY_THREAD_WIDTH);
    }
  }

  GroupMemoryBarrierWithGroupSync();

  // Fill indirect draw arguments.
  uint bits = gs_material_flag[mat_chunk_index];
  if (bits != 0) {
    const uint mat_base_index = mat_chunk_index * 32;
    while (bits != 0) {
      const uint first_bit = firstbitlow(bits);
      const uint mat_index = mat_base_index + first_bit;
      bits &= ~(0x1u << first_bit);

      const uint arg_addr = mat_index * 16;
      uint store_addr = 0;
      INTERLOCKED_ADD_RWBUFFER(out_indirect_draw_arguments, arg_addr + 4, 1, store_addr);

      const uint tile_no = group_id.y * g_push_constants.x_size + group_id.x;
      store_addr = ((mat_index * g_push_constants.num_of_tiles) + store_addr) * 4;
      STORE_RWBUFFER(out_tile_index, store_addr, tile_no);
    }
  }
}