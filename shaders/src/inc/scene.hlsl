#ifndef _SCENE_HLSL_
#define _SCENE_HLSL_

#ifdef HALA_HLSL
#include "types.hlsl"
#endif

#ifndef HALA_NO_GLOBAL_PUSH_CONSTANT
  BEGIN_PUSH_CONSTANTS(PushConstants)
  #ifdef USE_GLOBAL_MESHLETS
    #ifdef USE_MESH_SHADER
    uint meshlet_count;
    #else
    uint object_index;
    uint material_index;
    uint draw_index;
    #endif
  #else
    uint object_index;
    uint material_index;
    uint draw_index;
    #ifdef USE_MESH_SHADER
    uint meshlet_count;
    #endif
  #endif
  END_PUSH_CONSTANTS(PushConstants, g_push_constants)
#endif

BEGIN_UNIFORM_BUFFER(0, 0, GlobalUniform)
  float4x4 v_mtx;       // The view matrix.
  float4x4 p_mtx;       // The projection matrix.
  float4x4 vp_mtx;      // The view-projection matrix.
  float4x4 i_vp_mtx;    // The inverse view-projection matrix.

  float4 frustum_planes[6]; // The view frustum planes.
END_UNIFORM_BUFFER(0, 0, GlobalUniform, g_global_uniform)

BEGIN_UNIFORM_BUFFER(0, 1, CameraData)
  Camera data[MAX_CAMERAS];
END_UNIFORM_BUFFER(0, 1, CameraData, g_cameras)

BEGIN_UNIFORM_BUFFER(0, 2, LightBuffer)
  Light data[MAX_LIGHTS];
END_UNIFORM_BUFFER(0, 2, LightBuffer, g_lights)

BEGIN_BUFFER(0, 3, DrawData)
END_BUFFER(0, 3, DrawData, g_draw_data)

BEGIN_BUFFER(0, 4, Meshlet)
END_BUFFER(0, 4, Meshlet, g_global_meshlets)

BEGIN_UNIFORM_BUFFER_BINDLESS(1, 0, MaterialBuffer)
  Material data;
END_UNIFORM_BUFFER_BINDLESS(1, 0, MaterialBuffer, g_materials)

BEGIN_UNIFORM_BUFFER_BINDLESS(1, 1, ObjectUniform)
  float4x4 m_mtx;     // The model matrix
  float4x4 i_m_mtx;   // The inverse model matrix
  float4x4 mv_mtx;    // The model-view matrix
  float4x4 t_mv_mtx;  // The transposed model-view matrix
  float4x4 it_mv_mtx; // The inverse transposed model-view matrix
  float4x4 mvp_mtx;   // The model-view-projection matrix
END_UNIFORM_BUFFER_BINDLESS(1, 1, ObjectUniform, g_per_object_uniforms)

struct Vertex {
  float position_x;
  float position_y;
  float position_z;
  float normal_x;
  float normal_y;
  float normal_z;
  float tangent_x;
  float tangent_y;
  float tangent_z;
  float tex_coord_x;
  float tex_coord_y;
};

BEGIN_BUFFER_BINDLESS(1, 2, Vertex)
END_BUFFER_BINDLESS(1, 2, Vertex, g_vertices)

BEGIN_BUFFER_BINDLESS(1, 3, uint)
END_BUFFER_BINDLESS(1, 3, uint, g_indices)

BEGIN_BUFFER_BINDLESS(1, 4, Meshlet)
END_BUFFER_BINDLESS(1, 4, Meshlet, g_meshlets)

BEGIN_BUFFER_BINDLESS(1, 5, uint)
END_BUFFER_BINDLESS(1, 5, uint, g_unique_vertices)

#ifdef HALA_HLSL
[[vk::binding(6, 1)]]
ByteAddressBuffer g_unique_primitives[];
#else
BEGIN_BUFFER_BINDLESS(1, 6, uint)
END_BUFFER_BINDLESS(1, 6, uint, g_unique_primitives)
#endif

TEXTURE2D_BINDLESS(2, 0, g_textures)

SAMPLER_BINDLESS(2, 1, g_samplers)

#ifdef USE_MESH_SHADER
struct MeshShaderPayLoad {
  uint meshlet_indices[TASK_SHADER_GROUP_SIZE];
};
#endif

bool is_sphere_frustum_culled(const float3 center, const float radius) {
  for (uint i = 0; i < 6; ++i) {
    const float distance = dot(g_global_uniform.frustum_planes[i], float4(center, 1.0));
    if (distance <= -radius) {
      return true;
    }
  }
  return false;
}

bool is_box_frustum_culled(const float3 aabb_min_ws, const float3 aabb_max_ws) {
  const float4 points[8] = {
    float4(aabb_min_ws.x, aabb_min_ws.y, aabb_min_ws.z, 1.0),
    float4(aabb_min_ws.x, aabb_min_ws.y, aabb_max_ws.z, 1.0),
    float4(aabb_min_ws.x, aabb_max_ws.y, aabb_min_ws.z, 1.0),
    float4(aabb_min_ws.x, aabb_max_ws.y, aabb_max_ws.z, 1.0),
    float4(aabb_max_ws.x, aabb_min_ws.y, aabb_min_ws.z, 1.0),
    float4(aabb_max_ws.x, aabb_min_ws.y, aabb_max_ws.z, 1.0),
    float4(aabb_max_ws.x, aabb_max_ws.y, aabb_min_ws.z, 1.0),
    float4(aabb_max_ws.x, aabb_max_ws.y, aabb_max_ws.z, 1.0)
  };

  for (uint i = 0; i < 6; ++i) {
    uint out_count = 0;
    ANNOTATION_UNROLL
    for (uint j = 0; j < 8; ++j) {
      const float distance = dot(g_global_uniform.frustum_planes[i], points[j]);
      if (distance < 0.0) {
        out_count++;
      }
    }
    if (out_count == 8) {
      return true;
    }
  }
  return false;
}

#endif // _SCENE_HLSL_