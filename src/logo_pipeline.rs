//! Logo pipeline: separate fill regions from outline strokes.

use crate::vectorizer::{marching_squares_contours, Point};

/// Outline ring mask = dilated XOR eroded foreground.
pub fn outline_mask(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    let dilated = dilate(mask, width, height);
    let eroded = erode(mask, width, height);
    dilated
        .iter()
        .zip(eroded.iter())
        .map(|(&d, &e)| d && !e)
        .collect()
}

fn dilate(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    let mut out = vec![false; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut v = false;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                        v |= mask[ny as usize * width + nx as usize];
                    }
                }
            }
            out[y * width + x] = v;
        }
    }
    out
}

fn erode(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    let mut out = vec![false; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut v = true;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        v = false;
                    } else {
                        v &= mask[ny as usize * width + nx as usize];
                    }
                }
            }
            out[y * width + x] = v;
        }
    }
    out
}

/// Extract fill contours and outline contours for a color mask.
pub fn extract_logo_layers(
    mask: &[bool],
    width: usize,
    height: usize,
) -> (Vec<Vec<Point>>, Vec<Vec<Point>>) {
    let fill = marching_squares_contours(mask, width, height);
    let ring = outline_mask(mask, width, height);
    let outline = marching_squares_contours(&ring, width, height);
    (fill, outline)
}

/// Darken outline color slightly for visual separation.
pub fn outline_color(color: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
    (
        color.0.saturating_sub(40),
        color.1.saturating_sub(40),
        color.2.saturating_sub(40),
        color.3,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_ring_on_square() {
        let w = 10;
        let h = 10;
        let mut mask = vec![false; w * h];
        for y in 2..8 {
            for x in 2..8 {
                mask[y * w + x] = true;
            }
        }
        let ring = outline_mask(&mask, w, h);
        let fill_count = mask
            .iter()
            .zip(ring.iter())
            .filter(|(m, r)| **m && !**r)
            .count();
        assert!(ring.iter().any(|&v| v));
        assert!(fill_count < mask.iter().filter(|&&v| v).count());
    }

    #[test]
    fn extract_layers_non_empty() {
        let w = 12;
        let h = 12;
        let mut mask = vec![false; w * h];
        for y in 3..9 {
            for x in 3..9 {
                mask[y * w + x] = true;
            }
        }
        let (fill, outline) = extract_logo_layers(&mask, w, h);
        assert!(!fill.is_empty());
        assert!(!outline.is_empty());
    }
}
