//! Compare original (median-cut + RDP) vs enhanced (Bézier) pipeline performance.
//!
//! Run: `cargo bench --bench pipeline_comparison`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use img2svg::image_processor::ImageData;
use img2svg::svg_generator::generate_svg_to;
use img2svg::vectorizer::vectorize;
use img2svg::{generate_enhanced_svg, vectorize_enhanced, EnhancedOptions};
use rgb::RGBA8;
use std::io::Cursor;

fn checkerboard(size: u32) -> ImageData {
    let w = size as usize;
    let h = size as usize;
    let mut pixels = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let on = ((x / 8) + (y / 8)) % 2 == 0;
            let v = if on { 255 } else { 0 };
            pixels.push(RGBA8::new(v, v, v, 255));
        }
    }
    ImageData {
        width: size,
        height: size,
        pixels,
    }
}

fn bench_original(c: &mut Criterion) {
    let mut group = c.benchmark_group("original_pipeline");
    group.sample_size(20);

    for size in [128u32, 256, 512] {
        let image = checkerboard(size);
        let pixels = (size * size) as u64;
        group.throughput(Throughput::Elements(pixels));

        group.bench_with_input(
            BenchmarkId::new("vectorize_and_svg", size),
            &image,
            |b, img| {
                b.iter(|| {
                    let vd = vectorize(black_box(img), 16, 0.1, 5, false).unwrap();
                    let mut buf = Cursor::new(Vec::new());
                    generate_svg_to(&vd, &mut buf).unwrap();
                    black_box(buf.into_inner().len())
                });
            },
        );
    }
    group.finish();
}

fn bench_enhanced(c: &mut Criterion) {
    let mut group = c.benchmark_group("enhanced_pipeline");
    group.sample_size(20);

    let opts = EnhancedOptions {
        num_colors: 16,
        preprocess: false,
        ..Default::default()
    };

    for size in [128u32, 256, 512] {
        let image = checkerboard(size);
        let pixels = (size * size) as u64;
        group.throughput(Throughput::Elements(pixels));

        group.bench_with_input(
            BenchmarkId::new("vectorize_and_svg", size),
            &image,
            |b, img| {
                b.iter(|| {
                    let data =
                        vectorize_enhanced(black_box(img), black_box(&opts)).unwrap();
                    let svg = generate_enhanced_svg(&data);
                    black_box(svg.len())
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_original, bench_enhanced);
criterion_main!(benches);
