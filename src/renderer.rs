/* ~~/src/renderer.rs */

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, ImageEncoder, Rgba, RgbaImage};
use rand::Rng;

use crate::config::RenderConfig;
use crate::font::{load_font_with_fallback, FontManager};
use crate::syntax::{HighlightedLine, SyntaxHighlighter};
use crate::themes::{get_theme, Theme};

pub struct SnippetRenderer {
  theme: Theme,
  config: RenderConfig,
  highlighter: SyntaxHighlighter,
  font_manager: FontManager,
}

impl SnippetRenderer {
  pub fn new(theme_name: &str, config: RenderConfig) -> Result<Self> {
    let theme = get_theme(theme_name).ok_or_else(|| anyhow!("Unknown theme: {}", theme_name))?;

    let highlighter = SyntaxHighlighter::new();

    // Load font with fallback chain
    let font_size = config.get_scaled_font_size();
    let font_manager = load_font_with_fallback(font_size)?;

    Ok(Self {
      theme,
      config,
      highlighter,
      font_manager,
    })
  }

  pub fn render_snippet(&self, code: &str, language: &str) -> Result<String> {
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
    &self,
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

      if self.config.line_numbers {
        let line_num = format!("{:3} ", line_index + 1);
        let line_num_color = rgba_from_hex(&self.theme.comment.hex)?;
        x += self.draw_text(image, &line_num, x, y, font_size, line_num_color)?;
        x += (10.0 * self.config.export_size) as u32; // Add some spacing
      }
      for token in &line.tokens {
        let token_color = rgba_from_hex(&token.color.hex)?;
        x += self.draw_text(image, &token.text, x, y, font_size, token_color)?;
      }
    }
    Ok(())
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

  fn draw_circle(
    &self,
    image: &mut RgbaImage,
    x: i32,
    y: i32,
    radius: i32,
    color_hex: &str,
  ) -> Result<()> {
    let color = rgba_from_hex(color_hex)?;

    // Simple circle drawing algorithm
    for dy in -radius..=radius {
      for dx in -radius..=radius {
        if dx * dx + dy * dy <= radius * radius {
          let px = x + dx;
          let py = y + dy;
          if px >= 0 && py >= 0 && (px as u32) < image.width() && (py as u32) < image.height() {
            image.put_pixel(px as u32, py as u32, color);
          }
        }
      }
    }
    Ok(())
  }

  fn draw_rounded_rect(
    &self,
    image: &mut RgbaImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    radius: f32,
    color: Rgba<u8>,
  ) -> Result<()> {
    let scaled_radius = radius * self.config.export_size;

    // Clamp radius to not exceed half the smaller dimension
    let max_radius = (width.min(height) as f32 / 2.0).min(scaled_radius);

    for py in 0..height {
      for px in 0..width {
        let pixel_x = x + px as i32;
        let pixel_y = y + py as i32;

        // Check bounds
        if pixel_x < 0
          || pixel_y < 0
          || pixel_x >= image.width() as i32
          || pixel_y >= image.height() as i32
        {
          continue;
        }

        if self.is_inside_rounded_rect(
          px as f32,
          py as f32,
          width as f32,
          height as f32,
          max_radius,
        ) {
          image.put_pixel(pixel_x as u32, pixel_y as u32, color);
        }
      }
    }

    Ok(())
  }

  fn is_inside_rounded_rect(&self, x: f32, y: f32, width: f32, height: f32, radius: f32) -> bool {
    // Check if point is in the main rectangular area (excluding corners)
    if x >= radius && x <= width - radius {
      return true; // Inside horizontal strip
    }
    if y >= radius && y <= height - radius {
      return true; // Inside vertical strip
    }

    // Check corner regions
    let corners = [
      (radius, radius),                  // Top-left
      (width - radius, radius),          // Top-right
      (radius, height - radius),         // Bottom-left
      (width - radius, height - radius), // Bottom-right
    ];

    for (cx, cy) in corners.iter() {
      let dx = x - cx;
      let dy = y - cy;
      let distance_sq = dx * dx + dy * dy;

      // If we're in this corner's quadrant and within the circle
      if (x <= radius && y <= radius && cx == &radius && cy == &radius)
        || (x >= width - radius && y <= radius && cx == &(width - radius) && cy == &radius)
        || (x <= radius && y >= height - radius && cx == &radius && cy == &(height - radius))
        || (x >= width - radius
          && y >= height - radius
          && cx == &(width - radius)
          && cy == &(height - radius))
      {
        return distance_sq <= radius * radius;
      }
    }

    false
  }

