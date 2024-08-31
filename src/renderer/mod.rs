mod renderer_trait;
mod renderer_impl;
mod renderer_setup;
mod renderer_pass;
mod renderer_debug;

use std::collections::HashMap;

use hala_renderer::{
  scene::gpu,
  renderer::{
    HalaRendererInfo,
    HalaRendererResources,
    HalaRendererData,
    HalaRendererStatistics,
  },
  graphics_program::HalaGraphicsProgram,
  compute_program::HalaComputeProgram,
  shader_cache::HalaShaderCache,
};

/// The debug settings.
#[derive(Debug, Default, Clone, Copy)]
pub struct DebugSettings {
  pub show_hiz: bool,
  pub hiz_level: u32,
  pub show_triangle: bool,
  pub show_meshlet: bool,
  pub show_visibility: bool,
  pub show_material_depth: bool,
  pub show_albedo: bool,
  pub show_normal: bool,
  pub grid_line_width: u32,
  pub disable_culling: bool,
  pub one_pass_culling: bool,
}

/// The global uniform.
#[repr(C, align(4))]
#[derive(Debug, Clone, Copy)]
pub struct GlobalUniform {
  // The view matrix.
  pub v_mtx: glam::Mat4,
  // The projection matrix.
  pub p_mtx: glam::Mat4,
  // The view-projection matrix.
  pub vp_mtx: glam::Mat4,
  // The inverse view-projection matrix.
  pub i_vp_mtx: glam::Mat4,

  // The camera frustum planes.
  pub frustum_planes: [glam::Vec4; 6],
}

/// The per-object uniform.
#[repr(C, align(4))]
#[derive(Debug, Clone, Copy)]
pub struct ObjectUniform {
  // The model matrix.
  pub m_mtx: glam::Mat4,
  // The inverse model matrix.
  pub i_m_mtx: glam::Mat4,
  // The model-view matrix.
  pub mv_mtx: glam::Mat4,
  // The transposed model-view matrix.
  pub t_mv_mtx: glam::Mat4,
  // The inverse transposed model-view matrix.
  pub it_mv_mtx: glam::Mat4,
  // The model-view-projection matrix.
  pub mvp_mtx: glam::Mat4,
}

/// The global constants.
#[allow(dead_code)]
pub struct GlobalConstants {
}

#[allow(dead_code)]
impl GlobalConstants {
  pub const CLASSIFY_TILE_WIDTH: u32 = 64;
  pub const CLASSIFY_THREAD_WIDTH: u32 = 16;
  pub const CLASSIFY_MATERIAL_MAX: u32 = 256;
  pub const CLASSIFY_DEPTH_RANGE: u32 = Self::CLASSIFY_MATERIAL_MAX * 32;
}

/// The visibility renderer.
pub struct VisRenderer {

  pub(crate) info: HalaRendererInfo,
  pub(crate) resources: std::mem::ManuallyDrop<HalaRendererResources>,
  pub(crate) data: HalaRendererData,
  pub(crate) statistics: HalaRendererStatistics,

  pub(crate) debug_settings: DebugSettings,

  pub(crate) static_descriptor_set: std::mem::ManuallyDrop<hala_gfx::HalaDescriptorSet>,
  pub(crate) dynamic_descriptor_set: Option<hala_gfx::HalaDescriptorSet>,
  pub(crate) textures_descriptor_set: Option<hala_gfx::HalaDescriptorSet>,

  pub(crate) global_uniform_buffer: std::mem::ManuallyDrop<hala_gfx::HalaBuffer>,
  pub(crate) object_uniform_buffers: Vec<Vec<hala_gfx::HalaBuffer>>,

  pub(crate) scene_in_gpu: Option<gpu::HalaScene>,

  pub(crate) graphics_programs: HashMap<String, HalaGraphicsProgram>,
  pub(crate) graphics_descriptor_sets: HashMap<String, hala_gfx::HalaDescriptorSet>,

  pub(crate) compute_programs: HashMap<String, HalaComputeProgram>,
  pub(crate) compute_descriptor_sets: HashMap<String, hala_gfx::HalaDescriptorSet>,

  pub(crate) visibility_image: std::mem::ManuallyDrop<hala_gfx::HalaImage>,
  pub(crate) depth_image: std::mem::ManuallyDrop<hala_gfx::HalaImage>,
  pub(crate) material_depth_image: std::mem::ManuallyDrop<hala_gfx::HalaImage>,
  pub(crate) albedo_image: std::mem::ManuallyDrop<hala_gfx::HalaImage>,
  pub(crate) normal_image: std::mem::ManuallyDrop<hala_gfx::HalaImage>,
  pub(crate) hiz_image: std::mem::ManuallyDrop<hala_gfx::HalaImage>,

  pub(crate) pre_culling_flags: Option<hala_gfx::HalaBuffer>,

  pub(crate) hiz_descriptor_sets: Vec<hala_gfx::HalaDescriptorSet>,

  pub(crate) point_sampler: std::mem::ManuallyDrop<hala_gfx::HalaSampler>,

  pub(crate) indirect_draw_buffer: std::mem::ManuallyDrop<hala_gfx::HalaBuffer>,
  pub(crate) tile_index_buffer: std::mem::ManuallyDrop<hala_gfx::HalaBuffer>,

}

/// The Drop imlpementation for the visibility renderer.
impl Drop for VisRenderer {

  fn drop(&mut self) {
    self.hiz_descriptor_sets.clear();

    self.pre_culling_flags = None;

    self.scene_in_gpu = None;

    self.compute_descriptor_sets.clear();
    self.compute_programs.clear();

    self.graphics_descriptor_sets.clear();
    self.graphics_programs.clear();

    self.object_uniform_buffers.clear();

    self.textures_descriptor_set = None;
    self.dynamic_descriptor_set = None;

    HalaShaderCache::get_instance().borrow_mut().clear();

    unsafe {
      std::mem::ManuallyDrop::drop(&mut self.tile_index_buffer);
      std::mem::ManuallyDrop::drop(&mut self.indirect_draw_buffer);

      std::mem::ManuallyDrop::drop(&mut self.point_sampler);

      std::mem::ManuallyDrop::drop(&mut self.hiz_image);
      std::mem::ManuallyDrop::drop(&mut self.normal_image);
      std::mem::ManuallyDrop::drop(&mut self.albedo_image);
      std::mem::ManuallyDrop::drop(&mut self.material_depth_image);
      std::mem::ManuallyDrop::drop(&mut self.depth_image);
      std::mem::ManuallyDrop::drop(&mut self.visibility_image);

      std::mem::ManuallyDrop::drop(&mut self.global_uniform_buffer);

      std::mem::ManuallyDrop::drop(&mut self.static_descriptor_set);

      std::mem::ManuallyDrop::drop(&mut self.resources);
    }

    log::info!("The HalaRenderer \"{}\" is dropped.", self.info.name);
  }

}