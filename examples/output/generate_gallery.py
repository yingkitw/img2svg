#!/usr/bin/env python3
"""Regenerate examples/output/quality_gallery.png — the input→output showcase
embedded in README.md.

Prerequisites:
  - img2svg built in release mode: `cargo build --release`
  - rsvg-convert (librsvg) and Pillow installed

Run from the repo root:
  python3 examples/output/generate_gallery.py
"""
import os
import subprocess
import sys
from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BIN = os.path.join(ROOT, "target", "release", "img2svg")
IN = os.path.join(ROOT, "examples", "input")
WORK = "/tmp/img2svg_gallery"
OUT = os.path.join(ROOT, "examples", "output", "quality_gallery.png")

# (input, case name, flags, render size)
CASES = [
    ("logo.png", "Flat-color logo", "--preset logo", 400),
    ("lenna.png", "Photograph", "-c 16", 512),
    ("icon.png", "Small icon", "--preset icon", 128),
    ("gradient.png", "Gradient", "--gradients", 400),
    ("transparency.png", "Transparency", "", 400),
]

TH = 300          # thumbnail height
PAD = 24
LABEL_W = 230
ARROW_W = 60
TITLE_H = 44
SUB_H = 26


def run(cmd):
    print("+", " ".join(cmd))
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def main():
    os.makedirs(WORK, exist_ok=True)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    if not os.path.exists(BIN):
        sys.exit(f"build the release binary first: cargo build --release (missing {BIN})")

    pairs = []
    for inp, name, flags, size in CASES:
        svg = os.path.join(WORK, inp.replace(".png", ".svg"))
        png = os.path.join(WORK, inp.replace(".png", "_out.png"))
        cmd = [BIN, "-i", os.path.join(IN, inp), "-o", svg]
        if flags:
            cmd += flags.split()
        run(cmd)
        run(["rsvg-convert", "-w", str(size), "-h", str(size), svg, "-o", png])
        pairs.append((inp, png, name, flags))

    try:
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 20)
        font_s = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 15)
        font_t = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 26)
    except Exception:
        font = font_s = font_t = ImageFont.load_default()

    def scale(im):
        w = int(im.width * TH / im.height)
        return im.resize((w, TH), Image.LANCZOS)

    rows = []
    for inp, png, name, flags in pairs:
        a = scale(Image.open(os.path.join(IN, inp)).convert("RGBA"))
        b = scale(Image.open(png).convert("RGBA"))
        row_w = LABEL_W + a.width + ARROW_W + b.width + PAD * 4
        row_h = TH + SUB_H + PAD
        row = Image.new("RGBA", (row_w, row_h), (255, 255, 255, 255))
        d = ImageDraw.Draw(row)
        d.text((PAD, PAD + 8), name, fill=(20, 20, 20, 255), font=font)
        d.text((PAD, PAD + 34), flags or "(default)", fill=(120, 120, 120, 255), font=font_s)
        x = LABEL_W
        row.paste(a, (x, PAD + SUB_H), a)
        d.text((x, PAD), "Raster input", fill=(90, 90, 90, 255), font=font_s)
        x += a.width + ARROW_W // 2 - 10
        d.text((x, PAD + TH // 2 - 14), "→", fill=(150, 150, 150, 255), font=font_t)
        x += ARROW_W // 2 + 10
        row.paste(b, (x, PAD + SUB_H), b)
        d.text((x, PAD), "SVG output", fill=(90, 90, 90, 255), font=font_s)
        rows.append(row)

    W = max(r.width for r in rows)
    H = TITLE_H + sum(r.height for r in rows) + PAD
    gallery = Image.new("RGBA", (W, H), (255, 255, 255, 255))
    d = ImageDraw.Draw(gallery)
    d.text((PAD, 10), "img2svg — raster → SVG quality samples", fill=(20, 20, 20, 255), font=font_t)
    y = TITLE_H
    for r in rows:
        gallery.paste(r, ((W - r.width) // 2, y), r)
        y += r.height

    flat = Image.new("RGB", gallery.size, (255, 255, 255))
    flat.paste(gallery, (0, 0), gallery)
    flat.save(OUT, "PNG")
    print("saved", OUT, flat.size)


if __name__ == "__main__":
    main()
