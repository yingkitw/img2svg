use crate::image_processor::Result;
use crate::vectorizer::{Point, VectorizedData};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn generate_svg(vectorized_data: &VectorizedData, output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path)?;
    let bg = vectorized_data.background_color;
    let bg_str = format!("#{:02x}{:02x}{:02x}", bg.0, bg.1, bg.2);

    writeln!(
        file,
        r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
        vectorized_data.width, vectorized_data.height,
        vectorized_data.width, vectorized_data.height
    )?;
    writeln!(
        file,
        r#"  <rect width="{}" height="{}" fill="{}"/>"#,
        vectorized_data.width, vectorized_data.height, bg_str
    )?;

    for curve in &vectorized_data.curves {
        let color_str = format!(
            "#{:02x}{:02x}{:02x}",
            curve.color.0, curve.color.1, curve.color.2
        );

        let path_str = if !curve.subpaths.is_empty() {
            create_multi_path_string(&curve.subpaths)
        } else if !curve.points.is_empty() {
            create_subpath_string(&curve.points, curve.is_closed)
        } else {
            continue;
        };

        if path_str.is_empty() {
            continue;
        }

        writeln!(
            file,
            r#"  <path d="{}" fill="{}" stroke="none"/>"#,
            path_str, color_str
        )?;
    }

    writeln!(file, "</svg>")?;
    Ok(())
}

pub fn generate_svg_advanced(vectorized_data: &VectorizedData, output_path: &Path) -> Result<()> {
    // Advanced mode uses the same output — colors are already grouped by the vectorizer
    generate_svg(vectorized_data, output_path)
}

/// Build a single SVG path `d` attribute containing multiple M...Z subpaths.
pub fn create_multi_path_string(subpaths: &[Vec<Point>]) -> String {
    let mut path = String::new();
    for sp in subpaths {
        if sp.len() < 3 {
            continue;
        }
        if !path.is_empty() {
            path.push(' ');
        }
        path.push_str(&create_subpath_string(sp, true));
    }
    path
}

/// Format a coordinate compactly: use integer if close to whole, else 1 decimal.
fn fmt_coord(v: f64) -> String {
    let rounded = (v * 2.0).round() / 2.0; // snap to 0.5 grid
    if (rounded - rounded.round()).abs() < 0.01 {
        format!("{}", rounded.round() as i32)
    } else {
        format!("{:.1}", rounded)
    }
}

/// Convert a single list of points into an SVG subpath using line segments.
/// Marching squares + RDP already produces accurate contours; line segments
/// are compact and browsers anti-alias them smoothly.
pub fn create_subpath_string(pts: &[Point], closed: bool) -> String {
    let n = pts.len();
    if n == 0 {
        return String::new();
    }

    let mut path = format!("M{} {}", fmt_coord(pts[0].x), fmt_coord(pts[0].y));

    for i in 1..n {
        path.push_str(&format!("L{} {}", fmt_coord(pts[i].x), fmt_coord(pts[i].y)));
    }
    if closed {
        path.push('Z');
    }

    path
}

#[cfg(test)]
mod tests {
    include!("svg_generator_tests.rs");
}
