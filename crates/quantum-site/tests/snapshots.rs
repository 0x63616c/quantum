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
