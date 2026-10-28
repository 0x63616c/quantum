//! Discovery and wire-schema vocabulary shared by the typed tool definitions.
use crate::{
    arguments::{Input, schema},
    tools::Handler,
};
use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{Map, Value, json};
use std::sync::{Arc, LazyLock};

pub(crate) struct Spec {
    pub tool: Tool,
    pub validator: jsonschema::Validator,
    pub mutation: bool,
    pub handler: Handler,
}
pub(crate) static CATALOG: LazyLock<Vec<Spec>> = LazyLock::new(crate::tools::specs);

impl Spec {
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: built-in input schema"
    )]
    pub fn new<T: Input>(
        name: &'static str,
        description: &'static str,
        scope: Scope,
        output: Output,
        handler: Handler,
    ) -> Self {
        let mutation = matches!(scope, Scope::Write | Scope::Create);
        let mut schema = schema::<T>();
        let mut common = match scope {
            Scope::Read => crate::arguments::ReadCommon::fields(),
            Scope::Write => crate::arguments::WriteCommon::fields(),
            Scope::Create => crate::arguments::NewCommon::fields(),
            Scope::Session => vec![],
        };
        let fields = T::fields()
            .into_iter()
            .map(|(name, _, required)| (name, schema["properties"][name].clone(), required));
        common.extend(fields);
        let full = object(common);
        schema["properties"] = full["properties"].clone();
        schema["required"] = full["required"].clone();
        schema["$schema"] = json!("https://json-schema.org/draft/2020-12/schema");
        let validator = jsonschema::validator_for(&schema).expect("built-in input schema");
        let mut tool = Tool::new(
            format!("quantum_{name}"),
            description,
            schema
                .as_object()
                .expect("schema generator returns a JSON object")
                .clone(),
        );
        tool.output_schema = Some(Arc::new(
            output_schema(output, scope)
                .as_object()
                .expect("output schema generator returns a JSON object")
                .clone(),
        ));
        let writes = mutation || matches!(output, Output::Save | Output::Glb | Output::Png);
        tool.annotations = Some(
            ToolAnnotations::default()
                .read_only(!writes)
                .destructive(writes)
                .idempotent(true)
                .open_world(false),
        );
        Self {
            tool,
            validator,
            mutation,
            handler,
        }
    }
}

/// Selects common arguments, mutation admission and the handler context together.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    Session,
    Create,
    Read,
    Write,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Output {
    Capabilities,
    Document,
    Save,
    Glb,
    Png,
    Voxels,
    Query,
    Layers,
    Layer,
    Palette,
    Mapping,
    PaletteText,
    Render,
    Comments,
    Message,
    History,
    UndoRedo,
    Version,
    Versions,
    Change,
}

