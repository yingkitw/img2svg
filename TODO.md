# TODO

## Done
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

## Pending
- [ ] Add unit tests for vectorizer (marching_squares_contours, smooth_boundary, rdp_simplify)
- [ ] Add unit tests for svg_generator (create_subpath_string, create_multi_path_string)
- [ ] Handle transparency/alpha channel in SVG output
- [ ] Support JPEG, WebP, BMP input formats (already supported via `image` crate, needs testing)
- [ ] Optimize memory usage for large images (>4K)
- [ ] Add progress logging for large image conversions
- [ ] Consider SVG gradient detection for smoother gradient rendering
