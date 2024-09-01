use std::rc::Rc;

use hala_renderer::error::HalaRendererError;

use hala_renderer::renderer::{
  HalaRendererInfo,
  HalaRendererResources,
  HalaRendererData,
  HalaRendererStatistics,
  HalaRendererTrait,
};

use super::{
  GlobalUniform,
  ObjectUniform,
  GlobalConstants,
  VisRenderer,
};

/// The RendererTrait implementation for the visibility renderer.
impl HalaRendererTrait for VisRenderer {

  fn info(&self) -> &HalaRendererInfo {
    &self.info
  }

  fn info_mut(&mut self) -> &mut HalaRendererInfo {
    &mut self.info
  }

  fn resources(&self) -> &HalaRendererResources {
    &self.resources
  }

  fn resources_mut(&mut self) -> &mut HalaRendererResources {
    &mut self.resources
  }

  fn data(&self) -> &HalaRendererData {
    &self.data
  }

  fn data_mut(&mut self) -> &mut HalaRendererData {
    &mut self.data
  }

  fn statistics(&self) -> &HalaRendererStatistics {
    &self.statistics
  }

  fn statistics_mut(&mut self) -> &mut HalaRendererStatistics {
    &mut self.statistics
  }

  fn get_descriptor_sizes() -> Vec<(hala_gfx::HalaDescriptorType, usize)> {
    vec![
      (
        hala_gfx::HalaDescriptorType::INPUT_ATTACHMENT,
        16,
      ),
      (
        hala_gfx::HalaDescriptorType::STORAGE_IMAGE,
        16,
      ),
      (
        hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
        32,
      ),
      (
        hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
        256,
      ),
      (
        hala_gfx::HalaDescriptorType::SAMPLED_IMAGE,
        256,
      ),
      (
        hala_gfx::HalaDescriptorType::SAMPLER,
        256,
      ),
      (
        hala_gfx::HalaDescriptorType::COMBINED_IMAGE_SAMPLER,
        256,
      ),
    ]
  }

