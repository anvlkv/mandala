use std::ops::Add;

use euclid::{
    default::{Transform2D, Translation2D},
    Rotation2D, Scale,
};
use ordered_float::OrderedFloat;

use crate::{Angle, Arc, CubicCurve, Float, Line, Point, QuadraticCurve, Size, SvgArc, Vector};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PathSegment {
    /// point
    Point(Point),
    /// staright line
    Line(Line),
    /// svg arc from poit to point
    Arc(SvgArc),
    /// sweep arc with center and angle
    SweepArc(Arc),
    /// quadratic curve
    QuadraticCurve(QuadraticCurve),
    /// cubic curv
    CubicCurve(CubicCurve),
}

impl PathSegment {
    /// flip the segment along the vertical axis, where the axis is positioned at a given `x` coordinate
    pub fn flip_along_y(&self, x_pos_axis: Float) -> Self {
        match self {
            PathSegment::Point(p) => {
                PathSegment::Point(Point::new(x_pos_axis - (p.x - x_pos_axis), p.y))
            }
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
            PathSegment::SweepArc(s) => PathSegment::SweepArc(Arc {
                center: Point::new(x_pos_axis - (s.center.x - x_pos_axis), s.center.y),
                radii: s.radii,
                start_angle: s.start_angle,
                sweep_angle: s.sweep_angle,
                x_rotation: s.x_rotation,
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

    /// flip the segment along the horizontal axis, where the axis is positioned at a given `y` coordinate
    pub fn flip_along_x(&self, y_pos_axis: Float) -> Self {
        match self {
            PathSegment::Point(p) => {
                PathSegment::Point(Point::new(p.x, y_pos_axis - (p.y - y_pos_axis)))
            }
            PathSegment::Line(s) => PathSegment::Line(Line {
                to: Point::new(s.from.x, y_pos_axis - (s.from.y - y_pos_axis)),
                from: Point::new(s.to.x, y_pos_axis - (s.to.y - y_pos_axis)),
            }),
            PathSegment::Arc(s) => PathSegment::Arc(SvgArc {
                to: Point::new(s.from.x, y_pos_axis - (s.from.y - y_pos_axis)),
                from: Point::new(s.to.x, y_pos_axis - (s.to.y - y_pos_axis)),
                radii: s.radii,
                x_rotation: s.x_rotation,
                flags: s.flags,
            }),
            PathSegment::SweepArc(s) => PathSegment::SweepArc(Arc {
                center: Point::new(s.center.x, y_pos_axis - (s.center.y - y_pos_axis)),
                radii: s.radii,
                start_angle: s.start_angle,
                sweep_angle: s.sweep_angle,
                x_rotation: s.x_rotation,
            }),
            PathSegment::QuadraticCurve(s) => PathSegment::QuadraticCurve(QuadraticCurve {
                to: Point::new(s.from.x, y_pos_axis - (s.from.y - y_pos_axis)),
                ctrl: Point::new(s.ctrl.x, y_pos_axis - (s.ctrl.y - y_pos_axis)),
                from: Point::new(s.to.x, y_pos_axis - (s.to.y - y_pos_axis)),
            }),
            PathSegment::CubicCurve(s) => PathSegment::CubicCurve(CubicCurve {
                to: Point::new(s.from.x, y_pos_axis - (s.from.y - y_pos_axis)),
                ctrl1: Point::new(s.ctrl1.x, y_pos_axis - (s.ctrl1.y - y_pos_axis)),
                ctrl2: Point::new(s.ctrl2.x, y_pos_axis - (s.ctrl2.y - y_pos_axis)),
                from: Point::new(s.to.x, y_pos_axis - (s.to.y - y_pos_axis)),
            }),
        }
    }

    /// length of the segment
    pub fn length(&self) -> Float {
        self.flattened()
            .into_iter()
            .fold(0.0, |l, ln| l + ln.length())
    }

    /// start point
    pub fn from(&self) -> Point {
        match self {
            PathSegment::Point(p) => *p,
            PathSegment::Line(s) => s.from,
            PathSegment::Arc(s) => s.from,
            PathSegment::SweepArc(s) => s.from(),
            PathSegment::QuadraticCurve(s) => s.from,
            PathSegment::CubicCurve(s) => s.from,
        }
    }

    /// end point
    pub fn to(&self) -> Point {
        match self {
            PathSegment::Point(p) => *p,
            PathSegment::Line(s) => s.to,
            PathSegment::Arc(s) => s.to,
            PathSegment::SweepArc(s) => s.to(),
            PathSegment::QuadraticCurve(s) => s.to,
            PathSegment::CubicCurve(s) => s.to,
        }
    }

    /// Key points of this segment
    pub fn key_pts(&mut self) -> Vec<&mut Point> {
        match self {
            PathSegment::Point(p) => vec![p],
            PathSegment::Line(l) => vec![&mut l.from, &mut l.to],
            PathSegment::Arc(a) => {
                vec![&mut a.from, &mut a.to]
            }
            PathSegment::SweepArc(a) => {
                vec![&mut a.center]
            }
            PathSegment::QuadraticCurve(q) => vec![&mut q.from, &mut q.ctrl, &mut q.to],
            PathSegment::CubicCurve(c) => vec![&mut c.from, &mut c.ctrl1, &mut c.ctrl2, &mut c.to],
        }
    }

    pub fn transform(&self, t: &Transform2D<Float>) -> Self {
        match self {
            PathSegment::Point(p) => PathSegment::Point(t.transform_point(*p)),
            PathSegment::Line(s) => PathSegment::Line(s.clone().transformed(t)),
            PathSegment::Arc(s) => PathSegment::Arc(SvgArc {
                from: t.transform_point(s.from),
                to: t.transform_point(s.to),
                radii: t.transform_vector(s.radii),
                x_rotation: s.x_rotation,
                flags: s.flags,
            }),
            PathSegment::SweepArc(s) => PathSegment::SweepArc(Arc {
                center: t.transform_point(s.center),
                radii: t.transform_vector(s.radii),
                start_angle: s.start_angle,
                sweep_angle: s.sweep_angle,
                x_rotation: s.x_rotation,
            }),
            PathSegment::QuadraticCurve(s) => PathSegment::QuadraticCurve(s.clone().transformed(t)),
            PathSegment::CubicCurve(s) => PathSegment::CubicCurve(s.clone().transformed(t)),
        }
    }

    /// translate the segment
    pub fn translate(&self, by: Vector) -> Self {
        match self {
            PathSegment::Point(p) => PathSegment::Point(p.add_size(&Size::new(by.x, by.y))),
            PathSegment::Line(s) => PathSegment::Line(s.clone().translate(by)),
            PathSegment::Arc(s) => PathSegment::Arc(SvgArc {
                from: Point::new(s.from.x + by.x, s.from.y + by.y),
                to: Point::new(s.to.x + by.x, s.to.y + by.y),
                radii: s.radii,
                x_rotation: s.x_rotation,
                flags: s.flags,
            }),
            PathSegment::SweepArc(s) => PathSegment::SweepArc(Arc {
                center: Point::new(s.center.x + by.x, s.center.y + by.y),
                radii: s.radii,
                start_angle: s.start_angle,
                sweep_angle: s.sweep_angle,
                x_rotation: s.x_rotation,
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
            PathSegment::Point(p) => PathSegment::Point(Point::new(
                p.x * by.radians.cos() - p.y * by.radians.sin(),
                p.x * by.radians.sin() + p.y * by.radians.cos(),
            )),
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
            PathSegment::SweepArc(s) => PathSegment::SweepArc(Arc {
                center: Point::new(
                    s.center.x * by.radians.cos() - s.center.y * by.radians.sin(),
                    s.center.x * by.radians.sin() + s.center.y * by.radians.cos(),
                ),
                radii: s.radii,
                start_angle: s.start_angle + by,
                sweep_angle: s.sweep_angle,
                x_rotation: s.x_rotation,
            }),
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
            PathSegment::Point(p) => PathSegment::Point(Point::new(p.x * scale, p.y * scale)),
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
            PathSegment::SweepArc(s) => PathSegment::SweepArc(Arc {
                center: Point::new(s.center.x * scale, s.center.y * scale),
                radii: Vector::new(s.radii.x * scale, s.radii.y * scale),
                start_angle: s.start_angle,
                sweep_angle: s.sweep_angle,
                x_rotation: s.x_rotation,
            }),
            PathSegment::QuadraticCurve(l) => {
                PathSegment::QuadraticCurve(l.clone().transformed(&Scale::new(scale)))
            }
            PathSegment::CubicCurve(l) => {
                PathSegment::CubicCurve(l.clone().transformed(&Scale::new(scale)))
            }
        }
    }

    /// find intersections with the other segment
    pub fn line_intersection(&self, line: &Line) -> Option<Vec<Point>> {
        match self {
            PathSegment::Point(pt) => {
                if line.distance_to_point(*pt) <= Float::EPSILON.powi(2) {
                    Some(vec![*pt])
                } else {
                    None
                }
            }
            PathSegment::Line(ln) => ln.intersection(line).map(|pt| vec![pt]),
            PathSegment::QuadraticCurve(q) => {
                let i = q.line_segment_intersections(line);
                if !i.is_empty() {
                    Some(Vec::from_iter(i))
                } else {
                    None
                }
            }
            PathSegment::CubicCurve(c) => {
                let i = c.line_segment_intersections(line);
                if !i.is_empty() {
                    Some(Vec::from_iter(i))
                } else {
                    None
                }
            }
            PathSegment::Arc(a) => {
                let mut i = vec![];
                a.for_each_cubic_bezier(&mut |b| {
                    let ii = b.line_segment_intersections(line);
                    i.extend(ii)
                });
                if !i.is_empty() {
                    Some(i)
                } else {
                    None
                }
            }
            PathSegment::SweepArc(a) => {
                let mut i = vec![];
                a.for_each_cubic_bezier(&mut |b| {
                    let ii = b.line_segment_intersections(line);
                    i.extend(ii)
                });
                if !i.is_empty() {
                    Some(i)
                } else {
                    None
                }
            }
        }
    }

    /// naive tolerance
    pub fn tolerable(&self) -> Float {
        match self {
            PathSegment::Line(_) | PathSegment::Point(_) => 0.0,
            PathSegment::Arc(a) => {
                let r = a.radii.x.min(a.radii.y);
                r / (r * Angle::two_pi().radians)
            }
            PathSegment::SweepArc(a) => {
                let r = a.radii.x.min(a.radii.y);
                r / (r * a.sweep_angle.radians.abs())
            }
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
        .max(lyon_geom::Scalar::epsilon_for(Float::EPSILON).powi(2))
        .min(1.0)
    }

    /// flattened curve with naive tolerance
    pub fn flattened(&self) -> Vec<Line> {
        let tolerance = self.tolerable();
        match self {
            PathSegment::Point(l) => vec![Line { from: *l, to: *l }],
            PathSegment::Line(l) => vec![*l],
            PathSegment::Arc(a) => {
                let mut lns = vec![];
                a.for_each_flattened(tolerance, &mut |ln| {
                    lns.push(*ln);
                });
                lns
            }
            PathSegment::SweepArc(a) => {
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

#[cfg(test)]
mod segment_tests {
    use super::*;
    use crate::{Angle, Vector};

    #[test]
    fn test_arc_segment_scale() {
        let arc = PathSegment::Arc(SvgArc {
            from: Point::new(1.0, 1.0),
            to: Point::new(2.0, 0.0),
            radii: Vector::new(1.0, 1.0),
            x_rotation: Angle::degrees(40.0),
            flags: Default::default(),
        });

        let arc = arc.scale(2.0);

        match arc {
            PathSegment::Arc(s) => {
                assert_eq!(s.radii, Vector::new(2.0, 2.0));
            }
            _ => panic!("Expected an arc segment"),
        }
    }

    #[test]
    fn test_segment_translate() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let translated_line = line.translate(Vector::new(1.0, 1.0));
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
    fn test_tolerable() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let arc = PathSegment::Arc(SvgArc {
            from: Point::new(1.0, 1.0),
            to: Point::new(2.0, 0.0),
            radii: Vector::new(1.0, 1.0),
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

        assert_eq!(line.tolerable(), 1.0000000000000001e-16);

        assert_eq!(arc.tolerable(), 0.15915494309189535);
        assert_eq!(quadratic_curve.tolerable(), 0.616057448634553);
        assert_eq!(cubic_curve.tolerable(), 0.5749251040792732);
    }

    #[test]
    fn test_segment_intersection() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(2.0, 2.0),
        });

        let intersections = line.line_intersection(&Line {
            from: Point::new(2.0, 0.0),
            to: Point::new(0.0, 2.0),
        });
        assert!(intersections.is_some());
        let intersections = intersections.unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0], Point::new(1.0, 1.0));

        let quadratic_curve = PathSegment::QuadraticCurve(QuadraticCurve {
            from: Point::new(0.0, 1.0),
            ctrl: Point::new(1.0, 2.0),
            to: Point::new(2.0, 1.0),
        });
        let intersections = quadratic_curve.line_intersection(&Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(2.0, 2.0),
        });
        assert!(intersections.is_some());
        let intersections = intersections.unwrap();
        assert_eq!(intersections.len(), 1);

        assert_eq!(
            intersections[0],
            Point::new(1.414213562373095, 1.414213562373095)
        );

        let arc = PathSegment::Arc(SvgArc {
            from: Point::new(1.0, 0.0),
            to: Point::new(0.0, 1.0),
            radii: Vector::new(1.0, 1.0),
            x_rotation: Angle::degrees(0.0),
            flags: Default::default(),
        });
        let intersections = arc.line_intersection(&Line {
            from: Point::new(0.0, 1.0),
            to: Point::new(2.0, 1.0),
        });
        assert!(intersections.is_some());

        let intersections = intersections.unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(
            intersections[0],
            Point::new(2.6722199095606244e-30, 0.9999999999999977)
        );
    }
}
