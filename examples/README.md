# Test Images for img2svg Conversion

This directory contains test images of varying complexity to validate the img2svg conversion capability.

## Test Images

### 1. [simple.png](input/simple.png) - Basic Complexity
**Purpose**: Test basic shape recognition and color conversion

**Contents**:
- Red circle
- Blue rectangle
- Green triangle
- Yellow rectangle

**What it validates**:
- Basic geometric shape detection
- Solid color accuracy
- Simple edge detection
- Basic SVG path generation

**Expected conversion quality**: Should achieve near-perfect conversion with minimal file size overhead.


### 2. [medium.png](input/medium.png) - Medium Complexity
**Purpose**: Test overlapping shapes and color blending

**Contents**:
- Four-quadrant background with different pastel colors
- Three overlapping circles with primary colors
- Additional non-overlapping shapes (rectangle and triangle)

**What it validates**:
- Overlapping shape handling
- Edge detection at shape boundaries
- Color priority/layering in overlapping regions
- Mixed shape types in single image

**Expected conversion quality**: Should handle overlaps correctly, though some color blending at edges may vary.


### 3. [complex.png](input/complex.png) - High Complexity
**Purpose**: Test performance and accuracy with many elements

**Contents**:
- Grid pattern with color variations
- 10 overlapping circles in various positions
- 8 rectangles with different colors
- 6 triangles in a row
- 20 small detail circles

**What it validates**:
- Performance with many objects (40+ shapes)
- Color variation handling
- Pattern recognition
- Small detail preservation
- File size optimization

**Expected conversion quality**: Should maintain reasonable performance while preserving most visual details. Some minor detail loss acceptable.


### 4. [very_complex.png](input/very_complex.png) - Very High Complexity (Stress Test)
**Purpose**: Stress test the converter with maximum complexity

**Contents**:
- 20 concentric circles (rainbow gradient pattern)
- 4 corner decorations with 8x8 pixel grids each (256 small squares total)
- Stripe overlay pattern
- 100 randomly scattered small shapes (circles and rectangles)

**What it validates**:
- Performance under heavy load (400+ elements)
- Radial pattern handling
- High-frequency detail preservation
- Random shape distribution
- Color gradient simulation
- File size scalability

**Expected conversion quality**: This is a stress test. Significant detail loss and/or large file sizes are acceptable. Focus on whether the converter completes without errors.


### 5. [gradient.png](input/gradient.png) - Gradient Test
**Purpose**: Test color gradient handling

**Contents**:
- Horizontal gradient (red to green)
- Vertical gradient (blue to cyan)
- Simulated using pixel-by-pixel color variations

**What it validates**:
- Gradient recognition and approximation
- Color transition smoothness
- SVG gradient generation vs. discrete shapes
- Color accuracy across spectrum

**Expected conversion quality**: Since SVG supports gradients, the converter should ideally detect and generate SVG gradients rather than hundreds of discrete shapes.


### 6. [logo.png](input/logo.png) - Brand Logo
**Purpose**: Flat logo with few solid colors and sharp edges

**Contents**:
- Dark background with teal chevron mark
- Red accent circle and yellow bar

**What it validates**:
- Logo/icon conversion with low color count
- Sharp corner preservation (Bézier + corner injection)
- Small file size with `-c 8`

**Recommended command**:
```bash
img2svg -i examples/input/logo.png -o examples/output/logo.svg --preset logo
```


### 7. [line_art.png](input/line_art.png) - Line Art
**Purpose**: Thin 1–2px strokes on white background

**Contents**:
- Grid of single-pixel lines
- Arc and diagonal strokes

**What it validates**:
- Thin stripe fast path (contours < 2px → SVG rects)
- High path count handling
- Line art / technical drawing use case

**Recommended command**:
```bash
img2svg -i examples/input/line_art.png -o examples/output/line_art.svg -c 4
```


### 8. [transparency.png](input/transparency.png) - Alpha Channel
**Purpose**: Overlapping semi-transparent shapes (RGBA)

**Contents**:
- Red, blue, and green shapes with varying alpha
- Transparent background regions