  fn draw_rounded_rect_top_only(
    &self,
    image: &mut RgbaImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    radius: f32,
    color: Rgba<u8>,
  ) -> Result<()> {
    let scaled_radius = radius * self.config.export_size;

    // Clamp radius to not exceed half the smaller dimension
    let max_radius = (width.min(height) as f32 / 2.0).min(scaled_radius);

    for py in 0..height {
      for px in 0..width {
        let pixel_x = x + px as i32;
        let pixel_y = y + py as i32;

        // Check bounds
        if pixel_x < 0
          || pixel_y < 0
          || pixel_x >= image.width() as i32
          || pixel_y >= image.height() as i32
        {
          continue;
        }

        if self.is_inside_rounded_rect_top_only(
          px as f32,
          py as f32,
          width as f32,
          height as f32,
          max_radius,
        ) {
          image.put_pixel(pixel_x as u32, pixel_y as u32, color);
        }
      }
    }

    Ok(())
  }

  fn is_inside_rounded_rect_top_only(
    &self,
    x: f32,
    y: f32,
    width: f32,
    _height: f32,
    radius: f32,
  ) -> bool {
    // For title bar, we want rounded corners only at the top

    // Check if point is in the main rectangular area (excluding top corners)
    if x >= radius && x <= width - radius {
      return true; // Inside horizontal strip
    }
    if y >= radius {
      return true; // Inside lower area (no rounding at bottom)
    }

    // Check only top corner regions
    let top_corners = [
      (radius, radius),         // Top-left
      (width - radius, radius), // Top-right
    ];

    for (cx, cy) in top_corners.iter() {
      let dx = x - cx;
      let dy = y - cy;
      let distance_sq = dx * dx + dy * dy;

      // If we're in this corner's quadrant and within the circle
      if (x <= radius && y <= radius && cx == &radius && cy == &radius)
        || (x >= width - radius && y <= radius && cx == &(width - radius) && cy == &radius)
      {
        return distance_sq <= radius * radius;
      }
    }

    false
  }

  fn draw_gradient_backdrop(&self, image: &mut RgbaImage, width: u32, height: u32) -> Result<()> {
    let mut rng = rand::thread_rng();

    // Generate random gradient colors
    let color1 = self.generate_random_gradient_color(&mut rng);
    let color2 = self.generate_random_gradient_color(&mut rng);

    // Choose random gradient direction
    let gradient_type = rng.gen_range(0..4);

    for y in 0..height {
      for x in 0..width {
        let pixel_color = match gradient_type {
          0 => self.linear_gradient_horizontal(x, width, color1, color2),
          1 => self.linear_gradient_vertical(y, height, color1, color2),
          2 => self.radial_gradient(x, y, width, height, color1, color2),
          _ => self.diagonal_gradient(x, y, width, height, color1, color2),
        };

        // Apply noise effect if enabled
        let final_color = if self.config.noise_effect {
          self.apply_noise_effect(pixel_color, &mut rng)
        } else {
          pixel_color
        };

        image.put_pixel(x, y, final_color);
      }
    }

    Ok(())
  }

