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

#[cfg(feature = "3d")]
pub type Vector = mint::Vector3<Float>;

#[cfg(feature = "2d")]
pub type Vector = mint::Vector2<Float>;

#[cfg(feature = "3d")]
pub type Point = mint::Point3<Float>;

#[cfg(feature = "2d")]
pub type Point = mint::Point2<Float>;

#[cfg(all(feature = "f64", feature = "3d"))]
pub type GlVec = glam::DVec3;

#[cfg(all(feature = "f32", feature = "3d"))]
pub type GlVec = glam::Vec3;

#[cfg(all(feature = "f64", feature = "2d"))]
pub type GlVec = glam::DVec2;

#[cfg(all(feature = "f32", feature = "2d"))]
pub type GlVec = glam::Vec2;

#[cfg(all(feature = "f64", feature = "3d"))]
pub type Affine = glam::DAffine3;

#[cfg(all(feature = "f32", feature = "3d"))]
pub type Affine = glam::Affine3A;

#[cfg(all(feature = "f64", feature = "2d"))]
pub type Affine = glam::DAffine2;

#[cfg(all(feature = "f32", feature = "2d"))]
pub type Affine = glam::Affine2;
