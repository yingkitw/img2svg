# img2svg Specification

## Purpose
Convert raster images (PNG, JPEG, etc.) to scalable vector graphics (SVG) with filled color regions.

## Input
- Raster image file (PNG, JPEG, GIF, BMP, ICO, TIFF, WebP, PNM, TGA, DDS, Farbfeld via `image` crate)
- CLI Parameters:
  - `--input` / `-i`: Input image file or directory (batch mode)
  - `--output` / `-o`: Output SVG file or directory
  - `--max-size`: Auto-resize images exceeding this dimension (default: 4096)
  - `--colors` / `-c`: Number of colors for quantization (1-64, default: 16)
  - `--threshold` / `-t`: Edge detection threshold (0.0-1.0, default: 0.1)
  - `--smooth` / `-s`: Path smoothing level (0-10, default: 5)
  - `--preprocess` / `-p`: Apply edge-preserving smoothing and color reduction
  - `--original`: Use original pipeline (line segments, RDP simplification)
  - `--hierarchical`: Enable hierarchical decomposition (original pipeline only)
  - `--advanced` / `-a`: Use advanced SVG generation (original pipeline only)
  - `--preset`: Tuned parameter bundle — `logo`, `photo`, or `icon` (explicit `-c`/`-s`/`-t`/`-p` override preset values)
  - `--superpixel`: SLIC superpixel segmentation instead of k-means quantization
  - `--centerline`: Centerline/stroke tracing for line art
  - `--optimize-corners`: Potrace-style Bézier corner optimization
  - `--optimize-curves`: Potrace `-O` style merge of nearly-linear curve segments
  - `--logo-mode`: Separate fill regions and outline strokes (auto-enabled for `--preset logo`)
  - `--min-region-area`: Despeckle threshold in pixels (default 20; Potrace turdsize equivalent)
  - `--no-despeckle`: Disable despeckle pass
  - `--tune`: Generate HTML parameter comparison grid
  - `--progress`: Show stage-based progress bar on stderr during vectorization
  - `--autotrace`: Auto-select tuning parameters (colors, smoothing, simplification, preprocess, optimizations) based on image characteristics
  - `--format`: Output format — `svg`, `eps`, `pdf`, or `ai` (default `svg`; inferred from `-o` extension)

### MCP Tool (`convert_image_to_svg`)
- Default pipeline: **enhanced Bézier** (matches CLI without `--original`)
- `input_path`, `output_path` (required)
- `legacy` (boolean, default false): median-cut / RDP line-segment pipeline
- `num_colors`, `preprocess` (default true), `preset` (`logo`|`photo`|`icon`)
- `centerline`, `logo_mode`, `superpixel`, `optimize_corners` (booleans)
- `output_format`: `svg`, `eps`, `pdf`, `ai`, `dxf`, `geo-json`, or `gimppath`
- `smooth_level`, `threshold` (legacy only)
- `max_size`, `progress_token`
- Response `details`: `pipeline`, `output_format`, `output_size_bytes`, `width`, `height`, `colors_used`

### MCP Tool (`convert_image_directory`)
- Batch convert all supported images in `input_dir` → `output_dir` (enhanced pipeline only)

### MCP Tool (`convert_image_directory_to_pdf`)
- Combine all supported images in `input_dir` → one multi-page PDF at `output_path` (enhanced pipeline only)
- `input_dir`, `output_path` (required; output must be a `.pdf` file path, not a directory)
- Same optional flags as `convert_image_directory` (no `output_format`)
- Response `details`: `pipeline`, `output_format` (`pdf`), `output_size_bytes`, `output_path`, `total`, `converted`, `errors`, `files`
- `input_dir`, `output_dir` (required); same optional flags as single-file (except `legacy`)
- Response `details`: `total`, `converted`, `errors`, `files[]` with per-file `success` / `error`

## Output
- Vector file: SVG (default), EPS, minimal PDF, Illustrator-compatible AI wrapper, DXF, GeoJSON, or GIMP path
- Background color auto-detected and rendered as `<rect>`
- One `<path>` per non-background color, using merged subpaths
- ViewBox and dimensions match input image
- **Multi-page PDF** (`write_multi_page_pdf`, `batch_convert_to_pdf`, CLI `img2svg -i dir/ -o combined.pdf`, MCP `convert_image_directory_to_pdf`): one image per page, MediaBox sized to each image's dimensions, correct xref cross-reference table

## Pipeline

