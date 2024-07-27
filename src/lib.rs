mod artboard;
mod epoch;
mod generator;
mod mandala;
mod path;
mod segment;

#[cfg(feature = "f64")]
pub type Float = f64;

#[cfg(feature = "f32")]
pub type Float = f32;

pub use artboard::*;
pub use epoch::*;
pub use generator::*;
pub use mandala::*;
pub use path::*;
pub use segment::*;

mod points {
    use crate::Float;

    use euclid::{
        default::{Box2D, Point2D, Rect as Rect2D, Size2D, Vector2D},
        Angle as Angle2D,
    };

    pub type Point = Point2D<Float>;
    pub type Rect = Rect2D<Float>;
    pub type Size = Size2D<Float>;
    pub type Vector = Vector2D<Float>;
    pub type Angle = Angle2D<Float>;
    pub type BBox = Box2D<Float>;

    use lyon_geom::{
        Arc as Arc2D, CubicBezierSegment, LineSegment, QuadraticBezierSegment, SvgArc as SvgArc2D,
    };

    pub use lyon_geom::ArcFlags;

    pub type Arc = Arc2D<Float>;
    pub type Line = LineSegment<Float>;
    pub type QuadraticCurve = QuadraticBezierSegment<Float>;
    pub type CubicCurve = CubicBezierSegment<Float>;
    pub type SvgArc = SvgArc2D<Float>;
}

pub use points::*;
