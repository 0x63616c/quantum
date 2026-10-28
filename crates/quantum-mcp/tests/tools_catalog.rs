//! Keep the versioned catalog used by the site byte-identical to MCP discovery.
use quantum_mcp::QuantumServer;
use std::{fs, path::Path};

#[test]
fn tools_catalog_matches_server() {
    let catalog = format!(
        "{}\n",
        serde_json::to_string_pretty(&QuantumServer::tools()).unwrap()
    );
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/mcp/v1/tools.json");
    if std::env::var("UPDATE_TOOLS_JSON").as_deref() == Ok("1") {
        fs::write(&path, &catalog).expect("write MCP tool catalog");
    }
    let committed = fs::read_to_string(path).expect("read spec/mcp/v1/tools.json");
    assert!(
        committed == catalog,
        "MCP tool catalog drifted. Regenerate with: UPDATE_TOOLS_JSON=1 cargo test -p quantum-mcp --locked --test tools_catalog"
    );
}

#[test]
fn compatibility_versions_describe_current_output_and_legacy_readers() {
    let compatibility: serde_json::Value =
        serde_json::from_str(include_str!("../../../spec/mcp/v1/compatibility.json")).unwrap();
    assert_eq!(compatibility["api_version"], quantum_mcp::API_VERSION);
    let aliases = compatibility["legacy_api_versions"].as_array().unwrap();
    assert_eq!(aliases, &[serde_json::json!("voxelite.v1")]);
    assert_ne!(aliases[0], quantum_mcp::API_VERSION);
    for tool in serde_json::to_value(QuantumServer::tools())
        .unwrap()
        .as_array()
        .unwrap()
    {
        assert_eq!(
            tool["outputSchema"]["properties"]["api_version"]["const"],
            quantum_mcp::API_VERSION
        );
    }
}
