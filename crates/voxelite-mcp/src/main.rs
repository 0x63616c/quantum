//! Stdio entry point. Standard output is reserved for MCP JSON-RPC messages.
use rmcp::ServiceExt;
use voxelite_mcp::VoxeliteServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = VoxeliteServer.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
