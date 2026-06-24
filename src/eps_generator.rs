//! EPS and minimal PDF export from vector data.

use crate::bezier_fitter::{bezier_to_svg_path_to, BezierCurve};
use crate::enhanced_vectorizer::EnhancedVectorData;
use crate::image_processor::Result;
use crate::svg_generator::color_with_alpha;
use crate::vectorizer::{Point, VectorizedData};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn flip_y(y: f64, height: u32) -> f64 {
    height as f64 - y
}

fn write_eps_header<W: Write>(writer: &mut W, width: u32, height: u32) -> Result<()> {
    writeln!(writer, "%!PS-Adobe-3.0 EPSF-3.0")?;
    writeln!(writer, "%%BoundingBox: 0 0 {} {}", width, height)?;
    writeln!(writer, "%%HiResBoundingBox: 0 0 {} {}", width, height)?;
    writeln!(writer, "0 {} translate", height)?;
    writeln!(writer, "1 -1 scale")?;
    Ok(())
}

fn points_to_eps_path<W: Write>(writer: &mut W, points: &[Point], closed: bool, height: u32) -> Result<()> {
    if points.is_empty() {
        return Ok(());
    }
    writeln!(
        writer,
        "{:.2} {:.2} moveto",
        points[0].x,
        flip_y(points[0].y, height)
    )?;
    for p in points.iter().skip(1) {
        writeln!(writer, "{:.2} {:.2} lineto", p.x, flip_y(p.y, height))?;
    }
    if closed {
        writeln!(writer, "closepath")?;
    }
    Ok(())
}

fn curves_to_eps_path<W: Write>(writer: &mut W, curves: &[BezierCurve], closed: bool, height: u32) -> Result<()> {
    if curves.is_empty() {
        return Ok(());
    }
    let mut buf = Vec::new();
    bezier_to_svg_path_to(&mut buf, curves, closed)?;
    let s = String::from_utf8(buf).unwrap_or_default();
    for token in s.split_whitespace() {
        if token.starts_with('M') {
            let coords: Vec<f64> = token[1..]
                .split(',')
                .filter_map(|v| v.parse().ok())
                .collect();
            if coords.len() >= 2 {
                writeln!(writer, "{:.2} {:.2} moveto", coords[0], flip_y(coords[1], height))?;
            }
        } else if token.starts_with('L') {
            let coords: Vec<f64> = token[1..]
                .split(',')
                .filter_map(|v| v.parse().ok())
                .collect();
            if coords.len() >= 2 {
                writeln!(writer, "{:.2} {:.2} lineto", coords[0], flip_y(coords[1], height))?;
            }
        } else if token == "Z" {
            writeln!(writer, "closepath")?;
        }
    }
    Ok(())
}

/// Write legacy pipeline data as EPS.
pub fn write_vectorized_eps(data: &VectorizedData, output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path)?;
    write_eps_header(&mut file, data.width, data.height)?;

    let (bg, bg_op) = color_with_alpha(data.background_color);
    let bg_rgb = parse_hex(&bg);
    writeln!(
        &mut file,
        "{:.3} {:.3} {:.3} setrgbcolor",
        bg_rgb.0, bg_rgb.1, bg_rgb.2
    )?;
    writeln!(
        &mut file,
        "0 0 {} {} rectfill",
        data.width,
        data.height
    )?;
    let _ = bg_op;

    for curve in &data.curves {
        let (hex, _) = color_with_alpha(curve.color);
        let rgb = parse_hex(&hex);
        writeln!(&mut file, "{:.3} {:.3} {:.3} setrgbcolor", rgb.0, rgb.1, rgb.2)?;
        for sub in &curve.subpaths {
            points_to_eps_path(&mut file, sub, curve.is_closed, data.height)?;
            writeln!(&mut file, "fill")?;
        }
        if !curve.points.is_empty() {
            points_to_eps_path(&mut file, &curve.points, curve.is_closed, data.height)?;
            writeln!(&mut file, "fill")?;
        }
    }

    writeln!(&mut file, "showpage")?;
    Ok(())
}