### Default Pipeline (Bézier curves)
1. **Load** → `image_processor::load_image` → `ImageData`
2. **Auto-resize** → `image_processor::resize_if_needed` → Lanczos3 downscale if > `--max-size`
3. **[Auto] Detect** → Count unique colors, determine adaptive target (256/128/64)
4. **[Auto] Preprocess** → LUT bilateral filter for photos (auto-detected when >1000 colors)
5. **Edge detect** → `edge_detector::detect_edges_sobel` → Sobel gradient edge map
6. **Quantize** → `enhanced_quantizer::quantize_edge_aware` → K-means++ init → k-means refine → majority-vote smoothing
7. **Recolor** → Average original pixels per quantized region for richer color fidelity
8. **Contour** → `vectorizer::marching_squares_contours` → Per-color binary mask → sub-pixel contours
9. **[Fast path]** Thin stripes (< 2px) → direct SVG rect via `svg_override`
10. **[Parallel] Smooth** → `path_simplifier::smooth_with_corners` → Gaussian smoothing preserving corners
11. **[Parallel] Simplify** → `path_simplifier::visvalingam_whyatt` → Area-based simplification with corner preservation
12. **[Parallel] Snap** → Edge snapping + `inject_image_corners` + `dedup_consecutive`
13. **[Parallel] Fit** → `bezier_fitter::BezierFitter::fit_path` → Cubic Bézier with Newton-Raphson + G1 continuity
14. **SVG emit** → `enhanced_vectorizer::generate_enhanced_svg` → Background rect + gap-filling strokes + color grouping

### Original Pipeline (--original flag)
1. **Load** → `image_processor::load_image` → `ImageData { width, height, pixels: Vec<RGBA8> }`
2. **[Optional] Preprocess** → `preprocessor::preprocess` → Bilateral filter + color reduction
3. **Quantize** → `image_processor::quantize_colors` → Median-cut algorithm
4. **Group** → Group pixels by quantized color, sort by area (largest = background)
5. **Contour** → `vectorizer::marching_squares_contours` → Per-color binary mask → sub-pixel contours (outer + hole contours)
6. **Filter** → If not `--hierarchical`, discard hole contours (positive signed area) to avoid filling holes
7. **Smooth** → `vectorizer::smooth_boundary` → Gaussian neighbor averaging
8. **Simplify** → `vectorizer::rdp_simplify` → Ramer-Douglas-Peucker with epsilon=2.0
9. **SVG emit** → `svg_generator::generate_svg` → Background rect + merged line-segment paths (with `fill-rule="evenodd"` when holes are present)

### Preprocessing Mode (--preprocess)
Applied before quantization for photographs:
1. **LUT Bilateral filter**: Fast edge-preserving smoothing with precomputed range-weight LUT (σ_color=40.0, radius=2, 2 iterations)
2. **Color reduction**: Posterization to ~128 color levels (50% reduction)

## Quality Criteria

### Functional Requirements
- ✅ All test images in `examples/input/` convert without panics
- ✅ SVG output contains filled regions (not just strokes)
- ✅ Proper z-ordering: background rect, then colors by decreasing area
- ✅ Edge snapping: boundary points snap to image edges
- ✅ Path simplification removes redundant points while preserving shape
- ✅ 455 test executions passing (197 lib + 202 bin + 16 MCP + 37 integration + 3 doctests); 249+ unique tests (run with `cargo test --features mcp` to include the MCP binary tests)

### Performance Targets
- Simple graphics (50x50): <100ms conversion time
- Medium graphics (100x100): <500ms conversion time
- Large images (1000x1000): <1s conversion time
- Memory usage: <10x input image size

### File Size Targets
- Simple logo: SVG smaller or comparable to PNG (better compression)
- Medium complexity: SVG 1-2x PNG size (acceptable for vector benefits)
- Complex illustrations: SVG up to 3x PNG (trade-off for scalability)

## Supported Features

| Feature | Status | Notes |
|---------|--------|-------|
| PNG input | ✅ | Full support |
| JPEG input | ✅ | Full support via image crate |
| BMP input | ✅ | Full support via image crate |
| WebP input | ✅ | Full support via image crate |
| TIFF input | ✅ | Full support via image crate |
| Transparency/Alpha | ✅ | Preserved as `fill-opacity`/`stroke-opacity` in SVG |
| CMYK colorspace | ✅ | JPEG APP14 detection + `cmyk_to_rgb` conversion |
| Grayscale images | ✅ | Converted to RGB (R=G=B) |
| Animated GIF | ✅ | Frame-by-frame numbered SVG output (CLI) |

## Test Suite

### Unit Tests (273 unique tests total)
- `image_processor_tests.rs`: ~25 tests
  - Image data creation and validation
  - Color quantization with various parameters
  - Edge cases (empty, single color, zero colors)
  - Median-cut algorithm tests
  - Animated GIF loading, CMYK conversion, color space detection
