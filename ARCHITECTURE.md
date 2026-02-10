# Architecture

## Module Structure

```
src/
├── main.rs            # CLI entry point, orchestrates pipeline
├── cli.rs             # Clap-based argument parsing
├── image_processor.rs # Image loading, median-cut color quantization
├── vectorizer.rs      # Marching-squares contour tracing, smoothing, simplification
└── svg_generator.rs   # SVG file generation with merged subpaths per color
```

## Data Flow

```
PNG/JPEG → load_image → ImageData
         → quantize_colors (median-cut) → ImageData (reduced palette)
         → group pixels by color, sort by area
         → per color: binary mask → marching_squares_contours → Vec<Vec<Point>>
         → smooth_boundary (Gaussian averaging) → Vec<Point>
         → rdp_simplify (epsilon=2.0) → Vec<Point>
         → merge subpaths into Curve { subpaths, color }
         → generate_svg → background rect + one <path> per color
```

## Key Algorithms

- **Median-cut quantization**: Recursively splits color space along widest channel
- **Marching squares**: Traces sub-pixel contours on binary mask per color
- **Gaussian smoothing**: Neighbor averaging that preserves point count
- **RDP simplification**: Ramer-Douglas-Peucker with epsilon=2.0
- **Background detection**: Largest-area color becomes SVG rect (not traced)
- **Subpath merging**: All contours of same color → single `<path>` with `M...Z` subpaths

## Dependencies

- `image` — raster image loading
- `clap` — CLI argument parsing
- `rgb` — RGBA8 pixel type
- `anyhow` / `thiserror` — error handling
- `svg`, `petgraph`, `serde` — currently unused, candidates for removal
