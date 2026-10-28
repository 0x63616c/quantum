//! Each tool's typed input, metadata and handler are registered together here.
use crate::{
    arguments::{WriteCommon, arguments, schema},
    catalog::*,
    comments, edit,
    error::{Failure, Result},
    exports, files, images, palettes,
    session::{self, Session, change, info, layers, read},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use quantum_core::*;
use quantum_format::session::{DocumentSession, SavePolicy};
use quantum_plugin_api::{CommentLayer, CommentsAction, CommentsQuery, PaletteAction};
use quantum_plugin_host::Host;
use rmcp::model::ContentBlock;
use serde_json::{Value, json};
use std::path::Path;

type ToolResult = Result<(Value, Vec<ContentBlock>)>;
pub(crate) enum Handler {
    Session(fn(&mut Session, &Value, &mut Host) -> ToolResult),
    Document(fn(&mut DocumentCx<'_>, &Value) -> ToolResult),
}
pub(crate) struct DocumentCx<'a> {
    pub root: &'a Path,
    pub client_id: Uuid,
    pub doc: &'a mut DocumentSession,
    pub host: &'a mut Host,
    pub name: &'a str,
    pub write: &'a WriteCommon,
}
impl DocumentCx<'_> {
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: change and tool extras are constructed as JSON objects"
    )]
    fn edit(&mut self, command: Command, extras: Value) -> ToolResult {
        let engine = &mut self.doc.engine;
        let operation_id = self.write.identity.operation_id.get()?;
        let receipt = engine.execute(Transaction {
            expected_revision: engine.revision(),
            id: operation_id,
            actor: Actor {
                origin: Origin::Agent,
                client_id: self.client_id,
                tool: format!("quantum_{}", self.name),
            },
            label: self.write.label.default(self.name.replace('_', " "))?,
            command,
        })?;
        let mut data = change(&receipt);
        data.as_object_mut()
            .expect("change and tool extras are constructed as JSON objects")
            .extend(
                extras
                    .as_object()
                    .expect("change and tool extras are constructed as JSON objects")
                    .clone(),
            );
        data["operation_id"] = json!(operation_id);
        data["head_id"] = json!(engine.head());
        read(engine, data)
    }
}

macro_rules! tools {
    ($($ty:ident => $handler:ident [$scope:ident, $output:ident] $description:literal
       { $($fields:tt)* } $(where $constraint:path)?; )*) => {
        $(arguments!($ty { $($fields)* } $(where $constraint)?);)*
        pub(crate) fn specs() -> Vec<Spec> {
            let mut specs = vec![$(Spec::new::<$ty>(stringify!($handler), $description, Scope::$scope, Output::$output,
                tools!(@route $scope $ty $handler))),*];
            specs.sort_by(|a,b| a.tool.name.cmp(&b.tool.name));
            specs
        }
    };
    (@route Create $ty:ident $handler:ident) => { tools!(@route Session $ty $handler) };
    (@route Read $ty:ident $handler:ident) => { tools!(@route Document $ty $handler) };
    (@route Write $ty:ident $handler:ident) => { tools!(@route Document $ty $handler) };
    (@route Session $ty:ident $handler:ident) => { Handler::Session(|s, a, h| $handler(s, &$ty::from(a), h)) };
    (@route Document $ty:ident $handler:ident) => { Handler::Document(|cx, a| $handler(cx, &$ty::from(a))) };
}