  /// Commit all GPU resources.
  /// return: The result.
  fn commit(&mut self) -> Result<(), HalaRendererError> {
    let context = self.resources.context.borrow();
    let scene = self.scene_in_gpu.as_ref().ok_or(HalaRendererError::new("The scene in GPU is none!", None))?;
    let meshlets = scene.meshlets.as_ref().ok_or(HalaRendererError::new("The global meshlet buffer is none!", None))?;
    let meshlet_draw_data = scene.meshlet_draw_data.as_ref().ok_or(HalaRendererError::new("The draw data buffer is none!", None))?;

    // Assert camera count.
    if scene.camera_view_matrices.is_empty() || scene.camera_proj_matrices.is_empty() {
      return Err(HalaRendererError::new("There is no camera in the scene!", None));
    }

    // Update static descriptor set.
    self.static_descriptor_set.update_uniform_buffers(0, 0, &[self.global_uniform_buffer.as_ref()]);
    self.static_descriptor_set.update_uniform_buffers(0, 1, &[scene.cameras.as_ref()]);
    self.static_descriptor_set.update_uniform_buffers(0, 2, &[scene.lights.as_ref()]);
    self.static_descriptor_set.update_storage_buffers(0, 3, &[meshlet_draw_data]);
    self.static_descriptor_set.update_storage_buffers(0, 4, &[meshlets]);

    // Collect vertex and index buffers.
    let mut vertex_buffers = Vec::new();
    let mut index_buffers = Vec::new();
    let mut meshlet_buffers = Vec::new();
    let mut meshlet_vertex_buffers = Vec::new();
    let mut meshlet_primitive_buffers = Vec::new();
    for mesh in scene.meshes.iter() {
      for primitive in mesh.primitives.iter() {
        vertex_buffers.push(primitive.vertex_buffer.as_ref());
        index_buffers.push(primitive.index_buffer.as_ref());
        if let Some(meshlet_buffer) = &primitive.meshlet_buffer {
          meshlet_buffers.push(meshlet_buffer);
        }
        meshlet_vertex_buffers.push(primitive.meshlet_vertex_buffer.as_ref().ok_or(HalaRendererError::new("The meshlet vertex buffer is none!", None))?);
        meshlet_primitive_buffers.push(primitive.meshlet_primitive_buffer.as_ref().ok_or(HalaRendererError::new("The meshlet primitive buffer is none!", None))?);
      }
    }

    // Create dynamic descriptor set.
    let dynamic_descriptor_set = hala_gfx::HalaDescriptorSet::new(
      Rc::clone(&context.logical_device),
      Rc::clone(&self.resources.descriptor_pool),
      hala_gfx::HalaDescriptorSetLayout::new(
        Rc::clone(&context.logical_device),
        &[
          hala_gfx::HalaDescriptorSetLayoutBinding { // Materials uniform buffers.
            binding_index: 0,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: scene.materials.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Object uniform buffers.
            binding_index: 1,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: scene.meshes.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Vertex storage buffers.
            binding_index: 2,
            descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
            descriptor_count: vertex_buffers.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Meshlet vertex storage buffers.
            binding_index: 3,
            descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
            descriptor_count: meshlet_vertex_buffers.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Meshlet primitive storage buffers.
            binding_index: 4,
            descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
            descriptor_count: meshlet_primitive_buffers.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
        ],
        "main_dynamic.descriptor_set_layout",
      )?,
      context.swapchain.num_of_images,
      0,
      "main_dynamic.descriptor_set",
    )?;

    for (mesh_index, _mesh) in scene.meshes.iter().enumerate() {
      // Create object uniform buffer.
      let mut buffers = Vec::with_capacity(context.swapchain.num_of_images);
      for index in 0..context.swapchain.num_of_images {
        let buffer = hala_gfx::HalaBuffer::new(
          Rc::clone(&context.logical_device),
          std::mem::size_of::<ObjectUniform>() as u64,
          hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
          hala_gfx::HalaMemoryLocation::CpuToGpu,
          &format!("object_{}_{}.uniform_buffer", mesh_index, index),
        )?;

        buffers.push(buffer);
      }

      self.object_uniform_buffers.push(buffers);
    }

    for index in 0..context.swapchain.num_of_images {
      dynamic_descriptor_set.update_uniform_buffers(
        index,
        0,
        scene.materials.as_slice(),
      );
      dynamic_descriptor_set.update_uniform_buffers(
        index,
        1,
        self.object_uniform_buffers.iter().map(|buffers| &buffers[index]).collect::<Vec<_>>().as_slice(),
      );
      dynamic_descriptor_set.update_storage_buffers(
        index,
        2,
        vertex_buffers.as_slice(),
      );
      if !meshlet_vertex_buffers.is_empty() {
        dynamic_descriptor_set.update_storage_buffers(
          index,
          3,
          meshlet_vertex_buffers.as_slice(),
        );
      }
      if !meshlet_primitive_buffers.is_empty() {
        dynamic_descriptor_set.update_storage_buffers(
          index,
          4,
          meshlet_primitive_buffers.as_slice(),
        );
      }
    }

    // Create texture descriptor set.
    let textures_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      Rc::clone(&context.logical_device),
      Rc::clone(&self.resources.descriptor_pool),
      hala_gfx::HalaDescriptorSetLayout::new(
        Rc::clone(&context.logical_device),
        &[
          hala_gfx::HalaDescriptorSetLayoutBinding { // All textures in the scene.
            binding_index: 0,
            descriptor_type: hala_gfx::HalaDescriptorType::SAMPLED_IMAGE,
            descriptor_count: scene.textures.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // All samplers in the scene.
            binding_index: 1,
            descriptor_type: hala_gfx::HalaDescriptorType::SAMPLER,
            descriptor_count: scene.textures.len() as u32,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
        ],
        "textures.descriptor_set_layout",
      )?,
      0,
      "textures.descriptor_set",
    )?;

    let textures: &Vec<_> = scene.textures.as_ref();
    let samplers: &Vec<_> = scene.samplers.as_ref();
    let images: &Vec<_> = scene.images.as_ref();
    let mut final_images = Vec::new();
    let mut final_samplers = Vec::new();
    for (sampler_index, image_index) in textures.iter().enumerate() {
      let image = images.get(*image_index as usize).ok_or(HalaRendererError::new("The image is none!", None))?;
      let sampler = samplers.get(sampler_index).ok_or(HalaRendererError::new("The sampler is none!", None))?;
      final_images.push(image);
      final_samplers.push(sampler);
    }
    if !final_images.is_empty() && !final_samplers.is_empty() {
      textures_descriptor_set.update_sampled_images(0, 0, final_images.as_slice());
      textures_descriptor_set.update_samplers(0, 1, final_samplers.as_slice());
    }

    self.dynamic_descriptor_set = Some(dynamic_descriptor_set);
    self.textures_descriptor_set = Some(textures_descriptor_set);

    let pre_culling_flags = hala_gfx::HalaBuffer::new(
      Rc::clone(&context.logical_device),
      std::mem::size_of::<u32>() as u64 * scene.meshlet_count as u64,
      hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
      hala_gfx::HalaMemoryLocation::GpuOnly,
      "pre_culling_flags.buffer",
    )?;

    self.pre_culling_flags = Some(pre_culling_flags);

    Ok(())
  }

  /// Check and restore the device.
  /// param width: The width of the window.
  /// param height: The height of the window.
  /// return: The result.
  fn check_and_restore_device(&mut self, width: u32, height: u32) -> Result<(), HalaRendererError> {
    if self.data().is_device_lost {
      unsafe {
        std::mem::ManuallyDrop::drop(&mut self.hiz_image);
        std::mem::ManuallyDrop::drop(&mut self.normal_image);
        std::mem::ManuallyDrop::drop(&mut self.albedo_image);
        std::mem::ManuallyDrop::drop(&mut self.material_depth_image);
        std::mem::ManuallyDrop::drop(&mut self.visibility_image);
        std::mem::ManuallyDrop::drop(&mut self.depth_image);
      }
      let (
        visibility_image,
        depth_image,
        material_depth_image,
        albedo_image,
        normal_image,
        hiz_image,
      ) = Self::create_offscreen_images(self.resources(), width, height, false)?;
      self.visibility_image = std::mem::ManuallyDrop::new(visibility_image);
      self.depth_image = std::mem::ManuallyDrop::new(depth_image);
      self.material_depth_image = std::mem::ManuallyDrop::new(material_depth_image);
      self.albedo_image = std::mem::ManuallyDrop::new(albedo_image);
      self.normal_image = std::mem::ManuallyDrop::new(normal_image);
      self.hiz_image = std::mem::ManuallyDrop::new(hiz_image);

      unsafe {
        std::mem::ManuallyDrop::drop(&mut self.tile_index_buffer);
      }
      let tile_index_buffer = {
        let x = (width + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
        let y = (height + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
        let num_of_tiles = x * y;
        hala_gfx::HalaBuffer::new(
          Rc::clone(&self.resources().context.borrow().logical_device),
          std::mem::size_of::<u32>() as u64 * num_of_tiles as u64 * GlobalConstants::CLASSIFY_DEPTH_RANGE as u64,
          hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
          hala_gfx::HalaMemoryLocation::GpuOnly,
          "tile_index.buffer",
        )?
      };
      self.tile_index_buffer = std::mem::ManuallyDrop::new(tile_index_buffer);

      self.setup_visibility()?;
    }
    self.check_and_restore_swapchain(width, height)?;

    Ok(())
  }

  /// Update the renderer.
  /// param delta_time: The delta time.
  /// param width: The width of the window.
  /// param height: The height of the window.
  /// param ui_fn: The draw UI function.
  /// return: The result.
  fn update<F>(&mut self, _delta_time: f64, width: u32, height: u32, ui_fn: F) -> Result<(), HalaRendererError>
    where F: FnOnce(usize, &hala_gfx::HalaCommandBufferSet) -> Result<(), hala_gfx::HalaGfxError>
  {
    self.pre_update(width, height)?;

    let scene = self.scene_in_gpu.as_ref().ok_or(HalaRendererError::new("The scene in GPU is none!", None))?;

    // Update global uniform buffer(Only use No.1 camera).
    let vp_mtx = scene.camera_proj_matrices[0] * scene.camera_view_matrices[0];
    let global_uniform = GlobalUniform {
      v_mtx: scene.camera_view_matrices[0],
      p_mtx: scene.camera_proj_matrices[0],
      vp_mtx,
      i_vp_mtx: vp_mtx.inverse(),
      frustum_planes: Self::calc_frustum_planes(&vp_mtx, true, true),
    };
    self.global_uniform_buffer.update_memory(0, &[global_uniform])?;

    // Update object uniform buffers.
    for (mesh_index, mesh) in scene.meshes.iter().enumerate() {
      // Prepare object data.
      let mv_mtx = scene.camera_view_matrices[0] * mesh.transform;
      let object_uniform = ObjectUniform {
        m_mtx: mesh.transform,
        i_m_mtx: mesh.transform.inverse(),
        mv_mtx,
        t_mv_mtx: mv_mtx.transpose(),
        it_mv_mtx: mv_mtx.inverse().transpose(),
        mvp_mtx: scene.camera_proj_matrices[0] * mv_mtx,
      };

      for index in 0..self.resources.context.borrow().swapchain.num_of_images {
        let buffer = self.object_uniform_buffers[mesh_index][index].as_ref();
        buffer.update_memory(0, &[object_uniform])?;
      }
    }

    self.record_command_buffer(
      self.data.image_index,
      &self.resources.graphics_command_buffers,
      &self.resources.compute_command_buffers,
      ui_fn,
    )?;

    Ok(())
  }

}