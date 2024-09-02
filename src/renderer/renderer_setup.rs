use hala_renderer::error::HalaRendererError;

use crate::renderer::VisRenderer;

/// The debug implementation of the visibility renderer.
impl VisRenderer {

  /// Setup the debug resources.
  /// return: The result.
  pub(crate) fn setup_debug(&mut self) -> Result<(), HalaRendererError> {
    let attachment_to_screen_descriptor_set = self.graphics_descriptor_sets.get("attachment_to_screen")
      .ok_or(HalaRendererError::new("Failed to find the attachment to screen descriptor set.", None))?;
    if self.debug_settings.show_albedo {
      attachment_to_screen_descriptor_set.update_input_attachments(
        0,
        0,
        &[self.albedo_image.as_ref()],
      );
    } else if self.debug_settings.show_normal {
      attachment_to_screen_descriptor_set.update_input_attachments(
        0,
        0,
        &[self.normal_image.as_ref()],
      );
    } else {
      attachment_to_screen_descriptor_set.update_input_attachments(
        0,
        0,
        &[self.material_depth_image.as_ref()],
      );
    }

    let id_buffer_visualization_descriptor_set = self.graphics_descriptor_sets.get("id_buffer_visualization")
      .ok_or(HalaRendererError::new("Failed to find the id buffer visualization descriptor set.", None))?;
    id_buffer_visualization_descriptor_set.update_input_attachments(
      0,
      0,
      &[self.visibility_image.as_ref()],
    );

    Ok(())
  }

  /// Setup the visibility resources once.
  /// return: The result.
  pub(crate) fn setup_once_visibility(&mut self) -> Result<(), HalaRendererError> {
    Ok(())
  }

  /// Setup the visibility resources on device lost.
  /// return: The result.
  pub(crate) fn setup_visibility(&mut self) -> Result<(), HalaRendererError> {
    let depth_reduction_descriptor_set = self.graphics_descriptor_sets.get("depth_reduction")
      .ok_or(HalaRendererError::new("Failed to find the depth reduction descriptor set.", None))?;
    depth_reduction_descriptor_set.update_sampled_images(
      0,
      0,
      &[self.depth_image.as_ref()],
    );

    let one_pass_culling_descriptor_set = self.graphics_descriptor_sets.get("one_pass_culling")
      .ok_or(HalaRendererError::new("Failed to find the one pass culling descriptor set.", None))?;
    one_pass_culling_descriptor_set.update_sampled_images(
      0,
      0,
      &[self.hiz_image.as_ref()],
    );

    let pre_culling_descriptor_set = self.graphics_descriptor_sets.get("pre_culling")
      .ok_or(HalaRendererError::new("Failed to find the pre culling descriptor set.", None))?;
    pre_culling_descriptor_set.update_sampled_images(
      0,
      0,
      &[self.hiz_image.as_ref()],
    );
    pre_culling_descriptor_set.update_storage_buffers(
      0,
      1,
      &[self.pre_culling_flags.as_ref().unwrap()],
    );

    let visibility_buffer_descriptor_set = self.graphics_descriptor_sets.get("visibility_buffer")
      .ok_or(HalaRendererError::new("Failed to find the visibility buffer descriptor set.", None))?;
    visibility_buffer_descriptor_set.update_sampled_images(
      0,
      0,
      &[self.hiz_image.as_ref()],
    );
    visibility_buffer_descriptor_set.update_storage_buffers(
      0,
      1,
      &[self.pre_culling_flags.as_ref().unwrap()],
    );

    let material_depth_descriptor_set = self.graphics_descriptor_sets.get("material_depth")
      .ok_or(HalaRendererError::new("Failed to find the material depth descriptor set.", None))?;
    material_depth_descriptor_set.update_input_attachments(
      0,
      0,
      &[self.visibility_image.as_ref()],
    );
    material_depth_descriptor_set.update_input_attachments(
      0,
      1,
      &[self.depth_image.as_ref()],
    );

    let clear_indirect_buffer_descriptor_set = self.compute_descriptor_sets.get("clear_indirect_buffer")
      .ok_or(HalaRendererError::new("Failed to find the clear indirect buffer descriptor set.", None))?;
    clear_indirect_buffer_descriptor_set.update_storage_buffers(
      0,
      0,
      &[self.indirect_draw_buffer.as_ref()],
    );

    let material_classification_descriptor_set = self.compute_descriptor_sets.get("material_classification")
      .ok_or(HalaRendererError::new("Failed to find the material classification descriptor set.", None))?;
    material_classification_descriptor_set.update_combined_image_samplers(
      0,
      0,
      &[(self.visibility_image.as_ref(), self.point_sampler.as_ref())],
    );
    material_classification_descriptor_set.update_combined_image_samplers(
      0,
      1,
      &[(self.depth_image.as_ref(), self.point_sampler.as_ref())],
    );
    material_classification_descriptor_set.update_storage_buffers(
      0,
      2,
      &[self.indirect_draw_buffer.as_ref()],
    );
    material_classification_descriptor_set.update_storage_buffers(
      0,
      3,
      &[self.tile_index_buffer.as_ref()],
    );

    let material_tile_descriptor_set = self.graphics_descriptor_sets.get("material_tile")
      .ok_or(HalaRendererError::new("Failed to find the material tile descriptor set.", None))?;
    material_tile_descriptor_set.update_storage_buffers(
      0,
      0,
      &[self.tile_index_buffer.as_ref()],
    );
    material_tile_descriptor_set.update_input_attachments(
      0,
      1,
      &[self.visibility_image.as_ref()],
    );

    let lighting_descriptor_set = self.graphics_descriptor_sets.get("lighting")
      .ok_or(HalaRendererError::new("Failed to find the lighting descriptor set.", None))?;
    lighting_descriptor_set.update_input_attachments(
      0,
      0,
      &[self.albedo_image.as_ref()],
    );
    lighting_descriptor_set.update_input_attachments(
      0,
      1,
      &[self.normal_image.as_ref()],
    );
    lighting_descriptor_set.update_input_attachments(
      0,
      2,
      &[self.depth_image.as_ref()],
    );

    Ok(())
  }

}