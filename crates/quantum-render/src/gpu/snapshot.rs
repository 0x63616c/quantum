//! Shared snapshot synchronization for native and headless frames.
use super::*;

/// The worker and headless path must invalidate precisely the same caches.
pub(super) fn sync_snapshot(
    occupancy: &mut OccupancyCache,
    mesh: &mut ChunkMeshes,
    snapshot: Option<Snapshot>,
    mut stats: Option<&mut FrameStats>,
) -> RenderResult<Vec<Pos>> {
    let Some(snapshot) = snapshot else {
        return Ok(Vec::new());
    };
    let started = stats.as_ref().map(|_| Instant::now());
    occupancy.sync(&snapshot)?;
    if let Some(stats) = stats.as_mut() {
        stats.occupancy_sync_ms = elapsed_ms(started);
    }
    let started = stats.as_ref().map(|_| Instant::now());
    let updates = mesh.sync_validated(&snapshot)?;
    if let Some(stats) = stats {
        stats.mesh_sync_ms = elapsed_ms(started);
    }
    Ok(updates)
}

pub(super) fn error(e: impl std::fmt::Display) -> RenderError {
    RenderError::Invalid(format!("GPU viewport: {e}"))
}
