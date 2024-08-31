BEGIN_PUSH_CONSTANTS(MaterialTilePushConstants)
  uint2 screen_size;
  uint tile_size_x;
  uint num_of_tiles;
  uint material_index;
  uint grid_line_width;
END_PUSH_CONSTANTS(MaterialTilePushConstants, g_push_constants)

#ifdef HALA_HLSL

struct ToFragment {
  [[vk::location(0)]] float4 position : SV_Position;
};

#endif