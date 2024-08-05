use cfg_if::cfg_if;

use crate::{GlVec, Point, VectorValuedFn};

/// Quadratic Bezier curve with one control point
#[derive(Debug, Clone, Copy)]
pub struct QuadraticCurve {
    pub start: Point,
    pub control: Point,
    pub end: Point,
}

impl Default for QuadraticCurve {
    fn default() -> Self {
        Self {
            start: GlVec::default().into(),
            control: GlVec::default().into(),
            end: GlVec::default().into(),
        }
    }
}

impl VectorValuedFn for QuadraticCurve {
    fn eval(&self, t: crate::Float) -> crate::Vector {
        crate::Vector {
            x: (1.0 - t).powi(2) * self.start.x
                + 2.0 * (1.0 - t) * t * self.control.x
                + t.powi(2) * self.end.x,
            y: (1.0 - t).powi(2) * self.start.y
                + 2.0 * (1.0 - t) * t * self.control.y
                + t.powi(2) * self.end.y,
            #[cfg(feature = "3d")]
            z: (1.0 - t).powi(2) * self.start.z
                + 2.0 * (1.0 - t) * t * self.control.z
                + t.powi(2) * self.end.z,
        }
    }

    fn length(&self) -> crate::Float {
        let mut length = 0.0;
        let num_segments = 100;
        for i in 0..num_segments {
            let t1 = i as crate::Float / num_segments as crate::Float;
            let t2 = (i + 1) as crate::Float / num_segments as crate::Float;
            let p1 = self.eval(t1);
            let p2 = self.eval(t2);
            length += (p2.x - p1.x).hypot(p2.y - p1.y);
            cfg_if! { if #[cfg(feature = "3d")] {
                length += (p2.z - p1.z).abs();
            }}
        }
        length
    }
}

/// Cubic Bezier curve with two control points
#[derive(Debug, Clone, Copy)]
pub struct CubicCurve {
    pub start: Point,
    pub control1: Point,
    pub control2: Point,
    pub end: Point,
}

impl Default for CubicCurve {
    fn default() -> Self {
        Self {
            start: GlVec::default().into(),
            control1: GlVec::default().into(),
            control2: GlVec::default().into(),
            end: GlVec::default().into(),
        }
    }
}

impl VectorValuedFn for CubicCurve {
    fn eval(&self, t: crate::Float) -> crate::Vector {
        crate::Vector {
            x: (1.0 - t).powi(3) * self.start.x
                + 3.0 * (1.0 - t).powi(2) * t * self.control1.x
                + 3.0 * (1.0 - t) * t.powi(2) * self.control2.x
                + t.powi(3) * self.end.x,
            y: (1.0 - t).powi(3) * self.start.y
                + 3.0 * (1.0 - t).powi(2) * t * self.control1.y
                + 3.0 * (1.0 - t) * t.powi(2) * self.control2.y
                + t.powi(3) * self.end.y,
            #[cfg(feature = "3d")]
            z: (1.0 - t).powi(3) * self.start.z
                + 3.0 * (1.0 - t).powi(2) * t * self.control1.z
                + 3.0 * (1.0 - t) * t.powi(2) * self.control2.z
                + t.powi(3) * self.end.z,
        }
    }

    fn length(&self) -> crate::Float {
        let mut length = 0.0;
        let num_segments = 100;
        for i in 0..num_segments {
            let t1 = i as crate::Float / num_segments as crate::Float;
            let t2 = (i + 1) as crate::Float / num_segments as crate::Float;
            let p1 = self.eval(t1);
            let p2 = self.eval(t2);
            length += (p2.x - p1.x).hypot(p2.y - p1.y);
            cfg_if! { if #[cfg(feature = "3d")] {
                length += (p2.z - p1.z).abs();
            }}
        }
        length
    }
}

#[cfg(test)]
mod curve_tests {
    use super::*;
    use crate::test_util::test_name;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_quadratic_curve_eval() {
        let curve = QuadraticCurve {
            start: Point {
                x: 0.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            control: Point {
                x: 1.0,
                y: 1.0,
                #[cfg(feature = "3d")]
                z: 1.0,
            },
            end: Point {
                x: 2.0,
                y: 2.0,
                #[cfg(feature = "3d")]
                z: 2.0,
            },
        };
        assert_debug_snapshot!(
            test_name("QuadraticCurve"),
            [
                curve.eval(0.0),
                curve.eval(0.25),
                curve.eval(0.5),
                curve.eval(0.75),
                curve.eval(1.0),
            ]
        );
    }

    #[test]
    fn test_cubic_curve_eval() {
        let curve = CubicCurve {
            start: Point {
                x: 0.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            control1: Point {
                x: 1.0,
                y: 1.0,
                #[cfg(feature = "3d")]
                z: 1.0,
            },
            control2: Point {
                x: 2.0,
                y: 2.0,
                #[cfg(feature = "3d")]
                z: 2.0,
            },
            end: Point {
                x: 3.0,
                y: 3.0,
                #[cfg(feature = "3d")]
                z: 3.0,
            },
        };
        assert_debug_snapshot!(
            test_name("CubicCurve"),
            [
                curve.eval(0.0),
                curve.eval(0.25),
                curve.eval(0.5),
                curve.eval(0.75),
                curve.eval(1.0),
            ]
        );
    }
}
