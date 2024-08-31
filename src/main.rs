use anyhow::{
  Result,
  Context,
};

use clap::{arg, Command};

use hala_imgui::{
  HalaApplication,
  HalaImGui,
};

use hala_renderer::{
  scene,
  renderer::HalaRendererTrait,
  shader_cache::HalaShaderCache,
};

mod config;
mod renderer;

use renderer::{
  DebugSettings,
  VisRenderer,
};

/// The settings of the application.
#[derive(Debug, Default, Clone)]
pub(crate) struct MySettings {
  pub debug_settings: DebugSettings,
}

/// The application.
struct MyApplication {
  log_file: String,
  config: config::AppConfig,
  settings: MySettings,
  renderer: Option<VisRenderer>,
  imgui: Option<HalaImGui>,
}

impl Drop for MyApplication {
  fn drop(&mut self) {
    self.imgui = None;
  }
}

/// The implementation of the SDF renderer application.
impl MyApplication {

  pub fn new() -> Result<Self> {
    // Parse the command line arguments.
    let matches = cli().get_matches();
    let log_file = match matches.get_one::<String>("log") {
      Some(log_file) => log_file,
      None => "./logs/renderer.log"
    };
    let config_file = matches.get_one::<String>("config").with_context(|| "Failed to get the config file path.")?;

    // Load the configure.
    let config = config::load_app_config(config_file)?;
    log::debug!("Config: {:?}", config);
    config::validate_app_config(&config)?;

    // Create out directory.
    std::fs::create_dir_all("./out")
      .with_context(|| "Failed to create the output directory: ./out")?;

    Ok(Self {
      log_file: log_file.to_string(),
      config,
      settings: MySettings::default(),
      renderer: None,
      imgui: None,
    })
  }

}


/// The implementation of the application trait for the SDF renderer application.
impl HalaApplication for MyApplication {

  fn get_log_console_fmt(&self) -> &str {
    "{d(%H:%M:%S)} {h({l:<5})} {t:<20.20} - {m}{n}"
  }
  fn get_log_file_fmt(&self) -> &str {
    "{d(%Y-%m-%d %H:%M:%S)} {h({l:<5})} {f}:{L} - {m}{n}"
  }
  fn get_log_file(&self) -> &std::path::Path {
    std::path::Path::new(self.log_file.as_str())
  }
  fn get_log_file_size(&self) -> u64 {
    1024 * 1024 /* 1MB */
  }
  fn get_log_file_roller_count(&self) -> u32 {
    5
  }

  fn get_window_title(&self) -> &str {
    "Visibility Renderer"
  }
  fn get_window_size(&self) -> winit::dpi::PhysicalSize<u32> {
    winit::dpi::PhysicalSize::new(self.config.window.width as u32, self.config.window.height as u32)
  }

  fn get_imgui(&self) -> Option<&HalaImGui> {
    self.imgui.as_ref()
  }
  fn get_imgui_mut(&mut self) -> Option<&mut HalaImGui> {
    self.imgui.as_mut()
  }

  /// The before run function.
  /// param width: The width of the window.
  /// param height: The height of the window.
  /// param window: The window.
  /// return: The result.
  fn before_run(&mut self, _width: u32, _height: u32, window: &winit::window::Window) -> Result<()> {
    let now = std::time::Instant::now();
    let mut scene = scene::cpu::HalaScene::new(&self.config.scene_file)?;
    log::info!("Load scene used {}ms.", now.elapsed().as_millis());

    // Setup the renderer.
    let gpu_req = hala_gfx::HalaGPURequirements {
      width: self.config.window.width as u32,
      height: self.config.window.height as u32,
      version: (1, 3, 0),
      require_srgb_surface: true,
      require_mesh_shader: true,
      require_ray_tracing: false,
      require_10bits_output: false,
      is_low_latency: true,
      require_depth: true,
      require_printf_in_shader: cfg!(debug_assertions),
      require_depth_stencil_resolve: true,
      ..Default::default()
    };

    // Create the renderer.
    let mut renderer = VisRenderer::new(
      "Visibility Renderer",
      &gpu_req,
      window,
    )?;

    let features = ["HALA_VISIBILITY_RENDERING", "GLOBAL_MESHLETS"];
    let feature_folder_name = features.join("#");

    let shaders_dir = if cfg!(debug_assertions) {
      &format!("shaders/output/debug/hala-vis-renderer/{}", feature_folder_name)
    } else {
      &format!("shaders/output/release/hala-vis-renderer/{}", feature_folder_name)
    };
    HalaShaderCache::get_instance().borrow_mut().set_shader_dir(shaders_dir);

    renderer.set_scene(&mut scene)?;
    renderer.commit()?;
    renderer.load_gpu_programs(&self.config.programs_file)?;

    // Setup the imgui.
    self.imgui = Some(HalaImGui::new(
      std::rc::Rc::clone(&(*renderer.resources().context)),
      false,
    )?);

    self.renderer = Some(renderer);

    Ok(())
  }

  /// The after run function.
  fn after_run(&mut self) {
    if let Some(renderer) = &mut self.renderer.take() {
      renderer.wait_idle().expect("Failed to wait the renderer idle.");
      self.imgui = None;
    }
  }

