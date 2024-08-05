mod angle;
mod paths;
mod primitives;
mod transform;
mod vector_valued;

pub use angle::*;
pub use paths::*;
pub use primitives::*;
pub use transform::*;
pub use vector_valued::*;

#[cfg(test)]
pub(crate) mod test_util {
    #[cfg(all(feature = "f64", feature = "3d"))]
    const FEAT: &str = "f64-3d";
    #[cfg(all(feature = "f32", feature = "3d"))]
    const FEAT: &str = "f32-3d";
    #[cfg(all(feature = "f64", feature = "2d"))]
    const FEAT: &str = "f64-2d";
    #[cfg(all(feature = "f32", feature = "2d"))]
    const FEAT: &str = "f32-2d";

    pub fn test_name(name: &str) -> String {
        format!("{FEAT}-{name}")
    }
}
