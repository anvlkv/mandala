mod command;

#[cfg(feature = "styled")]
mod style;

pub use command::*;
#[cfg(feature = "styled")]
pub use style::*;

use crate::{Angle, Float, Point, Size, Transform, Vector};

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
    /// draw a closed rectangle shape
    pub fn rect(top_left: Point, size: Size) -> Self {
        let mut path = Self::default();

        path.move_to(top_left)
            .line_to(Point::from([top_left.x + size.width, top_left.y]))
            .line_to(Point::from([
                top_left.x + size.width,
                top_left.y + size.height,
            ]))
            .line_to(Point::from([top_left.x, top_left.y + size.height]))
            .close_path();

        path
    }

    /// draw a closed polygon shape
    pub fn polygon(center: Point, size: Size, n_sides: usize, start_angle: Angle) -> Self {
        let mut path = Self::default();

        let angle_step = Angle::two_pi() / n_sides as Float;

        for i in 0..n_sides {
            let angle = start_angle + angle_step * i as Float;
            let (sin, cos) = angle.sin_cos();
            let x = center.x + size.width * cos;
            let y = center.y + size.height * sin;
            if i == 0 {
                path.move_to(Point::from([x, y]));
            } else {
                path.line_to(Point::from([x, y]));
            }
        }
        path.close_path();

        path
    }

    /// draw a closed ellipse shape
    ///
    /// the shape is drawn with 4 arcs
    pub fn ellipse(center: Point, radius_x: Float, radius_y: Float) -> Self {
        let mut path = Self::default();

        let start_angle = Angle::zero();
        let arc_angle = Angle::frac_pi_2();

        let (sin, cos) = start_angle.sin_cos();
        let start_point = Point::from([center.x + radius_x * cos, center.y + radius_y * sin]);
        path.move_to(start_point);

        for i in 0..4 {
            let current_start_angle = start_angle + arc_angle * i as Float;
            let current_end_angle = current_start_angle + arc_angle;
            let (sin_end, cos_end) = current_end_angle.sin_cos();

            let arc_end =
                Point::from([center.x + radius_x * cos_end, center.y + radius_y * sin_end]);

            path.arc_to(
                arc_end,
                Vector::from([radius_x, radius_y]),
                Angle::zero(),
                false,
                true,
            );
        }

        path.close_path();

        path
    }

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

    pub fn close_path(&mut self) {
        self.commands
            .push(PathCommand::To(PathCommandOp::ClosePath));
    }

    /// render all path commands as 2D SVG path `d` attribute value
    pub fn to_svg_path_d(&self) -> String {
        self.commands.iter().map(|c| c.to_svg_path_d()).collect()
    }

    /// coompute the length of the path
    pub fn length(&self) -> Float {
        self.lengths().iter().sum()
    }

    /// apply transformation to all commands
    ///
    /// does not affect style
    pub fn transformed(&self, t: Transform) -> Self {
        Self {
            commands: self.commands.iter().map(|c| c.transformed(t)).collect(),
            #[cfg(feature = "styled")]
            style: self.style.clone(),
        }
    }

    /// given the position of axis along `x` flip (mirror) the path
    pub fn flip_horizontal(&self, pos: Float) -> Self {
        Self {
            commands: self
                .commands
                .iter()
                .map(|c| c.flip_horizontal(pos))
                .collect(),
            #[cfg(feature = "styled")]
            style: self.style.clone(),
        }
    }
    /// given the position of axis along `y` flip (mirror) the path
    pub fn flip_vertical(&self, pos: Float) -> Self {
        let mut transformed_commands = Vec::new();
        for command in &self.commands {
            transformed_commands.push(command.flip_vertical(pos));
        }
        Self {
            commands: transformed_commands,
            #[cfg(feature = "styled")]
            style: self.style.clone(),
        }
    }

    // pub fn sampling_iter(&self, from: Option<Point>) {
    //     let from = from.unwrap_or(Point::zero());
    //     let lengths = self.lengths();
    //     let total_length = lengths.iter().sum();
    //     let len_fr = lengths.iter().map(|l| l / total_length);

    // }

    fn from(&self) -> Point {
        self.commands
            .first()
            .map(|c| match c {
                PathCommand::To(PathCommandOp::Move(pt)) => *pt,
                PathCommand::By(PathCommandOp::Move(by)) => by.to_point(),
                _ => Point::zero(),
            })
            .unwrap_or(Point::zero())
    }

    fn lengths(&self) -> Vec<Float> {
        let mut from = self.from();
        let mut len = vec![];
        let mut it = self.commands.iter().peekable();

        while let Some(cm) = it.next() {
            len.push(cm.length(from));

            from = cm.to(from);
            if it.peek().map(|c| c.is_close()).unwrap_or(false) {
                len.push(from.distance_to(self.from()));
                break;
            }
        }

        len
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let top_left = Point::from([0.0, 0.0]);
        let size = Size::from([10.0, 20.0]);
        let path = Path::rect(top_left, size);
        assert_eq!(path.commands.len(), 5); // Move, 3 Line, Close
    }

    #[test]
    fn test_polygon_creation() {
        let center = Point::from([0.0, 0.0]);
        let size = Size::from([10.0, 10.0]);
        let n_sides = 5;
        let start_angle = Angle::zero();
        let path = Path::polygon(center, size, n_sides, start_angle);
        assert_eq!(path.commands.len(), n_sides + 1); // Move, n Line, Close
    }

    #[test]
    fn test_ellipse_creation() {
        let center = Point::from([0.0, 0.0]);
        let radius_x = 10.0;
        let radius_y = 5.0;
        let path = Path::ellipse(center, radius_x, radius_y);
        assert_eq!(path.commands.len(), 6); // Move, 4 Arc, Close
    }

    #[test]
    fn test_path_length_calculation() {
        let top_left = Point::from([0.0, 0.0]);
        let size = Size::from([10.0, 20.0]);
        let path = Path::rect(top_left, size);
        let length = path.length();
        // ha-ha
        assert_eq!(length, 60.0); // Sum of all line lengths in the rectangle
    }

    #[test]
    fn test_path_transformation() {
        let top_left = Point::from([0.0, 0.0]);
        let size = Size::from([10.0, 20.0]);
        let path = Path::rect(top_left, size);
        let transform = Transform::translation(10.0, 10.0);
        let transformed_path = path.transformed(transform);
        assert_eq!(transformed_path.commands.len(), 5); // Same number of commands
    }

    #[test]
    fn test_path_d() {
        let mut path = Path::default();

        path.move_to([0.0, 10.0].into())
            .move_by([1.0, 1.0].into())
            .line_to([2.0, 3.0].into())
            .line_by([1.0, 1.0].into())
            .cubic_curve_to([3.0, 4.0].into(), [2.5, 3.5].into(), [2.8, 3.8].into())
            .cubic_curve_by([1.0, 1.0].into(), [0.5, 0.5].into(), [0.8, 0.8].into())
            .quadratic_curve_to([5.0, 6.0].into(), [4.5, 5.5].into())
            .quadratic_curve_by([1.0, 1.0].into(), [0.5, 0.5].into())
            .arc_to(
                [7.0, 8.0].into(),
                [2.0, 2.0].into(),
                Angle::zero(),
                false,
                true,
            )
            .arc_by(
                [1.0, 1.0].into(),
                [1.0, 1.0].into(),
                Angle::zero(),
                false,
                true,
            )
            .close_path();

        insta::assert_snapshot!(path.to_svg_path_d());
    }
}
