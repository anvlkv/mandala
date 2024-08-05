use cfg_if::cfg_if;

use crate::{Affine, GlVec, VectorValuedFn};

pub struct Transform<'v> {
    pub affine: Affine,
    pub source: &'v dyn VectorValuedFn,
}

impl<'v> VectorValuedFn for Transform<'v> {
    fn eval(&self, t: crate::Float) -> crate::Vector {
        let value = self.source.eval(t);
        cfg_if! {
            if #[cfg(feature = "3d")] {
                self.affine.transform_point3(value.into()).into()
            }
            else {
                self.affine.transform_point2(value.into()).into()
            }
        }
    }

    fn length(&self) -> crate::Float {
        let mut samples = self.sample_evenly(1000).into_iter().map(|v| GlVec::from(v));
        let mut length = 0.0;
        let mut prev = samples.next().unwrap();

        for point in samples {
            length += (point - prev).length();
            prev = point;
        }
        length
    }
}
