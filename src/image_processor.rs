pub use anyhow::Result;
use rgb::RGBA8;

#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<RGBA8>,
}

pub fn load_image(path: &std::path::Path) -> Result<ImageData> {
    let img = image::open(path)?;
    let rgba = img.to_rgba8();

    let pixels: Vec<RGBA8> = rgba
        .pixels()
        .map(|p| RGBA8::new(p[0], p[1], p[2], p[3]))
        .collect();

    Ok(ImageData {
        width: rgba.width(),
        height: rgba.height(),
        pixels,
    })
}

/// Median-cut color quantization for better color space coverage.
pub fn quantize_colors(image_data: &ImageData, num_colors: usize) -> Result<ImageData> {
    if num_colors == 0 {
        return Err(anyhow::anyhow!("num_colors must be greater than 0"));
    }

    let palette = median_cut(&image_data.pixels, num_colors);

    let mut quantized_pixels = Vec::with_capacity(image_data.pixels.len());
    for pixel in &image_data.pixels {
        let closest = palette
            .iter()
            .min_by_key(|c| {
                let dr = c.r as i32 - pixel.r as i32;
                let dg = c.g as i32 - pixel.g as i32;
                let db = c.b as i32 - pixel.b as i32;
                dr * dr + dg * dg + db * db
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to quantize: empty palette"))?;
        quantized_pixels.push(RGBA8::new(closest.r, closest.g, closest.b, pixel.a));
    }

    Ok(ImageData {
        width: image_data.width,
        height: image_data.height,
        pixels: quantized_pixels,
    })
}

/// Median-cut: recursively split the color box along its widest channel.
fn median_cut(pixels: &[RGBA8], num_colors: usize) -> Vec<RGBA8> {
    if num_colors == 0 {
        return vec![];
    }

    // Collect unique-ish colors (sample for performance on large images)
    let mut colors: Vec<(u8, u8, u8)> = Vec::new();
    let step = (pixels.len() / 50000).max(1);
    for (i, p) in pixels.iter().enumerate() {
        if i % step == 0 {
            colors.push((p.r, p.g, p.b));
        }
    }
    if colors.is_empty() {
        return vec![RGBA8::new(0, 0, 0, 255)];
    }

    let mut boxes: Vec<Vec<(u8, u8, u8)>> = vec![colors];
    while boxes.len() < num_colors {
        // Find the box with the largest range to split
        let mut best_idx = 0;
        let mut best_range = 0u16;
        for (i, b) in boxes.iter().enumerate() {
            let range = box_max_range(b);
            if range > best_range || (range == best_range && b.len() > boxes[best_idx].len()) {
                best_range = range;
                best_idx = i;
            }
        }
        if boxes[best_idx].len() < 2 {
            break;
        }
        let to_split = boxes.remove(best_idx);
        let (a, b) = split_box(to_split);
        if !a.is_empty() {
            boxes.push(a);
        }
        if !b.is_empty() {
            boxes.push(b);
        }
    }

    boxes.iter().map(|b| box_average(b)).collect()
}

pub fn box_max_range(colors: &[(u8, u8, u8)]) -> u16 {
    let (mut rmin, mut rmax) = (255u8, 0u8);
    let (mut gmin, mut gmax) = (255u8, 0u8);
    let (mut bmin, mut bmax) = (255u8, 0u8);
    for &(r, g, b) in colors {
        rmin = rmin.min(r); rmax = rmax.max(r);
        gmin = gmin.min(g); gmax = gmax.max(g);
        bmin = bmin.min(b); bmax = bmax.max(b);
    }
    let rr = (rmax - rmin) as u16;
    let gr = (gmax - gmin) as u16;
    let br = (bmax - bmin) as u16;
    rr.max(gr).max(br)
}

pub fn split_box(mut colors: Vec<(u8, u8, u8)>) -> (Vec<(u8, u8, u8)>, Vec<(u8, u8, u8)>) {
    let (mut rmin, mut rmax) = (255u8, 0u8);
    let (mut gmin, mut gmax) = (255u8, 0u8);
    let (mut bmin, mut bmax) = (255u8, 0u8);
    for &(r, g, b) in &colors {
        rmin = rmin.min(r); rmax = rmax.max(r);
        gmin = gmin.min(g); gmax = gmax.max(g);
        bmin = bmin.min(b); bmax = bmax.max(b);
    }
    let rr = rmax - rmin;
    let gr = gmax - gmin;
    let br = bmax - bmin;

    if rr >= gr && rr >= br {
        colors.sort_by_key(|c| c.0);
    } else if gr >= br {
        colors.sort_by_key(|c| c.1);
    } else {
        colors.sort_by_key(|c| c.2);
    }

    let mid = colors.len() / 2;
    let right = colors.split_off(mid);
    (colors, right)
}

pub fn box_average(colors: &[(u8, u8, u8)]) -> RGBA8 {
    if colors.is_empty() {
        return RGBA8::new(0, 0, 0, 255);
    }
    let (mut sr, mut sg, mut sb) = (0u64, 0u64, 0u64);
    for &(r, g, b) in colors {
        sr += r as u64;
        sg += g as u64;
        sb += b as u64;
    }
    let n = colors.len() as u64;
    RGBA8::new((sr / n) as u8, (sg / n) as u8, (sb / n) as u8, 255)
}

#[cfg(test)]
mod tests {
    include!("image_processor_tests.rs");
}
