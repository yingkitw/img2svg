//! Flood-fill region extraction with 8-connectivity and Moore neighborhood
//! contour tracing. Replaces marching-squares for exact region margins.
//!
//! Ported from vec project's ContourTracer for better region boundary accuracy.

use crate::vectorizer::Point;
use std::collections::VecDeque;

/// A region extracted from the quantized image.
#[derive(Debug, Clone)]
pub struct Region {
    pub color: (u8, u8, u8, u8),
    pub boundary: Vec<Point>,
    pub area: usize,
}

/// Extract regions by flood-filling on palette indices.
/// Each region's boundary is traced using Moore neighborhood tracing.
pub fn extract_regions_by_index(
    width: u32,
    height: u32,
    indices: &[usize],
    palette: &[rgb::RGBA8],
    min_area: usize,
) -> Vec<Region> {
    let w = width as usize;
    let h = height as usize;
    let total = w * h;
    if indices.len() != total {
        return Vec::new();
    }

    let mut visited = vec![false; total];
    let mut regions = Vec::new();

    for y in 0..h {
        let row_offset = y * w;
        for x in 0..w {
            let idx = row_offset + x;
            if visited[idx] {
                continue;
            }

            let seed_idx = indices[idx];
            let region_pixels = flood_fill_by_index(
                x as u32, y as u32, width, height, indices, seed_idx, &mut visited,
            );

            if region_pixels.len() < min_area {
                continue;
            }

            // Build bitmap for boundary extraction
            let mut region_bitmap = vec![false; total];
            for &(rx, ry) in &region_pixels {
                region_bitmap[ry as usize * w + rx as usize] = true;
            }

            let boundary = follow_boundary(&region_bitmap, &region_pixels, width, height);

            if boundary.len() >= 3 {
                let c = &palette[seed_idx];
                regions.push(Region {
                    color: (c.r, c.g, c.b, c.a),
                    boundary,
                    area: region_pixels.len(),
                });
            }
        }
    }

    regions
}

/// 8-connectivity flood fill by exact palette index match.
fn flood_fill_by_index(
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    indices: &[usize],
    seed_idx: usize,
    visited: &mut [bool],
) -> Vec<(u32, u32)> {
    let w = width as usize;
    let iw = width as i32;
    let ih = height as i32;

    let start_flat = start_y as usize * w + start_x as usize;
    let mut region = Vec::new();
    let mut queue = VecDeque::new();

    queue.push_back((start_x, start_y));
    visited[start_flat] = true;

    while let Some((x, y)) = queue.pop_front() {
        region.push((x, y));

        let ix = x as i32;
        let iy = y as i32;

        // 8-connectivity
        for (dx, dy) in [
            (0i32, 1i32), (1, 0), (0, -1), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ] {
            let nx = ix + dx;
            let ny = iy + dy;
            if nx >= 0 && nx < iw && ny >= 0 && ny < ih {
                let nidx = ny as usize * w + nx as usize;
                if !visited[nidx] && indices[nidx] == seed_idx {
                    visited[nidx] = true;
                    queue.push_back((nx as u32, ny as u32));
                }
            }
        }
    }

    region
}

