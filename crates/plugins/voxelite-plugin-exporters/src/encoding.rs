//! Deterministic, CPU-only GLB 2.0 and lossless RGBA PNG export.
use serde_json::json;
use std::io::Write;
use voxelite_plugin_api::{ExportError, ExportResult, Snapshot, greedy_mesh};

/// Maximum encoded output, matching portable artifact admission.
pub const MAX_EXPORT_BYTES: usize = 128 * 1024 * 1024;

/// Encode bounded RGBA8 pixels as deterministic lossless sRGB PNG.
/// Dimensions are independent of row layout; exactly width*height*4 bytes are required.
pub fn png_rgba(width: u32, height: u32, rgba: &[u8]) -> ExportResult<Vec<u8>> {
    let pixels = u64::from(width) * u64::from(height);
    if width == 0
        || height == 0
        || width > 2048
        || height > 2048
        || pixels > 4_194_304
        || pixels * 4 != rgba.len() as u64
    {
        return Err(ExportError::Invalid(
            "RGBA dimensions/length exceed 2048² or mismatch".into(),
        ));
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba)?;
        writer.finish()?;
    }
    Ok(bytes)
}
/// Encode visible composite geometry as a glTF 2.0 binary container.
/// Vertex colors are linear RGB, opaque, with outward normals and CCW winding.
pub fn glb(snapshot: &Snapshot) -> ExportResult<Vec<u8>> {
    let mut bytes = Vec::new();
    write_glb(snapshot, &mut bytes)?;
    Ok(bytes)
}
/// Stream expanded GLB attributes and indices after bounded mesh preflight.
/// No external buffers, textures, URIs or extension execution are emitted.
#[expect(
    clippy::expect_used,
    reason = "internal invariant: mesh validates slots"
)]
pub fn write_glb(snapshot: &Snapshot, mut writer: impl Write) -> ExportResult<()> {
    let mesh = greedy_mesh(snapshot)?;
    if mesh.quads.is_empty() {
        let root = json!({"asset":{"version":"2.0","generator":"Voxelite"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{}]});
        let mut json = serde_json::to_vec(&root)?;
        pad(&mut json, b' ');
        let total = 12 + 8 + json.len();
        header(&mut writer, total)?;
        chunk_header(&mut writer, json.len(), 0x4e4f534a)?;
        writer.write_all(&json)?;
        return Ok(());
    }
    let vertices = mesh.quads.len() * 4;
    let indices = mesh.quads.len() * 6;
    // POSITION f32x3 + NORMAL f32x3 + COLOR_0 f32x4, followed by u32 indices.
    let vertex_bytes = vertices
        .checked_mul(40)
        .ok_or_else(|| ExportError::Invalid("vertex size overflow".into()))?;
    let bin_bytes = vertex_bytes + indices * 4;
    if bin_bytes + 65536 > MAX_EXPORT_BYTES {
        return Err(ExportError::Invalid(format!(
            "expanded GLB needs {} bytes; cap {MAX_EXPORT_BYTES}",
            bin_bytes + 65536
        )));
    }
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for q in &mesh.quads {
        for p in q.vertices() {
            for a in 0..3 {
                min[a] = min[a].min(p[a]);
                max[a] = max[a].max(p[a]);
            }
        }
    }
    let root = json!({
        "asset":{"version":"2.0","generator":"Voxelite"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0,"name":snapshot.name}],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0,"NORMAL":1,"COLOR_0":2},"indices":3,"material":0,"mode":4}]}],
        "materials":[{"pbrMetallicRoughness":{"baseColorFactor":[1,1,1,1],"metallicFactor":0,"roughnessFactor":1},"alphaMode":"OPAQUE","doubleSided":false}],
        "buffers":[{"byteLength":bin_bytes}],
        "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":vertex_bytes,"byteStride":40,"target":34962},{"buffer":0,"byteOffset":vertex_bytes,"byteLength":indices*4,"target":34963}],
        "accessors":[
            {"bufferView":0,"byteOffset":0,"componentType":5126,"count":vertices,"type":"VEC3","min":min,"max":max},
            {"bufferView":0,"byteOffset":12,"componentType":5126,"count":vertices,"type":"VEC3"},
            {"bufferView":0,"byteOffset":24,"componentType":5126,"count":vertices,"type":"VEC4"},
            {"bufferView":1,"byteOffset":0,"componentType":5125,"count":indices,"type":"SCALAR"}
        ]
    });
    let mut json = serde_json::to_vec(&root)?;
    pad(&mut json, b' ');
    header(&mut writer, 12 + 8 + json.len() + 8 + bin_bytes)?;
    chunk_header(&mut writer, json.len(), 0x4e4f534a)?;
    writer.write_all(&json)?;
    chunk_header(&mut writer, bin_bytes, 0x004e4942)?;
    for q in &mesh.quads {
        let rgba = snapshot
            .palette
            .get(q.slot)
            .expect("mesh validates slots")
            .rgba;
        let color = [
            srgb_to_linear(rgba[0]),
            srgb_to_linear(rgba[1]),
            srgb_to_linear(rgba[2]),
            1.0,
        ];
        for p in q.vertices() {
            for f in p.into_iter().chain(q.normal()).chain(color) {
                writer.write_all(&f.to_le_bytes())?;
            }
        }
    }
    for i in 0..mesh.quads.len() {
        let base = (i * 4) as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            writer.write_all(&(base + index).to_le_bytes())?;
        }
    }
    Ok(())
}
fn header(w: &mut impl Write, len: usize) -> std::io::Result<()> {
    w.write_all(b"glTF")?;
    w.write_all(&2u32.to_le_bytes())?;
    w.write_all(&(len as u32).to_le_bytes())
}
fn chunk_header(w: &mut impl Write, len: usize, kind: u32) -> std::io::Result<()> {
    w.write_all(&(len as u32).to_le_bytes())?;
    w.write_all(&kind.to_le_bytes())
}
fn pad(bytes: &mut Vec<u8>, value: u8) {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(value);
    }
}
fn srgb_to_linear(value: u8) -> f32 {
    let c = f32::from(value) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
