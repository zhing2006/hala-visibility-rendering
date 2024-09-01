//////////////////////////////////////////////////////////////////////////////
// Global definitions for the material classification pass.
#define CLASSIFY_TILE_WIDTH                 (64)
#define CLASSIFY_THREAD_WIDTH               (16)
#define CLASSIFY_NUM_OF_MATERIALS_PER_GROUP (256)
#define CLASSIFY_DEPTH_RANGE                (CLASSIFY_NUM_OF_MATERIALS_PER_GROUP * 32)

//////////////////////////////////////////////////////////////////////////////
// Barycentric derivatives definition.
struct BaryDeriv {
  float3 lambda;
  float3 ddx;
  float3 ddy;
};

// http://filmicworlds.com/blog/visibility-buffer-rendering-with-material-graphs/
BaryDeriv calc_full_bary(float4 pt0, float4 pt1, float4 pt2, float2 pixel_ndc, float2 win_size) {
#ifdef HALA_HLSL
  BaryDeriv ret = (BaryDeriv)0;
#else
  BaryDeriv ret;
  ret.lambda = float3(0, 0, 0);
  ret.ddx = float3(0, 0, 0);
  ret.ddy = float3(0, 0, 0);
#endif

  const float3 inv_w = rcp(float3(pt0.w, pt1.w, pt2.w));

  const float2 ndc0 = pt0.xy * inv_w.x;
  const float2 ndc1 = pt1.xy * inv_w.y;
  const float2 ndc2 = pt2.xy * inv_w.z;

  const float inv_det = rcp(determinant(float2x2(ndc2 - ndc1, ndc0 - ndc1)));
  ret.ddx = float3(ndc1.y - ndc2.y, ndc2.y - ndc0.y, ndc0.y - ndc1.y) * inv_det * inv_w;
  ret.ddy = float3(ndc2.x - ndc1.x, ndc0.x - ndc2.x, ndc1.x - ndc0.x) * inv_det * inv_w;
  float ddx_sum = dot(ret.ddx, float3(1, 1, 1));
  float ddy_sum = dot(ret.ddy, float3(1, 1, 1));

  const float2 delta_vec = pixel_ndc - ndc0;
  const float interp_inv_w = inv_w.x + delta_vec.x * ddx_sum + delta_vec.y * ddy_sum;
  const float interp_w = rcp(interp_inv_w);

  ret.lambda.x = interp_w * (inv_w[0] + delta_vec.x * ret.ddx.x + delta_vec.y * ret.ddy.x);
  ret.lambda.y = interp_w * (0.0      + delta_vec.x * ret.ddx.y + delta_vec.y * ret.ddy.y);
  ret.lambda.z = interp_w * (0.0      + delta_vec.x * ret.ddx.z + delta_vec.y * ret.ddy.z);

  ret.ddx *= (2.0 / win_size.x);
  ret.ddy *= (2.0 / win_size.y);
  ddx_sum *= (2.0 / win_size.x);
  ddy_sum *= (2.0 / win_size.y);

  ret.ddy *= -1.0;
  ddy_sum *= -1.0;

  const float interp_w_ddx = 1.0 / (interp_inv_w + ddx_sum);
  const float interp_w_ddy = 1.0 / (interp_inv_w + ddy_sum);

  ret.ddx = interp_w_ddx * (ret.lambda * interp_inv_w + ret.ddx) - ret.lambda;
  ret.ddy = interp_w_ddy * (ret.lambda * interp_inv_w + ret.ddy) - ret.lambda;

  return ret;
}

float3 interpolate_with_deriv(BaryDeriv deriv, float v0, float v1, float v2) {
  const float3 merged_v = float3(v0, v1, v2);
  float3 ret;
  ret.x = dot(merged_v, deriv.lambda);
  ret.y = dot(merged_v, deriv.ddx);
  ret.z = dot(merged_v, deriv.ddy);
  return ret;
}

void calc_deriv_float(BaryDeriv deriv, float v0, float v1, float v2, out float v, out float dx, out float dy) {
  const float3 x = interpolate_with_deriv(deriv, v0, v1, v2);
  v = x.x;
  dx = x.y;
  dy = x.z;
}

