# Architecture

## Module Structure

```
src/
├── main.rs                 # CLI entry point, single-file + batch mode orchestration
├── cli.rs                  # Clap argument parsing, presets, is_supported_image()
├── lib.rs                  # Library API with public exports + rustdoc
├── image_processor.rs      # Image loading, auto-resize (Lanczos3), median-cut quantization
├── vectorizer.rs           # Marching-squares contour tracing, smoothing, RDP, hole detection
├── gradient_detector.rs    # Linear and radial gradient detection for native SVG gradients
├── svg_generator.rs        # SVG file generation with merged subpaths, alpha, evenodd fill-rule, streaming writers
├── preprocessor.rs         # Image preprocessing (LUT bilateral filter, color reduction)
├── image_filters.rs        # Image filter re-exports + median/weighted median
├── image_filters/          # Per-category filter implementations
│   ├── canny.rs            # Canny edge detector
│   ├── color_ops.rs        # Color temperature, saturation, HSL, LAB
│   ├── contrast.rs         # Histogram equalization, CLAHE, adaptive tone mapping
│   ├── denoising.rs        # Non-local means, Perona-Malik anisotropic diffusion
│   ├── effects.rs          # Emboss, edge glow, sepia, vignette
│   ├── enhancement.rs      # Unsharp mask, Laplacian sharpen, high-boost, gamma
│   ├── morphology.rs       # Erosion, dilation, opening, closing, top-hat
│   ├── segmentation.rs     # Watershed, mean shift, SLIC superpixels
│   └── thresholding.rs     # Otsu, adaptive thresholding
├── despeckle.rs             # Potrace-style speckle / small-region removal
├── centerline_tracer.rs      # Skeletonization + centerline polyline tracing
├── logo_pipeline.rs        # Fill/outline layer separation for logos
├── eps_generator.rs        # EPS/PDF/AI vector export
├── tune.rs                 # HTML parameter comparison grid (--tune)
├── mcp.rs                  # MCP (Model Context Protocol) server
├── edge_detector.rs        # Sobel edge detection (enhanced pipeline)
├── enhanced_quantizer.rs   # K-means++ / k-means refinement, edge-aware quantization (enhanced)
├── region_extractor.rs     # 8-connectivity flood-fill, Moore contour tracing (enhanced)
├── path_simplifier.rs      # Visvalingam-Whyatt with corner preservation (enhanced)
├── bezier_fitter.rs        # Cubic Bézier fitting with Newton-Raphson (enhanced)
├── enhanced_vectorizer.rs  # Enhanced pipeline orchestrator (enhanced)
└── *_tests.rs              # Unit tests for each module

tests/
└── integration_tests.rs    # Integration tests for full pipeline

benches/
└── pipeline_comparison.rs  # Criterion: original vs enhanced pipeline timing

examples/
├── input/                  # Test input images (PNG)
│   ├── simple.png
│   ├── gradient.png
│   ├── medium.png
│   ├── complex.png
│   ├── very_complex.png
│   ├── lenna.png
│   ├── logo.png            # Flat brand logo (few colors)
│   ├── line_art.png        # Thin strokes / grid lines
│   ├── transparency.png    # RGBA / fill-opacity
│   └── icon.png            # 128×128 app icon
├── output/                 # Generated SVG files
│   ├── simple.svg
│   ├── gradient.svg
│   ├── medium.svg
│   ├── complex.svg
│   └── very_complex.svg
├── comparison.html         # Side-by-side original vs enhanced viewer
├── README.md               # Per-image purpose and recommended flags
└── convert_all.sh          # Batch conversion script
```

## Data Flow

### Standard Pipeline
```
PNG/JPEG → load_image → ImageData
         ↓
         [Optional: preprocess → PreprocessOptions.photo()]
         ↓
         quantize_colors (median-cut) → ImageData (reduced palette)
         ↓
         group pixels by color, sort by area (largest first)
         ↓
         per color: binary mask → marching_squares_contours → Vec<Vec<Point>>
         ↓
         [Optional: filter hole contours (signed_polygon_area > 0) if !hierarchical]
         ↓
         smooth_boundary (Gaussian averaging) → Vec<Point>
         ↓
         rdp_simplify (epsilon=2.0) → Vec<Point>
         ↓
         snap_to_edges (boundary detection) → Vec<Point>
         ↓
         merge subpaths into Curve { subpaths, color }
         ↓
         generate_svg → background rect + one <path> per color (+ fill-rule="evenodd" for holes, alpha channel support)
```

