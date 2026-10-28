//! GPU resource limit contracts.
use super::*;
#[test]
fn occupancy_respects_both_buffer_and_binding_limits() {
    let maximum = Occupancy::new(quantum_core::Region::new([0; 3], [256; 3]).unwrap()).unwrap();
    let bytes = std::mem::size_of_val(maximum.words()) as u64;
    let mut limits = wgpu::Limits::default();
    assert!(occupancy_fits(bytes, &limits));
    limits.max_buffer_size = bytes;
    limits.max_storage_buffer_binding_size = bytes;
    assert!(occupancy_fits(bytes, &limits));
    limits.max_buffer_size = bytes - 1;
    assert!(!occupancy_fits(bytes, &limits));
    limits.max_buffer_size = bytes;
    limits.max_storage_buffer_binding_size = bytes - 1;
    assert!(!occupancy_fits(bytes, &limits));
}
