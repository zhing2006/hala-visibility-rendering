#define HALA_NO_GLOBAL_PUSH_CONSTANT

#ifdef HALA_HLSL

  #include "scene.hlsl"

#else

  #include "scene.glsl"

#endif

TEXTURE2D(3, 0, in_depth);

#ifdef HALA_HLSL

  struct ToFragment {
    float4 position : SV_Position;
  };

  struct FragmentOutput {
    float depth : SV_Target0;
  };

  FragmentOutput main(ToFragment input) {
    FragmentOutput output;
    #define IN_POSITION input.position
    #define OUT_DEPTH output.depth

#else

  layout(location = 0) out float out_depth;

  void main() {
    #define IN_POSITION gl_FragCoord
    #define OUT_DEPTH out_depth

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.

  uint2 pos = uint2(IN_POSITION.xy);
  uint2 xy = pos * 2;
  float depth = LOAD_SAMPLE(in_depth, xy, 0).r;
  depth = min(depth, LOAD_SAMPLE(in_depth, xy + uint2(1, 0), 0).r);
  depth = min(depth, LOAD_SAMPLE(in_depth, xy + uint2(0, 1), 0).r);
  depth = min(depth, LOAD_SAMPLE(in_depth, xy + uint2(1, 1), 0).r);
  OUT_DEPTH = depth;

  // End Function Code.
  //////////////////////////////////////////////////////////////////////////

#ifdef HALA_HLSL
  return output;
#endif
}