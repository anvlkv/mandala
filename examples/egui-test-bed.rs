use egui::{Ui, Visuals};
use egui_plotter::EguiBackend;
use mandala::*;
use plotters::{
    coord::{self, types},
    prelude::*,
};

#[cfg(feature = "f32")]
const SIZE: f32 = 1000.0;

#[cfg(feature = "f64")]
const SIZE: f64 = 1000.0;

#[cfg(feature = "f64")]
type Coord = types::RangedCoordf64;

#[cfg(feature = "f32")]
type Coord = types::RangedCoordf32;

#[cfg(feature = "3d")]
type ChartType<'a> =
    ChartContext<'a, EguiBackend<'a>, coord::ranged3d::Cartesian3d<Coord, Coord, Coord>>;

#[cfg(feature = "2d")]
type ChartType<'a> = ChartContext<'a, EguiBackend<'a>, coord::cartesian::Cartesian2d<Coord, Coord>>;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mandala paths test bed",
        options,
        Box::new(|cc| Box::new(MandalaApp::new(cc))),
    )
}

#[derive(Default, PartialEq, Eq)]
enum Tabs {
    Arcs,
    Curves,
    #[default]
    Lines,
    Path(PathExample),
}

#[derive(PartialEq, Eq)]
enum PathExample {
    Segments,
    Rect,
    Polygon,
}

#[derive(Default)]
struct MandalaApp {
    tab: Tabs,
    line: Line,
    line_segment: LineSegment,
    arc: SweepArc,
    arc_segment: ArcSegment,
    cubic: CubicCurve,
    quad: QuadraticCurve,
    rect_origin: GlVec,
    rect_size: GlVec,
    polygon_center: GlVec,
    polygon_size: GlVec,
    polygon_sides: usize,
    polygon_start_angle: Angle,
}

