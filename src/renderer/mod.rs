/* ~~/src/renderer/mod.rs */

// third-party crates
use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose;
use image::{ImageBuffer, ImageEncoder, Rgba, RgbaImage};

// local modules
use crate::config::RenderConfig;
use crate::font::{FontManager, load_font_with_fallback};
use crate::layout::{ComplexTextRenderer, has_complex_script};
use crate::syntax::{HighlightedLine, SyntaxHighlighter};
use crate::themes::{Theme, get_theme};
mod color;
mod drawing;
mod gradient;

use self::color::{darken_color, rgba_from_hex};

pub struct SnippetRenderer {
  theme: Theme,
  config: RenderConfig,
  highlighter: SyntaxHighlighter,
  font_manager: FontManager,
  /// Complex text renderer for Thai, Arabic, and other scripts requiring shaping
  complex_renderer: Option<ComplexTextRenderer>,
}

impl SnippetRenderer {
  pub fn new(theme_name: &str, config: RenderConfig) -> Result<Self> {
    let theme = get_theme(theme_name).ok_or_else(|| anyhow!("Unknown theme: {}", theme_name))?;

    let highlighter = SyntaxHighlighter::new();

    // Load font with fallback chain
    let font_size = config.get_scaled_font_size();
    let font_manager = load_font_with_fallback(font_size)?;

    // Always initialize complex text renderer so system font fallback can shape
    // Thai/Arabic/Indic text even when custom Thai font paths are unavailable.
    let complex_renderer = Some(ComplexTextRenderer::new(font_size)?);

    Ok(Self {
      theme,
      config,
      highlighter,
      font_manager,
      complex_renderer,
    })
  }

  pub fn render_snippet(&mut self, code: &str, language: &str) -> Result<String> {
    let highlighted_lines = self.highlighter.highlight_code(code, language, &self.theme);
    let line_count = highlighted_lines.len() as u32;

    // Get base line height from font metrics (unscaled)
    let base_line_height = self.font_manager.get_line_height();

    // Apply line height multiplier but NOT export scaling yet (that's done in get_actual_height)
    let line_height = (base_line_height as f32 * self.config.line_height) as u32;
    let padding = self.config.padding; // Use unscaled padding

    // Calculate window controls height (unscaled)
    let window_controls_height = if self.config.window_controls {
      40 // Base window controls height
    } else {
      0
    };

    // Calculate content area height (unscaled)
    let content_height = if line_count > 0 {
      line_count * line_height
    } else {
      line_height // Minimum height for empty content
    };

    // Calculate panel dimensions (unscaled)
    let panel_height = content_height + (padding * 2) + window_controls_height;

    // Calculate final image dimensions with panel padding
    let final_width = self.config.get_actual_width() + (self.config.get_scaled_panel_padding() * 2);
    let final_height =
      self.config.get_actual_height(panel_height) + (self.config.get_scaled_panel_padding() * 2);

    // Create image with panel padding
    let mut image = ImageBuffer::new(final_width, final_height);

    // Draw gradient backdrop if enabled
    if self.config.gradient_backdrop {
      self.draw_gradient_backdrop(&mut image, final_width, final_height)?;
    } else {
      // Fill with solid background
      let bg_color = rgba_from_hex(&self.theme.background.hex)?;
      for pixel in image.pixels_mut() {
        *pixel = bg_color;
      }
    }

    // Create panel area (offset by panel padding)
    let panel_x = self.config.get_scaled_panel_padding();
    let panel_y = self.config.get_scaled_panel_padding();
    let panel_actual_width = self.config.get_actual_width();
    let panel_actual_height = self.config.get_actual_height(panel_height);

    // Draw panel background with rounded corners
    let panel_bg_color = rgba_from_hex(&self.theme.background.hex)?;
    self.draw_rounded_rect(
      &mut image,
      panel_x as i32,
      panel_y as i32,
      panel_actual_width,
      panel_actual_height,
      self.config.border_radius,
      panel_bg_color,
    )?;

    // Draw window frame if enabled (within the panel area)
    if self.config.window_controls {
      self.draw_window_frame(
        &mut image,
        panel_actual_width,
        panel_actual_height,
        padding,
        panel_x,
        panel_y,
      )?;
    }

    // Draw code content (within the panel area)
    // Note: draw_code_content now uses &mut self for complex renderer
    self.draw_code_content(
      &mut image,
      &highlighted_lines,
      padding,
      line_height,
      panel_x,
      panel_y,
    )?;

    // Convert to PNG and encode as base64
    let png_data = self.image_to_png_bytes(&image)?;
    let base64_data = general_purpose::STANDARD.encode(&png_data);

    Ok(format!("data:image/png;base64,{}", base64_data))
  }

