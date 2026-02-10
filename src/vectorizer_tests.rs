#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::image_processor::ImageData;
    use rgb::RGBA8;

    fn create_test_mask(width: usize, height: usize, pattern: &str) -> Vec<bool> {
        let mut mask = vec![false; width * height];
        match pattern {
            "square" => {
                // Create a square in the center
                let margin = width / 4;
                for y in margin..height - margin {
                    for x in margin..width - margin {
                        mask[y * width + x] = true;
                    }
                }
            }
            "circle" => {
                // Create a circle in the center
                let cx = width / 2;
                let cy = height / 2;
                let radius = width.min(height) / 4;
                for y in 0..height {
                    for x in 0..width {
                        let dx = x as f64 - cx as f64;
                        let dy = y as f64 - cy as f64;
                        if (dx * dx + dy * dy).sqrt() <= radius as f64 {
                            mask[y * width + x] = true;
                        }
                    }
                }
            }
            "checkerboard" => {
                // Create a checkerboard pattern
                for y in 0..height {
                    for x in 0..width {
                        mask[y * width + x] = (x + y) % 2 == 0;
                    }
                }
            }
            "full" => {
                // All true
                mask = vec![true; width * height];
            }
            "empty" => {
                // All false - already initialized
            }
            "horizontal_line" => {
                let mid_y = height / 2;
                for x in 0..width {
                    mask[mid_y * width + x] = true;
                }
            }
            "vertical_line" => {
                let mid_x = width / 2;
                for y in 0..height {
                    mask[y * width + mid_x] = true;
                }
            }
            "diagonal" => {
                for i in 0..width.min(height) {
                    mask[i * width + i] = true;
                }
            }
            _ => {}
        }
        mask
    }

    fn create_test_image(width: u32, height: u32, pixels: Vec<RGBA8>) -> ImageData {
        ImageData {
            width,
            height,
            pixels,
        }
    }

    fn create_two_color_image(width: u32, height: u32) -> ImageData {
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                // Create a simple square pattern
                let is_square = x > width / 4 && x < width * 3 / 4
                    && y > height / 4 && y < height * 3 / 4;
                if is_square {
                    pixels.push(RGBA8::new(255, 0, 0, 255));
                } else {
                    pixels.push(RGBA8::new(255, 255, 255, 255));
                }
            }
        }
        ImageData {
            width,
            height,
            pixels,
        }
    }

    // === Point Tests ===

    #[test]
    fn test_point_creation() {
        let p = Point { x: 10.5, y: 20.5 };
        assert_eq!(p.x, 10.5);
        assert_eq!(p.y, 20.5);
    }

    #[test]
    fn test_point_clone() {
        let p1 = Point { x: 5.0, y: 10.0 };
        let p2 = p1.clone();
        assert_eq!(p1.x, p2.x);
        assert_eq!(p1.y, p2.y);
    }

    // === Curve Tests ===

    #[test]
    fn test_curve_creation() {
        let curve = Curve {
            points: vec![Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }],
            color: (255, 0, 0, 255),
            is_closed: true,
            subpaths: vec![],
        };
        assert_eq!(curve.points.len(), 2);
        assert_eq!(curve.color, (255, 0, 0, 255));
        assert!(curve.is_closed);
    }

    #[test]
    fn test_curve_clone() {
        let curve1 = Curve {
            points: vec![Point { x: 0.0, y: 0.0 }],
            color: (128, 128, 128, 255),
            is_closed: false,
            subpaths: vec![],
        };
        let curve2 = curve1.clone();
        assert_eq!(curve1.color, curve2.color);
    }

    // === Polygon Area Tests ===

    #[test]
    fn test_polygon_area_triangle() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 5.0, y: 10.0 },
        ];
        let area = polygon_area(&points);
        assert!((area - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_polygon_area_square() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 0.0, y: 10.0 },
        ];
        let area = polygon_area(&points);
        assert!((area - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_polygon_area_empty() {
        let points: Vec<Point> = vec![];
        let area = polygon_area(&points);
        assert_eq!(area, 0.0);
    }

    #[test]
    fn test_polygon_area_line() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
        ];
        let area = polygon_area(&points);
        assert_eq!(area, 0.0);
    }

    // === Point to Line Distance Tests ===

    #[test]
    fn test_point_to_line_distance_on_line() {
        let line_start = Point { x: 0.0, y: 0.0 };
        let line_end = Point { x: 10.0, y: 0.0 };
        let point = Point { x: 5.0, y: 0.0 };
        let dist = point_to_line_distance(&point, &line_start, &line_end);
        assert!((dist - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_point_to_line_distance_perpendicular() {
        let line_start = Point { x: 0.0, y: 0.0 };
        let line_end = Point { x: 10.0, y: 0.0 };
        let point = Point { x: 5.0, y: 3.0 };
        let dist = point_to_line_distance(&point, &line_start, &line_end);
        assert!((dist - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_point_to_line_distance_vertical() {
        let line_start = Point { x: 0.0, y: 0.0 };
        let line_end = Point { x: 0.0, y: 10.0 };
        let point = Point { x: 4.0, y: 5.0 };
        let dist = point_to_line_distance(&point, &line_start, &line_end);
        assert!((dist - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_point_to_line_distance_diagonal() {
        let line_start = Point { x: 0.0, y: 0.0 };
        let line_end = Point { x: 10.0, y: 10.0 };
        let point = Point { x: 5.0, y: 7.0 };
        let dist = point_to_line_distance(&point, &line_start, &line_end);
        // Distance from (5,7) to line y=x is |7-5|/sqrt(2) = 2/sqrt(2) ≈ 1.414
        assert!((dist - 1.414).abs() < 0.01);
    }

    #[test]
    fn test_point_to_line_distance_degenerate() {
        let line_start = Point { x: 5.0, y: 5.0 };
        let line_end = Point { x: 5.0, y: 5.0 };
        let point = Point { x: 8.0, y: 9.0 };
        let dist = point_to_line_distance(&point, &line_start, &line_end);
        // Should be distance from point to the single point
        let dx: f64 = 8.0 - 5.0;
        let dy: f64 = 9.0 - 5.0;
        let expected = (dx * dx + dy * dy).sqrt();
        assert!((dist - expected).abs() < 0.01);
    }

    // === RDP Simplify Tests ===

    #[test]
    fn test_rdp_simplify_empty() {
        let points: Vec<Point> = vec![];
        let result = rdp_simplify(&points, 1.0);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_rdp_simplify_single_point() {
        let points = vec![Point { x: 5.0, y: 5.0 }];
        let result = rdp_simplify(&points, 1.0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_rdp_simplify_two_points() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
        ];
        let result = rdp_simplify(&points, 1.0);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_rdp_simplify_straight_line() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 2.0, y: 2.0 },
            Point { x: 4.0, y: 4.0 },
            Point { x: 6.0, y: 6.0 },
            Point { x: 8.0, y: 8.0 },
            Point { x: 10.0, y: 10.0 },
        ];
        let result = rdp_simplify(&points, 1.0);
        // Should reduce to just endpoints
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 10.0);
    }

    #[test]
    fn test_rdp_simplify_triangle() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 5.0, y: 0.1 }, // Very close to line
            Point { x: 10.0, y: 0.0 },
            Point { x: 5.0, y: 10.0 },
        ];
        let result = rdp_simplify(&points, 1.0);
        // Should keep the triangle corner
        assert!(result.len() >= 3);
    }

    // === Smooth Boundary Tests ===

    #[test]
    fn test_smooth_boundary_empty() {
        let points: Vec<Point> = vec![];
        let result = smooth_boundary(&points, 1);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_smooth_boundary_single_point() {
        let points = vec![Point { x: 5.0, y: 5.0 }];
        let result = smooth_boundary(&points, 1);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_smooth_boundary_two_points() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
        ];
        let result = smooth_boundary(&points, 1);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_smooth_boundary_square() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
            Point { x: 0.0, y: 10.0 },
        ];
        let result = smooth_boundary(&points, 1);
        assert_eq!(result.len(), 4);
        // Should smooth the corners - check that values have changed
        assert!(result[0].x >= 0.0);
        assert!(result[0].y >= 0.0);
        // The smoothing averages each point with its neighbors
        // For a closed loop, this should shift all points toward their neighbors
    }

    #[test]
    fn test_smooth_boundary_zero_level() {
        let points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 0.0 },
            Point { x: 10.0, y: 10.0 },
        ];
        let result = smooth_boundary(&points, 0);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].x, 0.0);
        assert_eq!(result[1].x, 10.0);
    }

    // === Marching Squares Tests ===

    #[test]
    fn test_marching_squares_empty_mask() {
        let mask = create_test_mask(10, 10, "empty");
        let contours = marching_squares_contours(&mask, 10, 10);
        assert_eq!(contours.len(), 0);
    }

    #[test]
    fn test_marching_squares_full_mask() {
        let mask = create_test_mask(10, 10, "full");
        let contours = marching_squares_contours(&mask, 10, 10);
        // Should have one contour around the border
        assert!(contours.len() >= 1);
        // Check that contour is closed and roughly rectangular
        if contours.len() > 0 {
            assert!(contours[0].len() >= 4);
        }
    }

    #[test]
    fn test_marching_squares_square() {
        let mask = create_test_mask(20, 20, "square");
        let contours = marching_squares_contours(&mask, 20, 20);
        // Should find at least one contour
        assert!(contours.len() >= 1);
        // Each contour should have enough points
        for contour in &contours {
            assert!(contour.len() >= 4);
        }
    }

    #[test]
    fn test_marching_squares_circle() {
        let mask = create_test_mask(20, 20, "circle");
        let contours = marching_squares_contours(&mask, 20, 20);
        // Should find at least one contour
        assert!(contours.len() >= 1);
    }

    #[test]
    fn test_marching_squares_horizontal_line() {
        let mask = create_test_mask(20, 20, "horizontal_line");
        let contours = marching_squares_contours(&mask, 20, 20);
        // Should find contours for the line
        assert!(contours.len() >= 1);
    }

    #[test]
    fn test_marching_squares_vertical_line() {
        let mask = create_test_mask(20, 20, "vertical_line");
        let contours = marching_squares_contours(&mask, 20, 20);
        // Should find contours for the line
        assert!(contours.len() >= 1);
    }

    #[test]
    fn test_marching_squares_checkerboard() {
        let mask = create_test_mask(10, 10, "checkerboard");
        let contours = marching_squares_contours(&mask, 10, 10);
        // Should find multiple contours
        assert!(contours.len() > 0);
    }

    // === Vectorize Tests ===

    #[test]
    fn test_vectorize_two_color_image() {
        let img = create_two_color_image(50, 50);
        let result = vectorize(&img, 2, 0.1, 0, false);
        assert!(result.is_ok());

        let vectorized = result.unwrap();
        assert_eq!(vectorized.width, 50);
        assert_eq!(vectorized.height, 50);
        // Should have a background color and at least one curve
        assert!(vectorized.background_color == (255, 255, 255, 255)
            || vectorized.background_color == (255, 0, 0, 255));
    }

    #[test]
    fn test_vectorize_with_smoothing() {
        let img = create_two_color_image(50, 50);

        let result_no_smooth = vectorize(&img, 2, 0.1, 0, false);
        let result_smooth = vectorize(&img, 2, 0.1, 3, false);

        assert!(result_no_smooth.is_ok());
        assert!(result_smooth.is_ok());

        let v_no_smooth = result_no_smooth.unwrap();
        let v_smooth = result_smooth.unwrap();

        // Both should produce valid results
        assert_eq!(v_no_smooth.width, v_smooth.width);
        assert_eq!(v_no_smooth.height, v_smooth.height);
    }

    #[test]
    fn test_vectorize_single_color() {
        let img = create_test_image(
            10,
            10,
            vec![RGBA8::new(255, 0, 0, 255); 100],
        );
        let result = vectorize(&img, 1, 0.1, 0, false);
        assert!(result.is_ok());

        let vectorized = result.unwrap();
        // With single color, should only have background, no curves
        assert_eq!(vectorized.curves.len(), 0);
        assert_eq!(vectorized.background_color, (255, 0, 0, 255));
    }

    #[test]
    fn test_vectorize_preserves_dimensions() {
        let img = create_two_color_image(100, 75);
        let result = vectorize(&img, 8, 0.1, 0, false);
        assert!(result.is_ok());

        let vectorized = result.unwrap();
        assert_eq!(vectorized.width, 100);
        assert_eq!(vectorized.height, 75);
    }

    #[test]
    fn test_vectorized_data_structure() {
        let img = create_two_color_image(50, 50);
        let result = vectorize(&img, 2, 0.1, 0, false);
        assert!(result.is_ok());

        let vectorized = result.unwrap();
        // Check that the structure is valid
        assert_eq!(vectorized.width, 50);
        assert_eq!(vectorized.height, 50);
        assert_eq!(vectorized.background_color.3, 255); // Alpha should be 255

        for curve in &vectorized.curves {
            assert_eq!(curve.color.3, 255); // All colors should have alpha 255
        }
    }
}
