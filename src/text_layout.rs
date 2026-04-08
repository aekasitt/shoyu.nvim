/* ~~/src/text_layout.rs */

use anyhow::Result;
use cosmic_text::fontdb;
use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use image::{Rgba, RgbaImage};

use crate::font::THAI_FONT_PATHS;
use crate::syntax::HighlightedToken;

/// Check if a line contains complex scripts requiring shaping
/// Returns true for Thai, Arabic, Indic scripts, CJK, and other
/// scripts that require OpenType shaping
pub fn has_complex_script(text: &str) -> bool {
  text.chars().any(|ch| {
    let code_point = ch as u32;
    // Thai: U+0E00–U+0E7F
    if (0x0E00..=0x0E7F).contains(&code_point) {
      return true;
    }
    // Arabic: U+0600–U+06FF, U+0750–U+077F, U+08A0–U+08FF, U+FB50–U+FDFF, U+FE70–U+FEFF
    if (0x0600..=0x06FF).contains(&code_point)
      || (0x0750..=0x077F).contains(&code_point)
      || (0x08A0..=0x08FF).contains(&code_point)
      || (0xFB50..=0xFDFF).contains(&code_point)
      || (0xFE70..=0xFEFF).contains(&code_point)
    {
      return true;
    }
    // Devanagari and other Indic scripts: U+0900–U+097F (Devanagari), plus extended ranges
    if (0x0900..=0x097F).contains(&code_point)
      || (0x0980..=0x09FF).contains(&code_point) // Bengali
      || (0x0A00..=0x0A7F).contains(&code_point) // Gurmukhi
      || (0x0A80..=0x0AFF).contains(&code_point) // Gujarati
      || (0x0B00..=0x0B7F).contains(&code_point) // Oriya
      || (0x0B80..=0x0BFF).contains(&code_point) // Tamil
      || (0x0C00..=0x0C7F).contains(&code_point) // Telugu
      || (0x0C80..=0x0CFF).contains(&code_point) // Kannada
      || (0x0D00..=0x0D7F).contains(&code_point)
    // Malayalam
    {
      return true;
    }
    // CJK Unified Ideographs: U+4E00–U+9FFF
    if (0x4E00..=0x9FFF).contains(&code_point) {
      return true;
    }
    // CJK Extensions
    if (0x3400..=0x4DBF).contains(&code_point)
      || (0x20000..=0x2A6DF).contains(&code_point)
      || (0x2A700..=0x2B73F).contains(&code_point)
      || (0x2B740..=0x2B81F).contains(&code_point)
    {
      return true;
    }
    false
  })
}

/// Renderer for complex text using cosmic-text
/// Handles Thai, Arabic, Indic scripts with proper shaping
pub struct ComplexTextRenderer {
  font_system: FontSystem,
  swash_cache: SwashCache,
  metrics: Metrics,
}

impl ComplexTextRenderer {
  /// Create a new complex text renderer with the given font size
  /// Explicitly loads Thai fonts for proper fallback support
  pub fn new(font_size: f32) -> Result<Self> {
    // Load Thai fonts explicitly into the font system
    let thai_fonts = load_thai_fonts();
    let font_system = FontSystem::new_with_fonts(thai_fonts);
    let swash_cache = SwashCache::new();
    let metrics = Metrics::new(font_size, font_size * 1.2);

    Ok(Self {
      font_system,
      swash_cache,
      metrics,
    })
  }

  /// Get line height for layout calculations
  pub fn get_line_height(&self) -> u32 {
    self.metrics.line_height as u32
  }

  /// Render a line of highlighted tokens with complex script support
  /// Returns the total width consumed
  pub fn render_line(
    &mut self,
    image: &mut RgbaImage,
    tokens: &[HighlightedToken],
    x: u32,
    y: u32,
    color: Rgba<u8>,
  ) -> Result<u32> {
    let mut buffer = Buffer::new(&mut self.font_system, self.metrics);

    // Build the text and track token boundaries for color application
    let mut line_text = String::new();

    for token in tokens {
      line_text.push_str(&token.text);
    }

    // Set the text in the buffer with default attributes
    let attrs = Attrs::new();
    buffer.set_text(&mut self.font_system, &line_text, attrs, Shaping::Advanced);

    // Shape the layout
    buffer.shape_until_scroll(&mut self.font_system, true);

    // Convert Rgba<u8> to cosmic-text Color
    let text_color = Color::rgba(color[0], color[1], color[2], color[3]);

    // Render the line
    let mut current_x = x as f32;
    let baseline_y = y as f32;

    for run in buffer.layout_runs() {
      for layout_glyph in run.glyphs.iter() {
        // Construct cache key from layout glyph info
        // CacheKey::new returns (CacheKey, i32, i32) for subpixel positioning
        let (cache_key, _x_offset, _y_offset) = cosmic_text::CacheKey::new(
          layout_glyph.font_id,
          layout_glyph.glyph_id,
          self.metrics.font_size,
          (layout_glyph.x, layout_glyph.y),
          cosmic_text::CacheKeyFlags::empty(),
        );

        // Get the physical glyph from swash cache
        // get_image returns &Option<SwashImage>
        // Clone the glyph data to avoid borrow checker issues
        let glyph_opt: Option<cosmic_text::SwashImage> = self
          .swash_cache
          .get_image(&mut self.font_system, cache_key)
          .as_ref()
          .map(|g| cosmic_text::SwashImage {
            source: g.source,
            content: g.content,
            data: g.data.clone(),
            placement: g.placement,
          });

        if let Some(glyph) = glyph_opt {
          let glyph_x = current_x + layout_glyph.x;
          let glyph_y = baseline_y + layout_glyph.y;

          // Render the glyph with the specified color
          self.render_glyph_to_image(image, &glyph, glyph_x, glyph_y, text_color)?;
        }
        current_x += layout_glyph.w;
      }
    }

    Ok((current_x - x as f32).ceil() as u32)
  }

