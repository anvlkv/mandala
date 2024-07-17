use std::{
    collections::{linked_list::IntoIter, LinkedList},
    ops::Add,
};

use euclid::{
    default::{Point2D, Translation2D, Vector2D},
    Angle, Rotation2D, Scale,
};
use lyon_geom::{Arc, CubicBezierSegment, LineSegment, QuadraticBezierSegment, SvgArc, Triangle};

use crate::Float;

#[derive(Debug, Clone)]
pub struct Path(LinkedList<Segment>);

impl Path {
    /// given the first segment create new path
    pub fn new(first: Segment) -> Self {
        Self(LinkedList::from_iter(vec![first]))
    }

    /// draw next segment of a continuoous path based on the last one
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

    /// total length of all path segments
    pub fn length(&self) -> Float {
        self.0.iter().fold(0.0, |l, segment| l + segment.length())
    }

    /// startingg point of the path
    pub fn from(&self) -> Point2D<Float> {
        self.0.back().map(|s| s.from()).unwrap_or_default()
    }

    /// end point of the path
    pub fn to(&self) -> Point2D<Float> {
        self.0.front().map(|s| s.to()).unwrap_or_default()
    }

    /// translate all segments
    pub fn translate(&self, by: Vector2D<Float>) -> Self {
        Self(LinkedList::from_iter(
            self.0.iter().map(|s| s.translate(by)),
        ))
    }

    /// rotate all segments
    pub fn rotate(&self, by: Angle<Float>) -> Self {
        Self(LinkedList::from_iter(self.0.iter().map(|s| s.rotate(by))))
    }

    /// scale all path segments
    pub fn scale(&mut self, scale: Float) {
        for s in self.0.iter_mut() {
            s.scale(scale);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Segment {
    /// staright line
    Line(LineSegment<Float>),
    /// arc
    Arc(SvgArc<Float>),
    /// triangle
    Triangle(Triangle<Float>),
    /// quadratic curve
    QuadraticCurve(QuadraticBezierSegment<Float>),
    /// cubic curv
    CubicCurve(CubicBezierSegment<Float>),
}

impl Segment {
    /// length of the segment
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

    /// start point
    pub fn from(&self) -> Point2D<Float> {
        match self {
            Segment::Line(s) => s.from,
            Segment::Arc(s) => s.from,
            Segment::Triangle(s) => s.a,
            Segment::QuadraticCurve(s) => s.from,
            Segment::CubicCurve(s) => s.from,
        }
    }

    /// end point
    pub fn to(&self) -> Point2D<Float> {
        match self {
            Segment::Line(s) => s.to,
            Segment::Arc(s) => s.to,
            Segment::Triangle(s) => s.c,
            Segment::QuadraticCurve(s) => s.to,
            Segment::CubicCurve(s) => s.to,
        }
    }

    /// translate the segment
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

    /// rotate the segment
    pub fn rotate(&self, by: Angle<Float>) -> Self {
        match self {
            Segment::Line(s) => Segment::Line(s.clone().transformed(&Rotation2D::new(by))),
            Segment::Arc(s) => {
                assert!(!s.is_straight_line(), "arc is a straight line... {s:#?}");
                let arc = s.to_arc();
                let bbox = arc.bounding_box();

                let center = LineSegment {
                    from: bbox.min,
                    to: bbox.max,
                }
                .transformed(&Rotation2D::new(by))
                .mid_point();
                let x_rotation = arc.x_rotation.add(by);

                let arc_r = Arc {
                    x_rotation,
                    center,
                    ..arc
                };
                Segment::Arc(arc_r.to_svg_arc())
            }
            Segment::Triangle(s) => Segment::Triangle(s.clone().transform(&Rotation2D::new(by))),
            Segment::QuadraticCurve(s) => {
                Segment::QuadraticCurve(s.clone().transformed(&Rotation2D::new(by)))
            }
            Segment::CubicCurve(s) => {
                Segment::CubicCurve(s.clone().transformed(&Rotation2D::new(by)))
            }
        }
    }

    /// scale the segment
    pub fn scale(&mut self, scale: Float) {
        match self {
            Segment::Line(l) => {
                *l = l.transformed(&Scale::new(scale));
            }
            Segment::Arc(l) => {
                let arc = l.to_arc();
                let bbox = arc.bounding_box();
                let center = LineSegment {
                    from: bbox.min,
                    to: bbox.max,
                }
                .transformed(&Scale::new(scale))
                .mid_point();
                let radii = Vector2D::new(arc.radii.x * scale, arc.radii.y * scale);
                let arc_r = Arc {
                    radii,
                    center,
                    ..arc
                };
                *l = arc_r.to_svg_arc();
            }
            Segment::Triangle(l) => *l = l.transform(&Scale::new(scale)),
            Segment::QuadraticCurve(l) => *l = l.transformed(&Scale::new(scale)),
            Segment::CubicCurve(l) => *l = l.transformed(&Scale::new(scale)),
        }
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
    fn test_path_scale() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        let mut path = Path::new(line);

        path.scale(2.0);

        let scaled_line = path.0.front().unwrap();
        match scaled_line {
            Segment::Line(s) => {
                assert_eq!(s.from, Point2D::new(0.0, 0.0));
                assert_eq!(s.to, Point2D::new(2.0, 2.0));
            }
            _ => panic!("Expected a line segment"),
        }
    }

    #[test]
    fn test_segment_scale() {
        let mut line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });

        line.scale(2.0);

        match line {
            Segment::Line(s) => {
                assert_eq!(s.from, Point2D::new(0.0, 0.0));
                assert_eq!(s.to, Point2D::new(2.0, 2.0));
            }
            _ => panic!("Expected a line segment"),
        }
    }

    #[test]
    fn test_arc_segment_scale() {
        let mut arc = Segment::Arc(SvgArc {
            from: Point2D::new(1.0, 1.0),
            to: Point2D::new(2.0, 0.0),
            radii: Vector2D::new(1.0, 1.0),
            x_rotation: Angle::degrees(40.0),
            flags: Default::default(),
        });

        arc.scale(2.0);

        match arc {
            Segment::Arc(s) => {
                assert_eq!(s.radii, Vector2D::new(2.0, 2.0));
            }
            _ => panic!("Expected an arc segment"),
        }
    }

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
