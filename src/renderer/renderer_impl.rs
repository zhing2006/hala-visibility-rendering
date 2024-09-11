use std::rc::Rc;
use std::collections::HashMap;

use hala_gfx::HalaGPURequirements;

use hala_renderer::error::HalaRendererError;
use hala_renderer::scene::{
  cpu,
  loader,
};

use hala_renderer::{
  renderer::{
    HalaRendererInfo,
    HalaRendererResources,
    HalaRendererData,
    HalaRendererStatistics,
    HalaRendererTrait,
  },
  graphics_program::{
    HalaGraphicsProgramDesc,
    HalaGraphicsProgram,
  },
  compute_program::{
    HalaComputeProgramDesc,
    HalaComputeProgram,
  },
};

use super::{
  DebugSettings,
  GlobalUniform,
  VisRenderer,
};

use crate::config::GPUProgramsConfig;
use crate::renderer::GlobalConstants;

type GraphicsProgramResult = Result<(HashMap<String, HalaGraphicsProgram>, HashMap<String, hala_gfx::HalaDescriptorSet>), HalaRendererError>;
type ComputeProgramResult = Result<(HashMap<String, HalaComputeProgram>, HashMap<String, hala_gfx::HalaDescriptorSet>), HalaRendererError>;

/// The implementation of the visibility renderer.
impl VisRenderer {

  /// Calculate the frustum planes.
  /// param vp_mtx: The view-projection matrix.
  /// param is_z_reversed: Whether the Z is reversed.
  /// param is_infinite: Whether the frustum is infinite.
  /// return: The frustum planes.
  pub fn calc_frustum_planes(vp_mtx: &glam::Mat4, is_z_reversed: bool, is_infinite: bool) -> [glam::Vec4; 6] {
    let near_z: f32 = if is_z_reversed { 1.0 } else { 0.0 };
    let far_z: f32 = 0.5;

    let i_vp_mtx = vp_mtx.inverse();
    let base_points = [
      glam::Vec4::new(-1.0,  1.0, near_z, 1.0),
      glam::Vec4::new( 1.0,  1.0, near_z, 1.0),
      glam::Vec4::new(-1.0, -1.0, near_z, 1.0),
      glam::Vec4::new( 1.0, -1.0, near_z, 1.0),
      glam::Vec4::new(-1.0,  1.0, far_z, 1.0),
      glam::Vec4::new( 1.0,  1.0, far_z, 1.0),
      glam::Vec4::new(-1.0, -1.0, far_z, 1.0),
      glam::Vec4::new( 1.0, -1.0, far_z, 1.0),
    ];
    let frustum_points = base_points.iter().map(|point| {
      let point_ws = i_vp_mtx * *point;
      (point_ws / point_ws.w).truncate()
    }).collect::<Vec<_>>();

    let calc_plane = |a: usize, b: usize, c: usize| -> glam::Vec4 {
      let ab = frustum_points[b] - frustum_points[a];
      let ac = frustum_points[c] - frustum_points[a];
      let normal = ab.cross(ac).normalize();
      let d = -normal.dot(frustum_points[a]);
      glam::Vec4::new(normal.x, normal.y, normal.z, d)
    };

    [
      calc_plane(0, 2, 4),  // Left
      calc_plane(1, 5, 3),  // Right
      calc_plane(0, 4, 1),  // Top
      calc_plane(2, 3, 6),  // Bottom
      calc_plane(0, 1, 2),  // Near
      if is_infinite {
        calc_plane(0, 1, 2) // Infinite use the same plane as near.
      } else {
        calc_plane(4, 6, 5) // Far
      },
    ]
  }

