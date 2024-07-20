use euclid::default::{Point2D, Rect};

use crate::Float;

pub fn rand_pt_in_bounds<R>(rng: &mut R, bounds: Rect<Float>) -> Point2D<Float>
where
    R: rand::Rng,
{
    let x = if bounds.x_range().is_empty() {
        bounds.max_x()
    } else {
        rng.gen_range(bounds.x_range())
    };

    let y = if bounds.y_range().is_empty() {
        bounds.max_y()
    } else {
        rng.gen_range(bounds.y_range())
    };

    Point2D::new(x, y)
}
