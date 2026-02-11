//! img2svg - A high-quality image to SVG converter library
//!
//! This library provides functionality to convert raster images (PNG, JPEG, etc.)
//! into scalable vector graphics (SVG) format.
//!
//! ## Features
//!
//! - **Color quantization** using median-cut algorithm
//! - **Marching squares** contour tracing for accurate shape detection
//! - **Ramer-Douglas-Peucker** simplification for clean paths
//! - **Gaussian smoothing** for natural curves
//!
//! ## Example
//!
//! ```rust,no_run
//! use img2svg::{convert, ConversionOptions};
//! use std::path::Path;
//!
//! let options = ConversionOptions {
//!     num_colors: 16,
//!     smooth_level: 5,
//!     ..Default::default()
//! };
//!
//! convert(Path::new("input.png"), Path::new("output.svg"), &options)
//!     .expect("Conversion failed");
//! ```

pub mod image_processor;
pub mod svg_generator;
pub mod vectorizer;
pub mod preprocessor;

pub use image_processor::{load_image, quantize_colors, ImageData};
pub use svg_generator::{generate_svg, generate_svg_advanced};
pub use vectorizer::{vectorize, Curve, Point, VectorizedData};
pub use preprocessor::{preprocess, PreprocessOptions};
pub use anyhow::Result;

/// Options for image to SVG conversion
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Number of colors for quantization (default: 16)
    pub num_colors: usize,
    /// Edge detection threshold 0.0-1.0 (default: 0.1)
    pub threshold: f64,
    /// Path smoothing level 0-10 (default: 5)
    pub smooth_level: u8,
    /// Enable hierarchical decomposition (default: false)
    pub hierarchical: bool,
    /// Use advanced SVG generation (default: false)
    pub advanced: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            num_colors: 16,
            threshold: 0.1,
            smooth_level: 5,
            hierarchical: false,
            advanced: false,
        }
    }
}

/// Convert an image file to SVG
///
/// # Arguments
///
/// * `input_path` - Path to the input image file
/// * `output_path` - Path to the output SVG file
/// * `options` - Conversion options
///
/// # Example
///
/// ```rust,no_run
/// use img2svg::{convert, ConversionOptions};
/// use std::path::Path;
///
/// let options = ConversionOptions::default();
/// convert(Path::new("input.png"), Path::new("output.svg"), &options)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn convert(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    options: &ConversionOptions,
) -> Result<()> {
    // Load the image
    let image_data = load_image(input_path)?;

    // Vectorize the image
    let vectorized_data = vectorize(
        &image_data,
        options.num_colors,
        options.threshold,
        options.smooth_level,
        options.hierarchical,
    )?;

    // Generate SVG output
    if options.advanced {
        generate_svg_advanced(&vectorized_data, output_path)?;
    } else {
        generate_svg(&vectorized_data, output_path)?;
    }

    Ok(())
}

/// Convert image data directly to SVG string
///
/// This is useful when you have image data in memory and want to get
/// the SVG content as a string without writing to a file.
///
/// # Arguments
///
/// * `image_data` - The image data to convert
/// * `options` - Conversion options
///
/// # Returns
///
/// A String containing the SVG content
pub fn convert_to_svg_string(image_data: &ImageData, options: &ConversionOptions) -> Result<String> {
    let vectorized_data = vectorize(
        image_data,
        options.num_colors,
        options.threshold,
        options.smooth_level,
        options.hierarchical,
    )?;

    // Generate SVG to a temporary location, then read it back
    use std::io::Write;
    let mut buffer = Vec::new();

    // Write SVG header
    let bg = vectorized_data.background_color;
    let bg_str = format!("#{:02x}{:02x}{:02x}", bg.0, bg.1, bg.2);

    writeln!(
        &mut buffer,
        r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
        vectorized_data.width,
        vectorized_data.height,
        vectorized_data.width,
        vectorized_data.height
    )?;
    writeln!(
        &mut buffer,
        r#"  <rect width="{}" height="{}" fill="{}"/>"#,
        vectorized_data.width,
        vectorized_data.height,
        bg_str
    )?;

    // Write curves
    for curve in &vectorized_data.curves {
        let color_str = format!(
            "#{:02x}{:02x}{:02x}",
            curve.color.0, curve.color.1, curve.color.2
        );

        let path_str = if !curve.subpaths.is_empty() {
            svg_generator::create_multi_path_string(&curve.subpaths)
        } else if !curve.points.is_empty() {
            svg_generator::create_subpath_string(&curve.points, curve.is_closed)
        } else {
            continue;
        };

        if path_str.is_empty() {
            continue;
        }

        writeln!(
            &mut buffer,
            r#"  <path d="{}" fill="{}" stroke="none"/>"#,
            path_str, color_str
        )?;
    }

    writeln!(&mut buffer, "</svg>")?;

    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion_options_default() {
        let options = ConversionOptions::default();
        assert_eq!(options.num_colors, 16);
        assert_eq!(options.threshold, 0.1);
        assert_eq!(options.smooth_level, 5);
        assert!(!options.hierarchical);
        assert!(!options.advanced);
    }
}
