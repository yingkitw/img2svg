#!/bin/bash
# Convert all test images to SVG for validation

# Don't use 'set -e' because we want to continue even if individual conversions fail

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Change to project root to ensure paths work correctly
cd "$PROJECT_ROOT"

INPUT_DIR="examples/input"
OUTPUT_DIR="examples/output"

echo "=== img2svg Test Suite ==="
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust first."
    exit 1
fi

# Check if project is built
echo "Building project..."
cargo build --release
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Track statistics
total=0
success=0
failed=0

# Check if there are any test images
if ! ls "$INPUT_DIR"/*.png 1> /dev/null 2>&1; then
    echo "Error: No PNG files found in $INPUT_DIR"
    echo "Run 'python3 generate_test_images.py' to generate test images first."
    exit 1
fi

# Convert each test image
for img in "$INPUT_DIR"/*.png; do
    if [ -f "$img" ]; then
        filename=$(basename "$img" .png)
        output="$OUTPUT_DIR/${filename}.svg"

        echo "Converting $filename.png..."

        # Time the conversion (use python for ms precision on macOS)
        start_time=$(python3 -c 'import time; print(int(time.time()*1000))')
        if cargo run --release --bin img2svg -- --input "$img" --output "$output" 2>&1; then
            end_time=$(python3 -c 'import time; print(int(time.time()*1000))')
            duration=$(( end_time - start_time ))

            # Get file sizes
            input_size=$(stat -f%z "$img" 2>/dev/null || stat -c%s "$img" 2>/dev/null)
            input_kb=$((input_size / 1024))

            # Check if output file was created
            if [ -f "$output" ]; then
                output_size=$(stat -f%z "$output" 2>/dev/null || stat -c%s "$output" 2>/dev/null)
                output_kb=$((output_size / 1024))

                echo "  ✓ Success in ${duration}ms"
                echo "    Input:  ${input_kb}KB"
                echo "    Output: ${output_kb}KB"
            else
                echo "  ✗ Conversion command succeeded but output file not created"
                ((failed++)) || true
                ((total++)) || true
                continue
            fi
            echo ""

            ((success++)) || true
        else
            echo "  ✗ Failed"
            echo ""
            ((failed++)) || true
        fi

        ((total++)) || true
    fi
done

# Print summary
echo "=== Summary ==="
echo "Total:  $total"
echo "Passed: $success"
echo "Failed: $failed"
echo ""

if [ $failed -eq 0 ]; then
    echo "✓ All conversions completed successfully!"
    echo ""
    echo "Output files are in: $OUTPUT_DIR"
    echo ""
    echo "Next steps:"
    echo "  1. Open the SVG files in a browser or image viewer"
    echo "  2. Compare with original PNG files"
    echo "  3. Check file sizes and visual quality"
    echo "  4. Review examples/README.md for validation criteria"
else
    echo "✗ Some conversions failed. Check the errors above."
    exit 1
fi
