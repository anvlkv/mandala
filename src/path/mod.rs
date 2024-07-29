mod command;
#[cfg(feature = "styled")]
mod style;

pub use command::*;
#[cfg(feature = "styled")]
pub use style::*;

use crate::{Angle, Point, Vector};

/// chain of path commands drawing continuous line or shape
///
/// optionally styled with feature `styled`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Path {
    /// all commands defining the path
    pub commands: Vec<PathCommand>,
    /// individual style applied to this path
    #[cfg(feature = "styled")]
    pub style: Option<PathStyle>,
}

impl Path {
    pub fn move_to(&mut self, to: Point) -> &mut Self {
        self.commands.push(PathCommand::To(PathCommandOp::Move(to)));
        self
    }

    pub fn move_by(&mut self, by: Vector) -> &mut Self {
        self.commands.push(PathCommand::By(PathCommandOp::Move(by)));
        self
    }

    pub fn line_to(&mut self, to: Point) -> &mut Self {
        self.commands.push(PathCommand::To(PathCommandOp::Line(to)));
        self
    }

    pub fn line_by(&mut self, by: Vector) -> &mut Self {
        self.commands.push(PathCommand::By(PathCommandOp::Line(by)));
        self
    }

    pub fn cubic_curve_to(&mut self, to: Point, ctrl1: Point, ctrl2: Point) -> &mut Self {
        self.commands
            .push(PathCommand::To(PathCommandOp::CubicCurve {
                to,
                ctrl1,
                ctrl2,
            }));
        self
    }

    pub fn cubic_curve_by(&mut self, by: Vector, ctrl1: Vector, ctrl2: Vector) -> &mut Self {
        self.commands
            .push(PathCommand::By(PathCommandOp::CubicCurve {
                to: by,
                ctrl1,
                ctrl2,
            }));
        self
    }

    pub fn quadratic_curve_to(&mut self, to: Point, ctrl: Point) -> &mut Self {
        self.commands
            .push(PathCommand::To(PathCommandOp::QudraticCurve { to, ctrl }));
        self
    }

    pub fn quadratic_curve_by(&mut self, by: Vector, ctrl: Vector) -> &mut Self {
        self.commands
            .push(PathCommand::By(PathCommandOp::QudraticCurve {
                to: by,
                ctrl,
            }));
        self
    }

    pub fn arc_to(
        &mut self,
        to: Point,
        radii: Vector,
        x_rotation: Angle,
        large_arc: bool,
        sweep: bool,
    ) -> &mut Self {
        self.commands.push(PathCommand::To(PathCommandOp::Arc {
            to,
            radii,
            x_rotation,
            large_arc,
            sweep,
        }));
        self
    }

    pub fn arc_by(
        &mut self,
        by: Vector,
        radii: Vector,
        x_rotation: Angle,
        large_arc: bool,
        sweep: bool,
    ) -> &mut Self {
        self.commands.push(PathCommand::By(PathCommandOp::Arc {
            to: by,
            radii,
            x_rotation,
            large_arc,
            sweep,
        }));
        self
    }

    pub fn close_path(&mut self) -> &mut Self {
        self.commands
            .push(PathCommand::To(PathCommandOp::ClosePath));
        self
    }
}
