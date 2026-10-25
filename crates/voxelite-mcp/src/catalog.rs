//! Single source for discovery, input validation, and the checked-in API catalog.
use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{Map, Value, json};
use std::sync::{Arc, LazyLock};

pub(crate) struct Spec {
    pub tool: Tool,
    pub validator: jsonschema::Validator,
    pub mutation: bool,
}

pub(crate) static CATALOG: LazyLock<Vec<Spec>> = LazyLock::new(build);
type Field = (&'static str, Value, bool);
fn req(name: &'static str, value: Value) -> Field { (name, value, true) }
fn opt(name: &'static str, value: Value) -> Field { (name, value, false) }
fn string(max: usize) -> Value { json!({"type":"string","minLength":1,"maxLength":max}) }
fn integer(min: i64, max: i64) -> Value { json!({"type":"integer","minimum":min,"maximum":max}) }
fn boolean() -> Value { json!({"type":"boolean"}) }
fn choice(values: &[&str]) -> Value { json!({"type":"string","enum":values}) }
fn array(items: Value, min: usize, max: usize) -> Value { json!({"type":"array","items":items,"minItems":min,"maxItems":max}) }
fn uuid() -> Value { json!({"type":"string","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"}) }
fn revision() -> Value { json!({"type":"string","pattern":"^(0|[1-9][0-9]{0,19})$","description":"Captured monotonic revision, encoded as a decimal u64 string."}) }
fn nullable(v: Value) -> Value { json!({"anyOf":[v,{"type":"null"}]}) }
fn object(fields: Vec<Field>) -> Value {
    let required: Vec<_> = fields.iter().filter(|f| f.2).map(|f| f.0).collect();
    let properties: Map<_, _> = fields.into_iter().map(|(k,v,_)| (k.to_owned(),v)).collect();
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
fn pos() -> Value { array(integer(-32768,32767),3,3) }
fn region() -> Value { object(vec![req("min",pos()),req("max",array(integer(-32767,32768),3,3))]) }
fn slot(air: bool) -> Value { json!({"anyOf":[integer(if air {0} else {1},256),string(128)],"description":"Stable slot number or unique swatch name; zero is air for explicit writes only."}) }
fn layer() -> Value { let mut s=string(128); s["description"]=json!("Layer UUID or unique exact name. Never inferred from UI selection."); s }
fn mirror() -> Value { let mut axes=array(choice(&["x","y","z"]),0,3); axes["uniqueItems"]=json!(true); object(vec![req("axes",axes),req("plane2",array(integer(-65536,65536),3,3))]) }
fn mode() -> Value { choice(&["place","paint","erase"]) }
fn voxel_fields() -> Vec<Field> { vec![req("layer",layer()),req("voxels",array(object(vec![req("at",pos()),req("slot",slot(true))]),1,65536)),opt("mode",choice(&["place","paint"])),opt("mirror",mirror())] }
fn erase_fields() -> Vec<Field> { vec![req("layer",layer()),req("region",region()),opt("mirror",mirror())] }
fn fill_fields() -> Vec<Field> { vec![req("layer",layer()),req("seed",pos()),req("slot",slot(false)),opt("region",region()),opt("cap",integer(1,1048576)),opt("mirror",mirror())] }
fn box_fields() -> Vec<Field> { vec![req("layer",layer()),req("region",region()),opt("slot",slot(false)),opt("hollow",boolean()),opt("mode",mode()),opt("mirror",mirror())] }
fn read_fields() -> Vec<Field> { vec![req("model_id",uuid()),opt("revision",revision())] }
fn write_fields(new: bool) -> Vec<Field> {
    let mut f=vec![req("expected_epoch",uuid()),req("operation_id",uuid())];
    if !new { f.extend([req("model_id",uuid()),req("expected_revision",revision())]); }
    f.push(opt("label",string(80))); f
}
fn render_fields() -> Vec<Field> {
    let number=json!({"type":"number","minimum":-1e9,"maximum":1e9});
    let camera=object(vec![req("eye",array(number.clone(),3,3)),req("target",array(number.clone(),3,3)),req("up",array(number,3,3)),req("projection",choice(&["orthographic","perspective"])),opt("ortho_height",json!({"type":"number","exclusiveMinimum":0,"maximum":1e9})),opt("fov_y_degrees",json!({"type":"number","exclusiveMinimum":1,"exclusiveMaximum":179}))]);
    vec![opt("views",array(choice(&["iso","front","back","left","right","top"]),1,6)),opt("camera",camera),opt("width",integer(1,1024)),opt("height",integer(1,1024)),opt("perspective",boolean()),opt("background",array(integer(0,255),4,4)),opt("image_mode",choice(&["contact_sheet","individual","both"]))]
}
fn output_schema() -> Value {
    let error=object(vec![req("code",string(80)),req("message",string(4096)),req("retryable",boolean()),req("recovery",string(4096)),opt("path",json!({"type":"string"})),opt("command_index",integer(0,255)),opt("current_revision",revision())]);
    json!({"type":"object","properties":{"api_version":{"const":"voxelite.v1"},"session_id":uuid(),"epoch":uuid(),"ok":{"type":"boolean"},"data":{"type":"object"},"error":error},"required":["api_version","session_id","epoch","ok"],"additionalProperties":false,"oneOf":[{"properties":{"ok":{"const":true}},"required":["data"],"not":{"required":["error"]}},{"properties":{"ok":{"const":false}},"required":["error"],"not":{"required":["data"]}}]})
}
fn build() -> Vec<Spec> {
    let mut specs=Vec::new();
    let mut add=|name: &'static str, description: &'static str, mut fields: Vec<Field>, scope: u8| {
        let mutation=scope==2 || scope==3;
        let mut common=match scope {1=>read_fields(),2=>write_fields(false),3=>write_fields(true),_=>vec![]};
        common.append(&mut fields);
        let mut schema=object(common); schema["$schema"]=json!("https://json-schema.org/draft/2020-12/schema");
        let validator=jsonschema::validator_for(&schema).expect("built-in input schema");
        let mut tool=Tool::new(format!("voxelite_{name}"),description,schema.as_object().unwrap().clone());
        tool.output_schema=Some(Arc::new(output_schema().as_object().unwrap().clone()));
        tool.annotations=Some(ToolAnnotations::default().read_only(!mutation && !matches!(name,"document_save"|"export_glb"|"export_png")).destructive(mutation || matches!(name,"document_save"|"export_glb"|"export_png")).idempotent(true).open_world(false));
        specs.push(Spec {tool,validator,mutation});
    };
    add("capabilities_get","Discover voxelite.v1, session epoch, limits, CPU renderer, exact palette presets and open model IDs. Start here; use returned epoch and revisions for edits.",vec![],0);
    add("document_create","Create a revision-zero document and visible unlocked Base layer. Default bounds [0,0,0]..[32,32,32]. Returns model/layer IDs. Up to eight documents per session; never replaces an open model.",vec![req("name",string(128)),opt("bounds",region()),opt("model_id",uuid()),opt("layer_id",uuid())],3);
    add("document_open","Open a canonical .voxl file relative to the configured project root, preserving IDs, comments, versions and unknown metadata. Validation finishes before installation. Duplicate model IDs reject.",vec![req("path",string(1024))],3);
    add("document_save","Atomically save .voxl under the project root. New paths must not exist; overwriting the same opened/saved path checks its last SHA-256. Does not create an undo entry. Changes stay in memory until saved.",vec![req("path",string(1024))],2);
    add("document_info","Read document identity, bounds, layers, counts, dirty state and current revision. Optional revision must equal the live revision; no UI state is used.",vec![],1);
    add("voxels_get","Read explicit integer coordinates on one layer, including air as slot 0. Returns values in request order. Out-of-bounds coordinates reject the entire read.",vec![req("layer",layer()),req("positions",array(pos(),1,10000))],1);
    add("voxels_set","Atomically place or paint explicit cells, at most 65,536. Slot 0 erases; paint affects occupied cells only. Last duplicate wins before mirror expansion; mirror conflicts reject. Omitted mode=place, mirror=none. One undo entry.",voxel_fields(),2);
    add("voxels_erase_region","Erase occupied cells in min-inclusive/max-exclusive region. Mirror is explicit per call; omitted means none. Invalid/hidden/locked targets reject without partial edits.",erase_fields(),2);
    add("voxels_fill","Six-connected flood fill of the seed's slot, including air. Region defaults to document bounds; cap defaults to 1,048,576 visited cells. Cap exhaustion rejects atomically. Mirror never inherits UI state.",fill_fields(),2);
    add("voxels_box","Build an integer half-open solid box or one-cell shell (hollow=true). Default mode=place, hollow=false, mirror=none. slot required for place/paint, forbidden for erase. One atomic undo entry.",box_fields(),2);
    let variants:Vec<_>=[("set",voxel_fields()),("erase_region",erase_fields()),("fill",fill_fields()),("box",box_fields())].into_iter().map(|(k,mut f)|{f.push(req("kind",json!({"const":k})));object(f)}).collect();
    add("voxels_batch","Apply 1..256 ordered voxel operations atomically to one model: set, erase_region, fill, box. Each operation names its own layer/mirror; nesting and I/O are forbidden. Later operations see earlier edits; any failure rolls back all; one revision and undo entry.",vec![req("operations",array(json!({"oneOf":variants}),1,256))],2);
    add("query","Query bounds, counts or paged occupied voxels within a half-open region. view=layers (default) counts overlaps separately; composite uses highest visible layer. Optional layer restricts stored-layer queries. limit defaults 1000, max 10000. Use returned revision with next_offset to prevent mixed pages.",vec![req("kind",choice(&["bounds","counts","voxels"])),opt("region",region()),opt("layer",layer()),opt("view",choice(&["layers","composite"])),opt("offset",integer(0,1048576)),opt("limit",integer(1,10000))],1);
    add("layers_list","List stable layer IDs, exact names, visibility, lock state and stored voxel counts in bottom-to-top order.",vec![],1);
    add("layers_add","Insert an empty layer at bottom-to-top index (default append). Optional layer_id supports deterministic references. Maximum 64 layers.",vec![req("name",string(128)),opt("layer_id",uuid()),opt("index",integer(0,64))],2);
    add("layers_remove","Delete one unlocked layer and its voxels as one undoable edit. Final-layer deletion rejects; anchored comments remain and may become orphaned.",vec![req("layer",layer())],2);
    add("layers_rename","Rename a layer without changing its stable ID. Duplicate names are allowed, but future name-based access must be unambiguous.",vec![req("layer",layer()),req("name",string(128))],2);
    add("layers_reorder","Set an exact bottom-to-top permutation of all layer IDs/names. The highest visible layer wins at overlaps; omitted or duplicate layers reject.",vec![req("order",array(layer(),1,64))],2);
    add("layers_visibility","Set visibility explicitly. Hidden cells stay stored and cannot be edited until made visible. Metadata changes are undoable.",vec![req("layer",layer()),req("visible",boolean())],2);
    add("layers_lock","Set lock explicitly. Unlock in a separate operation before voxel editing; all mutations share the core lock rules and history.",vec![req("layer",layer()),req("locked",boolean())],2);
    add("palette_list","List stable palette slots, names, RGBA colors, display order and exact preset names. Slot 0 is reserved for air.",vec![],1);
    add("palette_set_slot","Add or replace stable slot 1..256 with opaque RGBA. Existing references change appearance, including locked layers. Omitted name creates an unnamed swatch. One undoable edit.",vec![req("slot",integer(1,256)),req("rgba",array(integer(0,255),4,4)),opt("name",string(128))],2);
    add("palette_apply_preset","Append exact preset colors into unused slots without recoloring existing voxels. Returns source-to-new slot mapping. Rejects atomically if capacity would exceed 256; originals are immutable.",vec![req("preset",choice(&["pico8","endesga32","sweetie16","grayscale","neutral","warm","vivid"]))],2);
    add("palette_import","Append colors from bounded .hex or .gpl text into unused slots; no implicit geometry recoloring. Returns source-to-new mapping; malformed input or insufficient capacity leaves palette unchanged.",vec![req("format",choice(&["hex","gpl"])),req("text",string(65536))],2);
    add("palette_export","Return palette as .hex or .gpl text in display order; GPL preserves representable names. Read-only, no filesystem side effects.",vec![req("format",choice(&["hex","gpl"]))],1);
    add("render","Render CPU PNG image content from one immutable revision. Default four views [iso,front,right,back] at 512², transparent background, orthographic, 2x AA, labeled contact sheet. Up to six views or one explicit camera; <=4,194,304 combined image/sheet pixels. image_mode=contact_sheet|individual|both. Returns camera metadata and hashes; no GPU/display required.",render_fields(),1);
    let mut png_fields=render_fields(); png_fields.push(req("path",string(1024)));
    add("export_png","Render and atomically write one PNG under project root. Default contact sheet; for individual image use one view. Existing paths reject. Returns size, SHA-256 and camera metadata; document/history unchanged.",png_fields,1);
    add("export_glb","Export deterministic GLB of the highest visible layer at each occupied coordinate. Atomically write a new path under project root; existing paths reject. Returns size and SHA-256; document/history unchanged.",vec![req("path",string(1024))],1);
    add("comments_list","List anchored threads including messages, agent/human roles, resolved state and anchor status. Optional state=open|resolved|all (default all). Paged at 50 by default, max 200; bind pages with revision.",vec![opt("state",choice(&["open","resolved","all"])),opt("offset",integer(0,512)),opt("limit",integer(1,200))],1);
    let anchor=json!({"oneOf":[object(vec![req("kind",json!({"const":"voxel"})),req("at",pos())]),object(vec![req("kind",json!({"const":"region"})),req("min",pos()),req("max",array(integer(-32767,32768),3,3))])]});
    add("comments_add","Create a persistent voxel/region anchored thread. Author is always agent, never caller-supplied. Text is plain text, max 4096 UTF-8 bytes. Returns thread/message IDs; undoable and saved in .voxl.",vec![req("anchor",anchor),opt("layer",layer()),req("text",string(4096)),opt("thread_id",uuid()),opt("message_id",uuid())],2);
    add("comments_reply","Append an agent-attributed plain-text reply to a thread, resolved or open. One undoable edit; returns message ID.",vec![req("thread_id",uuid()),req("text",string(4096)),opt("message_id",uuid())],2);
    for (name,desc) in [("comments_resolve","Resolve an anchored discussion; idempotent if already resolved. Preserves messages and anchor; undoable."),("comments_reopen","Reopen a resolved discussion; idempotent if already open. Preserves messages and anchor; undoable.")] {add(name,desc,vec![req("thread_id",uuid())],2);}
    add("history_read","Read chronological applied history with agent origins, tool names, labels and change summaries. Includes exact undo/redo entry IDs. Default page 50, max 200; history is bounded and resets on reopen.",vec![opt("offset",integer(0,1048576)),opt("limit",integer(1,200))],1);
    for (name,desc) in [("history_undo","Undo exactly the expected top entry. Requires captured epoch/revision/entry_id; intervening edits reject. Revision increases monotonically; original entry retains its agent attribution."),("history_redo","Redo exactly the expected redo entry. Requires captured epoch/revision/entry_id. A new edit clears the redo branch; stale IDs reject.")] {add(name,desc,vec![req("entry_id",uuid())],2);}
    add("versions_save","Persist current content as a named immutable snapshot with agent attribution and content hash. Up to 64 versions, no recursive libraries. One undoable library edit; save document to retain across process restarts.",vec![req("name",string(128)),opt("version_id",uuid()),opt("parent_id",uuid())],2);
    add("versions_list","List named version IDs, names, parent IDs, origin and immutable content hashes. Persistent across save/open; independent of undo retention. Never includes full voxel snapshots.",vec![],1);
    add("versions_restore","Restore named content as a new undoable edit, keeping the version library. Requires expected_head_id (null for empty history) in addition to revision/epoch. Core validates current locks before installation.",vec![req("version_id",uuid()),req("expected_head_id",nullable(uuid()))],2);
    specs.sort_by(|a,b|a.tool.name.cmp(&b.tool.name)); specs
}
