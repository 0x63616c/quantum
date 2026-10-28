//! Throwaway payload codecs, deliberately separate from the canonical format.
use crate::Result;
use rkyv::{Archive, Serialize, rancor::Error};
use serde::{Deserialize, de::value::SeqDeserializer};
use voxelite_core::{MAX_VOXELS, Volume};
use voxelite_format::{MAX_FILE_BYTES, MAX_METADATA_BYTES, Project};

const MAGIC: [u8; 8] = *b"PAYLOAD1";

#[derive(Archive, Serialize)]
pub(crate) struct Payload {
    magic: [u8; 8],
    schema: u32,
    metadata: Vec<u8>,
    layers: Vec<Vec<[i32; 4]>>,
}

/// Reuse the exact production tuple validation and chunk construction, without
/// allocating a Vec for each tuple or deserializing an intermediate native Vec.
pub(crate) fn volume(cells: impl Iterator<Item = [i32; 4]>) -> Result<Volume> {
    Ok(Volume::deserialize(SeqDeserializer::<
        _,
        serde::de::value::Error,
    >::new(cells.map(|c| {
        SeqDeserializer::<_, serde::de::value::Error>::new(c.into_iter())
    })))?)
}

fn supported(project: &Project) -> Result<()> {
    if !project.document.versions.is_empty() || project.build_history().is_some() {
        return Err("prototype does not support named versions or build recipes".into());
    }
    Ok(())
}

pub(crate) fn stage(project: &Project) -> Result<Payload> {
    supported(project)?;
    project.document.validate()?;
    let layers = project
        .document
        .content
        .layers
        .iter()
        .map(|layer| {
            layer
                .voxels
                .sorted()
                .into_iter()
                .map(|([x, y, z], s)| [x, y, z, i32::from(s)])
                .collect()
        })
        .collect();
    let mut metadata = project.clone();
    for layer in &mut metadata.document.content.layers {
        layer.voxels = Volume::default();
    }
    let metadata = metadata.to_bytes()?;
    if metadata.len() > MAX_METADATA_BYTES {
        return Err("prototype metadata limit".into());
    }
    Ok(Payload {
        magic: MAGIC,
        schema: 1,
        metadata,
        layers,
    })
}

pub(crate) fn archive(payload: &Payload) -> Result<Vec<u8>> {
    let bytes = rkyv::to_bytes::<Error>(payload)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err("prototype file limit".into());
    }
    Ok(bytes.into_vec())
}

pub(crate) fn encode(project: &Project) -> Result<Vec<u8>> {
    archive(&stage(project)?)
}

pub(crate) fn access(bytes: &[u8]) -> Result<&ArchivedPayload> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err("prototype file limit".into());
    }
    // Safe API always invokes bytecheck, including relative pointer bounds.
    let payload = rkyv::access::<ArchivedPayload, Error>(bytes)?;
    if payload.magic != MAGIC || payload.schema.to_native() != 1 {
        return Err("unknown prototype schema".into());
    }
    if payload.metadata.len() > MAX_METADATA_BYTES
        || payload.layers.len() > 64
        || payload.layers.iter().map(|v| v.len()).sum::<usize>() > MAX_VOXELS
    {
        return Err("prototype payload budget".into());
    }
    Ok(payload)
}

fn metadata(bytes: &[u8], layer_count: usize) -> Result<Project> {
    let project = voxelite_format::from_bytes(bytes)?;
    supported(&project)?;
    if project.document.content.layers.len() != layer_count
        || project
            .document
            .content
            .layers
            .iter()
            .any(|l| !l.voxels.is_empty())
    {
        return Err("prototype layer mismatch or nonempty metadata geometry".into());
    }
    Ok(project)
}

pub(crate) fn decode(bytes: &[u8]) -> Result<Project> {
    let payload = access(bytes)?;
    let mut project = metadata(&payload.metadata, payload.layers.len())?;
    for (layer, cells) in project
        .document
        .content
        .layers
        .iter_mut()
        .zip(payload.layers.iter())
    {
        layer.voxels = volume(cells.iter().map(|c| c.map(|v| v.to_native())))?;
    }
    // Bytecheck validates representation; core still validates bounds, palette,
    // duplicates, layer budgets and every other document invariant.
    project.document.validate()?;
    Ok(project)
}

