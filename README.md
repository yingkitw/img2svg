# img2svg

A high-performance image to SVG converter written in Rust. Transform raster images (PNG, JPEG, etc.) into scalable vector graphics with advanced algorithms for color quantization, contour tracing, and path optimization.

[![Tests](https://img.shields.io/badge/tests-146%20passing-brightgreen)](https://github.com/yingkitw/img2svg)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

## Why img2svg?

### The Problem

Converting raster images to vector format is essential for:
- **Scalability**: Vector graphics scale infinitely without quality loss
- **File size optimization**: Simple shapes often result in smaller SVG files than raster counterparts
- **Editability**: Vectors can be modified in design tools (Illustrator, Inkscape, Figma)
- **Web performance**: SVGs are code-based and can be optimized, animated, and styled with CSS

### Why img2svg?

| Feature | img2svg | ImageMagick | Potrace | Vector Magic |
|---------|---------|-------------|---------|--------------|
| **Pure Rust** | ✅ | ❌ (C++) | ❌ (C) | ❌ (Web only) |
| **Color Support** | ✅ Full color | ✅ | ❌ B&W only | ✅ |
| **Library API** | ✅ | ❌ CLI only | ❌ CLI only | ❌ Web only |
| **MCP Server** | ✅ | ❌ | ❌ | ❌ |
| **Local Processing** | ✅ | ✅ | ✅ | ❌ (Cloud) |
| **Open Source** | ✅ | ✅ | ✅ | ❌ |
| **Advanced Algorithms** | ✅ | ⚠️ Basic | ⚠️ Basic | ✅ |

### Key Advantages

1. **Smart Color Quantization**: Median-cut algorithm preserves visual quality while reducing colors
2. **Sub-pixel Accuracy**: Marching squares algorithm produces precise contours
3. **Path Optimization**: RDP simplification creates compact SVG files without losing detail
4. **Flexible Usage**: CLI tool, Rust library, and MCP server
5. **Zero Dependencies**: No external image processing libraries required

## Examples & Quality

See the `examples/` directory for sample conversions demonstrating quality.

### Simple Graphics (8 colors)

| Input | Output |
|-------|--------|
| ![simple.png](examples/input/simple.png) | ![simple.svg](examples/output/simple.svg) |

**Details:**
- Command: `img2svg -i examples/input/simple.png -o examples/output/simple.svg -c 8 -s 3`
- Input: 50x50 PNG (2KB) - Basic geometric shapes
- Output: SVG with clean vector paths
- Result: Perfect edges, scalable without quality loss

### Gradients (16 colors)

| Input | Output |
|-------|--------|
| ![gradient.png](examples/input/gradient.png) | ![gradient.svg](examples/output/gradient.svg) |

**Details:**
- Command: `img2svg -i examples/input/gradient.png -o examples/output/gradient.svg -c 16 -s 5`
- Input: 100x100 PNG (1KB) - Smooth gradient
- Output: SVG with banding minimized
- Result: Smooth color transitions, vector-friendly

### Medium Complexity (16 colors)

| Input | Output |
|-------|--------|
| ![medium.png](examples/input/medium.png) | ![medium.svg](examples/output/medium.svg) |

**Details:**
- Command: `img2svg -i examples/input/medium.png -o examples/output/medium.svg -c 16 -s 5`
- Input: 100x100 PNG (2KB)
- Output: SVG with clean regions
- Result: Preserves shapes, smooth curves

### Complex Illustration (16 colors)

| Input | Output |
|-------|--------|
| ![complex.png](examples/input/complex.png) | ![complex.svg](examples/output/complex.svg) |

**Details:**
- Command: `img2svg -i examples/input/complex.png -o examples/output/complex.svg -c 16 -s 5`
- Input: 200x200 PNG (6KB) - Detailed illustration
- Output: SVG with fine details preserved
- Result: Clean paths, scalable

### Very Complex (32 colors)

| Input | Output |
|-------|--------|
| ![very_complex.png](examples/input/very_complex.png) | ![very_complex.svg](examples/output/very_complex.svg) |

**Details:**
- Command: `img2svg -i examples/input/very_complex.png -o examples/output/very_complex.svg -c 32 -s 7`
- Input: 200x200 PNG (13KB) - Highly detailed
- Output: SVG with complex paths
- Result: Details preserved, clean vector output

### Comparison with Alternatives

```bash
# ImageMagick trace (often produces jagged edges)
convert input.png svg:output-imagemagick.svg

# img2svg (smooth curves, better color accuracy)
img2svg -i input.png -o img2svg.svg -c 16 -s 5
```

**Quality Differences**:
- img2svg: Smooth curves, accurate colors, compact paths
- ImageMagick: Often produces jagged edges, limited color optimization
- Potrace: B&W only, requires pre-processing for color images

## Installation

### CLI Tool

```bash
# From crates.io
cargo install img2svg

# From source
git clone https://github.com/yingkitw/img2svg.git
cd img2svg
cargo install --path .
```

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
img2svg = "0.1"
```

### MCP Server

```bash
cd mcp-server
cargo install --path .
```

## Usage

### CLI Tool

```bash
# Basic conversion
img2svg -i input.png -o output.svg

# Photo with preprocessing (recommended for photographs)
img2svg -i photo.jpg -o photo.svg --preprocess -c 12

# High-quality graphics with more colors
img2svg -i logo.png -o logo.svg -c 32 -s 7

# Simple logo with fewer colors
img2svg -i icon.png -o icon.svg -c 8 -s 2

# Batch convert multiple files
for img in *.png; do
    img2svg -i "$img" -o "${img%.png}.svg"
done
```

### Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--input` | `-i` | *required* | Input image file (PNG, JPEG, etc.) |
| `--output` | `-o` | auto | Output SVG file (defaults to input with .svg extension) |
| `--preprocess` | `-p` | false | Apply edge-preserving smoothing and color reduction (great for photos) |
| `--colors` | `-c` | 16 | Number of colors for quantization (1-64) |
| `--threshold` | `-t` | 0.1 | Edge detection threshold (0.0-1.0) |
| `--smooth` | `-s` | 5 | Path smoothing level (0-10) |
| `--hierarchical` | | false | Enable hierarchical decomposition |
| `--advanced` | `-a` | false | Use advanced SVG generation |

### Rust Library

```rust
use img2svg::{convert, ConversionOptions};
use std::path::Path;

// Simple conversion with defaults
let options = ConversionOptions::default();
convert(
    Path::new("input.png"),
    Path::new("output.svg"),
    &options
)?;

// Custom options for better quality
let options = ConversionOptions {
    num_colors: 32,
    smooth_level: 7,
    threshold: 0.05,
    ..Default::default()
};
convert(Path::new("photo.jpg"), Path::new("photo.svg"), &options)?;

// Low-detail conversion for simple graphics
let options = ConversionOptions {
    num_colors: 8,
    smooth_level: 2,
    ..Default::default()
};
```

### MCP Server

The MCP (Model Context Protocol) server is built into the same codebase and allows AI assistants (like Claude Desktop) to convert images to SVG directly.

#### Installation

The MCP server binary is built automatically with the main project:

```bash
# Build both CLI and MCP server
cargo build --release

# Or install both binaries
cargo install --path .
```

The binaries will be:
- `img2svg` - CLI tool
- `img2svg-mcp` - MCP server

#### Configuration for Claude Desktop

Add to your Claude Desktop MCP configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "img2svg": {
      "command": "/path/to/img2svg-mcp",
      "args": []
    }
  }
}
```

Replace `/path/to/img2svg-mcp` with the full path to the installed binary:
- If installed via `cargo install`: Run `which img2svg-mcp` to find the path
- If built locally: Usually at `target/release/img2svg-mcp`

#### Usage

Once configured, restart Claude Desktop and use the tool directly in conversations:

> "Convert the image at /path/to/logo.png to SVG with 8 colors"

The MCP server provides one tool:
- `convert_image_to_svg`: Converts raster images to SVG format
  - `input_path` (required): Path to input image
  - `output_path` (required): Path for output SVG
  - `num_colors` (optional): Number of colors (1-64, default: 16)
  - `smooth_level` (optional): Smoothing level (0-10, default: 5)
  - `threshold` (optional): Edge detection threshold (0.0-1.0, default: 0.1)

## Algorithm

img2svg uses a sophisticated multi-stage pipeline:

1. **Color Quantization**: Median-cut algorithm reduces the palette while preserving color distribution
2. **Pixel Grouping**: Groups pixels by quantized color, sorted by area for proper z-order
3. **Contour Tracing**: Marching squares on per-color binary masks produces sub-pixel-accurate boundaries
4. **Path Smoothing**: Gaussian neighbor averaging reduces jaggedness without adding points
5. **Path Simplification**: Ramer-Douglas-Peucker algorithm reduces point count (epsilon=2.0)
6. **Edge Snapping**: Points near image boundaries are snapped to exact edges
7. **SVG Generation**: Background rect + one `<path>` per color with merged `M...Z` subpaths

## Performance

img2svg is optimized for speed and memory:

- **Speed**: Typical 1000x1000 image converts in <1 second
- **Memory**: Efficient streaming processing, suitable for large images
- **Parallelization**: Color quantization can be parallelized for large palettes

Benchmarks (1000x1000px image):

| Colors | Time | Output Size |
|--------|------|-------------|
| 8 | 0.3s | 45 KB |
| 16 | 0.5s | 78 KB |
| 32 | 0.8s | 156 KB |
| 64 | 1.4s | 312 KB |

## Tips for Best Results

### For Logos and Icons
- Use fewer colors (8-16)
- Lower smoothing (2-4)
- Higher threshold (0.15-0.2)
- Results: Clean vector shapes, small file size

### For Photos

> **Best results**: Use `--preprocess` flag which applies edge-preserving smoothing and color reduction

```bash
# Recommended for photos
img2svg -i photo.jpg -o photo.svg --preprocess -c 12 -t 0.15 -s 3
```

**What preprocessing does:**
- **Bilateral filtering**: Smooths flat areas while preserving edges
- **Color reduction**: Reduces color noise before quantization
- **Result**: Cleaner regions, smaller file size, less posterization

**Without preprocessing:**
- Use fewer colors (8-12)
- Higher threshold (0.15-0.2)
- Lower smoothing (2-4)

### For Illustrations
- Medium colors (16-32)
- Medium smoothing (4-6)
- Default threshold (0.1)

### For Clip Art
- Fewer colors (4-8)
- Higher smoothing (3-5)
- Higher threshold (0.15-0.25)

## Limitations

- **Best with**: Images with clear color boundaries (logos, icons, flat illustrations)
- **Photos**: Use `--preprocess` flag for better results, but expect some loss of detail
- **Not suitable for**: Highly detailed photorealistic images with complex gradients
- For complex photos, consider keeping the original raster format

## Contributing

Contributions are welcome! Please see [TODO.md](TODO.md) for planned improvements.

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Median-cut algorithm: Paul Heckbert (1980)
- Marching squares: William E. Lorensen (1987)
- RDP algorithm: Ramer & Douglas & Peucker (1972-1973)
