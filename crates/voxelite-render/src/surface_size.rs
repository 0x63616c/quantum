use crate::{RenderError, RenderResult};

/// Physical display pixels -> even, supersampled NV12 dimensions. All targets
/// use these dimensions, including the scene, resolve and full-resolution luma.
pub(crate) fn surface_size(display: [u32; 2], limit: u32) -> RenderResult<[u32; 2]> {
    let invalid = || RenderError::Limit("GPU viewport exceeds target budget".into());
    let width = display[0].checked_mul(2).ok_or_else(invalid)?;
    let height = display[1].checked_mul(2).ok_or_else(invalid)?;
    // ~85 bytes per supersampled pixel: 4x MSAA color/geometry/depth,
    // resolves and three NV12 buffers. Cap at ~2 GiB plus shadow/mesh resources.
    if width == 0
        || height == 0
        || width > limit
        || height > limit
        || u64::from(width) * u64::from(height) > 24_000_000
    {
        return Err(invalid());
    }
    Ok([width, height])
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
    fn rejects_zero_overflow_device_and_allocation_limits() {
        for size in [[0, 10], [10, 0], [u32::MAX, 1], [8193, 1], [5000, 3000]] {
            assert!(surface_size(size, 16384).is_err(), "{size:?}");
        }
    }
}