impl MandalaApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Disable feathering as it causes artifacts
        let context = &cc.egui_ctx;

        context.tessellation_options_mut(|tess_options| {
            tess_options.feathering = false;
        });

        // Also enable light mode
        context.set_visuals(Visuals::light());

        Self::default()
    }

    fn arc_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("SweepArc");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.arc.center.x, 0.0..=SIZE).text("center x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.arc.center.y, 0.0..=SIZE).text("center y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.arc.center.z, 0.0..=SIZE).text("center z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.arc.radius.x, 0.0..=SIZE).text("radius x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.arc.radius.y, 0.0..=SIZE).text("radius y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.arc.radius.z, 0.0..=SIZE).text("radius z"),
                        );
                    });
                });
                ui.add(
                    egui::Slider::new(
                        self.arc.start_angle.radians_mut(),
                        0.0..=Angle::TAU.to_radians(),
                    )
                    .text("start_angle"),
                );
                ui.add(
                    egui::Slider::new(
                        self.arc.sweep_angle.radians_mut(),
                        0.0..=Angle::TAU.to_radians(),
                    )
                    .text("sweep_angle"),
                );
            });

            ui.vertical(|ui| {
                ui.heading("ArcSegment");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.start.x, 0.0..=SIZE)
                                .text("start x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.start.y, 0.0..=SIZE)
                                .text("start y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.start.z, 0.0..=SIZE)
                                .text("start z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.end.x, 0.0..=SIZE)
                                .text("end x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.end.y, 0.0..=SIZE)
                                .text("end y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.end.z, 0.0..=SIZE)
                                .text("end z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.radius.x, 0.0..=SIZE)
                                .text("radius x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.radius.y, 0.0..=SIZE)
                                .text("radius y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.arc_segment.radius.z, 0.0..=SIZE)
                                .text("radius z"),
                        );
                    });
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Checkbox::new(
                        &mut self.arc_segment.large_arc,
                        "large arc",
                    ));
                    ui.add(egui::Checkbox::new(
                        &mut self.arc_segment.poz_angle,
                        "positive arc",
                    ));
                });
            })
        });
    }

    fn line_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Line");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.line.origin.x, 0.0..=SIZE).text("origin x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.line.origin.y, 0.0..=SIZE).text("origin y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.line.origin.z, 0.0..=SIZE).text("origin z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.line.direction.x, 0.0..=SIZE)
                                .text("direction x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.line.direction.y, 0.0..=SIZE)
                                .text("direction y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.line.direction.z, 0.0..=SIZE)
                                .text("direction z"),
                        );
                    });
                });
            });

            ui.vertical(|ui| {
                ui.heading("LineSegment");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.line_segment.start.x, 0.0..=SIZE)
                                .text("start x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.line_segment.start.y, 0.0..=SIZE)
                                .text("start y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.line_segment.start.z, 0.0..=SIZE)
                                .text("start z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.line_segment.end.x, 0.0..=SIZE)
                                .text("end x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.line_segment.end.y, 0.0..=SIZE)
                                .text("end y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.line_segment.end.z, 0.0..=SIZE)
                                .text("end z"),
                        );
                    });
                });
            });
        });
    }

    fn curve_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Quadratic curve");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.quad.start.x, 0.0..=SIZE).text("start x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.quad.start.y, 0.0..=SIZE).text("start y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.quad.start.z, 0.0..=SIZE).text("start z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.quad.control.x, 0.0..=SIZE)
                                .text("control x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.quad.control.y, 0.0..=SIZE)
                                .text("control y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.quad.control.z, 0.0..=SIZE)
                                .text("control z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(egui::Slider::new(&mut self.quad.end.x, 0.0..=SIZE).text("end x"));
                        ui.add(egui::Slider::new(&mut self.quad.end.y, 0.0..=SIZE).text("end y"));
                        #[cfg(feature = "3d")]
                        ui.add(egui::Slider::new(&mut self.quad.end.z, 0.0..=SIZE).text("end z"));
                    });
                });
            });
            ui.vertical(|ui| {
                ui.heading("Cubic curve");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.cubic.start.x, 0.0..=SIZE).text("start x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.cubic.start.y, 0.0..=SIZE).text("start y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.cubic.start.z, 0.0..=SIZE).text("start z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.cubic.control1.x, 0.0..=SIZE)
                                .text("control 1 x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.cubic.control1.y, 0.0..=SIZE)
                                .text("control 1 y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.cubic.control1.z, 0.0..=SIZE)
                                .text("control 1 z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.cubic.control2.x, 0.0..=SIZE)
                                .text("control 2 x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.cubic.control2.y, 0.0..=SIZE)
                                .text("control 2 y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.cubic.control2.z, 0.0..=SIZE)
                                .text("control 2 z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(egui::Slider::new(&mut self.cubic.end.x, 0.0..=SIZE).text("end x"));
                        ui.add(egui::Slider::new(&mut self.cubic.end.y, 0.0..=SIZE).text("end y"));
                        #[cfg(feature = "3d")]
                        ui.add(egui::Slider::new(&mut self.cubic.end.z, 0.0..=SIZE).text("end z"));
                    });
                });
            });
        });
    }

    fn rect_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Rect");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.rect_origin.x, 0.0..=SIZE).text("origin x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.rect_origin.y, 0.0..=SIZE).text("origin y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.rect_origin.z, 0.0..=SIZE).text("origin z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(egui::Slider::new(&mut self.rect_size.x, 0.0..=SIZE).text("size x"));
                        ui.add(egui::Slider::new(&mut self.rect_size.y, 0.0..=SIZE).text("size y"));
                        #[cfg(feature = "3d")]
                        ui.add(egui::Slider::new(&mut self.rect_size.z, 0.0..=SIZE).text("size z"));
                    });
                });
            });
        });
    }

    fn polygon_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Polygon");
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.polygon_center.x, 0.0..=SIZE)
                                .text("center x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.polygon_center.y, 0.0..=SIZE)
                                .text("center y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.polygon_center.z, 0.0..=SIZE)
                                .text("center z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.polygon_size.x, 0.0..=SIZE).text("size x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.polygon_size.y, 0.0..=SIZE).text("size y"),
                        );
                        #[cfg(feature = "3d")]
                        ui.add(
                            egui::Slider::new(&mut self.polygon_size.z, 0.0..=SIZE).text("size z"),
                        );
                    });
                    ui.vertical(|ui| {
                        ui.add(egui::Slider::new(&mut self.polygon_sides, 3..=250).text("n sides"));
                        ui.add(
                            egui::Slider::new(
                                self.polygon_start_angle.radians_mut(),
                                Angle::ZERO.to_radians()..=Angle::TAU.to_radians(),
                            )
                            .text("start angle"),
                        );
                    });
                });
            });
        });
    }

    fn plot_arc(&self, chart: &mut ChartType) {
        chart
            .draw_series(LineSeries::new(make_series(self.arc), &BLUE))
            .unwrap()
            .label("Arc");
        chart
            .draw_series(LineSeries::new(make_series(self.arc_segment), &RED))
            .unwrap()
            .label("ArcSegment");
    }

    fn plot_curves(&self, chart: &mut ChartType) {
        chart
            .draw_series(LineSeries::new(make_series(self.cubic), &BLUE))
            .unwrap()
            .label("Cubic");
        chart
            .draw_series(LineSeries::new(make_series(self.quad), &RED))
            .unwrap()
            .label("Quadratic");
    }

    fn plot_lines(&self, chart: &mut ChartType) {
        chart
            .draw_series(LineSeries::new(make_series(self.line), &BLUE))
            .unwrap()
            .label("Line");
        chart
            .draw_series(LineSeries::new(make_series(self.line_segment), &RED))
            .unwrap()
            .label("LineSegment");
    }

    fn plot_path(&self, chart: &mut ChartType) {
        let path = mandala::Path::new(vec![
            Box::new(self.arc.clone()),
            Box::new(self.arc_segment.clone()),
            Box::new(self.line_segment.clone()),
            Box::new(self.cubic.clone()),
            Box::new(self.quad.clone()),
        ]);
        chart
            .draw_series(LineSeries::new(make_series(path), &BLUE))
            .unwrap()
            .label("Path");
    }

    fn plot_rect(&self, chart: &mut ChartType) {
        let plot = mandala::Path::rectangle(self.rect_origin.into(), self.rect_size.into());
        chart
            .draw_series(LineSeries::new(make_series(plot), &BLUE))
            .unwrap()
            .label("Rect");
    }
    fn plot_polygon(&self, chart: &mut ChartType) {
        let plot = mandala::Path::polygon(
            self.polygon_center.into(),
            self.polygon_size.into(),
            self.polygon_sides,
            self.polygon_start_angle,
        );

        chart
            .draw_series(LineSeries::new(make_series(plot), &BLUE))
            .unwrap()
            .label("Polygon");
    }
}

