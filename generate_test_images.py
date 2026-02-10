#!/usr/bin/env python3
"""Generate test images of varying complexity for img2svg conversion testing."""

from PIL import Image, ImageDraw
import os

# Create output directory
output_dir = "examples/input"
os.makedirs(output_dir, exist_ok=True)

# Image size
WIDTH, HEIGHT = 400, 400

def create_simple_image():
    """Simple: Basic geometric shapes with solid colors"""
    img = Image.new('RGB', (WIDTH, HEIGHT), 'white')
    draw = ImageDraw.Draw(img)

    # Red circle
    draw.ellipse([50, 50, 150, 150], fill='red')

    # Blue rectangle
    draw.rectangle([200, 50, 350, 150], fill='blue')

    # Green triangle
    draw.polygon([(125, 200), (50, 350), (200, 350)], fill='green')

    # Yellow rectangle
    draw.rectangle([250, 200, 350, 350], fill='yellow')

    img.save(f"{output_dir}/simple.png")
    print("Created simple.png - Basic shapes with solid colors")


def create_medium_image():
    """Medium: Multiple shapes with some overlapping and varied colors"""
    img = Image.new('RGB', (WIDTH, HEIGHT), 'white')
    draw = ImageDraw.Draw(img)

    # Background shapes
    draw.rectangle([0, 0, 200, 200], fill='#FFB6C1')  # Light pink
    draw.rectangle([200, 0, 400, 200], fill='#98FB98')  # Pale green
    draw.rectangle([0, 200, 200, 400], fill='#87CEEB')  # Sky blue
    draw.rectangle([200, 200, 400, 400], fill='#DDA0DD')  # Plum

    # Overlapping circles
    draw.ellipse([100, 100, 200, 200], fill='#FF6347')  # Tomato
    draw.ellipse([150, 100, 250, 200], fill='#4169E1')  # Royal blue
    draw.ellipse([200, 150, 300, 250], fill='#32CD32')  # Lime green

    # Additional shapes
    draw.rectangle([50, 250, 150, 350], fill='#FFD700')  # Gold
    draw.polygon([(300, 250), (250, 350), (350, 350)], fill='#FF4500')  # Orange red

    img.save(f"{output_dir}/medium.png")
    print("Created medium.png - Multiple shapes with overlapping")


