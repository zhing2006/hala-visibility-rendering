use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct WindowConfig {
  pub width: u16,
  pub height: u16,
}

/// Validate the window configure.
/// param: config: the configure.
/// return: the result of the validation.
pub fn validate_window_config(config: &WindowConfig) -> Result<()> {
  if config.width == 0 {
    return Err(anyhow::anyhow!("The width is 0."));
  }
  if config.height == 0 {
    return Err(anyhow::anyhow!("The height is 0."));
  }
  Ok(())
}
