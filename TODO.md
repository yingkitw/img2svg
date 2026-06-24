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
- [x] Unit tests for image_processor (13 tests)
- [x] Unit tests for vectorizer (35 tests)
- [x] Unit tests for svg_generator (20 tests)
- [x] Integration tests (25 tests): legacy pipeline, **enhanced pipeline**, **`convert` / `convert_to_svg_string`**, auto `num_colors`, **original vs enhanced parity**, **use-case examples** (logo, transparency), **superpixel/centerline/logo/EPS**
- [x] CLI unit tests: `is_supported_image`, **`--preset`** parsing and overrides (8 tests, `src/cli.rs`)
- [x] MCP server unit tests: initialize, tools/list schema, tools/call validation and happy path (6 tests, `src/mcp.rs`)
- [x] Library API tests (1 test)
- [x] Run `cargo test` for full counts (lib + binary targets + `img2svg-mcp` + integration + doctests)
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
- [x] 50+ new unit tests for enhanced modules
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
- [x] Superpixel segmentation as alternative to color quantization (`--superpixel`, SLIC + `quantize_superpixel`)
- [x] Interactive parameter tuning with HTML preview grid (`--tune`, `tune::generate_tune_html`)
- [x] Export to other vector formats (`--format eps|pdf|ai`, `eps_generator` module)
- [x] Non-local means (NLM) denoising (`NonLocalMeans` in `image_filters/denoising.rs`)
- [x] Anisotropic diffusion (Perona-Malik) (`PeronaMalik` in `image_filters/denoising.rs`)
- [x] Watershed segmentation (`watershed_segment` in `image_filters/segmentation.rs`)
- [x] Mean shift segmentation (`mean_shift_filter`, SLIC superpixels in `image_filters/segmentation.rs`)
- [x] Corner optimization / curve optimization pass (`--optimize-corners`, `optimize_bezier_corners`)
- [x] Centerline tracing mode for line art (`--centerline`, `centerline_tracer`)
- [x] Multi-pass outline + fill separation for logos (`--logo-mode`, `logo_pipeline`)

## Pending 🚧

*(none — all planned items implemented)*

## Maybe / Future Considerations 💭

- [ ] WebAssembly compilation for browser-based conversion
- [ ] GUI application for interactive parameter tuning
- [ ] Plugin system for custom color quantization algorithms
- [ ] Machine learning-based color palette generation
- [ ] Support for spot color (Pantone) matching
- [ ] Layer-aware vectorization for Photoshop files
- [ ] Vector-to-raster comparison tools

### Competitive gaps (from Potrace, vtracer, Inkscape trace)
- [x] Corner optimization / curve optimization pass (Potrace-style)
- [x] Centerline tracing mode for line art (vtracer)
- [x] Multi-pass outline + fill separation for logos
- [x] CLI `--preset logo|photo|icon` with tuned defaults
