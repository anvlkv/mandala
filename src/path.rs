use std::collections::{linked_list::IntoIter, LinkedList};

use euclid::{
    default::{Point2D, Translation2D, Vector2D},
    Angle, Rotation2D,
};
use lyon_geom::{CubicBezierSegment, LineSegment, QuadraticBezierSegment, SvgArc, Triangle};

use crate::Float;

#[derive(Debug, Clone)]
pub struct Path(LinkedList<Segment>);

#[derive(Debug, Clone)]
pub enum Segment {
    Line(LineSegment<Float>),
    Arc(SvgArc<Float>),
    Triangle(Triangle<Float>),
    QuadraticCurve(QuadraticBezierSegment<Float>),
    CubicCurve(CubicBezierSegment<Float>),
}

impl Segment {
    pub fn length(&self) -> Float {
        match self {
            Segment::Line(s) => s.length(),
            Segment::Arc(s) => {
                let mut len = 0.0;
                let mut sum = |q: &QuadraticBezierSegment<Float>| {
                    len += q.length();
                };

                s.for_each_quadratic_bezier(&mut sum);

                len
            }
            Segment::Triangle(s) => s.a.distance_to(s.b) + s.b.distance_to(s.c),
            Segment::QuadraticCurve(s) => s.length(),
            Segment::CubicCurve(s) => s.to_quadratic().length(),
        }
    }

    pub fn from(&self) -> Point2D<Float> {
        match self {
            Segment::Line(s) => s.from,
            Segment::Arc(s) => s.from,
            Segment::Triangle(s) => s.a,
            Segment::QuadraticCurve(s) => s.from,
            Segment::CubicCurve(s) => s.from,
        }
    }

    pub fn to(&self) -> Point2D<Float> {
        match self {
            Segment::Line(s) => s.to,
            Segment::Arc(s) => s.to,
            Segment::Triangle(s) => s.c,
            Segment::QuadraticCurve(s) => s.to,
            Segment::CubicCurve(s) => s.to,
        }
    }

    pub fn translate(&self, by: Vector2D<Float>) -> Self {
        match self {
            Segment::Line(s) => Segment::Line(s.clone().translate(by)),
            Segment::Arc(s) => Segment::Arc(SvgArc {
                from: Point2D::new(s.from.x + by.x, s.from.y + by.y),
                to: Point2D::new(s.to.x + by.x, s.to.y + by.y),
                radii: s.radii,
                x_rotation: s.x_rotation,
                flags: s.flags,
            }),
            Segment::Triangle(s) => {
                Segment::Triangle(s.clone().transform(&Translation2D::new(by.x, by.y)))
            }
            Segment::QuadraticCurve(s) => {
                Segment::QuadraticCurve(s.clone().transformed(&Translation2D::new(by.x, by.y)))
            }
            Segment::CubicCurve(s) => {
                Segment::CubicCurve(s.clone().transformed(&Translation2D::new(by.x, by.y)))
            }
        }
    }

    pub fn rotate(&self, by: Angle<Float>) -> Self {
        match self {
            Segment::Line(s) => Segment::Line(s.clone().transformed(&Rotation2D::new(by))),
            Segment::Arc(s) => Segment::Arc(SvgArc {
                x_rotation: by,
                ..s.clone()
            }),
            Segment::Triangle(s) => Segment::Triangle(s.clone().transform(&Rotation2D::new(by))),
            Segment::QuadraticCurve(s) => {
                Segment::QuadraticCurve(s.clone().transformed(&Rotation2D::new(by)))
            }
            Segment::CubicCurve(s) => {
                Segment::CubicCurve(s.clone().transformed(&Rotation2D::new(by)))
            }
        }
    }
}

impl Path {
    pub fn new(first: Segment) -> Self {
        Self(LinkedList::from_iter(vec![first]))
    }

    pub fn draw_next<F>(&mut self, draw: F)
    where
        F: Fn(&Segment) -> Segment,
    {
        let last = self.0.front().expect("at least one element");

        let next = draw(last);

        assert_eq!(
            last.to(),
            next.from(),
            "same path seggments must be continuous"
        );

        self.0.push_front(next);
    }

    pub fn length(&self) -> Float {
        self.0.iter().fold(0.0, |l, segment| l + segment.length())
    }

    pub fn translate(&self, by: Vector2D<Float>) -> Self {
        Self(LinkedList::from_iter(
            self.0.iter().map(|s| s.translate(by)),
        ))
    }

    pub fn rotate(&self, by: Angle<Float>) -> Self {
        Self(LinkedList::from_iter(self.0.iter().map(|s| s.rotate(by))))
    }

