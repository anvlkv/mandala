use derive_builder::Builder;
use uuid::Uuid;

use crate::{
    segment::{MandalaSegment, ReplicaSegment},
    Float, Path, Point, Size,
};

/// Mandala Epoch
///
/// lays out segments of [mandala::Mandala]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Builder, Clone)]
pub struct Epoch {
    /// id of the epoch
    #[builder(default = "uuid::Uuid::new_v4()")]
    pub id: Uuid,
    /// center of the epoch
    pub center: Point,
    /// layout mode of the epoch
    pub layout: EpochLayout,
    /// content of the epoch
    #[builder(default)]
    pub segments: Vec<EpochSegment>,
    /// whether the epoch should render its outline
    #[builder(default)]
    pub outline: bool,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum EpochSegment {
    /// Original segment
    Segment(MandalaSegment),
    /// Replica
    Replica(ReplicaSegment),
}

/// Epoch layout variants
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum EpochLayout {
    /// plain circular layout
    ///
    /// places each segment by simply rotating it
    Circle { radius: Float },
    /// elliptic layout
    ///
    /// places each segment by rotating it,
    /// performs additional scaling to match the size
    Ellipse { radii: Size },
    /// ploygonal layout
    ///
    /// places each segment along the edges of the polygon
    Polygon { n_sides: u8, radius: Float },
    /// rectangular layout
    ///
    /// places each segment along the edges of the rectangle
    Rectangle { rect: Size },
    /// grid layout
    ///
    /// places each segment in a cell of the grid
    ///
    /// unlike all the other layouts grid doesn't scale or rotate segments
    Grid { rows: u8, columns: u8, rect: Size },
}

#[derive(Debug, Clone)]
pub struct DrawArgs {
    /// segment number
    ///
    /// 1-based
    pub n: usize,
}

impl Epoch {
    pub fn draw_segment<D>(&mut self, draw_fn: &mut D)
    where
        D: FnMut(&DrawArgs) -> EpochSegment,
    {
    }

    /// renders all segments, all paths in global coordinates
    pub fn render(&self) -> Vec<Path> {
        self.segments
            .iter()
            .flat_map(|s| match s {
                EpochSegment::Segment(s) => s.render(),
                EpochSegment::Replica(r) => {
                    let original = self
                        .segments
                        .iter()
                        .find_map(|s| match s {
                            EpochSegment::Segment(s) => {
                                if s.id == r.replica_id {
                                    Some(s)
                                } else {
                                    None
                                }
                            }
                            EpochSegment::Replica(_) => None,
                        })
                        .expect("only same epoch replicas are supported; original not found");
                    r.render(original)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {}
