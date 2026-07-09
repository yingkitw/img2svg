// Integration tests for img2svg
use img2svg::{
    batch_convert_enhanced, convert, convert_enhanced, convert_enhanced_to_svg_string,
    convert_to_svg_string, generate_enhanced_svg, vectorize_enhanced, write_enhanced_eps,
    write_enhanced_svg, ConversionOptions, EnhancedOptions, EnhancedOutputFormat,
};
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

#[test]
fn test_enhanced_pipeline_checkerboard() {
    let test_img = PathBuf::from("/tmp/img2svg_enhanced_checker.png");
    let test_svg = PathBuf::from("/tmp/img2svg_enhanced_checker.svg");

    create_test_png(&test_img, 40, 40, "checkerboard");

    let image_data = load_image(&test_img).expect("load");
    let opts = EnhancedOptions {
        num_colors: 4,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("vectorize_enhanced");
    assert_eq!(data.width, 40);
    assert_eq!(data.height, 40);

    write_enhanced_svg(&data, &test_svg).expect("write_enhanced_svg");
    let svg = fs::read_to_string(&test_svg).expect("read svg");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("</svg>"));

    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_svg);
}

#[test]
fn test_generate_enhanced_svg_string() {
    let test_img = PathBuf::from("/tmp/img2svg_enhanced_string.png");
    create_test_png(&test_img, 32, 32, "solid");

    let image_data = load_image(&test_img).expect("load");
    let opts = EnhancedOptions {
        num_colors: 2,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("vectorize_enhanced");
    let svg = generate_enhanced_svg(&data);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("viewBox=\"0 0 32 32\""));

    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_convert_legacy_writes_svg() {
    let test_img = PathBuf::from("/tmp/img2svg_lib_convert.png");
    let test_svg = PathBuf::from("/tmp/img2svg_lib_convert.svg");

    create_test_png(&test_img, 48, 48, "circle");

    let options = ConversionOptions {
        num_colors: 8,
        smooth_level: 3,
        threshold: 0.1,
        hierarchical: false,
        advanced: false,
    };
    convert(&test_img, &test_svg, &options).expect("convert");

    let svg = fs::read_to_string(&test_svg).expect("read");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));

    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_svg);
}

#[test]
fn test_convert_to_svg_string_legacy() {
    let test_img = PathBuf::from("/tmp/img2svg_convert_str.png");
    create_test_png(&test_img, 24, 24, "checkerboard");

    let image_data = load_image(&test_img).expect("load");
    let options = ConversionOptions {
        num_colors: 2,
        smooth_level: 0,
        threshold: 0.1,
        hierarchical: false,
        advanced: false,
    };
    let svg = convert_to_svg_string(&image_data, &options).expect("convert_to_svg_string");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("width=\"24\""));
    assert!(svg.contains("height=\"24\""));

    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_enhanced_auto_color_count_zero() {
    let test_img = PathBuf::from("/tmp/img2svg_enhanced_auto_k.png");
    create_test_png(&test_img, 20, 20, "gradient");

    let image_data = load_image(&test_img).expect("load");
    let opts = EnhancedOptions {
        num_colors: 0,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("auto k");
    assert_eq!(data.width, 20);
    assert_eq!(data.height, 20);

    let svg = generate_enhanced_svg(&data);
    assert!(svg.contains("<svg"));

    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_enhanced_autotrace_mode() {
    let test_img = PathBuf::from("/tmp/img2svg_enhanced_autotrace.png");
    // 4-color checkerboard pattern triggers logo-like autotune
    create_test_png(&test_img, 80, 80, "checkerboard");

    let image_data = load_image(&test_img).expect("load");
    let opts = EnhancedOptions {
        autotrace: true,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("autotrace");
    assert_eq!(data.width, 80);
    assert_eq!(data.height, 80);

    let svg = generate_enhanced_svg(&data);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));

    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_enhanced_geojson_export() {
    let test_img = PathBuf::from("/tmp/test_geojson.png");
    let test_geojson = PathBuf::from("/tmp/test_geojson_output.json");
    create_test_png(&test_img, 40, 40, "circle");

    let image = load_image(&test_img).expect("load");
    let opts = EnhancedOptions { preprocess: false, ..Default::default() };
    let data = vectorize_enhanced(&image, &opts).expect("vectorize");
    img2svg::write_enhanced_geojson(&data, &test_geojson).expect("write geojson");

    let s = fs::read_to_string(&test_geojson).unwrap();
    assert!(s.contains("FeatureCollection"));
    assert!(s.contains("Polygon"));

    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_geojson);
}

#[test]
fn test_enhanced_dxf_export() {
    let test_img = PathBuf::from("/tmp/test_dxf.png");
    let test_dxf = PathBuf::from("/tmp/test_dxf_output.dxf");
    create_test_png(&test_img, 40, 40, "circle");

    let image = load_image(&test_img).expect("load");
    let opts = EnhancedOptions { preprocess: false, ..Default::default() };
    let data = vectorize_enhanced(&image, &opts).expect("vectorize");
    img2svg::write_enhanced_dxf(&data, &test_dxf).expect("write dxf");

    let s = fs::read_to_string(&test_dxf).unwrap();
    assert!(s.contains("ENTITIES"));
    assert!(s.contains("LWPOLYLINE"));
    assert!(s.contains("EOF"));

    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_dxf);
}

