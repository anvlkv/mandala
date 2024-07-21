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
    /// renders all epochs, all segments, all paths
    pub fn render(&self) -> Vec<Path> {
        self.epochs.iter().flat_map(|e| e.render()).collect()
    }
}

#[cfg(test)]
mod tests {}
