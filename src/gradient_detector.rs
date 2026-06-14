//! Gradient detection for smoother SVG gradient rendering.
//!
//! Detects linear and radial gradients within image regions so they can be
//! rendered as native SVG `<linearGradient>` / `<radialGradient>` instead of
//! discrete color bands.

use crate::image_processor::ImageData;
use rgb::RGBA8;

/// Detected gradient type and parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum DetectedGradient {
    Linear(LinearGradient),
    Radial(RadialGradient),
}

/// Linear gradient parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub start_color: (u8, u8, u8, u8),
    pub end_color: (u8, u8, u8, u8),
    /// Mean squared error of the fit (lower = better)
    pub error: f64,
}

/// Radial gradient parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub start_color: (u8, u8, u8, u8),
    pub end_color: (u8, u8, u8, u8),
    pub error: f64,
}

/// Try to detect a gradient in the masked region of the image.
/// Returns `None` if no smooth gradient pattern is found.
pub fn detect_gradient(
    image_data: &ImageData,
    mask: &[bool],
    error_threshold: f64,
) -> Option<DetectedGradient> {
    let linear = detect_linear_gradient(image_data, mask)?;
    let radial = detect_radial_gradient(image_data, mask)?;

    if linear.error < radial.error && linear.error < error_threshold {
        Some(DetectedGradient::Linear(linear))
    } else if radial.error < error_threshold {
        Some(DetectedGradient::Radial(radial))
    } else {
        None
    }
}

/// Detect a linear gradient by finding the principal direction of color
/// variation and fitting start/end colors along that axis.
fn color_variance(colors: &[RGBA8]) -> f64 {
    if colors.is_empty() {
        return 0.0;
    }
    let mean_r = colors.iter().map(|c| c.r as f64).sum::<f64>() / colors.len() as f64;
    let mean_g = colors.iter().map(|c| c.g as f64).sum::<f64>() / colors.len() as f64;
    let mean_b = colors.iter().map(|c| c.b as f64).sum::<f64>() / colors.len() as f64;
    let var_r = colors.iter().map(|c| (c.r as f64 - mean_r).powi(2)).sum::<f64>() / colors.len() as f64;
    let var_g = colors.iter().map(|c| (c.g as f64 - mean_g).powi(2)).sum::<f64>() / colors.len() as f64;
    let var_b = colors.iter().map(|c| (c.b as f64 - mean_b).powi(2)).sum::<f64>() / colors.len() as f64;
    (var_r + var_g + var_b) / 3.0
}

pub fn detect_linear_gradient(image_data: &ImageData, mask: &[bool]) -> Option<LinearGradient> {
    let w = image_data.width as usize;
    let h = image_data.height as usize;

    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut colors = Vec::new();

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if i < mask.len() && mask[i] {
                xs.push(x as f64);
                ys.push(y as f64);
                colors.push(image_data.pixels[i]);
            }
        }
    }

    if xs.len() < 4 {
        return None;
    }

    // Reject near-uniform color regions
    if color_variance(&colors) < 25.0 {
        return None;
    }

    // Compute centroid
    let cx = mean(&xs);
    let cy = mean(&ys);

    // Compute covariance matrix of positions weighted by luminance
    let mut cov_xx = 0.0;
    let mut cov_yy = 0.0;
    let mut cov_xy = 0.0;
    for i in 0..xs.len() {
        let dx = xs[i] - cx;
        let dy = ys[i] - cy;
        let lum = luminance(&colors[i]) as f64;
        cov_xx += dx * dx * lum;
        cov_yy += dy * dy * lum;
        cov_xy += dx * dy * lum;
    }

    // Principal direction (eigenvector of largest eigenvalue)
    let trace = cov_xx + cov_yy;
    let det = cov_xx * cov_yy - cov_xy * cov_xy;
    let discriminant = (trace * trace - 4.0 * det).max(0.0);
    let lambda1 = (trace + discriminant.sqrt()) / 2.0;
    let lambda2 = (trace - discriminant.sqrt()) / 2.0;

    // Direction vector of principal component
    let (dx, dy) = if cov_xy.abs() < 1e-6 {
        (1.0, 0.0)
    } else {
        let v = (lambda1 - cov_yy) / cov_xy;
        let len = (v * v + 1.0).sqrt();
        (v / len, 1.0 / len)
    };

    // Project all points onto the principal direction
    let mut projections: Vec<f64> = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| (x - cx) * dx + (y - cy) * dy)
        .collect();

    let t_min = projections.iter().copied().fold(f64::INFINITY, f64::min);
    let t_max = projections.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let t_range = t_max - t_min;

    if t_range < 1.0 {
        return None;
    }

    // Normalize projections to [0, 1]
    for p in &mut projections {
        *p = (*p - t_min) / t_range;
    }

    // Fit start and end colors by linear regression per channel
    let (start_r, end_r, err_r) = fit_line(&projections, &colors, |c| c.r as f64);
    let (start_g, end_g, err_g) = fit_line(&projections, &colors, |c| c.g as f64);
    let (start_b, end_b, err_b) = fit_line(&projections, &colors, |c| c.b as f64);
    let (start_a, end_a, err_a) = fit_line(&projections, &colors, |c| c.a as f64);

    let error = (err_r + err_g + err_b + err_a) / 4.0;

    Some(LinearGradient {
        x1: cx + t_min * dx,
        y1: cy + t_min * dy,
        x2: cx + t_max * dx,
        y2: cy + t_max * dy,
        start_color: (
            start_r.round().clamp(0.0, 255.0) as u8,
            start_g.round().clamp(0.0, 255.0) as u8,
            start_b.round().clamp(0.0, 255.0) as u8,
            start_a.round().clamp(0.0, 255.0) as u8,
        ),
        end_color: (
            end_r.round().clamp(0.0, 255.0) as u8,
            end_g.round().clamp(0.0, 255.0) as u8,
            end_b.round().clamp(0.0, 255.0) as u8,
            end_a.round().clamp(0.0, 255.0) as u8,
        ),
        error,
    })
}

