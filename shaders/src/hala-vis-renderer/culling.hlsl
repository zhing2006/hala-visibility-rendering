
// Convert the aabb to screen space.
//   true: the aabb is collide near plane.
//   false: the aabb is not collide near plane.
bool to_screen_aabb(in float4x4 vp_mtx, in float3 aabb_min_ws, in float3 aabb_max_ws, out float4 aabb, out float max_depth) {
  const float4 SX = mul(vp_mtx, float4(aabb_max_ws.x - aabb_min_ws.x, 0.0, 0.0, 0.0));
  const float4 SY = mul(vp_mtx, float4(0.0, aabb_max_ws.y - aabb_min_ws.y, 0.0, 0.0));
  const float4 SZ = mul(vp_mtx, float4(0.0, 0.0, aabb_max_ws.z - aabb_min_ws.z, 0.0));

  float4 P0 = mul(vp_mtx, float4(aabb_min_ws.x, aabb_min_ws.y, aabb_min_ws.z, 1.0));
  float4 P1 = P0 + SZ;
  float4 P2 = P0 + SY;
  float4 P3 = P2 + SZ;
  float4 P4 = P0 + SX;
  float4 P5 = P4 + SZ;
  float4 P6 = P4 + SY;
  float4 P7 = P6 + SZ;
  P0 /= P0.w;
  P1 /= P1.w;
  P2 /= P2.w;
  P3 /= P3.w;
  P4 /= P4.w;
  P5 /= P5.w;
  P6 /= P6.w;
  P7 /= P7.w;

  max_depth = max(max(max(max(max(max(max(P0.z, P1.z), P2.z), P3.z), P4.z), P5.z), P6.z), P7.z);

  if (max_depth >= 1.0) {
    return true;
  }

  aabb.xy = min(min(min(min(min(min(min(P0.xy, P1.xy), P2.xy), P3.xy), P4.xy), P5.xy), P6.xy), P7.xy);
  aabb.zw = max(max(max(max(max(max(max(P0.xy, P1.xy), P2.xy), P3.xy), P4.xy), P5.xy), P6.xy), P7.xy);
  aabb = aabb.xwzy * float4(0.5, -0.5, 0.5, -0.5) + 0.5;

  return false;

  // float4 points[8] = {
  //   float4(aabb_min_ws.x, aabb_min_ws.y, aabb_min_ws.z, 1.0),
  //   float4(aabb_min_ws.x, aabb_min_ws.y, aabb_max_ws.z, 1.0),
  //   float4(aabb_min_ws.x, aabb_max_ws.y, aabb_min_ws.z, 1.0),
  //   float4(aabb_min_ws.x, aabb_max_ws.y, aabb_max_ws.z, 1.0),
  //   float4(aabb_max_ws.x, aabb_min_ws.y, aabb_min_ws.z, 1.0),
  //   float4(aabb_max_ws.x, aabb_min_ws.y, aabb_max_ws.z, 1.0),
  //   float4(aabb_max_ws.x, aabb_max_ws.y, aabb_min_ws.z, 1.0),
  //   float4(aabb_max_ws.x, aabb_max_ws.y, aabb_max_ws.z, 1.0)
  // };

  // int point_index = 0;
  // ANNOTATION_UNROLL
  // for (point_index = 0; point_index < 8; point_index++) {
  //   points[point_index] = mul(vp_mtx, points[point_index]);
  //   points[point_index] /= points[point_index].w;
  //   points[point_index].xy = points[point_index].xy * float2(0.5, -0.5) + 0.5;
  // }

  // aabb.xy = aabb.zw = points[0].xy;
  // max_depth = points[0].z;
  // ANNOTATION_UNROLL
  // for (point_index = 1; point_index < 8; point_index++) {
  //   aabb.xy = min(aabb.xy, points[point_index].xy);
  //   aabb.zw = max(aabb.zw, points[point_index].xy);
  //   max_depth = max(max_depth, points[point_index].z);
  // }
  // if (max_depth >= 1.0) {
  //   return true;
  // }

  // return false;
}

// Check the aabb is occluded by hiz image.
//   true: the aabb is occluded.
//   false: the aabb is not occluded.
bool is_occluded(
#ifdef HALA_GLSL
  in texture2D hiz_image,
#else
  in Texture2D<float> hiz_image,
#endif
  in uint hiz_levels,
  in uint2 hiz_size,
  in float2 aabb_min,
  in float2 aabb_max,
  in float aabb_depth
) {
  const float4 rect = float4(aabb_min, aabb_max) * hiz_size.xyxy;
  const uint num_of_texels = uint(min(rect.z - rect.x, rect.w - rect.y)) + 1;
  const uint desired_mip = min(firstbithigh(num_of_texels), hiz_levels - 1);

  const float2 level_size = hiz_size / exp2(desired_mip);
  const float2 left_top = aabb_min * level_size;
  const float2 right_bottom = aabb_max * level_size;
  const uint start_x = max(uint(left_top.x), 0);
  const uint start_y = max(uint(left_top.y), 0);
  const uint end_x = min(uint(right_bottom.x) + (frac(right_bottom.x) > 0.0 ? 1 : 0), uint(level_size.x) - 1);
  const uint end_y = min(uint(right_bottom.y) + (frac(right_bottom.y) > 0.0 ? 1 : 0), uint(level_size.y) - 1);

  const float rect_z = aabb_depth;

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