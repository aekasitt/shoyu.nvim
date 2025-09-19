/* ~~/src/config.rs */

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
  pub width: u32,
  pub height: Option<u32>,
  pub padding: u32,
  pub line_height: f32,
  pub font_size: f32,
  pub font_family: String,
  pub background_color: String,
  pub window_controls: bool,
  pub window_title: Option<String>,
  pub line_numbers: bool,
  pub drop_shadow: bool,
  pub border_radius: f32,
  pub export_size: f32,        // Scale factor for high-res export
  pub panel_padding: u32,      // Extra padding around the entire panel
  pub gradient_backdrop: bool, // Enable randomized gradient backdrop
  pub noise_effect: bool,      // Enable noise effect on gradient
}

impl Default for RenderConfig {
  fn default() -> Self {
    Self {
      width: 1200,
      height: None, // Auto-calculate based on content
      padding: 64,
      line_height: 1.25, // Fine-tuned for optimized base line height calculation
      font_size: 18.0,
      font_family: String::from("Fira Code"),
      background_color: String::from("#1e1e1e"),
      window_controls: true,
      window_title: None,
      line_numbers: false,
      drop_shadow: true,
      border_radius: 8.0,
      export_size: 2.0,        // 2x for retina displays
      panel_padding: 80,       // Extra padding around the panel
      gradient_backdrop: true, // Enable gradient backdrop by default
      noise_effect: true,      // Enable noise effect by default
    }
  }
}

impl RenderConfig {
  pub fn get_actual_width(&self) -> u32 {
    (self.width as f32 * self.export_size) as u32
  }

  pub fn get_actual_height(&self, total_height: u32) -> u32 {
    let height = self.height.unwrap_or(total_height);
    (height as f32 * self.export_size) as u32
  }

  pub fn get_scaled_padding(&self) -> u32 {
    (self.padding as f32 * self.export_size) as u32
  }

  pub fn get_scaled_font_size(&self) -> f32 {
    self.font_size * self.export_size
  }

  pub fn get_scaled_panel_padding(&self) -> u32 {
    (self.panel_padding as f32 * self.export_size) as u32
  }
}