  fn generate_random_gradient_color(&self, rng: &mut impl Rng) -> Rgba<u8> {
    // Generate lighter colors that complement the theme
    let base_color = rgba_from_hex(&self.theme.background.hex).unwrap_or(Rgba([30, 30, 30, 255]));

    // Lighten the base color by adding a brightness boost
    let brightness_boost = 60.0; // Increase brightness
    let lightened_base = Rgba([
      ((base_color[0] as f32 + brightness_boost).clamp(0.0, 255.0)) as u8,
      ((base_color[1] as f32 + brightness_boost).clamp(0.0, 255.0)) as u8,
      ((base_color[2] as f32 + brightness_boost).clamp(0.0, 255.0)) as u8,
      255,
    ]);

    // Create variations of the lightened color with some randomization
    let variation_range = 30.0; // Reduced variation for smoother gradients
    let r = ((lightened_base[0] as f32 + rng.gen_range(-variation_range..variation_range))
      .clamp(0.0, 255.0)) as u8;
    let g = ((lightened_base[1] as f32 + rng.gen_range(-variation_range..variation_range))
      .clamp(0.0, 255.0)) as u8;
    let b = ((lightened_base[2] as f32 + rng.gen_range(-variation_range..variation_range))
      .clamp(0.0, 255.0)) as u8;

    Rgba([r, g, b, 255])
  }

  fn linear_gradient_horizontal(
    &self,
    x: u32,
    width: u32,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
  ) -> Rgba<u8> {
    let ratio = x as f32 / width as f32;
    self.interpolate_color(color1, color2, ratio)
  }

  fn linear_gradient_vertical(
    &self,
    y: u32,
    height: u32,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
  ) -> Rgba<u8> {
    let ratio = y as f32 / height as f32;
    self.interpolate_color(color1, color2, ratio)
  }

  fn radial_gradient(
    &self,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
  ) -> Rgba<u8> {
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let max_distance = ((width * width + height * height) as f32).sqrt() / 2.0;
    let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
    let ratio = (distance / max_distance).clamp(0.0, 1.0);
    self.interpolate_color(color1, color2, ratio)
  }

  fn diagonal_gradient(
    &self,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
  ) -> Rgba<u8> {
    let ratio = ((x as f32 / width as f32) + (y as f32 / height as f32)) / 2.0;
    self.interpolate_color(color1, color2, ratio)
  }

  fn interpolate_color(&self, color1: Rgba<u8>, color2: Rgba<u8>, ratio: f32) -> Rgba<u8> {
    let ratio = ratio.clamp(0.0, 1.0);
    let r = (color1[0] as f32 * (1.0 - ratio) + color2[0] as f32 * ratio) as u8;
    let g = (color1[1] as f32 * (1.0 - ratio) + color2[1] as f32 * ratio) as u8;
    let b = (color1[2] as f32 * (1.0 - ratio) + color2[2] as f32 * ratio) as u8;
    Rgba([r, g, b, 255])
  }

  fn apply_noise_effect(&self, color: Rgba<u8>, rng: &mut impl Rng) -> Rgba<u8> {
    let noise_strength = 15.0; // Adjust noise intensity
    let noise = rng.gen_range(-noise_strength..noise_strength);

    let r = ((color[0] as f32 + noise).clamp(0.0, 255.0)) as u8;
    let g = ((color[1] as f32 + noise).clamp(0.0, 255.0)) as u8;
    let b = ((color[2] as f32 + noise).clamp(0.0, 255.0)) as u8;

    Rgba([r, g, b, color[3]])
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

fn rgba_from_hex(hex: &str) -> Result<Rgba<u8>> {
  let hex = hex.trim_start_matches('#');
  if hex.len() != 6 {
    return Err(anyhow!("Invalid hex color format: {}", hex));
  }
  let r = u8::from_str_radix(&hex[0..2], 16)?;
  let g = u8::from_str_radix(&hex[2..4], 16)?;
  let b = u8::from_str_radix(&hex[4..6], 16)?;
  Ok(Rgba([r, g, b, 255]))
}

fn darken_color(hex: &str, factor: f32) -> Result<Rgba<u8>> {
  let base_color = rgba_from_hex(hex)?;
  let r = ((base_color[0] as f32) * (1.0 - factor)) as u8;
  let g = ((base_color[1] as f32) * (1.0 - factor)) as u8;
  let b = ((base_color[2] as f32) * (1.0 - factor)) as u8;
  Ok(Rgba([r, g, b, base_color[3]]))
}
