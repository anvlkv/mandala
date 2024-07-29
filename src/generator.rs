use std::ops::{Add, AddAssign};

use derive_builder::Builder;

use euclid::SideOffsets2D;
use rand::prelude::*;

use crate::{Angle, Float, Path, Point, Rect, Size, Vector};

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

/// Fill pattern generator
#[derive(Debug, Clone, Builder)]
pub struct Generator<F, R>
where
    F: Fn(&mut R, Size) -> Path + Clone + Copy + 'static,
    R: Rng + SeedableRng,
{
    /// Fill mode
    pub mode: GeneratorMode,
    /// Pattern renderer
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
    /// Render one block
    Block,
    /// repeat every N units along X axis
    XStep(Float),
    /// repeat every N units along Y axis
    YStep(Float),
    /// repeat every N units along XY axes (diagonal)
    XYStep { x: Float, y: Float },
    /// fill the grid
    GridStep {
        row_height: Float,
        column_width: Float,
    },
    /// symmetrical along X axis
    XSymmetry {
        mode: Box<GeneratorMode>,
        axis: Float,
    },
    /// symmetrical along Y axis
    YSymmetry {
        mode: Box<GeneratorMode>,
        axis: Float,
    },
}

impl GeneratorMode {
    /// create an iterator for the given bounds
    pub fn bounds_iter(&self, bounds: Rect) -> Box<dyn Iterator<Item = Rect> + '_> {
        match self {
            GeneratorMode::Block => {
                let mut b = Some(bounds);
                Box::new(std::iter::from_fn(move || b.take()))
            }
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
            GeneratorMode::XSymmetry { mode, axis } => {
                let mut off = SideOffsets2D::zero();
                off.top = *axis;
                let bounds = bounds.inner_rect(off);
                mode.bounds_iter(bounds)
            }
            GeneratorMode::YSymmetry { mode, axis } => {
                let mut off = SideOffsets2D::zero();
                off.left = *axis;
                let bounds = bounds.inner_rect(off);
                mode.bounds_iter(bounds)
            }
        }
    }

    fn handle_post_gen(&self, generated: Vec<Path>) -> Vec<Path> {
        match self {
            GeneratorMode::XSymmetry { axis, mode } => mode.handle_post_gen(
                generated
                    .iter()
                    .map(|p| p.flip_along_x(*axis))
                    .chain(generated.clone())
                    .collect(),
            ),
            GeneratorMode::YSymmetry { axis, mode } => mode.handle_post_gen(
                generated
                    .iter()
                    .map(|p| p.flip_along_y(*axis))
                    .chain(generated.clone())
                    .collect(),
            ),
            _ => generated,
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
        let gen_size_b = Rect::new(Point::zero(), gen_bounds.size);
        let it = self.mode.bounds_iter(gen_size_b).enumerate();
        let mut result = vec![];
        let render_fn = self.renderer;

        let rng = &mut self.rng;

        for (i, rect) in it {
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

        let by = gen_bounds.origin.to_vector();
        self.mode
            .handle_post_gen(result)
            .into_iter()
            .map(|p| p.translate(by))
            .collect()
    }
}

#[cfg(test)]
mod generator_tests {
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
        let expected_rects = [
            (0.0, 0.0, 10.0, 20.0),
            (10.0, 0.0, 10.0, 20.0),
            (20.0, 0.0, 10.0, 20.0),
        ];
        for &(x, y, width, height) in &expected_rects {
            assert_eq!(
                iter.next().unwrap(),
                Rect::new(Point::new(x, y), Size::new(width, height))
            );
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_generator_mode_y_step() {
        let mode = GeneratorMode::YStep(10.0);
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 30.0));
        let mut iter = mode.bounds_iter(bounds);
        let expected_rects = [
            (0.0, 0.0, 20.0, 10.0),
            (0.0, 10.0, 20.0, 10.0),
            (0.0, 20.0, 20.0, 10.0),
        ];
        for &(x, y, width, height) in &expected_rects {
            assert_eq!(
                iter.next().unwrap(),
                Rect::new(Point::new(x, y), Size::new(width, height))
            );
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_generator_mode_xy_step() {
        let mode = GeneratorMode::XYStep { x: 10.0, y: 10.0 };
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(30.0, 30.0));
        let mut iter = mode.bounds_iter(bounds);
        let expected_rects = [(0.0, 0.0), (10.0, 10.0), (20.0, 20.0)];
        for &(x, y) in &expected_rects {
            assert_eq!(
                iter.next().unwrap(),
                Rect::new(Point::new(x, y), Size::new(10.0, 10.0))
            );
        }
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
        let expected_rects = [
            (0.0, 0.0),
            (10.0, 0.0),
            (20.0, 0.0),
            (0.0, 10.0),
            (10.0, 10.0),
            (20.0, 10.0),
            (0.0, 20.0),
            (10.0, 20.0),
            (20.0, 20.0),
        ];
        for &(x, y) in &expected_rects {
            assert_eq!(
                iter.next().unwrap(),
                Rect::new(Point::new(x, y), Size::new(10.0, 10.0))
            );
        }
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

    #[test]
    fn test_generator_mode_x_symmetry() {
        let inner_mode = GeneratorMode::XStep(10.0);
        let mode = GeneratorMode::XSymmetry {
            mode: Box::new(inner_mode),
            axis: 15.0,
        };
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(30.0, 30.0));
        let mut iter = mode.bounds_iter(bounds);
        let expected_rects = [
            (0.0, 15.0, 10.0, 15.0),
            (10.0, 15.0, 10.0, 15.0),
            (20.0, 15.0, 10.0, 15.0),
        ];
        for &(x, y, width, height) in &expected_rects {
            assert_eq!(
                iter.next().unwrap(),
                Rect::new(Point::new(x, y), Size::new(width, height))
            );
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_generator_mode_y_symmetry() {
        let inner_mode = GeneratorMode::YStep(10.0);
        let mode = GeneratorMode::YSymmetry {
            mode: Box::new(inner_mode),
            axis: 15.0,
        };
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(30.0, 30.0));
        let mut iter = mode.bounds_iter(bounds);
        let expected_rects = [
            (15.0, 0.0, 15.0, 10.0),
            (15.0, 10.0, 15.0, 10.0),
            (15.0, 20.0, 15.0, 10.0),
        ];
        for &(x, y, width, height) in &expected_rects {
            assert_eq!(
                iter.next().unwrap(),
                Rect::new(Point::new(x, y), Size::new(width, height))
            );
        }
        assert!(iter.next().is_none());
    }
}
