/* ~~/src/renderer/gradient.rs */

// third-party crates
use anyhow::Result;
use image::{Rgba, RgbaImage};
use rand::Rng;

// local modules
use crate::renderer::SnippetRenderer;
use crate::renderer::color::rgba_from_hex;

impl SnippetRenderer {
  pub(super) fn draw_gradient_backdrop(
    &self,
    image: &mut RgbaImage,
    width: u32,
    height: u32,
  ) -> Result<()> {
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

  pub(super) fn generate_random_gradient_color(&self, rng: &mut impl Rng) -> Rgba<u8> {
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

  pub(super) fn linear_gradient_horizontal(
    &self,
    x: u32,
    width: u32,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
  ) -> Rgba<u8> {
    let ratio = x as f32 / width as f32;
    self.interpolate_color(color1, color2, ratio)
  }

  pub(super) fn linear_gradient_vertical(
    &self,
    y: u32,
    height: u32,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
  ) -> Rgba<u8> {
    let ratio = y as f32 / height as f32;
    self.interpolate_color(color1, color2, ratio)
  }

  pub(super) fn radial_gradient(
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

  pub(super) fn diagonal_gradient(
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

  pub(super) fn interpolate_color(
    &self,
    color1: Rgba<u8>,
    color2: Rgba<u8>,
    ratio: f32,
  ) -> Rgba<u8> {
    let ratio = ratio.clamp(0.0, 1.0);
    let r = (color1[0] as f32 * (1.0 - ratio) + color2[0] as f32 * ratio) as u8;
    let g = (color1[1] as f32 * (1.0 - ratio) + color2[1] as f32 * ratio) as u8;
    let b = (color1[2] as f32 * (1.0 - ratio) + color2[2] as f32 * ratio) as u8;
    Rgba([r, g, b, 255])
  }

  pub(super) fn apply_noise_effect(&self, color: Rgba<u8>, rng: &mut impl Rng) -> Rgba<u8> {
    let noise_strength = 15.0; // Adjust noise intensity
    let noise = rng.gen_range(-noise_strength..noise_strength);
    let r = ((color[0] as f32 + noise).clamp(0.0, 255.0)) as u8;
    let g = ((color[1] as f32 + noise).clamp(0.0, 255.0)) as u8;
    let b = ((color[2] as f32 + noise).clamp(0.0, 255.0)) as u8;
    Rgba([r, g, b, color[3]])
  }
}
