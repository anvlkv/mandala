use derive_builder::Builder;

use crate::{BBox, Float, Mandala, Path, Point};

/// Each chord of a mandala represents :
///
/// - normalized coordinate's space for drawing its contents
/// - two points in [Mandala] coodinates between which the chord spans
/// - transformation methods for its points between normalized and [Mandala] coordinate spaces
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq, Builder)]
pub struct Chord {
    /// starting point of a `Chord` in mandala coodinates
    ///
    /// together with `to` point determines the direction of drawing
    pub from: Point,
    /// end point of a `Chord` in mandala coodinates
    ///
    /// together with `from` point determines the direction of drawing
    pub to: Point,
    /// contents drawn by this `Chord` in normalized coordinates
    #[builder(setter(each(name = "draw")))]
    pub drawing: Vec<ChordDrawing>,
    /// a number to which the coordinate space of the `drawing` is normalized
    ///
    /// **Default: 100.0**
    #[builder(default = "100.0")]
    pub norm: Float,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ChordDrawing {
    /// [Path] drawing
    Paths {
        paths: Vec<Path>,
        #[cfg(feature = "styled")]
        /// optional style applied to all paths of this drawing
        ///
        /// individual [Path]'s style takes priority if present
        style: Option<crate::path::PathStyle>,
    },
    /// nested [Mandala]
    ///
    /// reintroduces the [Mandala] coordinate space
    ///
    /// get's scaled to the bounds defined in normalized coordinates
    Mandala {
        /// normalized bounds
        bounds: BBox,
        /// mandala with its own coordinate space
        mandala: Mandala,
    },
}