/// Extract boundary points from a region bitmap by scanning for boundary pixels.
/// A boundary pixel is one that has at least one 4-connected neighbor outside the region.
/// Returns an ordered boundary by tracing clockwise using 4-connectivity.
fn follow_boundary(
    region_bitmap: &[bool],
    region: &[(u32, u32)],
    width: u32,
    height: u32,
) -> Vec<Point> {
    let w = width as usize;
    let iw = width as i32;
    let ih = height as i32;

    let is_in_region = |x: i32, y: i32| -> bool {
        x >= 0 && x < iw && y >= 0 && y < ih && region_bitmap[y as usize * w + x as usize]
    };

    // Collect all boundary pixels (4-connected: has at least one non-region cardinal neighbor)
    let mut boundary_set = vec![false; w * height as usize];
    let mut start: Option<(i32, i32)> = None;

    for &(x, y) in region {
        let ix = x as i32;
        let iy = y as i32;
        let mut on_boundary = false;
        for &(dx, dy) in &[(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
            if !is_in_region(ix + dx, iy + dy) {
                on_boundary = true;
                break;
            }
        }
        if on_boundary {
            boundary_set[y as usize * w + x as usize] = true;
            match start {
                None => start = Some((ix, iy)),
                Some((sx, sy)) => {
                    if iy < sy || (iy == sy && ix < sx) {
                        start = Some((ix, iy));
                    }
                }
            }
        }
    }

    let (start_x, start_y) = match start {
        Some(s) => s,
        None => return Vec::new(),
    };

    // Trace the boundary clockwise using 4-connectivity only.
    // Directions: 0=right, 1=down, 2=left, 3=up
    let dirs_4: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

    let is_boundary = |x: i32, y: i32| -> bool {
        x >= 0 && x < iw && y >= 0 && y < ih && boundary_set[y as usize * w + x as usize]
    };

    let mut boundary = Vec::new();
    let mut cx = start_x;
    let mut cy = start_y;
    // Start direction: since start is topmost-leftmost, the pixel above is outside.
    // We enter from above (dir=3=up), so we start searching from the right of that: (3+1)%4 = 0 = right
    // But for proper clockwise tracing, we start looking one step back: (prev_dir + 3) % 4
    let mut prev_dir = 3usize; // came from above

    let max_steps = region.len() * 4 + 4;

    for step in 0..max_steps {
        boundary.push(Point {
            x: cx as f64,
            y: cy as f64,
        });

        // Search clockwise starting from direction (prev_dir + 3) % 4
        // This means: turn right from where we came, then sweep clockwise
        let search_start = (prev_dir + 3) % 4;
        let mut found = false;

        for i in 0..4 {
            let d = (search_start + i) % 4;
            let nx = cx + dirs_4[d].0;
            let ny = cy + dirs_4[d].1;

            if is_boundary(nx, ny) {
                cx = nx;
                cy = ny;
                prev_dir = d;
                found = true;
                break;
            }
        }

        if !found {
            break;
        }

        if cx == start_x && cy == start_y {
            // Check if we've completed the loop
            if step > 0 {
                break;
            }
        }
    }

    // Deduplicate consecutive identical points
    boundary.dedup_by(|a, b| (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() < 0.01);

    // Curvature-aware subsampling for large contours
    if boundary.len() > 2000 {
        curvature_aware_subsample(boundary)
    } else {
        boundary
    }
}

/// Curvature-aware subsampling: keeps more points where boundary curves,
/// aggressively subsamples straight segments.
fn curvature_aware_subsample(points: Vec<Point>) -> Vec<Point> {
    let n = points.len();
    if n <= 100 {
        return points;
    }

    let perimeter = estimate_perimeter(&points);
    let base_target = 4000.min(n);
    let adaptive_target = ((perimeter / 10.0) as usize).min(base_target).max(50);

    // Compute curvature at each point
    let mut curvatures = Vec::with_capacity(n);
    for i in 0..n {
        curvatures.push(compute_curvature(&points, i));
    }

    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;

    // Sort curvatures to find threshold
    let mut sorted_curv: Vec<f64> = curvatures.clone();
    sorted_curv.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let percentile_idx = adaptive_target.min(n - 2);
    let curvature_threshold = sorted_curv.get(percentile_idx).copied().unwrap_or(0.0);

    let mut kept = 2;
    for i in 1..n - 1 {
        if curvatures[i] >= curvature_threshold.max(0.01) {
            keep[i] = true;
            kept += 1;
        }
    }

    // Ensure minimum sampling density
    let max_gap = (perimeter / (2.0 * adaptive_target as f64)).max(5.0);
    enforce_max_gap(&points, &mut keep, max_gap);

    // If still need more, uniform sample
    if kept < adaptive_target {
        let remaining = adaptive_target - kept;
        let step = (n as f64 / remaining as f64).max(2.0) as usize;
        for i in (0..n).step_by(step) {
            keep[i] = true;
        }
    }

    points
        .into_iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, p)| p)
        .collect()
}