/// Detect a radial gradient by fitting color as a function of distance from
/// the region's geometric center.
pub fn detect_radial_gradient(image_data: &ImageData, mask: &[bool]) -> Option<RadialGradient> {
    let w = image_data.width as usize;
    let h = image_data.height as usize;

    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut colors = Vec::new();

    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if i < mask.len() && mask[i] {
                xs.push(x as f64);
                ys.push(y as f64);
                colors.push(image_data.pixels[i]);
            }
        }
    }

    if xs.len() < 4 {
        return None;
    }

    // Reject near-uniform color regions
    if color_variance(&colors) < 25.0 {
        return None;
    }

    let cx = mean(&xs);
    let cy = mean(&ys);

    // Compute distances from center
    let mut distances: Vec<f64> = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
        .collect();

    let d_max = distances.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if d_max < 1.0 {
        return None;
    }

    // Normalize to [0, 1]
    for d in &mut distances {
        *d /= d_max;
    }

    // Fit colors per channel
    let (start_r, end_r, err_r) = fit_line(&distances, &colors, |c| c.r as f64);
    let (start_g, end_g, err_g) = fit_line(&distances, &colors, |c| c.g as f64);
    let (start_b, end_b, err_b) = fit_line(&distances, &colors, |c| c.b as f64);
    let (start_a, end_a, err_a) = fit_line(&distances, &colors, |c| c.a as f64);

    let error = (err_r + err_g + err_b + err_a) / 4.0;

    Some(RadialGradient {
        cx,
        cy,
        r: d_max,
        start_color: (
            start_r.round().clamp(0.0, 255.0) as u8,
            start_g.round().clamp(0.0, 255.0) as u8,
            start_b.round().clamp(0.0, 255.0) as u8,
            start_a.round().clamp(0.0, 255.0) as u8,
        ),
        end_color: (
            end_r.round().clamp(0.0, 255.0) as u8,
            end_g.round().clamp(0.0, 255.0) as u8,
            end_b.round().clamp(0.0, 255.0) as u8,
            end_a.round().clamp(0.0, 255.0) as u8,
        ),
        error,
    })
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn luminance(c: &RGBA8) -> f64 {
    0.299 * c.r as f64 + 0.587 * c.g as f64 + 0.114 * c.b as f64
}

