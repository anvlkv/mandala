use std::{
    collections::{linked_list::IntoIter, LinkedList},
    ops::Add,
};

use euclid::{
    default::{Point2D, Rect, Translation2D, Vector2D},
    Angle, Rotation2D, Scale, Size2D,
};
use lyon_geom::{
    Arc, ArcFlags, CubicBezierSegment, LineSegment, QuadraticBezierSegment, SvgArc, Triangle,
};
use ordered_float::OrderedFloat;

use crate::{util::rand_pt_in_bounds, Float};

const EPSILON_POW_2: Float = 1e-8 * 1e-8;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Path(LinkedList<Segment>);

impl Path {
    /// Generate path given its bounds and generation mode
    ///
    /// Symmetry is along Y axis
    ///
    /// Detail level determines number of iterations to generate single pass,
    /// using symmetry essentially doubles the detalisation
    pub fn generate<R>(rng: &mut R, bounds: Rect<Float>, symmetry: bool, detail: u8) -> Self
    where
        R: rand::Rng,
    {
        let g_bounds = if symmetry {
            Rect::new(
                bounds.origin,
                Size2D::new(bounds.width() / 2.0, bounds.height()),
            )
        } else {
            bounds
        };

        let path_start = g_bounds.min();
        let path_end = if symmetry {
            Point2D::new(
                g_bounds.max_x(),
                g_bounds.max_y() * rng.gen_range((EPSILON_POW_2)..1.0),
            )
        } else {
            Point2D::new(g_bounds.max_x(), path_start.y)
        };
        let baseline = LineSegment {
            from: path_start,
            to: path_end,
        };

        let mut path = Self::new({
            let frac = rng.gen_range((EPSILON_POW_2)..0.5);

            let p_bl = baseline.before_split(frac);

            Segment::generate(rng, p_bl.bounding_box().to_rect(), path_start, p_bl.to)
        });

        let mut it = (0..=detail).peekable();

        while let Some(_) = it.next() {
            path.draw_next(|last| {
                if it.peek().is_some() {
                    let ln = LineSegment {
                        from: last.to(),
                        to: Point2D::new(
                            g_bounds.max_x() * rng.gen_range((EPSILON_POW_2)..1.0),
                            g_bounds.max_y() * rng.gen_range((EPSILON_POW_2)..1.0),
                        ),
                    };
                    Segment::generate(rng, g_bounds, ln.from, ln.to)
                } else {
                    let ln = LineSegment {
                        from: last.to(),
                        to: baseline.to,
                    };
                    Segment::generate(rng, g_bounds, ln.from, ln.to)
                }
            });
        }

        if symmetry {
            let lst = path.0.clone();

            for s in lst.iter() {
                path.draw_next(|_| Segment::flip_along_y(s, g_bounds.max_x()))
            }
        }

        path
    }

    /// Given the first segment create new path
    pub fn new(first: Segment) -> Self {
        Self(LinkedList::from_iter(vec![first]))
    }

    /// Draw next segment of a continuoous path based on the last one
    pub fn draw_next<F>(&mut self, mut draw: F)
    where
        F: FnMut(&Segment) -> Segment,
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
    pub fn from(&self) -> Point2D<Float> {
        self.0.back().map(|s| s.from()).unwrap_or_default()
    }

    /// end point of the path
    pub fn to(&self) -> Point2D<Float> {
        self.0.front().map(|s| s.to()).unwrap_or_default()
    }

    /// Translate all segments
    pub fn translate(&self, by: Vector2D<Float>) -> Self {
        Self(LinkedList::from_iter(
            self.0.iter().map(|s| s.translate(by)),
        ))
    }

    /// Rotate all segments
    pub fn rotate(&self, by: Angle<Float>) -> Self {
        Self(LinkedList::from_iter(self.0.iter().map(|s| s.rotate(by))))
    }

    /// Scale all path segments
    pub fn scale(&mut self, scale: Float) {
        for s in self.0.iter_mut() {
            s.scale(scale);
        }
    }