void calc_deriv_float2(BaryDeriv deriv, float2 v0, float2 v1, float2 v2, out float2 v, out float2 dx, out float2 dy) {
  const float3 x = interpolate_with_deriv(deriv, v0.x, v1.x, v2.x);
  const float3 y = interpolate_with_deriv(deriv, v0.y, v1.y, v2.y);
  v = float2(x.x, y.x);
  dx = float2(x.y, y.y);
  dy = float2(x.z, y.z);
}

void calc_deriv_float3(BaryDeriv deriv, float3 v0, float3 v1, float3 v2, out float3 v, out float3 dx, out float3 dy) {
  const float3 x = interpolate_with_deriv(deriv, v0.x, v1.x, v2.x);
  const float3 y = interpolate_with_deriv(deriv, v0.y, v1.y, v2.y);
  const float3 z = interpolate_with_deriv(deriv, v0.z, v1.z, v2.z);
  v = float3(x.x, y.x, z.x);
  dx = float3(x.y, y.y, z.y);
  dy = float3(x.z, y.z, z.z);
}

void calc_deriv_float4(BaryDeriv deriv, float4 v0, float4 v1, float4 v2, out float4 v, out float4 dx, out float4 dy) {
  const float3 x = interpolate_with_deriv(deriv, v0.x, v1.x, v2.x);
  const float3 y = interpolate_with_deriv(deriv, v0.y, v1.y, v2.y);
  const float3 z = interpolate_with_deriv(deriv, v0.z, v1.z, v2.z);
  const float3 w = interpolate_with_deriv(deriv, v0.w, v1.w, v2.w);
  v = float4(x.x, y.x, z.x, w.x);
  dx = float4(x.y, y.y, z.y, w.y);
  dy = float4(x.z, y.z, z.z, w.z);
}

//////////////////////////////////////////////////////////////////////////////
// Vertex attributes definition.
struct VertexAttributes {
  float3 position;
  float3 position_ddx;
  float3 position_ddy;
  float3 normal;
  float3 tangent;
  float2 texcoord;
  float2 texcoord_ddx;
  float2 texcoord_ddy;
};

