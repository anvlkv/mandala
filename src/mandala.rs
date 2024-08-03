use derive_builder::Builder;

use crate::{Angle, BBox, Chord, Path, PathCommand, Point, Size, Vector};

/// a [Mandala] represents a concentric drawing
/// consisting of multiple [Chords]
/// layed out along the perimeter of a [MandalaLayout]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Builder)]
pub struct Mandala {
    /// bounding box of the mandala and its coordinate space
    pub bounds: BBox,
    /// layout applied in drawing [Chord]s
    pub layout: MandalaLayout,
    /// contents of this mandala
    #[builder(setter(each(name = "chord")))]
    pub chords: Vec<Chord>,
    #[cfg(feature = "styled")]
    /// optional style applied to all contents of this [Mandala]
    ///
    /// individual [ChordDrawing]'s and [Path]'s styles will take priority if applied
    pub style: Option<crate::path::PathStyle>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DrawArgs {
    pub bbox: BBox,
    pub layout: MandalaLayout,
    pub last_chord: Option<Chord>,
    pub n_th_chord: usize,
}

impl MandalaBuilder {
    pub fn draw_chord<F>(&mut self, draw_fn: &mut F) -> &mut Self
    where
        F: FnMut(DrawArgs) -> Chord,
    {
        let bbox = self
            .bounds
            .as_ref()
            .expect("bounds must be set before drawing chords")
            .clone();
        let layout = self
            .layout
            .as_ref()
            .expect("layout must be set before drawing chords")
            .clone();

        let last_chord = self.chords.as_ref().map(|c| c.last()).flatten().cloned();
        let n_th_chord = self.chords.as_ref().map(|c| c.len()).unwrap_or_default() + 1;

        let args = DrawArgs {
            bbox,
            layout,
            last_chord,
            n_th_chord,
        };

        let chord = draw_fn(args);

        self.chord(chord)
    }
}

/// aranges [Chord]'s contents along the perimeter of a given shape
/// matching `start_angle` and `sweep_angle` results in full circle
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum MandalaLayout {
    Arc {
        center: Point,
        radii: Vector,
        start_angle: Angle,
        sweep_angle: Angle,
        x_rotation: Angle,
    },
    Rect {
        center: Point,
        size: Size,
        start_angle: Angle,
        sweep_angle: Angle,
    },
    Polygon {
        center: Point,
        size: Size,
        n_sides: usize,
        /// start point of the polygon
        angle_0: Angle,
        /// drawing area start angle
        start_angle: Angle,
        /// drawing area sweep
        sweep_angle: Angle,
    },
    /// a path drawn in global or higher mandala coordinates
    Path { path: Path, from: Point },
}

impl Into<Path> for MandalaLayout {
    fn into(self) -> Path {
        match self {
            Self::Path { path, from } => {
                let mut path = path.clone();
                path.commands
                    .insert(0, PathCommand::To(crate::PathCommandOp::Move(from)));
                path
            }
            Self::Arc {
                center,
                radii,
                start_angle,
                sweep_angle,
                x_rotation,
            } => {
                let mut path = Path::ellipse(center, radii.x, radii.y);

                path
            }
            Self::Rect {
                center,
                size,
                start_angle,
                sweep_angle,
            } => {
                let top_left =
                    Point::new(center.x - size.width / 2.0, center.y - size.height / 2.0);
                let mut path = Path::rect(top_left, size);
                path
            }
            Self::Polygon {
                center,
                size,
                n_sides,
                angle_0,
                start_angle,
                sweep_angle,
            } => {
                let path = Path::polygon(center, size, n_sides, angle_0);

                path
            }
        }
    }
}

#[cfg(test)]
mod mandala_tests {

    use crate::ChordBuilder;

    use super::*;

    #[test]
    fn test_mandala_builder_draw_chord() {
        let mut builder = MandalaBuilder::default();
        builder
            .bounds(BBox::new(Point::new(0.0, 0.0), Point::new(100.0, 100.0)))
            .layout(MandalaLayout::Arc {
                center: Point::new(50.0, 50.0),
                radii: Vector::new(50.0, 50.0),
                start_angle: Angle::zero(),
                sweep_angle: Angle::two_pi(),
                x_rotation: Angle::zero(),
            });

        // let mut draw_fn = |args: DrawArgs| {
        //     let start_point = args.layout.from(args.n_th_chord, args.bbox);
        //     let end_point = args.layout.to(args.n_th_chord, args.bbox);
        //     ChordBuilder::default()
        //         .from(start_point)
        //         .to(end_point)
        //         .draw(args.layout.draw_item(args.n_th_chord, args.bbox))
        //         .build()
        //         .expect("build chord")
        // };
        // builder.draw_chord(&mut draw_fn);

        let mandala = builder.build().unwrap();
        assert_eq!(mandala.chords.len(), 1);
    }
}
