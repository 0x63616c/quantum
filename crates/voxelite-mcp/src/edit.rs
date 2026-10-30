//! Typed voxel operations shared by individual tools and atomic batches.
use crate::{
    arguments::{arguments, schema},
    catalog::*,
    error::{Failure, Result},
};
use serde_json::{Value, json};
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
// Retain the original slot decoder: fractional JSON numbers get its specific
// diagnostic rather than serde's untagged-enum error.
#[derive(serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct Slot(Value);
impl Slot {
    fn resolve(&self, s: &Snapshot) -> Result<u16> {
        slot(s, &self.0)
    }
}
arguments!(MirrorInput {
    req axes: Vec<String> => { let mut s = array(choice(&["x", "y", "z"]), 0, 3); s["uniqueItems"] = json!(true); s },
    req plane2: Pos => array(integer(-65536,65536),3,3),
});
impl MirrorInput {
    fn resolve(self) -> Result<Mirror> {
        let axes = self.axes.get()?;
        let planes = self.plane2.get()?;
        Ok(Mirror {
            plane2: std::array::from_fn(|i| {
                axes.iter()
                    .any(|a| a == ["x", "y", "z"][i])
                    .then_some(planes[i])
            }),
        })
    }
}
fn resolve_mirror(field: &crate::arguments::Field<MirrorInput>) -> Result<Mirror> {
    field
        .optional()?
        .map(MirrorInput::resolve)
        .transpose()
        .map(Option::unwrap_or_default)
}
arguments!(Cell {
    req at: Pos => pos(),
    req slot: Slot => slot_schema(true),
});
arguments!(SetInput {
    req layer: String => layer_schema_input(),
    req voxels: Vec<Cell> => array(schema::<Cell>(),1,65536),
    opt mode: Mode => choice(&["place","paint"]),
    opt mirror: MirrorInput => schema::<MirrorInput>(),
});
arguments!(EraseInput {
    req layer: String => layer_schema_input(),
    req region: Region => region(),
    opt mirror: MirrorInput => schema::<MirrorInput>(),
});
arguments!(FillInput {
    req layer: String => layer_schema_input(),
    req seed: Pos => pos(),
    req slot: Slot => slot_schema(false),
    opt region: Region => region(),
    opt cap: usize => integer(1,1048576),
    opt mirror: MirrorInput => schema::<MirrorInput>(),
});
arguments!(BoxInput {
    req layer: String => layer_schema_input(),
    req region: Region => region(),
    opt slot: Slot => slot_schema(false),
    opt hollow: bool => boolean(),
    opt mode: Mode => mode(),
    opt mirror: MirrorInput => schema::<MirrorInput>(),
} where box_constraints);
impl SetInput {
    pub fn command(&self, s: &Snapshot) -> Result<Command> {
        let layer = layer(s, &self.layer.get()?)?;
        let mirror = resolve_mirror(&self.mirror)?;
        let voxels = self
            .voxels
            .get()?
            .iter()
            .map(|v| {
                Ok(Voxel {
                    at: v.at.get()?,
                    slot: v.slot.get()?.resolve(s)?,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Command::SetVoxels {
            layer,
            voxels,
            mode: self.mode.default(Mode::Place)?,
            mirror,
        })
    }
}
impl EraseInput {
    pub fn command(&self, s: &Snapshot) -> Result<Command> {
        let layer = layer(s, &self.layer.get()?)?;
        let mirror = resolve_mirror(&self.mirror)?;
        Ok(Command::EraseRegion {
            layer,
            region: self.region.get()?,
            mirror,
        })
    }
}
impl FillInput {
    pub fn command(&self, s: &Snapshot) -> Result<Command> {
        let layer = layer(s, &self.layer.get()?)?;
        let mirror = resolve_mirror(&self.mirror)?;
        Ok(Command::FloodFill {
            layer,
            seed: self.seed.get()?,
            slot: self.slot.get()?.resolve(s)?,
            region: self.region.optional()?,
            cap: self.cap.default(MAX_WORK)?,
            mirror,
        })
    }
}
impl BoxInput {
    pub fn command(&self, s: &Snapshot) -> Result<Command> {
        let layer = layer(s, &self.layer.get()?)?;
        let mirror = resolve_mirror(&self.mirror)?;
        Ok(Command::Primitive {
            layer,
            shape: Shape::Box {
                region: self.region.get()?,
                hollow: self.hollow.default(false)?,
            },
            mode: self.mode.default(Mode::Place)?,
            slot: self.slot.optional()?.map(|v| v.resolve(s)).transpose()?,
            mirror,
        })
    }
}

// Each batch variant reuses the same input type, schema and command builder as
// its standalone tool. The tag is appended to retain the published key order.
macro_rules! operations {
    ($($variant:ident => $kind:literal : $ty:ty),* $(,)?) => {
        #[derive(serde::Deserialize)]
        #[serde(tag = "kind")]
        pub(crate) enum Operation { $(#[serde(rename = $kind)] $variant($ty)),* }
        impl schemars::JsonSchema for Operation {
            fn schema_name() -> std::borrow::Cow<'static,str> { "Operation".into() }
            fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
                let variants = vec![$({
                    let mut s = schema::<$ty>();
                    s["properties"]["kind"] = json!({"const":$kind});
                    s["required"].as_array_mut().unwrap().push(json!("kind"));
                    s
                }),*];
                json!({"oneOf":variants}).try_into().unwrap()
            }
        }
        impl Operation {
            pub fn command(&self, s: &Snapshot) -> Result<Command> { match self { $(Self::$variant(a) => a.command(s)),* } }
        }
    };
}
operations!(Set => "set": SetInput, Erase => "erase_region": EraseInput, Fill => "fill": FillInput, Box => "box": BoxInput);
impl Operation {
    pub fn explicit_voxels(&self) -> Result<usize> {
        match self {
            Self::Set(a) => Ok(a.voxels.get()?.len()),
            _ => Ok(0),
        }
    }
}
