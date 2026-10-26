//! Validate the actual platform shaders even on a host without a GPU.
#[test]
fn viewport_shaders_validate() {
    for (name, source) in [
        ("viewport", include_str!("../src/gpu/viewport.wgsl")),
        ("resolve", include_str!("../src/gpu/resolve.wgsl")),
        ("nv12", include_str!("../src/gpu/nv12.wgsl")),
    ] {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|e| panic!("{name}: {}", e.emit_to_string(source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("{name}: {}", e.emit_to_string(source)));
    }
}

// Evaluate the actual pure WGSL conversion, rather than a duplicate CPU formula.
// This intentionally supports only the expression subset used by ycbcr().
fn evaluate(
    expressions: &naga::Arena<naga::Expression>,
    at: naga::Handle<naga::Expression>,
    rgb: &[f64],
) -> Vec<f64> {
    use naga::{BinaryOperator as B, Expression as E, Literal as L};
    let get = |at| evaluate(expressions, at, rgb);
    match &expressions[at] {
        E::FunctionArgument(0) => rgb.to_vec(),
        E::Literal(L::F32(v)) => vec![f64::from(*v)],
        E::Literal(L::F64(v) | L::AbstractFloat(v)) => vec![*v],
        E::AccessIndex { base, index } => vec![get(*base)[*index as usize]],
        E::Compose { components, .. } => components.iter().flat_map(|at| get(*at)).collect(),
        E::Binary { op, left, right } => {
            let (a, b) = (get(*left), get(*right));
            assert_eq!(a.len(), b.len());
            a.iter()
                .zip(b)
                .map(|(a, b)| match op {
                    B::Add => a + b,
                    B::Subtract => a - b,
                    B::Multiply => a * b,
                    B::Divide => a / b,
                    _ => panic!("unsupported conversion operator {op:?}"),
                })
                .collect()
        }
        expression => panic!("unsupported conversion expression {expression:?}"),
    }
}

#[test]
fn nv12_round_trips_through_the_vendored_gpui_matrix() {
    let module = naga::front::wgsl::parse_str(include_str!("../src/gpu/nv12.wgsl")).unwrap();
    let (_, convert) = module
        .functions
        .iter()
        .find(|(_, f)| f.name.as_deref() == Some("ycbcr"))
        .unwrap();
    let returned = convert
        .body
        .iter()
        .find_map(|s| match s {
            naga::Statement::Return { value } => *value,
            _ => None,
        })
        .unwrap();
    let metal = include_str!("../../../vendor/gpui-apple/src/shaders.metal");
    let transform = metal
        .split("const float4x4 ycbcrToRGBTransform =")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let columns: Vec<Vec<f64>> = transform
        .split("float4(")
        .skip(1)
        .map(|column| {
            column
                .split(')')
                .next()
                .unwrap()
                .split(',')
                .map(|v| v.trim().trim_end_matches('f').parse().unwrap())
                .collect()
        })
        .collect();
    assert_eq!(columns.len(), 4);
    // Include neutral grays, saturated primaries and mixed colors; NV12's 8-bit
    // quantization permits ~1.5 RGB code values of error even without subsampling.
    for r in [0., 0.1, 0.5, 1.] {
        for g in [0., 0.1, 0.5, 1.] {
            for b in [0., 0.1, 0.5, 1.] {
                let rgb = [r, g, b];
                let encoded = evaluate(&convert.expressions, returned, &rgb);
                for quantize in [false, true] {
                    let mut yuv = encoded.clone();
                    for value in &mut yuv {
                        // GPUI's four-decimal matrix has inverse gamut excursions
                        // of ~5e-6 at saturated red/blue, before UNORM clamping.
                        assert!((-1e-5..=1.00001).contains(value));
                        if quantize {
                            *value = (value.clamp(0., 1.) * 255.).round() / 255.;
                        }
                    }
                    yuv.push(1.);
                    for channel in 0..3 {
                        let decoded: f64 = columns
                            .iter()
                            .zip(&yuv)
                            .map(|(column, v)| column[channel] * v)
                            .sum();
                        assert!(
                            (decoded - rgb[channel]).abs() < if quantize { 0.006 } else { 1e-6 },
                            "{rgb:?} -> {yuv:?} -> {decoded}"
                        );
                    }
                }
            }
        }
    }
}
