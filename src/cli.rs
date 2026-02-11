use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "img2svg")]
#[command(about = "A high-quality image to SVG converter")]
#[command(version)]
pub struct Cli {
    /// Input image file
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output SVG file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

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

    /// Use enhanced pipeline (Bézier curves, edge-aware quantization, flood-fill regions)
    #[arg(short = 'e', long)]
    pub enhanced: bool,
}
