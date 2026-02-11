mod cli;
mod image_processor;
mod svg_generator;
mod vectorizer;
mod preprocessor;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use preprocessor::{preprocess, PreprocessOptions};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let output_path = cli.output.unwrap_or_else(|| {
        let mut path = cli.input.clone();
        path.set_extension("svg");
        path
    });

    println!(
        "Converting {} to {}...",
        cli.input.display(),
        output_path.display()
    );

    let mut image_data = image_processor::load_image(&cli.input)?;

    // Apply preprocessing if requested
    if cli.preprocess {
        eprintln!("Applying edge-preserving smoothing and color reduction...");
        let opts = PreprocessOptions::photo();
        image_data = preprocess(&image_data, &opts)?;
    }

    // Provide hints for photographs (images with many small color regions don't vectorize well)
    let total_pixels = image_data.width as usize * image_data.height as usize;
    let unique_colors = image_data.pixels.iter()
        .map(|p| (p.r, p.g, p.b))
        .collect::<std::collections::HashSet<_>>()
        .len();

    // If image has many unique colors (likely a photo), show a hint
    if unique_colors > 10000 && !cli.preprocess {
        eprintln!();
        eprintln!("Note: This image appears to be a photograph with many color variations.");
        eprintln!("For better results, try:");
        eprintln!("  --preprocess      (applies edge-preserving smoothing and color reduction)");
        eprintln!("  --colors 8-12     (fewer colors reduce posterization)");
        eprintln!("  --threshold 0.15  (higher threshold ignores subtle variations)");
        eprintln!();
        eprintln!("Note: Vectorization works best for images with clear color boundaries");
        eprintln!("(logos, icons, flat illustrations).");
        eprintln!();
    }

    let vectorized_data = vectorizer::vectorize(
        &image_data,
        cli.colors,
        cli.threshold,
        cli.smooth,
        cli.hierarchical,
    )?;

    if cli.advanced {
        svg_generator::generate_svg_advanced(&vectorized_data, &output_path)?;
    } else {
        svg_generator::generate_svg(&vectorized_data, &output_path)?;
    }

    println!("Conversion complete!");
    Ok(())
}
