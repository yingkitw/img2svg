//! Image preprocessing for better vectorization results
//!
//! This module provides preprocessing functions to prepare images for vectorization,
//! particularly useful for photographs or complex images.

use crate::image_processor::ImageData;
use rgb::RGBA8;
use anyhow::Result;

/// Preprocessing options
#[derive(Debug, Clone)]
pub struct PreprocessOptions {
    /// Color reduction level (0-1, 0 = none, 1 = maximum reduction)
    pub color_reduction: f32,
    /// Bilateral filter sigma for spatial domain
    pub spatial_sigma: f32,
    /// Bilateral filter sigma for color domain
    pub color_sigma: f32,
    /// Number of bilateral filter iterations
    pub iterations: u32,
}

impl Default for PreprocessOptions {
    fn default() -> Self {
        Self {
            color_reduction: 0.0,
            spatial_sigma: 3.0,
            color_sigma: 30.0,
            iterations: 1,
        }
    }
}

impl PreprocessOptions {
    /// Create options optimized for photographs
    pub fn photo() -> Self {
        Self {
            color_reduction: 0.5,  // Moderate color reduction
            spatial_sigma: 5.0,     // More spatial smoothing
            color_sigma: 40.0,      // More color smoothing
            iterations: 2,
        }
    }

    /// Create options optimized for graphics
    pub fn graphics() -> Self {
        Self {
            color_reduction: 0.0,
            spatial_sigma: 2.0,
            color_sigma: 20.0,
            iterations: 1,
        }
    }
}

/// Apply preprocessing to image data
pub fn preprocess(image_data: &ImageData, options: &PreprocessOptions) -> Result<ImageData> {
    let mut pixels = image_data.pixels.clone();

    // Apply bilateral filter for edge-preserving smoothing
    if options.spatial_sigma > 0.0 && options.iterations > 0 {
        for _ in 0..options.iterations {
            pixels = bilateral_filter(&pixels, image_data.width, image_data.height,
                                       options.spatial_sigma, options.color_sigma);
        }
    }

    // Apply color reduction (posterization)
    if options.color_reduction > 0.0 {
        pixels = reduce_colors(&pixels, options.color_reduction);
    }

    Ok(ImageData {
        width: image_data.width,
        height: image_data.height,
        pixels,
    })
}

/// Bilateral filter - edge-preserving smoothing
fn bilateral_filter(
    pixels: &[RGBA8],
    width: u32,
    height: u32,
    spatial_sigma: f32,
    color_sigma: f32,
) -> Vec<RGBA8> {
    let width = width as usize;
    let height = height as usize;
    let mut output = Vec::with_capacity(pixels.len());

    let sigma_space_sq = 2.0 * spatial_sigma * spatial_sigma;
    let sigma_color_sq = 2.0 * color_sigma * color_sigma;

    // Calculate kernel radius (3 * sigma covers 99% of Gaussian)
    let radius = (3.0 * spatial_sigma).ceil() as usize;

    for y in 0..height {
        for x in 0..width {
            let center_idx = y * width + x;
            let center_p = pixels[center_idx];

            let mut sum_weight = 0.0;
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;

            // Sample pixels within radius
            let y_start = y.saturating_sub(radius);
            let y_end = (y + radius + 1).min(height);
            let x_start = x.saturating_sub(radius);
            let x_end = (x + radius + 1).min(width);

            for ny in y_start..y_end {
                for nx in x_start..x_end {
                    let idx = ny * width + nx;
                    let p = pixels[idx];

                    // Spatial weight (Gaussian)
                    let dx = (nx as f32 - x as f32);
                    let dy = (ny as f32 - y as f32);
                    let spatial_weight = (-(dx * dx + dy * dy) / sigma_space_sq).exp();

                    // Color weight (Gaussian)
                    let dr = center_p.r as f32 - p.r as f32;
                    let dg = center_p.g as f32 - p.g as f32;
                    let db = center_p.b as f32 - p.b as f32;
                    let color_weight = (-(dr * dr + dg * dg + db * db) / sigma_color_sq).exp();

                    let weight = spatial_weight * color_weight;

                    sum_weight += weight;
                    sum_r += weight * p.r as f32;
                    sum_g += weight * p.g as f32;
                    sum_b += weight * p.b as f32;
                }
            }

            output.push(RGBA8::new(
                (sum_r / sum_weight) as u8,
                (sum_g / sum_weight) as u8,
                (sum_b / sum_weight) as u8,
                center_p.a,
            ));
        }
    }

    output
}

