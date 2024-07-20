use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;

use mandala::Mandala;
use piston::{Button, PressEvent, UpdateArgs, UpdateEvent};

pub struct App {
    gl: GlGraphics,
    mandala: Mandala,
    tick: bool,
    resize: Option<(f64, f64)>,
    size: (f64, f64),
}

const SIZE: u32 = 1000;
const SPACE: f64 = 20.0;
const LINE_THICKNESS: f64 = 0.75;

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

            let total = self.mandala.drawing.len() as f32;

            for (i, e) in self.mandala.drawing.clone().into_iter().enumerate() {
                for (j, p) in e.into_iter().enumerate() {
                    let color: [f32; 4] = [
                        1.0 / (i + 1) as f32,
                        1.0 / (j + 1) as f32,
                        1.0,
                        (1.0 / total) * (i + 1) as f32,
                    ];
                    for s in p.into_iter() {
                        for l in s.flattened() {
                            line(
                                color,
                                LINE_THICKNESS,
                                [l.from.x, l.from.y, l.to.x, l.to.y],
                                transform,
                                gl,
                            )
                        }
                    }
                }
            }
        });
    }

    fn btn(&mut self, _: Button) {
        self.mandala.generate_epoch();
    }

    fn update(&mut self, _: &UpdateArgs) {
        if let Some(new_size) = self.resize.take() {
            let size = new_size.0.min(new_size.1) - SPACE;
            self.mandala.resize(size);
            self.size = new_size;
        } else {
            self.mandala.generate_epoch();
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("mandala / piston 2D preview", [SIZE, SIZE])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        mandala: Mandala::new(SIZE as f64 - SPACE),
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

    println!("final mandala: {:#?}", app.mandala);
}
