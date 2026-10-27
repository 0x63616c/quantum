//! Execute the documentation's actual examples against the shipped adapters.
use serde_json::{Value, json};
use voxelite_mcp::VoxeliteServer;

fn generated_site() -> tempfile::TempDir {
    let output = tempfile::tempdir().unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_voxelite-site"))
        .arg(output.path())
        .current_dir(output.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}

#[test]
fn generated_docs_have_matching_markdown_exports_and_live_catalog() {
    let output = generated_site();
    let read = |path: &str| std::fs::read_to_string(output.path().join(path)).unwrap();
    let index = read("llms.txt");
    let full = read("llms-full.txt");
    for slug in ["", "getting-started", "mcp", "format", "skills", "plugins"] {
        let path = if slug.is_empty() {
            "docs".to_owned()
        } else {
            format!("docs/{slug}")
        };
        let markdown = read(&format!("{path}.md"));
        assert!(markdown.starts_with("# "));
        assert_eq!(markdown, read(&format!("{path}/index.md")));
        assert!(read(&format!("{path}/index.html")).contains(&format!(
            "rel=\"alternate\" type=\"text/markdown\" href=\"/{path}.md\""
        )));
        let url = format!("https://voxelite.worldwidewebb.co/{path}.md");
        assert!(index.contains(&format!("]({url}): ")));
        assert!(full.contains(&format!("Source: [{url}]({url})\n\n{markdown}")));
    }
    let catalog: Value = serde_json::from_str(&read("assets/mcp-tools.json")).unwrap();
    assert_eq!(
        catalog,
        serde_json::to_value(VoxeliteServer::tools()).unwrap()
    );
}

#[test]
fn plugins_are_discoverable_in_navigation_and_machine_readable_docs() {
    let output = generated_site();
    let read = |path: &str| std::fs::read_to_string(output.path().join(path)).unwrap();
    for path in ["index.html", "download/index.html", "docs/index.html"] {
        let html = read(path);
        let navigation = html
            .split("<nav aria-label=\"Main navigation\">")
            .nth(1)
            .unwrap()
            .split("</nav>")
            .next()
            .unwrap();
        assert!(
            navigation.contains("href=\"/docs/plugins/\""),
            "Missing Plugins navigation in {path}"
        );
    }
    assert!(read("docs/index.html").contains("<b>Plugins</b>"));
    for slug in ["getting-started", "mcp", "format", "skills", "plugins"] {
        let html = read(&format!("docs/{slug}/index.html"));
        let navigation = html
            .split("<nav class=\"docs-nav\" aria-label=\"Documentation\">")
            .nth(1)
            .unwrap()
            .split("</nav>")
            .next()
            .unwrap();
        assert!(navigation.contains("href=\"/docs/plugins/\""));
        if slug == "plugins" {
            assert!(navigation.contains("href=\"/docs/plugins/\" aria-current=\"page\""));
        }
    }
    let markdown = read("docs/plugins.md");
    assert!(markdown.starts_with("# Plugins\n\n"));
    assert_eq!(markdown, read("docs/plugins/index.md"));
    assert!(read("docs.md").contains("https://voxelite.worldwidewebb.co/docs/plugins.md"));
    assert!(markdown.contains("https://voxelite.worldwidewebb.co/docs/format/#plugin-data"));
    assert!(
        read("docs/format.md").contains("https://voxelite.worldwidewebb.co/docs/plugins/#data")
    );
    let url = "https://voxelite.worldwidewebb.co/docs/plugins";
    assert!(read("llms.txt").contains(&format!("- [Plugins]({url}.md): ")));
    assert!(read("llms-full.txt").contains(&format!("Source: [{url}.md]({url}.md)\n\n{markdown}")));
    for suffix in ["/", ".md"] {
        assert!(read("sitemap.xml").contains(&format!("<loc>{url}{suffix}</loc>")));
    }
}

#[test]
fn downloadable_project_is_valid_and_canonical() {
    let bytes = include_bytes!("../../../site/assets/minimal.voxl");
    let project = voxelite_format::from_bytes(bytes).expect("download must open");
    assert_eq!(project.to_bytes().unwrap(), bytes);
}

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
    let server = VoxeliteServer::new(root.path()).unwrap();
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
            .replace("<fresh UUID>", &uuid::Uuid::new_v4().to_string())
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
