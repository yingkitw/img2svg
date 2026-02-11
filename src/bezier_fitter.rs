//! Least-squares cubic Bézier fitting with Newton-Raphson reparameterization.
//!
//! Produces smooth curves from boundary points, much better than line segments.
//! Ported from vec project's BezierFitter.

use crate::vectorizer::Point;

/// A cubic Bézier curve segment.
#[derive(Debug, Clone)]
pub struct BezierCurve {
    pub start: Point,
    pub control1: Point,
    pub control2: Point,
    pub end: Point,
}

/// Fit cubic Bézier curves to a sequence of points.
pub struct BezierFitter {
    tolerance: f64,
    max_iterations: usize,
}

impl BezierFitter {
    pub fn new(tolerance: f64) -> Self {
        Self {
            tolerance,
            max_iterations: 12,
        }
    }

    /// Fit a path (sequence of points) into a series of cubic Bézier curves.
    /// If `closed`, a closing segment is added if endpoints don't match.
    pub fn fit_path(&self, points: &[Point], closed: bool) -> Vec<BezierCurve> {
        if points.len() < 2 {
            return Vec::new();
        }
        if points.len() == 2 {
            return vec![self.linear_to_cubic(&points[0], &points[1])];
        }

        // Detect sharp corners (angle < 120°) and split the path there.
        // This prevents the fitter from curving through what should be sharp edges.
        let corner_indices = self.detect_sharp_corners(points);

        let mut curves = Vec::new();

        if corner_indices.is_empty() {
            // No sharp corners — fit as one piece
            self.fit_segment(points, &mut curves);
        } else {
            // Split at corners and fit each segment independently
            let mut splits: Vec<usize> = Vec::new();
            splits.push(0);
            for &ci in &corner_indices {
                if ci > 0 && ci < points.len() - 1 {
                    splits.push(ci);
                }
            }
            splits.push(points.len() - 1);
            splits.dedup();

            for i in 0..splits.len() - 1 {
                let start = splits[i];
                let end = splits[i + 1];
                if end <= start {
                    continue;
                }
                let segment = &points[start..=end];
                if segment.len() >= 2 {
                    self.fit_segment(segment, &mut curves);
                }
            }
        }

        // Enforce G1 continuity between adjacent curves (only for smooth joins)
        if curves.len() > 1 && corner_indices.is_empty() {
            self.enforce_g1_continuity(&mut curves);
        }

        // Close the path if needed
        if closed && !curves.is_empty() {
            let last_end = &curves.last().unwrap().end;
            let first_start = &curves[0].start;
            let dx = last_end.x - first_start.x;
            let dy = last_end.y - first_start.y;
            if (dx * dx + dy * dy).sqrt() > 0.5 {
                curves.push(self.linear_to_cubic(last_end, first_start));
            }
        }

        // Clamp control points to prevent overshoot
        if !curves.is_empty() && points.len() >= 2 {
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for p in points {
                min_x = min_x.min(p.x);
                min_y = min_y.min(p.y);
                max_x = max_x.max(p.x);
                max_y = max_y.max(p.y);
            }
            let margin = ((max_x - min_x).max(max_y - min_y) * 0.15).max(2.0);
            let lo_x = min_x - margin;
            let lo_y = min_y - margin;
            let hi_x = max_x + margin;
            let hi_y = max_y + margin;

            for curve in &mut curves {
                curve.control1.x = curve.control1.x.clamp(lo_x, hi_x);
                curve.control1.y = curve.control1.y.clamp(lo_y, hi_y);
                curve.control2.x = curve.control2.x.clamp(lo_x, hi_x);
                curve.control2.y = curve.control2.y.clamp(lo_y, hi_y);
            }
        }

        curves
    }

