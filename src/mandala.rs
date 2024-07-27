use crate::{BBox, Epoch, Path};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Mandala {
    /// bounds of the mandala in global coordinates
    pub bounds: BBox,
    /// content
    pub epochs: Vec<Epoch>,
}

impl Mandala {
    /// new mandala with its inner bounds
    pub fn new(bounds: BBox) -> Self {
        Self {
            bounds,
            epochs: vec![],
        }
    }

    /// draw next epoch based on the last one if any
    pub fn draw_epoch<F>(&mut self, draw_fn: F)
    where
        F: Fn(Option<&Epoch>, &BBox) -> Epoch,
    {
        self.epochs.push(draw_fn(self.epochs.last(), &self.bounds))
    }

    /// renders all epochs, all segments, all paths
    pub fn render_paths(&self) -> Vec<Path> {
        self.epochs.iter().flat_map(|e| e.render_paths()).collect()
    }
}

#[cfg(test)]
mod mandala_tests {}
