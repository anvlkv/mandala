use std::{
    collections::{linked_list::IntoIter, LinkedList},
    ops::Add,
};

use euclid::{default::Translation2D, Rotation2D, Scale};

use ordered_float::OrderedFloat;

use crate::{Angle, Arc, CubicCurve, Float, Line, Point, QuadraticCurve, SvgArc, Vector};

/// Continuous path
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Path(LinkedList<PathSegment>);

impl Path {
    /// Given the first segment create new path
    pub fn new(first: PathSegment) -> Self {
        Self(LinkedList::from_iter(vec![first]))
    }

    /// Draw next segment of a continuoous path based on the last one
    pub fn draw_next<F>(&mut self, mut draw: F)
    where
        F: FnMut(&PathSegment) -> PathSegment,
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

    /// Total length of all path segments
    pub fn length(&self) -> Float {
        self.0.iter().fold(0.0, |l, segment| l + segment.length())
    }

    /// Startingg point of the path
    pub fn from(&self) -> Point {
        self.0.back().map(|s| s.from()).unwrap_or_default()
    }

    /// end point of the path
    pub fn to(&self) -> Point {
        self.0.front().map(|s| s.to()).unwrap_or_default()
    }

    /// Translate all segments
    pub fn translate(&self, by: Vector) -> Self {
        Self(LinkedList::from_iter(
            self.0.iter().map(|s| s.translate(by)),
        ))
    }

    /// Rotate all segments
    pub fn rotate(&self, by: Angle) -> Self {
        Self(LinkedList::from_iter(self.0.iter().map(|s| s.rotate(by))))
    }

    /// Scale all path segments
    pub fn scale(&self, scale: Float) -> Self {
        Self(LinkedList::from_iter(self.0.iter().map(|s| s.scale(scale))))
    }

    /// Key points of all path segments
    pub fn key_pts(&mut self) -> Vec<&mut Point> {
        self.0.iter_mut().flat_map(|s| s.key_pts()).collect()
    }

