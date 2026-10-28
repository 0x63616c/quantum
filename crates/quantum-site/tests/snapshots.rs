//! Snapshot real generator output without writing into the checkout.

#[test]
fn generated_public_documents() {
    let output = tempfile::tempdir().unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_quantum-site"))
        .arg(output.path())
        .current_dir(output.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    // These outputs contain no timestamps or absolute filesystem paths, so no
    // redactions are needed.
    for (name, path) in [
        ("sitemap", "sitemap.xml"),
        ("robots", "robots.txt"),
        ("llms", "llms.txt"),
        ("docs_overview", "docs/index.html"),
    ] {
        let text = std::fs::read_to_string(output.path().join(path)).unwrap();
        insta::assert_snapshot!(name, text);
    }
    for route in ["changelog", "blog", "blog/2026-10-28-rename"] {
        let html = std::fs::read_to_string(output.path().join(route).join("index.html")).unwrap();
        assert_eq!(html.matches("<h1>").count(), 1);
        assert!(!html.contains("@@"));
        assert!(html.contains("/changelog/"));
        assert!(html.contains("/blog/"));
    }
    let release_source = include_str!("../../../CHANGELOG.md");
    let exported = std::fs::read_to_string(output.path().join("changelog/index.md")).unwrap();
    assert!(exported.contains("## [Unreleased]"));
    assert!(
        exported.contains(
            release_source
                .lines()
                .find(|line| line.starts_with("- "))
                .unwrap()
        )
    );
    let post =
        std::fs::read_to_string(output.path().join("blog/2026-10-28-rename/index.html")).unwrap();
    assert!(post.contains("automatically migrates"));
    // The published catalog is the spec file re-emitted; MCP's own snapshots
    // (quantum-mcp/tests/wire_snapshots.rs) cover each tool's schema, so only
    // check the site copy matches the spec and snapshot the tool order.
    let catalog: serde_json::Value = serde_json::from_slice(
        &std::fs::read(output.path().join("assets/mcp-tools.json")).unwrap(),
    )
    .unwrap();
    let spec: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/mcp/v1/tools.json")).unwrap();
    assert_eq!(
        catalog, spec,
        "site MCP catalog must match spec/mcp/v1/tools.json"
    );
    let names: Vec<_> = catalog
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    insta::assert_json_snapshot!("catalog_tool_order", names);
}
