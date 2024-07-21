use derive_builder::Builder;
use uuid::Uuid;

use crate::{Angle, Float, Mandala, Path, Point};

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
    ///
    /// -50.0 along the circumference axis matches the leftmost position, 50.0 is the rightmost
    ///
    /// 0.0 along the radius axis matches the edge of the inner circle, 100.0 is the outter circle
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
    Mandala(Mandala),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Builder)]
pub struct ReplicaSegment {
    /// id of this segment
    #[builder(default = "uuid::Uuid::new_v4()")]
    pub id: Uuid,
    /// id of the segment to replicate
    pub replica_id: Uuid,
    /// adjusted angle of the replicated segment
    pub angle_base: Angle,
}

impl ReplicaSegment {
    /// render all paths of the original with the angle_base of the replica
    pub fn render(&self, original: &MandalaSegment) -> Vec<Path> {
        let diff = self.angle_base - original.angle_base;
        let drawing = original.render();
        drawing.into_iter().map(|r| r.rotate(diff)).collect()
    }
}

impl MandalaSegment {
    /// creates a replica of the segment
    pub fn replicate(&self, angle: Angle) -> ReplicaSegment {
        ReplicaSegmentBuilder::default()
            .replica_id(self.id)
            .angle_base(angle)
            .build()
            .expect("build replica")
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
        let mut rendition = vec![];

        for d in self.drawing.iter() {
            match d {
                SegmentDrawing::Path(paths) => {
                    for p in paths.iter() {
                        let mut path = p.clone();
                        for pt in path.key_pts() {
                            let (x, y) = self.to_global(pt.x, pt.y);
                            pt.x = x;
                            pt.y = y;
                        }

                        rendition.push(path)
                    }
                }
                SegmentDrawing::Mandala(m) => rendition.extend(m.render()),
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
            Point::new(3.898441400170306, 4.487846495580896),
            "from point"
        );
        assert_eq!(
            rendered_path.to(),
            Point::new(3.909476572005575, 4.523711154386932),
            "to point"
        );
    }
}
