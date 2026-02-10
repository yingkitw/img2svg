mod cli;
mod image_processor;
mod svg_generator;
mod vectorizer;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

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

    let image_data = image_processor::load_image(&cli.input)?;
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
