use std::ops::Add;

use derive_builder::Builder;
use euclid::{
    default::{Point2D, Rect, Vector2D},
    Angle,
};
use lyon_geom::Arc;

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
            x_rotation: Angle::zero(),
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

            match &self.segment_rule {
                SegmentRule::Path(p) => {
                    paths.push(
                        p.rotate(Angle::radians(start_angle).add(Angle::pi()))
                            .translate(translate_by),
                    );
                }
                SegmentRule::EveryNth(p, nth) => {
                    if i % nth == 0 {
                        paths.push(
                            p.rotate(Angle::radians(start_angle).add(Angle::pi()))
                                .translate(translate_by),
                        );
                    }
                }
                SegmentRule::OddEven(odd_p, even_p) => {
                    let p = if i % 2 == 0 { even_p } else { odd_p };
                    paths.push(
                        p.rotate(Angle::radians(start_angle).add(Angle::pi()))
                            .translate(translate_by),
                    );
                }
                SegmentRule::None => break,
            }
        }

        paths
    }

    /// Given the bounds draw segment path
    pub fn draw_segment<F>(&mut self, draw: F)
    where
        F: Fn(Rect<Float>, Rect<Float>) -> SegmentRule,
    {
        let outer_arc = Arc {
            center: self.center,
            radii: Vector2D::new(self.radius, self.radius),
            start_angle: Angle::zero(),
            sweep_angle: Angle::radians(Angle::<Float>::two_pi().radians / self.segments as f64),
            x_rotation: Angle::zero(),
        };
        let inner_arc = Arc {
            center: self.center,
            radii: Vector2D::new(self.radius - self.breadth, self.radius - self.breadth),
            start_angle: Angle::zero(),
            sweep_angle: outer_arc.sweep_angle,
            x_rotation: Angle::zero(),
        };

        let min_len = inner_arc.bounding_box().width();

        let max_len = outer_arc.bounding_box().width();

        let min_rect = Rect::from_points([
            Point2D::new(0.0, 0.0),
            Point2D::new(0.0, self.breadth),
            Point2D::new(min_len, 0.0),
            Point2D::new(min_len, self.breadth),
        ]);

        let max_rect = Rect::from_points([
            Point2D::new(0.0, 0.0),
            Point2D::new(0.0, self.breadth),
            Point2D::new(max_len, 0.0),
            Point2D::new(max_len, self.breadth),
        ]);

        self.segment_rule = draw(min_rect, max_rect);
    }
}

/// How to draw one segment
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

#[cfg(test)]
mod tests {
    use euclid::Point2D;
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
}
