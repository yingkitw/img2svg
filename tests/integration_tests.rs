// Integration tests for img2svg
use img2svg::image_processor::{load_image, quantize_colors};
use img2svg::svg_generator::{generate_svg, generate_svg_advanced};
use img2svg::vectorizer::vectorize;
use std::fs;
use std::path::PathBuf;

// Create a simple test image programmatically
fn create_test_png(path: &PathBuf, width: u32, height: u32, pattern: &str) {
    let mut pixel_data: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let (r, g, b) = match pattern {
                "gradient" => {
                    let r = (x * 255 / width.max(1)) as u8;
                    let g = (y * 255 / height.max(1)) as u8;
                    let b = 128;
                    (r, g, b)
                }
                "checkerboard" => {
                    let size = 10;
                    let is_white = ((x / size) + (y / size)) % 2 == 0;
                    if is_white { (255, 255, 255) } else { (0, 0, 0) }
                }
                "circle" => {
                    let cx = width / 2;
                    let cy = height / 2;
                    let radius = width.min(height) / 4;
                    let dx = x as i32 - cx as i32;
                    let dy = y as i32 - cy as i32;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < (radius * radius) as i32 {
                        (255, 0, 0)
                    } else {
                        (255, 255, 255)
                    }
                }
                "solid" => (128, 128, 128),
                _ => (255, 255, 255),
            };
            pixel_data.push(r);
            pixel_data.push(g);
            pixel_data.push(b);
            pixel_data.push(255);
        }
    }

    let img: image::RgbaImage = image::ImageBuffer::from_raw(width, height, pixel_data).unwrap();
    img.save(path).expect("Failed to save test image");
}

#[test]
fn test_full_pipeline_gradient() {
    let test_img = PathBuf::from("/tmp/test_gradient.png");
    let test_svg = PathBuf::from("/tmp/test_gradient_output.svg");

    create_test_png(&test_img, 50, 50, "gradient");

    // Load image
    let image_data = load_image(&test_img).expect("Failed to load image");
    assert_eq!(image_data.width, 50);
    assert_eq!(image_data.height, 50);

    // Quantize colors
    let quantized = quantize_colors(&image_data, 8).expect("Failed to quantize");
    assert_eq!(quantized.width, 50);
    assert_eq!(quantized.height, 50);

    // Vectorize
    let vectorized = vectorize(&quantized, 8, 0.1, 2, false).expect("Failed to vectorize");
    assert_eq!(vectorized.width, 50);
    assert_eq!(vectorized.height, 50);

    // Generate SVG
    generate_svg(&vectorized, &test_svg).expect("Failed to generate SVG");

    // Verify SVG was created and is valid
    assert!(test_svg.exists());
    let svg_content = fs::read_to_string(&test_svg).expect("Failed to read SVG");
    assert!(svg_content.contains("<svg"));
    assert!(svg_content.contains("</svg>"));
    assert!(svg_content.contains("width=\"50\""));
    assert!(svg_content.contains("height=\"50\""));

    // Cleanup
    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_svg);
}

#[test]
fn test_full_pipeline_checkerboard() {
    let test_img = PathBuf::from("/tmp/test_checkerboard.png");
    let test_svg = PathBuf::from("/tmp/test_checkerboard_output.svg");

    create_test_png(&test_img, 40, 40, "checkerboard");

    // Full pipeline
    let image_data = load_image(&test_img).expect("Failed to load image");
    let quantized = quantize_colors(&image_data, 2).expect("Failed to quantize");
    let vectorized = vectorize(&quantized, 2, 0.1, 0, false).expect("Failed to vectorize");
    generate_svg(&vectorized, &test_svg).expect("Failed to generate SVG");

    // Verify SVG
    assert!(test_svg.exists());
    let svg_content = fs::read_to_string(&test_svg).expect("Failed to read SVG");
    assert!(svg_content.contains("<svg"));

    // Cleanup
    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_svg);
}