- `vectorizer_tests.rs`: ~41 tests
  - Marching squares contour tracing
  - Polygon area calculation (signed/unsigned)
  - Point-in-polygon detection
  - Hole detection (`has_holes`)
  - Hierarchical vs non-hierarchical mode
  - RDP simplification
  - Boundary smoothing
  - Full vectorization pipeline
- `svg_generator_tests.rs`: ~26 tests
  - SVG generation with various inputs
  - Alpha channel output (`fill-opacity`)
  - Hierarchical evenodd fill-rule
  - Coordinate formatting
  - Subpath string creation
  - Multi-path handling
  - Streaming writer-based generation
- `image_filters/*`: ~50 tests across Canny, color ops, contrast, effects, enhancement, morphology, segmentation, thresholding, denoising
- `enhanced_vectorizer.rs`: ~18 tests
  - Enhanced pipeline vectorization
  - SVG output including alpha and streaming
  - Progress-bar path
  - Autotrace heuristic selection (logo, icon, photo)
- `extra_backends.rs`: 4 tests
  - Bézier flattening
  - GeoJSON, DXF, and Gimppath writers
- `lib.rs`: 3 tests
  - Conversion options defaults
  - `is_supported_image`
  - `convert_enhanced_to_svg_string`
- `cli.rs`: 13 tests
  - Preset defaults and override semantics
  - Supported image extension checks
  - `--progress` and `--autotrace` flag parsing
  - `--format` inference and explicit format parsing
- `mcp.rs`: 14 tests
  - Server initialization, tool schema, validation, enhanced default, legacy, preset, EPS export, batch directory, progress

### Integration Tests (34 total)
- Full pipeline conversion tests (legacy + enhanced)
- Test image creation (gradient, checkerboard, circle, solid)
- SVG output validation
- `convert_to_svg_string` round-trip
- Enhanced pipeline with auto color count
- Various color counts and smoothing levels
- File size and dimension preservation

## Example Outputs

### simple.png (50x50, basic shapes)
```
Command: img2svg -i simple.png -o simple.svg
Result: Clean vector shapes, perfect edges
Colors: 8
File size: PNG 2KB → SVG ~1-2KB
```

### gradient.png (100x100, smooth gradient)
```
Command: img2svg -i gradient.png -o gradient.svg -c 16 -s 5
Result: Smooth color transitions with minimal banding
Colors: 16
File size: PNG 1KB → SVG ~1KB
```

### complex.png (200x200, detailed)
```
Command: img2svg -i complex.png -o complex.svg -c 32 -s 7
Result: Fine details preserved, clean paths
Colors: 32
File size: PNG 6KB → SVG ~6KB
```

### lenna.png (512x512, photograph)
```
Without preprocessing: 87KB SVG, 15 paths, 16 colors (posterized)
With --preprocess: 28KB SVG, 11 paths, 12 colors (cleaner)

Recommended: img2svg -i lenna.png -o lenna.svg --preprocess -c 12
```

## API Stability

### Stable (v0.1.0)
- `load_image`
- `quantize_colors`
- `vectorize` (hierarchical mode fully functional)
- `generate_svg` (alpha channel + evenodd fill-rule support, zero-length segment removal)
- `generate_svg_advanced`
- `generate_svg_to` / `write_subpath_to` / `write_multi_path_to` (streaming writer-based SVG generation for original pipeline)
- `generate_enhanced_svg_to` (incremental writer-based SVG generation for enhanced pipeline)
- `convert`
- `convert_to_svg_string`
- `convert_enhanced` / `convert_enhanced_to_svg_string` / `convert_enhanced_data`
- `batch_convert_enhanced` / `is_supported_image`
- `despeckle` / `min_region_area` on `EnhancedOptions`
- `optimize_bezier_corners` / `optimize_bezier_curves`
- `write_enhanced_output` / `EnhancedOutputFormat`
- Gradient detection (`detect_gradient`, `detect_linear_gradient`, `detect_radial_gradient`)
- Animated GIF loading (`load_animated_gif`, `GifFrame`) and per-frame SVG conversion (`convert_animated_gif`)
- CMYK color space support (`cmyk_to_rgb`, `image_data_from_cmyk`, `detect_color_space`, `ColorSpace`)
- Image filters (`median_filter`, `weighted_median_filter`, `CannyEdgeDetector`, `UnsharpMask`, `HighBoostFilter`, `HistogramEqualizer`, `CLAHE`, `AdaptiveToneMapper`, `SmartThreshold`, etc.)

### Experimental
- `preprocess` API (may change parameters)
- Advanced SVG generation with layers
- MCP real-time progress notifications (notifications/progress)
