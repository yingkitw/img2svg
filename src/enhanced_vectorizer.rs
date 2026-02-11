//! Enhanced vectorization pipeline ported from the vec project.
//!
//! Pipeline: bilateral filter → Sobel edge detection → edge-aware quantization
//! → flood-fill region extraction → Gaussian smoothing with corner preservation
//! → Visvalingam-Whyatt simplification → cubic Bézier fitting → SVG output
//! with gap-filling strokes and color grouping.
//!
//! The original marching-squares pipeline is preserved in vectorizer.rs for comparison.

use crate::bezier_fitter::{bezier_to_svg_path, BezierCurve, BezierFitter};
use crate::edge_detector::detect_edges_sobel;
use crate::enhanced_quantizer::{
    adaptive_color_count, count_distinct_colors, quantize_edge_aware,
};
use crate::image_processor::ImageData;
use crate::path_simplifier::{detect_corners, smooth_with_corners, visvalingam_whyatt};
use crate::preprocessor::{preprocess, PreprocessOptions};
use crate::region_extractor::detect_background_color;
use crate::vectorizer::{marching_squares_contours, Point};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Write;

/// Enhanced vectorization options.
#[derive(Debug, Clone)]
pub struct EnhancedOptions {
    /// Number of colors (0 = auto-detect based on image size)
    pub num_colors: usize,
    /// Curve fitting tolerance (lower = more accurate, larger SVG)
    pub curve_tolerance: f64,
    /// Path simplification tolerance
    pub simplification_tolerance: f64,
    /// Corner detection threshold in degrees
    pub corner_threshold: f64,
    /// Minimum region area in pixels
    pub min_region_area: usize,
    /// Edge detection threshold for edge-aware quantization
    pub edge_threshold: u8,
    /// Number of majority-vote smoothing passes
    pub smoothing_passes: usize,
    /// Smoothing window size for boundary points
    pub smooth_window: usize,
    /// Whether to apply bilateral filter preprocessing
    pub preprocess: bool,
    /// Whether to recolor from original image
    pub recolor: bool,
}

impl Default for EnhancedOptions {
    fn default() -> Self {
        Self {
            num_colors: 0, // auto
            curve_tolerance: 2.0,
            simplification_tolerance: 1.5,
            corner_threshold: 60.0,
            min_region_area: 20,
            edge_threshold: 25,
            smoothing_passes: 2,
            smooth_window: 3,
            preprocess: true,
            recolor: true,
        }
    }
}

/// Result of enhanced vectorization.
pub struct EnhancedVectorData {
    pub width: u32,
    pub height: u32,
    pub background_color: (u8, u8, u8, u8),
    pub paths: Vec<EnhancedPath>,
}

/// A vectorized path with Bézier curves.
#[derive(Debug, Clone)]
pub struct EnhancedPath {
    pub curves: Vec<BezierCurve>,
    pub color: (u8, u8, u8, u8),
    pub area: usize,
}

