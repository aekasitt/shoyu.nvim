use anyhow::{anyhow, Result};
use image::Rgba;

pub(super) fn rgba_from_hex(hex: &str) -> Result<Rgba<u8>> {
  let hex = hex.trim_start_matches('#');
  if hex.len() != 6 {
    return Err(anyhow!("Invalid hex color format: {}", hex));
  }
  let r = u8::from_str_radix(&hex[0..2], 16)?;
  let g = u8::from_str_radix(&hex[2..4], 16)?;
  let b = u8::from_str_radix(&hex[4..6], 16)?;
  Ok(Rgba([r, g, b, 255]))
}

pub(super) fn darken_color(hex: &str, factor: f32) -> Result<Rgba<u8>> {
  let base_color = rgba_from_hex(hex)?;
  let r = ((base_color[0] as f32) * (1.0 - factor)) as u8;
  let g = ((base_color[1] as f32) * (1.0 - factor)) as u8;
  let b = ((base_color[2] as f32) * (1.0 - factor)) as u8;
  Ok(Rgba([r, g, b, base_color[3]]))
}