  /// Create a new renderer.
  /// param name: The name of the renderer.
  /// param gpu_req: The GPU requirements of the renderer.
  /// param window: The window of the renderer.
  /// return: The renderer.
  pub fn new(
    name: &str,
    gpu_req: &HalaGPURequirements,
    window: &winit::window::Window,
  ) -> Result<Self, HalaRendererError> {
    let width = gpu_req.width;
    let height = gpu_req.height;

    let resources = HalaRendererResources::new(
      name,
      gpu_req,
      window,
      &Self::get_descriptor_sizes(),
    )?;

    let static_descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
      Rc::clone(&resources.context.borrow().logical_device),
      Rc::clone(&resources.descriptor_pool),
      hala_gfx::HalaDescriptorSetLayout::new(
        Rc::clone(&resources.context.borrow().logical_device),
        &[
          hala_gfx::HalaDescriptorSetLayoutBinding { // Global uniform buffer.
            binding_index: 0,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Cameras uniform buffer.
            binding_index: 1,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Lights uniform buffer.
            binding_index: 2,
            descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Draw data storage buffer.
            binding_index: 3,
            descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
          hala_gfx::HalaDescriptorSetLayoutBinding { // Global meshlet storage buffer.
            binding_index: 4,
            descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
            descriptor_count: 1,
            stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::COMPUTE
              | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH | hala_gfx::HalaShaderStageFlags::VERTEX,
            binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
          },
        ],
        "main_static.descriptor_set_layout",
      )?,
      0,
      "main_static.descriptor_set",
    )?;

    // Create global uniform buffer.
    let global_uniform_buffer = hala_gfx::HalaBuffer::new(
      Rc::clone(&resources.context.borrow().logical_device),
      std::mem::size_of::<GlobalUniform>() as u64,
      hala_gfx::HalaBufferUsageFlags::UNIFORM_BUFFER,
      hala_gfx::HalaMemoryLocation::CpuToGpu,
      "global.uniform_buffer",
    )?;

    let (
      visibility_image,
      depth_image,
      material_depth_image,
      albedo_image,
      normal_image,
      hiz_image,
    ) = Self::create_offscreen_images(&resources, width, height, false)?;

    // Create the point sampler.
    let point_sampler = hala_gfx::HalaSampler::new(
      Rc::clone(&resources.context.borrow().logical_device),
      (hala_gfx::HalaFilter::NEAREST, hala_gfx::HalaFilter::NEAREST),
      hala_gfx::HalaSamplerMipmapMode::NEAREST,
      (hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE, hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE, hala_gfx::HalaSamplerAddressMode::CLAMP_TO_EDGE),
      0.0,
      false,
      0.0,
      (0.0, 0.0),
      "point.sampler",
    )?;

    // Create the HiZ descriptor sets.
    let mut hiz_descriptor_sets = Vec::new();
    for mip_level in 0..hiz_image.mip_levels {
      let descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
        Rc::clone(&resources.context.borrow().logical_device),
        Rc::clone(&resources.descriptor_pool),
        hala_gfx::HalaDescriptorSetLayout::new(
          Rc::clone(&resources.context.borrow().logical_device),
          &[
            hala_gfx::HalaDescriptorSetLayoutBinding {
              binding_index: 0,
              descriptor_type: hala_gfx::HalaDescriptorType::SAMPLED_IMAGE,
              descriptor_count: 1,
              stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::VERTEX | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH,
              binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
            },
            hala_gfx::HalaDescriptorSetLayoutBinding {
              binding_index: 1,
              descriptor_type: hala_gfx::HalaDescriptorType::SAMPLER,
              descriptor_count: 1,
              stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT | hala_gfx::HalaShaderStageFlags::VERTEX | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH,
              binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
            },
          ],
          &format!("hiz.descriptor_set_layout[{}]", mip_level),
        )?,
        0,
        &format!("hiz.descriptor_set[{}]", mip_level),
      )?;

      descriptor_set.update_sampled_images_with_view(
        0,
        0,
        &[hiz_image.mip_views[mip_level as usize]],
      );
      descriptor_set.update_samplers(
        0,
        1,
        &[point_sampler.as_ref()],
      );

      hiz_descriptor_sets.push(descriptor_set);
    }