  /// Render a cached glyph image to the target image
  fn render_glyph_to_image(
    &self,
    image: &mut RgbaImage,
    glyph: &cosmic_text::SwashImage,
    x: f32,
    y: f32,
    color: Color,
  ) -> Result<()> {
    let img_width = image.width() as i32;
    let img_height = image.height() as i32;

    let width = glyph.placement.width as i32;
    let height = glyph.placement.height as i32;
    let offset_x = glyph.placement.left as i32;
    let offset_y = -glyph.placement.top as i32; // Negate because top is negative for upward offset

    // Get color components from cosmic-text Color
    let (fg_r, fg_g, fg_b, fg_a) = (color.r(), color.g(), color.b(), color.a());

    // Handle different content types
    match &glyph.content {
      cosmic_text::SwashContent::Mask => {
        // Grayscale mask - data is 1 byte per pixel
        let data = &glyph.data;
        for row in 0..height {
          for col in 0..width {
            let alpha = data[(row * width + col) as usize];
            if alpha == 0 {
              continue;
            }

            let pixel_x = x as i32 + col + offset_x;
            let pixel_y = y as i32 + row + offset_y;

            if pixel_x < 0 || pixel_x >= img_width || pixel_y < 0 || pixel_y >= img_height {
              continue;
            }

            // Blend the glyph alpha with the foreground color alpha
            let blended_alpha = ((alpha as u16 * fg_a as u16) / 255) as u8;

            let existing = image.get_pixel(pixel_x as u32, pixel_y as u32);
            let blended =
              blend_alpha_pixel(*existing, Rgba([fg_r, fg_g, fg_b, 255]), blended_alpha);
            image.put_pixel(pixel_x as u32, pixel_y as u32, blended);
          }
        }
      }
      cosmic_text::SwashContent::Color => {
        // Color data - 4 bytes per pixel (RGBA)
        let data = &glyph.data;
        for row in 0..height {
          for col in 0..width {
            let i = (row * width + col) as usize * 4;
            let alpha = data[i + 3];
            if alpha == 0 {
              continue;
            }

            let pixel_x = x as i32 + col + offset_x;
            let pixel_y = y as i32 + row + offset_y;

            if pixel_x < 0 || pixel_x >= img_width || pixel_y < 0 || pixel_y >= img_height {
              continue;
            }

            // For color glyphs, use the glyph's color blended with the requested color
            // or just use the glyph color directly for emoji
            let existing = image.get_pixel(pixel_x as u32, pixel_y as u32);
            let blended = if fg_r == 255 && fg_g == 255 && fg_b == 255 {
              // If white text color, use the glyph's actual color (for emoji)
              blend_alpha_pixel(
                *existing,
                Rgba([data[i], data[i + 1], data[i + 2], 255]),
                alpha,
              )
            } else {
              // Otherwise tint with the requested color
              let tinted_alpha = ((alpha as u16 * fg_a as u16) / 255) as u8;
              blend_alpha_pixel(*existing, Rgba([fg_r, fg_g, fg_b, 255]), tinted_alpha)
            };
            image.put_pixel(pixel_x as u32, pixel_y as u32, blended);
          }
        }
      }
      cosmic_text::SwashContent::SubpixelMask => {
        // Subpixel mask - more complex, skip for now
        // Could implement proper subpixel blending in future
      }
    }

    Ok(())
  }
}

/// Blend a pixel with foreground color using alpha
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
    255,
  ])
}

/// Load Thai fonts for cosmic-text FontSystem
/// Returns a vector of fontdb::Source for all available Thai fonts
fn load_thai_fonts() -> Vec<fontdb::Source> {
  let mut sources = Vec::new();

  for path in THAI_FONT_PATHS {
    if std::path::Path::new(path).exists() {
      if let Ok(data) = std::fs::read(path) {
        sources.push(fontdb::Source::Binary(std::sync::Arc::new(data)));
      }
    }
  }

  sources
}