pub(crate) type Field = (&'static str, Value, bool);
pub(crate) fn req(name: &'static str, value: Value) -> Field {
    (name, value, true)
}
pub(crate) fn opt(name: &'static str, value: Value) -> Field {
    (name, value, false)
}
pub(crate) fn string(max: usize) -> Value {
    json!({"type":"string","minLength":1,"maxLength":max})
}
pub(crate) fn bounded_name(max: usize) -> Value {
    let mut schema = string(max);
    schema["pattern"] = json!("^[^\\u0000-\\u001f\\u007f-\\u009f]+$");
    schema["description"] = json!(format!(
        "Nonempty text, at most {max} UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    ));
    schema
}
pub(crate) fn rgba() -> Value {
    json!({"type":"array","prefixItems":[integer(0,255),integer(0,255),integer(0,255),{"const":255}],"items":false,"minItems":4,"maxItems":4,"description":"Opaque sRGB RGBA8; alpha must equal 255."})
}
pub(crate) fn integer(min: i64, max: i64) -> Value {
    json!({"type":"integer","minimum":min,"maximum":max})
}
pub(crate) fn boolean() -> Value {
    json!({"type":"boolean"})
}
pub(crate) fn choice(values: &[&str]) -> Value {
    json!({"type":"string","enum":values})
}
pub(crate) fn array(items: Value, min: usize, max: usize) -> Value {
    json!({"type":"array","items":items,"minItems":min,"maxItems":max})
}
pub(crate) fn uuid() -> Value {
    json!({"type":"string","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"})
}
pub(crate) fn revision() -> Value {
    json!({"type":"string","pattern":"^(0|[1-9][0-9]{0,19})$","description":"Captured monotonic revision, encoded as a decimal u64 string."})
}
pub(crate) fn nullable(v: Value) -> Value {
    json!({"anyOf":[v,{"type":"null"}]})
}
pub(crate) fn object(fields: Vec<Field>) -> Value {
    let required: Vec<_> = fields.iter().filter(|f| f.2).map(|f| f.0).collect();
    let properties: Map<_, _> = fields
        .into_iter()
        .map(|(k, v, _)| (k.to_owned(), v))
        .collect();
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
pub(crate) fn pos() -> Value {
    array(integer(-32768, 32767), 3, 3)
}
pub(crate) fn region() -> Value {
    let mut schema = object(vec![
        req("min", pos()),
        req("max", array(integer(-32767, 32768), 3, 3)),
    ]);
    schema["description"] = json!(
        "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
    );
    schema
}
pub(crate) fn anchor() -> Value {
    json!({"oneOf":[object(vec![req("kind",json!({"const":"voxel"})),req("at",pos())]),object(vec![req("kind",json!({"const":"region"})),req("min",pos()),req("max",array(integer(-32767,32768),3,3))])]})
}
pub(crate) fn slot_schema(air: bool) -> Value {
    json!({"anyOf":[integer(if air {0} else {1},256),string(128)],"description":"Stable slot number or unique swatch name; zero is air for explicit writes only."})
}
pub(crate) fn layer_schema_input() -> Value {
    let mut s = bounded_name(128);
    s["description"] = json!("Layer UUID or unique exact name. Never inferred from UI selection.");
    s
}
pub(crate) fn mode() -> Value {
    choice(&["place", "paint", "erase"])
}
pub(crate) fn camera_schema() -> Value {
    schema::<crate::images::CameraInput>()
}

pub(crate) fn box_constraints(schema: &mut Value) {
    schema["if"] = json!({"properties":{"mode":{"const":"erase"}},"required":["mode"]});
    schema["then"] = json!({"not":{"required":["slot"]}});
    schema["else"] = json!({"required":["slot"]});
}
pub(crate) fn output_schema(shape: Output, scope: Scope) -> Value {
    let error = object(vec![
        req("code", string(80)),
        req("message", string(4096)),
        req("retryable", boolean()),
        req("recovery", string(4096)),
        opt("path", json!({"type":"string"})),
        opt("command_index", integer(0, 255)),
        opt("current_revision", revision()),
    ]);
    let data = data_schema(shape, scope);
    json!({"type":"object","properties":{"api_version":{"const":"quantum.v1"},"session_id":uuid(),"epoch":uuid(),"ok":{"type":"boolean"},"data":data,"error":error},"required":["api_version","session_id","epoch","ok"],"additionalProperties":false,"oneOf":[{"properties":{"ok":{"const":true}},"required":["data"],"not":{"required":["error"]}},{"properties":{"ok":{"const":false}},"required":["error"],"not":{"required":["data"]}}]})
}
pub(crate) fn count() -> Value {
    integer(0, i64::MAX)
}
pub(crate) fn sha256() -> Value {
    json!({"type":"string","pattern":"^[0-9a-f]{64}$"})
}
pub(crate) fn origin() -> Value {
    choice(&["agent", "human"])
}
pub(crate) fn layer_schema() -> Value {
    object(vec![
        req("id", uuid()),
        req("name", bounded_name(128)),
        req("visible", boolean()),
        req("locked", boolean()),
        req("voxel_count", count()),
    ])
}
pub(crate) fn document_fields() -> Vec<Field> {
    vec![
        req("model_id", uuid()),
        req("revision", revision()),
        req("name", bounded_name(128)),
        req("bounds", region()),
        req("layers", array(layer_schema(), 1, 64)),
        req("dirty", boolean()),
        req("voxel_count", count()),
        req("palette_slots", integer(1, 256)),
        req("comments", count()),
        req("versions", integer(0, 64)),
        req("head_id", nullable(uuid())),
        req("redo_id", nullable(uuid())),
    ]
}
pub(crate) fn image_schema(view: bool) -> Value {
    let mut fields = vec![
        req(
            "view",
            choice(&[
                "iso",
                "front",
                "back",
                "left",
                "right",
                "top",
                "camera",
                "contact_sheet",
            ]),
        ),
        req("width", integer(1, 2048)),
        req("height", integer(1, 2048)),
        req("sha256", sha256()),
        req("bytes", count()),
    ];
    if view {
        fields.push(req("camera", camera_schema()));
    }
    object(fields)
}
pub(crate) fn render_output_fields() -> Vec<Field> {
    vec![
        req("backend", json!({"const":"cpu"})),
        req("bounds", region()),
        req("views", array(image_schema(true), 1, 6)),
        req("images", array(image_schema(false), 1, 7)),
        req(
            "image_mode",
            choice(&["contact_sheet", "individual", "both"]),
        ),
    ]
}
/// Output objects allow additive fields at every nesting level within this API
/// major. Values of known fields remain fully typed and locally bounded.
pub(crate) fn additive(schema: &mut Value) {
    match schema {
        Value::Object(map) => {
            if map.get("additionalProperties") == Some(&Value::Bool(false)) {
                map.insert("additionalProperties".into(), Value::Bool(true));
            }
            for value in map.values_mut() {
                additive(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                additive(value);
            }
        }
        _ => {}
    }
}
pub(crate) fn data_schema(shape: Output, scope: Scope) -> Value {
    let mut fields = if scope == Scope::Session {
        vec![]
    } else {
        vec![req("model_id", uuid()), req("revision", revision())]
    };
    if scope == Scope::Write && shape != Output::Save {
        fields.extend([
            req("operation_id", uuid()),
            req("undo_entry_id", nullable(uuid())),
            req("head_id", nullable(uuid())),
            req("changed_voxels", count()),
            req("added", count()),
            req("removed", count()),
            req("recolored", count()),
            req("changed_bounds", nullable(region())),
            req("changed_layers", array(uuid(), 0, 64)),
            req("palette_changed", boolean()),
            req("metadata_changed", boolean()),
            req("work_units", count()),
            req("durability", json!({"const":"memory"})),
        ]);
    }
    match shape {
        Output::Capabilities => fields.extend([
            req("api_version", json!({"const":"quantum.v1"})),
            req("protocol_versions", array(choice(&["2025-11-25", "2026-07-28"]), 2, 2)),
            req("limits", object(["open_documents", "request_bytes", "result_bytes", "batch_operations", "explicit_voxels", "read_voxels", "work_units", "operation_receipts", "receipt_bytes"].into_iter().map(|key| req(key, count())).collect())),
            req("documents", array(object(document_fields()), 0, 8)),
            req("renderer", json!({"const":"cpu"})),
            req("formats", array(string(16), 1, 6)), req("presets", array(string(32), 1, 7)),
            req("durability", json!({"const":"memory"})),
        ]),
        Output::Document => fields = document_fields(),
        Output::Save | Output::Glb | Output::Png => {
            fields.extend([req("path", string(1024)), req("bytes", count()), req("sha256", sha256())]);
            match shape {
                Output::Save => fields.push(req("dirty", json!({"const":false}))),
                Output::Glb => fields.push(req("mime_type", json!({"const":"model/gltf-binary"}))),
                _ => fields.extend(render_output_fields()),
            }
        }
        Output::Voxels => fields.extend([
            req("layer_id", uuid()),
            req("voxels", array(object(vec![req("at", pos()), req("slot", integer(0, 256))]), 1, 10000)),
        ]),
        Output::Query => fields.extend([
            req("bounds", region()), req("region", region()), req("occupied_bounds", nullable(region())),
            req("voxel_count", count()), req("view", choice(&["layers", "composite"])),
            opt("slots", json!({"type":"object","propertyNames":{"pattern":"^[0-9]{1,3}$"},"additionalProperties":count()})),
            opt("next_offset", nullable(count())),
            opt("voxels", array(object(vec![req("layer_id", nullable(uuid())), req("at", pos()), req("slot", integer(1, 256))]), 0, 10000)),
        ]),
        Output::Layers => fields.push(req("layers", array(layer_schema(), 1, 64))),
        Output::Layer => fields.push(req("layer_id", uuid())),
        Output::Palette => fields.extend([
            req("palette", object(vec![
                req("slots", array(object(vec![req("index", integer(1, 256)), req("rgba", rgba()), opt("name", bounded_name(128))]), 1, 256)),
                req("order", array(integer(1, 256), 1, 256)),
            ])),
            req("presets", array(string(32), 1, 7)),
        ]),
        Output::Mapping => fields.push(req("slot_mapping", array(object(vec![req("source", integer(1, 256)), req("slot", integer(1, 256))]), 1, 256))),
        Output::PaletteText => fields.extend([req("text", string(65536)), req("format", choice(&["hex", "gpl", "pal"]))]),
        Output::Render => fields.extend(render_output_fields()),
        Output::Comments => fields.extend([
            req("comments", array(object(vec![
                req("thread", object(vec![
                    req("id", uuid()), req("anchor", anchor()), req("layer", nullable(uuid())), req("resolved", boolean()),
                    req("messages", array(object(vec![req("id", uuid()), req("origin", origin()), req("text", string(8192)), req("tool", nullable(bounded_name(128)))]), 0, 200)),
                ])),
                req("message_count", count()), req("next_message_offset", nullable(count())),
                req("anchor_state", choice(&["visible", "hidden", "orphaned"])),
            ]), 0, 200)),
            req("total", count()), req("next_offset", nullable(count())),
        ]),
        Output::Message => fields.extend([req("thread_id", uuid()), req("message_id", uuid())]),
        Output::History => fields.extend([
            req("entries", array(object(vec![
                req("entry_id", uuid()), req("origin", origin()), req("tool", bounded_name(128)), req("label", bounded_name(128)),
                req("base_revision", revision()), req("result_revision", revision()),
                req("added", count()), req("removed", count()), req("recolored", count()), req("changed_bounds", nullable(region())),
            ]), 0, 200)),
            req("total", count()), req("next_offset", nullable(count())),
            req("head_id", nullable(uuid())), req("redo_id", nullable(uuid())),
            req("initial_retained", boolean()), req("history_bytes", count()),
        ]),
        Output::UndoRedo => fields.push(req("redo_id", nullable(uuid()))),
        Output::Version => fields.push(req("version_id", uuid())),
        Output::Versions => fields.push(req("versions", array(object(vec![
            req("id", uuid()), req("name", bounded_name(128)), req("parent_id", nullable(uuid())), req("origin", origin()), req("content_hash", sha256()),
        ]), 0, 64))),
        _ => {}
    }
    let mut schema = object(fields);
    additive(&mut schema);
    schema
}