    // Create indirect draw buffer.
    let indirect_draw_buffer = hala_gfx::HalaBuffer::new(
      Rc::clone(&resources.context.borrow().logical_device),
      std::mem::size_of::<hala_gfx::HalaIndirectDrawCommand>() as u64 * GlobalConstants::CLASSIFY_DEPTH_RANGE as u64,
      hala_gfx::HalaBufferUsageFlags::INDIRECT_BUFFER | hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
      hala_gfx::HalaMemoryLocation::GpuOnly,
      "indirect_draw.buffer",
    )?;
    let tile_index_buffer = {
      let x = (width + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
      let y = (height + GlobalConstants::CLASSIFY_TILE_WIDTH - 1) / GlobalConstants::CLASSIFY_TILE_WIDTH;
      let num_of_tiles = x * y;
      hala_gfx::HalaBuffer::new(
        Rc::clone(&resources.context.borrow().logical_device),
        std::mem::size_of::<u32>() as u64 * num_of_tiles as u64 * GlobalConstants::CLASSIFY_DEPTH_RANGE as u64,
        hala_gfx::HalaBufferUsageFlags::STORAGE_BUFFER | hala_gfx::HalaBufferUsageFlags::TRANSFER_SRC,
        hala_gfx::HalaMemoryLocation::GpuOnly,
        "tile_index.buffer",
      )?
    };

    // Return the renderer.
    log::debug!("A HalaRenderer \"{}\"[{} x {}] is created.", name, width, height);
    Ok(Self {
      info: HalaRendererInfo::new(name, width, height),
      resources,
      data: HalaRendererData::new(),
      statistics: HalaRendererStatistics::new(),

      debug_settings: DebugSettings::default(),

      static_descriptor_set,
      dynamic_descriptor_set: None,
      textures_descriptor_set: None,

      global_uniform_buffer,
      object_uniform_buffers: Vec::new(),

      scene_in_gpu: None,

      graphics_programs: HashMap::new(),
      graphics_descriptor_sets: HashMap::new(),

      compute_programs: HashMap::new(),
      compute_descriptor_sets: HashMap::new(),

      visibility_image: std::mem::ManuallyDrop::new(visibility_image),
      depth_image: std::mem::ManuallyDrop::new(depth_image),
      material_depth_image: std::mem::ManuallyDrop::new(material_depth_image),
      albedo_image: std::mem::ManuallyDrop::new(albedo_image),
      normal_image: std::mem::ManuallyDrop::new(normal_image),
      hiz_image: std::mem::ManuallyDrop::new(hiz_image),

      pre_culling_flags: None,

      hiz_descriptor_sets,

      point_sampler,

      indirect_draw_buffer,
      tile_index_buffer: std::mem::ManuallyDrop::new(tile_index_buffer),
    })
  }

