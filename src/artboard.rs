use crate::{BBox, Mandala};

/// the artboard is responsible for holding,
/// incrementally rendering mandalas,
/// and other operations with them
pub struct Artboard {
    pub bounds: BBox,
    pub mandalas: Vec<Mandala>,
}