  /// The update function.
  /// param delta_time: The delta time.
  /// return: The result.
  fn update(&mut self, delta_time: f64, width: u32, height: u32) -> Result<()> {
    if let Some(imgui) = self.imgui.as_mut() {
      imgui.begin_frame(
        delta_time,
        width,
        height,
        |ui| -> Result<()> {
          if let Some(renderer) = self.renderer.as_mut() {
            ui.window("Visibility Renderer")
              .collapsed(false, imgui::Condition::FirstUseEver)
              .position([10.0, 10.0], imgui::Condition::FirstUseEver)
              .always_auto_resize(true)
              .build(|| -> Result<()> {
                let mut is_debug_settings_changed = false;

                ui.text("Options:");
                ui.separator();
                let mut culling_index = if self.settings.debug_settings.disable_culling {
                  1
                } else if self.settings.debug_settings.one_pass_culling {
                  2
                } else {
                  0
                };
                is_debug_settings_changed |= ui.radio_button("Disable Culling", &mut culling_index, 1);
                is_debug_settings_changed |= ui.radio_button("One Pass Culling", &mut culling_index, 2);
                is_debug_settings_changed |= ui.radio_button("Two Pass Culling", &mut culling_index, 0);

                ui.text("Debug Views:");
                ui.separator();
                let mut debug_view_index = if self.settings.debug_settings.show_hiz {
                  1
                } else if self.settings.debug_settings.show_triangle {
                  2
                } else if self.settings.debug_settings.show_meshlet {
                  3
                } else if self.settings.debug_settings.show_visibility {
                  4
                } else if self.settings.debug_settings.show_material_depth {
                  5
                } else if self.settings.debug_settings.show_albedo {
                  6
                } else if self.settings.debug_settings.show_normal {
                  7
                } else {
                  0
                };

                is_debug_settings_changed |= ui.radio_button("None", &mut debug_view_index, 0);
                ui.same_line();
                is_debug_settings_changed |= ui.radio_button("Hi-Z", &mut debug_view_index, 1);

                is_debug_settings_changed |= ui.slider("Hi-Z Level", 0u32, 4u32, &mut self.settings.debug_settings.hiz_level);

                is_debug_settings_changed |= ui.radio_button("Triangle", &mut debug_view_index, 2);
                ui.same_line();
                is_debug_settings_changed |= ui.radio_button("Meshlet", &mut debug_view_index, 3);

                is_debug_settings_changed |= ui.radio_button("Visibility", &mut debug_view_index, 4);
                ui.same_line();
                is_debug_settings_changed |= ui.radio_button("Material Depth", &mut debug_view_index, 5);

                ui.text("Debug Tile Settings:");
                ui.separator();

                is_debug_settings_changed |= ui.radio_button("Albedo", &mut debug_view_index, 6);
                ui.same_line();
                is_debug_settings_changed |= ui.radio_button("Normal", &mut debug_view_index, 7);

                is_debug_settings_changed |= ui.slider("Grid Line", 0u32, 4u32, &mut self.settings.debug_settings.grid_line_width);

                if is_debug_settings_changed {
                  self.settings.debug_settings.disable_culling = culling_index == 1;
                  self.settings.debug_settings.one_pass_culling = culling_index == 2;

                  self.settings.debug_settings.show_hiz = debug_view_index == 1;
                  self.settings.debug_settings.show_triangle = debug_view_index == 2;
                  self.settings.debug_settings.show_meshlet = debug_view_index == 3;
                  self.settings.debug_settings.show_visibility = debug_view_index == 4;
                  self.settings.debug_settings.show_material_depth = debug_view_index == 5;
                  self.settings.debug_settings.show_albedo = debug_view_index == 6;
                  self.settings.debug_settings.show_normal = debug_view_index == 7;

                  renderer.update_debug_settings(self.settings.debug_settings)?;
                }

                ui.text("Debug Logs:");
                ui.separator();
                if ui.button("Pass 1 Culling Flags") {
                  renderer.debug_culling_flags()?;
                }
                ui.same_line();
                if ui.button("Culling Results") {
                  renderer.debug_culling_results()?;
                }

                if ui.button("Indirect Draw") {
                  renderer.debug_indirect_draw()?;
                }
                ui.same_line();
                if ui.button("Tile Index") {
                  renderer.debug_tile_index()?;
                }

                Ok(())
              }
            );
          }

          Ok(())
        }
      )?;
      imgui.end_frame()?;
    }

    if let Some(renderer) = &mut self.renderer {
      renderer.update(
        delta_time,
        width,
        height,
        |index, command_buffers| {
          if let Some(imgui) = self.imgui.as_mut() {
            imgui.draw(index, command_buffers)?;
          }

          Ok(())
        }
      )?;
    }

    Ok(())
  }

  /// The render function.
  /// return: The result.
  fn render(&mut self) -> Result<()> {
    if let Some(renderer) = &mut self.renderer {
      renderer.render()?;
    }

    Ok(())
  }

}


/// The command line interface.
fn cli() -> Command {
  Command::new("hala-subpasses")
    .about("The Deferred Renderer.")
    .arg_required_else_help(true)
    .arg(arg!(-l --log <LOG_FILE> "The file path of the log file. Default is ./logs/renderer.log."))
    .arg(arg!(-c --config [CONFIG_FILE] "The file path of the config file."))
}

/// The normal main function.
fn main() -> Result<()> {
  // Initialize the application.
  let mut app = MyApplication::new()?;
  app.init()?;

  // Run the application.
  app.run()?;

  Ok(())
}
