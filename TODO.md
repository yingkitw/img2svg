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
- [x] Integration tests (9 tests)
- [x] Library API tests (1 test)
- [x] Total: 160 tests passing

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
- [x] --enhanced CLI flag to select new pipeline (original preserved for comparison)
- [x] 50+ new unit tests for enhanced modules
- [x] SVG size optimization: L for linear curves, C for true Bézier, collinear L merge
- [x] Distance-based collinear merge catches diagonal staircases from marching squares
- [x] Adaptive simplification tolerance (2x for photos, 1.5 for graphics)
- [x] Visual comparison page with lenna.png, 6 test images total
- [x] Control point clamping to image bounds (fixes corner bulging in medium.png)
- [x] Degenerate thin-line path filtering (fixes very_complex.png bloat)
- [x] Auto-preprocessing for photos in enhanced pipeline (bilateral filter + color reduction)

## Pending 🚧

### Core Features
- [ ] Handle transparency/alpha channel in SVG output (preserve as opacity)
- [ ] Add hierarchical decomposition mode (flag exists, implementation incomplete)
- [ ] SVG gradient detection for smoother gradient rendering
- [ ] Support for CMYK color space input

### Performance
- [ ] Optimize memory usage for large images (>4K)
- [ ] Progress logging for large image conversions
- [ ] Incremental SVG generation for memory-constrained environments

### Input/Output
- [ ] Test JPEG, WebP, BMP input formats thoroughly
- [ ] SVG optimization (remove redundant points, merge adjacent paths)
- [ ] Support for animated GIF input
- [ ] Batch conversion mode from CLI

### MCP Server
- [ ] Add streaming file reading for large images
- [ ] Add progress callbacks to MCP tool responses
- [ ] Support for custom preprocessing parameters in MCP

### Documentation
- [ ] API reference documentation (rustdoc for all public functions)
- [ ] More example images demonstrating different use cases
- [ ] Performance benchmarks comparing original vs enhanced pipeline

### Advanced Features
- [ ] Superpixel segmentation as alternative to color quantization
- [ ] Adaptive threshold based on image content
- [ ] Interactive mode for parameter tuning with live preview
- [ ] Export to other vector formats (PDF, EPS, AI)

## Maybe / Future Considerations 💭

- [ ] WebAssembly compilation for browser-based conversion
- [ ] GUI application for interactive parameter tuning
- [ ] Plugin system for custom color quantization algorithms
- [ ] Machine learning-based color palette generation
- [ ] Support for spot color (Pantone) matching
- [ ] Layer-aware vectorization for Photoshop files
- [ ] Vector-to-raster comparison tools
