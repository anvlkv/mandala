use leptos::*;
use leptos_meta::*;
use leptos_use::{use_interval, UseIntervalReturn};
use mandala::*;
use rand::rngs::SmallRng;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let center = Point::new(400.0, 400.0);
    let sweep = Angle::frac_pi_3();
    let UseIntervalReturn { counter, .. } = use_interval(50);

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

    let cubic_renderer = |_rng: &mut SmallRng, _| {
        Path::new(PathSegment::CubicCurve(CubicCurve {
            from: Point::new(0.0, 0.0),
            ctrl1: Point::new(3.0, 5.0),
            ctrl2: Point::new(-7.0, -5.0),
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

    let (segments, set_segments) = create_signal({
        let segment_lines = MandalaSegmentBuilder::default()
            .drawing(vec![SegmentDrawing::Path(pattern)])
            .angle_base(Angle::zero())
            .sweep(sweep)
            .center(center)
            .r_base(80.0)
            .breadth(0.6)
            .build()
            .unwrap();

        let segment_arcs = MandalaSegmentBuilder::default()
            .drawing(vec![SegmentDrawing::Path(arc_pattern)])
            .angle_base(sweep)
            .sweep(sweep)
            .center(center)
            .r_base(80.0)
            .breadth(0.4)
            .build()
            .unwrap();

        let segment_cubics = MandalaSegmentBuilder::default()
            .drawing(vec![SegmentDrawing::Path(cubic_pattern)])
            .angle_base(sweep * 2.0)
            .sweep(sweep)
            .center(center)
            .r_base(80.0)
            .breadth(0.6)
            .build()
            .unwrap();

        let segment_quads = MandalaSegmentBuilder::default()
            .drawing(vec![SegmentDrawing::Path(quad_pattern)])
            .angle_base(sweep * 3.0)
            .sweep(sweep)
            .center(center)
            .r_base(80.0)
            .breadth(0.6)
            .build()
            .unwrap();

        let radius = 100.0;

        let mut epoch = EpochBuilder::default()
            .center(center.add_size(&Size::new(300.0, 0.0)))
            .layout(EpochLayout::Circle {
                radius: radius * 0.52,
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

        let mut draw_fn = |args: &DrawArgs| {
            MandalaSegmentBuilder::default()
                .angle_base(args.start_angle)
                .sweep(Angle::frac_pi_4())
                .center(args.center)
                .r_base(radius)
                .breadth(0.45)
                .drawing(vec![SegmentDrawing::Path(pattern.clone())])
                .build()
                .unwrap()
        };

        epoch.draw_fill(&mut draw_fn);

        let mut mndl = Mandala::new(BBox::new(Point::zero(), Point::splat(300.0)));

        mndl.draw_epoch(|_, _| epoch.translate(Vector::new(-100.0, 300.0)).scale(0.75));

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
            let mut ep = epoch.translate(Vector::new(-100.0, 300.0)).scale(1.75);
            if let Some(sg) = ep.segments.last_mut() {
                sg.drawing.push(SegmentDrawing::Mandala {
                    mandala: mndl_2.clone(),
                    placement_box: BBox::new(Point::splat(0.0), Point::splat(100.0)),
                })
            }
            ep
        });

        (
            segment_lines,
            segment_arcs,
            segment_cubics,
            segment_quads,
            epoch,
            mndl,
        )
    });

    create_effect(move |_| {
        let c = counter.get();

        set_segments.update(|segments| {
            let (segment_lines, segment_arcs, segment_cubics, segment_quads, ep, _) = segments;

            segment_quads.angle_base += Angle::degrees(1.0);
            segment_cubics.angle_base += Angle::degrees(1.0);
            segment_arcs.angle_base += Angle::degrees(1.0);
            segment_lines.angle_base += Angle::degrees(1.0);

            if c.rem_euclid(9) == 0 {
                ep.layout = match ep.layout {
                    EpochLayout::Circle { radius } => EpochLayout::Ellipse {
                        radii: Size::new(radius, radius / 2.0),
                    },
                    EpochLayout::Ellipse { radii } => EpochLayout::Polygon {
                        n_sides: 7,
                        radius: radii.width,
                        start: Angle::frac_pi_4() * c as f64,
                    },
                    EpochLayout::Polygon { radius, .. } => EpochLayout::Rectangle {
                        rect: Size::new(radius, radius * 2.0),
                    },
                    EpochLayout::Rectangle { rect } => EpochLayout::Circle { radius: rect.width },
                };
            }
        })
    });

    view! {
        <Html lang="en" dir="ltr" attr:data-theme="light"/>

        // sets the document title
        <Title text="Mandala wasm preview"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <svg>
            {move || drawing.clone().iter().map(|p| view!{
                <path d={p.translate(Vector::new(0.0,  15.0)).to_svg_path_d()} stroke="orange"/>
            }).collect_view()}

            {move || {
                let (
                    segment_quads,
                    segment_cubics,
                    segment_arcs,
                    segment_lines,
                    epoch,
                    mndl
                ) = segments.get();

                let v1 = segment_quads.render().iter().map(|p| view!{
                    <path d={p.to_svg_path_d()} stroke="blue"/>
                }).collect_view();
                let v2 = segment_arcs.render().iter().map(|p| view!{
                    <path d={p.to_svg_path_d()} stroke="red"/>
                }).collect_view();
                let v3 = segment_cubics.render().iter().map(|p| view!{
                    <path d={p.to_svg_path_d()} stroke="green"/>
                }).collect_view();
                let v4 = segment_lines.render().iter().map(|p| view!{
                    <path d={p.to_svg_path_d()} stroke="purple"/>
                }).collect_view();
                let v5 = epoch.render().iter().map(|p| view!{
                    <path d={p.to_svg_path_d()} stroke="white"/>
                }).collect_view();
                let v6= mndl.render().iter().map(|p| view!{
                    <path d={p.to_svg_path_d()} stroke="yellow" fill="none"/>
                }).collect_view();

                view!{
                    {v1}
                    {v2}
                    {v3}
                    {v4}
                    {v5}
                    {v6}
                }
            }}

            {(3..=10).map(|i| {
                let d = Path::polygon(i, Rect::new(Point::zero(), Size::new(50.0, 50.0)), Angle::zero()).to_svg_path_d();
                let t =format!("translate(900.0, {})", (i - 2) * 60);
                log::debug!("d: {d}");
                view!{
                    <g transform={t}>
                        <path d={d} stroke="white"/>
                    </g>
                }
            }).collect_view()}

        {(3..=10).map(|i| {
            let r = i  as f64 * 5.07;
            let pt = Point::new(
                1100.0,
                (i) as f64 * (r + 5.07)
            );
            let d = Path::circle(pt, r).to_svg_path_d();
            log::debug!("r: {r} d: {d} pt: {pt:?}");
            view!{
                <path d={d} stroke="white"/>
            }
        }).collect_view()}
        </svg>
    }
}