### Default Pipeline (Bézier curves)
```
PNG/JPEG → load_image → ImageData
         ↓
         resize_if_needed (Lanczos3 downscale if > --max-size)
         ↓
         [Auto: detect unique colors, adaptive color count (256/128/64)]
         ↓
         [Optional: LUT bilateral_filter (edge-preserving smoothing)]
         ↓
         detect_edges_sobel → EdgeMap (Sobel gradient magnitudes)
         ↓
         quantize_edge_aware (k-means++ init → k-means refine → majority-vote smoothing)
         ↓
         recolor_from_original (average original pixels per quantized region)
         ↓
         group pixels by quantized color → per-color binary mask
         ↓
         marching_squares_contours (shared with original pipeline, proven robust)
         ↓
         parallel per contour (rayon):
           thin stripe fast path (height/width < 2px → direct SVG rect via svg_override)
           OR full pipeline:
             smooth_with_corners (Gaussian, corner-preserving)
             → visvalingam_whyatt (area-based simplification, corner-preserving)
             → snap_to_edges (boundary snapping)
             → inject_image_corners (insert 90° corner points at edge transitions)
             → dedup_consecutive (remove near-duplicate points from snapping)
             → BezierFitter::fit_path (corner-aware splitting → cubic Bézier + Newton-Raphson)
             → clamp control points to image bounds
         ↓
         sort by area (back-to-front), detect_background_color (border pixels)
         ↓
         generate_enhanced_svg → L for lines, C for curves, collinear merge, gap-filling strokes
         ↓
         [Batch → PDF only] write_multi_page_pdf → one MediaBox-sized page per image,
           cross-reference table with correct byte offsets
```

### Preprocessing Pipeline (for photographs)
```
ImageData → bilateral_filter (edge-preserving smoothing)
         ↓
         reduce_colors (posterization)
         ↓
         ImageData (cleaner regions, less noise)
         ↓
         [continues to standard or enhanced pipeline]
```

## Key Algorithms

### Original Pipeline Algorithms
- **Median-cut quantization**: Recursively splits color space along widest channel
- **Marching squares**: Traces sub-pixel contours on binary mask per color (outer boundaries + holes)
- **Signed polygon area**: Distinguishes outer boundaries (CW, negative area) from holes (CCW, positive area)
- **Hole filtering** (non-hierarchical): Discards hole contours to prevent incorrect hole filling
- **Point-in-polygon**: Ray-casting test for `has_holes` detection
- **Gaussian smoothing**: Neighbor averaging that preserves point count
- **RDP simplification**: Ramer-Douglas-Peucker with epsilon=2.0
- **Background detection**: Largest-area color becomes SVG rect (not traced)
- **Subpath merging**: All contours of same color → single `<path>` with `M...Z` subpaths
- **Evenodd fill-rule**: Applied when holes are present in hierarchical mode
- **Alpha channel**: Rendered as `fill-opacity`/`stroke-opacity` when alpha < 255
- **Zero-length segment removal**: Duplicate consecutive formatted coordinates are skipped in path strings
- **Streaming SVG generation**: `generate_svg_to` writes directly to any `Write` implementor without intermediate strings

