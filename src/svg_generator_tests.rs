#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::vectorizer::{Point, Curve, VectorizedData};
    use std::fs;
    use std::path::PathBuf;

    fn create_test_vectorized_data() -> VectorizedData {
        VectorizedData {
            width: 100,
            height: 100,
            background_color: (255, 255, 255, 255),
            curves: vec![
                Curve {
                    points: vec![
                        Point { x: 10.0, y: 10.0 },
                        Point { x: 90.0, y: 10.0 },
                        Point { x: 90.0, y: 90.0 },
                        Point { x: 10.0, y: 90.0 },
                    ],
                    color: (255, 0, 0, 255),
                    is_closed: true,
                    subpaths: vec![],
                },
                Curve {
                    points: vec![
                        Point { x: 30.0, y: 30.0 },
                        Point { x: 70.0, y: 30.0 },
                        Point { x: 70.0, y: 70.0 },
                        Point { x: 30.0, y: 70.0 },
                    ],
                    color: (0, 0, 255, 255),
                    is_closed: true,
                    subpaths: vec![],
                },
            ],
        }
    }

    fn create_test_vectorized_data_with_subpaths() -> VectorizedData {
        VectorizedData {
            width: 100,
            height: 100,
            background_color: (255, 255, 255, 255),
            curves: vec![
                Curve {
                    points: vec![],
                    color: (255, 0, 0, 255),
                    is_closed: true,
                    subpaths: vec![
                        vec![
                            Point { x: 10.0, y: 10.0 },
                            Point { x: 40.0, y: 10.0 },
                            Point { x: 40.0, y: 40.0 },
                            Point { x: 10.0, y: 40.0 },
                        ],
                        vec![
                            Point { x: 60.0, y: 60.0 },
                            Point { x: 90.0, y: 60.0 },
                            Point { x: 90.0, y: 90.0 },
                            Point { x: 60.0, y: 90.0 },
                        ],
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_generate_svg_basic() {
        let data = create_test_vectorized_data();
        let output_path = PathBuf::from("/tmp/test_output.svg");

        let result = generate_svg(&data, &output_path);
        assert!(result.is_ok());

        // Check that file was created
        assert!(output_path.exists());

        // Read and verify content
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("width=\"100\""));
        assert!(content.contains("height=\"100\""));
        assert!(content.contains("</svg>"));

        // Clean up
        let _ = fs::remove_file(&output_path);
    }

    #[test]
    fn test_generate_svg_with_background_color() {
        let data = create_test_vectorized_data();
        let output_path = PathBuf::from("/tmp/test_background.svg");

        let result = generate_svg(&data, &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        // Check for background rect with white color
        assert!(content.contains("#ffffff"));

        let _ = fs::remove_file(&output_path);
    }

    #[test]
    fn test_generate_svg_with_curves() {
        let data = create_test_vectorized_data();
        let output_path = PathBuf::from("/tmp/test_curves.svg");

        let result = generate_svg(&data, &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        // Check for curve colors
        assert!(content.contains("#ff0000")); // Red
        assert!(content.contains("#0000ff")); // Blue
        // Check for path elements
        assert!(content.contains("<path"));

        let _ = fs::remove_file(&output_path);
    }

    #[test]
    fn test_generate_svg_advanced() {
        let data = create_test_vectorized_data();
        let output_path = PathBuf::from("/tmp/test_advanced.svg");

        let result = generate_svg_advanced(&data, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());

        let _ = fs::remove_file(&output_path);
    }

    #[test]
    fn test_generate_svg_with_subpaths() {
        let data = create_test_vectorized_data_with_subpaths();
        let output_path = PathBuf::from("/tmp/test_subpaths.svg");

        let result = generate_svg(&data, &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        // Should have path elements
        assert!(content.contains("<path"));

        let _ = fs::remove_file(&output_path);
    }

    #[test]
    fn test_generate_svg_empty_curves() {
        let data = VectorizedData {
            width: 50,
            height: 50,
            background_color: (128, 128, 128, 255),
            curves: vec![],
        };
        let output_path = PathBuf::from("/tmp/test_empty.svg");

        let result = generate_svg(&data, &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<svg"));
        assert!(content.contains("#808080")); // Gray background

        let _ = fs::remove_file(&output_path);
    }

    // === fmt_coord Tests ===

    #[test]
    fn test_fmt_coord_integer() {
        assert_eq!(fmt_coord(5.0), "5");
        assert_eq!(fmt_coord(10.0), "10");
        assert_eq!(fmt_coord(0.0), "0");
    }

    #[test]
    fn test_fmt_coord_half() {
        assert_eq!(fmt_coord(5.5), "5.5");
        assert_eq!(fmt_coord(10.5), "10.5");
    }

    #[test]
    fn test_fmt_coord_rounding() {
        // Should round to nearest 0.5, then format as integer if close to whole
        assert_eq!(fmt_coord(5.24), "5");    // Rounds to 5.0, formats as "5"
        assert_eq!(fmt_coord(5.26), "5.5");  // Rounds to 5.5
        assert_eq!(fmt_coord(5.74), "5.5");  // Rounds to 5.5
        assert_eq!(fmt_coord(5.76), "6");    // Rounds to 6.0, formats as "6"
    }

    #[test]
    fn test_fmt_coord_negative() {
        assert_eq!(fmt_coord(-5.0), "-5");
        assert_eq!(fmt_coord(-5.5), "-5.5");
    }

    // === create_subpath_string Tests ===

    #[test]
    fn test_create_subpath_string_empty() {
        let points: Vec<Point> = vec![];
        let result = create_subpath_string(&points, true);
        assert_eq!(result, "");
    }

    #[test]
    fn test_create_subpath_string_single_point() {
        let points = vec![Point { x: 10.0, y: 20.0 }];
        let result = create_subpath_string(&points, false);
        assert_eq!(result, "M10 20");
    }

    #[test]
    fn test_create_subpath_string_two_points() {
        let points = vec![
            Point { x: 10.0, y: 20.0 },
            Point { x: 30.0, y: 40.0 },
        ];
        let result = create_subpath_string(&points, false);
        assert_eq!(result, "M10 20L30 40");
    }

    #[test]
    fn test_create_subpath_string_closed() {
        let points = vec![
            Point { x: 10.0, y: 10.0 },
            Point { x: 20.0, y: 10.0 },
            Point { x: 20.0, y: 20.0 },
        ];
        let result = create_subpath_string(&points, true);
        assert!(result.contains("M10 10"));
        assert!(result.contains("L20 10"));
        assert!(result.contains("L20 20"));
        assert!(result.ends_with('Z'));
    }

    #[test]
    fn test_create_subpath_string_not_closed() {
        let points = vec![
            Point { x: 10.0, y: 10.0 },
            Point { x: 20.0, y: 10.0 },
            Point { x: 20.0, y: 20.0 },
        ];
        let result = create_subpath_string(&points, false);
        assert!(result.contains("M10 10"));
        assert!(result.contains("L20 10"));
        assert!(result.contains("L20 20"));
        assert!(!result.ends_with('Z'));
    }

    #[test]
    fn test_create_subpath_string_with_decimals() {
        let points = vec![
            Point { x: 10.5, y: 20.5 },
            Point { x: 30.25, y: 40.75 },
        ];
        let result = create_subpath_string(&points, false);
        assert_eq!(result, "M10.5 20.5L30.5 41"); // Note the rounding
    }

    // === create_multi_path_string Tests ===

    #[test]
    fn test_create_multi_path_string_empty() {
        let subpaths: Vec<Vec<Point>> = vec![];
        let result = create_multi_path_string(&subpaths);
        assert_eq!(result, "");
    }

    #[test]
    fn test_create_multi_path_string_single_subpath() {
        let subpaths = vec![
            vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: 10.0, y: 0.0 },
                Point { x: 10.0, y: 10.0 },
            ],
        ];
        let result = create_multi_path_string(&subpaths);
        assert!(!result.is_empty());
        assert!(result.contains("M0 0"));
    }

    #[test]
    fn test_create_multi_path_string_multiple_subpaths() {
        let subpaths = vec![
            vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: 10.0, y: 0.0 },
                Point { x: 10.0, y: 10.0 },
            ],
            vec![
                Point { x: 20.0, y: 20.0 },
                Point { x: 30.0, y: 20.0 },
                Point { x: 30.0, y: 30.0 },
            ],
        ];
        let result = create_multi_path_string(&subpaths);
        // Should have space-separated subpaths
        assert!(result.contains("M0 0"));
        assert!(result.contains("M20 20"));
    }

    #[test]
    fn test_create_multi_path_string_skips_small_subpaths() {
        let subpaths = vec![
            vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: 10.0, y: 0.0 },
            ], // Only 2 points, should be skipped
            vec![
                Point { x: 20.0, y: 20.0 },
                Point { x: 30.0, y: 20.0 },
                Point { x: 30.0, y: 30.0 },
            ], // 3 points, should be included
        ];
        let result = create_multi_path_string(&subpaths);
        // First subpath should be skipped (less than 3 points)
        assert!(!result.contains("M0 0"));
        // Second subpath should be included
        assert!(result.contains("M20 20"));
    }
}
