use mandala::VectorValuedFn;
use mndl_macro::valued_struct;

#[test]
fn test_circle() {
    use std::f32::consts::PI;

    valued_struct! {
        #[derive(Debug, Clone)]
        struct Circle {
            center: [f32; 3],
            radii: [f32; 2]
        },
        x(t) -> self.center[0] + self.radii[0] * (t * PI * 2.0).cos(),
        y(t) -> self.center[1] + self.radii[1] * (t * PI * 2.0).sin(),
        z(_) -> self.center[2]
    }

    let example = Circle {
        center: [0.0, 0.0, 0.0],
        radii: [20.0, 20.0],
    };
    let samples = example.sample_evenly(100).collect::<Vec<_>>();

    insta::assert_debug_snapshot!(samples);

    insta::assert_debug_snapshot!(example.to_shader_code());
}