### Enhanced Pipeline Algorithms (merged from vec project)
- **K-means++ initialization**: Probabilistic centroid selection proportional to distance²
- **K-means refinement**: 8 iterations with perceptual color distance (R=2, G=4, B=3)
- **Sobel edge detection**: Gradient-based edge map for edge-aware quantization
- **Edge-aware quantization**: Majority-vote smoothing on non-edge pixels (3×3 window, 2 passes)
- **Marching squares contours**: Shared with original pipeline for robust sub-pixel contour extraction
- **Corner-aware Bézier splitting**: 30° turn threshold detects sharp corners from marching squares chamfers
- **Visvalingam-Whyatt simplification**: Area-based point removal with corner preservation
- **Harris-like corner detection**: Multi-scale corner response with non-maximum suppression
- **Cubic Bézier fitting**: Least-squares fit with Newton-Raphson reparameterization
- **G1 continuity**: Smooth tangent transitions between adjacent Bézier curves
- **Control point clamping**: Prevents overshoot beyond data bounds (15% margin)
- **Gap-filling strokes**: Thin stroke matching fill color eliminates visible seams
- **Color grouping**: Consecutive same-color paths merged into compound paths
- **Border background detection**: Most frequent color along image border pixels
- **SVG L/C optimization**: Uses `L` for linear segments, `C` for true Bézier curves
- **Distance-based collinear merge**: Merges consecutive `L` segments within 1.5px of a line
- **Adaptive simplification**: 2× tolerance for photos (many colors), 1.5 for graphics
- **Image corner injection**: Detects edge transitions after snapping, inserts exact 90° corner points
- **Consecutive point dedup**: Removes near-duplicate points from snapping that break angle detection
- **Thin stripe fast path**: Contours < 2px height/width → direct SVG rect via `svg_override`
- **Adaptive smoothing passes**: 4 passes for complex graphics (17-1000 colors), 2 for photos
- **Alpha channel support**: `fill-opacity`/`stroke-opacity` emitted for partially transparent colors
- **Evenodd fill-rule**: `fill-rule="evenodd"` applied when hole subpaths are present
- **Progress logging**: Stage-by-stage `eprintln!` output during vectorization for large images
- **Incremental SVG generation**: `generate_enhanced_svg_to` writes directly to `Write` implementors
- **Adjustable coordinate precision**: `EnhancedOptions.coordinate_precision` (1–6, default 2) flows through the `bezier_to_svg_path_to_prec` writer chain; non-SVG formats and thin-stripe rects (integers) are unaffected
- **Parameterized curve merging**: `EnhancedOptions.optimize_tolerance` (default 1.5) controls Potrace `-O` style merging via `optimize_bezier_curves_with_tolerance`
- **Native gradient fills**: opt-in `EnhancedOptions.detect_gradients` runs `gradient_detector` per color mask against the **original** image and emits `<defs>` `<linearGradient>`/`<radialGradient>` fills (`url(#img2svg-grad-N)`), deduplicated and emitted before all paths
- **Zero-length segment removal**: Duplicate formatted coordinates skipped in `create_subpath_string`

### New Features (merged from vec project)
- **Batch directory processing**: `img2svg -i dir/ -o out/` converts all supported images
- **Auto-resize**: Lanczos3 downscale for images exceeding `--max-size` (default 4096)
- **Original recoloring**: Each quantized region recolored with average original pixel color
- **LUT bilateral filter**: Precomputed range-weight LUT with fixed-point arithmetic (radius=2)
- **Hierarchical decomposition**: `--hierarchical` preserves hole contours with `fill-rule="evenodd"`
- **Weighted median filter**: Gaussian-like weighting (center=4, edge=2, corner=1)
- **High-boost filter**: Aggressive sharpening via `original + boost*(original - blurred)`
- **Adaptive tone mapping**: Per-pixel local gamma based on neighborhood mean luminance
- **Rand 0.10 compatibility**: `rng()` / `random_range()` / `random()` API (was `thread_rng()` / `gen_range()` / `gen()`)
- **SmartThreshold**: Auto-selects Otsu vs adaptive thresholding based on histogram separability measure
- **Zero-length segment removal**: Skips duplicate formatted coordinates in SVG path strings
- **Gradient detection**: `detect_gradient` identifies linear and radial gradients via PCA + linear regression per channel for native SVG gradient rendering
- **Animated GIF support**: `load_animated_gif` loads all frames with delays; CLI outputs numbered SVG files (`output_001.svg`, etc.)
- **CMYK color space support**: `cmyk_to_rgb` converts CMYK to RGB; `detect_color_space` inspects JPEG APP14 markers to detect CMYK source images
- **Fully streaming enhanced pipeline**: `generate_enhanced_svg_to` streams paths directly with zero-allocation path writing (`bezier_to_svg_path_to`), no intermediate `ColorGroup` collection or `path_data` strings
- **Multi-page PDF writer**: `write_multi_page_pdf` emits a proper PDF 1.4 document (Catalog → Pages tree → per-page Page + Content stream objects) with a correct cross-reference table. Each page's `MediaBox` matches its source image dimensions and the y-axis is flipped during path emission (`c`/`l`/`h` operators). `write_enhanced_pdf` is a single-page wrapper. `batch_convert_to_pdf` vectorizes every image in a directory and combines the results into one PDF

### Preprocessing Algorithms
- **LUT Bilateral filter**: Fast edge-preserving smoothing
  - Precomputed 256-bin range weight LUT (fixed-point 10-bit)
  - Radius 2, color sigma configurable
  - Much faster than naive Gaussian bilateral (no exp() in hot loop)
