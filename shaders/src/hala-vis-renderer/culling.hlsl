
// Convert the aabb to screen space.
//   true: the aabb is collide near plane.
//   false: the aabb is not collide near plane.
bool to_screen_aabb(in float4x4 mvp_mtx, in float3 aabb_min_ws, in float3 aabb_max_ws, out float3 aabb_min_screen, out float3 aabb_max_screen) {
  float4 points[8] = {
    float4(aabb_min_ws.x, aabb_min_ws.y, aabb_min_ws.z, 1.0),
    float4(aabb_min_ws.x, aabb_min_ws.y, aabb_max_ws.z, 1.0),
    float4(aabb_min_ws.x, aabb_max_ws.y, aabb_min_ws.z, 1.0),
    float4(aabb_min_ws.x, aabb_max_ws.y, aabb_max_ws.z, 1.0),
    float4(aabb_max_ws.x, aabb_min_ws.y, aabb_min_ws.z, 1.0),
    float4(aabb_max_ws.x, aabb_min_ws.y, aabb_max_ws.z, 1.0),
    float4(aabb_max_ws.x, aabb_max_ws.y, aabb_min_ws.z, 1.0),
    float4(aabb_max_ws.x, aabb_max_ws.y, aabb_max_ws.z, 1.0)
  };

  int point_index = 0;
  ANNOTATION_UNROLL
  for (point_index = 0; point_index < 8; point_index++) {
    points[point_index] = mul(mvp_mtx, points[point_index]);
    points[point_index] /= points[point_index].w;
    points[point_index].xy = points[point_index].xy * float2(0.5, -0.5) + 0.5;
  }

  aabb_min_screen = aabb_max_screen = points[0].xyz;
  float min_z, max_z;
  min_z = max_z = points[0].z;
  ANNOTATION_UNROLL
  for (point_index = 1; point_index < 8; point_index++) {
    aabb_min_screen = min(aabb_min_screen, points[point_index].xyz);
    aabb_max_screen = max(aabb_max_screen, points[point_index].xyz);
    min_z = min(min_z, points[point_index].z);
    max_z = max(max_z, points[point_index].z);
  }
  if (max_z >= 1.0) {
    return true;
  }

  aabb_min_screen = clamp(aabb_min_screen, float3(0, 0, 0), float3(1, 1, 1));
  aabb_max_screen = clamp(aabb_max_screen, float3(0, 0, 0), float3(1, 1, 1));
  return false;
}

// Check the aabb is occluded by hiz image.
//   true: the aabb is occluded.
//   false: the aabb is not occluded.
#ifdef HALA_GLSL
bool is_occluded(in texture2D hiz_image, in uint hiz_levels, in uint2 hiz_size, in float3 aabb_min, in float3 aabb_max)
#else
bool is_occluded(in Texture2D<float> hiz_image, in uint hiz_levels, in uint2 hiz_size, in float3 aabb_min, in float3 aabb_max)
#endif
{
  const float4 rect = float4(aabb_min.xy, aabb_max.xy) * hiz_size.xyxy;
  const uint num_of_texels = uint(min(rect.z - rect.x, rect.w - rect.y)) + 1;
  const uint desired_mip = min(firstbithigh(num_of_texels), hiz_levels - 1);

  const float2 level_size = hiz_size / exp2(desired_mip);
  const float2 left_top = aabb_min.xy * level_size;
  const float2 right_bottom = aabb_max.xy * level_size;
  const uint start_x = max(uint(left_top.x), 0);
  const uint start_y = max(uint(left_top.y), 0);
  const uint end_x = min(uint(right_bottom.x) + (frac(right_bottom.x) > 0.0 ? 1 : 0), uint(level_size.x) - 1);
  const uint end_y = min(uint(right_bottom.y) + (frac(right_bottom.y) > 0.0 ? 1 : 0), uint(level_size.y) - 1);

  const float rect_z = aabb_max.z;

  float texel_far_z = 1.0;
  for (uint y = start_y; y <= end_y; y++) {
    for (uint x = start_x; x <= end_x; x++) {
      const uint2 uv = uint2(x, y);
      const float z = LOAD_SAMPLE(hiz_image, uv, desired_mip).r;
      texel_far_z = min(texel_far_z, z);
    }
  }
  // printf("start: %d %d, end: %d %d, far_z: %f, rect_z: %f, texel_far_z: %f\n", start_x, start_y, end_x, end_y, texel_far_z, rect_z, texel_far_z);

  const float EPSILON = 1e-7;
  return texel_far_z < 1.0 && rect_z <= (texel_far_z - EPSILON);
}