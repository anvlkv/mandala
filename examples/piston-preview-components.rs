use glutin_window::GlutinWindow as Window;
use mandala::{
    Angle, ArcFlags, BBox, CubicCurve, DrawArgs, Epoch, EpochBuilder, EpochLayout, FillValue,
    GeneratorBuilder, GeneratorMode, Line, Mandala, MandalaSegment, MandalaSegmentBuilder, Path,
    PathSegment, Point, QuadraticCurve, Rect, SegmentDrawing, Size, SvgArc, Transform, Vector,
};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use piston::{UpdateArgs, UpdateEvent};
use rand::rngs::SmallRng;

pub struct App {
    gl: GlGraphics,
    drawing: Vec<Path>,
    segment_lines: MandalaSegment,
    segment_arcs: MandalaSegment,
    segment_cubics: MandalaSegment,
    segment_quads: MandalaSegment,
    segment_drawing_lines: Vec<Path>,
    segment_drawing_arcs: Vec<Path>,
    segment_drawing_cubics: Vec<Path>,
    segment_drawing_qads: Vec<Path>,
    epoch_drawing: Vec<Path>,
    epoch: Epoch,
    _mandala: Mandala,
    mandala_drawing: Vec<Path>,
    update_t: f64,
}

const SIZE: u32 = 800;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.7];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 0.7];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 0.7];
        const PURPLE: [f32; 4] = [1.0, 0.0, 1.0, 0.7];
        const STROKE: f64 = 0.5;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c.transform.trans(10.0, 10.0);

            for p in self
                .drawing
                .clone()
                .into_iter()
                .chain(self.segment_drawing_lines.clone())
                .chain(self.segment_drawing_arcs.clone())
                .chain(self.segment_drawing_cubics.clone())
                .chain(self.segment_drawing_qads.clone())
                .chain(self.epoch_drawing.clone())
                .chain(self.mandala_drawing.clone())
            {
                for s in p.into_iter() {
                    let clr = match s {
                        mandala::PathSegment::Arc(_) => RED,
                        mandala::PathSegment::QuadraticCurve(_) => BLUE,
                        mandala::PathSegment::CubicCurve(_) => PURPLE,
                        _ => WHITE,
                    };

                    for l in s.flattened() {
                        line(
                            clr,
                            STROKE,
                            [l.from.x, l.from.y, l.to.x, l.to.y],
                            transform,
                            gl,
                        );
                    }
                }
            }
        });
    }

    fn update(&mut self, u: &UpdateArgs) {
        self.segment_lines.angle_base += Angle::radians(u.dt);
        self.segment_drawing_lines = self.segment_lines.render_paths();
        self.segment_arcs.angle_base += Angle::radians(u.dt);
        self.segment_drawing_arcs = self.segment_arcs.render_paths();
        self.segment_cubics.angle_base += Angle::radians(u.dt);
        self.segment_drawing_cubics = self.segment_cubics.render_paths();
        self.segment_quads.angle_base += Angle::radians(u.dt);
        self.segment_drawing_qads = self.segment_quads.render_paths();

        self.update_t += u.dt;

        if self.update_t >= 0.5 {
            self.update_t = 0.0;
            self.epoch.layout = match self.epoch.layout {
                EpochLayout::Circle { radius } => EpochLayout::Ellipse {
                    radii: Size::new(radius, radius / 2.0),
                },
                EpochLayout::Ellipse { radii } => EpochLayout::Polygon {
                    n_sides: 7,
                    radius: radii.width,
                    start: Angle::zero(),
                },
                EpochLayout::Polygon { radius, .. } => EpochLayout::Rectangle {
                    rect: Size::new(radius, radius * 2.0),
                },
                EpochLayout::Rectangle { rect } => EpochLayout::Circle { radius: rect.width },
            };
            self.epoch_drawing = self.epoch.render_paths();
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("preview components", [SIZE, SIZE])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let center = Point::new(180.0, 250.0);
    let sweep = Angle::frac_pi_3();

    let mut drawing = Vec::new();
    let renderer = |_rng: &mut SmallRng, _| {
        Path::new(PathSegment::Line(Line {
            from: Point::new(0.0, 0.0),
            to: Point::new(10.0, 3.0),
        }))
    };

    let mut gen = GeneratorBuilder::default()
        .renderer(renderer)
        .transform(Transform::Rotate(FillValue::Incremental {
            init: Angle::radians(0.001),
            increment: Angle::radians(0.01),
        }))
        .mode(GeneratorMode::GridStep {
            row_height: 8.0,
            column_width: 10.0,
        })
        .build()
        .unwrap();

    let pattern = gen.generate(Rect::from_size(Size::new(100.0, 100.0)));

    drawing.extend(pattern.clone());

    let segment_lines = MandalaSegmentBuilder::default()
        .drawing(vec![SegmentDrawing::Path(pattern)])
        .angle_base(Angle::zero())
        .sweep(sweep)
        .center(center)
        .r_base(80.0)
        .breadth(0.4)
        .build()
        .unwrap();

    let arc_renderer = |_rng: &mut SmallRng, _| {
        Path::new(PathSegment::Arc(SvgArc {
            from: Point::new(0.0, 0.0),
            to: Point::new(10.0, 10.0),
            radii: Vector::new(5.0, 5.0),
            x_rotation: Angle::degrees(0.0),
            flags: ArcFlags {
                large_arc: false,
                sweep: true,
            },
        }))
    };

    let mut arc_gen = GeneratorBuilder::default()
        .renderer(arc_renderer)
        .transform(Transform::Rotate(FillValue::Incremental {
            init: Angle::radians(0.0),
            increment: Angle::radians(0.1),
        }))
        .mode(GeneratorMode::GridStep {
            row_height: 10.0,
            column_width: 10.0,
        })
        .build()
        .unwrap();

    let arc_pattern = arc_gen.generate(Rect::from_size(Size::new(100.0, 100.0)));

    drawing.extend(
        arc_pattern
            .clone()
            .into_iter()
            .map(|p| p.translate(Vector::new(120.0, 0.0))),
    );

    let segment_arcs = MandalaSegmentBuilder::default()
        .drawing(vec![SegmentDrawing::Path(arc_pattern)])
        .angle_base(sweep)
        .sweep(sweep)
        .center(center)
        .r_base(80.0)
        .breadth(0.4)
        .build()
        .unwrap();

    let cubic_renderer = |_rng: &mut SmallRng, _| {
        Path::new(PathSegment::CubicCurve(CubicCurve {
            from: Point::new(0.0, 0.0),
            ctrl1: Point::new(3.0, 5.0),
            ctrl2: Point::new(7.0, 5.0),
            to: Point::new(10.0, 0.0),
        }))
    };

    let mut cubic_gen = GeneratorBuilder::default()
        .renderer(cubic_renderer)
        .transform(Transform::Scale(FillValue::Incremental {
            init: 1.0,
            increment: 0.1,
        }))
        .mode(GeneratorMode::XStep(10.0))
        .build()
        .unwrap();

    let cubic_pattern = cubic_gen.generate(Rect::from_size(Size::new(100.0, 100.0)));

    drawing.extend(
        cubic_pattern
            .clone()
            .into_iter()
            .map(|p| p.translate(Vector::new(120.0 * 2.0, 0.0))),
    );

    let segment_cubics = MandalaSegmentBuilder::default()
        .drawing(vec![SegmentDrawing::Path(cubic_pattern)])
        .angle_base(sweep * 2.0)
        .sweep(sweep)
        .center(center)
        .r_base(80.0)
        .breadth(0.4)
        .build()
        .unwrap();

    let quad_renderer = |_rng: &mut SmallRng, _| {
        Path::new(PathSegment::QuadraticCurve(QuadraticCurve {
            from: Point::new(0.0, 0.0),
            ctrl: Point::new(5.0, 10.0),
            to: Point::new(10.0, 0.0),
        }))
    };

    let mut quad_gen = GeneratorBuilder::default()
        .renderer(quad_renderer)
        .transform(Transform::Translate(FillValue::Incremental {
            init: Vector::new(0.0, 0.0),
            increment: Vector::new(1.0, 1.0),
        }))
        .mode(GeneratorMode::YStep(10.0))
        .build()
        .unwrap();

    let quad_pattern = quad_gen.generate(Rect::from_size(Size::new(100.0, 100.0)));

    drawing.extend(
        quad_pattern
            .clone()
            .into_iter()
            .map(|p| p.translate(Vector::new(120.0 * 3.0, 0.0))),
    );

    let segment_quads = MandalaSegmentBuilder::default()
        .drawing(vec![SegmentDrawing::Path(quad_pattern)])
        .angle_base(sweep * 3.0)
        .sweep(sweep)
        .center(center)
        .r_base(80.0)
        .breadth(0.4)
        .build()
        .unwrap();

    let radius = 100.0;
    let breadth = 50.0;

    let ep_center = center.add_size(&Size::new(0.0, 300.0));
    let mut simple_ep = EpochBuilder::default()
        .center(ep_center)
        .layout(EpochLayout::Circle {
            radius: radius - breadth,
        })
        .outline(true)
        .build()
        .unwrap();

    let renderer = |_rng: &mut SmallRng, _: Size| Path::rect(Size::splat(100.0), Point::splat(0.0));

    let mut gen = GeneratorBuilder::default()
        .renderer(renderer)
        .mode(GeneratorMode::YStep(25.0))
        .build()
        .unwrap();

    let pattern = gen.generate(Rect::from_size(Size::new(100.0, 100.0)));

    drawing.extend(
        pattern
            .clone()
            .into_iter()
            .map(|p| p.translate(Vector::new(120.0 * 5.0, 0.0))),
    );

    let mut draw_fn = |args: &DrawArgs| {
        MandalaSegmentBuilder::default()
            .angle_base(args.start_angle)
            .sweep(Angle::frac_pi_4())
            .center(args.center)
            .r_base(radius)
            .breadth(0.5)
            .drawing(vec![SegmentDrawing::Path(pattern.clone())])
            .build()
            .unwrap()
    };

    simple_ep.draw_fill(&mut draw_fn);

    drawing.extend(simple_ep.render_paths());

    let ep_center = center.add_size(&Size::new(300.0, 0.0));
    let mut epoch = EpochBuilder::default()
        .center(ep_center)
        .layout(EpochLayout::Circle {
            radius: radius - breadth,
        })
        .outline(true)
        .build()
        .unwrap();

    let renderer = |_rng: &mut SmallRng, _| {
        Path::new(PathSegment::Arc(SvgArc {
            from: Point::new(0.0, 0.0),
            to: Point::new(10.0, 3.0),
            radii: Vector::splat(15.0),
            x_rotation: Angle::zero(),
            flags: ArcFlags::default(),
        }))
        // Path::new(PathSegment::Line(Line {
        //     from: Point::new(0.0, 0.0),
        //     to: Point::new(10.0, 3.0),
        // }))
    };

    let mut gen = GeneratorBuilder::default()
        .renderer(renderer)
        .transform(Transform::Rotate(FillValue::Rand(vec![
            Angle::zero(),
            Angle::frac_pi_4(),
            Angle::frac_pi_2(),
        ])))
        .mode(GeneratorMode::GridStep {
            row_height: 8.0,
            column_width: 10.0,
        })
        .build()
        .unwrap();

    let pattern = gen.generate(Rect::from_size(Size::new(100.0, 100.0)));

    drawing.extend(
        pattern
            .clone()
            .into_iter()
            .map(|p| p.translate(Vector::new(120.0 * 4.0, 0.0))),
    );

    let mut draw_fn = |args: &DrawArgs| {
        MandalaSegmentBuilder::default()
            .angle_base(args.start_angle)
            .sweep(Angle::frac_pi_4())
            .center(args.center)
            .r_base(radius)
            .breadth(0.5)
            .drawing(vec![SegmentDrawing::Path(pattern.clone())])
            .build()
            .unwrap()
    };

    epoch.draw_fill(&mut draw_fn);

    let epoch_drawing = epoch.render_paths();

    let mut mndl = Mandala::new(BBox::new(Point::zero(), Point::splat(250.0)));

    mndl.draw_epoch(|_, _| simple_ep.translate(Vector::new(300.0, 0.0)).scale(0.75));

    let mndl_2 = {
        let mut m = mndl.clone();
        m.epochs = m
            .epochs
            .into_iter()
            .map(|ep| ep.translate(Vector::new(-150.0, -300.0)))
            .collect();

        m
    };

    mndl.draw_epoch(|_, _| {
        let mut ep = epoch.translate(Vector::new(0.0, 300.0)).scale(2.25);

        if let Some(sg) = ep.segments.last_mut() {
            sg.drawing.push(SegmentDrawing::Mandala {
                mandala: mndl_2.clone(),
                placement_box: BBox::from_size(Size::splat(50.0)),
            });
        }
        ep
    });

    let mandala_drawing = mndl.render_paths();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        update_t: 0.0,
        segment_drawing_lines: segment_lines.render_paths(),
        segment_drawing_arcs: segment_arcs.render_paths(),
        segment_drawing_cubics: segment_cubics.render_paths(),
        segment_drawing_qads: segment_quads.render_paths(),
        _mandala: mndl,
        mandala_drawing,
        drawing,
        segment_lines,
        segment_arcs,
        segment_cubics,
        segment_quads,
        epoch_drawing,
        epoch,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}