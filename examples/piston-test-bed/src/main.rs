use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use mandala::{
    CubicBezierSegment, EpochBuilder, LineSegment, Mandala, Path, Point2D, Segment, SegmentRule,
};

pub struct App {
    gl: GlGraphics,
    mandala: Mandala,
    drawing: Vec<Path>,
    tick: bool,
    scale: f64,
}

const SIZE: u32 = 800;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.7];

        // let square = rectangle::square(0.0, 0.0, 50.0);
        // let rotation = self.rotation;
        // let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        // let (x, y) = (0.0, 0.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c.transform.trans(10.0, 10.0);

            if self.tick {
                circle_arc(WHITE, 2.0, 0.0, 8.0, [0.0, 0.0, 8.0, 8.0], transform, gl);
                self.tick = false;
            } else {
                self.tick = true;
            }

            let transform = transform.scale(self.scale, self.scale).trans(
                args.window_size[0] / 2.0 - (args.window_size[0] / 2.0) * self.scale,
                args.window_size[1] / 2.0 - (args.window_size[1] / 2.0) * self.scale,
            );

            // line(WHITE, 1.0, [0.0, 0.0, 400.0, 400.0], transform, gl)

            for (i, p) in self.drawing.clone().into_iter().enumerate() {
                for (j, s) in p.into_iter().enumerate() {
                    let color: [f32; 4] = [1.0 / (i + 1) as f32, 1.0 / (j + 1) as f32, 1.0, 1.0];
                    match s {
                        mandala::Segment::Line(l) => line(
                            color,
                            0.25,
                            [l.from.x, l.from.y, l.to.x, l.to.y],
                            transform,
                            gl,
                        ),
                        mandala::Segment::Arc(l) => {
                            l.for_each_flattened(0.1, &mut |f| {
                                line(
                                    color,
                                    0.25,
                                    [f.from.x, f.from.y, f.to.x, f.to.y],
                                    transform,
                                    gl,
                                );
                            })
                            // let arc = l.to_arc();
                            // let bx = arc.bounding_range_x();
                            // let by = arc.bounding_range_y();

                            // circle_arc(
                            //     CLR,
                            //     0.25,
                            //     arc.start_angle.radians,
                            //     arc.end_angle().radians,
                            //     [bx.0, by.0, bx.1, by.1],
                            //     transform,
                            //     gl,
                            // )
                        }
                        mandala::Segment::Triangle(l) => {
                            line(color, 0.25, [l.a.x, l.a.y, l.b.x, l.b.y], transform, gl);
                            line(color, 0.25, [l.b.x, l.b.y, l.c.x, l.c.y], transform, gl);
                        }
                        mandala::Segment::QuadraticCurve(l) => {
                            l.for_each_flattened(0.1, &mut |f| {
                                line(
                                    color,
                                    0.25,
                                    [f.from.x, f.from.y, f.to.x, f.to.y],
                                    transform,
                                    gl,
                                );
                            })
                        }
                        mandala::Segment::CubicCurve(l) => l.for_each_flattened(0.1, &mut |f| {
                            line(
                                color,
                                0.25,
                                [f.from.x, f.from.y, f.to.x, f.to.y],
                                transform,
                                gl,
                            );
                        }),
                    }
                }
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.mandala.draw_epoch(|last| {
            //
            let mut epoch = EpochBuilder::default()
                .radius(last.radius - last.breadth - 1.0)
                .breadth(last.radius / 4.0)
                .segments(last.segments + 8)
                .center(last.center)
                .build()
                .unwrap();

            epoch.draw_segment(|min, max| {
                let mut path = Path::new(Segment::CubicCurve(CubicBezierSegment {
                    from: Point2D::new(0.0, 0.0),
                    ctrl1: Point2D::new(min.max_x() / 2.0, min.max_y()),
                    ctrl2: Point2D::new(max.max_x() / 2.0, max.max_y()),
                    to: Point2D::new(min.width(), 0.0),
                }));
                path.draw_next(|last| {
                    Segment::Line(LineSegment {
                        from: last.to(),
                        to: Point2D::new(0.0, 0.0),
                    })
                });
                path.draw_next(|last| {
                    Segment::Line(LineSegment {
                        from: last.to(),
                        to: Point2D::new(0.0, min.max_y()),
                    })
                });
                path.draw_next(|last| {
                    Segment::Line(LineSegment {
                        from: last.to(),
                        to: Point2D::new(min.max_x(), min.max_y()),
                    })
                });
                path.draw_next(|last| {
                    Segment::Line(LineSegment {
                        from: last.to(),
                        to: Point2D::new(min.max_x(), min.min_y()),
                    })
                });
                path.draw_next(|last| {
                    Segment::Line(LineSegment {
                        from: last.to(),
                        to: Point2D::new(max.max_x(), max.min_y()),
                    })
                });
                path.draw_next(|last| {
                    Segment::Line(LineSegment {
                        from: last.to(),
                        to: Point2D::new(max.max_x(), max.max_y()),
                    })
                });
                path.draw_next(|last| {
                    Segment::Line(LineSegment {
                        from: last.to(),
                        to: Point2D::new(max.min_x(), max.max_y()),
                    })
                });
                path.draw_next(|last| {
                    Segment::Line(LineSegment {
                        from: last.to(),
                        to: Point2D::new(max.min_x(), max.min_y()),
                    })
                });

                SegmentRule::Path(path)
            });

            epoch
        });
        self.drawing.extend(
            self.mandala
                .epochs
                .last()
                .map(|e| e.render_paths())
                .unwrap_or_default(),
        );

        self.scale += 0.01;
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [SIZE, SIZE])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        mandala: Mandala::new((SIZE - 20) as f64),
        drawing: vec![],
        tick: true,
        scale: 1.0,
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