    /// Detect sharp corners (turn angle > 60°) in a point sequence.
    /// The angle measures the turn between consecutive edge vectors:
    /// 0° = straight, 90° = right angle, 180° = U-turn.
    /// Returns indices of corner points.
    fn detect_sharp_corners(&self, points: &[Point]) -> Vec<usize> {
        let n = points.len();
        if n < 3 {
            return Vec::new();
        }
        let threshold_rad = 30.0f64.to_radians(); // 30° turn = sharp corner (catches 45° chamfers from marching squares)
        let mut corners = Vec::new();

        for i in 1..n - 1 {
            let v1x = points[i].x - points[i - 1].x;
            let v1y = points[i].y - points[i - 1].y;
            let v2x = points[i + 1].x - points[i].x;
            let v2y = points[i + 1].y - points[i].y;
            let len1 = (v1x * v1x + v1y * v1y).sqrt();
            let len2 = (v2x * v2x + v2y * v2y).sqrt();
            if len1 < 1e-6 || len2 < 1e-6 {
                continue;
            }
            let cos_angle = ((v1x * v2x + v1y * v2y) / (len1 * len2)).clamp(-1.0, 1.0);
            let turn_angle = cos_angle.acos(); // 0=straight, π=U-turn
            if turn_angle > threshold_rad {
                corners.push(i);
            }
        }

        corners
    }

    /// Recursively fit cubic Bézier to a segment of points.
    fn fit_segment(&self, points: &[Point], curves: &mut Vec<BezierCurve>) {
        if points.len() < 2 {
            return;
        }
        if points.len() == 2 {
            curves.push(self.linear_to_cubic(&points[0], &points[1]));
            return;
        }

        // Check if points are nearly collinear — use linear Bézier
        if self.is_nearly_linear(points) {
            curves.push(self.linear_to_cubic(&points[0], &points[points.len() - 1]));
            return;
        }

        if points.len() == 3 {
            curves.push(self.fit_three_points(points));
            return;
        }

        const MAX_POINTS_PER_SEGMENT: usize = 40;
        if points.len() > MAX_POINTS_PER_SEGMENT {
            // Split at the point of maximum curvature (corner) instead of midpoint
            let split = self.find_best_split(points);
            self.fit_segment(&points[..=split], curves);
            self.fit_segment(&points[split..], curves);
            return;
        }

        // Initial chord-length parameterization
        let mut t_values = self.chord_length_parameterize(points);

        // Iterative fit with Newton-Raphson reparameterization
        let mut best_curve = self.least_squares_fit(points, &t_values);
        let (mut best_err, mut best_idx) = self.max_fitting_error(&best_curve, points);

        if best_err <= self.tolerance {
            curves.push(best_curve);
            return;
        }

        for _ in 0..self.max_iterations {
            let new_t = self.newton_raphson_reparameterize(&best_curve, points, &t_values);
            t_values = new_t;

            let new_curve = self.least_squares_fit(points, &t_values);
            let (new_err, new_idx) = self.max_fitting_error(&new_curve, points);

            if new_err < best_err {
                best_curve = new_curve;
                best_err = new_err;
                best_idx = new_idx;

                if best_err <= self.tolerance {
                    curves.push(best_curve);
                    return;
                }
            } else {
                break;
            }
        }

        if points.len() <= 3 {
            curves.push(best_curve);
        } else {
            let split = best_idx.max(2).min(points.len() - 2);
            self.fit_segment(&points[..=split], curves);
            self.fit_segment(&points[split..], curves);
        }
    }

    /// Check if a sequence of points is nearly collinear (max deviation < threshold).
    /// For long segments, uses a relative threshold (1% of segment length) to avoid
    /// fitting curves to what are essentially straight lines with tiny deviations.
    fn is_nearly_linear(&self, points: &[Point]) -> bool {
        if points.len() < 3 {
            return true;
        }
        let start = &points[0];
        let end = &points[points.len() - 1];
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let line_len = (dx * dx + dy * dy).sqrt();
        if line_len < 1e-6 {
            return true;
        }
        // Use the larger of: fixed tolerance, or 1% of segment length.
        // This prevents long near-vertical/horizontal lines from being curved
        // due to tiny pixel-level deviations after simplification.
        let threshold = (self.tolerance * 0.5).max(line_len * 0.01);
        for p in &points[1..points.len() - 1] {
            let dist = ((p.y - start.y) * dx - (p.x - start.x) * dy).abs() / line_len;
            if dist > threshold {
                return false;
            }
        }
        true
    }

