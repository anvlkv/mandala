use std::ops::{Add, AddAssign};

use derive_builder::Builder;

use rand::prelude::*;

use crate::{Angle, Float, Line, Path, PathSegment, Point, Rect, Size, Vector};

pub fn rand_pt_in_bounds<R>(rng: &mut R, bounds: Rect) -> Point
where
    R: Rng,
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

    Point::new(x, y)
}

pub fn polygon(sides: u8, bounds: Rect) -> Path {
    let center = bounds.center();
    let radius = bounds.width().min(bounds.height()) / 2.0;
    let angle_step = 2.0 * std::f64::consts::PI / sides as f64;

    let mut path = Path::default();
    for i in 0..sides {
        let angle = i as f64 * angle_step;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        let point = Point::new(x, y);

        if i == 0 {
            path.move_to(point);
        } else {
            path.draw_next(|last| {
                PathSegment::Line(Line {
                    from: last.to(),
                    to: point,
                })
            });
        }
    }

    path
}

/// Fill pattern generator
#[derive(Debug, Clone, Builder)]
pub struct Generator<F, R>
where
    F: Fn(&mut R, Size) -> Path + Clone + Copy + 'static,
    R: Rng + SeedableRng,
{
    /// Fill mode
    pub mode: GeneratorMode,
    /// Pattern renderrer
    pub renderer: F,
    /// randomness generator
    #[builder(default = "R::from_entropy()")]
    pub rng: R,
    /// transform each generated path
    #[builder(default, setter(each(name = "transform")))]
    pub transformations: Vec<Transform>,
}

#[derive(Debug, Clone)]
pub enum Transform {
    Scale(FillValue<Float>),
    Rotate(FillValue<Angle>),
    Translate(FillValue<Vector>),
}

#[derive(Debug, Clone)]
pub enum FillValue<T>
where
    T: Clone + Copy + Add<Output = T> + AddAssign,
{
    /// apply same value to every fill step
    Static(T),
    /// increment the initial value by T*n
    Incremental { init: T, increment: T },
    /// apply varying values, start over if the number of values needed is greater than vec length
    Varying(Vec<T>),
    /// apply random value
    Rand(Vec<T>),
}

impl<T> FillValue<T>
where
    T: Clone + Copy + Add<Output = T> + AddAssign,
{
    pub fn value_at<R>(&self, i: usize, rng: &mut R) -> T
    where
        R: Rng,
    {
        match self {
            FillValue::Static(v) => *v,
            FillValue::Incremental { init, increment } => {
                let mut val = *init;
                for _ in 0..i {
                    val += *increment
                }
                val
            }
            FillValue::Varying(opts) => *opts
                .iter()
                .cycle()
                .nth(i)
                .expect("is the varying value empty?"),
            FillValue::Rand(opts) => {
                *SliceRandom::choose(opts.as_slice(), rng).expect("is the varying value empty?")
            }
        }
    }
}

/// Fill modes
#[derive(Debug, Clone)]
pub enum GeneratorMode {
    /// repeat every N units along X axis
    XStep(Float),
    /// repeat every N units along Y axis
    YStep(Float),
    /// repeat every N units along XY axis (diagonal)
    XYStep { x: Float, y: Float },
    /// fill the grid
    GridStep {
        row_height: Float,
        column_width: Float,
    },
}

impl GeneratorMode {
    /// create an iterator for the given bounds
    pub fn bounds_iter(&self, bounds: Rect) -> Box<dyn Iterator<Item = Rect> + '_> {
        match self {
            GeneratorMode::XStep(step) => {
                let mut x = bounds.min_x();
                Box::new(std::iter::from_fn(move || {
                    if x < bounds.max_x() {
                        let rect = Rect::new(
                            Point::new(x, bounds.min_y()),
                            Size::new(*step, bounds.height()),
                        );
                        x += step;
                        Some(rect)
                    } else {
                        None
                    }
                }))
            }
            GeneratorMode::YStep(step) => {
                let mut y = bounds.min_y();
                Box::new(std::iter::from_fn(move || {
                    if y < bounds.max_y() {
                        let rect = Rect::new(
                            Point::new(bounds.min_x(), y),
                            Size::new(bounds.width(), *step),
                        );
                        y += step;
                        Some(rect)
                    } else {
                        None
                    }
                }))
            }
            GeneratorMode::XYStep {
                x: x_step,
                y: y_step,
            } => {
                let mut x = bounds.min_x();
                let mut y = bounds.min_y();
                Box::new(std::iter::from_fn(move || {
                    if x < bounds.max_x() && y < bounds.max_y() {
                        let rect = Rect::new(Point::new(x, y), Size::new(*x_step, *y_step));
                        x += x_step;
                        y += y_step;
                        Some(rect)
                    } else {
                        None
                    }
                }))
            }
            GeneratorMode::GridStep {
                row_height,
                column_width,
            } => {
                let mut x = bounds.min_x();
                let mut y = bounds.min_y();
                Box::new(std::iter::from_fn(move || {
                    if x < bounds.max_x() && y < bounds.max_y() {
                        let rect =
                            Rect::new(Point::new(x, y), Size::new(*column_width, *row_height));
                        x += column_width;
                        if x >= bounds.max_x() {
                            x = bounds.min_x();
                            y += row_height;
                        }
                        Some(rect)
                    } else {
                        None
                    }
                }))
            }
        }
    }
}

