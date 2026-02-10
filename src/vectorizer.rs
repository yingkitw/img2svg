use crate::image_processor::Result;
use crate::image_processor::{quantize_colors, ImageData};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct Curve {
    pub points: Vec<Point>,
    pub color: (u8, u8, u8, u8),
    pub is_closed: bool,
    pub subpaths: Vec<Vec<Point>>,
}

#[derive(Debug)]
pub struct VectorizedData {
    pub curves: Vec<Curve>,
    pub width: u32,
    pub height: u32,
    pub background_color: (u8, u8, u8, u8),
}

/// Region-based vectorization using marching-squares contour tracing.
/// For each unique color, builds a binary mask and traces sub-pixel-accurate
/// contours that properly enclose all pixels of that color.
pub fn vectorize(
    image_data: &ImageData,
    num_colors: usize,
    _threshold: f64,
    smooth_level: u8,
    _hierarchical: bool,
) -> Result<VectorizedData> {
    let quantized = quantize_colors(image_data, num_colors)?;
    let width = quantized.width as usize;
    let height = quantized.height as usize;

    // Group pixels by quantized color
    let mut color_pixels: HashMap<(u8, u8, u8, u8), Vec<(usize, usize)>> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let p = quantized.pixels[y * width + x];
            let key = (p.r, p.g, p.b, p.a);
            color_pixels.entry(key).or_default().push((x, y));
        }
    }

    // Sort colors by pixel count (largest area first for proper z-order)
    let mut color_list: Vec<_> = color_pixels.into_iter().collect();
    color_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let mut curves = Vec::new();
    let total_pixels = width * height;

    // First color (largest area) becomes the background rect
    let background_color = color_list.first().map(|(c, _)| *c).unwrap_or((255, 255, 255, 255));

    for (color, pixels) in &color_list {
        // Skip background — it will be a rect in the SVG
        if *color == background_color {
            continue;
        }
        // Build binary mask for this color
        let mut mask = vec![false; total_pixels];
        for &(x, y) in pixels {
            mask[y * width + x] = true;
        }

        // Trace contours using marching squares on the mask
        let contours = marching_squares_contours(&mask, width, height);

        // Collect all sub-paths for this color into one Curve with merged points
        let mut color_subpaths: Vec<Vec<Point>> = Vec::new();

        let w = width as f64;
        let h = height as f64;

        for contour in contours {
            if contour.len() < 4 {
                continue;
            }
            if polygon_area(&contour) < 8.0 {
                continue;
            }
            let processed = if smooth_level > 0 {
                smooth_boundary(&contour, smooth_level)
            } else {
                contour
            };
            let simplified = rdp_simplify(&processed, 2.0);
            // Snap points near image edges to exact boundary AFTER smoothing/simplification
            // so smoothing can't pull boundary points away from the edge.
            let snap = 4.0;
            let mut snapped: Vec<Point> = simplified.into_iter().map(|p| Point {
                x: if p.x < snap { 0.0 } else if p.x > w - snap { w } else { p.x },
                y: if p.y < snap { 0.0 } else if p.y > h - snap { h } else { p.y },
            }).collect();
            // Remove consecutive duplicate points created by snapping
            snapped.dedup_by(|a, b| (a.x - b.x).abs() < 0.1 && (a.y - b.y).abs() < 0.1);
            // Re-check area after snapping — some paths collapse to near-zero
            if snapped.len() >= 3 && polygon_area(&snapped) >= 8.0 {
                color_subpaths.push(snapped);
            }
        }

        if !color_subpaths.is_empty() {
            curves.push(Curve {
                points: Vec::new(), // Will use subpaths instead
                color: *color,
                is_closed: true,
                subpaths: color_subpaths,
            });
        }
    }

    Ok(VectorizedData {
        curves,
        width: image_data.width,
        height: image_data.height,
        background_color,
    })
}