    /// Find the best split point for a long segment — the point of maximum angle change.
    fn find_best_split(&self, points: &[Point]) -> usize {
        let n = points.len();
        let mut best_idx = n / 2;
        let mut best_angle_change = 0.0f64;

        for i in 2..n - 2 {
            let v1x = points[i].x - points[i - 2].x;
            let v1y = points[i].y - points[i - 2].y;
            let v2x = points[i + 2].x - points[i].x;
            let v2y = points[i + 2].y - points[i].y;
            let len1 = (v1x * v1x + v1y * v1y).sqrt();
            let len2 = (v2x * v2x + v2y * v2y).sqrt();
            if len1 > 0.0 && len2 > 0.0 {
                let cross = (v1x * v2y - v1y * v2x).abs() / (len1 * len2);
                if cross > best_angle_change {
                    best_angle_change = cross;
                    best_idx = i;
                }
            }
        }

        best_idx.max(2).min(n - 2)
    }

    /// Newton-Raphson reparameterization: find better t values by minimizing |B(t) - P|^2.
    fn newton_raphson_reparameterize(
        &self,
        curve: &BezierCurve,
        points: &[Point],
        t_values: &[f64],
    ) -> Vec<f64> {
        let mut new_t = t_values.to_vec();
        for i in 1..points.len() - 1 {
            let t = t_values[i];
            let p = &points[i];

            let bt = self.evaluate(curve, t);
            let bt_prime = self.evaluate_derivative(curve, t);
            let bt_double_prime = self.evaluate_second_derivative(curve, t);

            let dx = bt.x - p.x;
            let dy = bt.y - p.y;
            let numerator = dx * bt_prime.x + dy * bt_prime.y;
            let denominator = bt_prime.x * bt_prime.x
                + bt_prime.y * bt_prime.y
                + dx * bt_double_prime.x
                + dy * bt_double_prime.y;

            if denominator.abs() > 1e-12 {
                new_t[i] = (t - numerator / denominator).clamp(0.0, 1.0);
            }
        }
        // Ensure monotonicity
        for i in 1..new_t.len() {
            if new_t[i] <= new_t[i - 1] {
                new_t[i] = new_t[i - 1] + 1e-10;
            }
        }
        new_t[0] = 0.0;
        *new_t.last_mut().unwrap() = 1.0;
        new_t
    }

    /// Least-squares cubic Bézier fit with given parameterization.
    fn least_squares_fit(&self, points: &[Point], t_values: &[f64]) -> BezierCurve {
        let n = points.len();
        let start = points[0].clone();
        let end = points[n - 1].clone();

        let mut a11 = 0.0;
        let mut a12 = 0.0;
        let mut a22 = 0.0;
        let mut bx1 = 0.0;
        let mut by1 = 0.0;
        let mut bx2 = 0.0;
        let mut by2 = 0.0;

        for i in 0..n {
            let t = t_values[i];
            let mt = 1.0 - t;
            let b1 = 3.0 * mt * mt * t;
            let b2 = 3.0 * mt * t * t;
            let b0 = mt * mt * mt;
            let b3 = t * t * t;

            a11 += b1 * b1;
            a12 += b1 * b2;
            a22 += b2 * b2;

            let rx = points[i].x - b0 * start.x - b3 * end.x;
            let ry = points[i].y - b0 * start.y - b3 * end.y;

            bx1 += b1 * rx;
            by1 += b1 * ry;
            bx2 += b2 * rx;
            by2 += b2 * ry;
        }

        let det = a11 * a22 - a12 * a12;

        let (control1, control2) = if det.abs() < 1e-12 {
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            (
                Point { x: start.x + dx / 3.0, y: start.y + dy / 3.0 },
                Point { x: start.x + 2.0 * dx / 3.0, y: start.y + 2.0 * dy / 3.0 },
            )
        } else {
            let inv_det = 1.0 / det;
            (
                Point {
                    x: (a22 * bx1 - a12 * bx2) * inv_det,
                    y: (a22 * by1 - a12 * by2) * inv_det,
                },
                Point {
                    x: (a11 * bx2 - a12 * bx1) * inv_det,
                    y: (a11 * by2 - a12 * by1) * inv_det,
                },
            )
        };

        BezierCurve { start, control1, control2, end }
    }

