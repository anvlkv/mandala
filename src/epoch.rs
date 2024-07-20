use std::ops::{Add, Div, Neg};

use derive_builder::Builder;
use euclid::{
    default::{Point2D, Rect, Size2D, Vector2D},
    Angle, Scale,
};
use lyon_geom::{Arc, LineSegment, Rotation};

use crate::{Float, Path, Segment};

/// Mandala Epoch
///
/// Draws a circle with radius and center
///
/// Circle is divided in segments with breadth
///
/// Each segment is filled with drawing rule
#[derive(Debug, Builder)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Epoch {
    /// number of segments
    pub segments: usize,
    /// radius of epoch
    pub radius: Float,
    /// breadth of segments, inwards, from edge to center
    pub breadth: Float,
    /// center
    pub center: Point2D<Float>,
    /// drawing rule for segments
    #[builder(default = "SegmentRule::None")]
    pub segment_rule: SegmentRule,
}

impl Epoch {
    /// Space available for drawing next epoch
    pub fn space_next(&self) -> Rect<Float> {
        let size = (self.radius - self.breadth) * 2.0;
        Rect::new(
            self.center.add_size(&Size2D::splat(-size / 2.0)),
            Size2D::splat(size),
        )
    }

    /// Render paths for every segment
    pub fn render_paths(&self) -> Vec<Path> {
        let mut paths = Vec::new();
        let radius = self.radius - self.breadth;
        let mut ring = Arc::circle(self.center, radius);
        // FIXME: not sure why is this needed but sometimes circle is of 0 length
        ring.x_rotation = Angle::pi();
        paths.push(Path::new(Segment::Arc(ring.to_svg_arc())));

        match &self.segment_rule {
            SegmentRule::Path(p) => {
                self.segment_transforms(&mut |tilt, translate_by, _| {
                    paths.push(p.rotate(tilt).translate(translate_by));
                });
            }
            SegmentRule::EveryNth(p, nth) => {
                self.segment_transforms(&mut |tilt, translate_by, i| {
                    if i.rem_euclid(*nth) == 0 {
                        paths.push(p.rotate(tilt).translate(translate_by));
                    }
                });
            }
            SegmentRule::OddEven(odd_p, even_p) => {
                self.segment_transforms(&mut |tilt, translate_by, i| {
                    let p = if i.rem_euclid(2) == 0 { even_p } else { odd_p };
                    paths.push(p.rotate(tilt).translate(translate_by));
                });
            }
            SegmentRule::None => {}
        }

        paths
    }

    /// Compute the necessary transformations for each segment
    pub fn segment_transforms<W>(&self, with: &mut W)
    where
        W: FnMut(Angle<Float>, Vector2D<Float>, usize),
    {
        let angle_step = Angle::<Float>::two_pi().radians / {
            if self.segments > 0 {
                self.segments as f64
            } else {
                1.0
            }
        };

        let radius = self.radius - self.breadth;

        for i in 1..=self.segments {
            let start_angle = (i - 1) as f64 * angle_step;

            let translate_by = Vector2D::new(
                self.center.x + radius * start_angle.cos(),
                self.center.y + radius * start_angle.sin(),
            );

            let tilt = Angle::radians(start_angle).add(Angle::frac_pi_2().neg());

            with(tilt, translate_by, i)
        }
    }

    /// Key points of all segments
    pub fn key_pts(&self) -> Vec<Point2D<Float>> {
        let mut all_pts = vec![];

        match &self.segment_rule {
            SegmentRule::Path(p) => {
                let pts = p.key_pts();
                self.segment_transforms(&mut |angle, translate, _| {
                    pts.iter().for_each(|p| {
                        let transformed = LineSegment {
                            from: self.center,
                            to: *p,
                        }
                        .transformed(&Rotation::new(angle))
                        .translate(translate)
                        .to();
                        all_pts.push(transformed);
                    });
                });
            }
            SegmentRule::EveryNth(p, nth) => {
                let pts = p.key_pts();
                self.segment_transforms(&mut |angle, translate, i| {
                    if i.rem_euclid(*nth) == 0 {
                        pts.iter().for_each(|p| {
                            let transformed = LineSegment {
                                from: self.center,
                                to: *p,
                            }
                            .transformed(&Rotation::new(angle))
                            .translate(translate)
                            .to();
                            all_pts.push(transformed);
                        });
                    }
                });
            }
            SegmentRule::OddEven(p1, p2) => {
                let pts_1 = p1.key_pts();
                let pts_2 = p2.key_pts();
                self.segment_transforms(&mut |angle, translate, i| {
                    let p = if i.rem_euclid(2) == 0 {
                        pts_1.clone()
                    } else {
                        pts_2.clone()
                    };

                    p.iter().for_each(|p| {
                        let transformed = LineSegment {
                            from: self.center,
                            to: *p,
                        }
                        .transformed(&Rotation::new(angle))
                        .translate(translate)
                        .to();
                        all_pts.push(transformed);
                    });
                });
            }
            SegmentRule::None => {}
        }

        all_pts
    }

