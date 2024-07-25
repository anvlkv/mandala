use derive_builder::Builder;
use uuid::Uuid;

use crate::{Angle, BBox, Float, Mandala, Path, Point};

/// radial segment
///
/// the drawing surface has two axes `r` and `c`;
/// `r` (radius) corresponds to `y` coordinate of [euclid::Point2D];
/// `c` (circumference) corresponds to `x` coordinate of [euclid::Point2D];
///
/// - `r` is from the inner to the outter edge, where 0 is the inner circle created computed from radius and breadth,
/// any **positive** number along `r` is towards the outter edge of this segment;
/// any **negative** number along `r` is towards the oposite edge
///
/// - `c` is along the length of the outter length of the segment,
/// where 0 matches the angle base,
/// any **positive** number along `c` is towards increased angle;
/// any **negative** number along `c` is towards decreased angle;
/// the `c` axis is **scaled** and has scale of 0 once it reaches the center of the circle;
///
///
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Builder, Default)]
pub struct MandalaSegment {
    /// id of the segment
    #[builder(default = "uuid::Uuid::new_v4()")]
    pub id: Uuid,
    /// inward distance from the outter edge of the segment
    /// from edge to center
    pub breadth: Float,
    /// radius of the segment
    /// from center to edge
    pub r_base: Float,
    /// angle of the main axis of the segment
    pub angle_base: Angle,
    /// angular "width" of the coordinate space of this segment
    pub sweep: Angle,
    /// center of the circle in global (x, y) coordinates
    pub center: Point,
    /// normalize local coordinates as fraction of corresponding radial dimension
    ///
    /// **Default** is 100.0
    #[builder(default = "100.0")]
    pub normalized: Float,
    /// the raw drawing of this segment.
    /// `x` is along the `c` axis of the segment
    /// `y` is along the `r` axis of the segment
    #[builder(default)]
    pub drawing: Vec<SegmentDrawing>,
}

/// the drawing of a segment
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum SegmentDrawing {
    /// plain drawing containing multiple path
    Path(Vec<Path>),
    /// nested [mandala::Mandala] drawing. Reintroduces coordinate system
    /// with a new center
    Mandala {
        mandala: Mandala,
        placement_box: BBox,
    },
}

impl MandalaSegment {
    /// creates a replica of the segment
    pub fn replicate(&self, angle: Angle) -> Self {
        Self {
            angle_base: angle,
            ..self.clone()
        }
    }

    /// converts the point from global (absolute) coordinates (x, y) to radial (local, normalized) coordinates (c, r)
    ///
    /// **important** not all coordinates can be recovered after conversion between global/local
    pub fn to_local(&self, x: Float, y: Float) -> (Float, Float) {
        let dx = x - self.center.x;
        let dy = y - self.center.y;
        let r = (dx * dx + dy * dy).sqrt();
        let theta = dy.atan2(dx);
        let c = (theta - self.angle_base.radians) / self.sweep.radians * self.normalized;
        let r_inner = self.r_base - self.breadth;
        let r_outer = self.r_base;
        let r_normalized = (r - r_inner) / (r_outer - r_inner) * self.normalized;
        (c, r_normalized)
    }

    /// compute the angle of a point relative to center
    pub fn to_angle(&self, x: Float, y: Float) -> Angle {
        let dx = x - self.center.x;
        let dy = y - self.center.y;
        Angle::radians(dy.atan2(dx))
    }

    /// converts the point from radial (local, normalized) coordinates (c, r) to global (absolute) (x, y)
    ///
    /// **important** not all coordinates can be recovered after conversion between global/local
    pub fn to_global(&self, c: Float, r: Float) -> (Float, Float) {
        let r_inner = self.r_base - self.breadth;
        let r_outer = self.r_base;
        let r_normalized = r / self.normalized * (r_outer - r_inner) + r_inner;
        let theta = self.angle_base.radians + c / self.normalized * self.sweep.radians;
        let x = self.center.x + r_normalized * theta.cos();
        let y = self.center.y + r_normalized * theta.sin();
        (x, y)
    }

    /// renders all path in global coordinates
    pub fn render(&self) -> Vec<Path> {
        self.render_with(|pt| {
            let (x, y) = self.to_global(pt.x, pt.y);
            Point::new(x, y)
        })
    }

    pub fn render_with<F>(&self, with_fn: F) -> Vec<Path>
    where
        F: Fn(&Point) -> Point,
    {
        let mut rendition = vec![];

        for d in self.drawing.iter() {
            match d {
                SegmentDrawing::Path(paths) => {
                    for p in paths.iter() {
                        let mut path = p.clone();
                        for pt in path.key_pts() {
                            *pt = with_fn(&pt);
                        }
                        rendition.push(path)
                    }
                }
                SegmentDrawing::Mandala {
                    mandala,
                    placement_box,
                } => {
                    let mut m_render = mandala.render();
                    // for path in m_render.iter_mut() {
                    //     for pt in path.key_pts() {
                    //         let diff = placement_box.center() - mandala.bounds.center();
                    //         *pt = with_fn(&(*pt + diff));
                    //     }
                    // }
                    rendition.extend(m_render);
                }
            }
        }

        rendition
    }
}

