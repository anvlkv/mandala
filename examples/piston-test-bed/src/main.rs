use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;

use mandala::{
    Angle, ArcFlags, EpochBuilder, Mandala, Path, Point2D, Segment, SegmentRule, SvgArc, Triangle,
    Vector2D,
};
use piston::{Button, PressEvent, UpdateArgs, UpdateEvent};

pub struct App {
    gl: GlGraphics,
    mandala: Mandala,
    drawing: Vec<Path>,
    tick: bool,
    resize: Option<(f64, f64)>,
    size: (f64, f64),
}

const SIZE: u32 = 800;
const SPACE: f64 = 20.0;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        if args.window_size[0] != self.size.0 || args.window_size[1] != self.size.1 {
            self.resize = Some((args.window_size[0], args.window_size[1]));
        }

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.7];

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);

            let transform = c.transform.trans(SPACE / 2.0, SPACE / 2.0);

            if self.tick {
                circle_arc(WHITE, 2.0, 0.0, 8.0, [0.0, 0.0, 8.0, 8.0], transform, gl);
                self.tick = false;
            } else {
                self.tick = true;
            }

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

    fn btn(&mut self, _: Button) {
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
                let mut path = Path::new(Segment::Arc(SvgArc {
                    from: min.min(),
                    to: Point2D::new(max.max_x(), min.min_y()),
                    radii: Vector2D::new(
                        (max.max_x() - min.min_x()) / 2.0,
                        (max.max_y() - min.min_y()) / 2.0,
                    ),
                    x_rotation: Angle::radians(0.0),
                    flags: ArcFlags::default(),
                }));

                path.draw_next(|last| {
                    Segment::Triangle(Triangle {
                        a: last.to(),
                        b: Point2D::new(max.max_x(), max.max_y()),
                        c: Point2D::new(min.min_x(), max.max_y()),
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
    }

    fn update(&mut self, _: &UpdateArgs) {
        if let Some(new_size) = self.resize.take() {
            let size = new_size.0.min(new_size.1) - SPACE;
            self.mandala.resize(size);
            self.size = new_size;
            self.drawing = self.mandala.render_drawing();
        }
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

    let mut app = App {
        gl: GlGraphics::new(opengl),
        mandala: Mandala::new(SIZE as f64 - SPACE),
        drawing: vec![],
        tick: true,
        resize: None,
        size: (SIZE as f64, SIZE as f64),
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.press_args() {
            app.btn(args)
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
