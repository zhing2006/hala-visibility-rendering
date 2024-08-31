use hala_renderer::{
  error::HalaRendererError,
  graphics_program::HalaGraphicsProgram,
};

use crate::renderer::{
  GlobalConstants,
  VisRenderer,
};

/// The debug implementation of the visibility renderer.
impl VisRenderer {

  /// Setup the swapchain begin barriers.
  /// param context: The context.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn setup_swapchain_begin_barriers(
    &self,
    context: &hala_gfx::HalaContext,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    graphics_command_buffers.set_swapchain_image_barrier(
      index,
      &context.swapchain,
      &hala_gfx::HalaImageBarrierInfo {
        old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
        new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
        dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
        src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
        dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
        ..Default::default()
      },
      &hala_gfx::HalaImageBarrierInfo {
        old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
        new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
        dst_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
        src_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
        dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
        aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH | if context.swapchain.has_stencil { hala_gfx::HalaImageAspectFlags::STENCIL } else { hala_gfx::HalaImageAspectFlags::empty() },
        ..Default::default()
      }
    );

    Ok(())
  }

  /// Setup the swapchain end barriers.
  /// param context: The context.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn setup_swapchain_end_barriers(
    &self,
    context: &hala_gfx::HalaContext,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    graphics_command_buffers.set_image_barriers(
      index,
      &[hala_gfx::HalaImageBarrierInfo {
        old_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        new_layout: hala_gfx::HalaImageLayout::PRESENT_SRC,
        src_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
        dst_access_mask: hala_gfx::HalaAccessFlags2::NONE,
        src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::BOTTOM_OF_PIPE,
        aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
        image: context.swapchain.images[index],
        ..Default::default()
      }],
    );

    Ok(())
  }

  /// Draw the scene.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// param require_push_constants: Whether require push constants.
  /// param graphics_program: The graphics program.
  /// param descriptor_set: The descriptor set.
  /// return: The result.
  pub(crate) fn draw_scene(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
    require_push_constants: bool,
    graphics_program: &HalaGraphicsProgram,
    descriptor_set: Option<&hala_gfx::HalaDescriptorSet>,
  ) -> Result<(), HalaRendererError> {
    let dynamic_descriptor_set = self.dynamic_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the dynamic descriptor set.", None))?;
    let texture_descriptor_set = self.textures_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the textures descriptor set.", None))?;

    if let Some(custom_descriptor_set) = descriptor_set {
      graphics_program.bind(
        index,
        graphics_command_buffers,
        &[
          self.static_descriptor_set.as_ref(),
          dynamic_descriptor_set,
          texture_descriptor_set,
          custom_descriptor_set,
        ],
      );
    } else {
      graphics_program.bind(
        index,
        graphics_command_buffers,
        &[
          self.static_descriptor_set.as_ref(),
          dynamic_descriptor_set,
          texture_descriptor_set,
        ],
      );
    };

    // Render the scene.
    let scene = self.scene_in_gpu.as_ref().ok_or(hala_gfx::HalaGfxError::new("The scene in GPU is none!", None))?;

    if require_push_constants {
      let mut push_constants = Vec::new();
      push_constants.extend_from_slice(&scene.meshlet_count.to_le_bytes());

      graphics_program.push_constants(
        index,
        graphics_command_buffers,
        0,
        push_constants.as_slice(),
      );
    }

    let dispatch_size_x = (scene.meshlet_count + 32 - 1) / 32;  // 32 threads per task group.
    graphics_command_buffers.draw_mesh_tasks(
      index,
      dispatch_size_x,
      1,
      1,
    );

    Ok(())
  }

  /// Draw the screen quad.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// param graphics_program: The graphics program.
  /// param descriptor_set: The descriptor set.
  /// return: The result.
  pub(crate) fn draw_screen_quad(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
    graphics_program: &HalaGraphicsProgram,
    descriptor_set: Option<&hala_gfx::HalaDescriptorSet>,
  ) -> Result<(), HalaRendererError> {
    let dynamic_descriptor_set = self.dynamic_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the dynamic descriptor set.", None))?;
    let texture_descriptor_set = self.textures_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the textures descriptor set.", None))?;

    if let Some(custom_descriptor_set) = descriptor_set {
      graphics_program.bind(
        index,
        graphics_command_buffers,
        &[
          self.static_descriptor_set.as_ref(),
          dynamic_descriptor_set,
          texture_descriptor_set,
          custom_descriptor_set,
        ],
      );
    } else {
      graphics_program.bind(
        index,
        graphics_command_buffers,
        &[
          self.static_descriptor_set.as_ref(),
          dynamic_descriptor_set,
          texture_descriptor_set,
        ],
      );
    };

    // Render the scene.
    {
      graphics_command_buffers.draw(
        index,
        4,
        1,
        0,
        0,
      );
    }

    Ok(())
  }

  /// The clear depth pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn clear_depth_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
          aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH,
          image: self.depth_image.raw,
          ..Default::default()
        },
      ],
    );

    graphics_command_buffers.begin_rendering_with(
      index,
      &[],
      Some(self.depth_image.as_ref()),
      (0, 0, self.info.width, self.info.height),
      &[],
      Some(0.0),
      None,
      hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
      hala_gfx::HalaAttachmentStoreOp::STORE,
      hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
    );

    graphics_command_buffers.end_rendering(index);

    Ok(())
  }

  /// The no culling visibility buffer pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn no_culling_visibility_buffer_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    // Setup barriers.
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.visibility_image.raw,
          ..Default::default()
        },
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
          aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH,
          image: self.depth_image.raw,
          ..Default::default()
        },
      ],
    );

    graphics_command_buffers.begin_rendering_with(
      index,
      &[self.visibility_image.as_ref()],
      Some(self.depth_image.as_ref()),
      (0, 0, self.info.width, self.info.height),
      &[Some([0.0, 0.0, 0.0, 1.0])],
      Some(0.0),
      None,
      hala_gfx::HalaAttachmentStoreOp::STORE,
      hala_gfx::HalaAttachmentStoreOp::STORE,
      hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
    );

    let no_culling_visibility_buffer_program = self.graphics_programs.get("no_culling_visibility_buffer")
      .ok_or(HalaRendererError::new("Failed to find the no culling visibility buffer program.", None))?;
    let no_culling_visibility_buffer_descriptor_set = self.graphics_descriptor_sets.get("no_culling_visibility_buffer");

    self.draw_scene(
      index,
      graphics_command_buffers,
      true,
      no_culling_visibility_buffer_program,
      no_culling_visibility_buffer_descriptor_set,
    )?;

    graphics_command_buffers.end_rendering(index);

    // Because we will skip the culling pass, we need to setup the depth barrier for lighting pass.
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
          new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::INPUT_ATTACHMENT_READ,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER,
          aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH,
          image: self.depth_image.raw,
          ..Default::default()
        },
      ],
    );

    Ok(())
  }

  /// The culling pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn culling_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
    graphics_program: &HalaGraphicsProgram,
    descriptor_set: Option<&hala_gfx::HalaDescriptorSet>,
  ) -> Result<(), HalaRendererError> {
    // Setup barriers.
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.visibility_image.raw,
          ..Default::default()
        },
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
          aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH,
          image: self.depth_image.raw,
          ..Default::default()
        },
      ],
    );

    graphics_command_buffers.begin_rendering_with(
      index,
      &[self.visibility_image.as_ref()],
      Some(self.depth_image.as_ref()),
      (0, 0, self.info.width, self.info.height),
      &[Some([0.0, 0.0, 0.0, 1.0])],
      Some(0.0),
      None,
      hala_gfx::HalaAttachmentStoreOp::STORE,
      hala_gfx::HalaAttachmentStoreOp::STORE,
      hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
    );

    let scene = self.scene_in_gpu.as_ref().ok_or(hala_gfx::HalaGfxError::new("The scene in GPU is none!", None))?;

    let mut push_constants = Vec::new();
    push_constants.extend_from_slice(&scene.meshlet_count.to_le_bytes());
    push_constants.extend_from_slice(&self.hiz_image.mip_levels.to_le_bytes());
    push_constants.extend_from_slice(&self.hiz_image.extent.width.to_le_bytes());
    push_constants.extend_from_slice(&self.hiz_image.extent.height.to_le_bytes());
    graphics_program.push_constants(
      index,
      graphics_command_buffers,
      0,
      push_constants.as_slice(),
    );

    self.draw_scene(
      index,
      graphics_command_buffers,
      false,
      graphics_program,
      descriptor_set,
    )?;

    graphics_command_buffers.end_rendering(index);

    Ok(())
  }

  /// The visibility buffer pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn visibility_buffer_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    // Setup visibility buffer barriers.
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.visibility_image.raw,
          ..Default::default()
        },
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
          aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH,
          image: self.depth_image.raw,
          ..Default::default()
        },
      ],
    );

    // Draw the visibility buffer.
    graphics_command_buffers.begin_rendering_with_ex(
      index,
      &[self.visibility_image.as_ref()],
      Some(self.depth_image.as_ref()),
      (0, 0, self.info.width, self.info.height),
      &[None],
      None,
      None,
      &[hala_gfx::HalaAttachmentLoadOp::LOAD],
      hala_gfx::HalaAttachmentLoadOp::LOAD,
      hala_gfx::HalaAttachmentLoadOp::DONT_CARE,
      &[hala_gfx::HalaAttachmentStoreOp::STORE],
      hala_gfx::HalaAttachmentStoreOp::STORE,
      hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
    );

    let scene = self.scene_in_gpu.as_ref().ok_or(hala_gfx::HalaGfxError::new("The scene in GPU is none!", None))?;
    let visibility_buffer_program = self.graphics_programs.get("visibility_buffer")
      .ok_or(HalaRendererError::new("Failed to find the visibility buffer program.", None))?;
    let visibility_buffer_descriptor_set = self.graphics_descriptor_sets.get("visibility_buffer");

    let mut push_constants = Vec::new();
    push_constants.extend_from_slice(&scene.meshlet_count.to_le_bytes());
    push_constants.extend_from_slice(&self.hiz_image.mip_levels.to_le_bytes());
    push_constants.extend_from_slice(&self.hiz_image.extent.width.to_le_bytes());
    push_constants.extend_from_slice(&self.hiz_image.extent.height.to_le_bytes());
    visibility_buffer_program.push_constants(
      index,
      graphics_command_buffers,
      0,
      push_constants.as_slice(),
    );

    self.draw_scene(
      index,
      graphics_command_buffers,
      false,
      visibility_buffer_program,
      visibility_buffer_descriptor_set,
    )?;

    graphics_command_buffers.end_rendering(index);

    Ok(())
  }

  /// The depth reduction pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn depth_reduction_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    // Setup depth reduction barriers.
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
          new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::INPUT_ATTACHMENT_READ | hala_gfx::HalaAccessFlags2::SHADER_READ,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER | hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH,
          image: self.depth_image.raw,
          ..Default::default()
        },
      ],
    );

    let depth_reduction_program = self.graphics_programs.get("depth_reduction")
      .ok_or(HalaRendererError::new("Failed to find the depth reduction program.", None))?;
    let depth_reduction_descriptor_set = self.graphics_descriptor_sets.get("depth_reduction");

    let mut width = self.hiz_image.extent.width;
    let mut height = self.hiz_image.extent.height;
    for mip_level in 0..self.hiz_image.mip_levels {
      // Setup the write barrier.
      graphics_command_buffers.set_image_barriers(
        index,
        &[hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::GENERAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.hiz_image.raw,
          base_mip_level: mip_level,
          ..Default::default()
        }],
      );

      // Set mip level viewport and scissor.
      graphics_command_buffers.set_viewport(
        index,
        0,
        &[
          (0., 0., width as f32, height as f32, 0., 1.),
        ],
      );
      graphics_command_buffers.set_scissor(
        index,
        0,
        &[
          (0, 0, width, height),
        ],
      );

      graphics_command_buffers.begin_rendering_with_view_ex(
        index,
        &[self.hiz_image.mip_views[mip_level as usize]],
        None,
        (0, 0, width, height),
        &[None],
        None,
        None,
        &[hala_gfx::HalaAttachmentLoadOp::DONT_CARE],
        hala_gfx::HalaAttachmentLoadOp::DONT_CARE,
        &[hala_gfx::HalaAttachmentStoreOp::STORE],
        hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
      );

      self.draw_screen_quad(
        index,
        graphics_command_buffers,
        depth_reduction_program,
        if mip_level == 0 { depth_reduction_descriptor_set } else { self.hiz_descriptor_sets.get(mip_level as usize - 1) },
      )?;

      graphics_command_buffers.end_rendering(index);

      // Set screen viewport and scissor.
      graphics_command_buffers.set_viewport(
        index,
        0,
        &[
          (
            0.,
            self.info.height as f32,
            self.info.width as f32,
            -(self.info.height as f32), // For vulkan y is down.
            0.,
            1.
          ),
        ],
      );
      graphics_command_buffers.set_scissor(
        index,
        0,
        &[
          (0, 0, self.info.width, self.info.height),
        ],
      );

      // Setup the read barrier.
      graphics_command_buffers.set_image_barriers(
        index,
        &[hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::GENERAL,
          new_layout: hala_gfx::HalaImageLayout::SHADER_READ_ONLY_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.hiz_image.raw,
          base_mip_level: mip_level,
          ..Default::default()
        }],
      );

      width >>= 1;
      height >>= 1;
    }

    Ok(())
  }

  /// The material depth pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn material_depth_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    // Setup material depth barriers.
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          new_layout: hala_gfx::HalaImageLayout::SHADER_READ_ONLY_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::INPUT_ATTACHMENT_READ | hala_gfx::HalaAccessFlags2::SHADER_READ,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER | hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.visibility_image.raw,
          ..Default::default()
        },
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::EARLY_FRAGMENT_TESTS | hala_gfx::HalaPipelineStageFlags2::LATE_FRAGMENT_TESTS,
          aspect_mask: hala_gfx::HalaImageAspectFlags::DEPTH,
          image: self.material_depth_image.raw,
          ..Default::default()
        },
      ],
    );

    // Draw material depth buffer.
    {
      graphics_command_buffers.begin_rendering_with(
        index,
        &[],
        Some(self.material_depth_image.as_ref()),
        (0, 0, self.info.width, self.info.height),
        &[],
        None,
        None,
        hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
        hala_gfx::HalaAttachmentStoreOp::STORE,
        hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
      );

      let material_depth_program = self.graphics_programs.get("material_depth")
        .ok_or(HalaRendererError::new("Failed to find the material depth program.", None))?;
      let material_depth_descriptor_set = self.graphics_descriptor_sets.get("material_depth");
      self.draw_screen_quad(
        index,
        graphics_command_buffers,
        material_depth_program,
        material_depth_descriptor_set,
      )?;

      graphics_command_buffers.end_rendering(index);
    }

    Ok(())
  }

  /// Clear the indirect buffer.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn clear_indirect_buffer(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    // Setup indirect buffer barriers.
    graphics_command_buffers.set_buffer_barriers(
      index,
      &[
        hala_gfx::HalaBufferBarrierInfo {
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          buffer: self.indirect_draw_buffer.raw,
          size: self.indirect_draw_buffer.size,
          ..Default::default()
        },
      ],
    );

    let scene = self.scene_in_gpu.as_ref().ok_or(hala_gfx::HalaGfxError::new("The scene in GPU is none!", None))?;
    let dynamic_descriptor_set = self.dynamic_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the dynamic descriptor set.", None))?;
    let texture_descriptor_set = self.textures_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the textures descriptor set.", None))?;

    let clear_indirect_buffer_program = self.compute_programs.get("clear_indirect_buffer")
      .ok_or(HalaRendererError::new("Failed to find the clear indirect buffer program.", None))?;
    let clear_indirect_buffer_descriptor_set = self.compute_descriptor_sets.get("clear_indirect_buffer")
      .ok_or(HalaRendererError::new("Failed to find the clear indirect buffer descriptor set.", None))?;
    clear_indirect_buffer_program.bind(
      index,
      graphics_command_buffers,
      &[
        self.static_descriptor_set.as_ref(),
        dynamic_descriptor_set,
        texture_descriptor_set,
        clear_indirect_buffer_descriptor_set,
      ],
    );

    graphics_command_buffers.dispatch(
      index,
      (scene.materials.len() as u32 + 32 - 1) / 32,
      1,
      1,
    );

    Ok(())
  }

  /// The material classification pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// param compute_command_buffers: The compute command buffers.
  /// return: The result.
  pub(crate) fn material_classification_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
    _compute_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    // Setup material classification barriers.
    graphics_command_buffers.set_buffer_barriers(
      index,
      &[
        hala_gfx::HalaBufferBarrierInfo {
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ | hala_gfx::HalaAccessFlags2::SHADER_WRITE,
          buffer: self.indirect_draw_buffer.raw,
          size: self.indirect_draw_buffer.size,
          ..Default::default()
        },
        hala_gfx::HalaBufferBarrierInfo {
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          buffer: self.tile_index_buffer.raw,
          size: self.tile_index_buffer.size,
          ..Default::default()
        },
      ],
    );

    let material_classification_program = self.compute_programs.get("material_classification")
      .ok_or(HalaRendererError::new("Failed to find the material classification program.", None))?;
    let material_classification_descriptor_set = self.compute_descriptor_sets.get("material_classification")
      .ok_or(HalaRendererError::new("Failed to find the material classification descriptor set.", None))?;
    let dynamic_descriptor_set = self.dynamic_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the dynamic descriptor set.", None))?;
    let texture_descriptor_set = self.textures_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the textures descriptor set.", None))?;

    let x = (self.info.width + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
    let y = (self.info.height + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
    let num_of_tiles = x * y;
    let mut push_constants = Vec::new();
    push_constants.extend_from_slice(&self.info.width.to_le_bytes());
    push_constants.extend_from_slice(&self.info.height.to_le_bytes());
    push_constants.extend_from_slice(&x.to_le_bytes());
    push_constants.extend_from_slice(&num_of_tiles.to_le_bytes());

    material_classification_program.bind(
      index,
      graphics_command_buffers,
      &[
        self.static_descriptor_set.as_ref(),
        dynamic_descriptor_set,
        texture_descriptor_set,
        material_classification_descriptor_set,
      ],
    );
    material_classification_program.push_constants(
      index,
      graphics_command_buffers,
      0,
      push_constants.as_slice(),
    );

    graphics_command_buffers.dispatch(
      index,
      x,
      y,
      1,
    );

    Ok(())
  }

  /// The material tile pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn material_tile_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    let scene = self.scene_in_gpu.as_ref().ok_or(hala_gfx::HalaGfxError::new("The scene in GPU is none!", None))?;

    let material_tile_program = self.graphics_programs.get("material_tile")
      .ok_or(HalaRendererError::new("Failed to find the material tile program.", None))?;
    let material_tile_descriptor_set = self.graphics_descriptor_sets.get("material_tile")
      .ok_or(HalaRendererError::new("Failed to find the material tile descriptor set.", None))?;
    let dynamic_descriptor_set = self.dynamic_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the dynamic descriptor set.", None))?;
    let texture_descriptor_set = self.textures_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the textures descriptor set.", None))?;

    // Setup material classification barriers.
    graphics_command_buffers.set_buffer_barriers(
      index,
      &[
        hala_gfx::HalaBufferBarrierInfo {
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::DRAW_INDIRECT,
          src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::INDIRECT_COMMAND_READ,
          buffer: self.indirect_draw_buffer.raw,
          size: self.indirect_draw_buffer.size,
          ..Default::default()
        },
        hala_gfx::HalaBufferBarrierInfo {
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COMPUTE_SHADER,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::VERTEX_SHADER | hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER,
          src_access_mask: hala_gfx::HalaAccessFlags2::SHADER_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::SHADER_READ,
          buffer: self.tile_index_buffer.raw,
          size: self.tile_index_buffer.size,
          ..Default::default()
        },
      ],
    );
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.albedo_image.raw,
          ..Default::default()
        },
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::UNDEFINED,
          new_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::NONE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::TOP_OF_PIPE,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.normal_image.raw,
          ..Default::default()
        },
      ],
    );

    graphics_command_buffers.begin_rendering_with_ex(
      index,
      &[self.albedo_image.as_ref(), self.normal_image.as_ref()],
      Some(self.material_depth_image.as_ref()),
      (0, 0, self.info.width, self.info.height),
      &[Some([0.0, 0.0, 0.0, 1.0]), Some([0.0, 0.0, 0.0, 1.0])],
      None,
      None,
      &[hala_gfx::HalaAttachmentLoadOp::CLEAR, hala_gfx::HalaAttachmentLoadOp::CLEAR],
      hala_gfx::HalaAttachmentLoadOp::LOAD,
      hala_gfx::HalaAttachmentLoadOp::DONT_CARE,
      &[hala_gfx::HalaAttachmentStoreOp::STORE, hala_gfx::HalaAttachmentStoreOp::STORE],
      hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
      hala_gfx::HalaAttachmentStoreOp::DONT_CARE,
    );

    material_tile_program.bind(
      index,
      graphics_command_buffers,
      &[
        self.static_descriptor_set.as_ref(),
        dynamic_descriptor_set,
        texture_descriptor_set,
        material_tile_descriptor_set,
      ],
    );

    let x = (self.info.width + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
    let y = (self.info.height + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
    let num_of_tiles = x * y;
    let num_of_materials = scene.materials.len();
    for material_index in 0..num_of_materials {
      let mut push_constants = Vec::new();
      push_constants.extend_from_slice(&self.info.width.to_le_bytes());
      push_constants.extend_from_slice(&self.info.height.to_le_bytes());
      push_constants.extend_from_slice(&x.to_le_bytes());
      push_constants.extend_from_slice(&num_of_tiles.to_le_bytes());
      push_constants.extend_from_slice(&(material_index as u32).to_le_bytes());
      push_constants.extend_from_slice(&self.debug_settings.grid_line_width.to_le_bytes());
      material_tile_program.push_constants(
        index,
        graphics_command_buffers,
        0,
        push_constants.as_slice(),
      );

      // NOTICE: In real world, you can change PSO here.

      graphics_command_buffers.draw_indirect(
        index,
        self.indirect_draw_buffer.as_ref(),
        material_index as u64 * std::mem::size_of::<hala_gfx::HalaIndirectDrawCommand>() as u64,
        1,
        std::mem::size_of::<hala_gfx::HalaIndirectDrawCommand>() as u32,
      );
    }

    graphics_command_buffers.end_rendering(index);

    Ok(())
  }

  /// The lighting pass.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// return: The result.
  pub(crate) fn lighting_pass(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
  ) -> Result<(), HalaRendererError> {
    let lighting_program = self.graphics_programs.get("lighting")
      .ok_or(HalaRendererError::new("Failed to find the lighting program.", None))?;
    let lighting_descriptor_set = self.graphics_descriptor_sets.get("lighting");

    // Setup material tile read barriers.
    graphics_command_buffers.set_image_barriers(
      index,
      &[
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          new_layout: hala_gfx::HalaImageLayout::SHADER_READ_ONLY_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::INPUT_ATTACHMENT_READ,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.albedo_image.raw,
          ..Default::default()
        },
        hala_gfx::HalaImageBarrierInfo {
          old_layout: hala_gfx::HalaImageLayout::COLOR_ATTACHMENT_OPTIMAL,
          new_layout: hala_gfx::HalaImageLayout::SHADER_READ_ONLY_OPTIMAL,
          src_access_mask: hala_gfx::HalaAccessFlags2::COLOR_ATTACHMENT_WRITE,
          dst_access_mask: hala_gfx::HalaAccessFlags2::INPUT_ATTACHMENT_READ,
          src_stage_mask: hala_gfx::HalaPipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
          dst_stage_mask: hala_gfx::HalaPipelineStageFlags2::FRAGMENT_SHADER,
          aspect_mask: hala_gfx::HalaImageAspectFlags::COLOR,
          image: self.normal_image.raw,
          ..Default::default()
        },
      ],
    );

    self.draw_screen_quad(
      index,
      graphics_command_buffers,
      lighting_program,
      lighting_descriptor_set,
    )?;

    Ok(())
  }

}