impl eframe::App for MandalaApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mandala paths test bed");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tabs::Lines, "Lines");
                ui.selectable_value(&mut self.tab, Tabs::Curves, "Curves");
                ui.selectable_value(&mut self.tab, Tabs::Arcs, "Arcs");
                ui.selectable_value(&mut self.tab, Tabs::Path(PathExample::Segments), "Path");
                ui.selectable_value(&mut self.tab, Tabs::Path(PathExample::Rect), "Rect");
                ui.selectable_value(&mut self.tab, Tabs::Path(PathExample::Polygon), "Polygon");
            });
            match &self.tab {
                Tabs::Arcs => {
                    self.arc_settings(ui);
                }
                Tabs::Curves => {
                    self.curve_settings(ui);
                }
                Tabs::Lines => {
                    self.line_settings(ui);
                }
                Tabs::Path(p) => match p {
                    PathExample::Segments => {
                        ui.vertical(|ui| {
                            ui.label("the test path is composed of all the other examples");
                            ui.label("configure on other tabs to see result here");
                        });
                    }
                    PathExample::Rect => {
                        self.rect_settings(ui);
                    }
                    PathExample::Polygon => {
                        self.polygon_settings(ui);
                    }
                },
            }

            ui.add_space(10.0);

            let root = EguiBackend::new(ui).into_drawing_area();

            #[cfg(feature = "3d")]
            let mut chart = ChartBuilder::on(&root)
                .margin(15)
                .margin_top(200)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_3d(0.0..SIZE, 0.0..SIZE, 0.0..SIZE)
                .unwrap();

            #[cfg(feature = "2d")]
            let mut chart = ChartBuilder::on(&root)
                .margin(15)
                .margin_top(200)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(0.0..SIZE, 0.0..SIZE)
                .unwrap();

            #[cfg(feature = "3d")]
            chart
                .configure_axes()
                .light_grid_style(BLACK.mix(0.15))
                .max_light_lines(3)
                .draw()
                .unwrap();

            #[cfg(feature = "2d")]
            chart.configure_mesh().max_light_lines(3).draw().unwrap();

            match &self.tab {
                Tabs::Arcs => {
                    self.plot_arc(&mut chart);
                }
                Tabs::Curves => {
                    self.plot_curves(&mut chart);
                }
                Tabs::Lines => {
                    self.plot_lines(&mut chart);
                }
                Tabs::Path(p) => match p {
                    PathExample::Segments => {
                        self.plot_path(&mut chart);
                    }
                    PathExample::Rect => {
                        self.plot_rect(&mut chart);
                    }
                    PathExample::Polygon => {
                        self.plot_polygon(&mut chart);
                    }
                },
            }
        });
    }
}

#[cfg(feature = "3d")]
type SeriesItem = (Float, Float, Float);
#[cfg(feature = "2d")]
type SeriesItem = (Float, Float);

fn make_series<V>(seg: V) -> impl Iterator<Item = SeriesItem>
where
    V: VectorValuedFn,
{
    seg.sample_optimal().into_iter().map(|v| {
        #[cfg(feature = "3d")]
        return (v.x, v.y, v.z);
        #[cfg(feature = "2d")]
        return (v.x, v.y);
    })
}
