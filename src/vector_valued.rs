use std::ops::RangeBounds;

use cfg_if::cfg_if;

use crate::{Float, GlVec, Vector};

/// the heart and soul of the `mandala`
///
/// all paths and transformations are defined as `VectorValueFn`
pub trait VectorValuedFn {
    /// evaluates the `VectorValuedFn` at `t` where `t` is between 0 and 1
    ///
    /// must return Vector2 or Vector3 depending on `2d` or `3d` feature
    fn eval(&self, t: Float) -> Vector;

    /// shader code equivalent of this struct
    ///
    /// must return `vec2` or `vec3` depending on `2d` or `3d` feature
    fn to_shader_code(&self) -> naga::Module;

    /// Sample the function over a range of `t` values
    /// returning a collection of points
    fn sample_range<R>(&self, range: R, num_samples: usize) -> impl Iterator<Item = Vector>
    where
        R: RangeBounds<Float>,
    {
        (0..num_samples).map(move |i| {
            let t = match range.start_bound() {
                std::ops::Bound::Included(start) => *start,
                std::ops::Bound::Excluded(start) => *start,
                std::ops::Bound::Unbounded => 0.0,
            } + (match range.end_bound() {
                std::ops::Bound::Included(end) => *end,
                std::ops::Bound::Excluded(end) => *end,
                std::ops::Bound::Unbounded => 1.0,
            } - match range.start_bound() {
                std::ops::Bound::Included(start) => *start,
                std::ops::Bound::Excluded(start) => *start,
                std::ops::Bound::Unbounded => 0.0,
            }) * (i as Float / (num_samples - 1) as Float);
            self.eval(t)
        })
    }

    /// Sample the function evenly from 0 to 1,
    /// useful for generating a uniform set of points
    /// along the path.
    fn sample_evenly(&self, num_samples: usize) -> impl Iterator<Item = Vector> {
        self.sample_range(0.0..1.0, num_samples)
    }

    /// Compute the derivative of the function,
    /// which can be useful for determining tangents, normals, and curvature.
    fn derivative(&self, t: Float) -> Vector {
        let h = Float::EPSILON.powi(2);
        let t1 = t + h;
        let t2 = t - h;
        let p1: Vector = self.eval(t1).into();
        let p2: Vector = self.eval(t2).into();
        let d = {
            cfg_if! {
                if #[cfg(feature="3d")] {
                    GlVec::from((p1.x, p1.y, p1.z)) - GlVec::from((p2.x, p2.y, p2.z))
                } else {
                    GlVec::from((p1.x, p1.y)) - GlVec::from((p2.x, p2.y))
                }
            }
        };

        (d / (2.0 * h)).into()
    }

    /// Compute the normal vector at a given `t` value.
    fn normal(&self, t: Float) -> Vector {
        let d: GlVec = self.derivative(t).into();
        cfg_if! {
            if #[cfg(feature = "3d")] {
                let magnitude = (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
                let normalized = d / magnitude;
                Vector {
                    x: -normalized.y,
                    y: normalized.x,
                    z: normalized.z,
                }
            } else {
                let magnitude = (d.x * d.x + d.y * d.y).sqrt();
                let normalized = d / magnitude;
                Vector {
                    x: -normalized.y,
                    y: normalized.x,
                }
            }
        }
    }
}
