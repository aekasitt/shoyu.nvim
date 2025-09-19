/* ~~/src/font.rs */

use anyhow::{anyhow, Result};
use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};

pub struct FontManager {
  font: Font,
  size: f32,
}

pub struct GlyphInfo {
  pub data: Vec<u8>,
  pub width: usize,
  pub height: usize,
  pub advance_width: f32,
  pub bearing_x: i32,
  pub bearing_y: i32,
}

impl FontManager {
  pub fn new(font_data: &[u8], size: f32) -> Result<Self> {
    let font = Font::from_bytes(font_data, FontSettings::default())
      .map_err(|e| anyhow!("Failed to load font: {}", e))?;

    Ok(Self { font, size })
  }

  pub fn render_glyph(&self, character: char) -> GlyphInfo {
    let (metrics, bitmap) = self.font.rasterize(character, self.size);

    GlyphInfo {
      data: bitmap,
      width: metrics.width,
      height: metrics.height,
      advance_width: metrics.advance_width,
      bearing_x: metrics.xmin,
      bearing_y: metrics.ymin,
    }
  }

  pub fn get_line_height(&self) -> u32 {
    // For optimal code rendering, use a simple but effective approach
    // Most code editors use font size * 0.9 to 1.0 as the base line height
    // This gives tight but readable spacing that works well with multipliers

    if let Some(metrics) = self.font.horizontal_line_metrics(self.size) {
      // Use font metrics but cap it to prevent excessive spacing
      let metrics_height = metrics.ascent - metrics.descent;
      let font_size_height = self.size * 0.95; // Slightly tighter than font size

      // Use the smaller of the two to ensure tight baseline
      metrics_height.min(font_size_height).ceil() as u32
    } else {
      // Fallback: use 95% of font size for tight baseline
      (self.size * 0.95).ceil() as u32
    }
  }

  pub fn blend_glyph(
    &self,
    image: &mut RgbaImage,
    glyph: &GlyphInfo,
    x: i32,
    y: i32,
    color: Rgba<u8>,
  ) -> Result<()> {
    let img_width = image.width() as i32;
    let img_height = image.height() as i32;

    // Calculate the actual position considering bearing
    let glyph_x = x + glyph.bearing_x;

    // For proper baseline alignment, we need to position the glyph so that
    // its baseline aligns with the input y coordinate
    // bearing_y is the distance from baseline to bottom of glyph (negative for descenders)
    // We want: glyph_top_y = baseline_y - (glyph_height + bearing_y)
    let glyph_y = y - (glyph.height as i32 + glyph.bearing_y);

    for (i, &alpha) in glyph.data.iter().enumerate() {
      if alpha == 0 {
        continue;
      }

      let pixel_x = glyph_x + (i % glyph.width) as i32;
      let pixel_y = glyph_y + (i / glyph.width) as i32;

      // Bounds checking
      if pixel_x < 0 || pixel_x >= img_width || pixel_y < 0 || pixel_y >= img_height {
        continue;
      }

      let existing = image.get_pixel(pixel_x as u32, pixel_y as u32);
      let blended = blend_alpha_pixel(*existing, color, alpha);
      image.put_pixel(pixel_x as u32, pixel_y as u32, blended);
    }

    Ok(())
  }
}

fn blend_alpha_pixel(background: Rgba<u8>, foreground: Rgba<u8>, alpha: u8) -> Rgba<u8> {
  if alpha == 255 {
    return foreground;
  }
  if alpha == 0 {
    return background;
  }

  let alpha_f = alpha as f32 / 255.0;
  let inv_alpha = 1.0 - alpha_f;

  Rgba([
    (foreground[0] as f32 * alpha_f + background[0] as f32 * inv_alpha) as u8,
    (foreground[1] as f32 * alpha_f + background[1] as f32 * inv_alpha) as u8,
    (foreground[2] as f32 * alpha_f + background[2] as f32 * inv_alpha) as u8,
    255, // Keep full opacity for the result
  ])
}

/// Try to load font from various sources
pub fn load_font_with_fallback(preferred_size: f32) -> Result<FontManager> {
  let font_paths = [
    "./fonts/jet-brains-mono-regular.ttf",
    "./fonts/fira-code-regular.ttf",
    "/System/Library/Fonts/Monaco.ttf", // macOS fallback
    "/System/Library/Fonts/Menlo.ttc",  // macOS fallback
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", // Linux fallback
    "/Windows/Fonts/consola.ttf",       // Windows fallback
  ];

  // Try external fonts first
  for path in &font_paths {
    if std::path::Path::new(path).exists() {
      if let Ok(font_data) = std::fs::read(path) {
        if let Ok(font_manager) = FontManager::new(&font_data, preferred_size) {
          return Ok(font_manager);
        }
      }
    }
  }

  // Fallback to minimal embedded font data
  // For now, we'll create a very basic implementation
  create_fallback_font(preferred_size)
}

fn create_fallback_font(_size: f32) -> Result<FontManager> {
  // This is a placeholder for when no real font is available
  // In a production implementation, you'd embed a real TTF font here
  Err(anyhow!(
    "No suitable font found. Please place a TTF font in assets/fonts/ directory.\n\
         Recommended: JetBrains Mono, Fira Code, or any monospace programming font."
  ))
}
