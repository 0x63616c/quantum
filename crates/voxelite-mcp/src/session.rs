//! Serialized document authority and bounded operation receipts.
use crate::{
    API_VERSION,
    catalog::Spec,
    comments, edit,
    error::{Args, Failure, Result},
    exports, files, images, palettes,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use rmcp::model::{CallToolResult, ContentBlock};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};
use voxelite_core::*;
use voxelite_format::{
    Project,
    session::{DocumentSession, SavePolicy},
};
use voxelite_plugin_api::{CommentLayer, CommentsAction, CommentsQuery};

const MAX_RECEIPTS: usize = 4096;
const MAX_RECEIPT_BYTES: usize = 8 * 1024 * 1024;
const RECEIPT_RESERVATION: usize = 65536;
const MAX_RESULT_BYTES: usize = 12 * 1024 * 1024;
const MAX_EXPLICIT_VOXELS: usize = 65536;

/// Immutable render work captured on the authority thread and executed off-thread.
pub(crate) struct RenderTask {
    id: Uuid,
    epoch: Uuid,
    model: Uuid,
    revision: u64,
    pub snapshot: std::sync::Arc<Snapshot>,
    args: Value,
    passes: voxelite_render::RenderPlan,
}
impl RenderTask {
    pub fn run(self) -> CallToolResult {
        let result = (|| -> Result<CallToolResult> {
            let images = images::render(&self.snapshot, &Args(&self.args), &self.passes)?;
            let mut data = images.metadata;
            data["model_id"] = json!(self.model);
            data["revision"] = json!(self.revision.to_string());
            let envelope = json!({"api_version":API_VERSION,"session_id":self.id,"epoch":self.epoch,"ok":true,"data":data});
            let mut content = vec![ContentBlock::text(envelope.to_string())];
            content.extend(
                images
                    .pngs
                    .iter()
                    .map(|png| ContentBlock::image(STANDARD.encode(png), "image/png")),
            );
            let mut result = CallToolResult::success(content);
            result.structured_content = Some(envelope);
            if serde_json::to_vec(&result).expect("tool result").len() > MAX_RESULT_BYTES {
                return Err(Failure::new(
                    "limit_exceeded",
                    "encoded tool result exceeds 12 MiB; reduce render dimensions",
                ));
            }
            Ok(result)
        })();
        result.unwrap_or_else(|e| Session::error_for(self.id, self.epoch, e))
    }
}

