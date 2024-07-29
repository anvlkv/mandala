mod segment;

use std::collections::{linked_list::IntoIter, LinkedList};

use euclid::default::Transform2D;

use crate::{Angle, Arc, BBox, CubicCurve, Float, Line, Point, Rect, Size, SvgArc, Vector};

pub use segment::*;

/// Continuous path
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Path(LinkedList<PathSegment>);

impl Path {
    /// path shape rectangle
    pub fn rect(size: Size, center: Point) -> Self {
        let mut path = Self::default();

        let half_width = size.width / 2.0;
        let half_height = size.height / 2.0;

        let points = [
            Point::new(center.x - half_width, center.y - half_height),
            Point::new(center.x + half_width, center.y - half_height),
            Point::new(center.x + half_width, center.y + half_height),
            Point::new(center.x - half_width, center.y + half_height),
        ];

        for i in 0..4 {
            path.0.push_front(PathSegment::Line(Line {
                from: points[i],
                to: points[(i + 1) % 4],
            }));
        }

        debug_assert!(path.is_closed());

        path
    }

    /// path shape n-sides polygon
    pub fn polygon(sides: usize, bounds: Rect, start_angle: Angle) -> Self {
        let center = bounds.center();
        let radius = bounds.width().min(bounds.height()) / 2.0;
        let angle_step = Angle::two_pi() / sides as Float;

        let mut path = Self::default();

        let pt_0 = {
            let x = center.x + radius * start_angle.radians.cos();
            let y = center.y + radius * start_angle.radians.sin();
            Point::new(x, y)
        };

        for i in 1..=sides {
            let from = if i == 1 {
                pt_0
            } else {
                let angle = angle_step * (i - 1) as Float + start_angle;
                let x = center.x + radius * angle.radians.cos();
                let y = center.y + radius * angle.radians.sin();
                Point::new(x, y)
            };
            let to = if i == sides {
                pt_0
            } else {
                let angle = angle_step * i as Float + start_angle;
                let x = center.x + radius * angle.radians.cos();
                let y = center.y + radius * angle.radians.sin();
                Point::new(x, y)
            };

            path.0.push_front(PathSegment::Line(Line { from, to }))
        }

        debug_assert!(
            path.is_closed(),
            "path from: {:?}, path to {:?}",
            path.from(),
            path.to()
        );

        path
    }

    /// path shape circle
    pub fn circle(center: Point, radius: Float) -> Self {
        assert!(radius > 0.0);
        Self::new(PathSegment::SweepArc(Arc::circle(center, radius)))
    }

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

