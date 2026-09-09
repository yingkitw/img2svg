# TODO

## Done ✅

### Core Functionality
- [x] Fix index out of bounds crash in vectorizer.rs
- [x] Fix convert_all.sh (named CLI args, set -e arithmetic, macOS date)
- [x] Rewrite vectorizer: marching-squares contour tracing (replaces edge-tracing + Moore walk)
- [x] Median-cut color quantization (replaces naive frequency-based top-N)
- [x] Merge all contours per color into single `<path>` with subpaths
- [x] Background color auto-detection → SVG `<rect>` (skip tracing largest region)
- [x] Compact SVG output: integer coords, line segments, minimal formatting
- [x] Gaussian smoothing (point-count preserving, replaces Chaikin which doubled points)
- [x] RDP path simplification with epsilon=2.0
- [x] Fix edge gaps: expanded grid (width+2)x(height+2) + edge snap after smoothing/RDP
- [x] Filter degenerate subpaths (near-zero area after snapping)
- [x] All 5 test images converting successfully with good quality

### Testing
- [x] Unit tests for image_processor (~25 tests)
- [x] Unit tests for vectorizer (~41 tests)
- [x] Unit tests for svg_generator (~26 tests)
- [x] Integration tests (38 tests): legacy pipeline, **enhanced pipeline**, **`convert` / `convert_to_svg_string`**, **`convert_enhanced` / `convert_enhanced_to_svg_string`**, **`batch_convert_enhanced`**, auto `num_colors`, **original vs enhanced parity**, **use-case examples** (logo, transparency), **superpixel/centerline/logo/EPS/autotrace/DXF/GeoJSON/Gimppath**, **coordinate precision**, **animated GIF / CMYK / WebP / BMP / JPEG**
- [x] CLI unit tests: `is_supported_image`, **`--preset`** parsing and overrides, **`--progress`** / **`--autotrace`** flags, **`--format`** inference (13 tests, `src/cli.rs`)
- [x] MCP server unit tests: initialize, tools/list schema, tools/call validation and happy path (14 tests, `src/mcp.rs`)
- [x] MCP enhanced pipeline as default; `legacy`, `preset`, mode flags, and `output_format` parameters
- [x] MCP batch tool `convert_image_directory` + library `batch_convert_enhanced()`
- [x] Library API tests (3 doctests + 3 unit tests)
- [x] Run `cargo test` for full counts (203 lib + 219 bin + 17 `img2svg-mcp` + 38 integration + 3 doctests = 480 test executions; `cargo test --features mcp` for the MCP binary)
- [x] Criterion benchmark `benches/pipeline_comparison.rs` (original vs enhanced at 128/256/512 px)

### Project Structure
- [x] Library API (lib.rs)
- [x] CLI tool (main.rs)
- [x] MCP server (mcp.rs)
- [x] Three-in-one codebase (single Cargo.toml)
- [x] Comprehensive README documentation

### Photograph Support
- [x] Photo detection hint (>10,000 unique colors)
- [x] Image preprocessing module (preprocessor.rs)
- [x] Bilateral filter for edge-preserving smoothing
- [x] Color reduction (posterization) for cleaner regions
- [x] --preprocess CLI flag

