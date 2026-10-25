//! Stdio entry point. Standard output is reserved for MCP JSON-RPC messages.
use voxelite_mcp::VoxeliteServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut root = std::env::current_dir()?;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = args.next().ok_or("--root needs a directory")?.into(),
            "--schema" => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&VoxeliteServer::tools())?
                );
                return Ok(());
            }
            "--help" | "-h" => {
                println!(
                    "voxelite-mcp [--root DIRECTORY]\n\nServe voxelite.v1 over newline-delimited stdio JSON-RPC.\n--root DIRECTORY  Existing project directory (default current directory).\n--schema          Print the tool catalog as JSON and exit.\n--version         Print binary version and exit."
                );
                return Ok(());
            }
            "--version" | "-V" => {
                println!("voxelite-mcp {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}; see --help").into()),
        }
    }
    VoxeliteServer::new(root)?.serve_stdio().await?;
    Ok(())
}
