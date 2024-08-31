#define USE_MESH_SHADER

#define MAX_VERTEX_COUNT 64
#define MAX_TRIANGLE_COUNT 124
#define VERTICES_PER_THREAD DIV_UP(MAX_VERTEX_COUNT, MESH_SHADER_GROUP_SIZE)
#define TRIANGLE_PER_THREAD DIV_UP(MAX_TRIANGLE_COUNT, MESH_SHADER_GROUP_SIZE)

#ifdef HALA_GLSL

#include "scene.glsl"
#include "visualization.glsl"

#else

#include "scene.hlsl"
#include "visualization.hlsl"

#endif

//////////////////////////////////////////////////////////////////////////////
// Load the primitive index from the primitive index buffer.
#ifdef HALA_GLSL

#define primitive_index_buffer(draw_index) (g_unique_primitives[draw_index].data)
uvec3 load_primitive_index(uint index, uint draw_index) {
  const uint primitive_index = primitive_index_buffer(draw_index)[index];

#else

#define primitive_index_buffer(draw_index) (g_unique_primitives[draw_index])
inline uint3 load_primitive_index(uint index, uint draw_index) {
  const uint primitive_index = primitive_index_buffer(draw_index).Load(index * 4);

#endif
  const uint triangle_index0 = (primitive_index & 0xFF);
  const uint triangle_index1 = (primitive_index & 0xFF00) >> 8;
  const uint triangle_index2 = (primitive_index & 0xFF0000) >> 16;
  return uint3(triangle_index0, triangle_index1, triangle_index2);
}

#ifdef HALA_GLSL

layout(triangles) out;
layout(local_size_x = MESH_SHADER_GROUP_SIZE, local_size_y = 1, local_size_z = 1) in;
layout(max_vertices = MAX_VERTEX_COUNT, max_primitives = MAX_TRIANGLE_COUNT) out;

taskPayloadSharedEXT MeshShaderPayLoad ms_payload;
layout(location=0) out flat float3 out_color[];
layout(location=1) out flat uint out_material_index[];

void main() {
  #define triangles gl_PrimitiveTriangleIndicesEXT

  const uvec3 group_id = gl_WorkGroupID;
  const uvec3 group_thread_id = gl_LocalInvocationID;

  #define OUT_POSITION(index) gl_MeshVerticesEXT[index].gl_Position
  #define OUT_COLOR(index) out_color[index]
  #define OUT_MATERIAL_INDEX(index) out_material_index[index]

#else

struct ToFragment {
  float4 position: SV_Position;
  [[vk::location(0)]] nointerpolation float3 color : COLOR0;
  [[vk::location(1)]] nointerpolation uint material_index: MATERIAL_INDEX;
};

[outputtopology("triangle")]
[numthreads(MESH_SHADER_GROUP_SIZE, 1, 1)]
void main(
  out indices uint3 triangles[MAX_TRIANGLE_COUNT],
  out vertices ToFragment vertices[MAX_VERTEX_COUNT],
  in payload MeshShaderPayLoad ms_payload,
  uint3 group_id : SV_GroupID,
  uint3 group_thread_id : SV_GroupThreadID
) {
  #define OUT_POSITION(index) vertices[index].position
  #define OUT_COLOR(index) vertices[index].color
  #define OUT_MATERIAL_INDEX(index) vertices[index].material_index

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.
  const uint meshlet_index = ms_payload.meshlet_indices[group_id.x];
  // printf("[MESH SHADER] meshlet_index: %d\n", meshlet_index);
  // printf("[MESH SHADER] VERTEX_PER_THREAD: %d TRIANGLE_PER_THREAD: %d\n", VERTICES_PER_THREAD, TRIANGLE_PER_THREAD);
  // printf("[MESH SHADER] group_thread_id: %d\n", group_thread_id.x);

#ifdef HALA_GLSL
  const Meshlet meshlet = g_global_meshlets.data[meshlet_index];
  const DrawData draw_data = g_draw_data.data[meshlet.draw_index];
  #define per_object_data (g_per_object_uniforms[draw_data.object_index])
  #define vertex_buffer (g_vertices[meshlet.draw_index].data)
  #define vertex_index_buffer (g_unique_vertices[meshlet.draw_index].data)
#else
  const Meshlet meshlet = g_global_meshlets[meshlet_index];
  const DrawData draw_data = g_draw_data[meshlet.draw_index];
  const ObjectUniform per_object_data = g_per_object_uniforms[draw_data.object_index];
  StructuredBuffer<Vertex> vertex_buffer = g_vertices[meshlet.draw_index];
  StructuredBuffer<uint> vertex_index_buffer = g_unique_vertices[meshlet.draw_index];
#endif
  const float3 camera_position = g_cameras.data[0].position;

  SetMeshOutputCounts(meshlet.num_of_vertices, meshlet.num_of_primitives);

  // Per thread write one vertex.
  const uint vertex_id = group_thread_id.x;
  if (vertex_id < min(meshlet.num_of_vertices, MAX_VERTEX_COUNT)) {
    const uint vertex_index = vertex_index_buffer[meshlet.offset_of_vertices + vertex_id];
    const Vertex vertex = vertex_buffer[vertex_index];
    const float3 position = float3(vertex.position_x, vertex.position_y, vertex.position_z);
    const float4 h_position = mul(per_object_data.mvp_mtx, float4(position, 1.0));
    const float3 color = int_to_color(draw_data.object_index * 1000 + meshlet_index + 1);
    const float3 view_ws = normalize(camera_position - position);
    const float3 normal_ws = normalize(mul(per_object_data.m_mtx, float4(vertex.normal_x, vertex.normal_y, vertex.normal_z, 0.0)).xyz);

    OUT_POSITION(vertex_id) = h_position;
    OUT_COLOR(vertex_id) = color * dot(view_ws, normal_ws);
    OUT_MATERIAL_INDEX(vertex_id) = draw_data.material_index;
  }

  // Per thread write two triangles.
  uint triangle_id = group_thread_id.x * 2;
  if (triangle_id < min(meshlet.num_of_primitives, MAX_TRIANGLE_COUNT)) {
    triangles[triangle_id] = load_primitive_index(meshlet.offset_of_primitives + triangle_id, meshlet.draw_index);
    triangle_id++;
    if (triangle_id < min(meshlet.num_of_primitives, MAX_TRIANGLE_COUNT)) {
      triangles[triangle_id] = load_primitive_index(meshlet.offset_of_primitives + triangle_id, meshlet.draw_index);
    }
  }
  // End Function Code.
  //////////////////////////////////////////////////////////////////////////
}