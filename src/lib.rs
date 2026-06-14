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

/// Cubic Bézier curve fitting with Newton-Raphson reparameterization.
pub mod bezier_fitter;
/// Sobel edge detection for gradient-based edge maps.
pub mod edge_detector;
/// K-means++ initialization and edge-aware color quantization.
pub mod enhanced_quantizer;
/// Enhanced vectorization pipeline: marching squares → Bézier curves.
pub mod enhanced_vectorizer;
/// Linear and radial gradient detection for smoother SVG gradient fills.
pub mod gradient_detector;
/// Comprehensive image processing filters (noise reduction, edge enhancement,
/// contrast adjustment, thresholding, morphology, color operations, effects).
pub mod image_filters;
/// Image loading, auto-resize, and median-cut color quantization.
pub mod image_processor;
/// Visvalingam-Whyatt simplification with corner preservation.
pub mod path_simplifier;
/// LUT bilateral filter and color reduction for photo preprocessing.
pub mod preprocessor;
/// Background color detection via border-pixel frequency.
pub mod region_extractor;
/// SVG generation with compact output and alpha channel support.
pub mod svg_generator;
/// Original pipeline: marching squares, Gaussian smoothing, RDP simplification.
pub mod vectorizer;

/// Re-export anyhow Result for convenience.
pub use anyhow::Result;
/// Re-export enhanced vectorization pipeline types and functions.
pub use enhanced_vectorizer::{
    generate_enhanced_svg, generate_enhanced_svg_to, vectorize_enhanced, write_enhanced_svg,
    EnhancedOptions, EnhancedPath, EnhancedVectorData,
};
/// Re-export image filter functions and types.
pub use gradient_detector::{
    detect_gradient, detect_linear_gradient, detect_radial_gradient, DetectedGradient,
    LinearGradient, RadialGradient,
};
pub use image_filters::{
    median_filter, prewitt_edge_detection, roberts_edge_detection, weighted_median_filter,
    AdaptiveThreshold, AdaptiveToneMapper, CannyEdgeDetector, ColorOps, ColorTemperature,
    EmbossFilter, GammaCorrection, HighBoostFilter, HistogramEqualizer, LaplacianSharpen,
    MorphologyKind, MorphologyOp, OtsuThreshold, Saturation, SepiaFilter, SmartThreshold,
    UnsharpMask, VignetteFilter, VintageFilter, CLAHE, HSL, LAB,
};
/// Re-export image loading and quantization functions.
pub use image_processor::{load_image, quantize_colors, ImageData};
/// Re-export photo preprocessing functions.
pub use preprocessor::{preprocess, PreprocessOptions};
/// Re-export SVG generation functions.
pub use svg_generator::{
    generate_svg, generate_svg_advanced, generate_svg_to, write_multi_path_to, write_subpath_to,
};
/// Re-export original vectorization pipeline types and functions.
pub use vectorizer::{vectorize, Curve, Point, VectorizedData};

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
pub fn convert_to_svg_string(
    image_data: &ImageData,
    options: &ConversionOptions,
) -> Result<String> {
    let vectorized_data = vectorize(
        image_data,
        options.num_colors,
        options.threshold,
        options.smooth_level,
        options.hierarchical,
    )?;

    let mut buffer = Vec::new();
    svg_generator::generate_svg_to(&vectorized_data, &mut buffer)?;
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
