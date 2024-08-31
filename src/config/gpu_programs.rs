use std::path::Path;
use std::collections::HashMap;

use serde::Deserialize;

use anyhow::{Result, Context};

use hala_renderer::prelude::*;

/// The GPU programs configure.
#[derive(Deserialize)]
pub struct GPUProgramsConfig {
  #[serde(default)]
  pub compute_programs: HashMap<String, HalaComputeProgramDesc>,
  #[serde(default)]
  pub graphics_programs: HashMap<String, HalaGraphicsProgramDesc>,
}

/// The GPU programs configure implementation.
impl GPUProgramsConfig {

  /// Load the GPU programs configure.
  /// param: config_file: the configure file path.
  /// return: the GPU programs configure.
  pub fn load<P: AsRef<Path>>(config_path: P) -> Result<Self> {
    let path = config_path.as_ref();
    let config_str = std::fs::read_to_string(path)
      .with_context(|| format!("Failed to read the config file: {:?}", path))?;
    let config: Self = toml::from_str(&config_str)
      .with_context(|| format!("Failed to parse the config file: {:?}", path))?;
    Ok(config)
  }

}