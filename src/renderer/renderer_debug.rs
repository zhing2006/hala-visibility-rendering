use hala_renderer::error::HalaRendererError;

use super::{
  DebugSettings,
  GlobalConstants,
  VisRenderer,
};

/// The implementation of the visibility renderer.
impl VisRenderer {

  /// Update the debug view.
  /// param debug_view_settings: The debug view settings.
  /// return: The result.
  pub(crate) fn update_debug_settings(&mut self, debug_view_settings: DebugSettings) -> anyhow::Result<(), HalaRendererError> {
    self.debug_settings = debug_view_settings;

    self.setup_debug()?;

    Ok(())
  }

  /// Print the culling flags to the log.
  /// return: The result.
  pub(crate) fn debug_culling_flags(&self) -> anyhow::Result<(), HalaRendererError> {
    let scene = self.scene_in_gpu.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the scene in the GPU.", None))?;
    let compute_command_buffers = &self.resources.compute_command_buffers;
    let transfer_staging_buffer = &self.resources.transfer_staging_buffer;
    let pre_culling_flags = self.pre_culling_flags.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the pre culling flags.", None))?;

    let mut culling_flags = vec![0u32; scene.meshlet_count as usize];
    pre_culling_flags.download_gpu_memory_with_buffer(
      &mut culling_flags,
      transfer_staging_buffer,
      compute_command_buffers,
    )?;
    for (index, flag) in culling_flags.iter().enumerate() {
      log::debug!("[{}] Culling Flag: {}", index, flag);
    }

    Ok(())
  }

  /// Print the culling results to the log.
  /// return: The result.
  pub(crate) fn debug_culling_results(&self) -> anyhow::Result<(), HalaRendererError> {
    let scene = self.scene_in_gpu.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the scene in the GPU.", None))?;
    let compute_command_buffers = &self.resources.compute_command_buffers;
    let transfer_staging_buffer = &self.resources.transfer_staging_buffer;
    let pre_culling_flags = self.pre_culling_flags.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the pre culling flags.", None))?;

    let mut culling_flags = vec![0u32; scene.meshlet_count as usize];
    pre_culling_flags.download_gpu_memory_with_buffer(
      &mut culling_flags,
      transfer_staging_buffer,
      compute_command_buffers,
    )?;
    let mut culled_count = 0;
    for flag in culling_flags.iter() {
      if *flag == 0 {
        culled_count += 1;
      }
    }
    log::debug!("Culled Result: {} / {}, Culling Rate: {:.2}%", culled_count, scene.meshlet_count, culled_count as f32 / scene.meshlet_count as f32 * 100.0);

    Ok(())
  }

  /// Print the indirect draw buffer to the log.
  /// return: The result.
  pub(crate) fn debug_indirect_draw(&self) -> anyhow::Result<(), HalaRendererError> {
    let compute_command_buffers = &self.resources.compute_command_buffers;
    let transfer_staging_buffer = &self.resources.transfer_staging_buffer;
    let indirect_draw_buffer = &self.indirect_draw_buffer;

    let mut indirect_draw = vec![hala_gfx::HalaIndirectDrawCommand::default(); GlobalConstants::CLASSIFY_DEPTH_RANGE as usize];
    indirect_draw_buffer.download_gpu_memory_with_buffer(
      &mut indirect_draw,
      transfer_staging_buffer,
      compute_command_buffers,
    )?;
    for (index, id_args) in indirect_draw.iter().enumerate() {
      if id_args.instance_count == 0 {
        continue;
      }
      log::debug!("[{}] Draw Arguments: {:?}", index, id_args);
    }

    Ok(())
  }

  /// Print the tile index buffer to the log.
  /// return: The result.
  pub(crate) fn debug_tile_index(&self) -> anyhow::Result<(), HalaRendererError> {
    let scene = self.scene_in_gpu.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the scene in the GPU.", None))?;
    let compute_command_buffers = &self.resources.compute_command_buffers;
    let transfer_staging_buffer = &self.resources.transfer_staging_buffer;
    let indirect_draw_buffer = &self.indirect_draw_buffer;
    let tile_index_buffer = &self.tile_index_buffer;

    let mut indirect_draw = vec![hala_gfx::HalaIndirectDrawCommand::default(); GlobalConstants::CLASSIFY_DEPTH_RANGE as usize];
    indirect_draw_buffer.download_gpu_memory_with_buffer(
      &mut indirect_draw,
      transfer_staging_buffer,
      compute_command_buffers,
    )?;

    let x = (self.info.width + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
    let y = (self.info.height + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
    let num_of_tiles = x * y;
    let mut tile_index = vec![0u32; num_of_tiles as usize * GlobalConstants::CLASSIFY_DEPTH_RANGE as usize];
    tile_index_buffer.download_gpu_memory_with_buffer(
      &mut tile_index,
      transfer_staging_buffer,
      compute_command_buffers,
    )?;

    let tile_index_chunks = tile_index.chunks_exact(num_of_tiles as usize);
    for (material_index, tile_index_chunk) in tile_index_chunks.enumerate() {
      if material_index >= scene.materials.len() {
        break;
      }

      let cmd = &indirect_draw[material_index];
      for (tile_index_index, tile) in tile_index_chunk.iter().enumerate() {
        if tile_index_index >= cmd.instance_count as usize {
          break;
        }
        log::debug!("[{}][{}] Tile Index: {}", material_index, tile_index_index, tile);
      }
    }

    Ok(())
  }

}