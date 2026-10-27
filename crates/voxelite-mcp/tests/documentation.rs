//! Execute the published tutorial against the shipped MCP adapter.
use serde_json::{Value, json};
use voxelite_core::Uuid;
use voxelite_mcp::VoxeliteServer;
use voxelite_plugins_builtin::mcp_host;

fn call(server: &VoxeliteServer, tool: &str, arguments: Value) -> Value {
    let response = server.call(tool, arguments).expect("known tool");
    assert_eq!(response.is_error, Some(false), "{response:?}");
    let envelope = response.structured_content.unwrap();
    assert_eq!(envelope["ok"], true, "{envelope}");
    envelope
}

#[test]
fn published_tutorial_renders_and_saves_a_reopenable_tower() {
    let root = tempfile::tempdir().unwrap();
    let server = VoxeliteServer::new(root.path(), mcp_host).unwrap();
    let epoch = call(&server, "voxelite_capabilities_get", json!({}))["epoch"].clone();
    let source = include_str!("../../../site/content/getting-started.html");
    let tutorial = source.split("id=\"first-model\"").nth(1).unwrap();
    let mut model = Value::Null;
    let mut revision = json!("0");
    let tools = [
        "voxelite_document_create",
        "voxelite_voxels_batch",
        "voxelite_render",
        "voxelite_document_save",
        "voxelite_export_glb",
    ];
    let examples: Vec<_> = tutorial
        .split("<code class=\"language-json\">")
        .skip(1)
        .collect();
    assert_eq!(examples.len(), tools.len());
    for (example, tool) in examples.into_iter().zip(tools) {
        let text = example
            .split("</code>")
            .next()
            .unwrap()
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("<returned epoch>", epoch.as_str().unwrap())
            .replace("<fresh UUID>", &Uuid::new_v4().to_string())
            .replace("<returned model_id>", model.as_str().unwrap_or(""))
            .replace("<current revision>", revision.as_str().unwrap());
        let arguments: Value = serde_json::from_str(&text).unwrap();
        if tool == "voxelite_render" {
            let response = server.call(tool, arguments.clone()).unwrap();
            let serialized = serde_json::to_value(&response).unwrap();
            assert!(
                serialized["content"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|block| block["type"] == "image" && block["mimeType"] == "image/png")
            );
        }
        let result = call(&server, tool, arguments);
        if tool == "voxelite_document_create" {
            model = result["data"]["model_id"].clone();
        }
        revision = call(
            &server,
            "voxelite_document_info",
            json!({"model_id": model}),
        )["data"]["revision"]
            .clone();
    }
    let info = call(
        &server,
        "voxelite_document_info",
        json!({"model_id": model}),
    );
    assert_eq!(info["data"]["voxel_count"], 1064);
    let bytes = std::fs::read(root.path().join("first-tower.voxl")).unwrap();
    let reopened = voxelite_format::from_bytes(&bytes).expect("saved tutorial opens");
    assert_eq!(reopened.to_bytes().unwrap(), bytes);
    let glb = std::fs::read(root.path().join("first-tower.glb")).unwrap();
    assert_eq!(&glb[..4], b"glTF");
    assert_eq!(
        u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize,
        glb.len()
    );
}