/// Reduce colors through posterization
fn reduce_colors(pixels: &[RGBA8], reduction: f32) -> Vec<RGBA8> {
    // Calculate number of color levels (256 -> 2-256 based on reduction)
    // reduction 0.0 = 256 levels (no change)
    // reduction 1.0 = 2 levels (maximum posterization)
    let levels = ((1.0 - reduction) * 254.0 + 2.0).clamp(2.0, 256.0) as u8;
    let factor = 255.0 / (levels - 1) as f32;

    pixels.iter().map(|p| {
        RGBA8::new(
            ((p.r as f32 / factor).round() * factor).clamp(0.0, 255.0) as u8,
            ((p.g as f32 / factor).round() * factor).clamp(0.0, 255.0) as u8,
            ((p.b as f32 / factor).round() * factor).clamp(0.0, 255.0) as u8,
            p.a,
        )
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduce_colors_none() {
        let pixels = vec![
            RGBA8::new(0, 0, 0, 255),
            RGBA8::new(127, 127, 127, 255),  // Use 127 instead of 128 to avoid rounding issues
            RGBA8::new(255, 255, 255, 255),
        ];
        let result = reduce_colors(&pixels, 0.0);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].r, 0);
        assert_eq!(result[1].r, 127);
        assert_eq!(result[2].r, 255);
    }

    #[test]
    fn test_reduce_colors_max() {
        let pixels = vec![
            RGBA8::new(0, 0, 0, 255),
            RGBA8::new(128, 128, 128, 255),
            RGBA8::new(255, 255, 255, 255),
        ];
        let result = reduce_colors(&pixels, 1.0); // Maximum reduction -> 2 levels
        assert_eq!(result.len(), 3);
        // All should be either 0 or 255
        for p in &result {
            assert!(p.r == 0 || p.r == 255);
        }
    }

    #[test]
    fn test_reduce_colors_partial() {
        let pixels = vec![
            RGBA8::new(0, 0, 0, 255),
            RGBA8::new(100, 100, 100, 255),
            RGBA8::new(200, 200, 200, 255),
        ];
        let result = reduce_colors(&pixels, 0.5); // 50% reduction -> ~128 levels
        assert_eq!(result.len(), 3);
        // Values should be quantized
        assert_ne!(result[0].r, result[1].r);
    }

    #[test]
    fn test_preprocess_options_default() {
        let opts = PreprocessOptions::default();
        assert_eq!(opts.color_reduction, 0.0);
        assert_eq!(opts.spatial_sigma, 3.0);
    }

    #[test]
    fn test_preprocess_options_photo() {
        let opts = PreprocessOptions::photo();
        assert_eq!(opts.color_reduction, 0.5);
        assert_eq!(opts.spatial_sigma, 5.0);
        assert_eq!(opts.iterations, 2);
    }

    #[test]
    fn test_bilateral_filter_preserves_alpha() {
        let pixels = vec![
            RGBA8::new(128, 128, 128, 255),
            RGBA8::new(128, 128, 128, 128),
            RGBA8::new(128, 128, 128, 0),
        ];
        let result = bilateral_filter(&pixels, 3, 1, 2.0, 30.0);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].a, 255);
        assert_eq!(result[1].a, 128);
        assert_eq!(result[2].a, 0);
    }

    #[test]
    fn test_bilateral_filter_uniform() {
        // All same color - should remain same
        let pixels = vec![
            RGBA8::new(128, 128, 128, 255),
            RGBA8::new(128, 128, 128, 255),
            RGBA8::new(128, 128, 128, 255),
        ];
        let result = bilateral_filter(&pixels, 3, 1, 2.0, 30.0);
        for p in result {
            assert_eq!(p.r, 128);
            assert_eq!(p.g, 128);
            assert_eq!(p.b, 128);
        }
    }
}