    /// Key points of all path segments
    pub fn key_pts(&self) -> Vec<Point2D<Float>> {
        self.0.iter().flat_map(|s| s.key_pts()).collect()
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// generate a path segment given its bounds, start and end points
    pub fn generate<R>(
        rng: &mut R,
        bounds: Rect<Float>,
        from: Point2D<Float>,
        to: Point2D<Float>,
    ) -> Self
    where
        R: rand::Rng,
    {
        match rng.gen_range(0..=4) {
            0 => Self::Line(LineSegment { from, to }),
            1 => Self::Arc(SvgArc {
                from,
                to,
                radii: {
                    let l_r = bounds.width().min(bounds.height());
                    let h_r = bounds.width().max(bounds.height());

                    Vector2D::new(
                        rng.gen_range(l_r..=h_r) / 2.0,
                        rng.gen_range(l_r..=h_r) / 2.0,
                    )
                },
                x_rotation: Angle::zero(),
                flags: ArcFlags::default(),
            }),
            2 => Self::Triangle(Triangle {
                a: from,
                b: rand_pt_in_bounds(rng, bounds),
                c: to,
            }),
            3 => Self::QuadraticCurve(QuadraticBezierSegment {
                from,
                ctrl: rand_pt_in_bounds(rng, bounds),
                to,
            }),
            4 => Self::CubicCurve(CubicBezierSegment {
                from,
                ctrl1: rand_pt_in_bounds(rng, bounds),
                ctrl2: rand_pt_in_bounds(rng, bounds),
                to,
            }),
            _ => unreachable!(),
        }
    }

    /// flip the segment along the vertical axis, where the axis is positioned at a given `x` coordinate
    pub fn flip_along_y(&self, x_pos_axis: Float) -> Self {
        match self {
            Segment::Line(s) => Segment::Line(LineSegment {
                to: Point2D::new(x_pos_axis - (s.from.x - x_pos_axis), s.from.y),
                from: Point2D::new(x_pos_axis - (s.to.x - x_pos_axis), s.to.y),
            }),
            Segment::Arc(s) => Segment::Arc(SvgArc {
                to: Point2D::new(x_pos_axis - (s.from.x - x_pos_axis), s.from.y),
                from: Point2D::new(x_pos_axis - (s.to.x - x_pos_axis), s.to.y),
                radii: s.radii,
                x_rotation: s.x_rotation,
                flags: s.flags,
            }),
            Segment::Triangle(s) => Segment::Triangle(Triangle {
                c: Point2D::new(x_pos_axis - (s.a.x - x_pos_axis), s.a.y),
                b: Point2D::new(x_pos_axis - (s.b.x - x_pos_axis), s.b.y),
                a: Point2D::new(x_pos_axis - (s.c.x - x_pos_axis), s.c.y),
            }),
            Segment::QuadraticCurve(s) => Segment::QuadraticCurve(QuadraticBezierSegment {
                to: Point2D::new(x_pos_axis - (s.from.x - x_pos_axis), s.from.y),
                ctrl: Point2D::new(x_pos_axis - (s.ctrl.x - x_pos_axis), s.ctrl.y),
                from: Point2D::new(x_pos_axis - (s.to.x - x_pos_axis), s.to.y),
            }),
            Segment::CubicCurve(s) => Segment::CubicCurve(CubicBezierSegment {
                to: Point2D::new(x_pos_axis - (s.from.x - x_pos_axis), s.from.y),
                ctrl1: Point2D::new(x_pos_axis - (s.ctrl1.x - x_pos_axis), s.ctrl1.y),
                ctrl2: Point2D::new(x_pos_axis - (s.ctrl2.x - x_pos_axis), s.ctrl2.y),
                from: Point2D::new(x_pos_axis - (s.to.x - x_pos_axis), s.to.y),
            }),
        }
    }

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
            Segment::Triangle(s) => s.ab().length() + s.bc().length() + s.ca().length(),
            Segment::QuadraticCurve(s) => s.length(),
            Segment::CubicCurve(s) => s.approximate_length(self.tolerable()),
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

    /// Key points of this segment
    pub fn key_pts(&self) -> Vec<Point2D<Float>> {
        match self {
            Segment::Line(l) => vec![l.from, l.to],
            Segment::Arc(a) => {
                let mut pts = vec![];
                a.for_each_quadratic_bezier(&mut |q| {
                    pts.push(q.to);
                    pts.push(q.from);
                });
                pts
            }
            Segment::Triangle(t) => vec![t.a, t.b, t.c],
            Segment::QuadraticCurve(q) => vec![q.to, q.from],
            Segment::CubicCurve(c) => {
                let mut pts = vec![c.to, c.from];
                c.for_each_inflection_t(&mut |t| {
                    pts.push(c.sample(t));
                });
                pts
            }
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

    /// find intersections with the other segment
    pub fn intersection(&self, other: &Self) -> Option<Vec<Point2D<Float>>> {
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
            Segment::Line(_) | Segment::Triangle(_) => 0.0,
            Segment::Arc(a) => a.radii.x.min(a.radii.y) / self.length(),
            Segment::QuadraticCurve(q) => quadratic_tolerance(*q).into(),
            Segment::CubicCurve(c) => {
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
        .max(1e-8)
    }

    /// flattened curve with naive tolerance
    pub fn flattened(&self) -> Vec<LineSegment<Float>> {
        let tolerance = self.tolerable();
        match self {
            Segment::Line(l) => vec![*l],
            Segment::Arc(a) => {
                let mut lns = vec![];
                a.for_each_flattened(tolerance, &mut |ln| {
                    lns.push(*ln);
                });
                lns
            }
            Segment::Triangle(t) => vec![t.ab(), t.bc(), t.ca()],
            Segment::QuadraticCurve(q) => {
                let mut lns = vec![];
                q.for_each_flattened(tolerance, &mut |ln| {
                    lns.push(*ln);
                });
                lns
            }
            Segment::CubicCurve(c) => {
                let mut lns = vec![];
                c.for_each_flattened(tolerance, &mut |ln| {
                    lns.push(*ln);
                });
                lns
            }
        }
    }
}

fn quadratic_tolerance(q: QuadraticBezierSegment<Float>) -> OrderedFloat<Float> {
    let b = q.bounding_triangle();
    let ab_l = b.ab().length();
    let ac_l = b.ac().length();
    let bc_l = b.bc().length();
    let s = ab_l.min(ac_l.min(bc_l));
    let l = q.length();

    (s / l).into()
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

        assert_eq!(path.length(), 11.101042829224609);
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

    #[test]
    fn test_tolerable() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        let arc = Segment::Arc(SvgArc {
            from: Point2D::new(1.0, 1.0),
            to: Point2D::new(2.0, 0.0),
            radii: Vector2D::new(1.0, 1.0),
            x_rotation: Angle::degrees(40.0),
            flags: Default::default(),
        });
        let triangle = Segment::Triangle(Triangle {
            a: Point2D::new(1.0, 1.0),
            b: Point2D::new(2.0, 1.0),
            c: Point2D::new(1.5, 2.0),
        });
        let quadratic_curve = Segment::QuadraticCurve(QuadraticBezierSegment {
            from: Point2D::new(1.0, 1.0),
            ctrl: Point2D::new(2.0, 2.0),
            to: Point2D::new(3.0, 1.0),
        });
        let cubic_curve = Segment::CubicCurve(CubicBezierSegment {
            from: Point2D::new(1.0, 1.0),
            ctrl1: Point2D::new(2.0, 2.0),
            ctrl2: Point2D::new(3.0, 0.0),
            to: Point2D::new(4.0, 1.0),
        });

        assert_eq!(line.tolerable(), 1e-8);
        assert_eq!(triangle.tolerable(), 1e-8);

        assert_eq!(arc.tolerable(), 0.6355488958496096);
        assert_eq!(quadratic_curve.tolerable(), 0.616057448634553);
        assert_eq!(cubic_curve.tolerable(), 0.5749251040792732);
    }

    #[test]
    fn test_segment_intersection() {
        let line = Segment::Line(LineSegment {
            from: Point2D::new(0.0, 0.0),
            to: Point2D::new(1.0, 1.0),
        });
        let arc = Segment::Arc(SvgArc {
            from: Point2D::new(1.0, 0.0),
            to: Point2D::new(0.0, 1.0),
            radii: Vector2D::new(1.0, 1.0),
            x_rotation: Angle::degrees(0.0),
            flags: Default::default(),
        });
        let quadratic_curve = Segment::QuadraticCurve(QuadraticBezierSegment {
            from: Point2D::new(0.0, 1.0),
            ctrl: Point2D::new(1.0, 2.0),
            to: Point2D::new(2.0, 1.0),
        });

        let intersections = line.intersection(&arc);
        assert!(intersections.is_some());
        let intersections = intersections.unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(
            intersections[0],
            Point2D::new(0.49999999999999994, 0.49999999999999994)
        );

        let intersections = line.intersection(&quadratic_curve);
        assert!(intersections.is_some());
        let intersections = intersections.unwrap();
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0], Point2D::new(1.0, 1.0));

        let intersections = arc.intersection(&quadratic_curve);
        assert!(intersections.is_none());
    }

    #[test]
    fn test_path_generation() {
        let mut rng = rand::thread_rng();
        let bounds = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(10.0, 10.0));
        let path = Path::generate(&mut rng, bounds, false, 5);

        assert!(path.length() > 0.0);
        assert_eq!(path.from(), Point2D::new(0.0, 0.0));
        assert_eq!(path.to(), Point2D::new(10.0, 0.0));
    }