  /// Create the offscreen images.
  /// param width: The width of the images.
  /// param height: The height of the images.
  /// return: The visibility image, depth image, material depth image, albedo image and normal image.
  pub fn create_offscreen_images(
    resources: &HalaRendererResources,
    width: u32,
    height: u32,
    use_small_gbuffer: bool,
  ) -> Result<(
    hala_gfx::HalaImage,
    hala_gfx::HalaImage,
    hala_gfx::HalaImage,
    hala_gfx::HalaImage,
    hala_gfx::HalaImage,
    hala_gfx::HalaImage,
  ), HalaRendererError> {
    // Create visibility render target.
    let visibility_image = hala_gfx::HalaImage::new_2d(
      Rc::clone(&resources.context.borrow().logical_device),
      hala_gfx::HalaImageUsageFlags::COLOR_ATTACHMENT | hala_gfx::HalaImageUsageFlags::INPUT_ATTACHMENT | hala_gfx::HalaImageUsageFlags::SAMPLED,
      hala_gfx::HalaFormat::R32_UINT,
      width,
      height,
      1,
      1,
      hala_gfx::HalaMemoryLocation::GpuOnly,
      "visibility.image",
    )?;

    // Create depth render target.
    let depth_image = hala_gfx::HalaImage::new_2d(
      Rc::clone(&resources.context.borrow().logical_device),
      hala_gfx::HalaImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | hala_gfx::HalaImageUsageFlags::INPUT_ATTACHMENT | hala_gfx::HalaImageUsageFlags::SAMPLED,
      hala_gfx::HalaFormat::D32_SFLOAT,
      width,
      height,
      1,
      1,
      hala_gfx::HalaMemoryLocation::GpuOnly,
      "offscreen_depth.image",
    )?;

    // Create material depth render target.
    let material_depth_image = hala_gfx::HalaImage::new_2d(
      Rc::clone(&resources.context.borrow().logical_device),
      hala_gfx::HalaImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | hala_gfx::HalaImageUsageFlags::INPUT_ATTACHMENT | hala_gfx::HalaImageUsageFlags::SAMPLED,
      hala_gfx::HalaFormat::D32_SFLOAT,
      width,
      height,
      1,
      1,
      hala_gfx::HalaMemoryLocation::GpuOnly,
      "material_depth.image",
    )?;

    // Create albedo render target.
    let albedo_image = hala_gfx::HalaImage::new_2d(
      Rc::clone(&resources.context.borrow().logical_device),
      hala_gfx::HalaImageUsageFlags::COLOR_ATTACHMENT | hala_gfx::HalaImageUsageFlags::INPUT_ATTACHMENT,
      if use_small_gbuffer { hala_gfx::HalaFormat::R8G8B8A8_UNORM } else { hala_gfx::HalaFormat::R32G32B32A32_SFLOAT },
      width,
      height,
      1,
      1,
      hala_gfx::HalaMemoryLocation::GpuOnly,
      "albedo.image",
    )?;

    // Create normal render target.
    let normal_image = hala_gfx::HalaImage::new_2d(
      Rc::clone(&resources.context.borrow().logical_device),
      hala_gfx::HalaImageUsageFlags::COLOR_ATTACHMENT | hala_gfx::HalaImageUsageFlags::INPUT_ATTACHMENT,
      if use_small_gbuffer { hala_gfx::HalaFormat::A2R10G10B10_UNORM_PACK32 } else { hala_gfx::HalaFormat::R32G32B32A32_SFLOAT },
      width,
      height,
      1,
      1,
      hala_gfx::HalaMemoryLocation::GpuOnly,
      "normal.image",
    )?;

    // Create Hi-Z render target.
    let hiz_image = hala_gfx::HalaImage::new_2d_with_seperate_views(
      Rc::clone(&resources.context.borrow().logical_device),
      hala_gfx::HalaImageUsageFlags::COLOR_ATTACHMENT | hala_gfx::HalaImageUsageFlags::SAMPLED,
      hala_gfx::HalaFormat::R32_SFLOAT,
      width / 2,
      height / 2,
      5,
      1,
      hala_gfx::HalaMemoryLocation::GpuOnly,
      "hiz.image",
    )?;

    Ok((
      visibility_image,
      depth_image,
      material_depth_image,
      albedo_image,
      normal_image,
      hiz_image,
    ))
  }

  /// Set the scene to be rendered.
  /// param scene_in_cpu: The scene in the CPU.
  /// return: The result.
  pub fn set_scene(&mut self, scene_in_cpu: &mut cpu::HalaScene) -> Result<(), HalaRendererError> {
    let context = self.resources.context.borrow();

    // Release the old scene in the GPU.
    self.scene_in_gpu = None;

    // Upload the new scene to the GPU.
    let scene_in_gpu = loader::HalaSceneGPUUploader::upload(
      &context,
      &self.resources.graphics_command_buffers,
      &self.resources.transfer_command_buffers,
      scene_in_cpu,
      true,
      true,
    false)?;

    if scene_in_gpu.materials.len() >= 256 * 32 {
      log::error!("The materials count \"{}\" is too large than the limit \"{}\".", scene_in_gpu.materials.len(), 256 * 32);
      return Err(HalaRendererError::new("The materials count is too large than the limit.", None));
    }

    self.scene_in_gpu = Some(scene_in_gpu);

    Ok(())
  }

