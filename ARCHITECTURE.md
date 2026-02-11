# Architecture

## Module Structure

```
src/
├── main.rs                 # CLI entry point, orchestrates pipeline
├── cli.rs                  # Clap-based argument parsing
├── lib.rs                  # Library API with public exports
├── image_processor.rs      # Image loading, median-cut color quantization (original)
├── vectorizer.rs           # Marching-squares contour tracing, smoothing, RDP (original)
├── svg_generator.rs        # SVG file generation with merged subpaths per color (original)
├── preprocessor.rs         # Image preprocessing (bilateral filter, color reduction)
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

examples/
├── input/                  # Test input images (PNG)
│   ├── simple.png
│   ├── gradient.png
│   ├── medium.png
│   ├── complex.png
│   └── very_complex.png
├── output/                 # Generated SVG files
│   ├── simple.svg
│   ├── gradient.svg
│   ├── medium.svg
│   ├── complex.svg
│   └── very_complex.svg
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
         smooth_boundary (Gaussian averaging) → Vec<Point>
         ↓
         rdp_simplify (epsilon=2.0) → Vec<Point>
         ↓
         snap_to_edges (boundary detection) → Vec<Point>
         ↓
         merge subpaths into Curve { subpaths, color }
         ↓
         generate_svg → background rect + one <path> per color
```

### Enhanced Pipeline (--enhanced flag)
```
PNG/JPEG → load_image → ImageData
         ↓
         [Auto: detect unique colors, adaptive color count (256/128/64)]
         ↓
         [Optional: bilateral_filter (edge-preserving smoothing)]
         ↓
         detect_edges_sobel → EdgeMap (Sobel gradient magnitudes)
         ↓
         quantize_edge_aware (k-means++ init → k-means refine → majority-vote smoothing)
         ↓
         group pixels by quantized color → per-color binary mask
         ↓
         marching_squares_contours (shared with original pipeline, proven robust)
         ↓
         parallel per contour (rayon):
           smooth_with_corners (Gaussian, corner-preserving)
           → visvalingam_whyatt (area-based simplification, corner-preserving)
           → snap_to_edges (boundary snapping)
           → BezierFitter::fit_path (corner-aware splitting → cubic Bézier + Newton-Raphson)
         ↓
         sort by area (back-to-front), detect_background_color (border pixels)
         ↓
         generate_enhanced_svg → L for lines, C for curves, collinear merge, gap-filling strokes
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
- **Marching squares**: Traces sub-pixel contours on binary mask per color
- **Gaussian smoothing**: Neighbor averaging that preserves point count
- **RDP simplification**: Ramer-Douglas-Peucker with epsilon=2.0
- **Background detection**: Largest-area color becomes SVG rect (not traced)
- **Subpath merging**: All contours of same color → single `<path>` with `M...Z` subpaths

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

### Preprocessing Algorithms
- **Bilateral filter**: Edge-preserving smoothing using spatial and color weights
  - Spatial kernel: Gaussian based on pixel distance
  - Color kernel: Gaussian based on color similarity
  - Preserves edges while smoothing flat areas
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
use img2svg::{convert, ConversionOptions};

let options = ConversionOptions {
    num_colors: 16,
    smooth_level: 5,
    threshold: 0.1,
    ..Default::default()
};
convert(Path::new("input.png"), Path::new("output.svg"), &options)?;
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

## Dependencies

- `image` (0.24) — raster image loading (PNG, JPEG, BMP, TIFF, WebP, etc.)
- `clap` (4.0) — CLI argument parsing
- `serde` / `serde_json` — Serialization for MCP protocol
- `rgb` (0.8) — RGBA8 pixel type
- `anyhow` — Error handling
- `thiserror` — Error derive macros
- `rayon` (1.10) — Parallel path processing (enhanced pipeline)
- `rand` (0.8) — K-means++ random centroid selection (enhanced pipeline)

### Test Dependencies
- (dev-dependencies only)
- Test images in `examples/input/`

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
- Tested up to 4K resolution (4096x4096)
- Memory efficient: streams well, suitable for large images
- Typical 1000x1000 image converts in <1 second
