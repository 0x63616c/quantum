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
fn discovery_schemas_express_conditional_inputs() {
    let f = Fixture::new();
    for (tool, args) in [
        (
            "voxels_box",
            json!({"layer":"Base","region":{"min":[0,0,0],"max":[1,1,1]}}),
        ),
        (
            "voxels_box",
            json!({"layer":"Base","region":{"min":[0,0,0],"max":[1,1,1]},"mode":"erase","slot":1}),
        ),
        (
            "voxels_batch",
            json!({"operations":[{"kind":"box","layer":"Base","region":{"min":[0,0,0],"max":[1,1,1]}}]}),
        ),
        ("palette_set_slot", json!({"slot":2,"rgba":[10,20,30,0]})),
    ] {
        let args = f.args(args);
        let schema = VoxeliteServer::tools()
            .into_iter()
            .find(|t| t.name == format!("voxelite_{tool}"))
            .unwrap()
            .input_schema;
        assert!(!jsonschema::is_valid(
            &Value::Object((*schema).clone()),
            &args
        ));
        assert_eq!(f.call(tool, args)["error"]["code"], "invalid_argument");
    }
    let camera = json!({"eye":[5,5,5],"target":[1,1,1],"up":[0,1,0],"projection":"orthographic","ortho_height":5});
    let mut missing = camera.clone();
    missing.as_object_mut().unwrap().remove("ortho_height");
    let mut wrong = camera.clone();
    wrong["fov_y_degrees"] = json!(45);
    for (tool, mut args) in [
        ("render", json!({"camera":camera,"views":["iso"]})),
        ("render", json!({"camera":camera,"perspective":false})),
        ("render", json!({"camera":missing})),
        ("render", json!({"camera":wrong})),
        ("export_png", json!({"path":"bad.png","image_mode":"both"})),
        (
            "export_png",
            json!({"path":"bad.png","image_mode":"individual"}),
        ),
        (
            "export_png",
            json!({"path":"bad.png","image_mode":"individual","views":["front","back"]}),
        ),
        (
            "query",
            json!({"kind":"counts","view":"composite","layer":"Base"}),
        ),
    ] {
        args["model_id"] = f.model.clone();
        let schema = VoxeliteServer::tools()
            .into_iter()
            .find(|t| t.name == format!("voxelite_{tool}"))
            .unwrap()
            .input_schema;
        assert!(!jsonschema::is_valid(
            &Value::Object((*schema).clone()),
            &args
        ));
        assert_eq!(f.call(tool, args)["error"]["code"], "invalid_argument");
    }
    assert_eq!(f.read("document_info", json!({}))["revision"], "0");
    assert!(!f.root.path().join("bad.png").exists());
}