    pub fn from(&self) -> Point2D<Float> {
        self.0.back().map(|s| s.from()).unwrap_or_default()
    }
    pub fn to(&self) -> Point2D<Float> {
        self.0.front().map(|s| s.to()).unwrap_or_default()
    }
}

impl IntoIterator for Path {
    type Item = Segment;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use euclid::{Angle, Vector2D};

    use super::*;

    #[test]
    fn test_path_length_with_multiple_segments() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        let mut path = Path::new(line);

        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(1.0, 1.0));
            Segment::Arc(SvgArc {
                from: Point2D::new(1.0, 1.0),
                to: Point2D::new(2.0, 0.0),
                radii: Vector2D::new(1.0, 1.0),
                x_rotation: Angle::degrees(40.0),
                flags: Default::default(),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(2.0, 0.0));
            Segment::Triangle(Triangle {
                a: Point2D::new(2.0, 0.0),
                b: Point2D::new(3.0, 0.0),
                c: Point2D::new(2.5, 1.0),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(2.5, 1.0));
            Segment::QuadraticCurve(QuadraticBezierSegment {
                from: Point2D::new(2.5, 1.0),
                ctrl: Point2D::new(3.0, 2.0),
                to: Point2D::new(4.0, 1.0),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(4.0, 1.0));
            Segment::CubicCurve(CubicBezierSegment {
                from: Point2D::new(4.0, 1.0),
                ctrl1: Point2D::new(5.0, 2.0),
                ctrl2: Point2D::new(6.0, 0.0),
                to: Point2D::new(7.0, 1.0),
            })
        });

        assert_eq!(path.length(), 9.983008840474714);
    }

    #[test]
    fn test_path_translate() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        let path = Path::new(line);
        let translated_path = path.translate(Vector2D::new(1.0, 1.0));
        let translated_line = translated_path.0.front().unwrap();
        match translated_line {
            Segment::Line(s) => {
                assert_eq!(s.from, Point2D::new(1.0, 1.0));
                assert_eq!(s.to, Point2D::new(2.0, 2.0));
            }
            _ => panic!("Expected a line segment"),
        }
    }

    #[test]
    fn test_segment_translate() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        let translated_line = line.translate(Vector2D::new(1.0, 1.0));
        match translated_line {
            Segment::Line(s) => {
                assert_eq!(s.from, Point2D::new(1.0, 1.0));
                assert_eq!(s.to, Point2D::new(2.0, 2.0));
            }
            _ => panic!("Expected a line segment"),
        }
    }

    #[test]
    fn test_segment_length() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        assert_eq!(line.length(), 1.4142135623730951);
    }

    #[test]
    fn test_path_draw_next() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        let mut path = Path::new(line);
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(1.0, 1.0));
            Segment::Line(LineSegment {
                from: Point2D::new(1.0, 1.0),
                to: Point2D::new(2.0, 2.0),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(2.0, 2.0));
            Segment::Arc(SvgArc {
                from: Point2D::new(2.0, 2.0),
                to: Point2D::new(3.0, 1.0),
                radii: Vector2D::new(1.0, 1.0),
                x_rotation: Angle::degrees(40.0),
                flags: Default::default(),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(3.0, 1.0));
            Segment::Triangle(Triangle {
                a: Point2D::new(3.0, 1.0),
                b: Point2D::new(4.0, 1.0),
                c: Point2D::new(3.5, 2.0),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(3.5, 2.0));
            Segment::QuadraticCurve(QuadraticBezierSegment {
                from: Point2D::new(3.5, 2.0),
                ctrl: Point2D::new(4.0, 3.0),
                to: Point2D::new(5.0, 2.0),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(5.0, 2.0));
            Segment::CubicCurve(CubicBezierSegment {
                from: Point2D::new(5.0, 2.0),
                ctrl1: Point2D::new(6.0, 3.0),
                ctrl2: Point2D::new(7.0, 1.0),
                to: Point2D::new(8.0, 2.0),
            })
        });
        assert_eq!(path.0.len(), 6);
    }

    #[test]
    fn test_path_from_and_to() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        let mut path = Path::new(line);

        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(1.0, 1.0));
            Segment::Arc(SvgArc {
                from: Point2D::new(1.0, 1.0),
                to: Point2D::new(2.0, 0.0),
                radii: Vector2D::new(1.0, 1.0),
                x_rotation: Angle::degrees(40.0),
                flags: Default::default(),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point2D::new(2.0, 0.0));
            Segment::Triangle(Triangle {
                a: Point2D::new(2.0, 0.0),
                b: Point2D::new(3.0, 0.0),
                c: Point2D::new(2.5, 1.0),
            })
        });

        assert_eq!(path.from(), Point2D::new(0.0, 0.0));
        assert_eq!(path.to(), Point2D::new(2.5, 1.0));
    }
}
