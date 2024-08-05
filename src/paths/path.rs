use crate::{Float, Vector, VectorValuedFn};

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
}