tools! {
    CapabilitiesGet => capabilities_get [Session, Capabilities]
        "Discover quantum.v1, session epoch, limits, CPU renderer, exact palette presets and open model IDs. Start here; use returned epoch and revisions for edits." {
    };
    CommentsAdd => comments_add [Write, Message]
        "Create a persistent voxel/region anchored thread. Author is always agent, never caller-supplied. Text is plain text, max 4096 UTF-8 bytes. Returns thread/message IDs; undoable and saved in .voxl." {
        req anchor: Anchor => anchor(),
        opt layer: String => layer_schema_input(),
        req text: String => string(4096),
        opt thread_id: Uuid => uuid(),
        opt message_id: Uuid => uuid(),
    };
    CommentsList => comments_list [Read, Comments]
        "List anchored threads including messages, agent/human roles, resolved state and anchor status. Optional state=open|resolved|all (default all). Thread and message pages default to 50, max 200. message_offset applies per thread; bind pages with revision." {
        opt state: String => choice(&["open", "resolved", "all"]),
        opt offset: usize => integer(0,1024),
        opt limit: usize => integer(1,200),
        opt message_offset: usize => integer(0,10000),
        opt message_limit: usize => integer(1,200),
    };
    CommentsReopen => comments_reopen [Write, Change]
        "Reopen a resolved discussion; idempotent if already open. Preserves messages and anchor; undoable." {
        req thread_id: Uuid => uuid(),
    };
    CommentsReply => comments_reply [Write, Message]
        "Append an agent-attributed plain-text reply to a thread, resolved or open. One undoable edit; returns message ID." {
        req thread_id: Uuid => uuid(),
        req text: String => string(4096),
        opt message_id: Uuid => uuid(),
    };
    CommentsResolve => comments_resolve [Write, Change]
        "Resolve an anchored discussion; idempotent if already resolved. Preserves messages and anchor; undoable." {
        req thread_id: Uuid => uuid(),
    };
    DocumentCreate => document_create [Create, Document]
        "Create a revision-zero document and visible unlocked Base layer. Default bounds [0,0,0]..[32,32,32]. Returns model/layer IDs. Up to eight documents per session; never replaces an open model." {
        req name: String => bounded_name(128),
        opt bounds: Region => region(),
        opt model_id: Uuid => uuid(),
        opt layer_id: Uuid => uuid(),
    };
    DocumentInfo => document_info [Read, Document]
        "Read document identity, bounds, layers, counts, dirty state and current revision. Optional revision must equal the live revision; no UI state is used." {
    };
    DocumentOpen => document_open [Create, Document]
        "Open a canonical .voxl file relative to the configured project root, preserving IDs, comments, versions and unknown metadata. Validation finishes before installation. Duplicate model IDs reject." {
        req path: String => string(1024),
    };
    DocumentSave => document_save [Write, Save]
        "Atomically save .voxl under the project root. New paths must not exist; overwriting the same opened/saved path checks its last SHA-256. Does not create an undo entry. Changes stay in memory until saved." {
        req path: String => string(1024),
    };
    ExportGlb => export_glb [Read, Glb]
        "Export deterministic GLB of the highest visible layer at each occupied coordinate. Atomically write a new path under project root; existing paths reject. Returns size and SHA-256; document/history unchanged." {
        req path: String => string(1024),
    };
    ExportPng => export_png [Read, Png]
        "Render and atomically write one PNG under project root. Default contact sheet; for individual image use one view. Existing paths reject. Returns size, SHA-256 and camera metadata; document/history unchanged." {
        flat render: images::RenderInput => (),
        req path: String => string(1024),
    } where images::png_constraints;
    HistoryRead => history_read [Read, History]
        "Read chronological applied history with agent origins, tool names, labels and change summaries. Includes exact undo/redo entry IDs. Default page 50, max 200; history is bounded and resets on reopen." {
        opt offset: usize => integer(0,1048576),
        opt limit: usize => integer(1,200),
    };
    HistoryRedo => history_redo [Write, UndoRedo]
        "Redo exactly the expected redo entry. Requires captured epoch/revision/entry_id. A new edit clears the redo branch; stale IDs reject." {
        req entry_id: Uuid => uuid(),
    };
    HistoryUndo => history_undo [Write, UndoRedo]
        "Undo exactly the expected top entry. Requires captured epoch/revision/entry_id; intervening edits reject. Revision increases monotonically; original entry retains its agent attribution." {
        req entry_id: Uuid => uuid(),
    };
    LayersAdd => layers_add [Write, Layer]
        "Insert an empty layer at bottom-to-top index (default append). Optional layer_id supports deterministic references. Maximum 64 layers." {
        req name: String => bounded_name(128),
        opt layer_id: Uuid => uuid(),
        opt index: usize => integer(0,64),
    };
    LayersList => layers_list [Read, Layers]
        "List stable layer IDs, exact names, visibility, lock state and stored voxel counts in bottom-to-top order." {
    };
    LayersLock => layers_lock [Write, Change]
        "Set lock explicitly. Unlock in a separate operation before voxel editing; all mutations share the core lock rules and history." {
        req layer: String => layer_schema_input(),
        req locked: bool => boolean(),
    };
    LayersRemove => layers_remove [Write, Change]
        "Delete one unlocked layer and its voxels as one undoable edit. Final-layer deletion rejects; anchored comments remain and may become orphaned." {
        req layer: String => layer_schema_input(),
    };
    LayersRename => layers_rename [Write, Change]
        "Rename a layer without changing its stable ID. Duplicate names are allowed, but future name-based access must be unambiguous." {
        req layer: String => layer_schema_input(),
        req name: String => bounded_name(128),
    };
    LayersReorder => layers_reorder [Write, Change]
        "Set an exact bottom-to-top permutation of all layer IDs/names. The highest visible layer wins at overlaps; omitted or duplicate layers reject." {
        req order: Vec<String> => array(layer_schema_input(),1,64),
    };
    LayersVisibility => layers_visibility [Write, Change]
        "Set visibility explicitly. Hidden cells stay stored and cannot be edited until made visible. Metadata changes are undoable." {
        req layer: String => layer_schema_input(),
        req visible: bool => boolean(),
    };
    PaletteApplyPreset => palette_apply_preset [Write, Mapping]
        "Append exact preset colors into unused slots without recoloring existing voxels. Returns source-to-new slot mapping. Rejects atomically if capacity would exceed 256; originals are immutable." {
        req preset: String => choice(&["pico8", "endesga32", "sweetie16", "grayscale", "neutral", "warm", "vivid"]),
    };
    PaletteExport => palette_export [Read, PaletteText]
        "Return palette as .hex, .gpl or JASC-PAL .pal text in display order; GPL preserves representable names. Read-only, no filesystem side effects." {
        req format: String => choice(&["hex", "gpl", "pal"]),
    };
    PaletteImport => palette_import [Write, Mapping]
        "Append colors from bounded .hex, .gpl or JASC-PAL .pal text into unused slots; no implicit geometry recoloring. Returns source-to-new mapping; malformed input or insufficient capacity leaves palette unchanged." {
        req format: String => choice(&["hex", "gpl", "pal"]),
        req text: String => string(65536),
    };
    PaletteList => palette_list [Read, Palette]
        "List stable palette slots, names, RGBA colors, display order and exact preset names. Slot 0 is reserved for air." {
    };
    PaletteSetSlot => palette_set_slot [Write, Change]
        "Add or replace stable slot 1..256 with opaque RGBA. Existing references change appearance, including locked layers. Omitted name creates an unnamed swatch. One undoable edit." {
        req slot: u16 => integer(1,256),
        req rgba: [u8;4] => rgba(),
        opt name: String => bounded_name(128),
    };
    Query => query [Read, Query]
        "Query bounds, counts or paged occupied voxels within a half-open region. view=layers (default) counts overlaps separately; composite uses highest visible layer. Optional layer restricts stored-layer queries. limit defaults 1000, max 10000. Use returned revision with next_offset to prevent mixed pages." {
        req kind: String => choice(&["bounds", "counts", "voxels"]),
        opt region: Region => region(),
        opt layer: String => layer_schema_input(),
        opt view: String => choice(&["layers", "composite"]),
        opt offset: usize => integer(0,1048576),
        opt limit: usize => integer(1,10000),
    } where query_constraints;
    Render => render [Read, Render]
        "Render CPU PNG image content from one immutable revision. Default four views [iso,front,right,back] at 512², transparent background, orthographic, 2x AA, labeled contact sheet. Up to six views or one explicit camera; <=4,194,304 combined image/sheet pixels. image_mode=contact_sheet|individual|both. Returns camera metadata and hashes; no GPU/display required." {
        flat render: images::RenderInput => (),
    } where images::render_constraints;
    VersionsList => versions_list [Read, Versions]
        "List named version IDs, names, parent IDs, origin and immutable content hashes. Persistent across save/open; independent of undo retention. Never includes full voxel snapshots." {
    };
    VersionsRestore => versions_restore [Write, Change]
        "Restore named content as a new undoable edit, keeping the version library. Requires expected_head_id (null for empty history) in addition to revision/epoch. Core validates current locks before installation." {
        req version_id: Uuid => uuid(),
        req expected_head_id: Option<Uuid> => nullable(uuid()),
    };
    VersionsSave => versions_save [Write, Version]
        "Persist current content as a named immutable snapshot with agent attribution and content hash. Up to 64 versions, no recursive libraries. One undoable library edit; save document to retain across process restarts." {
        req name: String => bounded_name(128),
        opt version_id: Uuid => uuid(),
        opt parent_id: Uuid => uuid(),
    };
    VoxelsBatch => voxels_batch [Write, Change]
        "Apply 1..256 ordered voxel operations atomically to one model: set, erase_region, fill, box. Each operation names its own layer/mirror; nesting and I/O are forbidden. Later operations see earlier edits; any failure rolls back all; one revision and undo entry." {
        req operations: Vec<edit::Operation> => array(schema::<edit::Operation>(),1,256),
    };
    VoxelsBox => voxels_box [Write, Change]
        "Build an integer half-open solid box or one-cell shell (hollow=true). Default mode=place, hollow=false, mirror=none. slot required for place/paint, forbidden for erase. One atomic undo entry." {
        flat input: edit::BoxInput => (),
    } where box_constraints;
    VoxelsEraseRegion => voxels_erase_region [Write, Change]
        "Erase occupied cells in min-inclusive/max-exclusive region. Mirror is explicit per call; omitted means none. Invalid/hidden/locked targets reject without partial edits." {
        flat input: edit::EraseInput => (),
    };
    VoxelsFill => voxels_fill [Write, Change]
        "Six-connected flood fill of the seed's slot, including air. Region defaults to document bounds; cap defaults to 1,048,576 visited cells. Cap exhaustion rejects atomically. Mirror never inherits UI state." {
        flat input: edit::FillInput => (),
    };
    VoxelsGet => voxels_get [Read, Voxels]
        "Read explicit integer coordinates on one layer, including air as slot 0. Returns values in request order. Out-of-bounds coordinates reject the entire read." {
        req layer: String => layer_schema_input(),
        req positions: Vec<Pos> => array(pos(),1,10000),
    };
    VoxelsSet => voxels_set [Write, Change]
        "Atomically place or paint explicit cells, at most 65,536. Slot 0 erases; paint affects occupied cells only. Last duplicate wins before mirror expansion; mirror conflicts reject. Omitted mode=place, mirror=none. One undo entry." {
        flat input: edit::SetInput => (),
    };
}
fn query_constraints(s: &mut Value) {
    s["not"] = json!({"required":["view","layer"],"properties":{"view":{"const":"composite"}}});
}