fn estimate_perimeter(points: &[Point]) -> f64 {
    let mut perimeter = 0.0;
    for i in 1..points.len() {
        let dx = points[i].x - points[i - 1].x;
        let dy = points[i].y - points[i - 1].y;
        perimeter += (dx * dx + dy * dy).sqrt();
    }
    perimeter
}

fn compute_curvature(points: &[Point], idx: usize) -> f64 {
    let n = points.len();
    if idx == 0 || idx >= n - 1 {
        return 0.0;
    }
    let v1x = points[idx].x - points[idx - 1].x;
    let v1y = points[idx].y - points[idx - 1].y;
    let v2x = points[idx + 1].x - points[idx].x;
    let v2y = points[idx + 1].y - points[idx].y;
    let len1 = (v1x * v1x + v1y * v1y).sqrt();
    let len2 = (v2x * v2x + v2y * v2y).sqrt();
    if len1 > 0.0 && len2 > 0.0 {
        (v1x * v2y - v1y * v2x).abs() / (len1 * len2)
    } else {
        0.0
    }
}

fn enforce_max_gap(points: &[Point], keep: &mut [bool], max_gap: f64) {
    let n = points.len();
    let mut last_kept = 0usize;

    for i in 1..n {
        if keep[i] {
            let mut gap_length = 0.0;
            for j in last_kept..i {
                let dx = points[j + 1].x - points[j].x;
                let dy = points[j + 1].y - points[j].y;
                gap_length += (dx * dx + dy * dy).sqrt();
            }
            if gap_length > max_gap {
                let num_insert = (gap_length / max_gap).ceil() as usize - 1;
                let step = (i - last_kept) as f64 / (num_insert + 1) as f64;
                for k in 1..=num_insert {
                    let insert_idx = last_kept + (k as f64 * step) as usize;
                    if insert_idx < i {
                        keep[insert_idx] = true;
                    }
                }
            }
            last_kept = i;
        }
    }
}

/// Detect background color by sampling border pixels (most frequent color).
pub fn detect_background_color(
    image_data: &crate::image_processor::ImageData,
) -> (u8, u8, u8, u8) {
    let w = image_data.width as usize;
    let h = image_data.height as usize;
    if w == 0 || h == 0 {
        return (255, 255, 255, 255);
    }

    let mut color_counts: std::collections::HashMap<u32, (usize, (u8, u8, u8, u8))> =
        std::collections::HashMap::new();

    let mut sample_border = |x: usize, y: usize| {
        let p = &image_data.pixels[y * w + x];
        let key = (p.r as u32) << 16 | (p.g as u32) << 8 | p.b as u32;
        let entry = color_counts
            .entry(key)
            .or_insert((0, (p.r, p.g, p.b, p.a)));
        entry.0 += 1;
    };

    // Sample all border pixels
    for x in 0..w {
        sample_border(x, 0);
        sample_border(x, h - 1);
    }
    for y in 1..h - 1 {
        sample_border(0, y);
        sample_border(w - 1, y);
    }

    // Deterministic tie-breaking: highest count wins; on tie, prefer lighter color
    // (lighter colors are more common backgrounds). Uses luminance as tie-breaker.
    color_counts
        .values()
        .max_by(|(count_a, ca), (count_b, cb)| {
            count_a.cmp(count_b).then_with(|| {
                let lum_a = ca.0 as u32 * 299 + ca.1 as u32 * 587 + ca.2 as u32 * 114;
                let lum_b = cb.0 as u32 * 299 + cb.1 as u32 * 587 + cb.2 as u32 * 114;
                lum_a.cmp(&lum_b)
            })
        })
        .map(|(_, color)| *color)
        .unwrap_or((255, 255, 255, 255))
}

