use std::ops::{Range, RangeBounds};

use derive_builder::Builder;
use uuid::Uuid;

use crate::{
    polygon,
    segment::{MandalaSegment, ReplicaSegment},
    Angle, Arc, Float, Line, Path, PathSegment, Point, Rect, Size, Vector,
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
    pub segments: Vec<EpochSegment>,
    /// whether the epoch should render its outline
    #[builder(default)]
    pub outline: bool,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum EpochSegment {
    /// Original segment
    Segment(MandalaSegment),
    /// Replica
    Replica(ReplicaSegment),
}

impl EpochSegment {
    fn as_original<'a>(&'a self, ep: &'a Epoch) -> Option<&'a MandalaSegment> {
        match self {
            EpochSegment::Segment(s) => Some(s),
            EpochSegment::Replica(r) => ep.segments.iter().find_map(|s| match s {
                EpochSegment::Segment(o) => {
                    if o.id == r.replica_id {
                        Some(o)
                    } else {
                        None
                    }
                }
                EpochSegment::Replica(_) => None,
            }),
        }
    }

    fn angle_base(&self) -> Angle {
        match self {
            EpochSegment::Segment(s) => s.angle_base,
            EpochSegment::Replica(r) => r.angle_base,
        }
    }
}

/// Epoch layout variants
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum EpochLayout {
    /// plain circular layout
    ///
    /// places each segment by simply rotating it
    Circle { radius: Float },
    /// elliptic layout
    ///
    /// places each segment by rotating it,
    /// performs additional scaling to match the size
    Ellipse { radii: Size },
    /// ploygonal layout
    ///
    /// places each segment along the edges of the polygon, around the shape
    Polygon { n_sides: u8, radius: Float },
    /// rectangular layout
    ///
    /// places each segment along the edges of the rectangle, around the shape
    Rectangle { rect: Size },
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
            let s = segment
                .as_original(&self)
                .expect("replica must be in the same epoch");
            angle + segment.angle_base() + s.sweep
        });
        let max_sweep = self
            .segments
            .iter()
            .fold(Angle::two_pi(), |angle, segment| {
                let s = segment
                    .as_original(&self)
                    .expect("replica must be in the same epoch");

                angle - s.sweep
            });

        let args = DrawArgs {
            n,
            start_angle,
            max_sweep,
            center: self.center,
        };

        let segment = draw_fn(&args);

        self.segments.push(EpochSegment::Segment(segment));
    }

    /// draw one segment and fill available space with replicas
    pub fn draw_fill<D>(&mut self, draw_fn: &mut D)
    where
        D: FnMut(&DrawArgs) -> MandalaSegment,
    {
        self.draw_segment(draw_fn);
        let last = self.segments.last().unwrap().clone();
        let mut angle_base = last.angle_base();
        let original = last.as_original(&self).unwrap().clone();
        let sweep = original.sweep;
        let max_sweep = self
            .segments
            .iter()
            .fold(Angle::two_pi(), |angle, segment| {
                let s = segment
                    .as_original(&self)
                    .expect("replica must be in the same epoch");

                angle - s.sweep
            });
        let steps = (max_sweep.radians / sweep.radians).floor() as usize;

        for _ in 0..steps {
            self.segments.push(EpochSegment::Replica(
                original.replicate(angle_base + sweep),
            ));
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

        let len = range.len();

        let start_angle = self.segments.iter().fold(Angle::zero(), |angle, segment| {
            let s = segment
                .as_original(&self)
                .expect("replica must be in the same epoch");
            angle + segment.angle_base() + s.sweep
        });

        let max_sweep = self
            .segments
            .iter()
            .fold(Angle::two_pi(), |angle, segment| {
                let s = segment
                    .as_original(&self)
                    .expect("replica must be in the same epoch");

                angle - s.sweep
            })
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
            args.max_sweep -= segment.sweep;

            self.segments.push(EpochSegment::Segment(segment));
        }
    }

    /// renders all segments, all paths in global coordinates
    pub fn render(&self) -> Vec<Path> {
        self.segments
            .iter()
            .flat_map(|s| match s {
                EpochSegment::Segment(s) => s.render(),
                EpochSegment::Replica(r) => {
                    let original = s
                        .as_original(&self)
                        .expect("replica must be in the same epoch");
                    r.render(original)
                }
            })
            .chain(if self.outline {
                Some(match self.layout {
                    EpochLayout::Circle { radius } => Path::new(PathSegment::Arc(
                        Arc::circle(self.center, radius).to_svg_arc(),
                    )),
                    EpochLayout::Ellipse { radii } => Path::new(PathSegment::Arc(
                        Arc {
                            center: self.center,
                            start_angle: Angle::zero(),
                            sweep_angle: Angle::two_pi(),
                            x_rotation: Angle::zero(),
                            radii: Vector::new(radii.width, radii.height),
                        }
                        .to_svg_arc(),
                    )),
                    EpochLayout::Rectangle { rect } => {
                        let mut path = Path::new(PathSegment::Line(Line {
                            from: Point::new(
                                self.center.x - rect.width / 2.0,
                                self.center.y - rect.height / 2.0,
                            ),
                            to: Point::new(
                                self.center.x + rect.width / 2.0,
                                self.center.y - rect.height / 2.0,
                            ),
                        }));
                        path.draw_next(|last| {
                            PathSegment::Line(Line {
                                from: last.to(),
                                to: Point::new(
                                    self.center.x + rect.width / 2.0,
                                    self.center.y + rect.height / 2.0,
                                ),
                            })
                        });
                        path.draw_next(|last| {
                            PathSegment::Line(Line {
                                from: last.to(),
                                to: Point::new(
                                    self.center.x - rect.width / 2.0,
                                    self.center.y + rect.height / 2.0,
                                ),
                            })
                        });
                        path.draw_next(|last| {
                            PathSegment::Line(Line {
                                from: last.to(),
                                to: Point::new(
                                    self.center.x - rect.width / 2.0,
                                    self.center.y - rect.height / 2.0,
                                ),
                            })
                        });

                        path
                    }
                    EpochLayout::Polygon { n_sides, radius } => {
                        let origin = Point::new(self.center.x - radius, self.center.y - radius);

                        polygon(n_sides, Rect::new(origin, Size::splat(radius * 2.0)))
                    }
                })
            } else {
                None
            })
            .collect()
    }
}

#[cfg(test)]
mod epoch_tests {
    use crate::{MandalaSegmentBuilder, SegmentDrawing};

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
                .breadth(1.0)
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
                .breadth(1.0)
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
                    .breadth(1.0)
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
            .segments(vec![EpochSegment::Segment(
                MandalaSegmentBuilder::default()
                    .breadth(1.0)
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
                    .unwrap(),
            )])
            .build()
            .unwrap();

        let rendered = epoch.render();
        assert_eq!(rendered.len(), 2);
    }
}
