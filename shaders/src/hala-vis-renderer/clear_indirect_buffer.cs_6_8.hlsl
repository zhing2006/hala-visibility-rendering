#define HALA_NO_GLOBAL_PUSH_CONSTANT

#ifdef HALA_HLSL

  #include "scene.hlsl"
  #include "visibility.hlsl"

#else

  #include "scene.glsl"
  #include "hala-vis-renderer\visibility.glsl"

#endif

BEGIN_PUSH_CONSTANTS(ClearIndirectBufferPushConstants)
  uint num_of_materials;
END_PUSH_CONSTANTS(ClearIndirectBufferPushConstants, g_push_constants)

BEGIN_RWBUFFER(3, 0, IndirectDrawArguments)
END_RWBUFFER(3, 0, IndirectDrawArguments, in_indirect_draw_arguments)

#ifdef HALA_HLSL

  [numthreads(32, 1, 1)]
  void main(
    uint3 group_id : SV_GroupID,
    uint3 group_thread_id : SV_GroupThreadID,
    uint3 dispatch_thread_id : SV_DispatchThreadID)
  {

#else

  layout(local_size_x = 32, local_size_y = 1, local_size_z = 1) in;

  void main() {
    const uint3 group_id = gl_WorkGroupID;
    const uint3 group_thread_id = gl_LocalInvocationID;
    const uint3 dispatch_thread_id = gl_GlobalInvocationID;

    #define in_indirect_draw_arguments (in_indirect_draw_arguments.data)

#endif

  if (dispatch_thread_id.x >= g_push_constants.num_of_materials) {
    return;
  }

#ifdef HALA_HLSL
  IndirectDrawArguments indirect_draw_arguments = (IndirectDrawArguments)0;
#else
  IndirectDrawArguments indirect_draw_arguments;
  indirect_draw_arguments.instance_count = 0;
  indirect_draw_arguments.first_vertex = 0;
  indirect_draw_arguments.first_instance = 0;
#endif
  indirect_draw_arguments.vertex_count = 4;
  in_indirect_draw_arguments[dispatch_thread_id.x] = indirect_draw_arguments;
}