fn capabilities_get(s: &mut Session, _: &CapabilitiesGet, host: &mut Host) -> ToolResult {
    s.capabilities(host)
}
fn document_create(s: &mut Session, a: &DocumentCreate, _: &mut Host) -> ToolResult {
    let data = s.install(|_, limits| {
        let mut document = Document::new(
            a.model_id.default(Uuid::new_v4())?,
            a.layer_id.default(Uuid::new_v4())?,
        );
        document.content.name = a.name.get()?;
        document.content.bounds = a.bounds.default(Region::default())?;
        Ok(DocumentSession::new(
            quantum_format::Project::new(document)?,
            None,
            limits,
        )?)
    })?;
    Ok((data, vec![]))
}
fn document_open(s: &mut Session, a: &DocumentOpen, _: &mut Host) -> ToolResult {
    let data = s.install(|root, limits| {
        let path = files::path(root, &a.path.get()?)?;
        Ok(DocumentSession::open(&path, limits)?)
    })?;
    Ok((data, vec![]))
}
fn document_info(cx: &mut DocumentCx<'_>, _: &DocumentInfo) -> ToolResult {
    Ok((info(cx.doc), vec![]))
}
fn document_save(cx: &mut DocumentCx<'_>, a: &DocumentSave) -> ToolResult {
    let path = files::path(cx.root, &a.path.get()?)?;
    let saved = cx
        .doc
        .prepare_save(&path, SavePolicy::CreateOrTracked)
        .write()?;
    let data = json!({"model_id":cx.doc.engine.document().id,"revision":cx.doc.engine.revision().to_string(),"path":a.path.get()?,"sha256":saved.fingerprint(),"bytes":saved.bytes(),"dirty":false});
    cx.doc.finish_save(saved);
    Ok((data, vec![]))
}
fn voxels_get(cx: &mut DocumentCx<'_>, a: &VoxelsGet) -> ToolResult {
    let engine = &cx.doc.engine;
    let s = &engine.document().content;
    let id = edit::layer(s, &a.layer.get()?)?;
    let l = s.layer(id)?;
    let positions = a.positions.get()?;
    let mut voxels = Vec::new();
    for at in positions {
        if !s.bounds.contains(at) {
            return Err(Error::OutOfBounds(at).into());
        }
        voxels.push(json!({"at":at,"slot":l.voxels.get(at)}));
    }
    read(engine, json!({"layer_id":id,"voxels":voxels}))
}
fn query(cx: &mut DocumentCx<'_>, a: &Query) -> ToolResult {
    read(
        &cx.doc.engine,
        session::query(&cx.doc.engine.document().content, a)?,
    )
}
fn voxels_set(cx: &mut DocumentCx<'_>, a: &VoxelsSet) -> ToolResult {
    let command = a.input.command(&cx.doc.engine.document().content)?;
    cx.edit(command, json!({}))
}
fn voxels_erase_region(cx: &mut DocumentCx<'_>, a: &VoxelsEraseRegion) -> ToolResult {
    let command = a.input.command(&cx.doc.engine.document().content)?;
    cx.edit(command, json!({}))
}
fn voxels_fill(cx: &mut DocumentCx<'_>, a: &VoxelsFill) -> ToolResult {
    let command = a.input.command(&cx.doc.engine.document().content)?;
    cx.edit(command, json!({}))
}
fn voxels_box(cx: &mut DocumentCx<'_>, a: &VoxelsBox) -> ToolResult {
    let command = a.input.command(&cx.doc.engine.document().content)?;
    cx.edit(command, json!({}))
}
fn voxels_batch(cx: &mut DocumentCx<'_>, a: &VoxelsBatch) -> ToolResult {
    let operations = a.operations.get()?;
    let mut commands = Vec::new();
    let mut explicit_voxels = 0;
    for (index, operation) in operations.iter().enumerate() {
        explicit_voxels += operation.explicit_voxels()?;
        if explicit_voxels > 65536 {
            let mut error = Failure::new("limit_exceeded", "a request may contain at most 65,536 explicit voxel entries across all batch operations")
                .at(format!("/operations/{index}/voxels"));
            error.command_index = Some(index);
            return Err(error);
        }
        let command = operation
            .command(&cx.doc.engine.document().content)
            .map_err(|mut e| {
                e.command_index = Some(index);
                e.path = Some(format!("/operations/{index}{}", e.path.unwrap_or_default()));
                e
            })?;
        commands.push(command);
    }
    cx.edit(Command::Batch(commands), json!({}))
}
fn layers_list(cx: &mut DocumentCx<'_>, _: &LayersList) -> ToolResult {
    read(&cx.doc.engine, json!({"layers":layers(&cx.doc.engine)}))
}
fn layers_add(cx: &mut DocumentCx<'_>, a: &LayersAdd) -> ToolResult {
    let id = a.layer_id.default(Uuid::new_v4())?;
    let command = Command::Layer(LayerEdit::Create {
        id,
        name: a.name.get()?,
        index: a
            .index
            .default(cx.doc.engine.document().content.layers.len())?,
    });
    cx.edit(command, json!({"layer_id":id}))
}
fn layers_remove(cx: &mut DocumentCx<'_>, a: &LayersRemove) -> ToolResult {
    let id = edit::layer(&cx.doc.engine.document().content, &a.layer.get()?)?;
    cx.edit(Command::Layer(LayerEdit::Delete(id)), json!({}))
}
fn layers_reorder(cx: &mut DocumentCx<'_>, a: &LayersReorder) -> ToolResult {
    let order = a
        .order
        .get()?
        .iter()
        .map(|l| edit::layer(&cx.doc.engine.document().content, l))
        .collect::<Result<_>>()?;
    cx.edit(Command::Layer(LayerEdit::Reorder(order)), json!({}))
}
fn layers_rename(cx: &mut DocumentCx<'_>, a: &LayersRename) -> ToolResult {
    let id = edit::layer(&cx.doc.engine.document().content, &a.layer.get()?)?;
    cx.edit(
        Command::Layer(LayerEdit::Update {
            id,
            name: a.name.optional()?,
            visible: None,
            locked: None,
        }),
        json!({}),
    )
}
fn layers_visibility(cx: &mut DocumentCx<'_>, a: &LayersVisibility) -> ToolResult {
    let id = edit::layer(&cx.doc.engine.document().content, &a.layer.get()?)?;
    cx.edit(
        Command::Layer(LayerEdit::Update {
            id,
            name: None,
            visible: a.visible.optional()?,
            locked: None,
        }),
        json!({}),
    )
}
fn layers_lock(cx: &mut DocumentCx<'_>, a: &LayersLock) -> ToolResult {
    let id = edit::layer(&cx.doc.engine.document().content, &a.layer.get()?)?;
    cx.edit(
        Command::Layer(LayerEdit::Update {
            id,
            name: None,
            visible: None,
            locked: a.locked.optional()?,
        }),
        json!({}),
    )
}
fn palette_list(cx: &mut DocumentCx<'_>, _: &PaletteList) -> ToolResult {
    read(
        &cx.doc.engine,
        json!({"palette":cx.doc.engine.document().content.palette,"presets":palettes::presets(cx.host)?}),
    )
}
fn palette_set_slot(cx: &mut DocumentCx<'_>, a: &PaletteSetSlot) -> ToolResult {
    let action = PaletteAction::SetSlot {
        slot: a.slot.get()?,
        color: Swatch {
            rgba: a.rgba.get()?,
            name: a.name.optional()?,
        },
    };
    let outcome = palettes::dispatch(cx.host, &cx.doc.engine.document().content.palette, action)?;
    let mut commands = outcome.commands;
    cx.edit(commands.remove(0), json!({}))
}
fn append_palette(cx: &mut DocumentCx<'_>, action: PaletteAction) -> ToolResult {
    let outcome = palettes::dispatch(cx.host, &cx.doc.engine.document().content.palette, action)?;
    let extras = json!({"slot_mapping":outcome.slot_mapping.iter().map(|(source, slot)|json!({"source":source,"slot":slot})).collect::<Vec<_>>()});
    cx.edit(Command::Batch(outcome.commands), extras)
}
fn palette_apply_preset(cx: &mut DocumentCx<'_>, a: &PaletteApplyPreset) -> ToolResult {
    append_palette(cx, PaletteAction::AppendPreset(a.preset.get()?))
}
fn palette_import(cx: &mut DocumentCx<'_>, a: &PaletteImport) -> ToolResult {
    append_palette(
        cx,
        PaletteAction::AppendImport {
            format: a.format.get()?,
            text: a.text.get()?,
        },
    )
}
fn palette_export(cx: &mut DocumentCx<'_>, a: &PaletteExport) -> ToolResult {
    let format = a.format.get()?;
    let bytes = exports::snapshot(cx.host, &format, &cx.doc.engine.document().content)?;
    let text =
        String::from_utf8(bytes).map_err(|e| Failure::new("export_failed", e.to_string()))?;
    read(&cx.doc.engine, json!({"format":format,"text":text}))
}
fn comments_list(cx: &mut DocumentCx<'_>, a: &CommentsList) -> ToolResult {
    let data = comments::list(
        cx.host,
        &cx.doc.engine.document().content,
        CommentsQuery {
            state: a.state.default("all".into())?,
            offset: a.offset.default(0)?,
            limit: a.limit.default(50)?,
            message_offset: a.message_offset.default(0)?,
            message_limit: a.message_limit.default(50)?,
        },
    )?;
    read(&cx.doc.engine, data)
}
fn comment_text(text: String) -> Result<String> {
    if text.len() > 4096 {
        return Err(Failure::invalid("comment text exceeds 4096 UTF-8 bytes").at("/text"));
    }
    Ok(text)
}
fn comments_add(cx: &mut DocumentCx<'_>, a: &CommentsAdd) -> ToolResult {
    let text = comment_text(a.text.get()?)?;
    let id = a.thread_id.default(Uuid::new_v4())?;
    let message = a.message_id.default(Uuid::new_v4())?;
    let s = &cx.doc.engine.document().content;
    let command = comments::command(
        cx.host,
        s,
        CommentsAction::Create {
            id,
            message,
            anchor: a.anchor.get()?,
            layer: CommentLayer::Explicit(
                a.layer
                    .optional()?
                    .map(|l| edit::layer(s, &l))
                    .transpose()?,
            ),
            text,
        },
    )?;
    cx.edit(command, json!({"thread_id":id,"message_id":message}))
}
fn comments_reply(cx: &mut DocumentCx<'_>, a: &CommentsReply) -> ToolResult {
    let text = comment_text(a.text.get()?)?;
    let message = a.message_id.default(Uuid::new_v4())?;
    let id = a.thread_id.get()?;
    let command = comments::command(
        cx.host,
        &cx.doc.engine.document().content,
        CommentsAction::Reply { id, message, text },
    )?;
    cx.edit(command, json!({"thread_id":id,"message_id":message}))
}
fn resolve_comment(cx: &mut DocumentCx<'_>, id: Uuid, resolved: bool) -> ToolResult {
    let command = comments::command(
        cx.host,
        &cx.doc.engine.document().content,
        CommentsAction::Resolve { id, resolved },
    )?;
    cx.edit(command, json!({}))
}
fn comments_resolve(cx: &mut DocumentCx<'_>, a: &CommentsResolve) -> ToolResult {
    resolve_comment(cx, a.thread_id.get()?, true)
}
fn comments_reopen(cx: &mut DocumentCx<'_>, a: &CommentsReopen) -> ToolResult {
    resolve_comment(cx, a.thread_id.get()?, false)
}
fn history_read(cx: &mut DocumentCx<'_>, a: &HistoryRead) -> ToolResult {
    let engine = &cx.doc.engine;
    let entries: Vec<_> = engine.history().collect();
    let offset = a.offset.default(0)?;
    let limit = a.limit.default(50)?;
    let page: Vec<_> = entries.iter().skip(offset).take(limit).map(|e|json!({"entry_id":e.id,"origin":e.actor.origin,"tool":e.actor.tool,"label":e.label,"base_revision":e.base_revision.to_string(),"result_revision":e.result_revision.to_string(),"added":e.changes.added,"removed":e.changes.removed,"recolored":e.changes.recolored,"changed_bounds":e.changes.bounds})).collect();
    let next = (offset + page.len() < entries.len()).then_some(offset + page.len());
    read(
        engine,
        json!({"entries":page,"total":entries.len(),"next_offset":next,"head_id":engine.head(),"redo_id":engine.redo_head(),"initial_retained":engine.initial_retained(),"history_bytes":engine.history_bytes()}),
    )
}
fn history_move(cx: &mut DocumentCx<'_>, receipt: Receipt) -> ToolResult {
    let mut data = change(&receipt);
    data["operation_id"] = json!(cx.write.identity.operation_id.get()?);
    data["head_id"] = json!(cx.doc.engine.head());
    data["redo_id"] = json!(cx.doc.engine.redo_head());
    read(&cx.doc.engine, data)
}
fn history_undo(cx: &mut DocumentCx<'_>, a: &HistoryUndo) -> ToolResult {
    let receipt = cx
        .doc
        .engine
        .undo(cx.doc.engine.revision(), a.entry_id.get()?)?;
    history_move(cx, receipt)
}
fn history_redo(cx: &mut DocumentCx<'_>, a: &HistoryRedo) -> ToolResult {
    let receipt = cx
        .doc
        .engine
        .redo(cx.doc.engine.revision(), a.entry_id.get()?)?;
    history_move(cx, receipt)
}
fn versions_list(cx: &mut DocumentCx<'_>, _: &VersionsList) -> ToolResult {
    read(
        &cx.doc.engine,
        json!({"versions":cx.doc.engine.versions().map(|v|json!({"id":v.id,"name":v.name,"parent_id":v.parent,"origin":v.origin,"content_hash":v.content_hash})).collect::<Vec<_>>()}),
    )
}
fn versions_save(cx: &mut DocumentCx<'_>, a: &VersionsSave) -> ToolResult {
    let id = a.version_id.default(Uuid::new_v4())?;
    cx.edit(
        Command::SaveVersion {
            id,
            name: a.name.get()?,
            parent: a.parent_id.optional()?,
        },
        json!({"version_id":id}),
    )
}
fn versions_restore(cx: &mut DocumentCx<'_>, a: &VersionsRestore) -> ToolResult {
    cx.edit(
        Command::RestoreVersion {
            id: a.version_id.get()?,
            expected_head: a.expected_head_id.get()?,
        },
        json!({}),
    )
}
fn render(cx: &mut DocumentCx<'_>, a: &Render) -> ToolResult {
    let engine = &cx.doc.engine;
    let images = images::render(
        &engine.document().content,
        &a.render,
        &cx.host.render_plan(),
    )?;
    let mut data = images.metadata;
    data["model_id"] = json!(engine.document().id);
    data["revision"] = json!(engine.revision().to_string());
    Ok((
        data,
        images
            .pngs
            .iter()
            .map(|png| ContentBlock::image(STANDARD.encode(png), "image/png"))
            .collect(),
    ))
}
fn export_png(cx: &mut DocumentCx<'_>, a: &ExportPng) -> ToolResult {
    if a.render.image_mode.default("contact_sheet".into())? == "both" {
        return Err(Failure::invalid(
            "export_png needs one PNG; use contact_sheet or one individual view",
        ));
    }
    let images = images::export(&cx.doc.engine.document().content, &a.render, cx.host)?;
    if images.pngs.len() != 1 {
        return Err(Failure::invalid(
            "export_png needs one PNG; use contact_sheet or one individual view",
        ));
    }
    let path = files::path(cx.root, &a.path.get()?)?;
    let bytes = &images.pngs[0];
    files::write(&path, bytes)?;
    let mut data = images.metadata;
    data["path"] = json!(a.path.get()?);
    data["sha256"] = json!(files::hash(bytes));
    data["bytes"] = json!(bytes.len());
    read(&cx.doc.engine, data)
}
fn export_glb(cx: &mut DocumentCx<'_>, a: &ExportGlb) -> ToolResult {
    let path = files::path(cx.root, &a.path.get()?)?;
    let bytes = exports::snapshot(cx.host, "glb", &cx.doc.engine.document().content)?;
    files::write(&path, &bytes)?;
    read(
        &cx.doc.engine,
        json!({"path":a.path.get()?,"sha256":files::hash(&bytes),"bytes":bytes.len(),"mime_type":"model/gltf-binary"}),
    )
}