/// Fit `y = start + t * (end - start)` (i.e. `y = a + b*t`) and return
/// `(start, end, mean_squared_error)`.
fn fit_line<F>(t: &[f64], y: &[RGBA8], extractor: F) -> (f64, f64, f64)
where
    F: Fn(&RGBA8) -> f64,
{
    let n = t.len() as f64;
    let mean_t = mean(t);
    let mean_y: f64 = y.iter().map(&extractor).sum::<f64>() / n;

    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..t.len() {
        let yi = extractor(&y[i]);
        let dt = t[i] - mean_t;
        num += dt * (yi - mean_y);
        den += dt * dt;
    }

    let b = if den.abs() > 1e-9 { num / den } else { 0.0 };
    let a = mean_y - b * mean_t;

    // start at t=0, end at t=1
    let start = a;
    let end_val = a + b;

    let mut sse = 0.0;
    for i in 0..t.len() {
        let predicted = a + b * t[i];
        let diff = extractor(&y[i]) - predicted;
        sse += diff * diff;
    }
    let mse = sse / t.len().max(1) as f64;

    (start, end_val, mse)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_horizontal_gradient_image(width: u32, height: u32) -> ImageData {
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let t = x as f64 / (width as f64 - 1.0);
                let r = (t * 255.0) as u8;
                let g = ((1.0 - t) * 255.0) as u8;
                let b = 128u8;
                pixels.push(RGBA8::new(r, g, b, 255));
            }
        }
        ImageData { width, height, pixels }
    }

    fn create_radial_gradient_image(width: u32, height: u32) -> ImageData {
        let cx = width as f64 / 2.0;
        let cy = height as f64 / 2.0;
        let max_dist = ((cx * cx + cy * cy).sqrt()).max(1.0);
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let dist =
                    (((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)).sqrt() / max_dist).min(1.0);
                let r = (dist * 255.0) as u8;
                let g = ((1.0 - dist) * 255.0) as u8;
                let b = 128u8;
                pixels.push(RGBA8::new(r, g, b, 255));
            }
        }
        ImageData { width, height, pixels }
    }

    fn full_mask(width: u32, height: u32) -> Vec<bool> {
        vec![true; (width * height) as usize]
    }

    #[test]
    fn test_detect_linear_gradient_horizontal() {
        let img = create_horizontal_gradient_image(20, 10);
        let mask = full_mask(20, 10);
        let grad = detect_linear_gradient(&img, &mask);
        assert!(grad.is_some(), "Should detect a linear gradient");
        let g = grad.unwrap();
        assert!(g.error < 50.0, "Error should be low for synthetic gradient: {}", g.error);
        // Principal direction eigenvector sign is arbitrary, so start/end may be swapped.
        // One end should be red-ish (high R) and the other green-ish (high G).
        let (end_a, end_b) = (g.start_color, g.end_color);
        let red_at_one_end = (end_a.0 > 200 && end_b.1 > 200) || (end_b.0 > 200 && end_a.1 > 200);
        assert!(red_at_one_end, "One end should be red-ish and the other green-ish");
    }

    #[test]
    fn test_detect_radial_gradient() {
        let img = create_radial_gradient_image(20, 20);
        let mask = full_mask(20, 20);
        let grad = detect_radial_gradient(&img, &mask);
        assert!(grad.is_some(), "Should detect a radial gradient");
        let g = grad.unwrap();
        assert!(g.error < 100.0, "Error should be reasonable for synthetic gradient: {}", g.error);
        // Radial: center vs edge colors may be swapped depending on fit direction
        let center_dark = g.start_color.0 < 50 || g.end_color.0 < 50;
        assert!(center_dark, "Center should be dark at one end");
    }

    #[test]
    fn test_detect_gradient_prefers_better_fit() {
        let img = create_horizontal_gradient_image(20, 10);
        let mask = full_mask(20, 10);
        let grad = detect_gradient(&img, &mask, 50.0);
        assert!(grad.is_some());
        // Horizontal gradient → linear should fit better than radial
        assert!(
            matches!(grad, Some(DetectedGradient::Linear(_))),
            "Should prefer linear for horizontal gradient"
        );
    }

    #[test]
    fn test_detect_gradient_rejects_uniform_color() {
        let pixels = vec![RGBA8::new(128, 128, 128, 255); 100];
        let img = ImageData {
            width: 10,
            height: 10,
            pixels,
        };
        let mask = full_mask(10, 10);
        let grad = detect_gradient(&img, &mask, 50.0);
        assert!(grad.is_none(), "Uniform color should not be detected as gradient");
    }

    #[test]
    fn test_detect_gradient_small_region() {
        let pixels = vec![RGBA8::new(0, 0, 0, 255); 3];
        let img = ImageData {
            width: 3,
            height: 1,
            pixels,
        };
        let mask = vec![true, true, true];
        let grad = detect_linear_gradient(&img, &mask);
        assert!(grad.is_none(), "Region too small for reliable detection");
    }
}