### Enhanced Pipeline (merged from vec project)
- [x] K-means++ initialization + k-means refinement (8 iterations) for better palette
- [x] Perceptual color distance weighting (R=2, G=4, B=3)
- [x] Sobel edge detection for edge-aware quantization
- [x] Edge-aware quantization with majority-vote smoothing (multi-pass, 3×3 window)
- [x] Marching-squares contour extraction (shared with original pipeline, proven robust)
- [x] Corner-aware Bézier fitting (30° threshold detects sharp turns from marching squares)
- [x] Visvalingam-Whyatt simplification with corner preservation (replaces RDP-only)
- [x] Cubic Bézier fitting with Newton-Raphson reparameterization (replaces line segments)
- [x] G1 continuity enforcement between adjacent curves
- [x] Control point clamping to prevent overshoot
- [x] Gap-filling strokes + consecutive same-color path grouping in SVG output
- [x] Border-pixel frequency background detection (replaces largest-area)
- [x] Recoloring from original image for true color accuracy
- [x] Adaptive color count based on image size (256/128/64)
- [x] Rayon parallel path processing (smooth → simplify → Bézier fit)
- [x] Bézier pipeline promoted to default; --original flag for legacy pipeline
- [x] 15+ unit tests for enhanced pipeline modules
- [x] SVG size optimization: L for linear curves, C for true Bézier, collinear L merge
- [x] Distance-based collinear merge catches diagonal staircases from marching squares
- [x] Adaptive simplification tolerance (2x for photos, 1.5 for graphics)
- [x] Visual comparison page with lenna.png, 6 test images total
- [x] Use-case example images: logo, line_art, transparency, icon (10 images in `examples/input/`)
- [x] CLI `--preset logo|photo|icon` with tuned defaults and override semantics
- [x] Control point clamping to image bounds (fixes corner bulging in medium.png)
- [x] Auto-preprocessing for photos in enhanced pipeline (bilateral filter + color reduction)
- [x] Image corner injection: detect edge transitions after snapping, insert 90° corner points
- [x] Consecutive point dedup after snapping (fixes fitter angle detection at image corners)
- [x] Thin stripe fast path: contours < 2px height/width → direct SVG rect via svg_override
- [x] Adaptive majority-vote smoothing (4 passes for complex graphics, 2 for photos)
- [x] Batch directory processing (`-i <dir> -o <dir>`)
- [x] Large image auto-resize (`--max-size`, default 4096, Lanczos3)
- [x] Recoloring from original image (average original pixels per quantized region)
- [x] Fast LUT-based bilateral filter (precomputed range-weight LUT, fixed-point arithmetic)
- [x] Handle transparency/alpha channel in SVG output (preserve as opacity)
- [x] Add hierarchical decomposition mode (hole detection + evenodd fill-rule)
- [x] Progress logging for large image conversions
- [x] Weighted median filter
- [x] High-boost filtering
- [x] Adaptive tone mapping
- [x] Rand 0.10 API compatibility fix (`rng()` / `random_range` / `random`)
- [x] Incremental SVG generation for enhanced pipeline (writer-based `generate_enhanced_svg_to`)
- [x] SVG path string optimization (zero-length segment removal in `create_subpath_string`)
- [x] Integration tests for JPEG and BMP input formats
- [x] SmartThreshold: auto-selects Otsu vs adaptive based on histogram separability
- [x] WebP input format integration test
- [x] MCP `preprocess` parameter (uses enhanced Bézier pipeline when enabled)
- [x] MCP detailed result info (pipeline, output_size_bytes, width, height, colors_used)
- [x] Streaming SVG generation for original pipeline (`generate_svg_to`, `write_subpath_to`, `write_multi_path_to`)
- [x] Refactor `convert_to_svg_string` to use streaming `generate_svg_to`
- [x] SVG gradient detection module (`detect_gradient`, `detect_linear_gradient`, `detect_radial_gradient`)
- [x] Support for animated GIF input (frame-by-frame conversion to numbered SVGs)
- [x] Support for CMYK color space input (`cmyk_to_rgb`, `detect_color_space`, `ColorSpace`)
- [x] Library `convert_enhanced()` / `convert_enhanced_to_svg_string()` / `convert_enhanced_data()` matching CLI default pipeline
- [x] Superpixel segmentation as alternative to color quantization (`--superpixel`, SLIC + `quantize_superpixel`)
- [x] Interactive parameter tuning with HTML preview grid (`--tune`, `tune::generate_tune_html`)
- [x] Export to other vector formats (`--format eps|pdf|ai`, `eps_generator` module)
- [x] Non-local means (NLM) denoising (`NonLocalMeans` in `image_filters/denoising.rs`)
- [x] Anisotropic diffusion (Perona-Malik) (`PeronaMalik` in `image_filters/denoising.rs`)
- [x] Watershed segmentation (`watershed_segment` in `image_filters/segmentation.rs`)
- [x] Mean shift segmentation (`mean_shift_filter`, SLIC superpixels in `image_filters/segmentation.rs`)
- [x] Corner optimization / curve optimization pass (`--optimize-corners`, `optimize_bezier_corners`)
- [x] Centerline tracing mode for line art (`--centerline`, `centerline_tracer`)
- [x] Batch MCP tool for directory conversion (`convert_image_directory`, `batch_convert_enhanced`)
- [x] Despeckle / speckle suppression (`despeckle.rs`, `--min-region-area`, `--no-despeckle`)
- [x] Spline optimization / curve merging pass (`optimize_bezier_curves`, `--optimize-curves`)
- [x] CLI progress bar (`--progress`) with stage-based progress reporting for enhanced pipeline
- [x] `--autotrace` mode that auto-selects tuning parameters based on image characteristics (logo / icon / photo heuristics)
- [x] Additional vector export backends: DXF, GeoJSON, Gimppath (`--format dxf|geo-json|gimppath`)
- [x] Multi-page PDF output for batch conversions (`img2svg -i dir/ -o combined.pdf` / `batch_convert_to_pdf` / MCP `convert_image_directory_to_pdf`)
- [x] Unit tests for multi-page PDF (structure, MediaBoxes, xref offsets, y-flip; 5 tests in `src/eps_generator.rs`)
- [x] Integration tests for single-page PDF export, multi-page PDF from pages, and batch directory → PDF (3 tests)
- [x] MCP `convert_image_directory_to_pdf` tool (schema + 2 tests; success path + missing-params error)
- [x] User-controlled SVG coordinate precision: `--precision` / `EnhancedOptions.coordinate_precision` / MCP `coordinate_precision` (vtracer `path_precision` parity; library API `*_prec` writers; 13 new tests, 468 total executions)
- [x] `--stats` flag: per-file conversion summary (dimensions, paths, colors, output size, elapsed ms)
- [x] `--optimize-tolerance` / `EnhancedOptions.optimize_tolerance` / MCP `optimize_tolerance`: parameterized curve-merge tolerance (Potrace `--opttolerance` parity)
- [x] Native SVG gradient output: `--gradients` / `EnhancedOptions.detect_gradients` / MCP `gradients` wires `gradient_detector` into the enhanced pipeline — detected regions render as `<defs>` `<linearGradient>`/`<radialGradient>` fills (opt-in; detection runs against the original image; 8 new tests, 480 total executions)
- [x] Documentation audit: fixed version drift, MCP install instructions (`--features mcp`), MCP schema enums, test counts, TOC anchors across README/SPEC/ARCHITECTURE
- [x] MEMORY.md created (pipeline, writer, CLI/MCP, and testing patterns harvested)
- [x] `--quality-preset low|medium|high` (CLI) — human-readable precision preset for SVG path coordinate decimals; overrides `--precision` when set
- [x] `--curve-tolerance` (CLI) — Bézier fitting strictness knob (default 2.0); wired through `main.rs::enhanced_options()` → `EnhancedOptions.curve_tolerance`
- [x] Whole-region gradient merging — `collect_gradient_defs` two-tier match: exact `DetectedGradient` equality first, then `gradient_is_similar` (same variant only). Merges adjacent regions with related gradients into a single `<defs>` entry
- [x] `--group-by-color` (CLI) — wrap each fill-color group in `<g data-img2svg-color="#hex">` for Inkscape-style hierarchical layers (default off; streaming-preserving)
- [x] **Generation quality pass (2026-09-09)** — four targeted improvements:
  - K-means empty-cluster reseeding (split largest cluster on empty slots), alpha-aware perceptual distance, 12-iteration refinement in `enhanced_quantizer.rs` (better palette coverage + transparency handling)
  - Interpolated (sub-pixel) marching squares: `marching_squares_contours_interp` + `build_smoothed_field` (Gaussian-blurred, bilinear-sampled) in `vectorizer.rs`; binary-mask topology preserved, edge crossings placed off the 0.5-grid for smoother contours. Wired as `--no-subpixel-contours` (default on) / `EnhancedOptions.subpixel_contours` / MCP `subpixel_contours`
  - Bézier fitter: centripetal parameterization (reduces overshoot at sharp turns) + G1 continuity that respects real sharp corners (smoothes error-split junctions without rounding off corners) in `bezier_fitter.rs`
  - Recolor methods: `RecolorMethod` enum (Mean/Median/Mode) in `enhanced_vectorizer.rs`; `--recolor-mode` CLI / `EnhancedOptions.recolor_method` / MCP `recolor_method` — median/mode avoid muddy averaged colors in flat-color graphics

