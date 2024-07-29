mod artboard;
mod epoch;
mod generator;
mod mandala;
mod path;
mod segment;

#[cfg(all(feature = "f64", feature = "f32"))]
compile_error!("only one feature at a time is allowed use 'f64' or 'f32'");

#[cfg(not(any(feature = "f64", feature = "f32")))]
compile_error!("at least one feature must be enabled 'f64' or 'f32'");

#[cfg(all(feature = "2d", feature = "3d"))]
compile_error!("only one feature at a time is allowed use '2d' or '3d'");

#[cfg(not(any(feature = "2d", feature = "3d")))]
compile_error!("at least one feature must be enabled '2d' or '3d'");

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

    use euclid::default::{Box2D, Point2D, Rect as Rect2D, Size2D, Vector2D};
    use euclid::Angle as Angle2D;

    pub type Point = Point2D<Float>;
    pub type Size = Size2D<Float>;
    pub type Vector = Vector2D<Float>;
    pub type BBox = Box2D<Float>;
    pub type Rect = Rect2D<Float>;
    pub type Angle = Angle2D<Float>;

    #[cfg(feature = "3d")]
    mod three_d {
        use super::Float;
        use euclid::default::{Box3D, Point3D, Size3D, Vector3D};

        pub type Point3d = Point3D<Float>;
        pub type Size3d = Size3D<Float>;
        pub type Vector3d = Vector3D<Float>;
        pub type BBox3d = Box3D<Float>;
    }

    #[cfg(feature = "3d")]
    pub use three_d::*;

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