/// Run the enhanced vectorization pipeline.
///
/// Uses enhanced quantization (k-means++, edge-aware) with the proven
/// marching-squares contour extraction, then applies Visvalingam-Whyatt
/// simplification and cubic Bézier fitting for smooth curves.
pub fn vectorize_enhanced(
    image_data: &ImageData,
    options: &EnhancedOptions,
) -> Result<EnhancedVectorData> {
    let width = image_data.width as usize;
    let height = image_data.height as usize;
    let pixel_count = width * height;
    let is_small = pixel_count < 10_000;

    // Detect if image has many colors (photos, gradients)
    let n_colors = count_distinct_colors(image_data);
    let is_many_colors = n_colors > 16;

    // Determine target color count
    let target_colors = if options.num_colors > 0 {
        options.num_colors
    } else if is_many_colors {
        adaptive_color_count(image_data)
    } else {
        n_colors.min(64)
    };

    // Optional preprocessing (bilateral filter for photos)
    let preprocessed = if options.preprocess && is_many_colors {
        let opts = PreprocessOptions::photo();
        preprocess(image_data, &opts)?
    } else {
        image_data.clone()
    };

    // Edge detection + edge-aware quantization (k-means++ with perceptual distance)
    let edges = detect_edges_sobel(&preprocessed);
    let (quantized, _indices, _palette) = quantize_edge_aware(
        &preprocessed,
        target_colors,
        &edges,
        options.edge_threshold,
        options.smoothing_passes,
    );

    // Group pixels by quantized color
    let mut color_pixels: HashMap<(u8, u8, u8, u8), Vec<(usize, usize)>> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let p = quantized.pixels[y * width + x];
            let key = (p.r, p.g, p.b, p.a);
            color_pixels.entry(key).or_default().push((x, y));
        }
    }

    // Sort colors by pixel count (largest area first for proper z-order)
    let mut color_list: Vec<_> = color_pixels.into_iter().collect();
    color_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    // Background detection using border pixels of quantized image
    let background_color = detect_background_color(&quantized);

    let w_f = width as f64;
    let h_f = height as f64;
    let fitter = BezierFitter::new(options.curve_tolerance);
    // For photos (many colors), use higher simplification tolerance to reduce SVG size
    let simp_tol = if is_small {
        options.simplification_tolerance.min(0.5)
    } else if is_many_colors {
        options.simplification_tolerance * 2.0
    } else {
        options.simplification_tolerance
    };
    // Minimum polygon area: larger for photos to skip tiny noise regions
    let min_poly_area = if is_many_colors { 20.0 } else { 8.0 };

    // For each color: build binary mask → marching squares → smooth → simplify → Bézier fit
    // Collect (color, pixel_count, contours) tuples for parallel processing
    let color_contours: Vec<((u8, u8, u8, u8), usize, Vec<Vec<Point>>)> = color_list
        .iter()
        .filter(|(color, _)| *color != background_color)
        .map(|(color, pixels)| {
            let mut mask = vec![false; pixel_count];
            for &(x, y) in pixels {
                mask[y * width + x] = true;
            }
            let contours = marching_squares_contours(&mask, width, height);
            (*color, pixels.len(), contours)
        })
        .collect();

    // Parallel: for each contour, smooth → simplify → Bézier fit
    let mut enhanced_paths: Vec<EnhancedPath> = color_contours
        .par_iter()
        .flat_map(|(color, pixel_count, contours)| {
            let mut paths = Vec::new();

            for contour in contours {
                if contour.len() < 4 {
                    continue;
                }
                if polygon_area(contour) < min_poly_area {
                    continue;
                }

                // Smooth with corner preservation (enhanced)
                let smoothed = smooth_with_corners(
                    contour,
                    options.smooth_window,
                    options.corner_threshold,
                );

                // Detect corners for Visvalingam-Whyatt (enhanced)
                let corners = detect_corners(&smoothed, options.corner_threshold);

                // Visvalingam-Whyatt simplification with corner preservation (enhanced)
                let simplified = visvalingam_whyatt(&smoothed, simp_tol * simp_tol, &corners);

                if simplified.len() < 3 {
                    continue;
                }

                // Snap points near image edges to exact boundary
                let snap = 4.0;
                let snapped: Vec<Point> = simplified
                    .into_iter()
                    .map(|p| Point {
                        x: if p.x < snap { 0.0 } else if p.x > w_f - snap { w_f } else { p.x },
                        y: if p.y < snap { 0.0 } else if p.y > h_f - snap { h_f } else { p.y },
                    })
                    .collect();

                if snapped.len() < 3 || polygon_area(&snapped) < min_poly_area {
                    continue;
                }

                // Cubic Bézier fitting with internal corner detection (enhanced)
                let mut curves = fitter.fit_path(&snapped, true);

                // Clamp control points to image bounds (prevents bulging corners)
                for curve in &mut curves {
                    curve.control1.x = curve.control1.x.clamp(0.0, w_f);
                    curve.control1.y = curve.control1.y.clamp(0.0, h_f);
                    curve.control2.x = curve.control2.x.clamp(0.0, w_f);
                    curve.control2.y = curve.control2.y.clamp(0.0, h_f);
                }

                if !curves.is_empty() {
                    paths.push(EnhancedPath {
                        curves,
                        color: *color,
                        area: *pixel_count,
                    });
                }
            }

            paths
        })
        .collect();

    // Sort: largest regions first (back-to-front layering)
    enhanced_paths.sort_unstable_by(|a, b| b.area.cmp(&a.area));

    Ok(EnhancedVectorData {
        width: image_data.width,
        height: image_data.height,
        background_color,
        paths: enhanced_paths,
    })
}