/// Write enhanced pipeline data as EPS.
pub fn write_enhanced_eps(data: &EnhancedVectorData, output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path)?;
    write_eps_header(&mut file, data.width, data.height)?;

    let (bg, _) = color_with_alpha(data.background_color);
    let bg_rgb = parse_hex(&bg);
    writeln!(
        &mut file,
        "{:.3} {:.3} {:.3} setrgbcolor",
        bg_rgb.0, bg_rgb.1, bg_rgb.2
    )?;
    writeln!(
        &mut file,
        "0 0 {} {} rectfill",
        data.width,
        data.height
    )?;

    for path in &data.paths {
        let (hex, _) = color_with_alpha(path.color);
        let rgb = parse_hex(&hex);
        writeln!(&mut file, "{:.3} {:.3} {:.3} setrgbcolor", rgb.0, rgb.1, rgb.2)?;
        if path.stroke_only {
            writeln!(&mut file, "1 setlinewidth")?;
        }
        if let Some(ref svg) = path.svg_override {
            write_svg_override_eps(&mut file, svg, data.height)?;
        } else {
            curves_to_eps_path(&mut file, &path.curves, !path.stroke_only, data.height)?;
        }
        if path.stroke_only {
            writeln!(&mut file, "stroke")?;
        } else {
            writeln!(&mut file, "fill")?;
        }
    }

    writeln!(&mut file, "showpage")?;
    Ok(())
}

fn write_svg_override_eps<W: Write>(writer: &mut W, svg: &str, height: u32) -> Result<()> {
    for token in svg.split_whitespace() {
        if let Some(rest) = token.strip_prefix('M') {
            let parts: Vec<f64> = rest.split(',').filter_map(|v| v.parse().ok()).collect();
            if parts.len() >= 2 {
                writeln!(writer, "{:.2} {:.2} moveto", parts[0], flip_y(parts[1], height))?;
            }
        } else if let Some(rest) = token.strip_prefix('L') {
            let parts: Vec<f64> = rest.split(',').filter_map(|v| v.parse().ok()).collect();
            if parts.len() >= 2 {
                writeln!(writer, "{:.2} {:.2} lineto", parts[0], flip_y(parts[1], height))?;
            }
        } else if token == "Z" {
            writeln!(writer, "closepath")?;
        }
    }
    Ok(())
}

/// Minimal PDF wrapper (single page, path content from EPS-style ops).
pub fn write_enhanced_pdf(data: &EnhancedVectorData, output_path: &Path) -> Result<()> {
    let eps_path = output_path.with_extension("eps.tmp");
    write_enhanced_eps(data, &eps_path)?;
    let eps = std::fs::read_to_string(&eps_path)?;
    let _ = std::fs::remove_file(&eps_path);

    let mut file = File::create(output_path)?;
    writeln!(&mut file, "%PDF-1.4")?;
    writeln!(&mut file, "1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj")?;
    writeln!(
        &mut file,
        "2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj"
    )?;
    writeln!(
        &mut file,
        "3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents 4 0 R >> endobj",
        data.width, data.height
    )?;
    let stream = format!(
        "q\n{} W {} H scale\n{} Q\n",
        data.width,
        data.height,
        eps.lines().skip(5).collect::<Vec<_>>().join("\n")
    );
    writeln!(
        &mut file,
        "4 0 obj << /Length {} >> stream",
        stream.len()
    )?;
    write!(&mut file, "{}", stream)?;
    writeln!(&mut file, "endstream endobj")?;
    writeln!(&mut file, "xref")?;
    writeln!(&mut file, "trailer << /Size 5 /Root 1 0 R >>")?;
    writeln!(&mut file, "startxref")?;
    writeln!(&mut file, "%%EOF")?;
    Ok(())
}

/// Write enhanced data as Adobe Illustrator-compatible EPS (.ai).
pub fn write_enhanced_ai(data: &EnhancedVectorData, output_path: &Path) -> Result<()> {
    write_enhanced_eps(data, output_path)
}

fn parse_hex(hex: &str) -> (f64, f64, f64) {
    let h = hex.trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0) as f64 / 255.0;
        (r, g, b)
    } else {
        (0.0, 0.0, 0.0)
    }
}
