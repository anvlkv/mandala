use crate::{Angle, GlVec, Point, Vector, VectorValuedFn};

/// sweeps an arc of radius with center, start and sweep angles
#[derive(Debug, Clone, Copy)]
pub struct SweepArc {
    pub radius: Vector,
    pub center: Point,
    pub start_angle: Angle,
    pub sweep_angle: Angle,
}

impl Default for SweepArc {
    fn default() -> Self {
        Self {
            radius: GlVec::default().into(),
            center: GlVec::default().into(),
            start_angle: Angle::default(),
            sweep_angle: Angle::default(),
        }
    }
}

impl VectorValuedFn for SweepArc {
    fn eval(&self, t: crate::Float) -> Vector {
        let angle = self.start_angle + self.sweep_angle * t;

        crate::Vector {
            x: self.center.x + self.radius.x * angle.cos(),
            y: self.center.y + self.radius.y * angle.sin(),
            #[cfg(feature = "3d")]
            z: self.center.z + self.radius.z * angle.sin(),
        }
    }

    fn length(&self) -> crate::Float {
        self.radius.x.hypot(self.radius.y) * self.sweep_angle.to_radians()
    }
}

/// draws an arc between two points
#[derive(Debug, Clone, Copy)]
pub struct ArcSegment {
    pub start: Point,
    pub end: Point,
    pub radius: Vector,
    /// draws largest of two arcs
    pub large_arc: bool,
    /// draws arc in the direction of increasing angle
    pub poz_angle: bool,
}

impl Default for ArcSegment {
    fn default() -> Self {
        Self {
            start: GlVec::default().into(),
            end: GlVec::default().into(),
            radius: GlVec::default().into(),
            large_arc: false,
            poz_angle: false,
        }
    }
}

impl ArcSegment {
    /// finds center of the arc based on `large_arc`
    /// and `poz_angle` flags
    pub fn arc_center(&self) -> Point {
        let mid_point = crate::Vector {
            x: (self.start.x + self.end.x) / 2.0,
            y: (self.start.y + self.end.y) / 2.0,
            #[cfg(feature = "3d")]
            z: (self.start.z + self.end.z) / 2.0,
        };

        let start_to_end: GlVec = crate::Vector {
            x: self.end.x - self.start.x,
            y: self.end.y - self.start.y,
            #[cfg(feature = "3d")]
            z: self.end.z - self.start.z,
        }
        .into();

        let start_to_mid: GlVec = crate::Vector {
            x: mid_point.x - self.start.x,
            y: mid_point.y - self.start.y,
            #[cfg(feature = "3d")]
            z: mid_point.z - self.start.z,
        }
        .into();

        let mut angle = start_to_end.angle_between(start_to_mid);

        if self.large_arc {
            angle = if self.poz_angle { angle } else { -angle };
        } else {
            angle = if self.poz_angle { -angle } else { angle };
        }

        let center_x = self.start.x + self.radius.x * angle.cos();
        let center_y = self.start.y + self.radius.y * angle.sin();
        #[cfg(feature = "3d")]
        let center_z = self.start.z + self.radius.z * angle.sin();

        crate::Vector {
            x: center_x,
            y: center_y,
            #[cfg(feature = "3d")]
            z: center_z,
        }
        .into()
    }
}

impl VectorValuedFn for ArcSegment {
    fn eval(&self, t: crate::Float) -> Vector {
        let center = self.arc_center();
        let start_angle = Angle::from_radians(
            (GlVec::from(self.end) - GlVec::from(self.start))
                .angle_between(GlVec::from(self.radius)),
        );

        let sweep_angle = if self.large_arc {
            Angle::PI
        } else {
            Angle::FRAC_PI_2
        };

        let angle = start_angle + sweep_angle * t;

        crate::Vector {
            x: center.x + self.radius.x * angle.cos(),
            y: center.y + self.radius.y * angle.sin(),
            #[cfg(feature = "3d")]
            z: center.z + self.radius.z * angle.sin(),
        }
    }

    fn length(&self) -> crate::Float {
        let start_angle = Angle::from_radians(
            (GlVec::from(self.end) - GlVec::from(self.start))
                .angle_between(GlVec::from(self.radius)),
        );

        let sweep_angle = if self.large_arc {
            Angle::PI
        } else {
            Angle::FRAC_PI_2
        };

        (start_angle + sweep_angle).to_radians() * self.radius.x.hypot(self.radius.y)
    }
}

#[cfg(test)]
mod arc_tests {
    use super::*;
    use crate::test_util::test_name;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_sweep_arc() {
        let arc = SweepArc {
            radius: Vector {
                x: 10.0,
                y: 10.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            center: Point {
                x: 0.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            start_angle: Angle::from_degrees(0.0),
            sweep_angle: Angle::from_degrees(90.0),
        };
        let points: Vec<_> = arc.sample_evenly(10);
        assert_debug_snapshot!(test_name("sweep-arc"), points);
    }

    #[test]
    fn test_arc_segment() {
        let arc = ArcSegment {
            start: Point {
                x: 0.0,
                y: 10.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            end: Point {
                x: 10.0,
                y: 0.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            radius: Vector {
                x: 10.0,
                y: 10.0,
                #[cfg(feature = "3d")]
                z: 0.0,
            },
            large_arc: true,
            poz_angle: true,
        };
        let points: Vec<_> = arc.sample_evenly(10);
        assert_debug_snapshot!(test_name("segment-arc"), points);
    }
}