#[cfg(test)]
mod test_segement {
    use crate::{Line, PathSegment};

    use super::*;

    #[test]
    fn test_builder() {
        let segment = MandalaSegmentBuilder::default()
            .breadth(1.0)
            .r_base(2.0)
            .angle_base(Angle::radians(0.5))
            .sweep(Angle::pi())
            .center(Point::new(3.0, 4.0))
            .drawing(vec![SegmentDrawing::Path(vec![])])
            .build()
            .expect("build segment");

        assert_eq!(segment.breadth, 1.0);
        assert_eq!(segment.r_base, 2.0);
        assert_eq!(segment.angle_base, Angle::radians(0.5));
        assert_eq!(segment.center, Point::new(3.0, 4.0));
        match &segment.drawing[0] {
            SegmentDrawing::Path(paths) => assert_eq!(paths.len(), 0),
            _ => panic!("Unexpected drawing type"),
        }
    }

    #[test]
    fn test_conversion_methods() {
        let segment = MandalaSegmentBuilder::default()
            .breadth(1.0)
            .r_base(2.0)
            .angle_base(Angle::radians(0.5))
            .sweep(Angle::pi())
            .center(Point::new(3.0, 4.0))
            .build()
            .expect("build segment");

        let c = 5.0;
        let r = 6.0;

        let (global_x_from_c, global_y_from_r) = segment.to_global(c, r);
        let (local_c_from_x, local_r_from_y) = segment.to_local(global_x_from_c, global_y_from_r);

        assert_eq!(
            (global_x_from_c, global_y_from_r),
            (3.8392861498283923, 4.647455603656524),
            "global coordinates"
        );

        assert_eq!(
            (local_c_from_x.round(), local_r_from_y.round()),
            (c, r),
            "local coordinates"
        );

        let diff_x = (local_c_from_x - local_c_from_x.round()).abs();
        let diff_y = (local_r_from_y - local_r_from_y.round()).abs();

        assert!(diff_x <= 0.000001);
        assert!(diff_y <= 0.000001);
    }

    #[test]
    fn test_path_segment_rendering() {
        let path = Path::new(PathSegment::Line(Line {
            from: Point::new(1.0, 2.0),
            to: Point::new(3.0, 4.0),
        }));
        let segment = MandalaSegmentBuilder::default()
            .breadth(1.0)
            .r_base(2.0)
            .angle_base(Angle::radians(0.5))
            .sweep(Angle::pi())
            .center(Point::new(3.0, 4.0))
            .drawing(vec![SegmentDrawing::Path(vec![path])])
            .build()
            .expect("build segment");

        let rendered_paths = segment.render();
        assert_eq!(rendered_paths.len(), 1);
        let rendered_path = &rendered_paths[0];
        assert_eq!(
            rendered_path.from(),
            Point::new(3.8793322159386543, 4.51688959557377),
            "from point"
        );
        assert_eq!(
            rendered_path.to(),
            Point::new(3.8617126862418067, 4.582281071622571),
            "to point"
        );
    }

    #[test]
    fn test_to_angle() {
        let segment = MandalaSegmentBuilder::default()
            .breadth(1.0)
            .r_base(2.0)
            .angle_base(Angle::radians(0.0))
            .sweep(Angle::pi())
            .center(Point::new(0.0, 0.0))
            .build()
            .expect("build segment");

        let test_cases = vec![
            (1.0, 1.0, Angle::radians(std::f64::consts::FRAC_PI_4)),
            (-1.0, 1.0, Angle::radians(3.0 * std::f64::consts::FRAC_PI_4)),
            (1.0, -1.0, Angle::radians(-std::f64::consts::FRAC_PI_4)),
            (
                -1.0,
                -1.0,
                Angle::radians(-3.0 * std::f64::consts::FRAC_PI_4),
            ),
            (0.0, 1.0, Angle::radians(std::f64::consts::FRAC_PI_2)),
            (0.0, -1.0, Angle::radians(-std::f64::consts::FRAC_PI_2)),
            (1.0, 0.0, Angle::radians(0.0)),
            (-1.0, 0.0, Angle::radians(std::f64::consts::PI)),
        ];

        for (x, y, expected_angle) in test_cases {
            let angle = segment.to_angle(x, y);
            assert_eq!(angle, expected_angle, "for point ({}, {})", x, y);
        }
    }
}
