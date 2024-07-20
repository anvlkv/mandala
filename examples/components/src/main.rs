use glutin_window::GlutinWindow as Window;
use mandala::{
    Angle, Arc, ArcFlags, CubicBezierSegment, Epoch, EpochBuilder, LineSegment, Mandala, Path,
    Point2D, QuadraticBezierSegment, Rect, Segment, SegmentRule, Size2D, SvgArc, Triangle,
    Vector2D,
};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use piston::{UpdateArgs, UpdateEvent};
use rand::{rngs::ThreadRng, thread_rng};

pub struct App {
    gl: GlGraphics,
    drawing: Vec<Path>,
    rng: ThreadRng,
    rands: Vec<Path>,
    update_time: f64,
}

const SIZE: u32 = 800;
const RND_SIZE: u32 = 60;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.7];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 0.7];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 0.7];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 0.7];
        const PURPLE: [f32; 4] = [1.0, 0.0, 1.0, 0.7];
        const STROKE: f64 = 0.5;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c.transform.trans(10.0, 10.0);

            for p in self.drawing.clone().into_iter() {
                for s in p.into_iter() {
                    let clr = match s {
                        mandala::Segment::Line(_) => WHITE,
                        mandala::Segment::Arc(_) => RED,
                        mandala::Segment::Triangle(_) => GREEN,
                        mandala::Segment::QuadraticCurve(_) => BLUE,
                        mandala::Segment::CubicCurve(_) => PURPLE,
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

            for (i, p) in self.rands.iter().enumerate() {
                let clr = match i {
                    0 | 5 => WHITE,
                    1 | 6 => RED,
                    2 | 7 => GREEN,
                    3 | 8 => BLUE,
                    4 | 9 => PURPLE,
                    _ => [0.5, 0.0, 0.5, 0.7],
                };
                for s in p.clone().into_iter() {
                    for l in s.flattened() {
                        line(
                            clr,
                            STROKE,
                            [l.from.x, l.from.y, l.to.x, l.to.y],
                            c.transform.trans(
                                (i as u32 * RND_SIZE) as f64 + 15.0,
                                (SIZE - RND_SIZE) as f64 - 15.0,
                            ),
                            gl,
                        );
                    }
                }
            }
        });
    }

    fn update(&mut self, u: &UpdateArgs) {
        self.update_time += u.dt;

        if self.update_time >= 0.01 {
            let size = RND_SIZE as f64;
            let mut rands = vec![];
            for i in 0..(SIZE / RND_SIZE) {
                let i = (i + 1) as u8;
                let bounds = Rect::from_size(Size2D::splat(size));
                let p = Path::generate(&mut self.rng, bounds, i.rem_euclid(2) == 0, i);
                rands.push(p);
            }
            self.rands = rands;
            self.update_time = 0.0;
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

    let mut drawing = Vec::new();

    // example segments
    drawing.push({
        Path::new(Segment::Arc(
            Arc {
                center: Point2D::new(30.0, 30.0),
                radii: Vector2D::new(10.0, 10.0),
                start_angle: Angle::zero(),
                sweep_angle: Angle::two_pi(),
                x_rotation: Angle::zero(),
            }
            .to_svg_arc(),
        ))
    });

    drawing.push({
        Path::new(Segment::CubicCurve(CubicBezierSegment {
            from: Point2D::new(50.0, 50.0),
            ctrl1: Point2D::new(40.0, 20.0),
            ctrl2: Point2D::new(60.0, 20.0),
            to: Point2D::new(50.0, 50.0),
        }))
    });

    drawing.push({
        Path::new(Segment::QuadraticCurve(QuadraticBezierSegment {
            from: Point2D::new(80.0, 50.0),
            ctrl: Point2D::new(80.0, 20.0),
            to: Point2D::new(90.0, 50.0),
        }))
    });

    drawing.push({
        Path::new(Segment::Triangle(Triangle {
            a: Point2D::new(100.0, 50.0),
            b: Point2D::new(120.0, 20.0),
            c: Point2D::new(140.0, 50.0),
        }))
    });

    drawing.push({
        Path::new(Segment::Line(LineSegment {
            from: Point2D::new(150.0, 50.0),
            to: Point2D::new(180.0, 20.0),
        }))
    });

    drawing.push({
        let mut p = Path::new(Segment::Line(LineSegment {
            from: Point2D::new(200.0, 20.0),
            to: Point2D::new(220.0, 50.0),
        }));

        p.draw_next(|last| {
            Segment::QuadraticCurve(QuadraticBezierSegment {
                from: last.to(),
                ctrl: Point2D::new(230.0, 20.0),
                to: Point2D::new(260.0, 50.0),
            })
        });

        p.draw_next(|last| {
            Segment::CubicCurve(CubicBezierSegment {
                from: last.to(),
                ctrl1: Point2D::new(270.0, 20.0),
                ctrl2: Point2D::new(275.0, 20.0),
                to: Point2D::new(280.0, 50.0),
            })
        });

        p.draw_next(|last| {
            Segment::Triangle(Triangle {
                a: last.to(),
                b: Point2D::new(295.0, 20.0),
                c: Point2D::new(305.0, 50.0),
            })
        });

        p.draw_next(|last| {
            Segment::Arc(SvgArc {
                from: last.to(),
                to: Point2D::new(325.0, 20.0),
                x_rotation: Angle::zero(),
                radii: Vector2D::new(10.0, 10.0),
                flags: Default::default(),
            })
        });

        p
    });

    drawing.push({
        Path::new(Segment::Line(LineSegment {
            from: Point2D::new(-10.0, 70.0),
            to: Point2D::new(SIZE as f64, 70.0),
        }))
    });

    // example epochs
    let epoch = Epoch {
        segments: 19,
        radius: 60.0,
        breadth: 20.0,
        center: Point2D::new(90.0, 160.0),
        segment_rule: SegmentRule::Path(Path::new(Segment::Triangle(Triangle {
            a: Point2D::new(-10.0, 0.0),
            b: Point2D::new(0.0, 20.0),
            c: Point2D::new(10.0, 0.0),
        }))),
    };

    drawing.extend(epoch.render_paths());

    let epoch = Epoch {
        segments: 120,
        radius: 60.0,
        breadth: 20.0,
        center: Point2D::new(260.0, 160.0),
        segment_rule: SegmentRule::EveryNth(
            Path::new(Segment::CubicCurve(CubicBezierSegment {
                from: Point2D::new(-10.0, 0.0),
                ctrl1: Point2D::new(-5.0, 20.0),
                ctrl2: Point2D::new(5.0, 20.0),
                to: Point2D::new(10.0, 0.0),
            })),
            3,
        ),
    };

    drawing.extend(epoch.render_paths());

    let epoch = Epoch {
        segments: 12,
        radius: 60.0,
        breadth: 20.0,
        center: Point2D::new(430.0, 160.0),
        segment_rule: SegmentRule::OddEven(
            Path::new(Segment::QuadraticCurve(QuadraticBezierSegment {
                from: Point2D::new(-10.0, 0.0),
                ctrl: Point2D::new(0.0, 27.0),
                to: Point2D::new(10.0, 0.0),
            })),
            Path::new(Segment::Arc(SvgArc {
                from: Point2D::new(-10.0, 0.0),
                to: Point2D::new(10.0, 0.0),
                radii: Vector2D::new(15.0, 15.0),
                x_rotation: Angle::zero(),
                flags: ArcFlags {
                    large_arc: true,
                    sweep: false,
                },
            })),
        ),
    };

    drawing.extend(epoch.render_paths());

    let mut epoch = Epoch {
        segments: 10,
        radius: 60.0,
        breadth: 20.0,
        center: Point2D::new(600.0, 160.0),
        segment_rule: SegmentRule::None,
    };

    epoch.draw_segment(|min, max| {
        let mut path_min = Path::new(Segment::Line(LineSegment {
            from: min.min(),
            to: Point2D::new(min.max_x(), min.min_y()),
        }));
        path_min.draw_next(|last| {
            Segment::Line(LineSegment {
                from: last.to(),
                to: Point2D::new(min.max_x(), min.max_y()),
            })
        });
        path_min.draw_next(|last| {
            Segment::Line(LineSegment {
                from: last.to(),
                to: Point2D::new(min.min_x(), min.max_y()),
            })
        });
        path_min.draw_next(|last| {
            Segment::Line(LineSegment {
                from: last.to(),
                to: Point2D::new(min.min_x(), min.min_y()),
            })
        });
        let mut path_max = Path::new(Segment::Line(LineSegment {
            from: max.min(),
            to: Point2D::new(max.max_x(), max.min_y()),
        }));
        path_max.draw_next(|last| {
            Segment::Line(LineSegment {
                from: last.to(),
                to: Point2D::new(max.max_x(), max.max_y()),
            })
        });
        path_max.draw_next(|last| {
            Segment::Line(LineSegment {
                from: last.to(),
                to: Point2D::new(max.min_x(), max.max_y()),
            })
        });
        path_max.draw_next(|last| {
            Segment::Line(LineSegment {
                from: last.to(),
                to: Point2D::new(max.min_x(), max.min_y()),
            })
        });

        SegmentRule::OddEven(path_min, path_max)
    });

    drawing.extend(epoch.render_paths());

    let mut mndl = Mandala::new(380.0);

    for i in 1..=25 {
        mndl.draw_epoch(|last, _| {
            let mut ep = EpochBuilder::default()
                .center(last.center)
                .radius(last.radius - 10.0)
                .breadth(10.0)
                .segments(30 - i + 2)
                .build()
                .unwrap();

            match i.rem_euclid(3) {
                0 => {
                    ep.draw_segment(|min, max| {
                        SegmentRule::Path(Path::new(Segment::Triangle(Triangle {
                            a: min.min(),
                            b: max.center(),
                            c: Point2D::new(min.max_x(), min.min_y()),
                        })))
                    });
                }
                _ => {
                    ep.draw_segment(|min, max| {
                        SegmentRule::Path(Path::new(Segment::Arc(SvgArc {
                            from: min.min(),
                            to: Point2D::new(min.max_x(), min.min_y()),
                            radii: Vector2D::splat(
                                min.center().distance_to(max.center()).max(30.0),
                            ),
                            x_rotation: Angle::zero(),
                            flags: ArcFlags {
                                large_arc: true,
                                sweep: true,
                            },
                        })))
                    });
                }
            }
            ep
        });
    }

    drawing.extend(
        mndl.render_drawing()
            .into_iter()
            .flat_map(|e| e.into_iter())
            .map(|p| p.translate(Vector2D::new(200.0, 300.0))),
    );

    let mut app = App {
        gl: GlGraphics::new(opengl),
        drawing,
        rng: thread_rng(),
        rands: vec![],
        update_time: 0.0,
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