// Fixed-width binary control: same 16-byte tuples, metadata and core validation.
// This separates skipping decimal JSON parsing from rkyv-specific benefits.
pub(crate) fn encode_packed(project: &Project) -> Result<Vec<u8>> {
    let payload = stage(project)?;
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&(payload.metadata.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&payload.metadata);
    bytes.extend_from_slice(&(payload.layers.len() as u32).to_le_bytes());
    for cells in payload.layers {
        bytes.extend_from_slice(&(cells.len() as u32).to_le_bytes());
        for cell in cells {
            for value in cell {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    if bytes.len() > MAX_FILE_BYTES {
        return Err("prototype file limit".into());
    }
    Ok(bytes)
}

fn take<'a>(bytes: &mut &'a [u8], count: usize) -> Result<&'a [u8]> {
    let (head, tail) = bytes.split_at_checked(count).ok_or("truncated payload")?;
    *bytes = tail;
    Ok(head)
}

fn number(bytes: &mut &[u8]) -> Result<usize> {
    Ok(u32::from_le_bytes(take(bytes, 4)?.try_into()?) as usize)
}

pub(crate) fn decode_packed(mut bytes: &[u8]) -> Result<Project> {
    if bytes.len() > MAX_FILE_BYTES || take(&mut bytes, 8)? != MAGIC {
        return Err("prototype header or file limit".into());
    }
    let count = number(&mut bytes)?;
    if count > MAX_METADATA_BYTES {
        return Err("prototype metadata limit".into());
    }
    let meta = take(&mut bytes, count)?;
    let layers = number(&mut bytes)?;
    let mut project = metadata(meta, layers)?;
    let mut total = 0;
    for layer in &mut project.document.content.layers {
        let count = number(&mut bytes)?;
        total += count;
        if total > MAX_VOXELS {
            return Err("prototype voxel limit".into());
        }
        let cells = take(&mut bytes, count * 16)?;
        layer.voxels = volume(cells.as_chunks::<16>().0.iter().map(|c| {
            std::array::from_fn(|i| {
                i32::from_le_bytes(c[i * 4..i * 4 + 4].try_into().expect("four-byte slice"))
            })
        }))?;
    }
    if !bytes.is_empty() {
        return Err("trailing payload".into());
    }
    project.document.validate()?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> Project {
        voxelite_format::from_bytes(include_bytes!(
            "../../../../spec/voxl/1.0/fixtures/populated.input.voxl"
        ))
        .unwrap()
    }

    #[test]
    fn both_codecs_preserve_canonical_bytes() {
        let project = project();
        let expected = project.to_bytes().unwrap();
        for (encode, decode) in [
            (
                encode as fn(&Project) -> Result<Vec<u8>>,
                decode as fn(&[u8]) -> Result<Project>,
            ),
            (encode_packed, decode_packed),
        ] {
            let bytes = encode(&project).unwrap();
            assert_eq!(decode(&bytes).unwrap().to_bytes().unwrap(), expected);
            for cut in 0..bytes.len() {
                assert!(decode(&bytes[..cut]).is_err(), "accepted truncation {cut}");
            }
        }
    }

    #[test]
    fn structural_and_semantic_corruption_is_rejected() {
        let project = project();
        let mut payload = stage(&project).unwrap();
        payload.schema = 2;
        assert!(decode(&archive(&payload).unwrap()).is_err());
        payload.schema = 1;
        for cells in [
            vec![[0, 0, 0, 0]],
            vec![[0, 0, 0, 2]],
            vec![[32, 0, 0, 1]],
            vec![[0, 0, 0, 1]; 2],
        ] {
            payload.layers[0] = cells;
            let bytes = archive(&payload).unwrap();
            assert!(access(&bytes).is_ok(), "representation is valid");
            assert!(decode(&bytes).is_err(), "domain validation must still run");
        }
        let mut bytes = encode(&project).unwrap();
        bytes.fill(255);
        assert!(access(&bytes).is_err());
    }

    #[test]
    fn metadata_and_layer_contracts_are_preserved() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&project().to_bytes().unwrap()).unwrap();
        value["version"]["minor"] = 8.into();
        value["future"] = serde_json::json!({"name": "wood 🌳", "flags": [true, null]});
        value["view"] = serde_json::json!({"eye": [1.5, 2.0, -3.0]});
        value["layers"][0]["future"] = serde_json::json!([2, 1]);
        let project = voxelite_format::from_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
        let bytes = encode(&project).unwrap();
        let mut misaligned = vec![0];
        misaligned.extend_from_slice(&bytes);
        assert_eq!(
            decode(&misaligned[1..]).unwrap().to_bytes().unwrap(),
            project.to_bytes().unwrap()
        );
        let mut payload = stage(&project).unwrap();
        payload.layers.clear();
        assert!(decode(&archive(&payload).unwrap()).is_err());
        let mut payload = stage(&project).unwrap();
        payload.metadata = project.to_bytes().unwrap();
        assert!(decode(&archive(&payload).unwrap()).is_err());
    }

    #[test]
    fn unsupported_history_is_explicitly_rejected() {
        let project = voxelite_format::from_bytes(include_bytes!(
            "../../../../examples/autumn-workshop.voxl"
        ))
        .unwrap();
        assert!(encode(&project).is_err());
        assert!(encode_packed(&project).is_err());
    }
}
