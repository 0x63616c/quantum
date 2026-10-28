# Documentation

A small world starts with a good first step.

Quantum is a macOS desktop voxel editor with an MCP interface and a native Rust/GPUI workspace. Create with an agent, work by hand, or move between both in a native session.

- [**Getting started**Install the app, connect a host, and make your first model.](https://quantum.worldwidewebb.co/docs/getting-started.md)
- [**MCP tools**The actual server catalog, with parameters and complete schemas.](https://quantum.worldwidewebb.co/docs/mcp.md)
- [**The .voxl format**Portable projects, canonical JSON, and the rules on disk.](https://quantum.worldwidewebb.co/docs/format.md)
- [**Agent skills**Reusable workflows for composition, palettes and visual review.](https://quantum.worldwidewebb.co/docs/skills.md)
- [**Plugins**Built-in features, the core boundary, and writing a plugin.](https://quantum.worldwidewebb.co/docs/plugins.md)

## Choose your workspace

| Workspace     | Best for                                            | Agent connection                           |
| ------------- | --------------------------------------------------- | ------------------------------------------ |
| Native editor | Hands-on editing and watching an agent work         | Local Unix socket through the stdio bridge |
| Headless MCP  | Scripted creation and CPU renders without a display | Stdio, with a project directory            |

These docs describe the implementation on the development branch. The [native Mac download](https://quantum.worldwidewebb.co/download/) is Developer ID signed and notarized by Apple, and checks for updates automatically from version 0.3.0.

Everything stays local. No account, cloud relay, or model-provider key is required by Quantum, and your projects never leave your machine. This website uses cookie-free Vercel Web Analytics and Speed Insights for anonymous page-view and performance counts; nothing you build is collected. Your chosen agent host may have its own requirements.
