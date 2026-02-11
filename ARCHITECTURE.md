# Architecture

## Module Structure

```
src/
├── main.rs                 # CLI entry point, orchestrates pipeline
├── cli.rs                  # Clap-based argument parsing
├── lib.rs                  # Library API with public exports
├── image_processor.rs      # Image loading, median-cut color quantization
├── vectorizer.rs           # Marching-squares contour tracing, smoothing, simplification
├── svg_generator.rs        # SVG file generation with merged subpaths per color
├── preprocessor.rs         # Image preprocessing (bilateral filter, color reduction)
├── mcp.rs                  # MCP (Model Context Protocol) server
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

### Preprocessing Pipeline (for photographs)
```
ImageData → bilateral_filter (edge-preserving smoothing)
         ↓
         reduce_colors (posterization)
         ↓
         ImageData (cleaner regions, less noise)
         ↓
         [continues to standard pipeline]
```

## Key Algorithms

### Core Algorithms
- **Median-cut quantization**: Recursively splits color space along widest channel
- **Marching squares**: Traces sub-pixel contours on binary mask per color
- **Gaussian smoothing**: Neighbor averaging that preserves point count
- **RDP simplification**: Ramer-Douglas-Peucker with epsilon=2.0
- **Background detection**: Largest-area color becomes SVG rect (not traced)
- **Subpath merging**: All contours of same color → single `<path>` with `M...Z` subpaths

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