/// Compute signed polygon area (Shoelace formula).
fn polygon_area(points: &[Point]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    (area / 2.0).abs()
}

/// Generate SVG string from enhanced vector data.
/// Uses gap-filling strokes and consecutive same-color path grouping.
pub fn generate_enhanced_svg(data: &EnhancedVectorData) -> String {
    let curve_count: usize = data.paths.iter().map(|p| p.curves.len()).sum();
    let mut svg = String::with_capacity(200 + curve_count * 80);

    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
"#,
        data.width, data.height, data.width, data.height
    ));

    // Background rect
    let bg = data.background_color;
    let bg_hex = format!("#{:02x}{:02x}{:02x}", bg.0, bg.1, bg.2);
    svg.push_str(&format!(
        "  <rect width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
        data.width, data.height, bg_hex
    ));

    // Group consecutive same-color paths
    let groups = group_by_color(&data.paths);

    for group in &groups {
        let color_hex = &group.color_hex;

        // Collect subpath data, converting thin stripes to rectangles
        let mut path_data = String::new();
        for path in &group.paths {
            let (min_x, min_y, max_x, max_y) = curve_bounds(&path.curves);
            let w = max_x - min_x;
            let h = max_y - min_y;
            // Thin horizontal stripe: wide but near-zero height → emit as 1px rect
            if h < 1.0 && w >= 2.0 {
                let y0 = min_y.round() as i64;
                let y1 = y0 + 1;
                path_data.push_str(&format!(
                    "M{},{}L{},{}L{},{}L{},{}Z",
                    min_x.round() as i64, y0,
                    max_x.round() as i64, y0,
                    max_x.round() as i64, y1,
                    min_x.round() as i64, y1,
                ));
                continue;
            }
            // Thin vertical stripe: tall but near-zero width → emit as 1px rect
            if w < 1.0 && h >= 2.0 {
                let x0 = min_x.round() as i64;
                let x1 = x0 + 1;
                path_data.push_str(&format!(
                    "M{},{}L{},{}L{},{}L{},{}Z",
                    x0, min_y.round() as i64,
                    x1, min_y.round() as i64,
                    x1, max_y.round() as i64,
                    x0, max_y.round() as i64,
                ));
                continue;
            }
            // Skip truly degenerate (tiny in both dimensions)
            if w < 1.0 || h < 1.0 {
                continue;
            }
            path_data.push_str(&bezier_to_svg_path(&path.curves, true));
        }

        if path_data.is_empty() {
            continue;
        }

        // Gap-filling stroke matching fill color
        svg.push_str(&format!(
            "  <path fill=\"{}\" stroke=\"{}\" stroke-width=\"0.5\" stroke-linejoin=\"round\" d=\"{}\"/>\n",
            color_hex, color_hex, path_data
        ));
    }

    svg.push_str("</svg>");
    svg
}

/// Write enhanced SVG to a file.
pub fn write_enhanced_svg(
    data: &EnhancedVectorData,
    output_path: &std::path::Path,
) -> Result<()> {
    let svg = generate_enhanced_svg(data);
    let mut file = std::fs::File::create(output_path)?;
    file.write_all(svg.as_bytes())?;
    Ok(())
}

struct ColorGroup {
    color_hex: String,
    paths: Vec<EnhancedPath>,
}

fn group_by_color(paths: &[EnhancedPath]) -> Vec<ColorGroup> {
    let mut groups: Vec<ColorGroup> = Vec::new();

    for path in paths {
        if path.curves.is_empty() {
            continue;
        }

        let color_hex = format!(
            "#{:02x}{:02x}{:02x}",
            path.color.0, path.color.1, path.color.2
        );

        // Only merge with immediately preceding group if same color
        if let Some(last) = groups.last_mut() {
            if last.color_hex == color_hex {
                last.paths.push(path.clone());
                continue;
            }
        }

        groups.push(ColorGroup {
            color_hex,
            paths: vec![path.clone()],
        });
    }

    groups
}