## Pending 🚧

*(none — all planned items implemented)*

## Maybe / Future Considerations 💭

Seeded by competitive intelligence (Potrace / vtracer / Inkscape, 2026-09-08):

- [x] Whole-region gradient merging: detect one gradient spanning multiple adjacent quantized bands (current detection is per-band; see MEMORY.md gradient notes) — **done 2026-09-15**: two-tier match (exact equality + same-variant similarity) in `collect_gradient_defs`
- [x] Multi-color hierarchical SVG layers (Inkscape-style grouped paths per color) — **done 2026-09-15**: `--group-by-color` flag wraps each fill-color group in `<g data-img2svg-color="#hex">`; streaming-preserving; default off
- [ ] vtracer-style stacked vs cutout layer hierarchy modes
- [ ] Plugin system for custom color quantization algorithms
- [ ] Machine learning-based color palette generation
- [ ] Support for spot color (Pantone) matching
- [ ] Layer-aware vectorization for Photoshop files
- [ ] Vector-to-raster comparison tools
- [ ] Multi-color hierarchical SVG layers (Inkscape-style grouped paths per color)

### Competitive gaps (from Potrace, vtracer, Inkscape trace)
- [x] Corner optimization / curve optimization pass (Potrace-style)
- [x] Centerline tracing mode for line art (vtracer)
- [x] Multi-pass outline + fill separation for logos
- [x] CLI `--preset logo|photo|icon` with tuned defaults
- [x] CLI progress bar (Potrace `--progress`)
- [x] DXF / GeoJSON / Gimppath export backends (Potrace)
- [x] Auto-algorithm selection / `--autotrace` mode (Inkscape Autotrace)
- [x] Multi-page PDF output for batch conversions (Potrace / Inkscape)

### Out of scope (will not implement)
- WebAssembly compilation (not in scope)
- GUI application / live interactive preview server (CLI and library only)
- Optical character recognition (OCR) text preservation (no OCR dependency)
