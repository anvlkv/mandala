use std::ops::{Add, Div, Neg};

use derive_builder::Builder;
use euclid::{
    default::{Point2D, Rect, Size2D, Vector2D},
    Angle, Scale,
};
use lyon_geom::{Arc, LineSegment};

use crate::{Float, Path, Segment};

/// Mandala Epoch
///
/// Draws a circle with radius and center
///
/// Circle is divided in segments with breadth
///
/// Each segment is filled with drawing rule
#[derive(Debug, Builder)]
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
    /// Generate paths for every segment
    pub fn render_paths(&self) -> Vec<Path> {
        let mut paths = Vec::new();
        let radius = self.radius - self.breadth;
        let ring = Arc {
            center: self.center,
            radii: Vector2D::new(radius, radius),
            start_angle: Angle::zero(),
            sweep_angle: Angle::two_pi(),
            // appears as 0 length when not rotated...
            x_rotation: Angle::two_pi(),
        };
        paths.push(Path::new(Segment::Arc(ring.to_svg_arc())));

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

            match &self.segment_rule {
                SegmentRule::Path(p) => {
                    paths.push(p.rotate(tilt).translate(translate_by));
                }
                SegmentRule::EveryNth(p, nth) => {
                    if i.rem_euclid(*nth) == 0 {
                        paths.push(p.rotate(tilt).translate(translate_by));
                    }
                }
                SegmentRule::OddEven(odd_p, even_p) => {
                    let p = if i.rem_euclid(2) == 0 { even_p } else { odd_p };
                    paths.push(p.rotate(tilt).translate(translate_by));
                }
                SegmentRule::None => break,
            }
        }

        paths
    }

    /// Given the bounds draw segment path
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

        assert!(
            !min_rect.is_empty() && !max_rect.is_empty(),
            "epoch has zero drawing area... {self:#?}"
        );

        self.segment_rule = draw(min_rect, max_rect);
    }

    /// scale epoch and all segments
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

    /// translate the epoch and its segments
    pub fn translate(&mut self, by: Vector2D<Float>) {
        self.segment_rule.translate(by);
        self.center = self.center.add_size(&Size2D::new(by.x, by.y));
    }
}

/// How to draw segments
///
/// Paths are zero based
#[derive(Debug, Clone)]
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
    /// scale the segment
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

    /// translate the segment
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