  /// Load all GPU programs.
  /// param path: The path to the GPU programs configure.
  /// return: The result.
  pub fn load_gpu_programs<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(), HalaRendererError> {
    let path = path.as_ref();
    let config = match GPUProgramsConfig::load(path) {
      Ok(config) => config,
      Err(err) => {
        log::error!("Failed to load the GPU programs configure: {:?}", err);
        return Err(HalaRendererError::new("Failed to load the GPU programs configure.", None));
      }
    };

    if cfg!(debug_assertions) {
      for (name, _) in config.graphics_programs.iter() {
        log::debug!("Graphics program \"{}\".", name);
      }
      for (name, _) in config.compute_programs.iter() {
        log::debug!("Compute program \"{}\".", name);
      }
    }

    // If we have cache file at ./out/pipeline_cache.bin, we can load it.
    let pipeline_cache = if std::path::Path::new("./out/pipeline_cache.bin").exists() {
      log::debug!("Load pipeline cache from file: ./out/pipeline_cache.bin");
      hala_gfx::HalaPipelineCache::with_cache_file(
        Rc::clone(&self.resources.context.borrow().logical_device),
        "./out/pipeline_cache.bin",
      )?
    } else {
      log::debug!("Create a new pipeline cache.");
      hala_gfx::HalaPipelineCache::new(
        Rc::clone(&self.resources.context.borrow().logical_device),
      )?
    };

    let (
      graphics_programs,
      graphics_descriptor_sets
    ) = self.create_graphics_program(
      &config.graphics_programs,
      &pipeline_cache,
    )?;
    self.graphics_programs = graphics_programs;
    self.graphics_descriptor_sets = graphics_descriptor_sets;

    let (
      compute_programs,
      compute_descriptor_sets
    ) = self.create_compute_program(
      &config.compute_programs,
      &pipeline_cache,
    )?;
    self.compute_programs = compute_programs;
    self.compute_descriptor_sets = compute_descriptor_sets;

    self.setup_debug()?;
    self.setup_once_visibility()?;
    self.setup_visibility()?;

    pipeline_cache.save("./out/pipeline_cache.bin")?;

    Ok(())
  }

  /// Create the graphics program.
  /// param program_config: The program configure.
  /// param pipeline_cache: The pipeline cache.
  /// param program_create_fn: The program create function.
  /// return: The result.
  fn create_graphics_program(
    &self,
    program_config: &HashMap<String, HalaGraphicsProgramDesc>,
    pipeline_cache: &hala_gfx::HalaPipelineCache,
  ) -> GraphicsProgramResult {
    let mut programs = HashMap::new();
    let mut descriptor_sets = HashMap::new();

    let dynamic_descriptor_set = self.dynamic_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the dynamic descriptor set.", None))?;
    let texture_descriptor_set = self.textures_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the textures descriptor set.", None))?;

    for (name, desc) in program_config.iter() {
      let descriptor_bindings = desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
        hala_gfx::HalaDescriptorSetLayoutBinding {
          binding_index: binding_index as u32,
          descriptor_type: *binding_type,
          descriptor_count: 1,
          stage_flags: hala_gfx::HalaShaderStageFlags::FRAGMENT
            | if *binding_type != hala_gfx::HalaDescriptorType::INPUT_ATTACHMENT {
              hala_gfx::HalaShaderStageFlags::VERTEX | hala_gfx::HalaShaderStageFlags::TASK | hala_gfx::HalaShaderStageFlags::MESH
            } else {
              hala_gfx::HalaShaderStageFlags::empty()
            },
          binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
        }
      }).collect::<Vec<_>>();
      let descriptor_set_layouts = if !descriptor_bindings.is_empty() {
        let descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
          Rc::clone(&self.resources.context.borrow().logical_device),
          Rc::clone(&self.resources.descriptor_pool),
          hala_gfx::HalaDescriptorSetLayout::new(
            Rc::clone(&self.resources.context.borrow().logical_device),
            descriptor_bindings.as_slice(),
            &format!("{}.descriptor_set_layout", name),
          )?,
          0,
          &format!("{}.descriptor_set", name),
        )?;
        descriptor_sets.insert(name.clone(), descriptor_set);