impl<F, R> Generator<F, R>
where
    F: Fn(&mut R, Size) -> Path + Clone + Copy + 'static,
    R: Rng + SeedableRng,
{
    /// runs generation
    pub fn generate(&mut self, gen_bounds: Rect) -> Vec<Path> {
        let mut it = self.mode.bounds_iter(gen_bounds).enumerate();
        let mut result = vec![];
        let render_fn = self.renderer;

        let rng = &mut self.rng;

        while let Some((i, rect)) = it.next() {
            let mut path = render_fn(rng, rect.size);
            for transofrm in self.transformations.iter() {
                match transofrm {
                    Transform::Scale(value) => {
                        let scale = value.value_at(i, rng);
                        path = path.scale(scale);
                    }
                    Transform::Rotate(value) => {
                        let angle = value.value_at(i, rng);
                        path = path.rotate(angle);
                    }
                    Transform::Translate(value) => {
                        let by = value.value_at(i, rng);
                        path = path.translate(by);
                    }
                }
            }
            result.push(path.translate(Vector::new(rect.origin.x, rect.origin.y)));
        }

        result
    }
}

#[cfg(test)]
mod generator_test {
    use lyon_geom::LineSegment;

    use crate::{PathSegment, Size};

    use super::*;

    #[test]
    fn test_rand_pt_in_bounds() {
        let mut rng = rand::thread_rng();
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        let pt = rand_pt_in_bounds(&mut rng, bounds);
        assert!(pt.x >= 0.0 && pt.x <= 10.0);
        assert!(pt.y >= 0.0 && pt.y <= 10.0);
    }

    #[test]
    fn test_polygon_generator() {
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));
        let path = polygon(4, bounds);
        assert_eq!(path.into_iter().len(), 4);
    }

    #[test]
    fn test_fill_value_static() {
        let value = FillValue::Static(5.0);
        let mut rng = rand::thread_rng();
        assert_eq!(value.value_at(0, &mut rng), 5.0);
        assert_eq!(value.value_at(10, &mut rng), 5.0);
    }

    #[test]
    fn test_fill_value_incremental() {
        let value = FillValue::Incremental {
            init: 1.0,
            increment: 2.0,
        };
        let mut rng = rand::thread_rng();
        assert_eq!(value.value_at(0, &mut rng), 1.0);
        assert_eq!(value.value_at(1, &mut rng), 3.0);
        assert_eq!(value.value_at(2, &mut rng), 5.0);
    }

    #[test]
    fn test_fill_value_varying() {
        let value = FillValue::Varying(vec![1.0, 2.0, 3.0]);
        let mut rng = rand::thread_rng();
        assert_eq!(value.value_at(0, &mut rng), 1.0);
        assert_eq!(value.value_at(1, &mut rng), 2.0);
        assert_eq!(value.value_at(2, &mut rng), 3.0);
        assert_eq!(value.value_at(3, &mut rng), 1.0);
    }

    #[test]
    fn test_fill_value_rand() {
        let value = FillValue::Rand(vec![1.0, 2.0, 3.0]);
        let mut rng = rand::thread_rng();
        let val = value.value_at(0, &mut rng);
        assert!(val == 1.0 || val == 2.0 || val == 3.0);
    }

    #[test]
    fn test_generator_mode_x_step() {
        let mode = GeneratorMode::XStep(10.0);
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(30.0, 20.0));
        let mut iter = mode.bounds_iter(bounds);
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 20.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(10.0, 0.0), Size::new(10.0, 20.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(20.0, 0.0), Size::new(10.0, 20.0))
        );
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_generator_mode_y_step() {
        let mode = GeneratorMode::YStep(10.0);
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 30.0));
        let mut iter = mode.bounds_iter(bounds);
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(0.0, 10.0), Size::new(20.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(0.0, 20.0), Size::new(20.0, 10.0))
        );
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_generator_mode_xy_step() {
        let mode = GeneratorMode::XYStep { x: 10.0, y: 10.0 };
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(30.0, 30.0));
        let mut iter = mode.bounds_iter(bounds);
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(10.0, 10.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(20.0, 20.0), Size::new(10.0, 10.0))
        );
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_generator_mode_grid_step() {
        let mode = GeneratorMode::GridStep {
            row_height: 10.0,
            column_width: 10.0,
        };
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(30.0, 30.0));
        let mut iter = mode.bounds_iter(bounds);
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(10.0, 0.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(20.0, 0.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(0.0, 10.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(10.0, 10.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(20.0, 10.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(0.0, 20.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(10.0, 20.0), Size::new(10.0, 10.0))
        );
        assert_eq!(
            iter.next().unwrap(),
            Rect::new(Point::new(20.0, 20.0), Size::new(10.0, 10.0))
        );
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_generator_simple_line() {
        use rand::rngs::SmallRng;

        let rng = SmallRng::seed_from_u64(64);
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0));

        let renderer = |_rng: &mut SmallRng, _| {
            Path::new(PathSegment::Line(LineSegment {
                from: Point::new(0.0, 0.0),
                to: Point::new(1.0, 1.0),
            }))
        };

        let mut generator = GeneratorBuilder::default()
            .mode(GeneratorMode::XStep(1.0))
            .renderer(renderer)
            .rng(rng)
            .build()
            .unwrap();

        let paths = generator.generate(bounds);

        assert_eq!(paths.len(), 10);
    }
}
