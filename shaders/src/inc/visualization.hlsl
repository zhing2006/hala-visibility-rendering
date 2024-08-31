#ifndef _VISUALIZATION_HLSL_
#define _VISUALIZATION_HLSL_

#ifdef HALA_HLSL
#include "hash.hlsl"
#include "color_mapping.hlsl"
#else
#include "hash.glsl"
#include "color_mapping.glsl"
#endif

float3 int_to_color(uint index) {
  uint Hash = hash_mix(index);

  float3 color = float3
  (
    (Hash >>  0) & 255,
    (Hash >>  8) & 255,
    (Hash >> 16) & 255
  );

  return color * (1.0f / 255.0f);
}

float3 float_to_color(float x) {
  return hsv_map_color(x);
}

float3 float_to_gr(float s) {
  return hue_to_rgb(lerp(0.333333f, 0.0f, saturate(s)));
}

float3 gr_to_turbo(float s) {
  return color_map_turbo(lerp(0.5f, 1.0f, saturate(s)));
}

#endif // _VISUALIZATION_HLSL_