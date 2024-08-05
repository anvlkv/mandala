use std::ops::Range;

use cfg_if::cfg_if;

use crate::{Float, GlVec, Vector};

/// the heart and soul of the `mandala`
///
/// all paths and transformations are defined as nested `VectorValueFn`
pub trait VectorValuedFn {
    /// evaluates the `VectorValuedFn` at `t` where `t` is between 0 and 1
    ///
    /// must return Vector2 or Vector3 depending on `2d` or `3d` feature
    fn eval(&self, t: Float) -> Vector;

    /// computes the length of a segment
    fn length(&self) -> Float;

    /// Sample the function over a range of `t` values
    /// returning a collection of points
    fn sample_range(&self, range: Range<Float>, num_samples: usize) -> Vec<Vector> {
        (0..num_samples)
            .map(move |i| {
                let t = range.start
                    + (range.end - range.start) * (i as Float / (num_samples - 1) as Float);
                self.eval(t)
            })
            .collect()
    }

    /// Sample the function evenly from 0 to 1,
    /// useful for generating a uniform set of points
    /// along the path.
    fn sample_evenly(&self, num_samples: usize) -> Vec<Vector> {
        self.sample_range(0.0..1.0, num_samples)
    }

    /// Sample the function evenly
    /// from 0 to 1 with optimal increment of `t`,
    /// useful for generating a uniform set of points
    /// along the path
    fn sample_optimal(&self) -> Vec<Vector> {
        let t_step = self.optimized_t_step();
        let num_samples = (1.0 / t_step).ceil() as usize;
        self.sample_evenly(num_samples)
    }

    /// Compute the derivative of the function,
    /// which can be useful for determining tangents, normals, and curvature.
    fn derivative(&self, t: Float) -> Vector {
        let h = Float::EPSILON.powi(2);
        let t1 = t + h;
        let t2 = t - h;
        let p1: GlVec = self.eval(t1).into();
        let p2: GlVec = self.eval(t2).into();
        let d = p1 - p2;

        (d / (2.0 * h)).into()
    }

    /// Compute the normal vector at a given `t` value.
    fn normal(&self, t: Float) -> Vector {
        let d: GlVec = self.derivative(t).into();
        cfg_if! {
            if #[cfg(feature = "3d")] {
                let magnitude = magnitude(d);
                let normalized = d / magnitude;
                // Use a consistent reference vector for cross product
                let ref_vec = GlVec::new(0.0, 0.0, 1.0); // Assuming z-axis as reference
                let cross_product = normalized.cross(ref_vec);
                let normal_magnitude = (cross_product.x * cross_product.x + cross_product.y * cross_product.y + cross_product.z * cross_product.z).sqrt();
                let normal = cross_product / normal_magnitude;
                Vector {
                    x: normal.x,
                    y: normal.y,
                    z: normal.z,
                }
            } else {
                let magnitude = magnitude(d);
                let normalized = d / magnitude;
                Vector {
                    x: -normalized.y,
                    y: normalized.x,
                }
            }
        }
    }

    /// finds optimal (error-free yet efficint) step for the `t` increment
    fn optimized_t_step(&self) -> Float {
        let length = self.length();
        1.0 / length
    }
}

pub(crate) fn magnitude(d: GlVec) -> Float {
    cfg_if! {
        if #[cfg(feature="3d")] {
            (d.x * d.x + d.y * d.y + d.z * d.z).sqrt()
        }
        else {
            (d.x * d.x + d.y * d.y).sqrt()
        }
    }
}
