use std::collections::HashSet;

use euclid::default::{Box2D, Point2D, Size2D, Vector2D};
use ordered_float::OrderedFloat;
use rand::{
    random,
    rngs::SmallRng,
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};

use crate::{util::rand_pt_in_bounds, Epoch, EpochBuilder, Float, Path, SegmentRule};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mandala {
    bounds: Box2D<Float>,
    epochs: Vec<Epoch>,
    rng: SmallRng,
    displacements: HashSet<Point2D<OrderedFloat<Float>>>,
    pub drawing: Vec<Vec<Path>>,
}

impl Mandala {
    /// create new mandala with a size and random seed
    pub fn new(size: Float) -> Self {
        let seed: u64 = random();
        Self::seeded(seed, size)
    }

    /// create new mandala given a size and a seed
    pub fn seeded(seed: u64, size: Float) -> Self {
        let bounds = Box2D::from_size(Size2D::splat(size));
        let center = bounds.center();
        let radius = size / 2.0;
        let outer_circle = EpochBuilder::default()
            .center(center)
            .segments(0)
            .radius(radius)
            .breadth(0.0)
            .segment_rule(SegmentRule::None)
            .build()
            .expect("build outter circle epoch");

        let drawing = vec![outer_circle.render_paths()];

        let rng = SmallRng::seed_from_u64(seed);

        Self {
            bounds,
            drawing,
            rng,
            epochs: vec![outer_circle],
            displacements: Default::default(),
        }
    }

    /// Render all paths
    pub fn render_drawing(&mut self) -> Vec<Vec<Path>> {
        self.drawing = self.epochs.iter().map(|e| e.render_paths()).collect();
        self.drawing.clone()
    }

    /// Draw new epoch based on the last one
    pub fn draw_epoch<F>(&mut self, mut draw: F)
    where
        F: FnMut(&Epoch, &mut SmallRng) -> Epoch,
    {
        self.epochs
            .push(draw(self.epochs.last().unwrap(), &mut self.rng));
        let last = self.epochs.last().unwrap();
        self.displacements.extend(
            last.key_pts()
                .iter()
                .map(|pt| Point2D::new(OrderedFloat(pt.x), OrderedFloat(pt.y))),
        );
        self.drawing.push(last.render_paths());
    }

    /// Generates random epoch
    ///
    /// The new epoch will be either concentric with the mandala or use a new displacement depending on the available space.
    /// Consequent epochs will be either concentric with the last epoch or use a new displacement.
    pub fn generate_epoch(&mut self) {
        const GEN_SEGMENTS: const_primes::Primes<10> = const_primes::Primes::new();

        let alt_displacements = self.propose_epoch_displacements();
        let size = self.bounds.width();
        let bnd_rect = self.bounds.to_rect();

        self.draw_epoch(move |last, rng| {
            let space_next = last.space_next();
            let breadth_ratio =
                *(SliceRandom::choose(GEN_SEGMENTS.as_slice(), rng).unwrap()) as f64;

            let mut ep = EpochBuilder::default();

            if space_next.area() > size / 2.0 {
                let r = space_next.width() / 2.0;

                ep.center(last.center)
                    .radius(r)
                    .breadth((r * 2.0) / breadth_ratio);
            } else if let Some(c) = IteratorRandom::choose(alt_displacements.iter(), rng) {
                let r = rng.gen_range(space_next.width()..=size / 2.0);

                ep.center(Point2D::new(c.x.0, c.y.0))
                    .radius(r)
                    .breadth((r * 2.0) / breadth_ratio);
            } else {
                let r = size / 2.0;
                ep.center(rand_pt_in_bounds(rng, bnd_rect))
                    .radius(r)
                    .breadth((r * 2.0) / breadth_ratio);
            }

            let segments = *SliceRandom::choose(GEN_SEGMENTS.as_slice(), rng).unwrap() as usize;

            let mut ep = ep.segments(segments).build().expect("build epoch");

            ep.draw_segment(|min, max| {
                let bounds =
                    Box2D::new(min.min(), Point2D::new(min.max_x(), max.max_y())).to_rect();
                let symmetry = rng.gen_ratio(2, 3);
                let detail = rng.gen_range(1..=8);

                SegmentRule::generate(rng, |rng| Path::generate(rng, bounds, symmetry, detail))
            });

            ep
        })
    }

    /// Resize the mandala to fit the given size
    pub fn resize(&mut self, new_size: Float) {
        let new_bounds = Box2D::from_size(Size2D::splat(new_size));
        let old_center = self.bounds.center();
        debug_assert_eq!(
            self.bounds.width(),
            self.bounds.height(),
            "non square bounds"
        );
        let scale = new_size / self.bounds.width();

        for ep in self.epochs.iter_mut() {
            ep.scale(scale, old_center);
        }

        self.bounds = new_bounds;
        self.drawing = self.render_drawing();
    }

    /// Translate the mandala and all its contents
    pub fn translate(&mut self, by: Vector2D<Float>) {
        self.bounds = self.bounds.translate(by);
        for ep in self.epochs.iter_mut() {
            ep.translate(by);
        }

        self.drawing = self.render_drawing();
    }