**What it validates**:
- Alpha channel preserved as `fill-opacity` in SVG
- Overlapping color region extraction

**Recommended command**:
```bash
img2svg -i examples/input/transparency.png -o examples/output/transparency.svg
```


### 9. [icon.png](input/icon.png) - App Icon (128×128)
**Purpose**: Small pixel-style icon for UI/app use

**Contents**:
- 128×128 camera-style icon on dark background
- Few distinct flat colors

**What it validates**:
- Small image conversion
- Icon-scale detail at low dimensions

**Recommended command**:
```bash
img2svg -i examples/input/icon.png -o examples/output/icon.svg -c 12 -s 2
```


### 10. [lenna.png](input/lenna.png) - Photograph
**Purpose**: Real-world photo with `--preprocess`

See main [README](../README.md) for lenna comparison (with/without preprocessing).


## Usage

### Running Tests

To test your img2svg converter:

```bash
# Test simple conversion
cargo run -- -i examples/input/simple.png -o examples/output/simple.svg

# Test medium complexity
cargo run -- -i examples/input/medium.png -o examples/output/medium.svg

# Test complex conversion
cargo run -- -i examples/input/complex.png -o examples/output/complex.svg

# Stress test
cargo run -- -i examples/input/very_complex.png -o examples/output/very_complex.svg

# Test gradient handling
cargo run -- -i examples/input/gradient.png -o examples/output/gradient.svg

# Logo / icon use cases
cargo run -- -i examples/input/logo.png -o examples/output/logo.svg -c 8 -s 2
cargo run -- -i examples/input/icon.png -o examples/output/icon.svg -c 12 -s 2

# Line art and transparency
cargo run -- -i examples/input/line_art.png -o examples/output/line_art.svg -c 4
cargo run -- -i examples/input/transparency.png -o examples/output/transparency.svg

# Photo with preprocessing
cargo run -- -i examples/input/lenna.png -o examples/output/lenna.svg --preprocess -c 12
```

### Validation Checklist

For each converted SVG, check:

- [ ] **Visual accuracy**: Does it look like the original when rendered?
- [ ] **File size**: Is the SVG size reasonable compared to the PNG?
- [ ] **Conversion time**: Was the conversion performed in acceptable time?
- [ ] **Shape count**: Does the SVG have a reasonable number of paths/elements?
- [ ] **Edge quality**: Are shape boundaries smooth and accurate?
- [ ] **Color accuracy**: Do colors match the original?
- [ ] **Browser rendering**: Does it render correctly in web browsers?

### Expected Results by Complexity

| Image | Expected Time | Expected SVG Size | Quality Expectation |
|-------|--------------|-------------------|---------------------|
| simple.png | < 0.1s | < 5 KB | Near perfect |
| medium.png | < 0.5s | < 20 KB | Good with minor overlap artifacts |
| complex.png | < 2s | < 100 KB | Good overall, minor detail loss acceptable |
| very_complex.png | < 10s | < 500 KB | Acceptable quality, focus on completion |
| gradient.png | < 1s | < 50 KB (with gradient detection) | Smooth gradients preferred |
| logo.png | < 0.2s | < 2 KB | Near perfect at `-c 8` |
| line_art.png | < 1s | varies (many paths) | Thin lines preserved |
| transparency.png | < 0.3s | < 2 KB | `fill-opacity` in SVG |
| icon.png | < 0.1s | < 2 KB | Clean small icon |
| lenna.png | < 2s | < 50 KB with `--preprocess` | Photo; use preprocessing |


## Regenerating Test Images

To regenerate the test images:

```bash
python3 generate_test_images.py
```

This will create fresh copies of all test images in the `input/` directory.

## Adding Custom Test Cases

To add your own test images:

1. Create a new PNG image in `input/` directory
2. Name it descriptively (e.g., `my_test_case.png`)
3. Document what it tests in this README
4. Run conversion and verify results

## Success Criteria

Your img2svg converter should:

1. Successfully convert all test images without crashing
2. Produce visually similar SVG output
3. Generate valid SVG files that render in browsers
4. Complete conversions in reasonable time
5. Optimize file size appropriately for complexity level
