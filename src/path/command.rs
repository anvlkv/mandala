use crate::{Angle, Point, Vector};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    /// draw to Point
    To(PathCommandOp<Point>),
    /// draw by Vector
    By(PathCommandOp<Vector>),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommandOp<Pv> {
    Move(Pv),
    Line(Pv),
    CubicCurve {
        to: Pv,
        ctrl1: Pv,
        ctrl2: Pv,
    },
    QudraticCurve {
        to: Pv,
        ctrl: Pv,
    },
    Arc {
        to: Pv,
        radii: Vector,
        x_rotation: Angle,
        large_arc: bool,
        sweep: bool,
    },
    ClosePath,
}