        vec![&self.static_descriptor_set.layout, &dynamic_descriptor_set.layout, &texture_descriptor_set.layout, &descriptor_sets[name].layout]
      } else {
        vec![&self.static_descriptor_set.layout, &dynamic_descriptor_set.layout, &texture_descriptor_set.layout]
      };

      let program = if !desc.color_formats.is_empty() || desc.depth_format.is_some() {
        HalaGraphicsProgram::new(
          Rc::clone(&self.resources.context.borrow().logical_device),
          descriptor_set_layouts.as_slice(),
          hala_gfx::HalaPipelineCreateFlags::default(),
          &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
          &[] as &[hala_gfx::HalaVertexInputBindingDescription],
          &[hala_gfx::HalaDynamicState::VIEWPORT, hala_gfx::HalaDynamicState::SCISSOR],
          desc,
          Some(pipeline_cache),
          name,
        )?
      } else {
        HalaGraphicsProgram::with_swapchain(
          Rc::clone(&self.resources.context.borrow().logical_device),
          &self.resources.context.borrow().swapchain,
          descriptor_set_layouts.as_slice(),
          hala_gfx::HalaPipelineCreateFlags::default(),
          &[] as &[hala_gfx::HalaVertexInputAttributeDescription],
          &[] as &[hala_gfx::HalaVertexInputBindingDescription],
          &[hala_gfx::HalaDynamicState::VIEWPORT, hala_gfx::HalaDynamicState::SCISSOR],
          desc,
          Some(pipeline_cache),
          name,
        )?
      };

      programs.insert(name.clone(), program);
    }

    Ok((programs, descriptor_sets))
  }

  /// Create the compute program.
  /// param program_config: The program configure.
  /// param pipeline_cache: The pipeline cache.
  /// return: The result.
  fn create_compute_program(
    &self,
    program_config: &HashMap<String, HalaComputeProgramDesc>,
    pipeline_cache: &hala_gfx::HalaPipelineCache,
  ) -> ComputeProgramResult {
    let mut programs = HashMap::new();
    let mut descriptor_sets = HashMap::new();

    let dynamic_descriptor_set = self.dynamic_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the dynamic descriptor set.", None))?;
    let texture_descriptor_set = self.textures_descriptor_set.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the textures descriptor set.", None))?;

    for (name, desc) in program_config.iter() {
      let descriptor_bindings = desc.bindings.iter().enumerate().map(|(binding_index, binding_type)| {
        hala_gfx::HalaDescriptorSetLayoutBinding {
          binding_index: binding_index as u32,
          descriptor_type: *binding_type,
          descriptor_count: 1,
          stage_flags: hala_gfx::HalaShaderStageFlags::COMPUTE,
          binding_flags: hala_gfx::HalaDescriptorBindingFlags::PARTIALLY_BOUND
        }
      }).collect::<Vec<_>>();
      let descriptor_set_layouts = if !descriptor_bindings.is_empty() {
        let descriptor_set = hala_gfx::HalaDescriptorSet::new_static(
          Rc::clone(&self.resources.context.borrow().logical_device),
          Rc::clone(&self.resources.descriptor_pool),
          hala_gfx::HalaDescriptorSetLayout::new(
            Rc::clone(&self.resources.context.borrow().logical_device),
            descriptor_bindings.as_slice(),
            &format!("{}.descriptor_set_layout", name),
          )?,
          0,
          &format!("{}.descriptor_set", name),
        )?;
        descriptor_sets.insert(name.clone(), descriptor_set);

        vec![&self.static_descriptor_set.layout, &dynamic_descriptor_set.layout, &texture_descriptor_set.layout, &descriptor_sets[name].layout]
      } else {
        vec![&self.static_descriptor_set.layout, &dynamic_descriptor_set.layout, &texture_descriptor_set.layout]
      };

      let program = HalaComputeProgram::new(
        Rc::clone(&self.resources.context.borrow().logical_device),
        descriptor_set_layouts.as_slice(),
        desc,
        Some(pipeline_cache),
        name,
      )?;

      programs.insert(name.clone(), program);
    }

    Ok((programs, descriptor_sets))
  }

  /// Record the rendering command buffer.
  /// param index: The index of the current image.
  /// param graphics_command_buffers: The graphics command buffers.
  /// param compute_command_buffers: The compute command buffers.
  /// param ui_fn: The draw UI function.
  /// return: The result.
  pub(crate) fn record_command_buffer<F>(
    &self,
    index: usize,
    graphics_command_buffers: &hala_gfx::HalaCommandBufferSet,
    compute_command_buffers: &hala_gfx::HalaCommandBufferSet,
    ui_fn: F,
  ) -> Result<(), HalaRendererError>
    where F: FnOnce(usize, &hala_gfx::HalaCommandBufferSet) -> Result<(), hala_gfx::HalaGfxError>
  {
    let context = self.resources.context.borrow();
    let scene = self.scene_in_gpu.as_ref()
      .ok_or(HalaRendererError::new("Failed to find the scene in the GPU.", None))?;

    // Prepare the command buffer and timestamp.
    graphics_command_buffers.reset(index, false)?;
    graphics_command_buffers.begin(index, hala_gfx::HalaCommandBufferUsageFlags::empty())?;
    graphics_command_buffers.reset_query_pool(index, &context.timestamp_query_pool, (index * 2) as u32, 2);
    graphics_command_buffers.write_timestamp(index, hala_gfx::HalaPipelineStageFlags2::NONE, &context.timestamp_query_pool, (index * 2) as u32);

    if cfg!(debug_assertions) {
      graphics_command_buffers.begin_debug_label(index, "Draw", [1.0, 1.0, 1.0, 1.0]);
    }

    // If there is no frame, we need to clear the depth image.
    if self.statistics.total_gpu_frames == 0 {
      self.clear_depth_pass(index, graphics_command_buffers)?;
      self.depth_reduction_pass(index, graphics_command_buffers)?;
    }

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

    let is_debug_view = self.debug_settings.show_triangle || self.debug_settings.show_meshlet;
    if is_debug_view {
      self.setup_swapchain_begin_barriers(&context, index, graphics_command_buffers)?;

      graphics_command_buffers.begin_rendering_with_swapchain(
        index,
        &context.swapchain,
        (0, 0, context.gpu_req.width, context.gpu_req.height),
        Some([25.0 / 255.0, 118.0 / 255.0, 210.0 / 255.0, 1.0]),
        Some(0.0),
        Some(0),
      );

      if self.debug_settings.show_triangle {
        let triangle_visualization_program = self.graphics_programs.get("triangle_visualization")
          .ok_or(HalaRendererError::new("Failed to find the triangle visualization program.", None))?;
        let triangle_visualization_descriptor_set = self.graphics_descriptor_sets.get("triangle_visualization");
        self.draw_scene(
          index,
          graphics_command_buffers,
          true,
          triangle_visualization_program,
          triangle_visualization_descriptor_set,
        )?;
      } else if self.debug_settings.show_meshlet {
        let meshlet_visualization_program = self.graphics_programs.get("meshlet_visualization")
          .ok_or(HalaRendererError::new("Failed to find the meshlet visualization program.", None))?;
        let meshlet_visualization_descriptor_set = self.graphics_descriptor_sets.get("meshlet_visualization");
        self.draw_scene(
          index,
          graphics_command_buffers,
          true,
          meshlet_visualization_program,
          meshlet_visualization_descriptor_set,
        )?;
      }
    } else {
      if self.debug_settings.disable_culling {
        // Write the visibility to the visibility buffer without culling.
        self.no_culling_visibility_buffer_pass(index, graphics_command_buffers)?;
      } else if self.debug_settings.one_pass_culling {
        // Culling the invisible meshlets by the last frame's Hi-Z buffer.
        let one_pass_culling_program = self.graphics_programs.get("one_pass_culling")
          .ok_or(HalaRendererError::new("Failed to find the one pass culling program.", None))?;
        let one_pass_culling_descriptor_set = self.graphics_descriptor_sets.get("one_pass_culling");
        self.culling_pass(index, graphics_command_buffers, one_pass_culling_program, one_pass_culling_descriptor_set)?;
        self.depth_reduction_pass(index, graphics_command_buffers)?;
      } else {
        // Culling the invisible meshlets by the last frame's Hi-Z buffer.
        let pre_culling_program = self.graphics_programs.get("pre_culling")
          .ok_or(HalaRendererError::new("Failed to find the pre culling program.", None))?;
        let pre_culling_descriptor_set = self.graphics_descriptor_sets.get("pre_culling");
        self.culling_pass(index, graphics_command_buffers, pre_culling_program, pre_culling_descriptor_set)?;
        self.depth_reduction_pass(index, graphics_command_buffers)?;
        // Culling the truely invisible meshlets by the current frame's Hi-Z buffer.
        // Write the visibility to the visibility buffer.
        self.visibility_buffer_pass(index, graphics_command_buffers)?;
        self.depth_reduction_pass(index, graphics_command_buffers)?;
      }
      // Write the material type to the depth buffer.
      self.material_depth_pass(index, graphics_command_buffers)?;
      // Clear the indirect draw buffer.
      self.clear_indirect_buffer(index, graphics_command_buffers)?;
      // Classify the screen tiles by the material type.
      self.material_classification_pass(index, graphics_command_buffers, compute_command_buffers)?;
      // Write G-Buffer by tiles.
      self.material_tile_pass(index, graphics_command_buffers)?;

      self.setup_swapchain_begin_barriers(&context, index, graphics_command_buffers)?;

      graphics_command_buffers.begin_rendering_with_swapchain(
        index,
        &context.swapchain,
        (0, 0, context.gpu_req.width, context.gpu_req.height),
        Some([25.0 / 255.0, 118.0 / 255.0, 210.0 / 255.0, 1.0]),
        Some(0.0),
        Some(0),
      );

      if self.debug_settings.show_hiz {
        let hiz_visualization_program = self.graphics_programs.get("hiz_visualization")
          .ok_or(HalaRendererError::new("Failed to find the Hi-Z visualization program.", None))?;
        hiz_visualization_program.push_constants_f32(
          index,
          graphics_command_buffers,
          0,
          &[100f32]
        );
        self.draw_screen_quad(
          index,
          graphics_command_buffers,
          hiz_visualization_program,
          Some(self.hiz_descriptor_sets[self.debug_settings.hiz_level as usize].as_ref()),
        )?;
      } else if self.debug_settings.show_visibility {
        let id_buffer_visualization_program = self.graphics_programs.get("id_buffer_visualization")
          .ok_or(HalaRendererError::new("Failed to find the id buffer visualization program.", None))?;
        let id_buffer_visualization_descriptor_set = self.graphics_descriptor_sets.get("id_buffer_visualization");
        self.draw_screen_quad(
          index,
          graphics_command_buffers,
          id_buffer_visualization_program,
          id_buffer_visualization_descriptor_set,
        )?;
      } else if self.debug_settings.show_material_depth || self.debug_settings.show_albedo || self.debug_settings.show_normal {
        if self.debug_settings.show_albedo {
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
            ],
          );
        } else if self.debug_settings.show_normal {
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
                image: self.normal_image.raw,
                ..Default::default()
              },
            ],
          );
        }

        let attachment_to_screen_program = self.graphics_programs.get("attachment_to_screen")
          .ok_or(HalaRendererError::new("Failed to find the attachment to screen program.", None))?;
        let attachment_to_screen_descriptor_set = self.graphics_descriptor_sets.get("attachment_to_screen");
        // Scale the material depth value to visualize.
        let scale = GlobalConstants::CLASSIFY_DEPTH_RANGE as f32 / scene.materials.len() as f32;
        let scales = if self.debug_settings.show_material_depth {
          [scale, scale, scale, 1.0]
        } else {
          [1.0, 1.0, 1.0, 1.0]
        };
        attachment_to_screen_program.push_constants_f32(
          index,
          graphics_command_buffers,
          0,
          &scales,
        );
        self.draw_screen_quad(
          index,
          graphics_command_buffers,
          attachment_to_screen_program,
          attachment_to_screen_descriptor_set,
        )?;
      } else {
        self.lighting_pass(index, graphics_command_buffers)?;
      }
    }

    ui_fn(index, graphics_command_buffers)?;

    graphics_command_buffers.end_rendering(index);

    self.setup_swapchain_end_barriers(&context, index, graphics_command_buffers)?;

    if cfg!(debug_assertions) {
      graphics_command_buffers.end_debug_label(index);
    }

    graphics_command_buffers.write_timestamp(
      index,
      hala_gfx::HalaPipelineStageFlags2::ALL_COMMANDS,
      &context.timestamp_query_pool,
      (index * 2 + 1) as u32);
    graphics_command_buffers.end(index)?;

    Ok(())
  }

}