#[test]
fn output_schemas_reject_corrupt_nested_results_and_allow_additions() {
    let f = Fixture::new();
    f.edit(
        "voxels_set",
        json!({"layer":"Base","voxels":[{"at":[0,0,0],"slot":1}]}),
    );
    f.edit(
        "comments_add",
        json!({"anchor":{"kind":"voxel","at":[0,0,0]},"text":"Checked"}),
    );
    f.edit("versions_save", json!({"name":"First"}));
    for (tool, args, path, invalid) in [
        (
            "document_info",
            json!({}),
            "/layers/0/visible",
            json!("yes"),
        ),
        ("layers_list", json!({}), "/layers/0/id", json!(17)),
        (
            "palette_list",
            json!({}),
            "/palette/slots/0/rgba/3",
            json!(0),
        ),
        (
            "voxels_get",
            json!({"layer":"Base","positions":[[0,0,0]]}),
            "/voxels/0/slot",
            json!("clay"),
        ),
        (
            "query",
            json!({"kind":"voxels"}),
            "/voxels/0/at/0",
            json!(0.5),
        ),
        (
            "comments_list",
            json!({}),
            "/comments/0/thread/messages/0/origin",
            json!("system"),
        ),
        (
            "history_read",
            json!({}),
            "/entries/0/result_revision",
            json!(1),
        ),
        (
            "versions_list",
            json!({}),
            "/versions/0/content_hash",
            json!("bad"),
        ),
        (
            "render",
            json!({"views":["iso"],"width":32,"height":32}),
            "/views/0/camera/ortho_height",
            json!("NaN"),
        ),
        (
            "render",
            json!({"views":["iso"],"width":32,"height":32}),
            "/images/0/sha256",
            json!("bad"),
        ),
    ] {
        let mut data = f.read(tool, args);
        let schema = VoxeliteServer::tools()
            .into_iter()
            .find(|t| t.name == format!("voxelite_{tool}"))
            .unwrap()
            .output_schema
            .unwrap();
        let validator = jsonschema::validator_for(&schema["properties"]["data"]).unwrap();
        assert!(validator.is_valid(&data));
        *data.pointer_mut(path).unwrap() = invalid;
        assert!(!validator.is_valid(&data), "{tool} accepted invalid {path}");
    }
    let mut info = f.read("document_info", json!({}));
    info["layers"][0]["future_field"] = json!(true);
    let schema = VoxeliteServer::tools()
        .into_iter()
        .find(|t| t.name == "voxelite_document_info")
        .unwrap()
        .output_schema
        .unwrap();
    assert!(jsonschema::is_valid(&schema["properties"]["data"], &info));
    let capabilities = f.call("capabilities_get", json!({}));
    assert_eq!(
        capabilities["data"]["documents"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn maximum_layer_summary_fits_reserved_receipt_and_replays() {
    let root = tempfile::tempdir().unwrap();
    let mut document = voxelite_core::Document::new(Uuid::new_v4(), Uuid::new_v4());
    document.content.layers = (0..64)
        .map(|_| voxelite_core::Layer::new(Uuid::new_v4(), "\"".repeat(128)))
        .collect();
    let project = voxelite_format::Project::new(document).unwrap();
    std::fs::write(
        root.path().join("maximum.voxl"),
        project.to_bytes().unwrap(),
    )
    .unwrap();
    let server = VoxeliteServer::new(root.path()).unwrap();
    let epoch = server
        .call("voxelite_capabilities_get", json!({}))
        .unwrap()
        .structured_content
        .unwrap()["epoch"]
        .clone();
    let args = json!({"path":"maximum.voxl","expected_epoch":epoch,"operation_id":Uuid::new_v4()});
    let result = server.call("voxelite_document_open", args.clone()).unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(serde_json::to_vec(&result).unwrap().len() <= 65536);
    assert_eq!(server.call("voxelite_document_open", args).unwrap(), result);
}

#[test]
fn simultaneous_exact_retries_commit_once_and_replay_after_later_edits() {
    let f = Fixture::new();
    let args = f.args(json!({"layer":"Base","voxels":[{"at":[1,1,1],"slot":1}]}));
    let results: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| f.server.call("voxelite_voxels_set", args.clone()).unwrap()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert!(results.iter().all(|r| r == &results[0]));
    assert_eq!(f.read("history_read", json!({}))["total"], 1);
    f.edit(
        "voxels_erase_region",
        json!({"layer":"Base","region":{"min":[0,0,0],"max":[2,2,2]}}),
    );
    assert_eq!(
        f.server.call("voxelite_voxels_set", args).unwrap(),
        results[0]
    );
    assert_eq!(f.read("document_info", json!({}))["voxel_count"], 0);
}
#[test]
fn atomic_batches_mirror_noop_and_operation_receipts() {
    let f = Fixture::new();
    let failed=f.fail("voxels_batch",json!({"operations":[{"kind":"set","layer":"Base","voxels":[{"at":[1,1,1],"slot":1}]},{"kind":"box","layer":"Base","region":{"min":[31,0,0],"max":[33,1,1]},"slot":1}]}),"out_of_bounds");
    assert_eq!(failed["error"]["command_index"], 1);
    assert_eq!(failed["error"]["path"], "/operations/1");
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
fn batch_explicit_voxel_budget_is_shared_and_rejection_is_atomic() {
    let f = Fixture::new();
    let cells = vec![json!({"at":[1,1,1],"slot":1}); 32769];
    let mut args = f.args(json!({"operations":[
        {"kind":"set","layer":"Base","voxels":cells},
        {"kind":"set","layer":"Base","voxels":cells}
    ]}));
    let result = f.call("voxels_batch", args.clone());
    assert_eq!(result["error"]["code"], "limit_exceeded");
    assert_eq!(result["error"]["command_index"], 1);
    assert_eq!(result["error"]["path"], "/operations/1/voxels");
    assert_eq!(f.read("document_info", json!({}))["revision"], "0");
    assert_eq!(f.read("document_info", json!({}))["voxel_count"], 0);
    for op in args["operations"].as_array_mut().unwrap() {
        op["voxels"].as_array_mut().unwrap().pop();
    }
    assert_eq!(f.call("voxels_batch", args)["ok"], true);
    assert_eq!(f.read("document_info", json!({}))["voxel_count"], 1);
}

#[test]
fn labels_are_validated_even_when_no_core_transaction_runs() {
    let f = Fixture::new();
    for label in ["\n", &"🦀".repeat(21)] {
        let result = f.fail(
            "document_save",
            json!({"path":"invalid.voxl","label":label}),
            "invalid_argument",
        );
        assert_eq!(result["error"]["path"], "/label");
        assert!(!f.root.path().join("invalid.voxl").exists());
        let result = f.call("document_create", json!({"name":"Invalid","label":label,"operation_id":Uuid::new_v4(),"expected_epoch":f.epoch}));
        assert_eq!(result["error"]["code"], "invalid_argument");
    }
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

#[test]
fn render_cameras_limits_and_exports_are_read_only() {
    let f = Fixture::new();
    f.edit(
        "voxels_box",
        json!({"layer":"Base","region":{"min":[0,0,0],"max":[2,2,2]},"slot":1}),
    );
    let before = f.read("document_info", json!({}));
    let rendered=f.read("render",json!({"camera":{"eye":[5,5,5],"target":[1,1,1],"up":[0,1,0],"projection":"perspective","fov_y_degrees":45},"width":32,"height":32,"image_mode":"individual"}));
    assert_eq!(
        rendered["views"][0]["camera"]["eye"],
        json!([5.0, 5.0, 5.0])
    );
    for args in [
        json!({"views":["iso"],"camera":{"eye":[5,5,5],"target":[1,1,1],"up":[0,1,0],"projection":"orthographic","ortho_height":5}}),
        json!({"camera":{"eye":[0,0,0],"target":[0,0,0],"up":[0,1,0],"projection":"orthographic","ortho_height":5}}),
        json!({"views":["iso","front","right","back","top","left"],"width":1024,"height":1024}),
    ] {
        let mut args = args;
        args["model_id"] = f.model.clone();
        assert_eq!(f.call("render", args)["ok"], false);
    }
    let png = f.read(
        "export_png",
        json!({"path":"one.png","views":["iso"],"width":32,"height":32}),
    );
    assert_eq!(
        png["bytes"],
        std::fs::metadata(f.root.path().join("one.png"))
            .unwrap()
            .len()
    );
    assert_eq!(
        f.call(
            "export_png",
            json!({"model_id":f.model,"path":"one.png","views":["iso"],"width":32,"height":32})
        )["error"]["code"],
        "file_exists"
    );
    let a = f.read("export_glb", json!({"path":"one.glb"}));
    let b = f.read("export_glb", json!({"path":"two.glb"}));
    assert_eq!(a["sha256"], b["sha256"]);
    assert_eq!(f.read("document_info", json!({})), before);
}
#[test]
fn concurrent_mutations_use_one_revision_and_paging_rejects_changed_heads() {
    let f = Fixture::new();
    let mut handles = Vec::new();
    for x in 0..8 {
        let server = f.server.clone();
        let args = f.args(json!({"layer":"Base","voxels":[{"at":[x,0,0],"slot":1}]}));
        handles.push((server, args));
    }
    let results: Vec<_> = std::thread::scope(|scope| {
        let tasks: Vec<_> = handles
            .into_iter()
            .map(|(server, args)| {
                scope.spawn(move || {
                    server
                        .call("voxelite_voxels_set", args)
                        .unwrap()
                        .structured_content
                        .unwrap()
                })
            })
            .collect();
        tasks.into_iter().map(|t| t.join().unwrap()).collect()
    });
    assert_eq!(results.iter().filter(|r| r["ok"] == true).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| r["error"]["code"] == "revision_conflict")
            .count(),
        7
    );
    assert_eq!(f.read("history_read", json!({}))["total"], 1);
    let r = f.call(
        "query",
        json!({"model_id":f.model,"revision":"0","kind":"voxels"}),
    );
    assert_eq!(r["error"]["code"], "revision_conflict");
}
#[test]
fn unicode_comments_restore_guards_and_palette_capacity_are_atomic() {
    let f = Fixture::new();
    assert_eq!(f.read("document_info", json!({}))["dirty"], true);
    f.fail(
        "comments_add",
        json!({"anchor":{"kind":"voxel","at":[0,0,0]},"text":"🦀".repeat(1025)}),
        "invalid_argument",
    );
    let version = f.edit("versions_save", json!({"name":"Empty"}))["version_id"].clone();
    f.edit(
        "voxels_set",
        json!({"layer":"Base","voxels":[{"at":[0,0,0],"slot":1}]}),
    );
    f.fail(
        "versions_restore",
        json!({"version_id":version,"expected_head_id":null}),
        "history_conflict",
    );
    f.edit("layers_lock", json!({"layer":"Base","locked":true}));
    let head = f.read("history_read", json!({}))["head_id"].clone();
    f.fail(
        "versions_restore",
        json!({"version_id":version,"expected_head_id":head}),
        "layer_blocked",
    );
    let before = f.read("palette_list", json!({}));
    f.fail(
        "palette_import",
        json!({"format":"hex","text":"ffffff\n".repeat(256)}),
        "limit_exceeded",
    );
    assert_eq!(f.read("palette_list", json!({})), before);
    f.fail(
        "palette_import",
        json!({"format":"gpl","text":"not a palette"}),
        "invalid_argument",
    );
}
#[test]
fn composite_reads_respect_visibility_and_duplicate_model_open_rejects() {
    let f = Fixture::new();
    f.edit("palette_set_slot", json!({"slot":2,"rgba":[1,2,3,255]}));
    let upper = f.edit("layers_add", json!({"name":"Upper"}))["layer_id"].clone();
    f.edit("voxels_batch",json!({"operations":[{"kind":"set","layer":"Base","voxels":[{"at":[0,0,0],"slot":1}]},{"kind":"set","layer":upper,"voxels":[{"at":[0,0,0],"slot":2}]}]}));
    assert_eq!(
        f.read("query", json!({"kind":"voxels","view":"composite"}))["voxels"][0]["slot"],
        2
    );
    assert_eq!(f.read("query", json!({"kind":"counts"}))["voxel_count"], 2);
    f.edit("layers_visibility", json!({"layer":upper,"visible":false}));
    assert_eq!(
        f.read("query", json!({"kind":"voxels","view":"composite"}))["voxels"][0]["slot"],
        1
    );
    f.edit("document_save", json!({"path":"duplicate.voxl"}));
    let r = f.call(
        "document_open",
        json!({"path":"duplicate.voxl","expected_epoch":f.epoch,"operation_id":Uuid::new_v4()}),
    );
    assert_eq!(r["error"]["code"], "model_already_open");
}

#[test]
fn discussion_message_pages_remain_accessible() {
    let f = Fixture::new();
    let thread = f.edit(
        "comments_add",
        json!({"anchor":{"kind":"voxel","at":[0,0,0]},"text":"First"}),
    )["thread_id"]
        .clone();
    f.edit(
        "comments_reply",
        json!({"thread_id":thread,"text":"Second"}),
    );
    f.edit("comments_reply", json!({"thread_id":thread,"text":"Third"}));
    let first = f.read("comments_list", json!({"message_limit":1}));
    assert_eq!(first["comments"][0]["message_count"], 3);
    assert_eq!(first["comments"][0]["next_message_offset"], 1);
    let second = f.read(
        "comments_list",
        json!({"message_offset":1,"message_limit":1,"revision":first["revision"]}),
    );
    assert_eq!(
        second["comments"][0]["thread"]["messages"][0]["text"],
        "Second"
    );
    let last = f.read(
        "comments_list",
        json!({"message_offset":2,"message_limit":1}),
    );
    assert!(last["comments"][0]["next_message_offset"].is_null());
    assert_eq!(
        f.read("comments_list", json!({"offset":1024}))["comments"],
        json!([])
    );
}
#[test]
fn open_edit_save_preserves_inert_unknown_project_metadata() {
    let f = Fixture::new();
    let mut project: Value = serde_json::from_str(include_str!(
        "../../../spec/voxl/1.0/fixtures/populated.input.voxl"
    ))
    .unwrap();
    project["future_optional"] = json!({"nested":[1,"retained"]});
    project["layers"][0]["future_layer"] = json!({"inert":true});
    std::fs::write(
        f.root.path().join("future.voxl"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let opened = f.call(
        "document_open",
        json!({"path":"future.voxl","expected_epoch":f.epoch,"operation_id":Uuid::new_v4()}),
    );
    assert_eq!(opened["ok"], true, "{opened}");
    let model = opened["data"]["model_id"].clone();
    let renamed=f.call("layers_rename",json!({"model_id":model,"expected_epoch":f.epoch,"operation_id":Uuid::new_v4(),"expected_revision":"0","layer":opened["data"]["layers"][0]["id"],"name":"Renamed"}));
    assert_eq!(renamed["ok"], true);
    let saved=f.call("document_save",json!({"model_id":model,"expected_epoch":f.epoch,"operation_id":Uuid::new_v4(),"expected_revision":renamed["data"]["revision"],"path":"future.voxl"}));
    assert_eq!(saved["ok"], true);
    let roundtrip: Value =
        serde_json::from_slice(&std::fs::read(f.root.path().join("future.voxl")).unwrap()).unwrap();
    assert_eq!(roundtrip["future_optional"], project["future_optional"]);
    assert_eq!(
        roundtrip["layers"][0]["future_layer"],
        project["layers"][0]["future_layer"]
    );
    assert_eq!(roundtrip["layers"][0]["name"], "Renamed");
}

#[test]
fn jasc_pal_import_and_export_preserve_existing_document_slots() {
    let f = Fixture::new();
    let before = f.read("palette_list", json!({}))["palette"].clone();
    let result = f.edit(
        "palette_import",
        json!({"format":"pal","text":"JASC-PAL\n0100\n2\n1 2 3\n1 2 3\n"}),
    );
    assert_eq!(
        result["slot_mapping"],
        json!([{"source":1,"slot":2},{"source":2,"slot":3}])
    );
    let after = f.read("palette_list", json!({}))["palette"].clone();
    assert_eq!(after["slots"][0], before["slots"][0]);
    let exported = f.read("palette_export", json!({"format":"pal"}));
    assert!(
        exported["text"]
            .as_str()
            .unwrap()
            .starts_with("JASC-PAL\n0100\n3\n")
    );
    assert!(
        exported["text"]
            .as_str()
            .unwrap()
            .ends_with("1 2 3\n1 2 3\n")
    );
    assert_eq!(f.read("palette_list", json!({}))["palette"], after);
}