    /// flatten all path segments
    pub fn flattened(&self) -> Vec<Line> {
        self.0.iter().flat_map(|s| s.flattened()).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PathSegment {
    /// staright line
    Line(Line),
    /// arc
    Arc(SvgArc),
    /// quadratic curve
    QuadraticCurve(QuadraticCurve),
    /// cubic curv
    CubicCurve(CubicCurve),
}

impl PathSegment {
    /// flip the segment along the vertical axis, where the axis is positioned at a given `x` coordinate
    pub fn flip_along_y(&self, x_pos_axis: Float) -> Self {
        match self {
            PathSegment::Line(s) => PathSegment::Line(Line {
                to: Point::new(x_pos_axis - (s.from.x - x_pos_axis), s.from.y),
                from: Point::new(x_pos_axis - (s.to.x - x_pos_axis), s.to.y),
            }),
            PathSegment::Arc(s) => PathSegment::Arc(SvgArc {
                to: Point::new(x_pos_axis - (s.from.x - x_pos_axis), s.from.y),
                from: Point::new(x_pos_axis - (s.to.x - x_pos_axis), s.to.y),
                radii: s.radii,
                x_rotation: s.x_rotation,
                flags: s.flags,
            }),
            PathSegment::QuadraticCurve(s) => PathSegment::QuadraticCurve(QuadraticCurve {
                to: Point::new(x_pos_axis - (s.from.x - x_pos_axis), s.from.y),
                ctrl: Point::new(x_pos_axis - (s.ctrl.x - x_pos_axis), s.ctrl.y),
                from: Point::new(x_pos_axis - (s.to.x - x_pos_axis), s.to.y),
            }),
            PathSegment::CubicCurve(s) => PathSegment::CubicCurve(CubicCurve {
                to: Point::new(x_pos_axis - (s.from.x - x_pos_axis), s.from.y),
                ctrl1: Point::new(x_pos_axis - (s.ctrl1.x - x_pos_axis), s.ctrl1.y),
                ctrl2: Point::new(x_pos_axis - (s.ctrl2.x - x_pos_axis), s.ctrl2.y),
                from: Point::new(x_pos_axis - (s.to.x - x_pos_axis), s.to.y),
            }),
        }
    }

    /// length of the segment
    pub fn length(&self) -> Float {
        match self {
            PathSegment::Line(s) => s.length(),
            PathSegment::Arc(s) => {
                let mut len = 0.0;
                let mut sum = |q: &QuadraticCurve| {
                    len += q.length();
                };

                s.for_each_quadratic_bezier(&mut sum);

                len
            }
            PathSegment::QuadraticCurve(s) => s.length(),
            PathSegment::CubicCurve(s) => s.approximate_length(self.tolerable()),
        }
    }

    /// start point
    pub fn from(&self) -> Point {
        match self {
            PathSegment::Line(s) => s.from,
            PathSegment::Arc(s) => s.from,
            PathSegment::QuadraticCurve(s) => s.from,
            PathSegment::CubicCurve(s) => s.from,
        }
    }

    /// end point
    pub fn to(&self) -> Point {
        match self {
            PathSegment::Line(s) => s.to,
            PathSegment::Arc(s) => s.to,
            PathSegment::QuadraticCurve(s) => s.to,
            PathSegment::CubicCurve(s) => s.to,
        }
    }

    /// Key points of this segment
    pub fn key_pts(&mut self) -> Vec<&mut Point> {
        match self {
            PathSegment::Line(l) => vec![&mut l.from, &mut l.to],
            PathSegment::Arc(a) => {
                vec![&mut a.from, &mut a.to]
            }
            PathSegment::QuadraticCurve(q) => vec![&mut q.from, &mut q.ctrl, &mut q.to],
            PathSegment::CubicCurve(c) => vec![&mut c.from, &mut c.ctrl1, &mut c.ctrl2, &mut c.to],
        }
    }

    /// translate the segment
    pub fn translate(&self, by: Vector) -> Self {
        match self {
            PathSegment::Line(s) => PathSegment::Line(s.clone().translate(by)),
            PathSegment::Arc(s) => PathSegment::Arc(SvgArc {
                from: Point::new(s.from.x + by.x, s.from.y + by.y),
                to: Point::new(s.to.x + by.x, s.to.y + by.y),
                radii: s.radii,
                x_rotation: s.x_rotation,
                flags: s.flags,
            }),
            PathSegment::QuadraticCurve(s) => {
                PathSegment::QuadraticCurve(s.clone().transformed(&Translation2D::new(by.x, by.y)))
            }
            PathSegment::CubicCurve(s) => {
                PathSegment::CubicCurve(s.clone().transformed(&Translation2D::new(by.x, by.y)))
            }
        }
    }

    /// rotate the segment
    pub fn rotate(&self, by: Angle) -> Self {
        match self {
            PathSegment::Line(s) => PathSegment::Line(s.clone().transformed(&Rotation2D::new(by))),
            PathSegment::Arc(s) => {
                assert!(!s.is_straight_line(), "arc is a straight line... {s:#?}");
                let arc = s.to_arc();
                let bbox = arc.bounding_box();

                let center = Line {
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
                PathSegment::Arc(arc_r.to_svg_arc())
            }
            PathSegment::QuadraticCurve(s) => {
                PathSegment::QuadraticCurve(s.clone().transformed(&Rotation2D::new(by)))
            }
            PathSegment::CubicCurve(s) => {
                PathSegment::CubicCurve(s.clone().transformed(&Rotation2D::new(by)))
            }
        }
    }

    /// scale the segment
    pub fn scale(&self, scale: Float) -> Self {
        match self {
            PathSegment::Line(l) => PathSegment::Line(l.clone().transformed(&Scale::new(scale))),
            PathSegment::Arc(l) => {
                let arc = l.to_arc();
                let bbox = arc.bounding_box();
                let center = Line {
                    from: bbox.min,
                    to: bbox.max,
                }
                .transformed(&Scale::new(scale))
                .mid_point();
                let radii = Vector::new(arc.radii.x * scale, arc.radii.y * scale);
                let arc_r = Arc {
                    radii,
                    center,
                    ..arc
                };
                PathSegment::Arc(arc_r.to_svg_arc())
            }
            PathSegment::QuadraticCurve(l) => {
                PathSegment::QuadraticCurve(l.clone().transformed(&Scale::new(scale)))
            }
            PathSegment::CubicCurve(l) => {
                PathSegment::CubicCurve(l.clone().transformed(&Scale::new(scale)))
            }
        }
    }

    /// find intersections with the other segment
    pub fn intersection(&self, other: &Self) -> Option<Vec<Point>> {
        let own_lines = self.flattened();
        let other_lines = other.flattened();

        let mut intersections = vec![];

        for ln in own_lines {
            for ln2 in other_lines.iter() {
                if let Some(pt) = ln.intersection(ln2) {
                    intersections.push(pt)
                }
            }
        }

        if intersections.is_empty() {
            None
        } else {
            Some(intersections)
        }
    }

    /// naive tolerance
    pub fn tolerable(&self) -> Float {
        match self {
            PathSegment::Line(_) => 0.0,
            PathSegment::Arc(a) => a.radii.x.min(a.radii.y) / self.length(),
            PathSegment::QuadraticCurve(q) => quadratic_tolerance(*q).into(),
            PathSegment::CubicCurve(c) => {
                let mut inflection = None;
                c.for_each_inflection_t(&mut |pt| {
                    inflection = Some(pt);
                });

                let mut quads = vec![];

                if let Some(t) = inflection {
                    let (before, after) = c.split(t);
                    quads.push(before.to_quadratic());
                    quads.push(after.to_quadratic());
                } else {
                    quads.push(c.to_quadratic())
                }

                let min_tol = quads.into_iter().map(quadratic_tolerance).min();

                min_tol
                    .unwrap_or_else(|| quadratic_tolerance(c.to_quadratic()))
                    .into()
            }
        }
        .max(lyon_geom::Scalar::epsilon_for(Float::EPSILON))
    }

    /// flattened curve with naive tolerance
    pub fn flattened(&self) -> Vec<Line> {
        let tolerance = self.tolerable();
        match self {
            PathSegment::Line(l) => vec![*l],
            PathSegment::Arc(a) => {
                let mut lns = vec![];
                a.for_each_flattened(tolerance, &mut |ln| {
                    lns.push(*ln);
                });
                lns
            }
            PathSegment::QuadraticCurve(q) => {
                let mut lns = vec![];
                q.for_each_flattened(tolerance, &mut |ln| {
                    lns.push(*ln);
                });
                lns
            }
            PathSegment::CubicCurve(c) => {
                let mut lns = vec![];
                c.for_each_flattened(tolerance, &mut |ln| {
                    lns.push(*ln);
                });
                lns
            }
        }
    }
}

fn quadratic_tolerance(q: QuadraticCurve) -> OrderedFloat<Float> {
    let b = q.bounding_triangle();
    let ab_l = b.ab().length();
    let ac_l = b.ac().length();
    let bc_l = b.bc().length();
    let s = ab_l.min(ac_l.min(bc_l));
    let l = q.length();

    (s / l).into()
}

impl IntoIterator for Path {
    type Item = PathSegment;

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
    fn test_mutating_key_pts() {
        let mut path = Path::new(PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        }));

        let mut key_pts = path.key_pts();
        assert_eq!(key_pts.len(), 2);

        key_pts[0].x = 2.0;
        key_pts[0].y = 2.0;
        key_pts[1].x = 3.0;
        key_pts[1].y = 3.0;

        let key_pts = path.key_pts();
        assert_eq!(key_pts[0], &Point::new(2.0, 2.0));
        assert_eq!(key_pts[1], &Point::new(3.0, 3.0));
    }

    #[test]
    fn test_path_scale() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let path = Path::new(line);

        let path = path.scale(2.0);

        let scaled_line = path.0.front().unwrap();
        match scaled_line {
            PathSegment::Line(s) => {
                assert_eq!(s.from, Point::new(0.0, 0.0));
                assert_eq!(s.to, Point::new(2.0, 2.0));
            }
            _ => panic!("Expected a line segment"),
        }
    }