#[test]
fn test_full_pipeline_circle() {
    let test_img = PathBuf::from("/tmp/test_circle.png");
    let test_svg = PathBuf::from("/tmp/test_circle_output.svg");

    create_test_png(&test_img, 60, 60, "circle");

    // Full pipeline with smoothing
    let image_data = load_image(&test_img).expect("Failed to load image");
    let quantized = quantize_colors(&image_data, 4).expect("Failed to quantize");
    let vectorized = vectorize(&quantized, 4, 0.1, 3, false).expect("Failed to vectorize");
    generate_svg(&vectorized, &test_svg).expect("Failed to generate SVG");

    // Verify SVG has curves
    assert!(test_svg.exists());
    let svg_content = fs::read_to_string(&test_svg).expect("Failed to read SVG");
    assert!(svg_content.contains("<path"));

    // Cleanup
    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_svg);
}

#[test]
fn test_full_pipeline_advanced_svg() {
    let test_img = PathBuf::from("/tmp/test_advanced.png");
    let test_svg = PathBuf::from("/tmp/test_advanced_output.svg");

    create_test_png(&test_img, 50, 50, "gradient");

    // Full pipeline with advanced SVG generation
    let image_data = load_image(&test_img).expect("Failed to load image");
    let quantized = quantize_colors(&image_data, 16).expect("Failed to quantize");
    let vectorized = vectorize(&quantized, 16, 0.1, 2, false).expect("Failed to vectorize");
    generate_svg_advanced(&vectorized, &test_svg).expect("Failed to generate advanced SVG");

    // Verify SVG
    assert!(test_svg.exists());
    let svg_content = fs::read_to_string(&test_svg).expect("Failed to read SVG");
    assert!(svg_content.contains("<svg"));

    // Cleanup
    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_svg);
}

#[test]
fn test_vectorize_preserves_content() {
    let test_img = PathBuf::from("/tmp/test_content.png");
    create_test_png(&test_img, 30, 30, "circle");

    let image_data = load_image(&test_img).expect("Failed to load image");
    let vectorized = vectorize(&image_data, 4, 0.1, 1, false).expect("Failed to vectorize");

    // Verify structure
    assert_eq!(vectorized.width, 30);
    assert_eq!(vectorized.height, 30);
    // Should have at least background and some curves
    assert!(vectorized.background_color.3 == 255);

    // Cleanup
    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_example_images_exist() {
    let examples_dir = PathBuf::from("examples/input");

    if examples_dir.exists() {
        let entries = fs::read_dir(&examples_dir).expect("Failed to read examples dir");
        let mut count = 0;

        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("png") {
                    count += 1;
                }
            }
        }

        assert!(count > 0, "No example PNG images found in examples/input");
    } else {
        println!("Warning: examples/input directory not found, skipping example images test");
    }
}

#[test]
fn test_various_color_counts() {
    let test_img = PathBuf::from("/tmp/test_colors.png");

    for &num_colors in &[2, 4, 8, 16, 32] {
        create_test_png(&test_img, 40, 40, "gradient");

        let image_data = load_image(&test_img).expect("Failed to load image");
        let result = quantize_colors(&image_data, num_colors);

        assert!(result.is_ok(), "Failed to quantize with {} colors", num_colors);

        let quantized = result.unwrap();
        assert_eq!(quantized.width, 40);
        assert_eq!(quantized.height, 40);
    }

    // Cleanup
    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_smoothing_levels() {
    let test_img = PathBuf::from("/tmp/test_smoothing.png");
    create_test_png(&test_img, 40, 40, "circle");

    let image_data = load_image(&test_img).expect("Failed to load image");

    for &smooth_level in &[0, 1, 3, 5, 10] {
        let result = vectorize(&image_data, 4, 0.1, smooth_level, false);
        assert!(result.is_ok(), "Failed to vectorize with smooth level {}", smooth_level);
    }

    // Cleanup
    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_svg_output_format() {
    let test_img = PathBuf::from("/tmp/test_format.png");
    let test_svg = PathBuf::from("/tmp/test_format.svg");

    create_test_png(&test_img, 100, 100, "solid");

    let image_data = load_image(&test_img).expect("Failed to load image");
    let vectorized = vectorize(&image_data, 1, 0.1, 0, false).expect("Failed to vectorize");
    generate_svg(&vectorized, &test_svg).expect("Failed to generate SVG");

    let svg_content = fs::read_to_string(&test_svg).expect("Failed to read SVG");

    // Check for required SVG elements
    assert!(svg_content.contains(r#"xmlns="http://www.w3.org/2000/svg""#));
    assert!(svg_content.contains("viewBox="));
    assert!(svg_content.contains("<rect")); // Background
    assert!(svg_content.contains("</svg>"));

    // Cleanup
    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_svg);
}
