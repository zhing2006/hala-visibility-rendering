#define USE_MESH_SHADER

#define MAX_VERTEX_COUNT 64
#define MAX_TRIANGLE_COUNT 124
#define VERTICES_PER_THREAD DIV_UP(MAX_VERTEX_COUNT, MESH_SHADER_GROUP_SIZE)
#define TRIANGLE_PER_THREAD DIV_UP(MAX_TRIANGLE_COUNT, MESH_SHADER_GROUP_SIZE)

#ifdef HALA_GLSL

#include "scene.glsl"
#include "hala-vis-renderer/visibility.glsl"

layout(triangles) out;
layout(local_size_x = MESH_SHADER_GROUP_SIZE, local_size_y = 1, local_size_z = 1) in;
layout(max_vertices = MAX_VERTEX_COUNT, max_primitives = MAX_TRIANGLE_COUNT) out;

taskPayloadSharedEXT MeshShaderPayLoad ms_payload;

void main() {
  #define triangles gl_PrimitiveTriangleIndicesEXT

  const uvec3 group_id = gl_WorkGroupID;
  const uvec3 group_thread_id = gl_LocalInvocationID;

  #define OUT_POSITION(index) gl_MeshVerticesEXT[index].gl_Position

  #define OUT_PRIMITIVE_ID(index) gl_MeshPrimitivesEXT[index].gl_PrimitiveID

  #define g_global_meshlets (g_global_meshlets.data)
  #define g_draw_data (g_draw_data.data)
#else

#include "scene.hlsl"
#include "visibility.hlsl"

struct ToFragment {
  float4 position: SV_Position;
};

struct ToFragmentPrimitive {
  uint primitive_id: SV_PrimitiveID;
};

[outputtopology("triangle")]
[numthreads(MESH_SHADER_GROUP_SIZE, 1, 1)]
void main(
  out indices uint3 triangles[MAX_TRIANGLE_COUNT],
  out primitives ToFragmentPrimitive primitives[MAX_TRIANGLE_COUNT],
  out vertices ToFragment vertices[MAX_VERTEX_COUNT],
  in payload MeshShaderPayLoad ms_payload,
  uint3 group_id : SV_GroupID,
  uint3 group_thread_id : SV_GroupThreadID
) {
  #define OUT_POSITION(index) vertices[index].position

  #define OUT_PRIMITIVE_ID(index) primitives[index].primitive_id

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.
  const uint meshlet_index = ms_payload.meshlet_indices[group_id.x];

  const Meshlet meshlet = g_global_meshlets[meshlet_index];
  const DrawData draw_data = g_draw_data[meshlet.draw_index];
#ifdef HALA_GLSL
  #define per_object_data (g_per_object_uniforms[draw_data.object_index])
  #define vertex_buffer (g_vertices[meshlet.draw_index].data)
  #define vertex_index_buffer (g_unique_vertices[meshlet.draw_index].data)
#else
  const ObjectUniform per_object_data = g_per_object_uniforms[draw_data.object_index];
  StructuredBuffer<Vertex> vertex_buffer = g_vertices[meshlet.draw_index];
  StructuredBuffer<uint> vertex_index_buffer = g_unique_vertices[meshlet.draw_index];
#endif

  SetMeshOutputCounts(meshlet.num_of_vertices, meshlet.num_of_primitives);

  // Per thread write one vertex.
  const uint vertex_id = group_thread_id.x;
  if (vertex_id < min(meshlet.num_of_vertices, MAX_VERTEX_COUNT)) {
    const uint vertex_index = vertex_index_buffer[meshlet.offset_of_vertices + vertex_id];
    const Vertex vertex = vertex_buffer[vertex_index];
    const float3 position = float3(vertex.position_x, vertex.position_y, vertex.position_z);
    const float4 h_position = mul(per_object_data.mvp_mtx, float4(position, 1.0));

    OUT_POSITION(vertex_id) = h_position;
  }

  // Per thread write two triangles.
  uint triangle_id = group_thread_id.x * 2;
  uint triangle_index = meshlet.offset_of_primitives + triangle_id;
  if (triangle_id < min(meshlet.num_of_primitives, MAX_TRIANGLE_COUNT)) {
    triangles[triangle_id] = load_primitive_index(triangle_index, meshlet.draw_index);
    uint primitive_id = pack_meshlet_triangle_index(meshlet_index, triangle_id);
    OUT_PRIMITIVE_ID(triangle_id) = int(primitive_id);

    triangle_id++;
    triangle_index = meshlet.offset_of_primitives + triangle_id;
    if (triangle_id < min(meshlet.num_of_primitives, MAX_TRIANGLE_COUNT)) {
      triangles[triangle_id] = load_primitive_index(triangle_index, meshlet.draw_index);
      primitive_id = pack_meshlet_triangle_index(meshlet_index, triangle_id);
      OUT_PRIMITIVE_ID(triangle_id) = int(primitive_id);
    }
  }
  // End Function Code.
  //////////////////////////////////////////////////////////////////////////
}