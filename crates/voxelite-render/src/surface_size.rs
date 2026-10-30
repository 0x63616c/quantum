use crate::{RenderError, RenderResult};

/// Physical display pixels -> even, supersampled NV12 dimensions. All targets
/// use these dimensions, including the scene, resolve and full-resolution luma.
/// Prefer 2x, then 1.5x, then 1x, rounding up to even dimensions for NV12.
pub(crate) fn surface_size(display: [u32; 2], limit: u32) -> RenderResult<[u32; 2]> {
    // ~85 bytes per supersampled pixel: 4x MSAA color/geometry/depth,
    // resolves and three NV12 buffers. Cap at ~2 GiB plus shadow/mesh resources.
    for halves in [4, 3, 2] {
        let [width, height] = display.map(|n| (u64::from(n) * halves).div_ceil(4) * 2);
        if width > 0
            && height > 0
            && width <= u64::from(limit)
            && height <= u64::from(limit)
            && width * height <= 24_000_000
        {
            return Ok([width as u32, height as u32]);
        }
    }
    Err(RenderError::SurfaceSize(
        "GPU viewport exceeds target budget".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supersampling_preserves_odd_dimensions_and_retina_pixels() {
        assert_eq!(surface_size([801, 601], 16384).unwrap(), [1602, 1202]);
        assert_eq!(surface_size([2400, 1600], 16384).unwrap(), [4800, 3200]);
    }

    #[test]
    fn steps_down_at_pixel_budget_boundaries() {
        for (display, expected) in [
            ([3000, 2000], [6000, 4000]), // Exactly 24M at 2x.
            ([3001, 2000], [4502, 3000]),
            ([4000, 2666], [6000, 4000]), // Exactly 24M after even rounding at 1.5x.
            ([4000, 2667], [4000, 2668]),
            ([4300, 2700], [4300, 2700]), // Full-screen viewport on a 5K display.
            ([5000, 3000], [5000, 3000]),
            ([6000, 4000], [6000, 4000]), // Exactly 24M at 1x.
        ] {
            assert_eq!(surface_size(display, 16384).unwrap(), expected);
        }
    }

    #[test]
    fn steps_down_at_device_limits_and_rounds_odd_dimensions_up() {
        for (limit, expected) in [
            (1602, [1602, 1202]),
            (1601, [1202, 902]),
            (1202, [1202, 902]),
            (1201, [802, 602]),
            (802, [802, 602]),
        ] {
            assert_eq!(surface_size([801, 601], limit).unwrap(), expected);
            assert_eq!(
                surface_size([601, 801], limit).unwrap(),
                [expected[1], expected[0]]
            );
        }
        assert_eq!(surface_size([8193, 1], 16384).unwrap(), [12290, 2]);
        assert_eq!(surface_size([8193, 1], 8194).unwrap(), [8194, 2]);
    }

    #[test]
    fn rejects_only_when_even_one_x_exceeds_limits_or_dimensions_are_zero() {
        for (display, limit) in [
            ([0, 10], 16384),
            ([10, 0], 16384),
            ([1, 1], 0),
            ([1, 1], 1),
            ([u32::MAX, 1], u32::MAX),
            ([u32::MAX, u32::MAX], u32::MAX),
            ([u32::MAX - 1, u32::MAX - 1], u32::MAX),
            ([801, 601], 801),
            ([16385, 1], 16384),
            ([6001, 4000], 16384),
        ] {
            assert!(
                matches!(
                    surface_size(display, limit),
                    Err(RenderError::SurfaceSize(_))
                ),
                "{display:?}, {limit}"
            );
        }
    }

    proptest::proptest! {
        #[test]
        fn preserves_every_target_that_fits_at_two_x(
            width in 1u32..=8192, height in 1u32..=8192,
        ) {
            let height = height.min(6_000_000 / width);
            let limit = width.max(height) * 2;
            proptest::prop_assert_eq!(surface_size([width, height], limit).unwrap(), [width * 2, height * 2]);
        }
    }
}