/// Marching squares contour tracing on a binary mask.
/// Produces sub-pixel contours at the boundary between true/false cells.
/// The grid has (width+1) x (height+1) vertices; each cell (x,y) corresponds
/// to pixel (x,y). A cell is "inside" if mask[y*width+x] is true.
fn marching_squares_contours(
    mask: &[bool],
    width: usize,
    height: usize,
) -> Vec<Vec<Point>> {
    // Pixel (px, py) occupies the square [px, px+1] x [py, py+1].
    // We build a grid of (width+2) x (height+2) cells so that the image pixels
    // map to cells [1..width] x [1..height], with a 1-cell padding of "outside"
    // on all sides. This ensures contours at image edges close properly.
    //
    // Corner (gx, gy) is "inside" if pixel (gx-1, gy-1) exists and is true.
    // Cell (cx, cy) has corners (cx,cy), (cx+1,cy), (cx+1,cy+1), (cx,cy+1).
    // Edge midpoints in image coordinates: cell (cx, cy) maps to position (cx-1, cy-1)
    // in pixel space, so edge midpoints are offset by -0.5 from cell coords.

    let grid_w = width + 2;
    let grid_h = height + 2;

    let corner_inside = |gx: usize, gy: usize| -> bool {
        if gx == 0 || gy == 0 || gx > width || gy > height {
            return false;
        }
        mask[(gy - 1) * width + (gx - 1)]
    };

    let mut edge_visited: HashMap<(usize, usize, u8), bool> = HashMap::new();

    let cell_case = |cx: usize, cy: usize| -> u8 {
        let tl = corner_inside(cx, cy) as u8;
        let tr = corner_inside(cx + 1, cy) as u8;
        let br = corner_inside(cx + 1, cy + 1) as u8;
        let bl = corner_inside(cx, cy + 1) as u8;
        (tl << 3) | (tr << 2) | (br << 1) | bl
    };

    // Edge midpoints in pixel coordinates.
    // Cell (cx, cy) in grid space → pixel space is (cx - 0.5, cy - 0.5).
    // Edge midpoints:
    //   top:    (cx + 0.5, cy)     → pixel (cx - 0.5 + 0.5, cy - 0.5)     = (cx, cy - 0.5)
    //   right:  (cx + 1,   cy+0.5) → pixel (cx - 0.5 + 1,   cy - 0.5+0.5) = (cx+0.5, cy)
    //   bottom: (cx + 0.5, cy + 1) → pixel (cx,              cy + 0.5)
    //   left:   (cx,       cy+0.5) → pixel (cx - 0.5,        cy)
    // But simpler: since corner (1,1) = pixel (0,0), the edge midpoint between
    // corners maps directly. We just subtract 0.5 from the raw grid midpoint.
    let w = width as f64;
    let h = height as f64;
    let edge_point = move |cx: usize, cy: usize, side: u8| -> Point {
        let (x, y) = match side {
            0 => (cx as f64 + 0.5, cy as f64),         // top edge midpoint in grid
            1 => ((cx + 1) as f64, cy as f64 + 0.5),   // right
            2 => (cx as f64 + 0.5, (cy + 1) as f64),   // bottom
            3 => (cx as f64, cy as f64 + 0.5),         // left
            _ => unreachable!(),
        };
        // Convert grid coords to pixel coords: subtract 1 (grid offset) + 0.5 = shift by -0.5
        // But the midpoint already adds 0.5, so net: subtract 0.5 from grid coords.
        // Then clamp to image bounds.
        Point {
            x: (x - 0.5).clamp(0.0, w),
            y: (y - 0.5).clamp(0.0, h),
        }
    };

    // For each case, the edges that form segments.
    // Returns pairs of (entry_side, exit_side).
    // Sides: 0=top, 1=right, 2=bottom, 3=left
    let case_edges = |case: u8| -> Vec<(u8, u8)> {
        match case {
            0 | 15 => vec![],
            1  => vec![(2, 3)],
            2  => vec![(1, 2)],
            3  => vec![(1, 3)],
            4  => vec![(0, 1)],
            5  => vec![(0, 1), (2, 3)], // saddle
            6  => vec![(0, 2)],
            7  => vec![(0, 3)],
            8  => vec![(3, 0)],
            9  => vec![(2, 0)],
            10 => vec![(3, 0), (1, 2)], // saddle
            11 => vec![(1, 0)],
            12 => vec![(3, 1)],
            13 => vec![(2, 1)],
            14 => vec![(3, 2)],
            _  => vec![],
        }
    };

    let opposite_side = |side: u8| -> u8 {
        match side {
            0 => 2,
            1 => 3,
            2 => 0,
            3 => 1,
            _ => unreachable!(),
        }
    };

    let neighbor_cell = move |cx: usize, cy: usize, side: u8| -> Option<(usize, usize)> {
        match side {
            0 if cy > 0          => Some((cx, cy - 1)),
            1 if cx + 1 < grid_w => Some((cx + 1, cy)),
            2 if cy + 1 < grid_h => Some((cx, cy + 1)),
            3 if cx > 0          => Some((cx - 1, cy)),
            _ => None,
        }
    };

    let mut contours = Vec::new();

    for cy in 0..grid_h {
        for cx in 0..grid_w {
            let case = cell_case(cx, cy);
            let edges = case_edges(case);

            for &(entry, exit) in &edges {
                if edge_visited.contains_key(&(cx, cy, entry)) {
                    continue;
                }

                // Start a new contour by following the chain
                let mut contour = Vec::new();
                let mut cur_cx = cx;
                let mut cur_cy = cy;
                let mut cur_entry = entry;
                let mut cur_exit = exit;

                let start_key = (cx, cy, entry);

                loop {
                    edge_visited.insert((cur_cx, cur_cy, cur_entry), true);
                    edge_visited.insert((cur_cx, cur_cy, cur_exit), true);
                    contour.push(edge_point(cur_cx, cur_cy, cur_exit));

                    // Move to neighbor cell through the exit edge
                    let next_entry_side = opposite_side(cur_exit);
                    let next_cell = neighbor_cell(cur_cx, cur_cy, cur_exit);

                    if let Some((ncx, ncy)) = next_cell {
                        let ncase = cell_case(ncx, ncy);
                        let nedges = case_edges(ncase);

                        // Find the edge pair that enters from next_entry_side
                        if let Some(&(ne, nx)) = nedges.iter().find(|&&(e, _)| e == next_entry_side) {
                            if (ncx, ncy, ne) == start_key {
                                break; // Closed contour
                            }
                            cur_cx = ncx;
                            cur_cy = ncy;
                            cur_entry = ne;
                            cur_exit = nx;
                        } else {
                            break; // Dead end
                        }
                    } else {
                        break; // Hit image boundary
                    }
                }

                if contour.len() >= 3 {
                    contours.push(contour);
                }
            }
        }
    }

    contours
}

