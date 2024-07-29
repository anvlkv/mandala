use std::ops::{Range, RangeBounds};

use derive_builder::Builder;
use euclid::Transform2D;
use uuid::Uuid;

use crate::{
    segment::MandalaSegment, Angle, Arc, Float, Line, Path, PathSegment, Point, Rect, Size, Vector,
};

/// Mandala Epoch
///
/// lays out segments of [mandala::Mandala]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Builder, Clone)]
pub struct Epoch {
    /// id of the epoch
    #[builder(default = "uuid::Uuid::new_v4()")]
    pub id: Uuid,
    /// center of the epoch
    pub center: Point,
    /// layout mode of the epoch
    pub layout: EpochLayout,
    /// content of the epoch
    #[builder(default)]
    pub segments: Vec<MandalaSegment>,
    /// whether the epoch should render its outline
    #[builder(default)]
    pub outline: bool,
}

/// Epoch layout variants
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum EpochLayout {
    /// plain circular layout
    ///
    /// places each segment by rotating it, and translating by the difference of radiuses between the segment and layout
    Circle { radius: Float },
    /// elliptic layout
    ///
    /// places each segment by rotating it, and translating by the difference of radiuses between the segment and layout at the base_angle
    Ellipse { radii: Size },
    /// ploygonal layout
    ///
    /// places each segment along the edges of the polygon, around the shape
    Polygon {
        n_sides: usize,
        radius: Float,
        start: Angle,
    },
    /// rectangular layout
    ///
    /// places each segment along the edges of the rectangle, around the shape
    Rectangle { rect: Size },
}

impl EpochLayout {
    pub fn outline(&self, center: Point) -> Path {
        match self {
            EpochLayout::Circle { radius } => Path::circle(center, *radius),
            EpochLayout::Ellipse { radii } => Path::new(PathSegment::SweepArc(Arc {
                center,
                start_angle: Angle::zero(),
                sweep_angle: Angle::two_pi(),
                x_rotation: Angle::zero(),
                radii: Vector::new(radii.width, radii.height),
            })),
            EpochLayout::Rectangle { rect } => Path::rect(*rect, center),
            EpochLayout::Polygon {
                n_sides,
                radius,
                start,
            } => {
                let origin = Point::new(center.x - radius, center.y - radius);

                Path::polygon(
                    *n_sides,
                    Rect::new(origin, Size::splat(radius * 2.0)),
                    *start,
                )
            }
        }
    }