/// Recolor regions using original (pre-quantized) image pixels for true color accuracy.
pub fn recolor_from_original(
    regions: &mut [Region],
    original: &crate::image_processor::ImageData,
    _indices: &[usize],
    _palette: &[rgb::RGBA8],
) {
    let w = original.width as usize;
    let h = original.height as usize;

    for region in regions.iter_mut() {
        // Sample original image colors along the boundary
        let mut sr: u64 = 0;
        let mut sg: u64 = 0;
        let mut sb: u64 = 0;
        let mut sa: u64 = 0;
        let mut count: u64 = 0;

        for pt in &region.boundary {
            let px = pt.x.round() as usize;
            let py = pt.y.round() as usize;
            if px < w && py < h {
                let p = &original.pixels[py * w + px];
                sr += p.r as u64;
                sg += p.g as u64;
                sb += p.b as u64;
                sa += p.a as u64;
                count += 1;
            }
        }

        if count > 0 {
            region.color = (
                (sr / count) as u8,
                (sg / count) as u8,
                (sb / count) as u8,
                (sa / count) as u8,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rgb::RGBA8;

    #[test]
    fn test_flood_fill_single_region() {
        // 4x4 image, all same index
        let indices = vec![0usize; 16];
        let mut visited = vec![false; 16];
        let region = flood_fill_by_index(0, 0, 4, 4, &indices, 0, &mut visited);
        assert_eq!(region.len(), 16);
    }

    #[test]
    fn test_flood_fill_two_regions() {
        // 4x4 image, left half index 0, right half index 1
        let mut indices = vec![0usize; 16];
        for y in 0..4 {
            for x in 2..4 {
                indices[y * 4 + x] = 1;
            }
        }
        let mut visited = vec![false; 16];
        let region0 = flood_fill_by_index(0, 0, 4, 4, &indices, 0, &mut visited);
        assert_eq!(region0.len(), 8);
        let region1 = flood_fill_by_index(2, 0, 4, 4, &indices, 1, &mut visited);
        assert_eq!(region1.len(), 8);
    }

    #[test]
    fn test_extract_regions_basic() {
        // 4x4 image with 2 colors
        let mut indices = vec![0usize; 16];
        for y in 0..4 {
            for x in 2..4 {
                indices[y * 4 + x] = 1;
            }
        }
        let palette = vec![
            RGBA8::new(255, 0, 0, 255),
            RGBA8::new(0, 0, 255, 255),
        ];
        let regions = extract_regions_by_index(4, 4, &indices, &palette, 1);
        assert_eq!(regions.len(), 2);
    }

    #[test]
    fn test_extract_regions_min_area_filter() {
        // Single pixel region should be filtered out with min_area=2
        let indices = vec![0, 1, 0, 0];
        let palette = vec![
            RGBA8::new(255, 0, 0, 255),
            RGBA8::new(0, 255, 0, 255),
        ];
        let regions = extract_regions_by_index(2, 2, &indices, &palette, 2);
        // Region with index 1 has only 1 pixel, should be filtered
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].color, (255, 0, 0, 255));
    }

    #[test]
    fn test_detect_background_color() {
        use crate::image_processor::ImageData;
        // 4x4 image, border is red, center is blue
        let mut pixels = vec![RGBA8::new(255, 0, 0, 255); 16];
        pixels[5] = RGBA8::new(0, 0, 255, 255);
        pixels[6] = RGBA8::new(0, 0, 255, 255);
        pixels[9] = RGBA8::new(0, 0, 255, 255);
        pixels[10] = RGBA8::new(0, 0, 255, 255);
        let img = ImageData { width: 4, height: 4, pixels };
        let bg = detect_background_color(&img);
        assert_eq!(bg, (255, 0, 0, 255));
    }

    #[test]
    fn test_curvature_straight_line() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 2.0, y: 0.0 },
        ];
        assert!(compute_curvature(&points, 1) < 0.01);
    }

    #[test]
    fn test_curvature_sharp_turn() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 1.0, y: 1.0 },
        ];
        assert!(compute_curvature(&points, 1) > 0.5);
    }
}
