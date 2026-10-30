//! Characterization captured before replacing the hand-written tool registry.
use serde_json::{Value, json};
use voxelite_core::Uuid;
use voxelite_mcp::VoxeliteServer;
use voxelite_plugins_builtin::mcp_host;

#[test]
fn tool_catalog() {
    insta::assert_json_snapshot!(VoxeliteServer::tools());
}

// Only the session identity/epoch are nondeterministic. Keep the actual text
// content, structured envelope and image bytes, including their key order.
#[expect(
    clippy::unwrap_used,
    reason = "test fixture or assertion helper reports invalid setup by panicking"
)]
fn capture(server: &VoxeliteServer, name: &str, args: Value) -> Value {
    let result = server.call(&format!("voxelite_{name}"), args).unwrap();
    let envelope = result.structured_content.as_ref().unwrap();
    let encoded = serde_json::to_string(&result)
        .unwrap()
        .replace(envelope["session_id"].as_str().unwrap(), "<session>")
        .replace(envelope["epoch"].as_str().unwrap(), "<epoch>");
    serde_json::from_str(&encoded).unwrap()
}

#[test]
fn representative_results() {
    let root = tempfile::tempdir().unwrap();
    let server = VoxeliteServer::new(root.path(), mcp_host).unwrap();
    let epoch = server
        .call("voxelite_capabilities_get", json!({}))
        .unwrap()
        .structured_content
        .unwrap()["epoch"]
        .clone();
    let model = Uuid::from_u128(1);
    let layer = Uuid::from_u128(2);
    let mut results = vec![capture(&server, "capabilities_get", json!({}))];
    results.push(capture(
        &server,
        "document_create",
        json!({
            "expected_epoch":epoch,"operation_id":Uuid::from_u128(3),
            "model_id":model,"layer_id":layer,"name":"Snapshot"
        }),
    ));
    let args = json!({"model_id":model,"expected_epoch":epoch,"expected_revision":"0",
        "operation_id":Uuid::from_u128(4),"layer":"Base",
        "voxels":[{"at":[1,2,3],"slot":1}],"label":"Place voxel"});
    results.push(capture(&server, "voxels_set", args.clone()));
    results.push(capture(&server, "voxels_set", args.clone()));
    for (name, mut args) in [
        ("document_info", json!({})),
        (
            "voxels_get",
            json!({"layer":"Base","positions":[[1,2,3],[0,0,0]]}),
        ),
        ("query", json!({"kind":"voxels","limit":1})),
        ("layers_list", json!({})),
        ("palette_list", json!({})),
        ("palette_export", json!({"format":"hex"})),
        ("history_read", json!({})),
        ("comments_list", json!({})),
        ("versions_list", json!({})),
        (
            "render",
            json!({"views":["front"],"width":8,"height":8,"image_mode":"individual"}),
        ),
    ] {
        args["model_id"] = json!(model);
        results.push(capture(&server, name, args));
    }
    let mut stale = args.clone();
    stale["operation_id"] = json!(Uuid::from_u128(5));
    results.push(capture(&server, "voxels_set", stale));
    let mut reused = args;
    reused["voxels"][0]["slot"] = json!(0);
    results.push(capture(&server, "voxels_set", reused));
    results.push(capture(
        &server,
        "history_undo",
        json!({
            "model_id":model,"expected_epoch":epoch,"expected_revision":"1",
            "operation_id":Uuid::from_u128(6),"entry_id":Uuid::from_u128(4)
        }),
    ));
    insta::assert_json_snapshot!(results);
}

