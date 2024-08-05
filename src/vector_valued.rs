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
        let num_samples = (1.0 / t_step).ceil().max(2.0) as usize;
        self.sample_evenly(num_samples)
    }

    /// Compute the derivative of the function,
    /// which can be useful for determining tangents, normals, and curvature.
    fn derivative(&self, t: Float) -> Vector {
        let h = Float::EPSILON;
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
        match d.try_normalize() {
            Some(n) => n.any_orthonormal_vector().into(),
            None => GlVec::default().into(),
        }
    }

    /// finds optimal (error-free yet efficint) step for the `t` increment
    fn optimized_t_step(&self) -> Float {
        let start: GlVec = self.eval(0.0).into();
        let mut t_step = 1.0;
        let max_err = Float::EPSILON * self.length();

        while t_step > Float::EPSILON {
            let mid_t = 0.5 * t_step;
            let mid_point: GlVec = self.eval(mid_t).into();
            let end_point: GlVec = self.eval(t_step).into();
            let linear_approx = start + (end_point - start) * mid_t;
            let error = magnitude(mid_point - linear_approx);

            if error < max_err {
                break;
            }

            t_step *= 0.5;
        }

        t_step
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
