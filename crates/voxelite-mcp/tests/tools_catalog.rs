//! Keep the versioned catalog used by the site byte-identical to MCP discovery.
use std::{fs, path::Path};
use voxelite_mcp::VoxeliteServer;

#[test]
fn tools_catalog_matches_server() {
    let catalog = format!(
        "{}\n",
        serde_json::to_string_pretty(&VoxeliteServer::tools()).unwrap()
    );
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/mcp/v1/tools.json");
    if std::env::var("UPDATE_TOOLS_JSON").as_deref() == Ok("1") {
        fs::write(&path, &catalog).expect("write MCP tool catalog");
    }
    let committed = fs::read_to_string(path).expect("read spec/mcp/v1/tools.json");
    assert!(
        committed == catalog,
        "MCP tool catalog drifted. Regenerate with: UPDATE_TOOLS_JSON=1 cargo test -p voxelite-mcp --locked --test tools_catalog"
    );
}
