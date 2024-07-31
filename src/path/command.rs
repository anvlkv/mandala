use crate::{Angle, Float, Point, Transform, Vector};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommand {
    /// draw to Point
    To(PathCommandOp<Point>),
    /// draw by Vector
    By(PathCommandOp<Vector>),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum PathCommandOp<Pv> {
    Move(Pv),
    Line(Pv),
    CubicCurve {
        to: Pv,
        ctrl1: Pv,
        ctrl2: Pv,
    },
    QudraticCurve {
        to: Pv,
        ctrl: Pv,
    },
    Arc {
        to: Pv,
        radii: Vector,
        x_rotation: Angle,
        large_arc: bool,
        sweep: bool,
    },
    ClosePath,
}

impl PathCommand {
    /// render path command as SVG path command
    pub fn to_svg_path_d(&self) -> String {
        match self {
            Self::To(PathCommandOp::Move(pt)) => format!("M {},{} ", pt.x, pt.y),
            Self::To(PathCommandOp::Line(pt)) => format!("L {},{} ", pt.x, pt.y),
            Self::To(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => format!(
                "C {},{} {},{} {},{} ",
                ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
            ),
            Self::To(PathCommandOp::QudraticCurve { to, ctrl }) => {
                format!("Q {},{} {},{} ", ctrl.x, ctrl.y, to.x, to.y)
            }
            Self::To(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => format!(
                "A {},{} {} {} {} {},{} ",
                radii.x,
                radii.y,
                x_rotation.to_degrees(),
                if *large_arc { 1 } else { 0 },
                if *sweep { 1 } else { 0 },
                to.x,
                to.y
            ),
            Self::To(PathCommandOp::ClosePath) => "Z ".to_string(),
            Self::By(PathCommandOp::Move(vec)) => format!("m {},{} ", vec.x, vec.y),
            Self::By(PathCommandOp::Line(vec)) => format!("l {},{} ", vec.x, vec.y),
            Self::By(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => format!(
                "c {},{} {},{} {},{} ",
                ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
            ),
            Self::By(PathCommandOp::QudraticCurve { to, ctrl }) => {
                format!("q {},{} {},{} ", ctrl.x, ctrl.y, to.x, to.y)
            }
            Self::By(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => format!(
                "a {},{} {} {} {} {},{} ",
                radii.x,
                radii.y,
                x_rotation.to_degrees(),
                if *large_arc { 1 } else { 0 },
                if *sweep { 1 } else { 0 },
                to.x,
                to.y
            ),
            Self::By(PathCommandOp::ClosePath) => "z ".to_string(),
        }
    }

    pub fn unwrap_arc(&self, from: Point) -> lyon_geom::SvgArc<Float> {
        match self {
            Self::To(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => lyon_geom::SvgArc {
                from,
                to: *to,
                radii: Vector::new(radii.x, radii.y),
                x_rotation: *x_rotation,
                flags: lyon_geom::ArcFlags {
                    large_arc: *large_arc,
                    sweep: *sweep,
                },
            },
            Self::By(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => lyon_geom::SvgArc {
                from: Point::new(from.x + to.x, from.y + to.y),
                to: Point::new(from.x + to.x * 2.0, from.y + to.y * 2.0),
                radii: Vector::new(radii.x, radii.y),
                x_rotation: *x_rotation,
                flags: lyon_geom::ArcFlags {
                    large_arc: *large_arc,
                    sweep: *sweep,
                },
            },
            _ => panic!("Not an Arc command"),
        }
    }

    pub fn unwrap_line(&self, from: Point) -> lyon_geom::LineSegment<Float> {
        match self {
            Self::To(PathCommandOp::Line(to)) => lyon_geom::LineSegment { from, to: *to },
            Self::By(PathCommandOp::Line(by)) => lyon_geom::LineSegment {
                from,
                to: Point::new(from.x + by.x, from.y + by.y),
            },
            _ => panic!("Not a Line command"),
        }
    }

    pub fn unwrap_cubic_curve(&self, from: Point) -> lyon_geom::CubicBezierSegment<Float> {
        match self {
            Self::To(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => {
                lyon_geom::CubicBezierSegment {
                    from,
                    to: *to,
                    ctrl1: *ctrl1,
                    ctrl2: *ctrl2,
                }
            }
            Self::By(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => {
                lyon_geom::CubicBezierSegment {
                    from,
                    to: Point::new(from.x + to.x, from.y + to.y),
                    ctrl1: Point::new(from.x + ctrl1.x, from.y + ctrl1.y),
                    ctrl2: Point::new(from.x + ctrl2.x, from.y + ctrl2.y),
                }
            }
            _ => panic!("Not a Cubic Curve command"),
        }
    }

    pub fn unwrap_quadratic_curve(&self, from: Point) -> lyon_geom::QuadraticBezierSegment<Float> {
        match self {
            Self::To(PathCommandOp::QudraticCurve { to, ctrl }) => {
                lyon_geom::QuadraticBezierSegment {
                    from,
                    to: *to,
                    ctrl: *ctrl,
                }
            }
            Self::By(PathCommandOp::QudraticCurve { to, ctrl }) => {
                lyon_geom::QuadraticBezierSegment {
                    from,
                    to: Point::new(from.x + to.x, from.y + to.y),
                    ctrl: Point::new(from.x + ctrl.x, from.y + ctrl.y),
                }
            }
            _ => panic!("Not a Quadratic Curve command"),
        }
    }

    pub fn is_arc(&self) -> bool {
        matches!(
            self,
            Self::To(PathCommandOp::Arc { .. }) | Self::By(PathCommandOp::Arc { .. })
        )
    }

    pub fn is_line(&self) -> bool {
        matches!(
            self,
            Self::To(PathCommandOp::Line(_)) | Self::By(PathCommandOp::Line(_))
        )
    }

    pub fn is_cubic_curve(&self) -> bool {
        matches!(
            self,
            Self::To(PathCommandOp::CubicCurve { .. }) | Self::By(PathCommandOp::CubicCurve { .. })
        )
    }

    pub fn is_quadratic_curve(&self) -> bool {
        matches!(
            self,
            Self::To(PathCommandOp::QudraticCurve { .. })
                | Self::By(PathCommandOp::QudraticCurve { .. })
        )
    }

    pub fn is_close(&self) -> bool {
        matches!(
            self,
            Self::To(PathCommandOp::ClosePath) | Self::By(PathCommandOp::ClosePath)
        )
    }

    pub fn length(&self, from: Point) -> Float {
        if self.is_line() {
            self.unwrap_line(from).length()
        } else if self.is_cubic_curve() {
            self.unwrap_cubic_curve(from)
                .approximate_length(lyon_geom::Scalar::epsilon_for(Float::EPSILON))
        } else if self.is_quadratic_curve() {
            self.unwrap_quadratic_curve(from).length()
        } else if self.is_arc() {
            let mut len = 0.0;
            self.unwrap_arc(from).for_each_quadratic_bezier(&mut |q| {
                len += q.length();
            });
            len
        } else {
            0.0
        }
    }

    pub fn to(&self, from: Point) -> Point {
        match self {
            Self::To(PathCommandOp::Move(to))
            | Self::To(PathCommandOp::Line(to))
            | Self::To(PathCommandOp::CubicCurve { to, .. })
            | Self::To(PathCommandOp::QudraticCurve { to, .. })
            | Self::To(PathCommandOp::Arc { to, .. }) => *to,
            Self::By(PathCommandOp::Move(by))
            | Self::By(PathCommandOp::Line(by))
            | Self::By(PathCommandOp::CubicCurve { to: by, .. })
            | Self::By(PathCommandOp::QudraticCurve { to: by, .. })
            | Self::By(PathCommandOp::Arc { to: by, .. }) => {
                Point::new(from.x + by.x, from.y + by.y)
            }
            _ => panic!("Unsupported command for 'to' operation"),
        }
    }

    pub fn transformed(&self, t: Transform) -> Self {
        match self {
            PathCommand::To(PathCommandOp::Move(to)) => {
                PathCommand::To(PathCommandOp::Move(t.transform_point(*to)))
            }
            PathCommand::By(PathCommandOp::Move(by)) => {
                PathCommand::By(PathCommandOp::Move(t.transform_vector(*by)))
            }
            PathCommand::To(PathCommandOp::Line(to)) => {
                PathCommand::To(PathCommandOp::Line(t.transform_point(*to)))
            }
            PathCommand::By(PathCommandOp::Line(by)) => {
                PathCommand::By(PathCommandOp::Line(t.transform_vector(*by)))
            }
            PathCommand::To(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => {
                PathCommand::To(PathCommandOp::CubicCurve {
                    to: t.transform_point(*to),
                    ctrl1: t.transform_point(*ctrl1),
                    ctrl2: t.transform_point(*ctrl2),
                })
            }
            PathCommand::By(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => {
                PathCommand::By(PathCommandOp::CubicCurve {
                    to: t.transform_vector(*to),
                    ctrl1: t.transform_vector(*ctrl1),
                    ctrl2: t.transform_vector(*ctrl2),
                })
            }
            PathCommand::To(PathCommandOp::QudraticCurve { to, ctrl }) => {
                PathCommand::To(PathCommandOp::QudraticCurve {
                    to: t.transform_point(*to),
                    ctrl: t.transform_point(*ctrl),
                })
            }
            PathCommand::By(PathCommandOp::QudraticCurve { to, ctrl }) => {
                PathCommand::By(PathCommandOp::QudraticCurve {
                    to: t.transform_vector(*to),
                    ctrl: t.transform_vector(*ctrl),
                })
            }
            PathCommand::To(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => PathCommand::To(PathCommandOp::Arc {
                to: t.transform_point(*to),
                radii: t.transform_vector(*radii),
                x_rotation: *x_rotation,
                large_arc: *large_arc,
                sweep: *sweep,
            }),
            PathCommand::By(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => PathCommand::By(PathCommandOp::Arc {
                to: t.transform_vector(*to),
                radii: t.transform_vector(*radii),
                x_rotation: *x_rotation,
                large_arc: *large_arc,
                sweep: *sweep,
            }),
            _ => self.clone(),
        }
    }

    /// given the position of horizontal axis flip (mirror) the path
    pub fn flip_vertical(&self, pos: Float) -> Self {
        match self {
            PathCommand::To(PathCommandOp::Move(to)) => {
                PathCommand::To(PathCommandOp::Move(Point::new(pos * 2.0 - to.x, to.y)))
            }
            PathCommand::By(PathCommandOp::Move(by)) => {
                PathCommand::By(PathCommandOp::Move(Vector::new(by.x, by.y)))
            }
            PathCommand::To(PathCommandOp::Line(to)) => {
                PathCommand::To(PathCommandOp::Line(Point::new(pos * 2.0 - to.x, to.y)))
            }
            PathCommand::By(PathCommandOp::Line(by)) => {
                PathCommand::By(PathCommandOp::Line(Vector::new(by.x, by.y)))
            }
            PathCommand::To(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => {
                PathCommand::To(PathCommandOp::CubicCurve {
                    to: Point::new(pos * 2.0 - to.x, to.y),
                    ctrl1: Point::new(pos * 2.0 - ctrl1.x, ctrl1.y),
                    ctrl2: Point::new(pos * 2.0 - ctrl2.x, ctrl2.y),
                })
            }
            PathCommand::By(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => {
                PathCommand::By(PathCommandOp::CubicCurve {
                    to: Vector::new(to.x, to.y),
                    ctrl1: Vector::new(ctrl1.x, ctrl1.y),
                    ctrl2: Vector::new(ctrl2.x, ctrl2.y),
                })
            }
            PathCommand::To(PathCommandOp::QudraticCurve { to, ctrl }) => {
                PathCommand::To(PathCommandOp::QudraticCurve {
                    to: Point::new(pos * 2.0 - to.x, to.y),
                    ctrl: Point::new(pos * 2.0 - ctrl.x, ctrl.y),
                })
            }
            PathCommand::By(PathCommandOp::QudraticCurve { to, ctrl }) => {
                PathCommand::By(PathCommandOp::QudraticCurve {
                    to: Vector::new(to.x, to.y),
                    ctrl: Vector::new(ctrl.x, ctrl.y),
                })
            }
            PathCommand::To(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => PathCommand::To(PathCommandOp::Arc {
                to: Point::new(pos * 2.0 - to.x, to.y),
                radii: Vector::new(radii.x, radii.y),
                x_rotation: *x_rotation + Angle::pi(),
                large_arc: *large_arc,
                sweep: *sweep,
            }),
            PathCommand::By(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => PathCommand::By(PathCommandOp::Arc {
                to: Vector::new(to.x, to.y),
                radii: Vector::new(radii.x, radii.y),
                x_rotation: *x_rotation + Angle::pi(),
                large_arc: *large_arc,
                sweep: *sweep,
            }),
            _ => self.clone(),
        }
    }

    /// given the position of vertical axis flip (mirror) the path
    pub fn flip_horizontal(&self, pos: Float) -> Self {
        match self {
            PathCommand::To(PathCommandOp::Move(to)) => {
                PathCommand::To(PathCommandOp::Move(Point::new(to.x, pos * 2.0 - to.y)))
            }
            PathCommand::By(PathCommandOp::Move(by)) => {
                PathCommand::By(PathCommandOp::Move(Vector::new(by.x, -by.y)))
            }
            PathCommand::To(PathCommandOp::Line(to)) => {
                PathCommand::To(PathCommandOp::Line(Point::new(to.x, pos * 2.0 - to.y)))
            }
            PathCommand::By(PathCommandOp::Line(by)) => {
                PathCommand::By(PathCommandOp::Line(Vector::new(by.x, -by.y)))
            }
            PathCommand::To(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => {
                PathCommand::To(PathCommandOp::CubicCurve {
                    to: Point::new(to.x, pos * 2.0 - to.y),
                    ctrl1: Point::new(ctrl1.x, pos * 2.0 - ctrl1.y),
                    ctrl2: Point::new(ctrl2.x, pos * 2.0 - ctrl2.y),
                })
            }
            PathCommand::By(PathCommandOp::CubicCurve { to, ctrl1, ctrl2 }) => {
                PathCommand::By(PathCommandOp::CubicCurve {
                    to: Vector::new(to.x, -to.y),
                    ctrl1: Vector::new(ctrl1.x, -ctrl1.y),
                    ctrl2: Vector::new(ctrl2.x, -ctrl2.y),
                })
            }
            PathCommand::To(PathCommandOp::QudraticCurve { to, ctrl }) => {
                PathCommand::To(PathCommandOp::QudraticCurve {
                    to: Point::new(to.x, pos * 2.0 - to.y),
                    ctrl: Point::new(ctrl.x, pos * 2.0 - ctrl.y),
                })
            }
            PathCommand::By(PathCommandOp::QudraticCurve { to, ctrl }) => {
                PathCommand::By(PathCommandOp::QudraticCurve {
                    to: Vector::new(to.x, -to.y),
                    ctrl: Vector::new(ctrl.x, -ctrl.y),
                })
            }
            PathCommand::To(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => PathCommand::To(PathCommandOp::Arc {
                to: Point::new(to.x, pos * 2.0 - to.y),
                radii: Vector::new(radii.x, -radii.y),
                x_rotation: *x_rotation + Angle::pi(),
                large_arc: *large_arc,
                sweep: !*sweep,
            }),
            PathCommand::By(PathCommandOp::Arc {
                to,
                radii,
                x_rotation,
                large_arc,
                sweep,
            }) => PathCommand::By(PathCommandOp::Arc {
                to: Vector::new(to.x, -to.y),
                radii: Vector::new(radii.x, -radii.y),
                x_rotation: *x_rotation + Angle::pi(),
                large_arc: *large_arc,
                sweep: !*sweep,
            }),
            _ => self.clone(),
        }
    }
}