pub(crate) struct Session {
    pub id: Uuid,
    pub epoch: Uuid,
    root: PathBuf,
    pub(crate) documents: BTreeMap<Uuid, DocumentSession>,
    receipts: BTreeMap<Uuid, (String, CallToolResult)>,
    receipt_bytes: usize,
}
impl Session {
    /// Create a session with an existing, canonical project directory.
    pub fn new(root: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            epoch: Uuid::new_v4(),
            root,
            documents: BTreeMap::new(),
            receipts: BTreeMap::new(),
            receipt_bytes: 0,
        }
    }
    pub(crate) fn attach(&mut self, project: Project) -> crate::error::Result<()> {
        // Attachment swaps in the editor's live document before serving requests.
        // Its idle shadow needs content only, not a second copy of build history.
        let id = project.document.id;
        let document = DocumentSession::placeholder(project)?;
        self.documents.clear();
        self.receipts.clear();
        self.receipt_bytes = 0;
        self.epoch = Uuid::new_v4();
        self.documents.insert(id, document);
        Ok(())
    }
    /// Validate and pin only read-only rendering; no history or authority is cloned.
    pub(crate) fn prepare_render(
        &self,
        args: Value,
        passes: voxelite_render::RenderPlan,
    ) -> Result<RenderTask> {
        let spec = crate::catalog::CATALOG
            .iter()
            .find(|s| s.tool.name == "voxelite_render")
            .expect("render catalog");
        if let Err(e) = spec.validator.validate(&args) {
            return Err(Failure::invalid(format!(
                "arguments do not match schema: {:?}",
                e.kind()
            )));
        }
        let a = Args(&args);
        let model = a.get("model_id")?;
        let doc = self
            .documents
            .get(&model)
            .ok_or_else(|| Failure::new("model_not_found", "model is not open in this session"))?;
        let revision = doc.engine.revision();
        if let Some(expected) = a.optional::<String>("revision")? {
            let expected = expected
                .parse::<u64>()
                .map_err(|_| Failure::invalid("revision exceeds u64"))?;
            if expected != revision {
                let mut e = Failure::new(
                    "revision_conflict",
                    format!("expected revision {expected}, current {revision}"),
                );
                e.current_revision = Some(revision.to_string());
                return Err(e);
            }
        }
        Ok(RenderTask {
            passes,
            id: self.id,
            epoch: self.epoch,
            model,
            revision,
            snapshot: std::sync::Arc::new(doc.engine.document().content.clone()),
            args,
        })
    }
    pub fn error(&self, e: Failure) -> CallToolResult {
        Self::error_for(self.id, self.epoch, e)
    }
    pub fn error_for(id: Uuid, epoch: Uuid, e: Failure) -> CallToolResult {
        let envelope =
            json!({"api_version":API_VERSION,"session_id":id,"epoch":epoch,"ok":false,"error":e});
        let mut r = CallToolResult::error(vec![ContentBlock::text(envelope.to_string())]);
        r.structured_content = Some(envelope);
        r
    }
    fn success(&self, data: Value, mut images: Vec<ContentBlock>) -> CallToolResult {
        let envelope = json!({"api_version":API_VERSION,"session_id":self.id,"epoch":self.epoch,"ok":true,"data":data});
        images.insert(0, ContentBlock::text(envelope.to_string()));
        let mut r = CallToolResult::success(images);
        r.structured_content = Some(envelope);
        r
    }
    pub fn call(&mut self, spec: &Spec, args: Value) -> CallToolResult {
        self.call_with_host(spec, args, &mut exports::host())
    }
    pub(crate) fn call_with_host(
        &mut self,
        spec: &Spec,
        args: Value,
        host: &mut voxelite_plugin_host::Host,
    ) -> CallToolResult {
        match self.execute(spec, &args, host) {
            Ok(r) => r,
            Err(e) => self.error(e),
        }
    }
    fn execute(
        &mut self,
        spec: &Spec,
        args: &Value,
        host: &mut voxelite_plugin_host::Host,
    ) -> Result<CallToolResult> {
        if let Err(e) = spec.validator.validate(args) {
            return Err(
                Failure::invalid(format!("arguments do not match schema: {:?}", e.kind()))
                    .at(e.instance_path().to_string()),
            );
        }
        let name = spec
            .tool
            .name
            .strip_prefix("voxelite_")
            .expect("catalog prefix");
        let a = Args(args);
        let mut identity = None;
        if spec.mutation {
            let id: Uuid = a.get("operation_id")?;
            let signature =
                files::hash(&serde_json::to_vec(&json!([name, args])).expect("JSON arguments"));
            if let Some((previous, result)) = self.receipts.get(&id) {
                return if previous == &signature {
                    Ok(result.clone())
                } else {
                    Err(Failure::new(
                        "operation_id_reused",
                        "operation_id was already used with different arguments",
                    ))
                };
            }
            let epoch: Uuid = a.get("expected_epoch")?;
            if epoch != self.epoch {
                return Err(Failure::new(
                    "epoch_conflict",
                    "session epoch changed; inspect capabilities and document state before retrying",
                ));
            }
            if let Some(label) = a.optional::<String>("label")?
                && (label.len() > 80 || validate_name(&label).is_err())
            {
                return Err(Failure::invalid(
                    "label must be 1..80 UTF-8 bytes without control characters",
                )
                .at("/label"));
            }
            self.reserve_receipt(name, &a)?;
            identity = Some((id, signature));
        }
        let (data, images) = match name {
            "capabilities_get" => (
                json!({"api_version":API_VERSION,"protocol_versions":["2025-11-25","2026-07-28"],"limits":{"open_documents":8,"request_bytes":16777216,"result_bytes":MAX_RESULT_BYTES,"batch_operations":256,"explicit_voxels":65536,"read_voxels":10000,"work_units":MAX_WORK,"operation_receipts":MAX_RECEIPTS,"receipt_bytes":MAX_RECEIPT_BYTES},"renderer":"cpu","formats":exports::extensions(host),"presets":["pico8","endesga32","sweetie16","grayscale","neutral","warm","vivid"],"durability":"memory","documents":self.documents.values().map(info).collect::<Vec<_>>()}),
                vec![],
            ),
            "document_create" | "document_open" => (self.open(name, &a)?, vec![]),
            _ => {
                let model_id: Uuid = a.get("model_id")?;
                let doc = self.documents.get_mut(&model_id).ok_or_else(|| {
                    Failure::new("model_not_found", "model is not open in this session")
                })?;
                let revision = doc.engine.revision();
                let expected: Option<String> = if spec.mutation {
                    Some(a.get("expected_revision")?)
                } else {
                    a.optional("revision")?
                };
                if let Some(expected) = expected {
                    let expected = expected
                        .parse::<u64>()
                        .map_err(|_| Failure::invalid("revision exceeds u64"))?;
                    if expected != revision {
                        let mut e = Failure::new(
                            "revision_conflict",
                            format!("expected revision {expected}, current {revision}"),
                        );
                        e.current_revision = Some(revision.to_string());
                        return Err(e);
                    }
                }
                dispatch(&self.root, self.id, doc, name, &a, host)?
            }
        };
        let result = self.success(data, images);
        let size = serde_json::to_vec(&result)
            .expect("serializable tool result")
            .len();
        if size > MAX_RESULT_BYTES {
            return Err(Failure::new(
                "limit_exceeded",
                "encoded tool result exceeds 12 MiB; reduce render dimensions or read page size",
            ));
        }
        if let Some((id, signature)) = identity {
            // Mutation results contain only bounded summaries, never image data.
            debug_assert!(size <= RECEIPT_RESERVATION);
            self.receipt_bytes += size;
            self.receipts.insert(id, (signature, result.clone()));
        }
        Ok(result)
    }
    fn reserve_receipt(&self, name: &str, a: &Args<'_>) -> Result<()> {
        // An edit must leave room to save every open document, including a
        // document about to be created. A save may consume its own reservation,
        // but cannot spend the capacity needed by other dirty documents. This
        // also prevents repeated saves of a clean document starving dirty ones.
        let saves = if name == "document_save" {
            let target: Uuid = a.get("model_id")?;
            self.documents
                .iter()
                .filter(|(id, doc)| **id != target && doc.needs_save())
                .count()
        } else {
            self.documents.len() + usize::from(matches!(name, "document_create" | "document_open"))
        };
        if self.receipts.len() + 1 + saves > MAX_RECEIPTS
            || self.receipt_bytes + (1 + saves) * RECEIPT_RESERVATION > MAX_RECEIPT_BYTES
        {
            let mut error = Failure::new(
                "limit_exceeded",
                "session receipt budget exhausted; remaining capacity is reserved for saving dirty documents",
            );
            error.recovery = "Save each dirty document, preserving operation_id on exact retries, then start a new session.";
            return Err(error);
        }
        Ok(())
    }
    fn open(&mut self, name: &str, a: &Args<'_>) -> Result<Value> {
        if self.documents.len() >= 8 {
            return Err(Failure::new(
                "limit_exceeded",
                "at most eight documents per session",
            ));
        }
        let limits = Limits {
            history_bytes: 16 * 1024 * 1024,
            ..Default::default()
        };
        let document = if name == "document_open" {
            let path = files::path(&self.root, &a.get::<String>("path")?)?;
            DocumentSession::open(&path, limits)?
        } else {
            let mut document = Document::new(
                a.default("model_id", Uuid::new_v4())?,
                a.default("layer_id", Uuid::new_v4())?,
            );
            // Construction precedes engine installation; subsequent content
            // edits can only pass through Transaction/Command.
            document.content.name = a.get("name")?;
            document.content.bounds = a.default("bounds", Region::default())?;
            DocumentSession::new(Project::new(document)?, None, limits)?
        };
        let id = document.engine.document().id;
        if self.documents.contains_key(&id) {
            return Err(Failure::new(
                "model_already_open",
                "document ID is already open; use its model_id",
            ));
        }
        let data = info(&document);
        self.documents.insert(id, document);
        Ok(data)
    }
}
fn layers(engine: &Engine) -> Vec<Value> {
    engine.document().content.layers.iter().map(|l|json!({"id":l.id,"name":l.name,"visible":l.visible,"locked":l.locked,"voxel_count":l.voxels.len()})).collect()
}
fn info(session: &DocumentSession) -> Value {
    let engine = &session.engine;
    let d = engine.document();
    json!({"model_id":d.id,"revision":engine.revision().to_string(),"name":d.content.name,"bounds":d.content.bounds,"dirty":session.needs_save(),"layers":layers(engine),"voxel_count":d.content.layers.iter().map(|l|l.voxels.len()).sum::<usize>(),"palette_slots":d.content.palette.slots.len(),"comments":d.content.comments.len(),"versions":d.versions.len(),"head_id":engine.head(),"redo_id":engine.redo_head()})
}
fn change(receipt: &Receipt) -> Value {
    json!({"revision":receipt.revision.to_string(),"undo_entry_id":receipt.entry,"changed_voxels":receipt.changes.voxel_count(),"added":receipt.changes.added,"removed":receipt.changes.removed,"recolored":receipt.changes.recolored,"changed_bounds":receipt.changes.bounds,"changed_layers":receipt.changes.layers,"palette_changed":receipt.changes.palette_changed,"metadata_changed":receipt.changes.metadata_changed,"work_units":receipt.work_units,"durability":"memory"})
}
fn dispatch(
    root: &std::path::Path,
    client_id: Uuid,
    doc: &mut DocumentSession,
    name: &str,
    a: &Args<'_>,
    host: &mut voxelite_plugin_host::Host,
) -> Result<(Value, Vec<ContentBlock>)> {
    if name == "document_info" {
        return Ok((info(doc), vec![]));
    }
    if name == "document_save" {
        let path = files::path(root, &a.get::<String>("path")?)?;
        let saved = doc
            .prepare_save(&path, SavePolicy::CreateOrTracked)
            .write()?;
        let data = json!({"model_id":doc.engine.document().id,"revision":doc.engine.revision().to_string(),"path":a.get::<String>("path")?,"sha256":saved.fingerprint(),"bytes":saved.bytes(),"dirty":false});
        doc.finish_save(saved);
        return Ok((data, vec![]));
    }
    let engine = &mut doc.engine;
    let s = &engine.document().content;
    if matches!(name, "comments_add" | "comments_reply") && a.get::<String>("text")?.len() > 4096 {
        return Err(Failure::invalid("comment text exceeds 4096 UTF-8 bytes").at("/text"));
    }
    let mut extras = json!({});
    let command = match name {
        "layers_list" => return read(engine, json!({"layers":layers(engine)})),
        "palette_list" => {
            return read(
                engine,
                json!({"palette":s.palette,"presets":palettes::presets(host)?}),
            );
        }
        "palette_export" => {
            let format: String = a.get("format")?;
            let bytes = exports::snapshot(host, &format, s)?;
            let text = String::from_utf8(bytes)
                .map_err(|e| Failure::new("export_failed", e.to_string()))?;
            return read(engine, json!({"format":format,"text":text}));
        }
        "voxels_get" => {
            let id = edit::layer(s, &a.get::<String>("layer")?)?;
            let l = s.layer(id)?;
            let positions: Vec<Pos> = a.get("positions")?;
            let mut voxels = Vec::new();
            for at in positions {
                if !s.bounds.contains(at) {
                    return Err(Error::OutOfBounds(at).into());
                }
                voxels.push(json!({"at":at,"slot":l.voxels.get(at)}));
            }
            return read(engine, json!({"layer_id":id,"voxels":voxels}));
        }
        "query" => return read(engine, query(s, a)?),
        "voxels_set" => edit::voxel_command(s, "set", a)?,
        "voxels_erase_region" => edit::voxel_command(s, "erase_region", a)?,
        "voxels_fill" => edit::voxel_command(s, "fill", a)?,
        "voxels_box" => edit::voxel_command(s, "box", a)?,
        "voxels_batch" => {
            let operations: Vec<Value> = a.get("operations")?;
            let mut commands = Vec::new();
            let mut explicit_voxels = 0;
            for (index, v) in operations.iter().enumerate() {
                explicit_voxels += v
                    .get("voxels")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len);
                if explicit_voxels > MAX_EXPLICIT_VOXELS {
                    let mut error = Failure::new("limit_exceeded", "a request may contain at most 65,536 explicit voxel entries across all batch operations")
                        .at(format!("/operations/{index}/voxels"));
                    error.command_index = Some(index);
                    return Err(error);
                }
                let parsed = edit::voxel_command(s, &Args(v).get::<String>("kind")?, &Args(v))
                    .map_err(|mut e| {
                        e.command_index = Some(index);
                        e.path = Some(format!("/operations/{index}{}", e.path.unwrap_or_default()));
                        e
                    })?;
                commands.push(parsed);
            }
            Command::Batch(commands)
        }
        "layers_add" => {
            let id = a.default("layer_id", Uuid::new_v4())?;
            extras = json!({"layer_id":id});
            Command::Layer(LayerEdit::Create {
                id,
                name: a.get("name")?,
                index: a.default("index", s.layers.len())?,
            })
        }
        "layers_remove" => Command::Layer(LayerEdit::Delete(edit::layer(
            s,
            &a.get::<String>("layer")?,
        )?)),
        "layers_reorder" => {
            let order: Vec<String> = a.get("order")?;
            Command::Layer(LayerEdit::Reorder(
                order
                    .iter()
                    .map(|l| edit::layer(s, l))
                    .collect::<Result<_>>()?,
            ))
        }
        "layers_rename" | "layers_visibility" | "layers_lock" => {
            Command::Layer(LayerEdit::Update {
                id: edit::layer(s, &a.get::<String>("layer")?)?,
                name: a.optional("name")?,
                visible: a.optional("visible")?,
                locked: a.optional("locked")?,
            })
        }
        "palette_set_slot" | "palette_apply_preset" | "palette_import" => {
            use voxelite_plugin_api::PaletteAction;
            let action = match name {
                "palette_set_slot" => PaletteAction::SetSlot {
                    slot: a.get("slot")?,
                    color: Swatch {
                        rgba: a.get("rgba")?,
                        name: a.optional("name")?,
                    },
                },
                "palette_apply_preset" => PaletteAction::AppendPreset(a.get("preset")?),
                _ => PaletteAction::AppendImport {
                    format: a.get("format")?,
                    text: a.get("text")?,
                },
            };
            let outcome = palettes::dispatch(host, &s.palette, action)?;
            if name != "palette_set_slot" {
                extras = json!({"slot_mapping": outcome.slot_mapping.iter().map(|(source, slot)| json!({"source":source,"slot":slot})).collect::<Vec<_>>()});
            }
            // Preserve the original command nesting and history serialization.
            let mut commands = outcome.commands;
            if name == "palette_set_slot" {
                commands.remove(0)
            } else {
                Command::Batch(commands)
            }
        }
        "comments_list" => {
            let data = comments::list(
                host,
                s,
                CommentsQuery {
                    state: a.default("state", "all".into())?,
                    offset: a.default("offset", 0)?,
                    limit: a.default("limit", 50)?,
                    message_offset: a.default("message_offset", 0)?,
                    message_limit: a.default("message_limit", 50)?,
                },
            )?;
            return read(engine, data);
        }
        "comments_add" => {
            let id = a.default("thread_id", Uuid::new_v4())?;
            let message = a.default("message_id", Uuid::new_v4())?;
            extras = json!({"thread_id":id,"message_id":message});
            comments::command(
                host,
                s,
                CommentsAction::Create {
                    id,
                    message,
                    anchor: a.get("anchor")?,
                    layer: CommentLayer::Explicit(
                        a.optional::<String>("layer")?
                            .map(|l| edit::layer(s, &l))
                            .transpose()?,
                    ),
                    text: a.get("text")?,
                },
            )?
        }
        "comments_reply" => {
            let message = a.default("message_id", Uuid::new_v4())?;
            extras = json!({"thread_id":a.get::<Uuid>("thread_id")?,"message_id":message});
            comments::command(
                host,
                s,
                CommentsAction::Reply {
                    id: a.get("thread_id")?,
                    message,
                    text: a.get("text")?,
                },
            )?
        }
        "comments_resolve" | "comments_reopen" => comments::command(
            host,
            s,
            CommentsAction::Resolve {
                id: a.get("thread_id")?,
                resolved: name == "comments_resolve",
            },
        )?,
        "history_read" => {
            let entries: Vec<_> = engine.history().collect();
            let offset: usize = a.default("offset", 0)?;
            let limit: usize = a.default("limit", 50)?;
            let page:Vec<_>=entries.iter().skip(offset).take(limit).map(|e|json!({"entry_id":e.id,"origin":e.actor.origin,"tool":e.actor.tool,"label":e.label,"base_revision":e.base_revision.to_string(),"result_revision":e.result_revision.to_string(),"added":e.changes.added,"removed":e.changes.removed,"recolored":e.changes.recolored,"changed_bounds":e.changes.bounds})).collect();
            let next = (offset + page.len() < entries.len()).then_some(offset + page.len());
            return read(
                engine,
                json!({"entries":page,"total":entries.len(),"next_offset":next,"head_id":engine.head(),"redo_id":engine.redo_head(),"initial_retained":engine.initial_retained(),"history_bytes":engine.history_bytes()}),
            );
        }
        "history_undo" | "history_redo" => {
            let receipt = if name == "history_undo" {
                engine.undo(engine.revision(), a.get("entry_id")?)?
            } else {
                engine.redo(engine.revision(), a.get("entry_id")?)?
            };
            let mut data = change(&receipt);
            data["operation_id"] = a.0["operation_id"].clone();
            data["head_id"] = json!(engine.head());
            data["redo_id"] = json!(engine.redo_head());
            return read(engine, data);
        }
        "versions_list" => {
            return read(
                engine,
                json!({"versions":engine.versions().map(|v|json!({"id":v.id,"name":v.name,"parent_id":v.parent,"origin":v.origin,"content_hash":v.content_hash})).collect::<Vec<_>>()}),
            );
        }
        "versions_save" => {
            let id = a.default("version_id", Uuid::new_v4())?;
            extras = json!({"version_id":id});
            Command::SaveVersion {
                id,
                name: a.get("name")?,
                parent: a.optional("parent_id")?,
            }
        }
        "versions_restore" => Command::RestoreVersion {
            id: a.get("version_id")?,
            expected_head: a.get("expected_head_id")?,
        },
        "render" | "export_png" => {
            // Export can only emit a single image; validate before expensive work.
            if name == "export_png"
                && a.default::<String>("image_mode", "contact_sheet".into())? == "both"
            {
                return Err(Failure::invalid(
                    "export_png needs one PNG; use contact_sheet or one individual view",
                ));
            }
            let images = if name == "export_png" {
                images::export(s, a, host)?
            } else {
                images::render(s, a, &host.render_plan())?
            };
            if name == "export_png" {
                if images.pngs.len() != 1 {
                    return Err(Failure::invalid(
                        "export_png needs one PNG; use contact_sheet or one individual view",
                    ));
                }
                let path = files::path(root, &a.get::<String>("path")?)?;
                let bytes = &images.pngs[0];
                files::write(&path, bytes)?;
                let mut data = images.metadata;
                data["path"] = a.0["path"].clone();
                data["sha256"] = json!(files::hash(bytes));
                data["bytes"] = json!(bytes.len());
                return read(engine, data);
            }
            let mut data = images.metadata;
            data["model_id"] = json!(engine.document().id);
            data["revision"] = json!(engine.revision().to_string());
            return Ok((
                data,
                images
                    .pngs
                    .iter()
                    .map(|png| ContentBlock::image(STANDARD.encode(png), "image/png"))
                    .collect(),
            ));
        }
        "export_glb" => {
            let path = files::path(root, &a.get::<String>("path")?)?;
            let bytes = exports::snapshot(host, "glb", s)?;
            files::write(&path, &bytes)?;
            return read(
                engine,
                json!({"path":a.0["path"],"sha256":files::hash(&bytes),"bytes":bytes.len(),"mime_type":"model/gltf-binary"}),
            );
        }
        _ => return Err(Failure::invalid("unimplemented catalog entry")),
    };
    let operation_id = a.get("operation_id")?;
    let receipt = engine.execute(Transaction {
        expected_revision: engine.revision(),
        id: operation_id,
        actor: Actor {
            origin: Origin::Agent,
            client_id,
            tool: format!("voxelite_{name}"),
        },
        label: a.default("label", name.replace('_', " "))?,
        command,
    })?;
    let mut data = change(&receipt);
    data.as_object_mut()
        .unwrap()
        .extend(extras.as_object().unwrap().clone());
    data["operation_id"] = json!(operation_id);
    data["head_id"] = json!(engine.head());
    read(engine, data)
}
fn read(engine: &Engine, mut data: Value) -> Result<(Value, Vec<ContentBlock>)> {
    data["model_id"] = json!(engine.document().id);
    data["revision"] = json!(engine.revision().to_string());
    Ok((data, vec![]))
}
fn query(s: &Snapshot, a: &Args<'_>) -> Result<Value> {
    let region = a.default("region", s.bounds)?;
    region.validate()?;
    if !s.bounds.contains_region(region) {
        return Err(Failure::new(
            "out_of_bounds",
            "query region exceeds document bounds",
        ));
    }
    let view: String = a.default("view", "layers".into())?;
    let target = a
        .optional::<String>("layer")?
        .map(|l| edit::layer(s, &l))
        .transpose()?;
    if view == "composite" && target.is_some() {
        return Err(Failure::invalid(
            "layer and composite view are mutually exclusive",
        ));
    }
    let mut cells = Vec::new();
    if view == "composite" {
        for (at, slot) in s.composite() {
            if region.contains(at) {
                cells.push((None, at, slot));
            }
        }
    } else {
        for l in &s.layers {
            if target.is_none_or(|t| t == l.id) {
                for (at, slot) in l.voxels.iter() {
                    if region.contains(at) {
                        cells.push((Some(l.id), at, slot));
                    }
                }
            }
        }
    }
    cells.sort_unstable_by_key(|(l, p, _)| (*l, *p));
    let mut bounds: Option<Region> = None;
    let mut slots: BTreeMap<u16, usize> = BTreeMap::new();
    for (_, at, slot) in &cells {
        *slots.entry(*slot).or_default() += 1;
        if let Some(b) = &mut bounds {
            for (i, c) in at.iter().enumerate() {
                b.min[i] = b.min[i].min(*c);
                b.max[i] = b.max[i].max(*c + 1);
            }
        } else {
            bounds = Some(Region {
                min: *at,
                max: at.map(|c| c + 1),
            });
        }
    }
    let kind: String = a.get("kind")?;
    let mut data = json!({"bounds":s.bounds,"region":region,"occupied_bounds":bounds,"voxel_count":cells.len(),"view":view});
    if kind == "counts" {
        data["slots"] = json!(slots);
    }
    if kind == "voxels" {
        let offset: usize = a.default("offset", 0)?;
        let limit: usize = a.default("limit", 1000)?;
        let page: Vec<_> = cells
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(l, at, slot)| json!({"layer_id":l,"at":at,"slot":slot}))
            .collect();
        data["next_offset"] =
            json!((offset + page.len() < cells.len()).then_some(offset + page.len()));
        data["voxels"] = json!(page);
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(session: &mut Session, name: &str, args: Value) -> Value {
        let spec = crate::catalog::CATALOG
            .iter()
            .find(|s| s.tool.name == name)
            .unwrap();
        session.call(spec, args).structured_content.unwrap()
    }

    #[test]
    fn exhausted_receipts_preserve_saves_for_every_dirty_document() {
        for byte_limit in [false, true] {
            let root = tempfile::tempdir().unwrap();
            // Match the public entry points: temporary paths may contain symlinks.
            let mut session = Session::new(root.path().canonicalize().unwrap());
            let epoch = session.epoch;
            let mut models = Vec::new();
            for _ in 0..2 {
                let result = call(
                    &mut session,
                    "voxelite_document_create",
                    json!({"name":"Unsaved","expected_epoch":epoch,"operation_id":Uuid::new_v4()}),
                );
                assert_eq!(result["ok"], true);
                models.push(result["data"]["model_id"].clone());
            }
            if byte_limit {
                // Inject a boundary state without allocating 8 MiB of receipts.
                session.receipt_bytes = MAX_RECEIPT_BYTES - 2 * RECEIPT_RESERVATION;
            } else {
                let receipt = session.success(json!({}), vec![]);
                while session.receipts.len() < MAX_RECEIPTS - 2 {
                    session
                        .receipts
                        .insert(Uuid::new_v4(), (String::new(), receipt.clone()));
                }
            }
            let mut args = json!({"model_id":models[0],"expected_epoch":epoch,"expected_revision":"0","operation_id":Uuid::new_v4(),"layer":"Base","voxels":[{"at":[0,0,0],"slot":1}]});
            let result = call(&mut session, "voxelite_voxels_set", args.clone());
            assert_eq!(result["error"]["code"], "limit_exceeded");
            args.as_object_mut().unwrap().remove("layer");
            args.as_object_mut().unwrap().remove("voxels");
            args["path"] = json!("../invalid.voxl");
            let count = session.receipts.len();
            assert_eq!(
                call(&mut session, "voxelite_document_save", args.clone())["ok"],
                false
            );
            assert_eq!(session.receipts.len(), count);
            args["path"] = json!("first.voxl");
            let saved = call(&mut session, "voxelite_document_save", args.clone());
            assert_eq!(saved["ok"], true, "{saved}");
            let mut redundant = args.clone();
            redundant["operation_id"] = json!(Uuid::new_v4());
            assert_eq!(
                call(&mut session, "voxelite_document_save", redundant)["error"]["code"],
                "limit_exceeded"
            );
            let mut second = args.clone();
            second["model_id"] = models[1].clone();
            second["path"] = json!("second.voxl");
            second["operation_id"] = json!(Uuid::new_v4());
            assert_eq!(
                call(&mut session, "voxelite_document_save", second)["ok"],
                true
            );
            assert_eq!(call(&mut session, "voxelite_document_save", args), saved);
            assert!(session.documents.values().all(|d| !d.needs_save()));
            assert!(session.receipts.len() <= MAX_RECEIPTS);
            assert!(session.receipt_bytes <= MAX_RECEIPT_BYTES);
            for path in ["first.voxl", "second.voxl"] {
                let project =
                    voxelite_format::from_bytes(&std::fs::read(root.path().join(path)).unwrap())
                        .unwrap();
                assert_eq!(project.document.content.name, "Unsaved");
            }
        }
    }
}

