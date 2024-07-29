use eframe::egui::{self, Widget};
use egui_plot::{PlotPoint, PlotPoints};
use mandala::*;
use uuid::Uuid;

const SIZE: f64 = 800.0;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([SIZE as f32, SIZE as f32]),
        ..Default::default()
    };
    eframe::run_native(
        "Mandala test bed",
        options,
        Box::new(|cc| Ok(Box::<MandalaApp>::default())),
    )
}

struct Settings {
    layout_radius: f64,
    segment_radius: f64,
    segment_breadth: f64,
    segment_sweep: f64,
}

struct MandalaApp {
    artboard: Artboard,
    settings: Settings,
    id: Uuid,
}

impl Default for MandalaApp {
    fn default() -> Self {
        let mut artboard = Artboard::new(BBox::new(Point::zero(), Point::new(SIZE, SIZE)));
        let settings = Settings {
            layout_radius: 50.0,
            segment_radius: 100.0,
            segment_breadth: 0.15,
            segment_sweep: Angle::frac_pi_4().radians,
        };

        let mut id = None;

        artboard.draw_mandala(&mut |b| {
            let mut m = Mandala::new(b.clone());
            id = Some(m.id);

            m.draw_epoch(|last, b| {
                let mut ep = EpochBuilder::default()
                    .center(b.center())
                    .layout(EpochLayout::Circle {
                        radius: settings.layout_radius,
                    })
                    .outline(true)
                    .build()
                    .unwrap();

                ep.draw_fill(&mut |args| {
                    let mut p = Path::new(PathSegment::QuadraticCurve(QuadraticCurve {
                        from: Point::zero(),
                        to: Point::new(50.0, 100.0),
                        ctrl: Point::new(45.0, 75.0),
                    }));
                    p.draw_next(|last| {
                        PathSegment::QuadraticCurve(QuadraticCurve {
                            from: last.to(),
                            to: Point::new(100.0, 0.0),
                            ctrl: Point::new(55.0, 75.0),
                        })
                    });
                    p.draw_next(|last| {
                        PathSegment::Line(Line {
                            from: last.to(),
                            to: Point::zero(),
                        })
                    });

                    MandalaSegmentBuilder::default()
                        .center(b.center())
                        .r_base(settings.segment_radius)
                        .angle_base(Angle::zero())
                        .sweep(Angle::radians(settings.segment_sweep))
                        .breadth(settings.segment_breadth)
                        .draw(SegmentDrawing::Path(vec![p]))
                        .build()
                        .unwrap()
                });

                ep
            });

            m
        });

        Self {
            artboard,
            settings,
            id: id.unwrap(),
        }
    }
}

impl eframe::App for MandalaApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mandala test bed");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    if ui
                        .add(
                            egui::Slider::new(&mut self.settings.layout_radius, 1.0..=SIZE / 2.0)
                                .text("layout radius"),
                        )
                        .changed()
                    {
                        self.artboard.update(&self.id)(|mndl| {
                            if let Some(ep) = mndl.epochs.first_mut() {
                                ep.layout = EpochLayout::Circle {
                                    radius: self.settings.layout_radius,
                                }
                            }
                        });
                    }
                    if ui
                        .add(
                            egui::Slider::new(&mut self.settings.segment_radius, 1.0..=SIZE / 2.0)
                                .text("segment radius"),
                        )
                        .changed()
                    {
                        self.artboard.update(&self.id)(|mndl| {
                            if let Some(ep) = mndl.epochs.first_mut() {
                                ep.segments
                                    .iter_mut()
                                    .for_each(|s| s.r_base = self.settings.segment_radius);
                            }
                        });
                    }
                });
                ui.vertical(|ui| {
                    if ui
                        .add(
                            egui::Slider::new(
                                &mut self.settings.segment_sweep,
                                Angle::zero().radians..=Angle::two_pi().radians,
                            )
                            .text("segment sweep"),
                        )
                        .changed()
                    {
                        self.artboard.update(&self.id)(|mndl| {
                            if let Some(ep) = mndl.epochs.first_mut() {
                                let mut s = ep.segments.first().cloned().unwrap();
                                ep.segments.clear();
                                s.sweep = Angle::radians(self.settings.segment_sweep);

                                ep.draw_fill(&mut |_| s.clone());
                            }
                        });
                    }
                    if ui
                        .add(
                            egui::Slider::new(&mut self.settings.segment_breadth, 0.0..=1.0)
                                .text("segment breadth"),
                        )
                        .changed()
                    {
                        self.artboard.update(&self.id)(|mndl| {
                            if let Some(ep) = mndl.epochs.first_mut() {
                                ep.segments
                                    .iter_mut()
                                    .for_each(|s| s.breadth = self.settings.segment_breadth);
                            }
                        });
                    }
                });
            });
            ui.add_space(10.0);
            egui_plot::Plot::new("Mandala").show(ui, |p_ui| {
                for path in self.artboard.view_paths() {
                    let pts = path
                        .flattened()
                        .into_iter()
                        .flat_map(|l| [[l.from.x, l.from.y], [l.to.x, l.to.y]])
                        .collect::<Vec<_>>();

                    p_ui.line(egui_plot::Line::new(PlotPoints::new(pts)));
                }
            });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Increment").clicked() {
            //     self.age += 1;
            // }
            // ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}