    fn chord_length_parameterize(&self, points: &[Point]) -> Vec<f64> {
        let n = points.len();
        let mut t = vec![0.0; n];
        for i in 1..n {
            let dx = points[i].x - points[i - 1].x;
            let dy = points[i].y - points[i - 1].y;
            t[i] = t[i - 1] + (dx * dx + dy * dy).sqrt();
        }
        let total = t[n - 1];
        if total > 0.0 {
            for ti in t.iter_mut() {
                *ti /= total;
            }
        }
        t[n - 1] = 1.0;
        t
    }

    fn fit_three_points(&self, points: &[Point]) -> BezierCurve {
        let p0 = &points[0];
        let p1 = &points[1];
        let p2 = &points[2];
        BezierCurve {
            start: p0.clone(),
            control1: Point {
                x: p0.x + 2.0 / 3.0 * (p1.x - p0.x),
                y: p0.y + 2.0 / 3.0 * (p1.y - p0.y),
            },
            control2: Point {
                x: p2.x + 2.0 / 3.0 * (p1.x - p2.x),
                y: p2.y + 2.0 / 3.0 * (p1.y - p2.y),
            },
            end: p2.clone(),
        }
    }

    fn linear_to_cubic(&self, start: &Point, end: &Point) -> BezierCurve {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        BezierCurve {
            start: start.clone(),
            control1: Point { x: start.x + dx / 3.0, y: start.y + dy / 3.0 },
            control2: Point { x: start.x + 2.0 * dx / 3.0, y: start.y + 2.0 * dy / 3.0 },
            end: end.clone(),
        }
    }

    fn max_fitting_error(&self, curve: &BezierCurve, points: &[Point]) -> (f64, usize) {
        let t_values = self.chord_length_parameterize(points);
        let mut max_err = 0.0;
        let mut max_idx = 0;
        for i in 1..points.len() - 1 {
            let curve_pt = self.evaluate(curve, t_values[i]);
            let dx = points[i].x - curve_pt.x;
            let dy = points[i].y - curve_pt.y;
            let err = (dx * dx + dy * dy).sqrt();
            if err > max_err {
                max_err = err;
                max_idx = i;
            }
        }
        (max_err, max_idx)
    }

    fn evaluate(&self, curve: &BezierCurve, t: f64) -> Point {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        Point {
            x: mt3 * curve.start.x
                + 3.0 * mt2 * t * curve.control1.x
                + 3.0 * mt * t2 * curve.control2.x
                + t3 * curve.end.x,
            y: mt3 * curve.start.y
                + 3.0 * mt2 * t * curve.control1.y
                + 3.0 * mt * t2 * curve.control2.y
                + t3 * curve.end.y,
        }
    }

    fn evaluate_derivative(&self, curve: &BezierCurve, t: f64) -> Point {
        let mt = 1.0 - t;
        let a_x = curve.control1.x - curve.start.x;
        let a_y = curve.control1.y - curve.start.y;
        let b_x = curve.control2.x - curve.control1.x;
        let b_y = curve.control2.y - curve.control1.y;
        let c_x = curve.end.x - curve.control2.x;
        let c_y = curve.end.y - curve.control2.y;

        Point {
            x: 3.0 * mt * mt * a_x + 6.0 * mt * t * b_x + 3.0 * t * t * c_x,
            y: 3.0 * mt * mt * a_y + 6.0 * mt * t * b_y + 3.0 * t * t * c_y,
        }
    }