    #[test]
    fn test_symmetrical_path_generation() {
        let mut rng = rand::thread_rng();
        let bounds = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(10.0, 10.0));
        let path = Path::generate(&mut rng, bounds, true, 20);

        assert!(path.length() > 0.0);
        assert_eq!(path.from(), Point2D::new(0.0, 0.0));
        assert_eq!(path.to(), Point2D::new(10.0, 0.0));
    }

    #[test]
    fn test_segment_generation() {
        let mut rng = rand::thread_rng();
        let bounds = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(10.0, 10.0));
        let from = Point2D::new(0.0, 0.0);
        let to = Point2D::new(10.0, 10.0);
        let segment = Segment::generate(&mut rng, bounds, from, to);

        match segment {
            Segment::Line(s) => {
                assert_eq!(s.from, from);
                assert_eq!(s.to, to);
            }
            Segment::Arc(s) => {
                assert_eq!(s.from, from);
                assert_eq!(s.to, to);
            }
            Segment::Triangle(s) => {
                assert_eq!(s.a, from);
                assert_eq!(s.c, to);
            }
            Segment::QuadraticCurve(s) => {
                assert_eq!(s.from, from);
                assert_eq!(s.to, to);
            }
            Segment::CubicCurve(s) => {
                assert_eq!(s.from, from);
                assert_eq!(s.to, to);
            }
        }
    }
}
