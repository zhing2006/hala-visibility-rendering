#ifndef _COLOR_SPACE_HLSL_
#define _COLOR_SPACE_HLSL_

//////////////////////////////////////////////////////////////////////////
// Rec. 709 and sRGB color space conversions.
float3 linear_to_rec709(float3 lin) {
  lin = max(float3(6.10352e-5.xxx), lin); // minimum positive non-denormal (fixes black problem on DX11 AMD and NV)
  return min(lin * 4.5, pow(max(lin, 0.018.xxx), 0.45.xxx) * 1.099 - 0.099);
}

float3 rec709_to_linear(float3 color) {
  color = max(6.10352e-5.xxx, color); // minimum positive non-denormal (fixes black problem on DX11 AMD and NV)
#ifdef HALA_HLSL
  return select(color > 0.081, pow((color + 0.099) / 1.099, 1.0 / 0.45), color / 4.5);
#else
  return float3(
    color.r > 0.081 ? pow((color.r + 0.099) / 1.099, 1.0 / 0.45) : color.r / 4.5,
    color.g > 0.081 ? pow((color.g + 0.099) / 1.099, 1.0 / 0.45) : color.g / 4.5,
    color.b > 0.081 ? pow((color.b + 0.099) / 1.099, 1.0 / 0.45) : color.b / 4.5
  );
#endif
}

//////////////////////////////////////////////////////////////////////////
// sRGB and linear color space conversions.
float linear_to_srgb_ch(float lin) {
  if(lin < 0.00313067) return lin * 12.92;
  return pow(lin, 1.0 / 2.4) * 1.055 - 0.055;
}

float3 linear_to_srgb(float3 lin) {
  return float3(
    linear_to_srgb_ch(lin.r),
    linear_to_srgb_ch(lin.g),
    linear_to_srgb_ch(lin.b));
}

float3 srgb_to_linear(float3 color) {
  color = max(6.10352e-5.xxx, color); // minimum positive non-denormal (fixes black problem on DX11 AMD and NV)
#ifdef HALA_HLSL
  return select(color > 0.04045, pow(color * (1.0 / 1.055) + 0.0521327, 2.4), color * (1.0 / 12.92));
#else
  return float3(
    color.r > 0.04045 ? pow(color.r * (1.0 / 1.055) + 0.0521327, 2.4) : color.r * (1.0 / 12.92),
    color.g > 0.04045 ? pow(color.g * (1.0 / 1.055) + 0.0521327, 2.4) : color.g * (1.0 / 12.92),
    color.b > 0.04045 ? pow(color.b * (1.0 / 1.055) + 0.0521327, 2.4) : color.b * (1.0 / 12.92)
  );
#endif
}

//////////////////////////////////////////////////////////////////////////
// Dolby PQ transforms
// ST.2084 and linear color space conversions.
float3 st2084_to_linear(float3 pq) {
  const float m1 = 0.1593017578125; // = 2610. / 4096. * .25;
  const float m2 = 78.84375; // = 2523. / 4096. *  128;
  const float c1 = 0.8359375; // = 2392. / 4096. * 32 - 2413./4096.*32 + 1;
  const float c2 = 18.8515625; // = 2413. / 4096. * 32;
  const float c3 = 18.6875; // = 2392. / 4096. * 32;
  const float C = 10000.;

  float3 Np = pow(pq, 1.0.xxx / m2);
  float3 L = Np - c1;
  L = max(0.0.xxx, L);
  L = L / (c2 - c3 * Np);
  L = pow(L, 1.0.xxx / m1);
  float3 P = L * C;

  return P;
}

