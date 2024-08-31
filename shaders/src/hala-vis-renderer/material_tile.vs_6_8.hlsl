#define HALA_NO_GLOBAL_PUSH_CONSTANT

#ifdef HALA_HLSL

  #include "scene.hlsl"
  #include "visibility.hlsl"
  #include "material_tile.hlsl"

  [[vk::binding(0, 3)]]
  ByteAddressBuffer in_tile_index;

  ToFragment main(uint instance_id : SV_InstanceID, uint vertex_id : SV_VertexID) {
    ToFragment output = (ToFragment)0;

    #define OUT_POSITION output.position

#else

  #include "scene.glsl"
  #include "hala-vis-renderer\visibility.glsl"
  #include "hala-vis-renderer\material_tile.hlsl"

  layout(set = 3, binding = 0) buffer TileIndexBuffer {
    uint in_tile_index[];
  };

  void main() {
    #define instance_id gl_InstanceIndex
    #define vertex_id gl_VertexIndex
    #define OUT_POSITION gl_Position

#endif

  //////////////////////////////////////////////////////////////////////////
  // Begin Function Code.

  const uint addr = g_push_constants.material_index * g_push_constants.num_of_tiles + instance_id;
  const uint tile_index = LOAD_BUFFER(in_tile_index, addr * 4);
  const uint offset_x = (((vertex_id << 1) & 2) >> 1);
  const uint offset_y = ((vertex_id & 2) >> 1);
  const uint tile_x = (tile_index % g_push_constants.tile_size_x) + offset_x;
  const uint tile_y = (tile_index / g_push_constants.tile_size_x) + offset_y;

  float2 pos = float2(tile_x * CLASSIFY_TILE_WIDTH, tile_y * CLASSIFY_TILE_WIDTH);
  ANNOTATION_BRANCH
  if (g_push_constants.grid_line_width > 0) {
    pos -= float2(offset_x == 1 ? float(g_push_constants.grid_line_width) : 0.0, offset_y == 1 ? float(g_push_constants.grid_line_width) : 0.0);
  }
  const float2 uv = pos / float2(g_push_constants.screen_size);
  const float depth = float(g_push_constants.material_index) / float(CLASSIFY_DEPTH_RANGE);
  OUT_POSITION = float4(uv * float2(2.0, -2.0) - float2(1.0, -1.0), depth, 1.0);

  // End Function Code.
  //////////////////////////////////////////////////////////////////////////

#ifdef HALA_HLSL
  return output;
#endif
}
