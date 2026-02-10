# img2svg Specification

## Purpose
Convert raster images (PNG, JPEG, etc.) to scalable vector graphics (SVG) with filled color regions.

## Input
- Raster image file (PNG, JPEG, BMP, WebP via `image` crate)
- Parameters: color count, smoothing level, hierarchical mode, advanced SVG mode

## Output
- SVG file with filled `<path>` elements representing color regions
- Background color auto-detected and rendered as `<rect>`
- One `<path>` per non-background color, using merged subpaths

## Pipeline

1. **Load** → `image_processor::load_image` → `ImageData { width, height, pixels: Vec<RGBA8> }`
2. **Quantize** → `image_processor::quantize_colors` → Median-cut algorithm, map each pixel to nearest palette color
3. **Group** → Group pixels by quantized color, sort by area (largest = background)
4. **Contour** → `vectorizer::marching_squares_contours` → Per-color binary mask → sub-pixel contours
5. **Smooth** → `vectorizer::smooth_boundary` → Gaussian neighbor averaging (point-count preserving)
6. **Simplify** → `vectorizer::rdp_simplify` → Ramer-Douglas-Peucker with epsilon=2.0
7. **SVG emit** → `svg_generator::generate_svg` → Background rect + merged line-segment paths per color

## Quality Criteria
- All test images in `examples/input/` must convert without panics
- SVG output must contain filled regions (not just strokes)
- Compact file size (simple.svg < 2KB from 2KB PNG)
- Proper z-ordering: background rect, then colors by decreasing area
