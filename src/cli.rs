use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "img2svg")]
#[command(about = "A high-quality image to SVG converter with Bézier curves")]
#[command(version)]
pub struct Cli {
    /// Input image file or directory (batch mode)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output SVG file or directory (batch mode)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Maximum image dimension (auto-resize larger images to prevent OOM)
    #[arg(long, default_value = "4096")]
    pub max_size: u32,

    /// Number of colors to quantize (default: 16)
    #[arg(short, long, default_value = "16")]
    pub colors: usize,

    /// Edge detection threshold (0.0-1.0, default: 0.1)
    #[arg(short, long, default_value = "0.1")]
    pub threshold: f64,

    /// Path smoothing level (0-10, default: 5)
    #[arg(short = 's', long, default_value = "5")]
    pub smooth: u8,

    /// Enable hierarchical decomposition for better quality
    #[arg(long)]
    pub hierarchical: bool,

    /// Use advanced SVG generation with layers
    #[arg(short, long)]
    pub advanced: bool,

    /// Apply preprocessing (edge-preserving smoothing + color reduction) for photos
    #[arg(short = 'p', long)]
    pub preprocess: bool,

    /// Use original pipeline (line segments, RDP simplification) instead of default Bézier
    #[arg(long)]
    pub original: bool,
}

/// Check if a file extension is a supported image format.
pub fn is_supported_image(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "bmp" | "png" | "jpg" | "jpeg" | "gif" | "ico" | "tiff" | "tif" | "webp" | "pnm" | "tga" | "dds" | "farbfeld"
        )
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::is_supported_image;
    use std::path::Path;

    #[test]
    fn supported_extensions_lowercase() {
        assert!(is_supported_image(Path::new("x.png")));
        assert!(is_supported_image(Path::new("x.jpeg")));
        assert!(is_supported_image(Path::new("x.webp")));
        assert!(is_supported_image(Path::new("x.tiff")));
    }

    #[test]
    fn supported_extensions_uppercase() {
        assert!(is_supported_image(Path::new("photo.PNG")));
        assert!(is_supported_image(Path::new("photo.JPG")));
    }

    #[test]
    fn unsupported_or_missing_extension() {
        assert!(!is_supported_image(Path::new("readme")));
        assert!(!is_supported_image(Path::new("file.txt")));
        assert!(!is_supported_image(Path::new("noext")));
    }
}