    pub fn scale(&self, scale: Float) -> Self {
        match self.clone() {
            EpochLayout::Circle { radius } => EpochLayout::Circle {
                radius: radius * scale,
            },
            EpochLayout::Ellipse { radii } => EpochLayout::Ellipse {
                radii: Size::new(radii.width * scale, radii.height * scale),
            },
            EpochLayout::Polygon {
                n_sides,
                radius,
                start,
            } => EpochLayout::Polygon {
                n_sides,
                radius: radius * scale,
                start,
            },
            EpochLayout::Rectangle { rect } => EpochLayout::Rectangle {
                rect: Size::new(rect.width * scale, rect.height * scale),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawArgs {
    /// segment number
    ///
    /// 1-based
    pub n: usize,
    /// the starting angle of this draw
    pub start_angle: Angle,
    /// available anuglar space
    pub max_sweep: Angle,
    /// center of the epoch
    pub center: Point,
}

impl Epoch {
    /// draws next segment of the epoch
    pub fn draw_segment<D>(&mut self, draw_fn: &mut D)
    where
        D: FnMut(&DrawArgs) -> MandalaSegment,
    {
        let n = self.segments.len() + 1;
        let start_angle = self.segments.iter().fold(Angle::zero(), |angle, segment| {
            angle + segment.angle_base + segment.sweep
        });
        let max_sweep = self
            .segments
            .iter()
            .fold(Angle::two_pi(), |angle, segment| angle - segment.sweep);

        let args = DrawArgs {
            n,
            start_angle,
            max_sweep,
            center: self.center,
        };

        let segment = draw_fn(&args);

        self.segments.push(segment);
    }

    /// draw one segment and fill available space with replicas
    pub fn draw_fill<D>(&mut self, draw_fn: &mut D)
    where
        D: FnMut(&DrawArgs) -> MandalaSegment,
    {
        self.draw_segment(draw_fn);
        let segment = self.segments.last().unwrap().clone();
        let mut angle_base = segment.angle_base;
        let sweep = segment.sweep;
        let max_sweep = self
            .segments
            .iter()
            .fold(Angle::two_pi(), |angle, segment| angle - segment.sweep);
        let steps = (max_sweep.radians / sweep.radians).floor() as usize;

        for _ in 0..steps {
            self.segments.push(segment.replicate(angle_base + sweep));
            angle_base += sweep;
        }
    }

    /// draw next segments in range
    pub fn draw_range<D, R>(&mut self, draw_fn: &mut D, range: R)
    where
        D: FnMut(&DrawArgs) -> MandalaSegment,
        R: RangeBounds<usize>,
    {
        use std::ops::Bound;

        let range: Range<usize> = match (range.start_bound(), range.end_bound()) {
            (Bound::Included(s), Bound::Included(e)) => Range {
                start: *s,
                end: *e + 1,
            },
            (Bound::Included(s), Bound::Excluded(e)) => Range { start: *s, end: *e },
            (Bound::Included(s), Bound::Unbounded) => Range {
                start: *s,
                end: usize::MAX,
            },
            (Bound::Excluded(s), Bound::Included(e)) => Range {
                start: s + 1,
                end: *e + 1,
            },
            (Bound::Excluded(s), Bound::Excluded(e)) => Range {
                start: s + 1,
                end: *e,
            },
            (Bound::Excluded(s), Bound::Unbounded) => Range {
                start: s + 1,
                end: usize::MAX,
            },
            (Bound::Unbounded, Bound::Included(e)) => Range {
                start: 0,
                end: *e + 1,
            },
            (Bound::Unbounded, Bound::Excluded(e)) => Range { start: 0, end: *e },
            (Bound::Unbounded, Bound::Unbounded) => Range {
                start: 0,
                end: usize::MAX,
            },
        };

        let len = range.clone().count();

        let start_angle = self.segments.iter().fold(Angle::zero(), |angle, segment| {
            angle + segment.angle_base + segment.sweep
        });

        let max_sweep = self
            .segments
            .iter()
            .fold(Angle::two_pi(), |angle, segment| angle - segment.sweep)
            / len as f64;

        let n = self.segments.len() + 1;

        let mut args = DrawArgs {
            n,
            start_angle,
            max_sweep,
            center: self.center,
        };

        for _ in range {
            let segment = draw_fn(&args);
            args.n += 1;
            args.start_angle += segment.sweep;

            self.segments.push(segment);
        }
    }

    /// renders all segments, all paths in global coordinates
    pub fn render_paths(&self) -> Vec<Path> {
        self.segments
            .iter()
            .flat_map(|s| self.layout_segment(s))
            .chain(if self.outline {
                Some(self.layout.outline(self.center))
            } else {
                None
            })
            .collect()
    }

    /// translates all direct segments
    ///
    /// returns epoch with a new id
    pub fn translate(&self, by: Vector) -> Self {
        let mut next = self.clone();
        next.segments = next.segments.iter().map(|s| s.translate(by)).collect();
        next.center = Transform2D::translation(by.x, by.y).transform_point(next.center);
        next.id = Uuid::new_v4();

        next
    }

    /// scales all direct segments
    ///
    /// returns epoch with a new id
    pub fn scale(&self, scale: Float) -> Self {
        let mut next = self.clone();
        next.layout = next.layout.scale(scale);
        next.segments = next.segments.iter().map(|s| s.scale(scale)).collect();
        next.id = Uuid::new_v4();

        next
    }

    fn layout_segment(&self, segment: &MandalaSegment) -> Vec<Path> {
        let outline = self.layout.outline(self.center);

        let segment_outline = Path::new(PathSegment::SweepArc(Arc {
            center: segment.center,
            radii: Vector::splat(segment.r_base - segment.normalized_breadth()),
            x_rotation: Angle::zero(),
            // increased testing area
            start_angle: segment.angle_base - Angle::frac_pi_4(),
            sweep_angle: segment.sweep + Angle::frac_pi_2(),
        }));

        let outline_box = outline.bounds();
        let test_len =
            outline_box.width().max(outline_box.height()) + segment.center.distance_to(self.center);

        segment.render_paths_with(|pt: &Point| {
            let mut g_pt = Point::from(segment.to_global(pt.x, pt.y));

            let test_line = {
                let mut l = Line {
                    from: segment.center,
                    to: g_pt,
                };

                l.set_length(l.length() + test_len);

                l
            };

            if let Some(cross_outline) = outline.line_intersection(&test_line) {
                if let Some(cross_segment) = segment_outline.line_intersection(&test_line) {
                    let d_x = cross_outline.x - cross_segment.x;
                    let d_y = cross_outline.y - cross_segment.y;

                    g_pt = Transform2D::translation(d_x, d_y).transform_point(g_pt)
                }
            }
            g_pt
        })
    }
}

#[cfg(test)]
mod epoch_tests {
    use crate::{Line, MandalaSegmentBuilder, SegmentDrawing};

    use super::*;

    #[test]
    fn test_draw_segment() {
        let mut epoch = EpochBuilder::default()
            .center(Point::new(0.0, 0.0))
            .layout(EpochLayout::Circle { radius: 10.0 })
            .build()
            .unwrap();

        epoch.draw_segment(&mut |args| {
            MandalaSegmentBuilder::default()
                .breadth(0.5)
                .r_base(2.0)
                .angle_base(args.start_angle)
                .sweep(args.max_sweep)
                .center(args.center)
                .drawing(vec![])
                .build()
                .unwrap()
        });

        assert_eq!(epoch.segments.len(), 1);
    }

    #[test]
    fn test_draw_fill() {
        let mut epoch = EpochBuilder::default()
            .center(Point::new(0.0, 0.0))
            .layout(EpochLayout::Circle { radius: 10.0 })
            .build()
            .unwrap();

        epoch.draw_fill(&mut |args| {
            MandalaSegmentBuilder::default()
                .breadth(0.5)
                .r_base(2.0)
                .angle_base(args.start_angle)
                .sweep(Angle::radians(0.5))
                .center(args.center)
                .drawing(vec![])
                .build()
                .unwrap()
        });

        assert_eq!(epoch.segments.len(), 12);
    }

    #[test]
    fn test_draw_range() {
        let mut epoch = EpochBuilder::default()
            .center(Point::new(0.0, 0.0))
            .layout(EpochLayout::Circle { radius: 10.0 })
            .build()
            .unwrap();

        epoch.draw_range(
            &mut |args| {
                MandalaSegmentBuilder::default()
                    .breadth(0.5)
                    .r_base(2.0)
                    .angle_base(args.start_angle)
                    .sweep(args.max_sweep)
                    .center(args.center)
                    .drawing(vec![])
                    .build()
                    .unwrap()
            },
            0..3,
        );

        assert_eq!(epoch.segments.len(), 3);
    }

    #[test]
    fn test_render() {
        let epoch = EpochBuilder::default()
            .center(Point::new(0.0, 0.0))
            .layout(EpochLayout::Circle { radius: 10.0 })
            .outline(true)
            .segments(vec![MandalaSegmentBuilder::default()
                .breadth(0.5)
                .r_base(2.0)
                .angle_base(Angle::zero())
                .sweep(Angle::two_pi())
                .center(Point::new(0.0, 0.0))
                .drawing(vec![SegmentDrawing::Path(vec![Path::new(
                    PathSegment::Line(Line {
                        from: Point::new(0.0, 0.0),
                        to: Point::new(1.0, 1.0),
                    }),
                )])])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let rendered = epoch.render_paths();
        assert_eq!(rendered.len(), 2);
    }

    #[test]
    fn test_circle_layout() {
        let epoch = EpochBuilder::default()
            .center(Point::new(0.0, 0.0))
            .layout(EpochLayout::Circle { radius: 10.0 })
            .outline(true)
            .segments(vec![MandalaSegmentBuilder::default()
                .breadth(0.5)
                .r_base(2.0)
                .angle_base(Angle::zero())
                .sweep(Angle::two_pi())
                .center(Point::new(0.0, 0.0))
                .drawing(vec![SegmentDrawing::Path(vec![Path::new(
                    PathSegment::Line(Line {
                        from: Point::new(0.0, 0.0),
                        to: Point::new(1.0, 1.0),
                    }),
                )])])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let rendered = epoch.render_paths();
        assert_eq!(rendered.len(), 2);
    }

    #[test]
    fn test_ellipse_layout() {
        let epoch = EpochBuilder::default()
            .center(Point::new(0.0, 0.0))
            .layout(EpochLayout::Ellipse {
                radii: Size::new(10.0, 5.0),
            })
            .outline(true)
            .segments(vec![MandalaSegmentBuilder::default()
                .breadth(0.5)
                .r_base(2.0)
                .angle_base(Angle::zero())
                .sweep(Angle::two_pi())
                .center(Point::new(0.0, 0.0))
                .drawing(vec![SegmentDrawing::Path(vec![Path::new(
                    PathSegment::Line(Line {
                        from: Point::new(0.0, 0.0),
                        to: Point::new(1.0, 1.0),
                    }),
                )])])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let rendered = epoch.render_paths();
        assert_eq!(rendered.len(), 2);
    }

    #[test]
    fn test_polygon_layout() {
        let epoch = EpochBuilder::default()
            .center(Point::new(0.0, 0.0))
            .layout(EpochLayout::Polygon {
                n_sides: 5,
                radius: 10.0,
                start: Angle::zero(),
            })
            .outline(true)
            .segments(vec![MandalaSegmentBuilder::default()
                .breadth(0.5)
                .r_base(2.0)
                .angle_base(Angle::zero())
                .sweep(Angle::two_pi())
                .center(Point::new(0.0, 0.0))
                .drawing(vec![SegmentDrawing::Path(vec![Path::new(
                    PathSegment::Line(Line {
                        from: Point::new(0.0, 0.0),
                        to: Point::new(1.0, 1.0),
                    }),
                )])])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let rendered = epoch.render_paths();
        assert_eq!(rendered.len(), 2);
    }

    #[test]
    fn test_rectangle_layout() {
        let epoch = EpochBuilder::default()
            .center(Point::new(0.0, 0.0))
            .layout(EpochLayout::Rectangle {
                rect: Size::new(10.0, 5.0),
            })
            .outline(true)
            .segments(vec![MandalaSegmentBuilder::default()
                .breadth(0.5)
                .r_base(2.0)
                .angle_base(Angle::zero())
                .sweep(Angle::two_pi())
                .center(Point::new(0.0, 0.0))
                .drawing(vec![SegmentDrawing::Path(vec![Path::new(
                    PathSegment::Line(Line {
                        from: Point::new(0.0, 0.0),
                        to: Point::new(1.0, 1.0),
                    }),
                )])])
                .build()
                .unwrap()])
            .build()
            .unwrap();

        let rendered = epoch.render_paths();
        assert_eq!(rendered.len(), 2);
    }
}
