use euclid::default::{Box2D, Point2D, Size2D};

use crate::{Epoch, EpochBuilder, Float, Path, SegmentRule};

pub struct Mandala {
    bounds: Box2D<Float>,
    epochs: Vec<Epoch>,
}

impl Mandala {
    pub fn new(size: Float) -> Self {
        let bounds = Box2D::from_size(Size2D::new(size, size));
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

    pub fn render_drawing(&self) -> Vec<Path> {
        self.epochs.iter().flat_map(|e| e.render_paths()).collect()
    }

    pub fn draw_epoch<F>(&mut self, draw: F)
    where
        F: Fn(&Epoch) -> Epoch,
    {
        self.epochs.push(draw(self.epochs.last().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use crate::Mandala;

    #[test]
    fn new() {
        _ = Mandala::new(200.0);
    }
}
