#define USE_MESH_SHADER
#define HALA_NO_GLOBAL_PUSH_CONSTANT

#ifdef HALA_GLSL

  #include "scene.glsl"
  #include "hala-vis-renderer\culling.glsl"

  layout(set = 3, binding = 0) uniform texture2D in_hiz_image;
  layout(set = 3, binding = 1) buffer PreCullingFlags {
    uint in_culling_flags[];
  };

#else

  #include "scene.hlsl"
  #include "culling.hlsl"

  [[vk::binding(0, 3)]] Texture2D<float> in_hiz_image;
  [[vk::binding(1, 3)]] ByteAddressBuffer in_culling_flags;

#endif

BEGIN_PUSH_CONSTANTS(PushConstants)
  uint meshlet_count;
  uint hiz_levels;
  uint2 hiz_size;
END_PUSH_CONSTANTS(PushConstants, g_push_constants)

#ifdef HALA_GLSL

  layout(local_size_x = TASK_SHADER_GROUP_SIZE, local_size_y = 1, local_size_z = 1) in;

  taskPayloadSharedEXT MeshShaderPayLoad ms_payload;

  void main() {
    uvec3 group_id = gl_WorkGroupID;
    uvec3 group_thread_id = gl_LocalInvocationID;
    uvec3 dispatch_thread_id = gl_GlobalInvocationID;

#else

  groupshared MeshShaderPayLoad ms_payload;

  [numthreads(TASK_SHADER_GROUP_SIZE, 1, 1)]
  void main(
    uint3 group_id : SV_GroupID,
    uint3 group_thread_id : SV_GroupThreadID,
    uint3 dispatch_thread_id : SV_DispatchThreadID
  ) {

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.
  const uint meshlet_index = dispatch_thread_id.x;
  if (meshlet_index >= g_push_constants.meshlet_count) {
    return;
  }

#ifdef HALA_GLSL
  const Meshlet meshlet = g_global_meshlets.data[meshlet_index];
  const DrawData draw_data = g_draw_data.data[meshlet.draw_index];
  #define per_object_data (g_per_object_uniforms[draw_data.object_index])
#else
  const Meshlet meshlet = g_global_meshlets[meshlet_index];
  const DrawData draw_data = g_draw_data[meshlet.draw_index];
  const ObjectUniform per_object_data = g_per_object_uniforms[draw_data.object_index];
#endif
  const float3 camera_position = g_cameras.data[0].position;

  // printf("[TASK SHADER] Draw Index: %d\n", meshlet.draw_index);
  // printf("[TASK SHADER] Material Index: %d\n", draw_data.material_index);

  bool is_visible = false;

  const uint culling_flag = LOAD_BUFFER(in_culling_flags, meshlet_index * 4);
  if (culling_flag == 0) {
    const float3 bound_box_min = mul(per_object_data.m_mtx, float4(meshlet.bound_sphere.xyz - meshlet.bound_sphere.w, 1.0)).xyz;
    const float3 bound_box_max = mul(per_object_data.m_mtx, float4(meshlet.bound_sphere.xyz + meshlet.bound_sphere.w, 1.0)).xyz;

    float3 aabb_min_screen, aabb_max_screen;
    if (!to_screen_aabb(g_global_uniform.vp_mtx, bound_box_min, bound_box_max, aabb_min_screen, aabb_max_screen)) {
      if (is_occluded(in_hiz_image, g_push_constants.hiz_levels, g_push_constants.hiz_size, aabb_min_screen, aabb_max_screen)) {
        is_visible = false;
        // printf("[TASK SHADER] Draw Index %d Meshlet %d is culled by occlusion test.\n", meshlet.draw_index, meshlet_index);
      } else {
        is_visible = true;
        // printf("[TASK SHADER] Draw Index %d Meshlet %d is visible.\n", meshlet.draw_index, meshlet_index);
      }
    }
  }

  if (is_visible) {
    const uint index = WavePrefixCountBits(is_visible);
    ms_payload.meshlet_indices[index] = meshlet_index;
  }

  // One meshlet to one mesh group.
  const uint visible_count = WaveActiveCountBits(is_visible);
  DISPATCH_MESH(visible_count, 1, 1, ms_payload);
  // End Function Code.
  //////////////////////////////////////////////////////////////////////////
}
