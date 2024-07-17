use euclid::default::{Box2D, Size2D, Vector2D};

use crate::{Epoch, EpochBuilder, Float, Path, SegmentRule};

pub struct Mandala {
    pub bounds: Box2D<Float>,
    pub epochs: Vec<Epoch>,
}

impl Mandala {
    /// create new mandala with a size
    pub fn new(size: Float) -> Self {
        let bounds = Box2D::from_size(Size2D::splat(size));
        let center = bounds.center();
        let radius = size / 2.0;
        let outer_circle = EpochBuilder::default()
            .center(center)
            .segments(0)
            .radius(radius)
            .breadth(0.0)
            .segment_rule(SegmentRule::None)
            .build()
            .expect("build outter circle epoch");

        Self {
            bounds,
            epochs: vec![outer_circle],
        }
    }

    /// render all paths
    pub fn render_drawing(&self) -> Vec<Path> {
        self.epochs.iter().flat_map(|e| e.render_paths()).collect()
    }

    /// draw new epoch based on the last one
    pub fn draw_epoch<F>(&mut self, mut draw: F)
    where
        F: FnMut(&Epoch) -> Epoch,
    {
        self.epochs.push(draw(self.epochs.last().unwrap()))
    }

    /// resize the mandala to fit given size
    pub fn resize(&mut self, new_size: Float) {
        let new_bounds = Box2D::from_size(Size2D::splat(new_size));
        let old_center = self.bounds.center();
        assert_eq!(
            self.bounds.width(),
            self.bounds.height(),
            "non square bounds"
        );
        let scale = new_size / self.bounds.width();

        for ep in self.epochs.iter_mut() {
            ep.scale(scale, old_center);
        }

        self.bounds = new_bounds;
    }

    /// translate the mandala and all its contents
    pub fn translate(&mut self, by: Vector2D<Float>) {
        self.bounds = self.bounds.translate(by);
        for ep in self.epochs.iter_mut() {
            ep.translate(by);
        }
    }
}

#[cfg(test)]
mod tests {
    use euclid::{Point2D, Vector2D};
    use lyon_geom::LineSegment;

    use crate::{Mandala, Path, Segment};

    #[test]
    fn test_draw_epoch() {
        let mut mandala = Mandala::new(200.0);
        mandala.draw_epoch(|epoch| {
            crate::EpochBuilder::default()
                .center(epoch.center)
                .segments(12)
                .radius(epoch.radius / 2.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::None)
                .build()
                .expect("build new epoch")
        });
        assert_eq!(mandala.epochs.len(), 2);
        let new_epoch = &mandala.epochs[1];
        assert_eq!(new_epoch.segments, 12);
        assert_eq!(new_epoch.radius, 50.0);
        assert_eq!(new_epoch.breadth, 5.0);
    }

    #[test]
    fn test_render_drawing() {
        let mut mandala = Mandala::new(200.0);
        mandala.draw_epoch(|epoch| {
            crate::EpochBuilder::default()
                .center(epoch.center)
                .segments(12)
                .radius(epoch.radius / 2.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::Path(Path::new(Segment::Line(
                    LineSegment {
                        from: Point2D::zero(),
                        to: Point2D::new(10.0, 10.0),
                    },
                ))))
                .build()
                .expect("build new epoch")
        });
        let drawing = mandala.render_drawing();
        assert_eq!(drawing.len(), 14); // 1 outer circle + 1 circle + 12 segments
    }

    #[test]
    fn test_resize() {
        let mut mandala = Mandala::new(200.0);
        mandala.draw_epoch(|epoch| {
            crate::EpochBuilder::default()
                .center(epoch.center)
                .segments(12)
                .radius(epoch.radius / 2.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::None)
                .build()
                .expect("build new epoch")
        });
        mandala.resize(400.0);
        assert_eq!(mandala.bounds.width(), 400.0);
        assert_eq!(mandala.bounds.height(), 400.0);
        assert_eq!(mandala.epochs[0].radius, 200.0);
        assert_eq!(mandala.epochs[1].radius, 100.0);
        assert_eq!(mandala.epochs[1].breadth, 10.0);
    }

    #[test]
    fn test_translate() {
        let mut mandala = Mandala::new(200.0);
        let translation = Vector2D::new(100.0, 100.0);
        mandala.translate(translation);
        assert_eq!(mandala.bounds.min.x, 100.0);
        assert_eq!(mandala.bounds.min.y, 100.0);
        assert_eq!(mandala.bounds.max.x, 300.0);
        assert_eq!(mandala.bounds.max.y, 300.0);
    }

    #[test]
    fn new() {
        _ = Mandala::new(200.0);
    }
}
