use glutin_window::GlutinWindow as Window;
use mandala::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use piston::{UpdateArgs, UpdateEvent};

const SIZE: u32 = 800;

pub struct App {
    gl: GlGraphics,
    drawing: Vec<Path>,
}

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

            for p in self.drawing.clone() {
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

    fn update(&mut self, _: &UpdateArgs) {}
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

    let drawing = Vec::new();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        drawing,
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
