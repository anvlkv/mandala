use crate::{Angle, Float, Point, Vector, VectorValuedFn};

use super::LineSegment;

pub type PathSegment = Box<dyn VectorValuedFn>;

/// Continus path constructed of multiple segments
#[derive(Default)]
pub struct Path {
    segments: Vec<PathSegment>,
    lengths: Vec<Float>,
}

impl Path {
    pub fn new(segments: Vec<PathSegment>) -> Self {
        let lengths = segments.iter().map(|s| s.length()).collect();

        Self { segments, lengths }
    }

    pub fn push(&mut self, segment: PathSegment) {
        self.lengths.push(segment.length());
        self.segments.push(segment);
    }

    /// draws a poligon
    pub fn polygon(center: Point, size: Vector, n_sides: usize, start_angle: Angle) -> Self {
        let mut segments = Vec::new();
        let angle_increment = Angle::TAU / n_sides as Float;
        let mut current_angle = start_angle;
        let mut previous_point = Point {
            x: center.x + size.x * current_angle.cos(),
            y: center.y + size.y * current_angle.sin(),
            #[cfg(feature = "3d")]
            z: center.z,
        };
        current_angle += angle_increment;

        for _ in 1..n_sides {
            let next_point = Point {
                x: center.x + size.x * current_angle.cos(),
                y: center.y + size.y * current_angle.sin(),
                #[cfg(feature = "3d")]
                z: center.z,
            };
            segments.push(Box::new(LineSegment {
                start: previous_point,
                end: next_point,
            }) as Box<dyn VectorValuedFn>);
            current_angle += angle_increment;
            previous_point = next_point;
        }

        // Close the polygon
        segments.push(Box::new(LineSegment {
            start: previous_point,
            end: Point {
                x: center.x + size.x * start_angle.cos(),
                y: center.y + size.y * start_angle.sin(),
                #[cfg(feature = "3d")]
                z: center.z,
            },
        }) as Box<dyn VectorValuedFn>);

        Self::new(segments)
    }

    /// draws a rectangle
    pub fn rectangle(origin: Point, size: Vector) -> Self {
        let points = [
            Point {
                x: origin.x,
                y: origin.y,
                #[cfg(feature = "3d")]
                z: origin.z,
            },
            Point {
                x: origin.x + size.x,
                y: origin.y,
                #[cfg(feature = "3d")]
                z: origin.z,
            },
            Point {
                x: origin.x + size.x,
                y: origin.y + size.y,
                #[cfg(feature = "3d")]
                z: origin.z,
            },
            Point {
                x: origin.x,
                y: origin.y + size.y,
                #[cfg(feature = "3d")]
                z: origin.z,
            },
        ];

        let segments = vec![
            Box::new(LineSegment {
                start: points[0],
                end: points[1],
            }) as Box<dyn VectorValuedFn>,
            Box::new(LineSegment {
                start: points[1],
                end: points[2],
            }) as Box<dyn VectorValuedFn>,
            Box::new(LineSegment {
                start: points[2],
                end: points[3],
            }) as Box<dyn VectorValuedFn>,
            Box::new(LineSegment {
                start: points[3],
                end: points[0],
            }) as Box<dyn VectorValuedFn>,
        ];

        Self::new(segments)
    }
}

impl VectorValuedFn for Path {
    fn eval(&self, t: crate::Float) -> crate::Vector {
        let total_length: Float = self.lengths.iter().sum();
        let mut accumulated_length: Float = 0.0;
        for (i, &length) in self.lengths.iter().enumerate() {
            if t * total_length < accumulated_length + length {
                let local_t = (t * total_length - accumulated_length) / length;
                return self.segments[i].eval(local_t);
            }
            accumulated_length += length;
        }
        self.segments.last().unwrap().eval(1.0)
    }

    fn length(&self) -> Float {
        self.lengths.iter().sum()
    }

    fn sample_optimal(&self) -> Vec<Vector> {
        let mut all: Vec<Vector> = self
            .segments
            .iter()
            .flat_map(|s| s.sample_optimal())
            .collect();

        all.dedup();

        all
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;
    use crate::{test_util::test_name, LineSegment, Point};
    use insta::assert_debug_snapshot;

    #[test]
    fn test_path_eval() {
        let line1 = Box::new(LineSegment {
            start: Point {
                x: 0.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            end: Point {
                x: 1.0,
                y: 1.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
        });
        let line2 = Box::new(LineSegment {
            start: Point {
                x: 1.0,
                y: 1.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            end: Point {
                x: 2.0,
                y: 2.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
        });
        let path = Path::new(vec![line1, line2]);

        assert_debug_snapshot!(
            test_name("path-eval"),
            [path.eval(0.0), path.eval(0.5), path.eval(1.0),]
        );
    }

    #[test]
    fn test_path_length() {
        let line1 = Box::new(LineSegment {
            start: Point {
                x: 0.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            end: Point {
                x: 1.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
        });
        let line2 = Box::new(LineSegment {
            start: Point {
                x: 1.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            end: Point {
                x: 2.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
        });
        let path = Path::new(vec![line1, line2]);

        assert_eq!(path.length(), 2.0);
    }

    #[test]
    fn test_path_sample_optimal() {
        let line1 = Box::new(LineSegment {
            start: Point {
                x: 0.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            end: Point {
                x: 1.0,
                y: 1.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
        });
        let line2 = Box::new(LineSegment {
            start: Point {
                x: 1.0,
                y: 1.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            end: Point {
                x: 2.0,
                y: 2.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
        });
        let path = Path::new(vec![line1, line2]);

        let samples = path.sample_optimal();
        assert_debug_snapshot!(test_name("path-optimal"), samples);
    }

    #[test]
    fn test_polygon() {
        let center = Point {
            x: 0.0,
            y: 0.0,
            #[cfg(feature = "3d")]
            z: 0.0,
        };
        let size = Vector {
            x: 1.0,
            y: 1.0,
            #[cfg(feature = "3d")]
            z: 0.0,
        };
        let n_sides = 4;
        let start_angle = Angle::from_degrees(0.0);
        let polygon = Path::polygon(center, size, n_sides, start_angle);

        let samples = polygon.sample_optimal();
        assert_debug_snapshot!(test_name("polygon"), samples);
    }

    #[test]
    fn test_rectangle() {
        let origin = Point {
            x: 0.0,
            y: 0.0,
            #[cfg(feature = "3d")]
            z: 0.0,
        };
        let size = Vector {
            x: 1.0,
            y: 1.0,
            #[cfg(feature = "3d")]
            z: 0.0,
        };
        let rectangle = Path::rectangle(origin, size);

        let samples = rectangle.sample_optimal();
        assert_debug_snapshot!(test_name("rectangle"), samples);
    }
}