#[test]
fn test_enhanced_gimppath_export() {
    let test_img = PathBuf::from("/tmp/test_gimppath.png");
    let test_gimppath = PathBuf::from("/tmp/test_gimppath_output.gimppath");
    create_test_png(&test_img, 40, 40, "circle");

    let image = load_image(&test_img).expect("load");
    let opts = EnhancedOptions { preprocess: false, ..Default::default() };
    let data = vectorize_enhanced(&image, &opts).expect("vectorize");
    img2svg::write_enhanced_gimppath(&data, &test_gimppath).expect("write gimppath");

    let s = fs::read_to_string(&test_gimppath).unwrap();
    assert!(s.contains("<svg"));
    assert!(s.contains("<path"));

    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_gimppath);
}

#[test]
fn test_load_jpeg_input() {
    let test_img = PathBuf::from("/tmp/img2svg_test_input.jpg");
    create_test_png(&test_img, 30, 30, "checkerboard");

    let image_data = load_image(&test_img).expect("Failed to load JPEG");
    assert_eq!(image_data.width, 30);
    assert_eq!(image_data.height, 30);
    assert_eq!(image_data.pixels.len(), 30 * 30);

    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_load_bmp_input() {
    let test_img = PathBuf::from("/tmp/img2svg_test_input.bmp");
    create_test_png(&test_img, 30, 30, "solid");

    let image_data = load_image(&test_img).expect("Failed to load BMP");
    assert_eq!(image_data.width, 30);
    assert_eq!(image_data.height, 30);
    assert_eq!(image_data.pixels.len(), 30 * 30);

    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_load_webp_input() {
    let test_img = PathBuf::from("/tmp/img2svg_test_input.webp");
    create_test_png(&test_img, 30, 30, "checkerboard");

    let image_data = load_image(&test_img).expect("Failed to load WebP");
    assert_eq!(image_data.width, 30);
    assert_eq!(image_data.height, 30);
    assert_eq!(image_data.pixels.len(), 30 * 30);

    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_load_animated_gif() {
    let test_gif = PathBuf::from("/tmp/img2svg_test_input.gif");

    // Create a simple static GIF (image crate handles GIF encoding via save_buffer)
    let mut pixel_data: Vec<u8> = Vec::with_capacity(10 * 10 * 4);
    for _ in 0..(10 * 10) {
        pixel_data.push(255); // R
        pixel_data.push(0);   // G
        pixel_data.push(0);   // B
        pixel_data.push(255); // A
    }
    image::save_buffer(&test_gif, &pixel_data, 10, 10, image::ColorType::Rgba8)
        .expect("Failed to create test GIF");

    // load_animated_gif should work for static GIFs too (returns single frame)
    let frames = img2svg::load_animated_gif(&test_gif).expect("Failed to load GIF");
    assert_eq!(frames.len(), 1, "Static GIF should return 1 frame");
    assert_eq!(frames[0].image_data.width, 10);
    assert_eq!(frames[0].image_data.height, 10);

    let first_pixel = frames[0].image_data.pixels[0];
    assert!(first_pixel.r > 200, "Frame should be red");

    let _ = fs::remove_file(&test_gif);
}

#[test]
fn test_original_and_enhanced_pipelines_same_image() {
    let test_img = PathBuf::from("/tmp/img2svg_pipeline_parity.png");
    create_test_png(&test_img, 64, 64, "checkerboard");

    let image_data = load_image(&test_img).expect("load");

    let original = vectorize(&image_data, 8, 0.1, 3, false).expect("original vectorize");
    let mut original_buf = Vec::new();
    img2svg::svg_generator::generate_svg_to(&original, &mut original_buf).expect("original svg");
    let original_svg = String::from_utf8(original_buf).expect("utf8");
    assert!(original_svg.contains("<svg"));
    assert!(original_svg.contains("width=\"64\""));

    let enhanced_opts = EnhancedOptions {
        num_colors: 8,
        preprocess: false,
        ..Default::default()
    };
    let enhanced =
        vectorize_enhanced(&image_data, &enhanced_opts).expect("enhanced vectorize");
    let enhanced_svg = generate_enhanced_svg(&enhanced);
    assert!(enhanced_svg.contains("<svg"));
    assert!(enhanced_svg.contains("viewBox=\"0 0 64 64\""));

    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_use_case_logo_converts() {
    let input = PathBuf::from("examples/input/logo.png");
    if !input.exists() {
        return;
    }
    let output = PathBuf::from("/tmp/img2svg_use_case_logo.svg");
    let image_data = load_image(&input).expect("load logo");
    let opts = EnhancedOptions {
        num_colors: 8,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("vectorize logo");
    write_enhanced_svg(&data, &output).expect("write logo svg");
    let svg = fs::read_to_string(&output).expect("read");
    assert!(svg.contains("<svg"));
    assert!(svg.to_lowercase().contains("4ecdc4"));
    let _ = fs::remove_file(&output);
}

#[test]
fn test_use_case_transparency_preserves_fill_opacity() {
    let input = PathBuf::from("examples/input/transparency.png");
    if !input.exists() {
        return;
    }
    let image_data = load_image(&input).expect("load transparency");
    assert!(
        image_data.pixels.iter().any(|p| p.a < 255),
        "transparency.png should contain non-opaque pixels"
    );
    let opts = EnhancedOptions {
        num_colors: 16,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("vectorize");
    let svg = generate_enhanced_svg(&data);
    assert!(
        svg.contains("fill-opacity") || svg.contains("stroke-opacity"),
        "SVG should preserve alpha as opacity attributes"
    );
}

#[test]
fn test_enhanced_superpixel_mode() {
    let test_img = PathBuf::from("/tmp/test_superpixel.png");
    create_test_png(&test_img, 40, 40, "checkerboard");
    let image_data = load_image(&test_img).expect("load");
    let opts = EnhancedOptions {
        num_colors: 8,
        use_superpixel: true,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("superpixel vectorize");
    assert!(!data.paths.is_empty());
    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_enhanced_centerline_mode() {
    let input = PathBuf::from("examples/input/line_art.png");
    if !input.exists() {
        return;
    }
    let image_data = load_image(&input).expect("load line art");
    let opts = EnhancedOptions {
        num_colors: 4,
        centerline: true,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("centerline vectorize");
    assert!(
        data.paths.iter().any(|p| p.stroke_only),
        "centerline mode should emit stroke-only paths"
    );
}

#[test]
fn test_enhanced_logo_mode() {
    let input = PathBuf::from("examples/input/logo.png");
    if !input.exists() {
        return;
    }
    let image_data = load_image(&input).expect("load logo");
    let opts = EnhancedOptions {
        num_colors: 8,
        logo_mode: true,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("logo mode vectorize");
    assert!(!data.paths.is_empty());
}

#[test]
fn test_enhanced_eps_export() {
    let test_img = PathBuf::from("/tmp/test_eps.png");
    let test_eps = PathBuf::from("/tmp/test_eps_output.eps");
    create_test_png(&test_img, 30, 30, "circle");
    let image_data = load_image(&test_img).expect("load");
    let opts = EnhancedOptions {
        num_colors: 8,
        preprocess: false,
        ..Default::default()
    };
    let data = vectorize_enhanced(&image_data, &opts).expect("vectorize");
    write_enhanced_eps(&data, &test_eps).expect("write eps");
    let eps = fs::read_to_string(&test_eps).expect("read eps");
    assert!(eps.contains("%!PS-Adobe"));
    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_eps);
}

#[test]
fn test_convert_enhanced_file_api() {
    let test_img = PathBuf::from("/tmp/test_convert_enhanced.png");
    let test_svg = PathBuf::from("/tmp/test_convert_enhanced_out.svg");
    create_test_png(&test_img, 32, 32, "circle");
    let opts = EnhancedOptions {
        num_colors: 8,
        preprocess: false,
        ..Default::default()
    };
    convert_enhanced(&test_img, &test_svg, &opts).expect("convert_enhanced");
    let svg = fs::read_to_string(&test_svg).expect("read");
    assert!(svg.contains("<svg"));
    let _ = fs::remove_file(&test_img);
    let _ = fs::remove_file(&test_svg);
}

#[test]
fn test_convert_enhanced_to_svg_string_api() {
    let test_img = PathBuf::from("/tmp/test_convert_enhanced_str.png");
    create_test_png(&test_img, 24, 24, "solid");
    let image = load_image(&test_img).expect("load");
    let opts = EnhancedOptions {
        num_colors: 4,
        preprocess: false,
        ..Default::default()
    };
    let svg = convert_enhanced_to_svg_string(&image, &opts).expect("to string");
    assert!(svg.contains("<svg"));
    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_despeckle_integration() {
    let test_img = PathBuf::from("/tmp/test_despeckle.png");
    create_test_png(&test_img, 20, 20, "checkerboard");
    let image = load_image(&test_img).expect("load");
    let with_speckle = EnhancedOptions {
        num_colors: 4,
        preprocess: false,
        min_region_area: 0,
        ..Default::default()
    };
    let without_despeckle = vectorize_enhanced(&image, &with_speckle).expect("vectorize");
    let with_despeckle = EnhancedOptions {
        min_region_area: 50,
        ..with_speckle
    };
    let despeckled = vectorize_enhanced(&image, &with_despeckle).expect("despeckle vectorize");
    assert!(despeckled.paths.len() <= without_despeckle.paths.len());
    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_enhanced_optimize_curves_reduces_segments() {
    let test_img = PathBuf::from("/tmp/test_optimize_curves.png");
    create_test_png(&test_img, 40, 10, "solid");
    let image = load_image(&test_img).expect("load");
    let base = EnhancedOptions {
        num_colors: 4,
        preprocess: false,
        min_region_area: 0,
        ..Default::default()
    };
    let plain = vectorize_enhanced(&image, &base).expect("plain");
    let optimized = vectorize_enhanced(
        &image,
        &EnhancedOptions {
            optimize_curves: true,
            ..base
        },
    )
    .expect("optimized");
    let plain_curves: usize = plain.paths.iter().map(|p| p.curves.len()).sum();
    let opt_curves: usize = optimized.paths.iter().map(|p| p.curves.len()).sum();
    assert!(opt_curves <= plain_curves);
    let _ = fs::remove_file(&test_img);
}

#[test]
fn test_batch_convert_enhanced_examples_input() {
    let input_dir = PathBuf::from("examples/input");
    if !input_dir.exists() {
        return;
    }
    let output_dir = PathBuf::from("/tmp/img2svg_batch_test_out");
    let _ = fs::remove_dir_all(&output_dir);
    let opts = EnhancedOptions {
        num_colors: 8,
        preprocess: false,
        ..Default::default()
    };
    let summary = batch_convert_enhanced(
        &input_dir,
        &output_dir,
        &opts,
        EnhancedOutputFormat::Svg,
        4096,
    )
    .expect("batch convert");
    assert!(summary.total >= 5);
    assert_eq!(summary.converted + summary.errors, summary.total);
    assert!(summary.converted >= 5);
    assert!(output_dir.join("simple.svg").exists());
    let _ = fs::remove_dir_all(&output_dir);
}