float3 linear_to_st2084(float3 lin) {
  const float m1 = 0.1593017578125; // = 2610. / 4096. * .25;
  const float m2 = 78.84375; // = 2523. / 4096. *  128;
  const float c1 = 0.8359375; // = 2392. / 4096. * 32 - 2413./4096.*32 + 1;
  const float c2 = 18.8515625; // = 2413. / 4096. * 32;
  const float c3 = 18.6875; // = 2392. / 4096. * 32;
  const float C = 10000.;

  float3 L = lin / C;
  float3 Lm = pow(L, m1.xxx);
  float3 N1 = (c1 + c2 * Lm);
  float3 N2 = (1.0 + c3 * Lm);
#ifdef HALA_HLSL
  float3 N = N1 * rcp(N2);
#else
  float3 N = N1 / N2;
#endif
  float3 P = pow(N, m2.xxx);

  return P;
}

//////////////////////////////////////////////////////////////////////////
// LMS and sRGB color space conversions.
BEGIN_CONST(float3x3, sRGB_2_LMS_MAT)
  17.8824, 43.5161, 4.1193,
   3.4557, 27.1554, 3.8671,
  0.02996, 0.18431, 1.4670
END_CONST()

BEGIN_CONST(float3x3, LMS_2_sRGB_MAT)
   0.0809, -0.1305,  0.1167,
  -0.0102,  0.0540, -0.1136,
  -0.0003, -0.0041,  0.6935
END_CONST()

float3 srgb_to_lms(float3 color) {
  return mul(sRGB_2_LMS_MAT, color);
}

float3 lms_to_srgb(float3 lms) {
  return mul(LMS_2_sRGB_MAT, lms);
}

//////////////////////////////////////////////////////////////////////////
// CIE XYZ and sRGB color space conversions.

BEGIN_CONST(float3x3, XYZ_2_sRGB_MAT)
   3.2409699419, -1.5373831776, -0.4986107603,
  -0.9692436363,  1.8759675015,  0.0415550574,
   0.0556300797, -0.2039769589,  1.0569715142
END_CONST()

BEGIN_CONST(float3x3, sRGB_2_XYZ_MAT)
  0.4123907993, 0.3575843394, 0.1804807884,
  0.2126390059, 0.7151686788, 0.0721923154,
  0.0193308187, 0.1191947798, 0.9505321522
END_CONST()

float3 srgb_to_xyz(float3 color) {
  return mul(sRGB_2_XYZ_MAT, color);
}

float3 xyz_to_srgb(float3 xyz) {
  return mul(XYZ_2_sRGB_MAT, xyz);
}

float luminance(float3 color, float3 factors) {
  return dot(color, factors);
}

// Luminance function for scene-referred linear colors.
float scene_luminance(float3 color) {
#ifdef HALA_HLSL
  return luminance(color, sRGB_2_XYZ_MAT._m10_m11_m12);
#else
  return luminance(color, sRGB_2_XYZ_MAT[1]);
#endif
}

//////////////////////////////////////////////////////////////////////////
// HSV and linear color space conversions.
float3 hue_to_rgb(float H) {
  float R = abs(H * 6 - 3) - 1;
  float G = 2 - abs(H * 6 - 2);
  float B = 2 - abs(H * 6 - 4);
  return saturate(float3(R, G, B));
}

float3 hsv_to_rgb(float3 HSV) {
  float3 RGB = hue_to_rgb(HSV.x);
  return ((RGB - 1) * HSV.y + 1) * HSV.z;
}

float3 rgb_to_hcv(float3 RGB) {
  // Based on work by Sam Hocevar and Emil Persson
  float4 P = (RGB.g < RGB.b)  ? float4(RGB.bg, -1.0f, 2.0f / 3.0f): float4(RGB.gb, 0.0f, -1.0f / 3.0f);
  float4 Q = (RGB.r < P.x)  ? float4(P.xyw, RGB.r)        : float4(RGB.r, P.yzx);
  float chroma = Q.x - min(Q.w, Q.y);
  float hue = abs((Q.w - Q.y) / (6.0f * chroma + 1e-10f) + Q.z);
  return float3(hue, chroma, Q.x);
}

float3 rgb_to_hsv(float3 RGB) {
  float3 HCV = rgb_to_hcv(RGB);
  float s = HCV.y / (HCV.z + 1e-10f);
  return float3(HCV.x, s, HCV.z);
}

#endif // _COLOR_SPACE_HLSL_