#[cfg(test)]
mod render_worker_tests {
    use super::*;
    #[test]
    fn render_task_pins_revision_and_does_not_borrow_live_authority() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(dir.path().canonicalize().unwrap());
        let doc = Document::new(Uuid::new_v4(), Uuid::new_v4());
        let model = doc.id;
        session.attach(Project::new(doc).unwrap()).unwrap();
        let task = session
            .prepare_render(
                json!({"model_id":model,"revision":"0","views":["front"],"width":32,"height":32}),
                exports::host().render_plan(),
            )
            .unwrap();
        let engine = &mut session.documents.get_mut(&model).unwrap().engine;
        engine
            .execute(Transaction {
                expected_revision: 0,
                id: Uuid::new_v4(),
                actor: Actor::new(Origin::Human),
                label: "While rendering".into(),
                command: Command::EditDocument {
                    name: Some("Still responsive".into()),
                    bounds: None,
                },
            })
            .unwrap();
        let result = std::thread::spawn(move || task.run()).join().unwrap();
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.structured_content.unwrap()["data"]["revision"], "0");
        assert_eq!(session.documents[&model].engine.revision(), 1);
        assert!(
            session
                .prepare_render(
                    json!({"model_id":model,"revision":"0"}),
                    exports::host().render_plan()
                )
                .is_err()
        );
    }
}