VertexAttributes get_vertex_attributes(in float2 screen_size, in float2 pixel_pos, in DrawData draw_data, in uint3 tri, in Meshlet meshlet) {
#ifdef HALA_HLSL
  const ObjectUniform per_object_data = g_per_object_uniforms[draw_data.object_index];
  StructuredBuffer<Vertex> vertex_buffer = g_vertices[meshlet.draw_index];
  StructuredBuffer<uint> vertex_index_buffer = g_unique_vertices[meshlet.draw_index];
#else
  #define per_object_data (g_per_object_uniforms[draw_data.object_index])
  #define vertex_buffer (g_vertices[meshlet.draw_index].data)
  #define vertex_index_buffer (g_unique_vertices[meshlet.draw_index].data)
#endif

  const uint vertex_index0 = vertex_index_buffer[meshlet.offset_of_vertices + tri.x];
  const uint vertex_index1 = vertex_index_buffer[meshlet.offset_of_vertices + tri.y];
  const uint vertex_index2 = vertex_index_buffer[meshlet.offset_of_vertices + tri.z];
  const Vertex vertex0 = vertex_buffer[vertex_index0];
  const Vertex vertex1 = vertex_buffer[vertex_index1];
  const Vertex vertex2 = vertex_buffer[vertex_index2];

  const float4 vp0 = float4(vertex0.position_x, vertex0.position_y, vertex0.position_z, 1.0);
  const float4 vp1 = float4(vertex1.position_x, vertex1.position_y, vertex1.position_z, 1.0);
  const float4 vp2 = float4(vertex2.position_x, vertex2.position_y, vertex2.position_z, 1.0);
  const float3 p0 = mul(per_object_data.m_mtx, vp0).xyz;
  const float3 p1 = mul(per_object_data.m_mtx, vp1).xyz;
  const float3 p2 = mul(per_object_data.m_mtx, vp2).xyz;
  const float3 n0 = normalize(mul(float4(vertex0.normal_x, vertex0.normal_y, vertex0.normal_z, 0.0), per_object_data.i_m_mtx).xyz);
  const float3 n1 = normalize(mul(float4(vertex1.normal_x, vertex1.normal_y, vertex1.normal_z, 0.0), per_object_data.i_m_mtx).xyz);
  const float3 n2 = normalize(mul(float4(vertex2.normal_x, vertex2.normal_y, vertex2.normal_z, 0.0), per_object_data.i_m_mtx).xyz);
  const float3 t0 = normalize(mul(float4(vertex0.tangent_x, vertex0.tangent_y, vertex0.tangent_z, 0.0), per_object_data.i_m_mtx).xyz);
  const float3 t1 = normalize(mul(float4(vertex1.tangent_x, vertex1.tangent_y, vertex1.tangent_z, 0.0), per_object_data.i_m_mtx).xyz);
  const float3 t2 = normalize(mul(float4(vertex2.tangent_x, vertex2.tangent_y, vertex2.tangent_z, 0.0), per_object_data.i_m_mtx).xyz);

  const float4 pt0 = mul(per_object_data.mvp_mtx, vp0);
  const float4 pt1 = mul(per_object_data.mvp_mtx, vp1);
  const float4 pt2 = mul(per_object_data.mvp_mtx, vp2);
  const float2 screen_pos = (pixel_pos + 0.5) / screen_size;
  const float2 clip_pos = screen_pos * float2(2, -2) + float2(-1, 1);
  BaryDeriv C = calc_full_bary(pt0, pt1, pt2, clip_pos, screen_size);

  VertexAttributes vertex_attributes;
  calc_deriv_float2(
    C,
    float2(vertex0.tex_coord_x, vertex0.tex_coord_y),
    float2(vertex1.tex_coord_x, vertex1.tex_coord_y),
    float2(vertex2.tex_coord_x, vertex2.tex_coord_y),
    vertex_attributes.texcoord,
    vertex_attributes.texcoord_ddx,
    vertex_attributes.texcoord_ddy
  );

  calc_deriv_float3(
    C,
    p0, p1, p2,
    vertex_attributes.position,
    vertex_attributes.position_ddx,
    vertex_attributes.position_ddy
  );

  float3 dx3, dy3;
  calc_deriv_float3(
    C,
    n0, n1, n2,
    vertex_attributes.normal,
    dx3, dy3
  );

  calc_deriv_float3(
    C,
    t0, t1, t2,
    vertex_attributes.tangent,
    dx3, dy3
  );

  return vertex_attributes;
}

//////////////////////////////////////////////////////////////////////////////
// Load the primitive index from the unique primitive buffer.
#ifdef HALA_GLSL

uint3 load_primitive_index(uint index, uint draw_index) {
  const uint primitive_index = g_unique_primitives[draw_index].data[index];

#else

inline uint3 load_primitive_index(uint index, uint draw_index) {
  const uint primitive_index = g_unique_primitives[draw_index].Load(index * 4);

#endif
  const uint triangle_index0 = (primitive_index & 0xFF);
  const uint triangle_index1 = (primitive_index & 0xFF00) >> 8;
  const uint triangle_index2 = (primitive_index & 0xFF0000) >> 16;
  return uint3(triangle_index0, triangle_index1, triangle_index2);
}

//////////////////////////////////////////////////////////////////////////////
// Pack the meshlet index and triangle index into a single uint.
uint pack_meshlet_triangle_index(uint meshlet_index, uint triangle_index) {
  return ((meshlet_index & 0x1FFFFFF) << 7) | (triangle_index & 0x7F);
}

//////////////////////////////////////////////////////////////////////////////
// Unpack the meshlet index and triangle index from a single uint.
void unpack_meshlet_triangle_index(uint packed_index, out uint meshlet_index, out uint triangle_index) {
  meshlet_index = (packed_index & 0xFFFFFF80) >> 7;
  triangle_index = packed_index & 0x7F;
}