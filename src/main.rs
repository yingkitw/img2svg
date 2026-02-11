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
use cli::{Cli, is_supported_image};
use preprocessor::{preprocess, PreprocessOptions};
use enhanced_vectorizer::{vectorize_enhanced, write_enhanced_svg, EnhancedOptions};
use std::path::{Path, PathBuf};

/// Process a single image file.
fn process_file(
    input_path: &Path,
    output_path: &Path,
    cli: &Cli,
) -> Result<()> {
    let mut image_data = image_processor::load_image(input_path)?;

    // Auto-resize large images to prevent OOM
    image_data = image_processor::resize_if_needed(image_data, cli.max_size);

    // Apply preprocessing if requested
    if cli.preprocess {
        eprintln!("  Applying edge-preserving smoothing and color reduction...");
        let opts = PreprocessOptions::photo();
        image_data = preprocess(&image_data, &opts)?;
    }

    // Provide hints for photographs
    let unique_colors = image_data.pixels.iter()
        .map(|p| (p.r, p.g, p.b))
        .collect::<std::collections::HashSet<_>>()
        .len();

    if unique_colors > 10000 && !cli.preprocess {
        eprintln!("  Note: photo detected ({} colors). Try --preprocess for better results.", unique_colors);
    }

    if cli.original {
        eprintln!("  Using original pipeline (line segments, RDP simplification)...");
        let vectorized_data = vectorizer::vectorize(
            &image_data,
            cli.colors,
            cli.threshold,
            cli.smooth,
            cli.hierarchical,
        )?;

        if cli.advanced {
            svg_generator::generate_svg_advanced(&vectorized_data, output_path)?;
        } else {
            svg_generator::generate_svg(&vectorized_data, output_path)?;
        }
    } else {
        let options = EnhancedOptions {
            num_colors: cli.colors,
            preprocess: cli.preprocess || EnhancedOptions::default().preprocess,
            ..Default::default()
        };
        let vector_data = vectorize_enhanced(&image_data, &options)?;
        write_enhanced_svg(&vector_data, output_path)?;
        eprintln!(
            "  {} paths, background #{:02x}{:02x}{:02x}",
            vector_data.paths.len(),
            vector_data.background_color.0,
            vector_data.background_color.1,
            vector_data.background_color.2,
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.input.is_dir() {
        // Batch mode: process all supported images in directory
        let output_dir = cli.output.clone().unwrap_or_else(|| cli.input.clone());
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }

        let mut count = 0u32;
        let mut errors = 0u32;
        let entries: Vec<_> = std::fs::read_dir(&cli.input)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file() && is_supported_image(&e.path()))
            .collect();

        let total = entries.len();
        eprintln!("Batch converting {} images from {}...", total, cli.input.display());

        for entry in &entries {
            let path = entry.path();
            let stem = path.file_stem().unwrap().to_string_lossy();
            let mut out_path = output_dir.join(stem.as_ref());
            out_path.set_extension("svg");

            eprintln!("[{}/{}] {} -> {}", count + errors + 1, total, path.display(), out_path.display());
            match process_file(&path, &out_path, &cli) {
                Ok(()) => count += 1,
                Err(e) => {
                    eprintln!("  Error: {}", e);
                    errors += 1;
                }
            }
        }

        println!("Batch complete: {} converted, {} errors.", count, errors);
    } else {
        // Single file mode
        let output_path = cli.output.clone().unwrap_or_else(|| {
            let mut path = cli.input.clone();
            path.set_extension("svg");
            path
        });

        println!("Converting {} to {}...", cli.input.display(), output_path.display());
        process_file(&cli.input, &output_path, &cli)?;
        println!("Conversion complete!");
    }

    Ok(())
}