        self.0.push_front(next);
    }

    /// insert a point to move to
    pub fn move_to(&mut self, pt: Point) {
        self.0.push_front(PathSegment::Point(pt));
    }

    /// tests if the path is closed
    pub fn is_closed(&self) -> bool {
        self.0
            .front()
            .zip(self.0.back())
            .map(|(f, b)| {
                b.from() == f.to()
                    || (b == f
                        && match b {
                            PathSegment::SweepArc(a) => a.sweep_angle >= Angle::two_pi(),
                            _ => false,
                        })
            })
            .unwrap_or(false)
    }

    /// Total length of all path segments
    pub fn length(&self) -> Float {
        self.0.iter().fold(0.0, |l, segment| l + segment.length())
    }

    /// starting point of the path
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

    /// Apply transform to all segments
    pub fn transform(&self, t: &Transform2D<Float>) -> Self {
        Self(LinkedList::from_iter(self.0.iter().map(|s| s.transform(t))))
    }

    /// Rotate all segments
    pub fn rotate(&self, by: Angle) -> Self {
        Self(LinkedList::from_iter(self.0.iter().map(|s| s.rotate(by))))
    }

    /// Scale all path segments
    pub fn scale(&self, scale: Float) -> Self {
        Self(LinkedList::from_iter(self.0.iter().map(|s| s.scale(scale))))
    }

    pub fn flip_along_x(&self, x_pos: Float) -> Self {
        Self(LinkedList::from_iter(
            self.0.iter().map(|s| s.flip_along_x(x_pos)),
        ))
    }
    pub fn flip_along_y(&self, y_pos: Float) -> Self {
        Self(LinkedList::from_iter(
            self.0.iter().map(|s| s.flip_along_y(y_pos)),
        ))
    }

    /// Key points of all path segments
    pub fn key_pts(&mut self) -> Vec<&mut Point> {
        self.0.iter_mut().flat_map(|s| s.key_pts()).collect()
    }

    /// flatten all path segments
    pub fn flattened(&self) -> Vec<Line> {
        self.0.iter().flat_map(|s| s.flattened()).collect()
    }

    /// render path to svg path.d
    pub fn to_svg_path_d(&self) -> String {
        let mut it = self.0.iter().rev().peekable();
        let first = it.next().expect("path must not be empty");

        let mut d = format!("M {},{}", first.from().x, first.from().y);

        match first {
            PathSegment::Point(_) => {}
            _ => it = self.0.iter().rev().peekable(),
        }

        // aka its the only arc in a path and its a closed shape
        let forced_svg_arc = self.is_closed()
            && it
                .peek()
                .map(|s| matches!(s, PathSegment::Arc(_) | PathSegment::SweepArc(_)))
                .unwrap_or(false);

        let svg_arc_path = |s: &SvgArc, d: &mut String| {
            d.push_str(&format!(
                " A {},{} {} {} {} {},{}",
                s.radii.x,
                s.radii.y,
                s.x_rotation.to_degrees(),
                s.flags.large_arc as u8,
                s.flags.sweep as u8,
                s.to.x,
                s.to.y
            ));
        };

        let svg_arc_curve = |s: &Arc, d: &mut String| {
            s.for_each_cubic_bezier(&mut |CubicCurve {
                                              to, ctrl1, ctrl2, ..
                                          }| {
                d.push_str(&format!(
                    " C {},{} {},{} {},{}",
                    ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
                ));
            });
        };

        for s in it {
            match s {
                PathSegment::Line(s) => {
                    d.push_str(&format!(" L {},{}", s.to.x, s.to.y));
                }
                PathSegment::QuadraticCurve(s) => {
                    d.push_str(&format!(
                        " Q {},{} {},{}",
                        s.ctrl.x, s.ctrl.y, s.to.x, s.to.y
                    ));
                }
                PathSegment::CubicCurve(s) => {
                    d.push_str(&format!(
                        " C {},{} {},{} {},{}",
                        s.ctrl1.x, s.ctrl1.y, s.ctrl2.x, s.ctrl2.y, s.to.x, s.to.y
                    ));
                }
                PathSegment::Point(p) => {
                    d.push_str(&format!(" M {},{}", p.x, p.y));
                }
                PathSegment::Arc(s) => {
                    if forced_svg_arc {
                        let a = s.to_arc();
                        svg_arc_curve(&a, &mut d);
                    } else {
                        svg_arc_path(s, &mut d);
                    }
                }
                PathSegment::SweepArc(a) => {
                    if forced_svg_arc {
                        svg_arc_curve(a, &mut d);
                    } else {
                        let s = a.to_svg_arc();
                        svg_arc_path(&s, &mut d);
                    }
                }
            }
        }

        if self.is_closed() {
            d.push_str(" Z");
        }

        d
    }

    /// find an intersection of the path with a given line
    pub fn line_intersection(&self, line: &Line) -> Option<Point> {
        self.0
            .iter()
            .find_map(|s| s.line_intersection(line).map(|v| v.into_iter().next()))
            .flatten()
    }

    /// compute bounding box of the path
    pub fn bounds(&self) -> BBox {
        BBox {
            min: Point::new(
                self.0
                    .iter()
                    .map(|s| s.from().x)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                self.0
                    .iter()
                    .map(|s| s.from().y)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            ),
            max: Point::new(
                self.0
                    .iter()
                    .map(|s| s.to().x)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                self.0
                    .iter()
                    .map(|s| s.to().y)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            ),
        }
    }
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
    use crate::{Angle, CubicCurve, Line, Point, QuadraticCurve, Rect, Size, SvgArc, Vector};
    use lyon_geom::ArcFlags;

    use super::*;

    #[test]
    fn test_polygon_path() {
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        let path = Path::polygon(4, bounds, Angle::zero());
        assert_eq!(path.into_iter().len(), 4);
    }

    #[test]
    fn test_rect_path() {
        let size = Size::new(10.0, 10.0);
        let center = Point::new(5.0, 5.0);
        let path = Path::rect(size, center);

        assert!(path.is_closed());
        assert_eq!(path.length(), 40.0, "perimeter");
        assert_eq!(path.into_iter().len(), 4); // 4 sides
    }

    #[test]
    fn test_circle_path() {
        for i in 1..=50 {
            let r = i as Float * 5.07;
            let c = Path::circle(Point::splat(r), r);
            let expected_len = r * 2.0 * std::f64::consts::PI;
            assert!(
                (c.length() - expected_len).abs() < 0.5,
                "length: {ln} to expected_len: {expected_len} with r: {r}",
                ln = c.length()
            );
            insta::assert_debug_snapshot!(format!("r: {r}"), c.to_svg_path_d());
        }

        let d = Path::circle(Point::zero(), 200.0).to_svg_path_d();
        insta::assert_debug_snapshot!(d);
    }

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
                radii: Vector::new(1.0, 1.0),
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

        assert_eq!(path.length(), 8.562808362950431);
    }

    #[test]
    fn test_path_translate() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let path = Path::new(line);
        let translated_path = path.translate(Vector::new(1.0, 1.0));
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
                radii: Vector::new(1.0, 1.0),
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
                radii: Vector::new(1.0, 1.0),
                x_rotation: Angle::degrees(40.0),
                flags: Default::default(),
            })
        });

        assert_eq!(path.from(), Point::new(0.0, 0.0));
        assert_eq!(path.to(), Point::new(2.0, 0.0));
    }

    #[test]
    fn test_path_flattening() {
        let rect = Path::rect(Size::splat(200.0), Point::zero()).flattened();
        insta::assert_debug_snapshot!(rect);
        let poly = Path::polygon(
            6,
            Rect::new(Point::zero(), Size::splat(200.0)),
            Angle::zero(),
        )
        .flattened();
        insta::assert_debug_snapshot!(poly);
        let circle = Path::circle(Point::zero(), 100.0).flattened();
        insta::assert_debug_snapshot!(circle);
        let line = Path::new(PathSegment::Line(Line {
            from: Point::zero(),
            to: Point::splat(20.0),
        }))
        .flattened();
        assert_eq!(line.len(), 1, "line.len");

        let all_path = {
            let mut p = Path::new(PathSegment::Line(Line {
                from: Point::zero(),
                to: Point::splat(20.0),
            }));
            p.draw_next(|last| {
                PathSegment::Arc(SvgArc {
                    from: last.to(),
                    to: Point::splat(40.40),
                    radii: Vector::splat(30.0),
                    x_rotation: Angle::zero(),
                    flags: ArcFlags::default(),
                })
            });
            p.draw_next(|last| {
                PathSegment::QuadraticCurve(QuadraticCurve {
                    from: last.to(),
                    ctrl: Point::splat(60.0),
                    to: Point::new(80.0, 20.0),
                })
            });
            p.draw_next(|last| {
                PathSegment::CubicCurve(CubicCurve {
                    from: last.to(),
                    ctrl1: Point::new(100.0, 40.0),
                    ctrl2: Point::new(120.0, 60.0),
                    to: Point::new(140.0, 80.0),
                })
            });
            p.draw_next(|last| {
                PathSegment::Line(Line {
                    from: last.to(),
                    to: Point::splat(160.0),
                })
            });
            p
        }
        .flattened();

        insta::assert_debug_snapshot!(all_path);
    }

    #[test]
    fn test_closed_path() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let mut path = Path::new(line);

        path.draw_next(|last| {
            assert_eq!(last.to(), Point::new(1.0, 1.0));
            PathSegment::Line(Line {
                from: last.to(),
                to: Point::new(0.0, 0.0),
            })
        });

        assert!(path.is_closed());
    }

    #[test]
    fn test_to_svg_path_d() {
        let line = PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
        });
        let mut path = Path::new(line);

        path.draw_next(|last| {
            PathSegment::Arc(SvgArc {
                from: last.to(),
                to: Point::new(2.0, 0.0),
                radii: Vector::new(1.0, 1.0),
                x_rotation: Angle::degrees(40.0),
                flags: Default::default(),
            })
        });
        path.draw_next(|last| {
            PathSegment::QuadraticCurve(QuadraticCurve {
                from: last.to(),
                ctrl: Point::new(3.0, 2.0),
                to: Point::new(4.0, 1.0),
            })
        });
        path.draw_next(|last| {
            PathSegment::CubicCurve(CubicCurve {
                from: last.to(),
                ctrl1: Point::new(5.0, 2.0),
                ctrl2: Point::new(6.0, 0.0),
                to: Point::new(7.0, 1.0),
            })
        });

        let svg_path_d = path.to_svg_path_d();
        insta::assert_debug_snapshot!(svg_path_d)
    }
}