    /// Given the bounds draw segment path
    ///
    /// The `draw` callback is called with:
    ///
    /// - bounding box of the inner arc
    /// - bounding box of the outter arc
    pub fn draw_segment<F>(&mut self, mut draw: F)
    where
        F: FnMut(Rect<Float>, Rect<Float>) -> SegmentRule,
    {
        let sweep_angle = Angle::<Float>::two_pi().div(self.segments as f64);
        let start_angle = Angle::frac_pi_2();
        let x_rotation = sweep_angle.div(2.0).neg();

        let outer_arc = Arc {
            center: self.center,
            radii: Vector2D::new(self.radius, self.radius),
            start_angle,
            sweep_angle,
            x_rotation,
        };

        let inner_radius = self.radius - self.breadth;
        let inner_arc = Arc {
            center: self.center,
            radii: Vector2D::new(inner_radius, inner_radius),
            start_angle,
            sweep_angle,
            x_rotation,
        };

        let outer_translate_by = Vector2D::new(
            self.center.x + inner_radius * start_angle.radians.cos(),
            self.center.y + inner_radius * start_angle.radians.sin(),
        );
        let inner_translate_by = Vector2D::new(
            self.center.x + inner_radius * start_angle.radians.cos(),
            self.center.y + inner_radius * start_angle.radians.sin(),
        );

        let min_rect = inner_arc
            .bounding_box()
            .to_rect()
            .translate(inner_translate_by.neg());
        let max_rect = outer_arc
            .bounding_box()
            .to_rect()
            .translate(outer_translate_by.neg());

        // assert!(
        if !min_rect.is_empty() && !max_rect.is_empty() {
            self.segment_rule = draw(min_rect, max_rect);
        }
        //     "epoch has zero drawing area... {self:#?}"
        // );
    }

    /// Scale epoch and all segments
    pub fn scale(&mut self, scale: Float, old_root_center: Point2D<Float>) {
        self.radius *= scale;
        self.breadth *= scale;
        let old_c_offset = LineSegment {
            from: old_root_center,
            to: self.center,
        };

        let new_c_offset = old_c_offset.transformed(&Scale::new(scale));

        self.center = new_c_offset.to();

        self.segment_rule.scale(scale);
    }

    /// Translate the epoch and its segments
    pub fn translate(&mut self, by: Vector2D<Float>) {
        self.segment_rule.translate(by);
        self.center = self.center.add_size(&Size2D::new(by.x, by.y));
    }
}

/// How to draw segments
///
/// Paths are zero based
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SegmentRule {
    /// Draw a path
    Path(Path),
    /// For every Nth segment draw a path
    EveryNth(Path, usize),
    /// For every odd segment draw the first path, every even the second
    OddEven(Path, Path),
    /// Draw nothing
    None,
}

impl SegmentRule {
    pub fn generate<R, G>(rng: &mut R, mut gen: G) -> Self
    where
        R: rand::Rng,
        G: FnMut(&mut R) -> Path,
    {
        match rng.gen_range(0..=2) {
            0 => Self::Path(gen(rng)),
            1 => Self::EveryNth(gen(rng), if rng.gen_bool(0.5) { 2 } else { 3 }),
            2 => Self::OddEven(gen(rng), gen(rng)),
            _ => unreachable!(),
        }
    }

    /// Scale the segment
    pub fn scale(&mut self, scale: Float) {
        match self {
            SegmentRule::Path(p) => p.scale(scale),
            SegmentRule::EveryNth(p, _) => p.scale(scale),
            SegmentRule::OddEven(p1, p2) => {
                p1.scale(scale);
                p2.scale(scale);
            }
            SegmentRule::None => {}
        }
    }

    /// Translate the segment
    pub fn translate(&mut self, by: Vector2D<Float>) {
        match self {
            SegmentRule::Path(p) => *p = p.translate(by),
            SegmentRule::EveryNth(p, _) => *p = p.translate(by),
            SegmentRule::OddEven(p1, p2) => {
                *p1 = p1.translate(by);
                *p2 = p2.translate(by);
            }
            SegmentRule::None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use euclid::{Point2D, Vector2D};
    use lyon_geom::QuadraticBezierSegment;

    use crate::{Epoch, Path, Segment, SegmentRule};

    #[test]
    fn render_epoch() {
        let epoch = Epoch {
            segments: 10,
            radius: 20.0,
            breadth: 5.0,
            center: Point2D::new(3.0, 3.0),
            segment_rule: SegmentRule::Path(Path::new(Segment::QuadraticCurve(
                QuadraticBezierSegment {
                    from: Point2D::new(0.0, 0.0),
                    to: Point2D::new(3.0, 0.0),
                    ctrl: Point2D::new(1.75, 2.0),
                },
            ))),
        };

        let rendered = epoch.render_paths();

        assert_eq!(rendered.len(), 11);
    }

    #[test]
    fn test_translate() {
        let mut epoch = Epoch {
            segments: 10,
            radius: 20.0,
            breadth: 5.0,
            center: Point2D::new(3.0, 3.0),
            segment_rule: SegmentRule::Path(Path::new(Segment::QuadraticCurve(
                QuadraticBezierSegment {
                    from: Point2D::new(0.0, 0.0),
                    to: Point2D::new(3.0, 0.0),
                    ctrl: Point2D::new(1.75, 2.0),
                },
            ))),
        };

        let translation = Vector2D::new(5.0, 5.0);
        epoch.translate(translation);

        assert_eq!(epoch.center, Point2D::new(8.0, 8.0));
    }

    #[test]
    fn test_scale() {
        let mut epoch = Epoch {
            segments: 10,
            radius: 20.0,
            breadth: 5.0,
            center: Point2D::new(3.0, 3.0),
            segment_rule: SegmentRule::Path(Path::new(Segment::QuadraticCurve(
                QuadraticBezierSegment {
                    from: Point2D::new(0.0, 0.0),
                    to: Point2D::new(3.0, 0.0),
                    ctrl: Point2D::new(1.75, 2.0),
                },
            ))),
        };

        let old_root_center = Point2D::new(0.0, 0.0);
        epoch.scale(2.0, old_root_center);

        assert_eq!(epoch.radius, 40.0);
        assert_eq!(epoch.breadth, 10.0);
        assert_eq!(epoch.center, Point2D::new(6.0, 6.0));
    }
}
