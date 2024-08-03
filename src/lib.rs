mod chord;
mod generator;
mod mandala;
mod path;
mod ray;

pub use chord::*;
pub use generator::*;
pub use mandala::*;
pub use path::*;
pub use ray::*;

#[cfg(all(feature = "f64", feature = "f32"))]
compile_error!("only one feature at a time is allowed use 'f64' or 'f32'");

#[cfg(not(any(feature = "f64", feature = "f32")))]
compile_error!("at least one feature must be enabled 'f64' or 'f32'");

#[cfg(feature = "f64")]
pub type Float = f64;

#[cfg(feature = "f32")]
pub type Float = f32;

mod points {
    use crate::Float;

    use euclid::default::Rect as Rect2D;
    use euclid::default::{Box2D, Point2D, Size2D, Vector2D};
    use euclid::Angle as Angle2D;

    pub type Rect = Rect2D<Float>;
    pub type Angle = Angle2D<Float>;
    pub type Point = Point2D<Float>;
    pub type Size = Size2D<Float>;
    pub type Vector = Vector2D<Float>;
    pub type BBox = Box2D<Float>;
    pub type Transform = lyon_geom::Transform<Float>;
}

pub use points::*;
