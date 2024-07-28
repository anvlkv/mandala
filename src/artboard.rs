use crate::{BBox, Mandala};

/// the artboard is responsible for holding,
/// incrementally rendering mandalas,
/// and other operations with them
pub struct Artboard {
    /// absolute bounds of the artboard
    pub bounds: BBox,
    /// all mandalas of the artboard
    pub mandalas: Vec<Mandala>,
}