    fn evaluate_second_derivative(&self, curve: &BezierCurve, t: f64) -> Point {
        let mt = 1.0 - t;
        let a_x = curve.control2.x - 2.0 * curve.control1.x + curve.start.x;
        let a_y = curve.control2.y - 2.0 * curve.control1.y + curve.start.y;
        let b_x = curve.end.x - 2.0 * curve.control2.x + curve.control1.x;
        let b_y = curve.end.y - 2.0 * curve.control2.y + curve.control1.y;

        Point {
            x: 6.0 * mt * a_x + 6.0 * t * b_x,
            y: 6.0 * mt * a_y + 6.0 * t * b_y,
        }
    }

    /// Enforce G1 continuity between adjacent curves.
    fn enforce_g1_continuity(&self, curves: &mut [BezierCurve]) {
        for i in 0..curves.len().saturating_sub(1) {
            let current_control2 = curves[i].control2.clone();
            let current_end = curves[i].end.clone();
            let next = &mut curves[i + 1];

            let t1x = current_end.x - current_control2.x;
            let t1y = current_end.y - current_control2.y;
            let t2x = next.control1.x - next.start.x;
            let t2y = next.control1.y - next.start.y;

            let len1 = (t1x * t1x + t1y * t1y).sqrt();
            let len2 = (t2x * t2x + t2y * t2y).sqrt();

            if len1 > 1e-10 && len2 > 1e-10 {
                let scale = len2 / len1;
                next.control1.x = next.start.x + t1x * scale;
                next.control1.y = next.start.y + t1y * scale;
            }
        }
    }
}

/// Format Bézier curves as SVG path data.
/// Uses `L` for near-linear curves and `C` for true curves to minimize SVG size.
/// Merges consecutive collinear `L` segments into a single `L`.
pub fn bezier_to_svg_path(curves: &[BezierCurve], closed: bool) -> String {
    if curves.is_empty() {
        return String::new();
    }

    let mut path = format!("M{},{}", fmt_num(curves[0].start.x), fmt_num(curves[0].start.y));

    let mut i = 0;
    while i < curves.len() {
        let curve = &curves[i];
        if is_linear_curve(curve) {
            // Merge consecutive collinear L segments using distance-based check.
            // This catches diagonal staircases from marching squares where
            // cross-product fails due to pixel-grid stepping.
            let start = &curves[i].start;
            let mut end = &curve.end;
            let mut j = i + 1;
            while j < curves.len() {
                let next = &curves[j];
                if !is_linear_curve(next) {
                    break;
                }
                // Check if ALL intermediate points lie within 1.5px of the
                // line from start to next.end (distance-based collinear test)
                let candidate_end = &next.end;
                let dx = candidate_end.x - start.x;
                let dy = candidate_end.y - start.y;
                let line_len = (dx * dx + dy * dy).sqrt();
                if line_len < 0.5 {
                    end = candidate_end;
                    j += 1;
                    continue;
                }
                // Check current end point distance to the proposed line
                let dist = ((end.y - start.y) * dx - (end.x - start.x) * dy).abs() / line_len;
                if dist < 1.5 {
                    end = candidate_end;
                    j += 1;
                } else {
                    break;
                }
            }
            path.push_str(&format!("L{},{}", fmt_num(end.x), fmt_num(end.y)));
            i = j;
        } else {
            path.push_str(&format!(
                "C{},{} {},{} {},{}",
                fmt_num(curve.control1.x), fmt_num(curve.control1.y),
                fmt_num(curve.control2.x), fmt_num(curve.control2.y),
                fmt_num(curve.end.x), fmt_num(curve.end.y),
            ));
            i += 1;
        }
    }

    if closed {
        path.push('Z');
    }

    path
}

/// Check if a cubic Bézier is effectively a straight line
/// (control points lie close to the start-end line).
fn is_linear_curve(curve: &BezierCurve) -> bool {
    let dx = curve.end.x - curve.start.x;
    let dy = curve.end.y - curve.start.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return true;
    }
    let d1 = ((curve.control1.y - curve.start.y) * dx - (curve.control1.x - curve.start.x) * dy).abs() / len;
    let d2 = ((curve.control2.y - curve.start.y) * dx - (curve.control2.x - curve.start.x) * dy).abs() / len;
    d1 < 1.0 && d2 < 1.0
}