fn curve_bounds(curves: &[BezierCurve]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for c in curves {
        for p in [&c.start, &c.control1, &c.control2, &c.end] {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
    }
    (min_x, min_y, max_x, max_y)
}

fn bounds_from_points(points: &[Point]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    (min_x, min_y, max_x, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rgb::RGBA8;

    fn make_test_image(w: u32, h: u32) -> ImageData {
        let mut pixels = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if x < w / 2 {
                    pixels.push(RGBA8::new(200, 0, 0, 255));
                } else {
                    pixels.push(RGBA8::new(0, 0, 200, 255));
                }
            }
        }
        ImageData { width: w, height: h, pixels }
    }

    #[test]
    fn test_enhanced_vectorize_basic() {
        let img = make_test_image(20, 20);
        let options = EnhancedOptions {
            num_colors: 4,
            preprocess: false,
            ..Default::default()
        };
        let result = vectorize_enhanced(&img, &options).unwrap();
        assert_eq!(result.width, 20);
        assert_eq!(result.height, 20);
        // Should produce at least one path
        assert!(!result.paths.is_empty());
    }

    #[test]
    fn test_enhanced_svg_generation() {
        let img = make_test_image(20, 20);
        let options = EnhancedOptions {
            num_colors: 4,
            preprocess: false,
            ..Default::default()
        };
        let result = vectorize_enhanced(&img, &options).unwrap();
        let svg = generate_enhanced_svg(&result);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<path"));
        // Should have gap-filling strokes
        assert!(svg.contains("stroke-width=\"0.5\""));
    }

    #[test]
    fn test_enhanced_options_default() {
        let opts = EnhancedOptions::default();
        assert_eq!(opts.num_colors, 0); // auto
        assert_eq!(opts.curve_tolerance, 2.0);
        assert_eq!(opts.smoothing_passes, 2);
        assert!(opts.preprocess);
        assert!(opts.recolor);
    }

    #[test]
    fn test_enhanced_vectorize_gradient() {
        // Gradient image
        let mut pixels = Vec::new();
        for y in 0..20 {
            for x in 0..20 {
                pixels.push(RGBA8::new(
                    (x * 12) as u8,
                    (y * 12) as u8,
                    128,
                    255,
                ));
            }
        }
        let img = ImageData { width: 20, height: 20, pixels };
        let options = EnhancedOptions {
            num_colors: 8,
            preprocess: false,
            ..Default::default()
        };
        let result = vectorize_enhanced(&img, &options).unwrap();
        assert_eq!(result.width, 20);
    }

    #[test]
    fn test_enhanced_vectorize_single_color() {
        let pixels = vec![RGBA8::new(128, 128, 128, 255); 100];
        let img = ImageData { width: 10, height: 10, pixels };
        let options = EnhancedOptions {
            num_colors: 4,
            preprocess: false,
            ..Default::default()
        };
        let result = vectorize_enhanced(&img, &options).unwrap();
        assert_eq!(result.width, 10);
    }

    #[test]
    fn test_group_by_color() {
        let paths = vec![
            EnhancedPath {
                curves: vec![BezierCurve {
                    start: Point { x: 0.0, y: 0.0 },
                    control1: Point { x: 1.0, y: 1.0 },
                    control2: Point { x: 2.0, y: 1.0 },
                    end: Point { x: 3.0, y: 0.0 },
                }],
                color: (255, 0, 0, 255),
                area: 100,
            },
            EnhancedPath {
                curves: vec![BezierCurve {
                    start: Point { x: 0.0, y: 0.0 },
                    control1: Point { x: 1.0, y: 1.0 },
                    control2: Point { x: 2.0, y: 1.0 },
                    end: Point { x: 3.0, y: 0.0 },
                }],
                color: (255, 0, 0, 255), // same color
                area: 50,
            },
            EnhancedPath {
                curves: vec![BezierCurve {
                    start: Point { x: 0.0, y: 0.0 },
                    control1: Point { x: 1.0, y: 1.0 },
                    control2: Point { x: 2.0, y: 1.0 },
                    end: Point { x: 3.0, y: 0.0 },
                }],
                color: (0, 0, 255, 255), // different color
                area: 80,
            },
        ];
        let groups = group_by_color(&paths);
        assert_eq!(groups.len(), 2); // red group + blue group
        assert_eq!(groups[0].paths.len(), 2); // two red paths merged
        assert_eq!(groups[1].paths.len(), 1);
    }
}