def create_complex_image():
    """Complex: Many shapes, patterns, and color variations"""
    img = Image.new('RGB', (WIDTH, HEIGHT), '#F5F5DC')  # Beige background
    draw = ImageDraw.Draw(img)

    # Create a grid pattern
    for x in range(0, WIDTH, 50):
        for y in range(0, HEIGHT, 50):
            color = f'#{(x*3%255):02x}{(y*2%255):02x}{((x+y)*2%255):02x}'
            if (x + y) % 100 == 0:
                draw.rectangle([x, y, x+48, y+48], fill=color)

    # Overlay various shapes
    colors = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#FFA07A', '#98D8C8',
              '#F7DC6F', '#BB8FCE', '#85C1E2', '#F8B500', '#00CED1']

    # Circles
    for i in range(10):
        x = 30 + i * 35
        y = 30 + (i % 3) * 100
        size = 25 + i * 3
        draw.ellipse([x, y, x+size, y+size], fill=colors[i % len(colors)])

    # Rectangles
    for i in range(8):
        x = 20 + i * 45
        y = 150 + (i % 2) * 80
        draw.rectangle([x, y, x+35, y+60], fill=colors[(i+3) % len(colors)])

    # Triangles
    for i in range(6):
        x = 50 + i * 60
        points = [(x, 300), (x-25, 370), (x+25, 370)]
        draw.polygon(points, fill=colors[(i+6) % len(colors)])

    # Small detail circles
    for i in range(20):
        x = 10 + (i % 10) * 40
        y = 350 + (i // 10) * 25
        draw.ellipse([x, y, x+8, y+8], fill=colors[i % len(colors)])

    img.save(f"{output_dir}/complex.png")
    print("Created complex.png - Many shapes with patterns")


def create_very_complex_image():
    """Very Complex: High detail with many color transitions and shapes"""
    img = Image.new('RGB', (WIDTH, HEIGHT), 'white')
    draw = ImageDraw.Draw(img)

    # Create concentric circles pattern (like a target)
    center_x, center_y = WIDTH // 2, HEIGHT // 2
    colors_rainbow = ['#FF0000', '#FF7F00', '#FFFF00', '#00FF00',
                      '#0000FF', '#4B0082', '#9400D3']

    for i in range(20, 0, -1):
        radius = i * 10
        color = colors_rainbow[i % len(colors_rainbow)]
        draw.ellipse([center_x - radius, center_y - radius,
                     center_x + radius, center_y + radius], fill=color)

    # Add corner decorations
    corners = [(0, 0), (WIDTH-80, 0), (0, HEIGHT-80), (WIDTH-80, HEIGHT-80)]
    for idx, (x, y) in enumerate(corners):
        # Mini pattern in each corner
        for i in range(8):
            for j in range(8):
                color = f'#{((i*30+idx*60)%255):02x}{((j*30+idx*40)%255):02x}{((i+j+idx)*20%255):02x}'
                draw.rectangle([x+i*10, y+j*10, x+i*10+8, y+j*10+8], fill=color)

    # Overlay semi-transparent effect simulation using alternating stripes
    for y in range(0, HEIGHT, 4):
        color = '#FFFFFF' if (y // 4) % 2 == 0 else '#E0E0E0'
        draw.rectangle([0, y, WIDTH, y+2], fill=color)

    # Add scattered small shapes
    import random
    random.seed(42)  # Reproducible
    for _ in range(100):
        x = random.randint(0, WIDTH-10)
        y = random.randint(0, HEIGHT-10)
        size = random.randint(3, 12)
        color = f'#{random.randint(0,255):02x}{random.randint(0,255):02x}{random.randint(0,255):02x}'
        shape_type = random.choice(['circle', 'rect'])
        if shape_type == 'circle':
            draw.ellipse([x, y, x+size, y+size], fill=color)
        else:
            draw.rectangle([x, y, x+size, y+size], fill=color)

    img.save(f"{output_dir}/very_complex.png")
    print("Created very_complex.png - High detail with many elements")


def create_gradient_test():
    """Gradient simulation: Test color transitions"""
    img = Image.new('RGB', (WIDTH, HEIGHT), 'white')
    draw = ImageDraw.Draw(img)

    # Horizontal gradient simulation
    for x in range(WIDTH):
        r = int(255 * (1 - x / WIDTH))
        g = int(255 * x / WIDTH)
        color = f'#{r:02x}{g:02x}80'
        draw.line([(x, 0), (x, HEIGHT//2)], fill=color)

    # Vertical gradient simulation
    for y in range(HEIGHT // 2, HEIGHT):
        b = int(255 * (y - HEIGHT//2) / (HEIGHT//2))
        color = f'#80{b:02x}{255-b:02x}'
        draw.line([(0, y), (WIDTH, y)], fill=color)

    img.save(f"{output_dir}/gradient.png")
    print("Created gradient.png - Color gradient transitions")


if __name__ == "__main__":
    print("Generating test images for img2svg conversion testing...\n")

    create_simple_image()
    create_medium_image()
    create_complex_image()
    create_very_complex_image()

    # Try gradient test, may not work in all PIL versions
    try:
        create_gradient_test()
    except Exception as e:
        print(f"Skipped gradient.png due to: {e}")

    print(f"\nAll test images saved to {output_dir}/")
    print("\nComplexity levels:")
    print("  1. simple.png - Basic geometric shapes (good for basic conversion)")
    print("  2. medium.png - Multiple overlapping shapes (tests edge handling)")
    print("  3. complex.png - Many shapes and patterns (tests performance)")
    print("  4. very_complex.png - High detail (stress test)")
    if os.path.exists(f"{output_dir}/gradient.png"):
        print("  5. gradient.png - Color gradients (tests color accuracy)")