- **Color reduction**: Posterization to reduce color noise
  - Quantizes each channel to fewer levels
  - Creates larger, more uniform color regions

## Exposed APIs

### CLI Tool
```bash
img2svg -i input.png -o output.svg --colors 16 --smooth 5 --threshold 0.1
```

### Library API
```rust
use img2svg::{convert_enhanced, convert, ConversionOptions, EnhancedOptions, batch_convert_to_pdf};

// Default pipeline (matches CLI without --original)
convert_enhanced(Path::new("input.png"), Path::new("output.svg"), &EnhancedOptions::default())?;

// Legacy pipeline (matches CLI --original)
convert(Path::new("input.png"), Path::new("output.svg"), &ConversionOptions::default())?;

// Combine a directory of images into one multi-page PDF
// (matches `img2svg -i dir/ -o combined.pdf`)
batch_convert_to_pdf(
    Path::new("examples/input"),
    Path::new("combined.pdf"),
    &EnhancedOptions::default(),
    4096,
)?;
```

### MCP Server
```json
{
  "mcpServers": {
    "img2svg": {
      "command": "/path/to/img2svg-mcp"
    }
  }
}
```

**Tool: `convert_image_to_svg`**
- `input_path`, `output_path` (required)
- **Default pipeline**: enhanced Bézier (matches CLI without `--original`)
- `legacy` (optional): Use median-cut / RDP line-segment pipeline
- `num_colors`, `preprocess`, `preset`, `centerline`, `logo_mode`, `superpixel`, `optimize_corners`, `output_format`
- `smooth_level`, `threshold` (legacy pipeline only)
- `max_size` (optional): Auto-resize images exceeding this dimension (default 4096) to prevent OOM
- `progress_token` (optional): Receives real-time `notifications/progress` during conversion
- **Response details**: Includes `pipeline`, `output_format`, `output_size_bytes`, `width`, `height`, `colors_used`

**Tool: `convert_image_directory`**
- `input_dir`, `output_dir` (required)
- Enhanced pipeline only; same optional flags as single-file convert
- Response `details`: `total`, `converted`, `errors`, `files`

**Tool: `convert_image_directory_to_pdf`**
- `input_dir`, `output_path` (required; `.pdf` file path, not a directory)
- Enhanced pipeline only; same optional flags as single-file convert (no `output_format`)
- Vectorizes every image in `input_dir` and combines into one multi-page PDF (one page per image, sized to its dimensions)
- Response `details`: `pipeline`, `output_format` (`pdf`), `output_size_bytes`, `output_path`, `total`, `converted`, `errors`, `files`

## Dependencies

- `image` (0.24) — raster image loading (PNG, JPEG, BMP, TIFF, WebP, etc.)
- `clap` (4.0) — CLI argument parsing
- `svg` (0.13) — SVG document building helpers
- `serde` / `serde_json` — optional, behind the `mcp` feature; serialization for MCP protocol
- `rgb` (0.8) — RGBA8 pixel type
- `anyhow` — Error handling
- `thiserror` — Error derive macros
- `rayon` (1.10) — Parallel path processing (enhanced pipeline)
- `rand` (0.10) — K-means++ random centroid selection (enhanced pipeline)

### Test Dependencies
- `criterion` (0.5, dev-dependency) — benchmark harness for `benches/pipeline_comparison.rs`
- Test images in `examples/input/`
- 486 test executions: 206 lib + 222 bin + 17 MCP + 38 integration + 3 doctests (lib and bin targets share most unit-test modules; `cli.rs` and now `gradient_detector` tests run in both targets). Run with `cargo test --features mcp` to include the MCP binary tests

## Performance Characteristics

### Time Complexity
- Color quantization: O(n * log(n)) where n = number of pixels
- Marching squares: O(n) where n = number of pixels
- RDP simplification: O(m^2) worst case, O(m log m) average, where m = points in contour
- Overall: O(n log n + c*m log m) where c = number of contours

### Space Complexity
- Image data: O(n) where n = width * height
- Binary mask per color: O(n)
- Contour points: O(p) where p = perimeter of color regions (typically << n)
- SVG output: O(c + t) where c = number of colors, t = total path points

### Scalability
- Auto-resize for images > 4096px (configurable via `--max-size`)
- Batch mode for directory processing
- Memory efficient: auto-resize prevents OOM on very large images
- Typical 1000x1000 image converts in <1 second
