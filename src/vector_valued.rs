use std::ops::Range;

use cfg_if::cfg_if;

use crate::{Float, GlVec, Point, Vector};

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

    /// start point
    fn start(&self) -> Point {
        self.eval(0.0).into()
    }

    /// end point
    fn end(&self) -> Point {
        self.eval(1.0).into()
    }

    /// mid point
    fn mid(&self) -> Point {
        self.eval(0.5).into()
    }

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
    /// optimizes the increment for every next step
    ///
    /// the default implementation is "universal" but does't promise the best performance
    fn sample_optimal(&self) -> Vec<Vector> {
        let mut points = Vec::new();

        if self.length() == 0.0 {
            return points;
        }

        let mut t = 0.0;
        let mut increment;

        let start_sample: GlVec = self.eval(0.0).into();
        let mid_sample: GlVec = self.eval(0.5).into();
        let end_sample: GlVec = self.eval(1.0).into();

        let start_to_mid = mid_sample - start_sample;
        let mid_to_end = end_sample - mid_sample;

        let start_to_mid_length = magnitude(start_to_mid);
        let mid_to_end_length = magnitude(mid_to_end);

        let tolerance = (start_to_mid_length + mid_to_end_length) * Float::EPSILON;

        while t < 1.0 {
            let derivative: GlVec = self.derivative(t).into();
            let length = magnitude(derivative);

            if length > tolerance {
                increment =
                    (0.1 / length).clamp(Float::EPSILON.powi(2), (1.0 - t).max(Float::EPSILON));
            } else {
                increment = tolerance;
            }

            points.push(self.eval(t));
            t += increment;

            if t > 1.0 {
                t = 1.0;
                points.push(self.eval(t));
                break;
            }
        }

        points
    }

    /// Compute the derivative of the function,
    /// which can be useful for determining tangents, normals, and curvature.
    fn derivative(&self, t: Float) -> Vector {
        let h = Float::EPSILON.powf(0.5);
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
            Some(n) => {
                #[cfg(feature = "3d")]
                return n.any_orthonormal_vector().into();
                #[cfg(feature = "2d")]
                return n.perp().into();
            }
            None => GlVec::default().into(),
        }
    }
}

#[allow(dead_code)]
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