    /// Finds intersections where there's no epoch's center yet
    pub fn propose_epoch_displacements(&self) -> HashSet<Point2D<OrderedFloat<Float>>> {
        let centers: HashSet<Point2D<OrderedFloat<Float>>> = HashSet::from_iter(
            self.epochs
                .iter()
                .map(|e| Point2D::new(OrderedFloat(e.center.x), OrderedFloat(e.center.y))),
        );

        self.displacements
            .iter()
            .filter(|p| !centers.contains(p))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use euclid::{Point2D, Vector2D};
    use lyon_geom::LineSegment;

    use crate::{Mandala, Path, Segment};

    #[test]
    fn test_draw_epoch() {
        let mut mandala = Mandala::new(200.0);
        mandala.draw_epoch(|epoch, _| {
            crate::EpochBuilder::default()
                .center(epoch.center)
                .segments(12)
                .radius(epoch.radius / 2.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::None)
                .build()
                .expect("build new epoch")
        });
        assert_eq!(mandala.epochs.len(), 2);
        let new_epoch = &mandala.epochs[1];
        assert_eq!(new_epoch.segments, 12);
        assert_eq!(new_epoch.radius, 50.0);
        assert_eq!(new_epoch.breadth, 5.0);
    }

    #[test]
    fn test_render_drawing() {
        let mut mandala = Mandala::new(200.0);
        mandala.draw_epoch(|epoch, _| {
            crate::EpochBuilder::default()
                .center(epoch.center)
                .segments(12)
                .radius(epoch.radius / 2.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::Path(Path::new(Segment::Line(
                    LineSegment {
                        from: Point2D::zero(),
                        to: Point2D::new(10.0, 10.0),
                    },
                ))))
                .build()
                .expect("build new epoch")
        });
        let drawing = mandala
            .render_drawing()
            .into_iter()
            .flat_map(|e| e.into_iter())
            .collect::<Vec<_>>();
        assert_eq!(drawing.len(), 14); // 1 outer circle + 1 circle + 12 segments
    }

    #[test]
    fn test_resize() {
        let mut mandala = Mandala::new(200.0);
        mandala.draw_epoch(|epoch, _| {
            crate::EpochBuilder::default()
                .center(epoch.center)
                .segments(12)
                .radius(epoch.radius / 2.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::None)
                .build()
                .expect("build new epoch")
        });
        mandala.resize(400.0);
        assert_eq!(mandala.bounds.width(), 400.0);
        assert_eq!(mandala.bounds.height(), 400.0);
        assert_eq!(mandala.epochs[0].radius, 200.0);
        assert_eq!(mandala.epochs[1].radius, 100.0);
        assert_eq!(mandala.epochs[1].breadth, 10.0);
    }

    #[test]
    fn test_translate() {
        let mut mandala = Mandala::new(200.0);
        let translation = Vector2D::new(100.0, 100.0);
        mandala.translate(translation);
        assert_eq!(mandala.bounds.min.x, 100.0);
        assert_eq!(mandala.bounds.min.y, 100.0);
        assert_eq!(mandala.bounds.max.x, 300.0);
        assert_eq!(mandala.bounds.max.y, 300.0);
    }

    #[test]
    fn new() {
        _ = Mandala::new(200.0);
    }

    #[test]
    fn test_propose_epoch_displacements() {
        let mut mandala = Mandala::new(200.0);
        mandala.draw_epoch(|epoch, _| {
            crate::EpochBuilder::default()
                .center(epoch.center)
                .segments(12)
                .radius(epoch.radius / 2.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::None)
                .build()
                .expect("build new epoch")
        });

        let displacements = mandala.propose_epoch_displacements();
        assert!(displacements.is_empty());
    }

    #[test]
    fn test_propose_epoch_displacements_with_intersections() {
        let mut mandala = Mandala::new(200.0);
        mandala.draw_epoch(|epoch, _| {
            crate::EpochBuilder::default()
                .center(epoch.center)
                .segments(12)
                .radius(epoch.radius / 2.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::None)
                .build()
                .expect("build new epoch")
        });

        // Add an epoch with a segment that intersects with the existing ones
        mandala.draw_epoch(|_, _| {
            crate::EpochBuilder::default()
                .center(Point2D::new(50.0, 50.0))
                .segments(1)
                .radius(48.0)
                .breadth(5.0)
                .segment_rule(crate::SegmentRule::Path(Path::new(Segment::Line(
                    LineSegment {
                        from: Point2D::new(100.0, 100.0),
                        to: Point2D::new(150.0, 150.0),
                    },
                ))))
                .build()
                .expect("build new epoch")
        });

        let displacements = mandala.propose_epoch_displacements();
        assert!(!displacements.is_empty());
    }

    #[test]
    fn test_generate_epoch() {
        let mut mandala = Mandala::seeded(42, 200.0);
        for _ in 0..5 {
            mandala.generate_epoch();
        }
        assert_eq!(mandala.epochs.len(), 6); // 1 initial + 5 generated
    }
}
