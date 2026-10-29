# Serena symbol navigation (trial)

[Serena](https://github.com/oraios/serena) is an MCP server that gives coding agents
symbol-level tools (`find_symbol`, `find_referencing_symbols`, `get_symbols_overview`,
symbol-scoped edits) backed by rust-analyzer. This page records how it is set up for
Codex on the agent box and what it costs. Issue #253.

## Repo side

- `.serena/project.yml` is the project config. Serena rewrites the file into its full
  template on first load, so the committed copy is that normalised form. Values that
  differ from the defaults: `project_name: quantum`, `language_servers: [rust]`,
  `added_modes: [no-memories]` (AGENTS.md and Tickets are the memory, so
  Serena's own memory files and onboarding stay off).
- `.serena/.gitignore` keeps everything else under `.serena/` (symbol cache,
  `project.local.yml`, memories) untracked.

## Agent machine side (not in this repo)

Install with uv, as the
[installation docs](https://oraios.github.io/serena/02-usage/010_installation.html) say:

```sh
uv tool install -p 3.13 serena-agent   # puts `serena` in ~/.local/bin
serena init                            # writes ~/.serena/serena_config.yml
rustup component add rust-analyzer     # Serena uses rust-analyzer from the toolchain
```

The [Rust language notes](https://oraios.github.io/serena/01-about/020_programming-languages.html)
say Serena "uses rust-analyzer from your toolchain". `rust-toolchain.toml` pins
`profile = "minimal"`, so the component has to be added once per box.

Codex block from the [clients page](https://oraios.github.io/serena/02-usage/030_clients.html),
in `~/.codex/config.toml` (Codex only reads MCP servers from user config):

```toml
[mcp_servers.serena]
startup_timeout_sec = 15
command = "/absolute/path/to/serena"   # e.g. the output of `command -v serena`
args = ["start-mcp-server", "--project-from-cwd", "--context=codex"]
env = { SERENA_USAGE_REPORTING = "false", CARGO_TARGET_DIR = "/absolute/path/to/quantum-ra-target" }
```

| Setting | Why |
| --- | --- |
| `--context=codex`, `--project-from-cwd`, `startup_timeout_sec = 15` | Exactly the documented Codex block. The MCP handshake takes about 1 s on the box; rust-analyzer starts in the background after that. |
| absolute `command` path | The clients page's "Common Pitfalls" section: a client may not find `serena` even when it is on `PATH`. |
| `SERENA_USAGE_REPORTING=false` | Opts out of the anonymous start-up ping described in the [privacy policy](https://oraios.github.io/serena/01-about/070_privacy.html). |
| separate `CARGO_TARGET_DIR` | rust-analyzer runs build scripts and `cargo check` on save. A separate target dir keeps those runs from taking the cargo lock on the shared build target. |

Changes to `~/.serena/serena_config.yml` from the `serena init` defaults
([configuration docs](https://oraios.github.io/serena/02-usage/050_configuration.html)):

| Setting | Why |
| --- | --- |
| `agent_interface: tools` | The default REPL interface is beta, ignores tool exclusions (including the `codex` context's), and exposes shell and filesystem access outside Codex's sandbox. The [security page](https://oraios.github.io/serena/02-usage/070_security.html) says restrictions "are effective for the tool interface only". |
| `web_dashboard_open_on_launch: false` | The box is headless. The [dashboard docs](https://oraios.github.io/serena/02-usage/060_dashboard.html) recommend keeping the dashboard on, so it still listens on `127.0.0.1:24282` and up. |
| `trusted_project_path_patterns` | Lists the main checkout and the per-ticket worktree glob so job worktrees activate without a prompt. |

Serena also documents a Codex `hooks.json` (reminders, plan guard, session cleanup).
It is not installed: hooks need Calum's approval (CONTRIBUTING.md).

## When to use which tool

See "Symbol navigation" in AGENTS.md.

## Measurements (box: 8 CPUs, 15 GB RAM, no swap; 29 Oct 2026)

| Step | Wall time | Peak RSS |
| --- | --- | --- |
| `serena project index`, cold | about 18 s (rust-analyzer ready in 14.5 s, 417 files indexed in 2.6 s) | 1.8 GB for the whole process tree, 786 MB for rust-analyzer |
| `serena project index`, warm | 3.3 s | 1.0 GB |
| MCP start-up and `find_symbol` | 0.9 s to start, 0.1–0.3 s per call | about 1.2 GB |
| first `find_referencing_symbols` in a session | 31–39 s | rust-analyzer grows to **3.8 GB** (4–5 GB for the tree) and stays there for the session |
| later reference lookups | 0.1–0.2 s | same |

Accuracy compared with `rg -w`:

- `EditorError`: `find_symbol` returns the one enum. References: 95 lines in 15 files,
  against 89 `rg` lines in 15 files. All 85 non-definition uses that rg finds are found, plus 10 `Self::…`
  uses inside the `From` impls that rg cannot attribute. The three
  `impl From<…> for EditorError` header lines are missing.
- `mesh_dirty`: one definition and one call site (`chunk_mesh.rs:104`), the same as rg.
  `mesh_dirty_with_pool` is not confused with it.
- `build_design`: all 5 definitions (FieldFrame, Swatch, Button, Panel, Tab). Each one's
  references give exactly its own call site. rg gives 10 lines and cannot say which call
  goes to which impl.

A `codex exec` check in a job worktree used `find_symbol`, `find_referencing_symbols` and
`search_for_pattern` and answered `chunk_mesh.rs:211` / `chunk_mesh.rs:104`.

**Memory caveat:** each Codex session starts its own Serena and rust-analyzer. With
about 4 GB per session after the first reference lookup, and about 12 GB free on the
box, more than two concurrent Serena-enabled jobs plus cargo builds can run out of
memory (there is no swap). Serena's HTTP mode, which lets agents share one server, is
the documented alternative ([workflow docs](https://oraios.github.io/serena/02-usage/040_workflow.html)).
