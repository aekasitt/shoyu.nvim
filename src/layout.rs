/* ~~/src/layout.rs */

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
    // Start with system fonts, then add explicit Thai font files when available.
    // This avoids disabling shaping on platforms where Thai fonts live in
    // different directories than our hardcoded fallback paths.
    let mut font_system = FontSystem::new();
    for source in load_thai_fonts() {
      font_system.db_mut().load_font_source(source);
    }
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
  ) -> Result<u32> {
    let mut buffer = Buffer::new(&mut self.font_system, self.metrics);

    // Build rich text spans so syntax token colors are preserved.
    let mut spans = Vec::with_capacity(tokens.len());
    let mut has_text = false;

    for token in tokens {
      if token.text.is_empty() {
        continue;
      }
      has_text = true;
      let (r, g, b) = token.color.rgb;
      let attrs = Attrs::new().color(Color::rgba(r, g, b, 255));
      spans.push((token.text.as_str(), attrs));
    }

    if !has_text {
      return Ok(0);
    }

    // Shape with per-token attributes so comments/keywords keep their colors.
    let default_attrs = Attrs::new();
    buffer.set_rich_text(
      &mut self.font_system,
      spans,
      default_attrs,
      Shaping::Advanced,
    );

    // Shape the layout
    buffer.shape_until_scroll(&mut self.font_system, true);

    let img_width = image.width() as i32;
    let img_height = image.height() as i32;
    let mut max_width = 0.0f32;

    for run in buffer.layout_runs() {
      let baseline_offset_y = y as f32 - run.line_y;
      let line_y = run.line_y as i32;
      max_width = max_width.max(run.line_w);

      for layout_glyph in run.glyphs.iter() {
        // Use physical glyph positioning from cosmic-text to keep combining
        // marks anchored to the base glyph instead of drifting into next cells.
        let physical_glyph = layout_glyph.physical((x as f32, baseline_offset_y), 1.0);
        let glyph_color = layout_glyph
          .color_opt
          .unwrap_or(Color::rgba(255, 255, 255, 255));

        self.swash_cache.with_pixels(
          &mut self.font_system,
          physical_glyph.cache_key,
          glyph_color,
          |px, py, color| {
            let pixel_x = physical_glyph.x + px;
            let pixel_y = line_y + physical_glyph.y + py;

            if pixel_x < 0 || pixel_x >= img_width || pixel_y < 0 || pixel_y >= img_height {
              return;
            }

            let existing = image.get_pixel(pixel_x as u32, pixel_y as u32);
            let blended = blend_color_pixel(*existing, color);
            image.put_pixel(pixel_x as u32, pixel_y as u32, blended);
          },
        );
      }
    }

    Ok(max_width.ceil() as u32)
  }
}

/// Blend a pixel with rendered glyph color using alpha
fn blend_color_pixel(background: Rgba<u8>, foreground: Color) -> Rgba<u8> {
  let alpha = foreground.a();
  if alpha == 255 {
    return Rgba([foreground.r(), foreground.g(), foreground.b(), 255]);
  }
  if alpha == 0 {
    return background;
  }

  let alpha_f = alpha as f32 / 255.0;
  let inv_alpha = 1.0 - alpha_f;

  Rgba([
    (foreground.r() as f32 * alpha_f + background[0] as f32 * inv_alpha) as u8,
    (foreground.g() as f32 * alpha_f + background[1] as f32 * inv_alpha) as u8,
    (foreground.b() as f32 * alpha_f + background[2] as f32 * inv_alpha) as u8,
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
