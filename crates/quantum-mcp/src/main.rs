//! Stdio entry point. Standard output is reserved for MCP JSON-RPC messages.
mod reports;
use quantum_mcp::QuantumServer;

#[tokio::main]
async fn main() -> miette::Result<()> {
    run().await.map_err(|error| reports::report(error.as_ref()))
}

#[expect(clippy::print_stdout, reason = "CLI output")]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut root = std::env::current_dir()?;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            #[cfg(unix)]
            "--connect" => {
                let path = args.next().ok_or("--connect needs a socket path")?;
                quantum_mcp::attached::proxy_stdio(std::path::Path::new(&path)).await?;
                return Ok(());
            }
            "--root" => root = args.next().ok_or("--root needs a directory")?.into(),
            "--schema" => {
                println!("{}", serde_json::to_string_pretty(&QuantumServer::tools())?);
                return Ok(());
            }
            "--help" | "-h" => {
                println!(
                    "quantum-mcp [--root DIRECTORY]\nquantum-mcp --connect SOCKET\n\nServe quantum.v1 over newline-delimited stdio JSON-RPC.\n--root DIRECTORY  Existing project directory (default current directory).\n--schema          Print the tool catalog as JSON and exit.\n--version         Print binary version and exit."
                );
                return Ok(());
            }
            "--version" | "-V" => {
                println!("quantum-mcp {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}; see --help").into()),
        }
    }
    QuantumServer::new(root, quantum_plugins_builtin::mcp_host)?
        .serve_stdio()
        .await?;
    Ok(())
}