/// Format a float compactly: integer if close to whole, else 2 decimal places trimmed.
fn fmt_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-4 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{:.2}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_to_cubic() {
        let fitter = BezierFitter::new(2.0);
        let start = Point { x: 0.0, y: 0.0 };
        let end = Point { x: 10.0, y: 10.0 };
        let curve = fitter.linear_to_cubic(&start, &end);
        assert_eq!(curve.start.x, 0.0);
        assert_eq!(curve.end.x, 10.0);
    }

    #[test]
    fn test_fit_path_two_points() {
        let fitter = BezierFitter::new(2.0);
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
        ];
        let curves = fitter.fit_path(&points, false);
        assert_eq!(curves.len(), 1);
    }

    #[test]
    fn test_fit_path_semicircle() {
        let fitter = BezierFitter::new(2.0);
        let points: Vec<Point> = (0..=20)
            .map(|i| {
                let t = i as f64 / 20.0 * std::f64::consts::PI;
                Point { x: t.cos() * 50.0 + 50.0, y: t.sin() * 50.0 }
            })
            .collect();
        let curves = fitter.fit_path(&points, false);
        assert!(!curves.is_empty());
        assert!(curves.len() <= 10);
    }

    #[test]
    fn test_fit_path_closed() {
        let fitter = BezierFitter::new(2.0);
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 0.0, y: 10.0 },
        ];
        let curves = fitter.fit_path(&points, true);
        assert!(!curves.is_empty());
    }

    #[test]
    fn test_newton_raphson_monotonic() {
        let fitter = BezierFitter::new(1.0);
        let curve = BezierCurve {
            start: Point { x: 0.0, y: 0.0 },
            control1: Point { x: 5.0, y: 10.0 },
            control2: Point { x: 10.0, y: 10.0 },
            end: Point { x: 15.0, y: 0.0 },
        };
        let points: Vec<Point> = (0..=10)
            .map(|i| fitter.evaluate(&curve, i as f64 / 10.0))
            .collect();
        let t_initial = fitter.chord_length_parameterize(&points);
        let t_refined = fitter.newton_raphson_reparameterize(&curve, &points, &t_initial);
        for i in 1..t_refined.len() {
            assert!(t_refined[i] >= t_refined[i - 1]);
        }
    }

    #[test]
    fn test_bezier_to_svg_path() {
        let curves = vec![BezierCurve {
            start: Point { x: 0.0, y: 0.0 },
            control1: Point { x: 5.0, y: 10.0 },
            control2: Point { x: 10.0, y: 10.0 },
            end: Point { x: 15.0, y: 0.0 },
        }];
        let path = bezier_to_svg_path(&curves, true);
        assert!(path.starts_with("M0,0"));
        assert!(path.contains("C5,10"));
        assert!(path.ends_with('Z'));
    }

    #[test]
    fn test_fmt_num_integer() {
        assert_eq!(fmt_num(5.0), "5");
        assert_eq!(fmt_num(5.0001), "5");
    }

    #[test]
    fn test_fmt_num_decimal() {
        assert_eq!(fmt_num(5.25), "5.25");
        assert_eq!(fmt_num(5.10), "5.1");
    }

    #[test]
    fn test_control_point_clamping() {
        let fitter = BezierFitter::new(2.0);
        // Points in a small region — control points should be clamped
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 5.0 },
            Point { x: 2.0, y: 0.0 },
            Point { x: 3.0, y: 5.0 },
            Point { x: 4.0, y: 0.0 },
        ];
        let curves = fitter.fit_path(&points, false);
        for curve in &curves {
            // Control points should be within reasonable bounds
            assert!(curve.control1.x >= -2.0 && curve.control1.x <= 6.0);
            assert!(curve.control1.y >= -2.0 && curve.control1.y <= 7.0);
        }
    }
}
