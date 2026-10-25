//! Convert validated wire arguments into the shared, atomic command language.
use crate::error::{Args, Failure, Result};
use serde_json::Value;
use voxelite_core::*;

pub(crate) fn layer(s: &Snapshot, value: &str) -> Result<Uuid> {
    if let Ok(id) = Uuid::parse_str(value) {
        s.layer(id)?;
        return Ok(id);
    }
    let matches: Vec<_> = s.layers.iter().filter(|l| l.name == value).collect();
    match matches.as_slice() {
        [l] => Ok(l.id),
        [] => Err(Failure::new(
            "layer_not_found",
            format!("no layer named {value}"),
        )),
        _ => Err(Failure::new(
            "ambiguous_name",
            format!(
                "multiple layers named {value}; use one of {:?}",
                matches.iter().map(|l| l.id).collect::<Vec<_>>()
            ),
        )),
    }
}
pub(crate) fn slot(s: &Snapshot, value: &Value) -> Result<u16> {
    if let Some(name) = value.as_str() {
        s.palette.resolve(name).map_err(Into::into)
    } else {
        serde_json::from_value(value.clone())
            .map_err(|_| Failure::invalid("slot must be 0..256 or a unique swatch name"))
    }
}
fn mirror(a: &Args<'_>) -> Result<Mirror> {
    let Some(value) = a.0.get("mirror") else {
        return Ok(Mirror::default());
    };
    let m = Args(value);
    let axes: Vec<String> = m.get("axes")?;
    let planes: Pos = m.get("plane2")?;
    Ok(Mirror {
        plane2: std::array::from_fn(|i| {
            axes.iter()
                .any(|a| a == ["x", "y", "z"][i])
                .then_some(planes[i])
        }),
    })
}
pub(crate) fn voxel_command(s: &Snapshot, kind: &str, a: &Args<'_>) -> Result<Command> {
    let layer = layer(s, &a.get::<String>("layer")?)?;
    let mirror = mirror(a)?;
    Ok(match kind {
        "set" => {
            let values: Vec<Value> = a.get("voxels")?;
            let voxels = values
                .iter()
                .map(|v| {
                    Ok(Voxel {
                        at: Args(v).get("at")?,
                        slot: slot(s, &v["slot"])?,
                    })
                })
                .collect::<Result<_>>()?;
            Command::SetVoxels {
                layer,
                voxels,
                mode: a.default("mode", Mode::Place)?,
                mirror,
            }
        }
        "erase_region" => Command::EraseRegion {
            layer,
            region: a.get("region")?,
            mirror,
        },
        "fill" => Command::FloodFill {
            layer,
            seed: a.get("seed")?,
            slot: slot(s, &a.0["slot"])?,
            region: a.optional("region")?,
            cap: a.default("cap", MAX_WORK)?,
            mirror,
        },
        "box" => Command::Primitive {
            layer,
            shape: Shape::Box {
                region: a.get("region")?,
                hollow: a.default("hollow", false)?,
            },
            mode: a.default("mode", Mode::Place)?,
            slot: a.0.get("slot").map(|v| slot(s, v)).transpose()?,
            mirror,
        },
        _ => return Err(Failure::invalid("unknown voxel operation")),
    })
}
pub(crate) fn preset(name: &str) -> Result<Palette> {
    Ok(match name {
        "pico8" => Preset::Pico8,
        "endesga32" => Preset::Endesga32,
        "sweetie16" => Preset::Sweetie16,
        "grayscale" => Preset::Grayscale,
        "neutral" => Preset::Neutral,
        "warm" => Preset::Warm,
        "vivid" => Preset::Vivid,
        _ => return Err(Failure::invalid("unknown preset")),
    }
    .palette())
}
pub(crate) fn append_palette(s: &Snapshot, p: Palette) -> Result<(Command, Value)> {
    let free: Vec<_> = (1..=256).filter(|i| s.palette.get(*i).is_none()).collect();
    if free.len() < p.slots.len() {
        return Err(Failure::new(
            "limit_exceeded",
            "not enough free palette slots; imports append without recoloring existing geometry",
        ));
    }
    let mut commands = Vec::new();
    let mut mapping = Vec::new();
    for (source, target) in p.order.iter().zip(free) {
        commands.push(Command::Palette(PaletteEdit::Add {
            slot: Some(target),
            color: p.get(*source).expect("validated palette").clone(),
        }));
        mapping.push(serde_json::json!({"source":source,"slot":target}));
    }
    Ok((
        Command::Batch(commands),
        serde_json::json!({"slot_mapping":mapping}),
    ))
}
