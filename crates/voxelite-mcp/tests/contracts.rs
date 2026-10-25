//! Adapter contracts: actual core behavior, strict schemas, and persistence.
use serde_json::{Value, json};
use voxelite_core::Uuid;
use voxelite_mcp::VoxeliteServer;

struct Fixture {
    server: VoxeliteServer,
    root: tempfile::TempDir,
    epoch: Value,
    model: Value,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let server = VoxeliteServer::new(root.path()).unwrap();
        let epoch = server
            .call("voxelite_capabilities_get", json!({}))
            .unwrap()
            .structured_content
            .unwrap()["epoch"]
            .clone();
        let created = server
            .call(
                "voxelite_document_create",
                json!({"name":"Test","operation_id":Uuid::new_v4(),"expected_epoch":epoch}),
            )
            .unwrap()
            .structured_content
            .unwrap();
        assert_eq!(created["ok"], true, "{created}");
        let model = created["data"]["model_id"].clone();
        Self {
            server,
            root,
            epoch,
            model,
        }
    }
    fn call(&self, name: &str, args: Value) -> Value {
        let tool = format!("voxelite_{name}");
        let result = self.server.call(&tool, args).unwrap();
        let value = result.structured_content.unwrap();
        let schema = VoxeliteServer::tools()
            .into_iter()
            .find(|t| t.name == tool)
            .unwrap()
            .output_schema
            .unwrap();
        let validator = jsonschema::validator_for(&Value::Object((*schema).clone())).unwrap();
        assert!(validator.is_valid(&value), "output schema: {value}");
        assert_eq!(result.is_error, Some(value["ok"] == false));
        value
    }
    fn read(&self, name: &str, mut args: Value) -> Value {
        args["model_id"] = self.model.clone();
        let r = self.call(name, args);
        assert_eq!(r["ok"], true, "{r}");
        r["data"].clone()
    }
    fn args(&self, mut args: Value) -> Value {
        let info = self.read("document_info", json!({}));
        args["model_id"] = self.model.clone();
        args["expected_epoch"] = self.epoch.clone();
        args["expected_revision"] = info["revision"].clone();
        args["operation_id"] = json!(Uuid::new_v4());
        args
    }
    fn edit(&self, name: &str, args: Value) -> Value {
        let r = self.call(name, self.args(args));
        assert_eq!(r["ok"], true, "{r}");
        r["data"].clone()
    }
    fn fail(&self, name: &str, args: Value, code: &str) -> Value {
        let r = self.call(name, self.args(args));
        assert_eq!(r["ok"], false, "{r}");
        assert_eq!(r["error"]["code"], code, "{r}");
        r
    }
}
#[test]
fn catalog_is_frozen_sorted_and_strict() {
    let tools = VoxeliteServer::tools();
    assert_eq!(tools.len(), 38);
    assert!(tools.windows(2).all(|w| w[0].name < w[1].name));
    let frozen: Value =
        serde_json::from_str(include_str!("../../../spec/mcp/v1/tools.json")).unwrap();
    assert_eq!(serde_json::to_value(&tools).unwrap(), frozen);
    for t in tools {
        assert_eq!(t.input_schema["additionalProperties"], false);
        assert!(t.description.unwrap().len() > 60);
    }
}
#[test]
fn atomic_batches_mirror_noop_and_operation_receipts() {
    let f = Fixture::new();
    let failed=f.fail("voxels_batch",json!({"operations":[{"kind":"set","layer":"Base","voxels":[{"at":[1,1,1],"slot":1}]},{"kind":"box","layer":"Base","region":{"min":[31,0,0],"max":[33,1,1]},"slot":1}]}),"out_of_bounds");
    assert_eq!(failed["error"]["command_index"], 1);
    assert_eq!(f.read("document_info", json!({}))["voxel_count"], 0);
    assert_eq!(f.read("history_read", json!({}))["total"], 0);
    let args=f.args(json!({"layer":"Base","voxels":[{"at":[1,1,1],"slot":1}],"mirror":{"axes":["x"],"plane2":[32,0,0]}}));
    let result = f.call("voxels_set", args.clone());
    assert_eq!(result["data"]["added"], 2);
    assert_eq!(f.call("voxels_set", args.clone()), result);
    let mut reused = args.clone();
    reused["voxels"][0]["slot"] = json!(0);
    assert_eq!(
        f.call("voxels_set", reused)["error"]["code"],
        "operation_id_reused"
    );
    let mut stale = args;
    stale["operation_id"] = json!(Uuid::new_v4());
    assert_eq!(
        f.call("voxels_set", stale)["error"]["code"],
        "revision_conflict"
    );
    let noop = f.edit(
        "voxels_set",
        json!({"layer":"Base","voxels":[{"at":[1,1,1],"slot":1}]}),
    );
    assert_eq!(noop["revision"], "1");
    assert!(noop["undo_entry_id"].is_null());
    let no_mirror = f.edit(
        "voxels_set",
        json!({"layer":"Base","voxels":[{"at":[2,1,1],"slot":1}]}),
    );
    assert_eq!(no_mirror["added"], 1);
    let history = f.read("history_read", json!({}));
    assert_eq!(history["total"], 2);
    assert_eq!(history["entries"][0]["origin"], "agent");
    f.edit("history_undo", json!({"entry_id":history["head_id"]}));
    assert_eq!(f.read("document_info", json!({}))["voxel_count"], 2);
    f.edit("history_redo", json!({"entry_id":history["head_id"]}));
    assert_eq!(f.read("document_info", json!({}))["voxel_count"], 3);
}
#[test]
fn strict_arguments_epoch_and_locked_hidden_layers() {
    let f = Fixture::new();
    let mut bad = f.args(json!({"layer":"Base","voxels":[{"at":[0,0,0],"slot":1,"surprise":1}]}));
    assert_eq!(
        f.call("voxels_set", bad.clone())["error"]["code"],
        "invalid_argument"
    );
    bad["voxels"][0].as_object_mut().unwrap().remove("surprise");
    bad["expected_epoch"] = json!(Uuid::new_v4());
    assert_eq!(f.call("voxels_set", bad)["error"]["code"], "epoch_conflict");
    f.edit("layers_lock", json!({"layer":"Base","locked":true}));
    f.fail(
        "voxels_set",
        json!({"layer":"Base","voxels":[{"at":[0,0,0],"slot":1}]}),
        "layer_blocked",
    );
    f.edit("layers_lock", json!({"layer":"Base","locked":false}));
    f.edit("layers_visibility", json!({"layer":"Base","visible":false}));
    f.fail(
        "voxels_box",
        json!({"layer":"Base","region":{"min":[0,0,0],"max":[1,1,1]},"slot":1}),
        "layer_blocked",
    );
    f.edit("layers_visibility", json!({"layer":"Base","visible":true}));
    let id = f.edit("layers_add", json!({"name":"Detail"}))["layer_id"].clone();
    f.edit("layers_rename", json!({"layer":id,"name":"Base"}));
    f.fail(
        "layers_lock",
        json!({"layer":"Base","locked":true}),
        "ambiguous_name",
    );
    let base = f.read("layers_list", json!({}))["layers"][0]["id"].clone();
    f.edit("layers_reorder", json!({"order":[id,base]}));
    f.edit("layers_remove", json!({"layer":id}));
    f.fail("layers_remove", json!({"layer":base}), "invalid_argument");
}
#[test]
fn fills_queries_palette_and_bounded_work() {
    let f = Fixture::new();
    f.edit(
        "palette_set_slot",
        json!({"slot":2,"rgba":[255,0,0,255],"name":"red"}),
    );
    f.edit(
        "voxels_box",
        json!({"layer":"Base","region":{"min":[0,0,0],"max":[3,3,3]},"slot":"red","hollow":true}),
    );
    assert_eq!(f.read("query", json!({"kind":"counts"}))["voxel_count"], 26);
    f.fail(
        "voxels_fill",
        json!({"layer":"Base","seed":[4,4,4],"slot":1,"cap":1}),
        "limit_exceeded",
    );
    f.edit(
        "voxels_fill",
        json!({"layer":"Base","seed":[1,1,1],"slot":1,"cap":8}),
    );
    let query = f.read("query", json!({"kind":"voxels","limit":10}));
    assert_eq!(query["voxel_count"], 27);
    assert_eq!(query["next_offset"], 10);
    let next = f.read(
        "query",
        json!({"kind":"voxels","offset":10,"revision":query["revision"],"limit":10}),
    );
    assert_eq!(next["voxels"].as_array().unwrap().len(), 10);
    let palette = f.read("palette_list", json!({}))["palette"].clone();
    let imported = f.edit("palette_apply_preset", json!({"preset":"pico8"}));
    assert_eq!(imported["slot_mapping"].as_array().unwrap().len(), 16);
    assert_eq!(
        f.read("palette_list", json!({}))["palette"]["slots"][0],
        palette["slots"][0]
    );
    f.edit(
        "palette_import",
        json!({"format":"hex","text":"abcdef\n123456\n"}),
    );
    let gpl = f.read("palette_export", json!({"format":"gpl"}));
    assert!(gpl["text"].as_str().unwrap().starts_with("GIMP Palette"));
    f.edit("palette_import", json!({"format":"gpl","text":gpl["text"]}));
    f.fail(
        "palette_set_slot",
        json!({"slot":2,"rgba":[1,2,3,100]}),
        "invalid_argument",
    );
    f.edit(
        "voxels_erase_region",
        json!({"layer":"Base","region":{"min":[0,0,0],"max":[3,3,3]}}),
    );
    assert_eq!(
        f.read("query", json!({"kind":"bounds"}))["occupied_bounds"],
        Value::Null
    );
}
#[test]
fn comments_versions_restore_and_save_reopen() {
    let f = Fixture::new();
    f.edit(
        "voxels_set",
        json!({"layer":"Base","voxels":[{"at":[0,0,0],"slot":1}]}),
    );
    let thread = f.edit(
        "comments_add",
        json!({"anchor":{"kind":"voxel","at":[0,0,0]},"text":"Refine silhouette"}),
    )["thread_id"]
        .clone();
    f.edit(
        "comments_reply",
        json!({"thread_id":thread,"text":"Adjusted"}),
    );
    f.edit("comments_resolve", json!({"thread_id":thread}));
    f.edit("comments_reopen", json!({"thread_id":thread}));
    let version = f.edit("versions_save", json!({"name":"Approved"}))["version_id"].clone();
    let original = f.read("versions_list", json!({}));
    let removed = f.edit(
        "voxels_set",
        json!({"layer":"Base","voxels":[{"at":[0,0,0],"slot":0}]}),
    );
    assert_eq!(
        f.read("comments_list", json!({}))["comments"][0]["anchor_state"],
        "orphaned"
    );
    f.edit(
        "versions_restore",
        json!({"version_id":version,"expected_head_id":removed["head_id"]}),
    );
    assert_eq!(
        f.read("versions_list", json!({}))["versions"],
        original["versions"]
    );
    f.edit("document_save", json!({"path":"model.voxl"}));
    let server = VoxeliteServer::new(f.root.path()).unwrap();
    let epoch = server
        .call("voxelite_capabilities_get", json!({}))
        .unwrap()
        .structured_content
        .unwrap()["epoch"]
        .clone();
    let reopened = server
        .call(
            "voxelite_document_open",
            json!({"path":"model.voxl","operation_id":Uuid::new_v4(),"expected_epoch":epoch}),
        )
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(reopened["data"]["model_id"], f.model);
    assert_eq!(reopened["data"]["revision"], "0");
    assert_eq!(reopened["data"]["versions"], 1);
    assert_eq!(reopened["data"]["voxel_count"], 1);
    let comments = server
        .call("voxelite_comments_list", json!({"model_id":f.model}))
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(
        comments["data"]["comments"][0]["thread"]["messages"][1]["origin"],
        "agent"
    );
    let history = server
        .call("voxelite_history_read", json!({"model_id":f.model}))
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(history["data"]["total"], 0);
}
#[test]
fn file_conflicts_invalid_open_and_root_scope() {
    let f = Fixture::new();
    f.edit("document_save", json!({"path":"saved.voxl"}));
    let bytes = std::fs::read(f.root.path().join("saved.voxl")).unwrap();
    f.edit("document_save", json!({"path":"saved.voxl"}));
    assert_eq!(
        std::fs::read(f.root.path().join("saved.voxl")).unwrap(),
        bytes
    );
    std::fs::write(f.root.path().join("saved.voxl"), b"external change").unwrap();
    f.fail(
        "document_save",
        json!({"path":"saved.voxl"}),
        "external_file_conflict",
    );
    f.fail(
        "document_save",
        json!({"path":"../escape.voxl"}),
        "invalid_argument",
    );
    let r = f.call(
        "document_open",
        json!({"path":"saved.voxl","operation_id":Uuid::new_v4(),"expected_epoch":f.epoch}),
    );
    assert_eq!(r["error"]["code"], "invalid_format");
    assert_eq!(f.read("document_info", json!({}))["model_id"], f.model);
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), f.root.path().join("linked")).unwrap();
        f.fail(
            "document_save",
            json!({"path":"linked/escape.voxl"}),
            "path_outside_root",
        );
    }
}