fn polygon_area(points: &[Point]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    area.abs() / 2.0
}

/// Light Gaussian smoothing that doesn't add points (unlike Chaikin).
/// Averages each point with its neighbors, preserving point count.
fn smooth_boundary(points: &[Point], level: u8) -> Vec<Point> {
    if level == 0 || points.len() < 3 {
        return points.to_vec();
    }

    let mut current = points.to_vec();
    let iterations = (level as usize).min(3);

    for _ in 0..iterations {
        let n = current.len();
        let mut new_points = Vec::with_capacity(n);

        for i in 0..n {
            let prev = &current[(i + n - 1) % n];
            let curr = &current[i];
            let next = &current[(i + 1) % n];

            new_points.push(Point {
                x: 0.25 * prev.x + 0.5 * curr.x + 0.25 * next.x,
                y: 0.25 * prev.y + 0.5 * curr.y + 0.25 * next.y,
            });
        }

        current = new_points;
    }

    current
}

/// Ramer-Douglas-Peucker path simplification.
fn rdp_simplify(points: &[Point], epsilon: f64) -> Vec<Point> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut max_dist = 0.0;
    let mut max_idx = 0;
    let first = &points[0];
    let last = &points[points.len() - 1];

    for i in 1..points.len() - 1 {
        let d = point_to_line_distance(&points[i], first, last);
        if d > max_dist {
            max_dist = d;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        let mut left = rdp_simplify(&points[..=max_idx], epsilon);
        let right = rdp_simplify(&points[max_idx..], epsilon);
        left.pop();
        left.extend(right);
        left
    } else {
        vec![first.clone(), last.clone()]
    }
}

fn point_to_line_distance(point: &Point, line_start: &Point, line_end: &Point) -> f64 {
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;
    let len_sq = dx * dx + dy * dy;

    if len_sq < 1e-10 {
        return ((point.x - line_start.x).powi(2) + (point.y - line_start.y).powi(2)).sqrt();
    }

    let t = ((point.x - line_start.x) * dx + (point.y - line_start.y) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);

    let proj_x = line_start.x + t * dx;
    let proj_y = line_start.y + t * dy;

    ((point.x - proj_x).powi(2) + (point.y - proj_y).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    include!("vectorizer_tests.rs");
}
