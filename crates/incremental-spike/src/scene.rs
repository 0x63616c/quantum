//! Deterministic terraced terrain and real engine transactions (outside timing).
use crate::Result;
use serde::Deserialize;
use voxelite_core::*;

pub(crate) fn height(x: i32, z: i32, size: i32) -> i32 {
    let hill = |v: i32| (v.rem_euclid(64) - 32).abs();
    (size / 2 + hill(x) + hill(z) / 2).min(size - 1)
}

pub(crate) fn terrain(size: i32) -> Result<Snapshot> {
    let cells = (0..size).flat_map(|z| {
        (0..size).flat_map(move |x| {
            let h = height(x, z, size);
            (0..h).map(move |y| {
                let slot = if y == h - 1 {
                    1
                } else if y > h - 5 {
                    2
                } else {
                    3
                };
                vec![x, y, z, slot]
            })
        })
    });
    let voxels = Volume::deserialize(serde::de::value::SeqDeserializer::<
        _,
        serde::de::value::Error,
    >::new(cells))?;
    let mut document = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    document.content.bounds = Region::new([0; 3], [size; 3])?;
    document.content.palette = Palette::from_colors(vec![
        Swatch::rgb(70, 140, 60),
        Swatch::rgb(120, 86, 54),
        Swatch::rgb(112, 112, 118),
    ])?;
    document.content.layers[0].voxels = voxels;
    document.validate()?;
    Ok(document.content)
}

pub(crate) fn edit(snapshot: &Snapshot, command: Command) -> Result<Snapshot> {
    let mut document = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    document.content = snapshot.clone();
    let mut engine = Engine::new(document)?;
    engine.execute(Transaction {
        expected_revision: engine.revision(),
        id: Uuid::from_u128(3),
        actor: Actor::new(Origin::Human),
        label: "Benchmark edit".into(),
        command,
    })?;
    Ok(engine.document().content.clone())
}

pub(crate) fn writes(snapshot: &Snapshot, voxels: Vec<Voxel>) -> Result<Snapshot> {
    edit(
        snapshot,
        Command::SetVoxels {
            layer: snapshot.layers[0].id,
            voxels,
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
    )
}

pub(crate) fn variant(snapshot: &Snapshot, workload: &str) -> Result<Snapshot> {
    let size = snapshot.bounds.max[0];
    match workload {
        "single" => writes(
            snapshot,
            vec![Voxel {
                at: [8, height(8, 8, size) - 1, 8],
                slot: 2,
            }],
        ),
        "boundary" => writes(
            snapshot,
            vec![Voxel {
                at: [16, height(16, 16, size) - 1, 16],
                slot: 0,
            }],
        ),
        "bulk" => writes(
            snapshot,
            (0..size / 16)
                .flat_map(|z| {
                    (0..size / 16).flat_map(move |x| {
                        // An 8x8 brush on every terrain tile: 16,384 actual writes at 256.
                        (4..12).flat_map(move |dz| {
                            (4..12).map(move |dx| {
                                let (x, z) = (x * 16 + dx, z * 16 + dz);
                                Voxel {
                                    at: [x, height(x, z, size) - 1, z],
                                    slot: 2,
                                }
                            })
                        })
                    })
                })
                .collect(),
        ),
        "palette" => edit(
            snapshot,
            Command::Palette(PaletteEdit::Set {
                slot: 1,
                color: Swatch::rgb(230, 170, 60),
            }),
        ),
        "full" | "noop" | "settings" => Ok(snapshot.clone()),
        _ => Err(format!("unknown workload {workload}").into()),
    }
}
