use crate::{magnitude, GlVec, Point, Vector, VectorValuedFn};

/// flat line in space with start and end
#[derive(Debug, Clone, Copy)]
pub struct LineSegment {
    pub start: Point,
    pub end: Point,
}

impl Default for LineSegment {
    fn default() -> Self {
        Self {
            start: GlVec::default().into(),
            end: GlVec::default().into(),
        }
    }
}

impl VectorValuedFn for LineSegment {
    fn eval(&self, t: crate::Float) -> crate::Vector {
        crate::Vector {
            x: self.start.x + (self.end.x - self.start.x) * t,
            y: self.start.y + (self.end.y - self.start.y) * t,
            #[cfg(feature = "3d")]
            z: self.start.z + (self.end.z - self.start.z) * t,
        }
    }

    fn length(&self) -> crate::Float {
        let d = GlVec::from(self.end) - GlVec::from(self.start);
        magnitude(d)
    }

    fn sample_optimal(&self) -> Vec<Vector> {
        vec![self.start.into(), self.end.into()]
    }
}

/// infinite line
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub direction: Vector,
    pub origin: Point,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            direction: GlVec::default().into(),
            origin: GlVec::default().into(),
        }
    }
}

impl VectorValuedFn for Line {
    fn length(&self) -> crate::Float {
        let d = GlVec::from(self.origin) - GlVec::from(self.direction);
        magnitude(d)
    }

    fn eval(&self, t: crate::Float) -> crate::Vector {
        crate::Vector {
            x: self.origin.x + self.direction.x * t,
            y: self.origin.y + self.direction.y * t,
            #[cfg(feature = "3d")]
            z: self.origin.z + self.direction.z * t,
        }
    }

    fn sample_optimal(&self) -> Vec<Vector> {
        vec![self.origin.into(), self.end().into()]
    }
}

#[cfg(test)]
mod line_tests {
    use super::*;
    use crate::test_util::test_name;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_line_segment_eval() {
        let line_segment = LineSegment {
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
                z: 1.0,
            },
        };
        assert_debug_snapshot!(
            test_name("line-segment"),
            [
                line_segment.eval(0.0),
                line_segment.eval(0.5),
                line_segment.eval(1.0)
            ]
        );
    }

    #[test]
    fn test_line_eval() {
        let line = Line {
            direction: Vector {
                x: 1.0,
                y: 1.0,
                #[cfg(feature = "3d")]
                z: 1.0,
            },
            origin: Point {
                x: 0.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
        };
        assert_debug_snapshot!(
            test_name("line"),
            [line.eval(0.0), line.eval(0.5), line.eval(1.0)]
        );
    }
}