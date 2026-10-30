//! Validate the actual platform shaders even on a host without a GPU.
#[test]
fn viewport_shaders_validate() {
    for (name, source) in [
        ("viewport", include_str!("../gpu/viewport.wgsl")),
        ("shadow fallback", include_str!("../gpu/shadow_map.wgsl")),
        ("resolve", include_str!("../gpu/resolve.wgsl")),
        ("nv12", include_str!("../gpu/nv12.wgsl")),
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

#[test]
fn both_gpu_lighting_paths_return_core_ambient_before_shadow_work() {
    for source in [
        include_str!("../gpu/viewport.wgsl"),
        include_str!("../gpu/shadow_map.wgsl"),
    ] {
        let module = naga::front::wgsl::parse_str(source).unwrap();
        let (_, function) = module
            .functions
            .iter()
            .find(|(_, f)| f.name.as_deref() == Some("surface_illumination"))
            .unwrap();
        // Only expression evaluation may precede the guard. No calls (including
        // shadow traversal/sampling) or stores are allowed before this early exit.
        let first = function
            .body
            .iter()
            .find(|s| !matches!(s, naga::Statement::Emit(_)))
            .unwrap();
        let naga::Statement::If {
            condition,
            accept,
            reject,
        } = first
        else {
            panic!("ambient guard must precede lighting work")
        };
        assert!(reject.is_empty());
        let naga::Expression::Binary {
            op: naga::BinaryOperator::NotEqual,
            left,
            right,
        } = function.expressions[*condition]
        else {
            panic!("ambient flag guard")
        };
        assert_eq!(evaluate(&function.expressions, right, &[]), [0.0]);
        let naga::Expression::Load { pointer } = function.expressions[left] else {
            panic!("uniform flag")
        };
        let naga::Expression::AccessIndex { base, index: 3 } = function.expressions[pointer] else {
            panic!("light.w flag")
        };
        let naga::Expression::AccessIndex { index: 2, .. } = function.expressions[base] else {
            panic!("globals.light member")
        };
        let returned = accept
            .iter()
            .find_map(|s| match s {
                naga::Statement::Return { value } => *value,
                _ => None,
            })
            .unwrap();
        assert_eq!(
            evaluate(&function.expressions, returned, &[]),
            [f64::from(crate::AMBIENT_ILLUMINATION as f32)]
        );
    }
}

// Evaluate pure shader math, rather than testing a duplicate CPU formula.
// This supports the expression subset used by conversion and shadow gradients.
fn evaluate(
    expressions: &naga::Arena<naga::Expression>,
    at: naga::Handle<naga::Expression>,
    inputs: &[Vec<f64>],
) -> Vec<f64> {
    use naga::{BinaryOperator as B, Expression as E, Literal as L};
    let get = |at| evaluate(expressions, at, inputs);
    match &expressions[at] {
        E::FunctionArgument(i) => inputs[*i as usize].clone(),
        E::Literal(L::F32(v)) => vec![f64::from(*v)],
        E::Literal(L::F64(v) | L::AbstractFloat(v)) => vec![*v],
        E::AccessIndex { base, index } => {
            let base = get(*base);
            // Matrix arguments are column-major, like WGSL and glam.
            let stride = if base.len() == 16 { 4 } else { 1 };
            base[*index as usize * stride..(*index as usize + 1) * stride].to_vec()
        }
        E::Swizzle {
            size,
            vector,
            pattern,
        } => {
            let vector = get(*vector);
            pattern[..*size as usize]
                .iter()
                .map(|c| vector[*c as usize])
                .collect()
        }
        E::Unary {
            op: naga::UnaryOperator::Negate,
            expr,
        } => get(*expr).iter().map(|v| -v).collect(),
        E::Math {
            fun: naga::MathFunction::Abs,
            arg,
            ..
        } => get(*arg).iter().map(|v| v.abs()).collect(),
        E::Math {
            fun: naga::MathFunction::Dot,
            arg,
            arg1: Some(b),
            ..
        } => {
            vec![get(*arg).iter().zip(get(*b)).map(|(a, b)| a * b).sum()]
        }
        E::Splat { size, value } => vec![get(*value)[0]; *size as usize],
        E::Compose { components, .. } => components.iter().flat_map(|at| get(*at)).collect(),
        E::Binary { op, left, right } => {
            let (a, b) = (get(*left), get(*right));
            assert!(a.len() == b.len() || a.len() == 1 || b.len() == 1);
            (0..a.len().max(b.len()))
                .map(|i| {
                    let (a, b) = (a[i % a.len()], b[i % b.len()]);
                    match op {
                        B::Add => a + b,
                        B::Subtract => a - b,
                        B::Multiply => a * b,
                        B::Divide => a / b,
                        _ => panic!("unsupported shader operator {op:?}"),
                    }
                })
                .collect()
        }
        expression => panic!("unsupported shader expression {expression:?}"),
    }
}

#[test]
fn nv12_round_trips_through_the_vendored_gpui_matrix() {
    let module = naga::front::wgsl::parse_str(include_str!("../gpu/nv12.wgsl")).unwrap();
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
    let metal = include_str!("../../../../vendor/gpui-apple/src/shaders.metal");
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
                let encoded = evaluate(&convert.expressions, returned, &[rgb.to_vec()]);
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

#[test]
fn shadow_filter_depth_tracks_the_receiver_plane() {
    use glam::{DMat4, DVec3, DVec4};
    let module = naga::front::wgsl::parse_str(include_str!("../gpu/shadow_map.wgsl")).unwrap();
    let (_, gradient) = module
        .functions
        .iter()
        .find(|(_, f)| f.name.as_deref() == Some("shadow_depth_gradient"))
        .unwrap();
    let returned = gradient
        .body
        .iter()
        .find_map(|s| match s {
            naga::Statement::Return { value } => *value,
            _ => None,
        })
        .unwrap();
    for light in [
        DVec3::new(0.3, 0.8, 0.5),
        DVec3::new(-0.6, 0.4, -0.7),
        DVec3::new(0.001, 0.999, 0.01),
    ] {
        let light = light.normalize();
        for (width, height) in [(0.5, 0.5), (4.0, 64.0), (128.0, 8.0)] {
            let projection = DMat4::orthographic_rh(-width, width, -height, height, -100., 100.)
                * DMat4::look_at_rh(DVec3::ZERO, -light, DVec3::Y);
            // The shader receives an f32 matrix, including its rounding error.
            let projection = projection.as_mat4().as_dmat4();
            let inverse = projection.inverse();
            for normal in [DVec3::X, DVec3::Y, DVec3::Z] {
                let normal = normal * normal.dot(light).signum();
                let slope = evaluate(
                    &gradient.expressions,
                    returned,
                    &[
                        normal.to_array().to_vec(),
                        projection.to_cols_array().to_vec(),
                    ],
                );
                let world = DVec3::new(0.13, -0.27, 0.41);
                let center = projection.project_point3(world);
                for y in -4..=4 {
                    for x in -4..=4 {
                        // Include the outer bilinear sample centers (up to +/-2 texels).
                        let du = f64::from(x) / (2.0 * 4096.0);
                        let dv = f64::from(y) / (2.0 * 4096.0);
                        let clip = DVec4::new(center.x + du * 2., center.y - dv * 2., 0., 1.);
                        let origin = (inverse * clip).truncate();
                        let ray = inverse.transform_vector3(DVec3::Z);
                        // Independent ray/plane intersection gives the expected depth.
                        let hit = origin + ray * ((world - origin).dot(normal) / ray.dot(normal));
                        let expected = projection.project_point3(hit).z;
                        let actual = center.z + slope[0] * du + slope[1] * dv;
                        assert!(
                            (actual - expected).abs() < 1e-7,
                            "{light:?}, {normal:?}, {width}x{height}, tap {x},{y}: {actual} != {expected}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn shadow_filter_bias_covers_bilinear_samples_at_every_texel_phase() {
    let module = naga::front::wgsl::parse_str(include_str!("../gpu/shadow_map.wgsl")).unwrap();
    let (_, filter) = module
        .functions
        .iter()
        .find(|(_, f)| f.name.as_deref() == Some("shadow_filter_depth"))
        .unwrap();
    let returned = filter
        .body
        .iter()
        .find_map(|s| match s {
            naga::Statement::Return { value } => *value,
            _ => None,
        })
        .unwrap();
    let texel = 1.0 / 4096.0;
    // Include diagonal slopes, grazing incidence and both gradient signs. A
    // center-only reference or bias using only the largest slope fails here.
    for gradient in [[0., 0.], [0.3, -0.7], [-4., 4.], [80., 35.], [-1., -100.]] {
        for y in -1..=1 {
            for x in -1..=1 {
                let tap = [f64::from(x) * texel, f64::from(y) * texel];
                let reference = evaluate(
                    &filter.expressions,
                    returned,
                    &[vec![0.5], gradient.to_vec(), tap.to_vec(), vec![texel]],
                )[0];
                for u in [0.0, 0.01, 0.5, 0.99] {
                    for v in [0.0, 0.01, 0.5, 0.99] {
                        for dx in [0., 1.] {
                            for dy in [0., 1.] {
                                let stored = 0.5
                                    + gradient[0] * (tap[0] + (dx - u) * texel)
                                    + gradient[1] * (tap[1] + (dy - v) * texel);
                                assert!(
                                    reference <= stored + 1e-12,
                                    "self-shadow: {gradient:?}, tap {tap:?}, phase {u},{v}"
                                );
                                // Bias stays within a texel footprint; a blocker
                                // two footprints nearer still casts its shadow.
                                let gap =
                                    2. * (gradient[0].abs() + gradient[1].abs()) * texel + 1e-6;
                                let blocker =
                                    0.5 + gradient[0] * tap[0] + gradient[1] * tap[1] - gap;
                                assert!(reference > blocker);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn occupancy_storage_layout_and_epsilon_match_the_host() {
    let module = naga::front::wgsl::parse_str(include_str!("../gpu/viewport.wgsl")).unwrap();
    let (_, layout) = module
        .types
        .iter()
        .find(|(_, t)| t.name.as_deref() == Some("Occupancy"))
        .unwrap();
    let naga::TypeInner::Struct { members, .. } = &layout.inner else {
        panic!("occupancy must be a storage struct")
    };
    assert_eq!(
        members.iter().map(|m| m.offset).collect::<Vec<_>>(),
        vec![0, 16, 32, 48, 64, 80]
    );
    let (_, epsilon) = module
        .constants
        .iter()
        .find(|(_, c)| c.name.as_deref() == Some("SHADOW_EPSILON"))
        .unwrap();
    assert_eq!(
        evaluate(&module.global_expressions, epsilon.init, &[]),
        vec![f64::from(crate::occupancy::SHADOW_EPSILON)]
    );
}
