// Appended inside the unchanged production module by build.rs. This is a private
// spike hook, never an addition to the renderer's public API.
#[cfg(feature = "salsa")]
pub(crate) fn from_padded(
    at: Pos,
    slots: &[u16; 8000],
    palette: &[Option<[f32; 3]>],
) -> ChunkMesh {
    mesh_chunk(
        at,
        &LocalCells {
            min: at.map(|v| v * 16 - 2),
            slots: *slots,
        },
        palette,
    )
}
