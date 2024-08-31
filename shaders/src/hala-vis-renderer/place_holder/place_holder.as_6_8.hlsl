#define USE_MESH_SHADER

#ifdef HALA_GLSL

  #include "scene.glsl"

  layout(local_size_x = TASK_SHADER_GROUP_SIZE, local_size_y = 1, local_size_z = 1) in;

  taskPayloadSharedEXT MeshShaderPayLoad ms_payload;

  void main() {
    uvec3 group_id = gl_WorkGroupID;
    uvec3 group_thread_id = gl_LocalInvocationID;
    uvec3 dispatch_thread_id = gl_GlobalInvocationID;

#else

  #include "scene.hlsl"

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

  bool is_visible = true;

  const float3 cone_apex = mul(per_object_data.m_mtx, float4(meshlet.cone_apex, 1.0)).xyz;
  const float3 cone_axis = normalize(mul(float4(meshlet.cone_axis, 0.0), per_object_data.i_m_mtx).xyz);
  if (dot(normalize(cone_apex - camera_position), cone_axis) >= meshlet.cone_cutoff)
    is_visible = false;

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