  fn draw_window_frame(
    &self,
    image: &mut RgbaImage,
    width: u32,
    _height: u32,
    padding: u32,
    offset_x: u32,
    offset_y: u32,
  ) -> Result<()> {
    let frame_height = (40.0 * self.config.export_size) as u32;

    // Draw window title bar with rounded corners (only top corners)
    let title_bar_color = darken_color(&self.theme.background.hex, 0.1)?;
    self.draw_rounded_rect_top_only(
      image,
      offset_x as i32,
      offset_y as i32,
      width,
      frame_height,
      self.config.border_radius,
      title_bar_color,
    )?;

    // Draw window controls (circles)
    let control_radius = (6.0 * self.config.export_size) as i32;
    let control_y = offset_y as i32 + (frame_height / 2) as i32;
    let control_spacing = (20.0 * self.config.export_size) as i32;
    let start_x = offset_x as i32 + (padding / 2) as i32;
    self.draw_circle(image, start_x, control_y, control_radius, "#ff5f56")?;
    self.draw_circle(
      image,
      start_x + control_spacing,
      control_y,
      control_radius,
      "#ffbd2e",
    )?;
    self.draw_circle(
      image,
      start_x + control_spacing * 2,
      control_y,
      control_radius,
      "#27ca3f",
    )?;

    // Draw window title if provided
    if let Some(_title) = &self.config.window_title {
      // Title drawing would go here - simplified for now
    }
    Ok(())
  }

  fn draw_code_content(
    &mut self,
    image: &mut RgbaImage,
    highlighted_lines: &[HighlightedLine],
    _padding: u32,
    line_height: u32,
    offset_x: u32,
    offset_y: u32,
  ) -> Result<()> {
    let font_size = self.config.get_scaled_font_size();
    let scaled_padding = self.config.get_scaled_padding();
    let start_y = offset_y
      + if self.config.window_controls {
        scaled_padding + (40.0 * self.config.export_size) as u32
      } else {
        scaled_padding
      };

    // Use scaled line height for actual rendering
    let scaled_line_height = (line_height as f32 * self.config.export_size) as u32;

    for (line_index, line) in highlighted_lines.iter().enumerate() {
      let y = start_y + (line_index as u32 * scaled_line_height);
      let mut x = offset_x + scaled_padding;

      // Check if line contains complex scripts requiring shaping
      let line_text: String = line.tokens.iter().map(|t| t.text.as_str()).collect();
      let needs_shaping = has_complex_script(&line_text);

      // Draw line numbers (always use simple rendering)
      if self.config.line_numbers {
        let line_num = format!("{:3} ", line_index + 1);
        let line_num_color = rgba_from_hex(&self.theme.comment.hex)?;
        x += self.draw_text(image, &line_num, x, y, font_size, line_num_color)?;
        x += (10.0 * self.config.export_size) as u32; // Add some spacing
      }

      // Route to appropriate renderer based on content
      if needs_shaping && self.complex_renderer.is_some() {
        // Use cosmic-text for complex scripts
        self.render_complex_line(image, &line.tokens, x, y)?;
      } else {
        // Use fontdue for simple ASCII text (fast path)
        for token in &line.tokens {
          let token_color = rgba_from_hex(&token.color.hex)?;
          x += self.draw_text(image, &token.text, x, y, font_size, token_color)?;
        }
      }
    }
    Ok(())
  }

  /// Render a line with complex script support using cosmic-text
  fn render_complex_line(
    &mut self,
    image: &mut RgbaImage,
    tokens: &[crate::syntax::HighlightedToken],
    x: u32,
    y: u32,
  ) -> Result<u32> {
    if let Some(ref mut renderer) = self.complex_renderer {
      renderer.render_line(image, tokens, x, y)
    } else {
      // Fallback to simple rendering if complex renderer unavailable
      let mut current_x = x;
      let font_size = self.config.get_scaled_font_size();
      for token in tokens {
        let color = rgba_from_hex(&token.color.hex)?;
        current_x += self.draw_text(image, &token.text, current_x, y, font_size, color)?;
      }
      Ok(current_x - x)
    }
  }

  fn draw_text(
    &self,
    image: &mut RgbaImage,
    text: &str,
    x: u32,
    y: u32,
    _font_size: f32, // Now using font_manager's size
    color: Rgba<u8>,
  ) -> Result<u32> {
    let mut current_x = x as i32;
    // The y coordinate already represents the baseline position
    // No additional calculation needed - use it directly
    let baseline_y = y as i32;

    for ch in text.chars() {
      // Skip control characters that might cause tofu glyphs
      if ch.is_control() && ch != '\t' {
        continue;
      }
      let glyph = self.font_manager.render_glyph(ch);

      // Blend the glyph onto the image using the calculated baseline
      self
        .font_manager
        .blend_glyph(image, &glyph, current_x, baseline_y, color)?;

      // Advance the position for the next character
      current_x += glyph.advance_width as i32;
    }

    Ok((current_x - x as i32) as u32)
  }

  fn image_to_png_bytes(&self, image: &RgbaImage) -> Result<Vec<u8>> {
    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    encoder.write_image(
      image.as_raw(),
      image.width(),
      image.height(),
      image::ColorType::Rgba8,
    )?;
    Ok(png_data)
  }
}
