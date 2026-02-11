# img2svg Specification

## Purpose
Convert raster images (PNG, JPEG, etc.) to scalable vector graphics (SVG) with filled color regions.

## Input
- Raster image file (PNG, JPEG, BMP, WebP, TIFF via `image` crate)
- CLI Parameters:
  - `--colors` / `-c`: Number of colors for quantization (1-64, default: 16)
  - `--threshold` / `-t`: Edge detection threshold (0.0-1.0, default: 0.1)
  - `--smooth` / `-s`: Path smoothing level (0-10, default: 5)
  - `--preprocess` / `-p`: Apply edge-preserving smoothing and color reduction
  - `--hierarchical`: Enable hierarchical decomposition
  - `--advanced` / `-a`: Use advanced SVG generation

## Output
- SVG file with filled `<path>` elements representing color regions
- Background color auto-detected and rendered as `<rect>`
- One `<path>` per non-background color, using merged subpaths
- ViewBox and dimensions match input image

## Pipeline

### Standard Pipeline
1. **Load** → `image_processor::load_image` → `ImageData { width, height, pixels: Vec<RGBA8> }`
2. **[Optional] Preprocess** → `preprocessor::preprocess` → Bilateral filter + color reduction
3. **Quantize** → `image_processor::quantize_colors` → Median-cut algorithm
4. **Group** → Group pixels by quantized color, sort by area (largest = background)
5. **Contour** → `vectorizer::marching_squares_contours` → Per-color binary mask → sub-pixel contours
6. **Smooth** → `vectorizer::smooth_boundary` → Gaussian neighbor averaging
7. **Simplify** → `vectorizer::rdp_simplify` → Ramer-Douglas-Peucker with epsilon=2.0
8. **SVG emit** → `svg_generator::generate_svg` → Background rect + merged line-segment paths

### Preprocessing Mode (--preprocess)
Applied before quantization for photographs:
1. **Bilateral filter**: Edge-preserving smoothing (σ_spatial=5.0, σ_color=40.0, 2 iterations)
2. **Color reduction**: Posterization to ~128 color levels (50% reduction)

## Quality Criteria

### Functional Requirements
- ✅ All test images in `examples/input/` convert without panics
- ✅ SVG output contains filled regions (not just strokes)
- ✅ Proper z-ordering: background rect, then colors by decreasing area
- ✅ Edge snapping: boundary points snap to image edges
- ✅ Path simplification removes redundant points while preserving shape
- ✅ 160 unit tests and 9 integration tests passing

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
| Transparency/Alpha | ⚠️ | Preserved in pixel data, rendered as opaque in SVG |
| CMYK colorspace | ❌ | Not supported (converts to RGB) |
| Grayscale images | ✅ | Converted to RGB (R=G=B) |
| Animated GIF | ❌ | First frame only |

## Test Suite

### Unit Tests (160 total)
- `image_processor_tests.rs`: 13 tests
  - Image data creation and validation
  - Color quantization with various parameters
  - Edge cases (empty, single color, zero colors)
  - Median-cut algorithm tests
- `vectorizer_tests.rs`: 35 tests
  - Marching squares contour tracing
  - Polygon area calculation
  - Point-to-line distance
  - RDP simplification
  - Boundary smoothing
  - Full vectorization pipeline
- `svg_generator_tests.rs`: 20 tests
  - SVG generation with various inputs
  - Coordinate formatting
  - Subpath string creation
  - Multi-path handling
- `lib_tests.rs`: 1 test
  - Conversion options defaults

### Integration Tests (9 total)
- Full pipeline conversion tests
- Test image creation (gradient, checkerboard, circle, solid)
- SVG output validation
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
- `vectorize`
- `generate_svg`
- `generate_svg_advanced`
- `convert`
- `convert_to_svg_string`

### Experimental
- `preprocess` API (may change parameters)
- Hierarchical decomposition mode
- Advanced SVG generation with layers