    #[test]
    fn test_segment_scale() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });

        let line = line.scale(2.0);

        match line {
            PathSegment::Line(s) => {
                assert_eq!(s.from, Point::new(0.0, 0.0));
                assert_eq!(s.to, Point::new(2.0, 2.0));
            }
            _ => panic!("Expected a line segment"),
        }
    }

    #[test]
    fn test_arc_segment_scale() {
        let arc = PathSegment::Arc(SvgArc {
            from: Point::new(1.0, 1.0),
            to: Point::new(2.0, 0.0),
            radii: Vector2D::new(1.0, 1.0),
            x_rotation: Angle::degrees(40.0),
            flags: Default::default(),
        });

        let arc = arc.scale(2.0);

        match arc {
            PathSegment::Arc(s) => {
                assert_eq!(s.radii, Vector2D::new(2.0, 2.0));
            }
            _ => panic!("Expected an arc segment"),
        }
    }

    #[test]
    fn test_path_length_with_multiple_segments() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let mut path = Path::new(line);

        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(1.0, 1.0));
            PathSegment::Arc(SvgArc {
                from: last.to(),
                to: Point::new(2.0, 0.0),
                radii: Vector2D::new(1.0, 1.0),
                x_rotation: Angle::degrees(40.0),
                flags: Default::default(),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(2.0, 0.0));
            PathSegment::QuadraticCurve(QuadraticCurve {
                from: last.to(),
                ctrl: Point::new(3.0, 2.0),
                to: Point::new(4.0, 1.0),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(4.0, 1.0));
            PathSegment::CubicCurve(CubicCurve {
                from: last.to(),
                ctrl1: Point::new(5.0, 2.0),
                ctrl2: Point::new(6.0, 0.0),
                to: Point::new(7.0, 1.0),
            })
        });

        assert_eq!(path.length(), 8.724776172089943);
    }

    #[test]
    fn test_path_translate() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let path = Path::new(line);
        let translated_path = path.translate(Vector2D::new(1.0, 1.0));
        let translated_line = translated_path.0.front().unwrap();
        match translated_line {
            PathSegment::Line(s) => {
                assert_eq!(s.from, Point::new(1.0, 1.0));
                assert_eq!(s.to, Point::new(2.0, 2.0));
            }
            _ => panic!("Expected a line segment"),
        }
    }

    #[test]
    fn test_segment_translate() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let translated_line = line.translate(Vector2D::new(1.0, 1.0));
        match translated_line {
            PathSegment::Line(s) => {
                assert_eq!(s.from, Point::new(1.0, 1.0));
                assert_eq!(s.to, Point::new(2.0, 2.0));
            }
            _ => panic!("Expected a line segment"),
        }
    }

    #[test]
    fn test_segment_length() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        assert_eq!(line.length(), 1.4142135623730951);
    }

    #[test]
    fn test_path_draw_next() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let mut path = Path::new(line);
        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(1.0, 1.0));
            PathSegment::Line(Line {
                from: last.to(),
                to: Point::new(2.0, 2.0),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(2.0, 2.0));
            PathSegment::Arc(SvgArc {
                from: last.to(),
                to: Point::new(3.0, 1.0),
                radii: Vector2D::new(1.0, 1.0),
                x_rotation: Angle::degrees(40.0),
                flags: Default::default(),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(3.0, 1.0));
            PathSegment::QuadraticCurve(QuadraticCurve {
                from: last.to(),
                ctrl: Point::new(4.0, 3.0),
                to: Point::new(5.0, 2.0),
            })
        });
        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(5.0, 2.0));
            PathSegment::CubicCurve(CubicCurve {
                from: last.to(),
                ctrl1: Point::new(6.0, 3.0),
                ctrl2: Point::new(7.0, 1.0),
                to: Point::new(8.0, 2.0),
            })
        });
        assert_eq!(path.0.len(), 5);
    }

    #[test]
    fn test_path_from_and_to() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let mut path = Path::new(line);

        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(1.0, 1.0));
            PathSegment::Arc(SvgArc {
                from: Point::new(1.0, 1.0),
                to: Point::new(2.0, 0.0),
                radii: Vector2D::new(1.0, 1.0),
                x_rotation: Angle::degrees(40.0),
                flags: Default::default(),
            })
        });

        assert_eq!(path.from(), Point::new(0.0, 0.0));
        assert_eq!(path.to(), Point::new(2.0, 0.0));
    }

    #[test]
    fn test_tolerable() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let arc = PathSegment::Arc(SvgArc {
            from: Point::new(1.0, 1.0),
            to: Point::new(2.0, 0.0),
            radii: Vector2D::new(1.0, 1.0),
            x_rotation: Angle::degrees(40.0),
            flags: Default::default(),
        });
        let quadratic_curve = PathSegment::QuadraticCurve(QuadraticCurve {
            from: Point::new(1.0, 1.0),
            ctrl: Point::new(2.0, 2.0),
            to: Point::new(3.0, 1.0),
        });
        let cubic_curve = PathSegment::CubicCurve(CubicCurve {
            from: Point::new(1.0, 1.0),
            ctrl1: Point::new(2.0, 2.0),
            ctrl2: Point::new(3.0, 0.0),
            to: Point::new(4.0, 1.0),
        });

        assert_eq!(line.tolerable(), 1e-8);

        assert_eq!(arc.tolerable(), 0.6355488958496096);
        assert_eq!(quadratic_curve.tolerable(), 0.616057448634553);
        assert_eq!(cubic_curve.tolerable(), 0.5749251040792732);
    }

    #[test]
    fn test_segment_intersection() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let arc = PathSegment::Arc(SvgArc {
            from: Point::new(1.0, 0.0),
            to: Point::new(0.0, 1.0),
            radii: Vector2D::new(1.0, 1.0),
            x_rotation: Angle::degrees(0.0),
            flags: Default::default(),
        });
        let quadratic_curve = PathSegment::QuadraticCurve(QuadraticCurve {
            from: Point::new(0.0, 1.0),
            ctrl: Point::new(1.0, 2.0),
            to: Point::new(2.0, 1.0),
        });

        let intersections = line.intersection(&arc);
        assert!(intersections.is_some());
        let intersections = intersections.unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(
            intersections[0],
            Point::new(0.49999999999999994, 0.49999999999999994)
        );

        let intersections = line.intersection(&quadratic_curve);
        assert!(intersections.is_some());
        let intersections = intersections.unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0], Point::new(1.0, 1.0));

        let intersections = arc.intersection(&quadratic_curve);
        assert!(intersections.is_none());
    }
}