#[test]
fn argument_errors() {
    let root = tempfile::tempdir().unwrap();
    let server = VoxeliteServer::new(root.path(), mcp_host).unwrap();
    let epoch = server
        .call("voxelite_capabilities_get", json!({}))
        .unwrap()
        .structured_content
        .unwrap()["epoch"]
        .clone();
    let model = Uuid::from_u128(1);
    capture(
        &server,
        "document_create",
        json!({"expected_epoch":epoch,
        "operation_id":Uuid::from_u128(2),"model_id":model,"name":"Errors"}),
    );
    let mut results = vec![];
    for (name, args) in [
        ("document_info", json!({})),
        ("document_info", json!({"model_id":model,"revision":null})),
        (
            "document_info",
            json!({"model_id":model,"revision":"18446744073709551616"}),
        ),
        ("document_info", json!({"model_id":model,"extra":true})),
        ("query", json!({"model_id":model,"kind":"bad"})),
        (
            "query",
            json!({"model_id":model,"kind":"counts","view":"composite","layer":"Base"}),
        ),
        ("render", json!({"model_id":model,"width":0})),
        ("render", json!({"model_id":model,"width":1.0})),
        (
            "export_png",
            json!({"model_id":model,"path":"bad.png","image_mode":"individual"}),
        ),
    ] {
        results.push(json!({"tool":name,"result":capture(&server, name, args)}));
    }
    for (name, mut args) in [
        (
            "voxels_set",
            json!({"layer":"Base","voxels":[{"at":[0,0,0],"slot":1,"extra":true}]}),
        ),
        (
            "voxels_set",
            json!({"layer":"Base","voxels":[{"at":[0.0,0,0],"slot":1}]}),
        ),
        (
            "voxels_box",
            json!({"layer":"Base","region":{"min":[0,0,0],"max":[1,1,1]}}),
        ),
        (
            "voxels_batch",
            json!({"operations":[{"kind":"box","layer":"Base","region":{"min":[0,0,0],"max":[1,1,1]}}]}),
        ),
        ("palette_set_slot", json!({"slot":2,"rgba":[1,2,3,0]})),
        (
            "document_save",
            json!({"path":"bad.voxl","label":"🦀".repeat(21)}),
        ),
        ("versions_restore", json!({"version_id":model})),
    ] {
        args["model_id"] = json!(model);
        args["expected_epoch"] = epoch.clone();
        args["expected_revision"] = json!("0");
        args["operation_id"] = json!(Uuid::from_u128(3));
        results.push(json!({"tool":name,"result":capture(&server, name, args)}));
    }
    insta::assert_json_snapshot!(results);
}

#[test]
fn numeric_decoding_and_error_precedence() {
    let root = tempfile::tempdir().unwrap();
    let server = VoxeliteServer::new(root.path(), mcp_host).unwrap();
    let epoch = server
        .call("voxelite_capabilities_get", json!({}))
        .unwrap()
        .structured_content
        .unwrap()["epoch"]
        .clone();
    let model = Uuid::from_u128(1);
    capture(
        &server,
        "document_create",
        json!({"expected_epoch":epoch,
        "operation_id":Uuid::from_u128(2),"model_id":model,"layer_id":Uuid::from_u128(3),"name":"Errors"}),
    );
    let mut results = vec![];
    for (name, args) in [
        (
            "voxels_set",
            json!({"layer":"Base","voxels":[{"at":[0.0,0,0],"slot":1}]}),
        ),
        (
            "voxels_set",
            json!({"layer":"Absent","voxels":[{"at":[0.0,0,0],"slot":1}]}),
        ),
        (
            "voxels_set",
            json!({"layer":"Base","voxels":[{"at":[0,0,0],"slot":1.0}]}),
        ),
        (
            "voxels_set",
            json!({"layer":"Base","mirror":{"axes":["x"],"plane2":[0.0,0,0]},"voxels":[{"at":[0.0,0,0],"slot":1}]}),
        ),
        (
            "voxels_box",
            json!({"layer":"Base","region":{"min":[0.0,0,0],"max":[1,1,1]},"slot":1}),
        ),
        (
            "voxels_batch",
            json!({"operations":[{"kind":"set","layer":"Base","voxels":[{"at":[0.0,0,0],"slot":1}]}]}),
        ),
        ("layers_add", json!({"name":"Layer","index":1.0})),
        ("palette_set_slot", json!({"slot":2.0,"rgba":[1,2,3,255]})),
        ("palette_set_slot", json!({"slot":2,"rgba":[1.0,2,3,255]})),
        (
            "comments_add",
            json!({"anchor":{"kind":"voxel","at":[0.0,0,0]},"text":"Message"}),
        ),
        (
            "voxels_fill",
            json!({"layer":"Base","seed":[0,0,0],"slot":1,"cap":1.0}),
        ),
    ] {
        for revision in ["0", "1"] {
            let mut args = args.clone();
            args["model_id"] = json!(model);
            args["expected_epoch"] = epoch.clone();
            args["expected_revision"] = json!(revision);
            args["operation_id"] = json!(Uuid::from_u128(4));
            results.push(
                json!({"tool":name,"revision":revision,"result":capture(&server, name, args)}),
            );
        }
    }
    for (name, args) in [
        (
            "voxels_get",
            json!({"layer":"Base","positions":[[0.0,0,0]]}),
        ),
        ("query", json!({"kind":"counts","offset":1.0})),
        ("query", json!({"kind":"voxels","limit":1.0})),
        ("render", json!({"width":1.0})),
        ("comments_list", json!({"offset":1.0})),
    ] {
        for id in [model, Uuid::nil()] {
            let mut args = args.clone();
            args["model_id"] = json!(id);
            results.push(json!({"tool":name,"model_id":id,"result":capture(&server, name, args)}));
        }
    }
    insta::assert_json_snapshot!(results);
}
