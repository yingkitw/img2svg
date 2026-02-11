mod cli;
mod image_processor;
mod svg_generator;
mod vectorizer;
mod preprocessor;
mod edge_detector;
mod enhanced_quantizer;
mod region_extractor;
mod path_simplifier;
mod bezier_fitter;
mod enhanced_vectorizer;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use preprocessor::{preprocess, PreprocessOptions};
use enhanced_vectorizer::{vectorize_enhanced, write_enhanced_svg, EnhancedOptions};

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

    if cli.enhanced {
        eprintln!("Using enhanced pipeline (Bézier curves, edge-aware quantization, flood-fill regions)...");
        let options = EnhancedOptions {
            num_colors: cli.colors,
            // Enhanced pipeline auto-preprocesses photos by default (preprocess=true).
            // CLI -p flag forces preprocessing even for non-photo images.
            preprocess: cli.preprocess || EnhancedOptions::default().preprocess,
            ..Default::default()
        };
        let vector_data = vectorize_enhanced(&image_data, &options)?;
        write_enhanced_svg(&vector_data, &output_path)?;
        eprintln!(
            "  {} paths, background #{:02x}{:02x}{:02x}",
            vector_data.paths.len(),
            vector_data.background_color.0,
            vector_data.background_color.1,
            vector_data.background_color.2,
        );
    } else {
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
    }

    println!("Conversion complete!");
    Ok(())
}
