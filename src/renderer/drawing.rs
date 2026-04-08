use anyhow::Result;
use image::{Rgba, RgbaImage};

use super::color::rgba_from_hex;
use super::SnippetRenderer;

impl SnippetRenderer {
  pub(super) fn draw_circle(
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

  pub(super) fn draw_rounded_rect(
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

  pub(super) fn is_inside_rounded_rect(
    &self,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
  ) -> bool {
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

    for (cx, cy) in &corners {
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

  pub(super) fn draw_rounded_rect_top_only(
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

  pub(super) fn is_inside_rounded_rect_top_only(
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

    for (cx, cy) in &top_corners {
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
}
