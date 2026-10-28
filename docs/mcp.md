# MCP tools

Coordinates in. Models out. Real visual feedback.

This reference is generated from the Rust server's live discovery catalog, API `quantum.v1`. Download the [complete JSON catalog](https://quantum.worldwidewebb.co/assets/mcp-tools.json). Run `quantum-mcp --schema` to inspect the catalog for your own build.

## Install MCP

[Install Quantum for Mac](https://quantum.worldwidewebb.co/download/) in Applications first. The app includes the MCP server. Open your project folder in your host, then choose an install method below. For [source builds](https://quantum.worldwidewebb.co/docs/getting-started/#build) or a different app location, use the manual configuration with your executable's absolute path.

### Cursor

[Add to Cursor](cursor://anysphere.cursor-deeplink/mcp/install?name=quantum&config=eyJjb21tYW5kIjoiL0FwcGxpY2F0aW9ucy9RdWFudHVtLmFwcC9Db250ZW50cy9NYWNPUy9xdWFudHVtLW1jcCIsImFyZ3MiOlsiLS1yb290IiwiJHt3b3Jrc3BhY2VGb2xkZXJ9Il19)

Review and confirm the server in Cursor. The file root is `${workspaceFolder}`, your open project folder. [View mcp.json](https://quantum.worldwidewebb.co/assets/integrations/cursor/mcp.json) for manual setup.

### Claude Code

Run from your project directory; `"$PWD"` sets the file root to that directory.

```sh
claude mcp add quantum -- '/Applications/Quantum.app/Contents/MacOS/quantum-mcp' --root "$PWD"
```

To make the server available across projects, add `--scope user` after `claude mcp add`. Its file root still points to the directory where you ran the command.

### Codex

Run from your project directory; `"$PWD"` sets the file root to that directory.

```sh
codex mcp add quantum -- '/Applications/Quantum.app/Contents/MacOS/quantum-mcp' --root "$PWD"
```

Or merge this into `~/.codex/config.toml` (or your trusted project's `.codex/config.toml`). Replace `<project dir>` with an existing absolute path.

```toml
[mcp_servers.quantum]
command = "/Applications/Quantum.app/Contents/MacOS/quantum-mcp"
args = ["--root","<project dir>"]
```

### VS Code

[Install in VS Code](vscode:mcp/install?%7B%22name%22%3A%22quantum%22%2C%22command%22%3A%22%2FApplications%2FQuantum.app%2FContents%2FMacOS%2Fquantum-mcp%22%2C%22args%22%3A%5B%22--root%22%2C%22%24%7BworkspaceFolder%7D%22%5D%7D)

Review and confirm the server in VS Code, or run this CLI command. Both use `${workspaceFolder}` for the file root, resolved by VS Code in your open project.

```sh
code --add-mcp '{"name":"quantum","command":"/Applications/Quantum.app/Contents/MacOS/quantum-mcp","args":["--root","${workspaceFolder}"]}'
```

[View .vscode/mcp.json](https://quantum.worldwidewebb.co/assets/integrations/vscode/mcp.json) for manual setup.

### Other

For Claude Desktop, Windsurf and other MCP hosts, merge this entry into the host's MCP settings. Preserve existing servers and replace `<project dir>` with an existing absolute path.

```json
{
  "mcpServers": {
    "quantum": {
      "command": "/Applications/Quantum.app/Contents/MacOS/quantum-mcp",
      "args": [
        "--root",
        "<project dir>"
      ]
    }
  }
}
```

## Install skills

Install all skills from the repository's `skills/` directory with the open Agent Skills CLI. This installs workflow instructions; configure MCP above to give them tools.

```sh
npx skills add 0x63616c/quantum
```

Add `--skill <name>` for one skill, `-g` for a global install, or `-a claude-code`, `-a cursor` or `-a codex` to pick an agent. [Choose an individual skill](https://quantum.worldwidewebb.co/docs/skills/#available-skills).

For a manual install, [download the skill folders](https://quantum.worldwidewebb.co/assets/quantum-skills.zip) or use a repository checkout. Copy each `skills/<name>/` folder into `.cursor/skills/`, `.claude/skills/`, or `~/.codex/skills/`. [Install, update and removal notes](https://quantum.worldwidewebb.co/assets/skills/README.md).

## What you get

MCP gives your agent voxel editing tools, palettes, real PNG renders, and project save/export. Skills teach it to compose, inspect and refine a model. Each standalone server owns its own session; save `.voxl` files before disconnecting. To edit the live native canvas, [copy the connection configuration from the app](https://quantum.worldwidewebb.co/docs/getting-started/#connect).

## Verify it works

1.  Restart or reconnect your host and list its MCP tools.
2.  Call `quantum_capabilities_get` with `{}`. Expect API `quantum.v1`, an epoch and available document IDs.
3.  Confirm the installed skills appear in your host, then ask it to use `quantum-workflow` for your first model.

## Shared contract

*   Stdio uses one JSON-RPC message per line. Stdout contains protocol messages only; diagnostics go to stderr.
*   Supported protocol profiles: `2025-11-25` (initialize and initialized notification) and `2026-07-28` (per-request protocol metadata).
*   Discover the epoch with `quantum_capabilities_get`. Writes use captured `expected_epoch`, a decimal-string `expected_revision` where required, and a fresh UUID `operation_id`.
*   Coordinate order is `[x, y, z]`, right-handed, Y up. Regions include `min` and exclude `max`. Slots are stable IDs, not display positions; zero means air only where the schema allows it.
*   Prefer layer UUIDs and stable slot numbers. Names must resolve uniquely. Unknown fields are rejected, including nested fields.
*   Atomic batches commit one undo entry or nothing. Content-edit results report revision, changed bounds/counts and history IDs. Check the result before updating your local state.
*   The same success JSON appears in `structuredContent` and the first text block. It contains `api_version`, `session_id`, `epoch`, `ok`, and `data` or `error`.

The attached native catalog omits `document_create`, `document_open`, and `document_save`. Use the native UI for file lifecycle. No screenshot-named tool or ellipsoid/cylinder tool is currently exposed: use `quantum_render` for model images and explicit voxel batches for other shapes.

## Render feedback

Default views are iso/front/right/back at 512 × 512 each, returned as a 1024 × 1024 contact sheet. `image_mode` can return individual images, a sheet, or both. Images are actual PNG MCP image blocks, never a simulated preview.

At most six views, 1024 × 1024 per view and 4,194,304 combined view/sheet pixels are allowed, with a 12 MiB encoded response budget. Front looks from +Z, right from +X, top from +Y with −Z up. Explicit cameras and named views are mutually exclusive. Lower dimensions or view count after a budget error.

## Errors, retries and files

| Code                                    | Next step                                                                         |
| --------------------------------------- | --------------------------------------------------------------------------------- |
| `revision_conflict`, `epoch_conflict`   | Read the current model/session, rebase the intended edit, use a new operation ID. |
| `operation_id_reused`                   | Recover the original request. A retry must have identical tool, arguments and ID. |
| `layer_blocked`                         | Confirm the target layer; make it visible or editable explicitly if intended.     |
| `history_conflict`                      | Read history and capture the exact current head/redo ID.                          |
| `invalid_argument`, `out_of_bounds`     | Correct the indicated field or half-open bounds.                                  |
| `limit_exceeded`                        | Reduce batch/page/render size. If receipts are exhausted, save and restart.       |
| `external_file_conflict`, `file_exists` | Preserve externally changed content; choose a new output path.                    |
| `path_outside_root`                     | Use a relative path within the configured file root. Parents must exist.          |
| `outcome_unknown`                       | Inspect the destination: rename may have succeeded before directory sync failed.  |

A domain error returns `isError: true` and `ok: false`, with a recovery hint. Transport success alone does not mean an edit succeeded. Cancellation does not undo an already committed edit. Exact retries replay successful receipts only within the current process.

File tools are confined to `--root`. New files are create-only. Saving over a previously opened/saved file requires its recorded hash to match. Save uses a sibling temporary file, sync and atomic installation. Export does not mark a document saved. Undo/retry state is session-local; comments and named versions persist in `.voxl`.

## All 38 tools

Generated directly from `QuantumServer::tools()`. Expand a schema for nested properties, defaults, limits, conditional requirements and the output envelope. A property marked optional may still be required by a schema condition.

- [capabilities\_get](https://quantum.worldwidewebb.co/docs/mcp/#quantum_capabilities_get)
- [comments\_add](https://quantum.worldwidewebb.co/docs/mcp/#quantum_comments_add)
- [comments\_list](https://quantum.worldwidewebb.co/docs/mcp/#quantum_comments_list)
- [comments\_reopen](https://quantum.worldwidewebb.co/docs/mcp/#quantum_comments_reopen)
- [comments\_reply](https://quantum.worldwidewebb.co/docs/mcp/#quantum_comments_reply)
- [comments\_resolve](https://quantum.worldwidewebb.co/docs/mcp/#quantum_comments_resolve)
- [document\_create](https://quantum.worldwidewebb.co/docs/mcp/#quantum_document_create)
- [document\_info](https://quantum.worldwidewebb.co/docs/mcp/#quantum_document_info)
- [document\_open](https://quantum.worldwidewebb.co/docs/mcp/#quantum_document_open)
- [document\_save](https://quantum.worldwidewebb.co/docs/mcp/#quantum_document_save)
- [export\_glb](https://quantum.worldwidewebb.co/docs/mcp/#quantum_export_glb)
- [export\_png](https://quantum.worldwidewebb.co/docs/mcp/#quantum_export_png)
- [history\_read](https://quantum.worldwidewebb.co/docs/mcp/#quantum_history_read)
- [history\_redo](https://quantum.worldwidewebb.co/docs/mcp/#quantum_history_redo)
- [history\_undo](https://quantum.worldwidewebb.co/docs/mcp/#quantum_history_undo)
- [layers\_add](https://quantum.worldwidewebb.co/docs/mcp/#quantum_layers_add)
- [layers\_list](https://quantum.worldwidewebb.co/docs/mcp/#quantum_layers_list)
- [layers\_lock](https://quantum.worldwidewebb.co/docs/mcp/#quantum_layers_lock)
- [layers\_remove](https://quantum.worldwidewebb.co/docs/mcp/#quantum_layers_remove)
- [layers\_rename](https://quantum.worldwidewebb.co/docs/mcp/#quantum_layers_rename)
- [layers\_reorder](https://quantum.worldwidewebb.co/docs/mcp/#quantum_layers_reorder)
- [layers\_visibility](https://quantum.worldwidewebb.co/docs/mcp/#quantum_layers_visibility)
- [palette\_apply\_preset](https://quantum.worldwidewebb.co/docs/mcp/#quantum_palette_apply_preset)
- [palette\_export](https://quantum.worldwidewebb.co/docs/mcp/#quantum_palette_export)
- [palette\_import](https://quantum.worldwidewebb.co/docs/mcp/#quantum_palette_import)
- [palette\_list](https://quantum.worldwidewebb.co/docs/mcp/#quantum_palette_list)
- [palette\_set\_slot](https://quantum.worldwidewebb.co/docs/mcp/#quantum_palette_set_slot)
- [query](https://quantum.worldwidewebb.co/docs/mcp/#quantum_query)
- [render](https://quantum.worldwidewebb.co/docs/mcp/#quantum_render)
- [versions\_list](https://quantum.worldwidewebb.co/docs/mcp/#quantum_versions_list)
- [versions\_restore](https://quantum.worldwidewebb.co/docs/mcp/#quantum_versions_restore)
- [versions\_save](https://quantum.worldwidewebb.co/docs/mcp/#quantum_versions_save)
- [voxels\_batch](https://quantum.worldwidewebb.co/docs/mcp/#quantum_voxels_batch)
- [voxels\_box](https://quantum.worldwidewebb.co/docs/mcp/#quantum_voxels_box)
- [voxels\_erase\_region](https://quantum.worldwidewebb.co/docs/mcp/#quantum_voxels_erase_region)
- [voxels\_fill](https://quantum.worldwidewebb.co/docs/mcp/#quantum_voxels_fill)
- [voxels\_get](https://quantum.worldwidewebb.co/docs/mcp/#quantum_voxels_get)
- [voxels\_set](https://quantum.worldwidewebb.co/docs/mcp/#quantum_voxels_set)

### quantum\_capabilities\_get

Discover quantum.v1, session epoch, limits, CPU renderer, exact palette presets and open model IDs. Start here; use returned epoch and revisions for edits.

| Argument | Required | Type / constraints |
| -------- | -------- | ------------------ |

Complete input schema

```json
{
  "type": "object",
  "properties": {},
  "required": [],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "api_version": {
          "const": "quantum.v1"
        },
        "protocol_versions": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": [
              "2025-11-25",
              "2026-07-28"
            ]
          },
          "minItems": 2,
          "maxItems": 2
        },
        "limits": {
          "type": "object",
          "properties": {
            "open_documents": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            "request_bytes": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            "result_bytes": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            "batch_operations": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            "explicit_voxels": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            "read_voxels": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            "work_units": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            "operation_receipts": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            "receipt_bytes": {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            }
          },
          "required": [
            "open_documents",
            "request_bytes",
            "result_bytes",
            "batch_operations",
            "explicit_voxels",
            "read_voxels",
            "work_units",
            "operation_receipts",
            "receipt_bytes"
          ],
          "additionalProperties": true
        },
        "documents": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "model_id": {
                "type": "string",
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
              },
              "revision": {
                "type": "string",
                "pattern": "^(0|[1-9][0-9]{0,19})$",
                "description": "Captured monotonic revision, encoded as a decimal u64 string."
              },
              "name": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
              },
              "bounds": {
                "type": "object",
                "properties": {
                  "min": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -32768,
                      "maximum": 32767
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "max": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -32767,
                      "maximum": 32768
                    },
                    "minItems": 3,
                    "maxItems": 3
                  }
                },
                "required": [
                  "min",
                  "max"
                ],
                "additionalProperties": true,
                "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
              },
              "layers": {
                "type": "array",
                "items": {
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "string",
                      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                    },
                    "name": {
                      "type": "string",
                      "minLength": 1,
                      "maxLength": 128,
                      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
                    },
                    "visible": {
                      "type": "boolean"
                    },
                    "locked": {
                      "type": "boolean"
                    },
                    "voxel_count": {
                      "type": "integer",
                      "minimum": 0,
                      "maximum": 9223372036854775807
                    }
                  },
                  "required": [
                    "id",
                    "name",
                    "visible",
                    "locked",
                    "voxel_count"
                  ],
                  "additionalProperties": true
                },
                "minItems": 1,
                "maxItems": 64
              },
              "dirty": {
                "type": "boolean"
              },
              "voxel_count": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              },
              "palette_slots": {
                "type": "integer",
                "minimum": 1,
                "maximum": 256
              },
              "comments": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              },
              "versions": {
                "type": "integer",
                "minimum": 0,
                "maximum": 64
              },
              "head_id": {
                "anyOf": [
                  {
                    "type": "string",
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "redo_id": {
                "anyOf": [
                  {
                    "type": "string",
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                  },
                  {
                    "type": "null"
                  }
                ]
              }
            },
            "required": [
              "model_id",
              "revision",
              "name",
              "bounds",
              "layers",
              "dirty",
              "voxel_count",
              "palette_slots",
              "comments",
              "versions",
              "head_id",
              "redo_id"
            ],
            "additionalProperties": true
          },
          "minItems": 0,
          "maxItems": 8
        },
        "renderer": {
          "const": "cpu"
        },
        "formats": {
          "type": "array",
          "items": {
            "type": "string",
            "minLength": 1,
            "maxLength": 16
          },
          "minItems": 1,
          "maxItems": 6
        },
        "presets": {
          "type": "array",
          "items": {
            "type": "string",
            "minLength": 1,
            "maxLength": 32
          },
          "minItems": 1,
          "maxItems": 7
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "api_version",
        "protocol_versions",
        "limits",
        "documents",
        "renderer",
        "formats",
        "presets",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_comments\_add

Create a persistent voxel/region anchored thread. Author is always agent, never caller-supplied. Text is plain text, max 4096 UTF-8 bytes. Returns thread/message IDs; undoable and saved in .voxl.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `anchor`            | Yes      | See schema                                                                                                                           |
| `layer`             | Optional | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `text`              | Yes      | string; maxLength: 4096                                                                                                              |
| `thread_id`         | Optional | string                                                                                                                               |
| `message_id`        | Optional | string                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "anchor": {
      "oneOf": [
        {
          "type": "object",
          "properties": {
            "kind": {
              "const": "voxel"
            },
            "at": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "kind",
            "at"
          ],
          "additionalProperties": false
        },
        {
          "type": "object",
          "properties": {
            "kind": {
              "const": "region"
            },
            "min": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            },
            "max": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32767,
                "maximum": 32768
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "kind",
            "min",
            "max"
          ],
          "additionalProperties": false
        }
      ]
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "text": {
      "type": "string",
      "minLength": 1,
      "maxLength": 4096
    },
    "thread_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "message_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "anchor",
    "text"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        },
        "thread_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "message_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability",
        "thread_id",
        "message_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_comments\_list

List anchored threads including messages, agent/human roles, resolved state and anchor status. Optional state=open|resolved|all (default all). Thread and message pages default to 50, max 200. message\_offset applies per thread; bind pages with revision.

| Argument         | Required | Type / constraints                                                    |
| ---------------- | -------- | --------------------------------------------------------------------- |
| `model_id`       | Yes      | string                                                                |
| `revision`       | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |
| `state`          | Optional | string; enum: \["open","resolved","all"\]                             |
| `offset`         | Optional | integer; minimum: 0; maximum: 1024                                    |
| `limit`          | Optional | integer; minimum: 1; maximum: 200                                     |
| `message_offset` | Optional | integer; minimum: 0; maximum: 10000                                   |
| `message_limit`  | Optional | integer; minimum: 1; maximum: 200                                     |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "state": {
      "type": "string",
      "enum": [
        "open",
        "resolved",
        "all"
      ]
    },
    "offset": {
      "type": "integer",
      "minimum": 0,
      "maximum": 1024
    },
    "limit": {
      "type": "integer",
      "minimum": 1,
      "maximum": 200
    },
    "message_offset": {
      "type": "integer",
      "minimum": 0,
      "maximum": 10000
    },
    "message_limit": {
      "type": "integer",
      "minimum": 1,
      "maximum": 200
    }
  },
  "required": [
    "model_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "comments": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "thread": {
                "type": "object",
                "properties": {
                  "id": {
                    "type": "string",
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                  },
                  "anchor": {
                    "oneOf": [
                      {
                        "type": "object",
                        "properties": {
                          "kind": {
                            "const": "voxel"
                          },
                          "at": {
                            "type": "array",
                            "items": {
                              "type": "integer",
                              "minimum": -32768,
                              "maximum": 32767
                            },
                            "minItems": 3,
                            "maxItems": 3
                          }
                        },
                        "required": [
                          "kind",
                          "at"
                        ],
                        "additionalProperties": true
                      },
                      {
                        "type": "object",
                        "properties": {
                          "kind": {
                            "const": "region"
                          },
                          "min": {
                            "type": "array",
                            "items": {
                              "type": "integer",
                              "minimum": -32768,
                              "maximum": 32767
                            },
                            "minItems": 3,
                            "maxItems": 3
                          },
                          "max": {
                            "type": "array",
                            "items": {
                              "type": "integer",
                              "minimum": -32767,
                              "maximum": 32768
                            },
                            "minItems": 3,
                            "maxItems": 3
                          }
                        },
                        "required": [
                          "kind",
                          "min",
                          "max"
                        ],
                        "additionalProperties": true
                      }
                    ]
                  },
                  "layer": {
                    "anyOf": [
                      {
                        "type": "string",
                        "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                      },
                      {
                        "type": "null"
                      }
                    ]
                  },
                  "resolved": {
                    "type": "boolean"
                  },
                  "messages": {
                    "type": "array",
                    "items": {
                      "type": "object",
                      "properties": {
                        "id": {
                          "type": "string",
                          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                        },
                        "origin": {
                          "type": "string",
                          "enum": [
                            "agent",
                            "human"
                          ]
                        },
                        "text": {
                          "type": "string",
                          "minLength": 1,
                          "maxLength": 8192
                        },
                        "tool": {
                          "anyOf": [
                            {
                              "type": "string",
                              "minLength": 1,
                              "maxLength": 128,
                              "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                              "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
                            },
                            {
                              "type": "null"
                            }
                          ]
                        }
                      },
                      "required": [
                        "id",
                        "origin",
                        "text",
                        "tool"
                      ],
                      "additionalProperties": true
                    },
                    "minItems": 0,
                    "maxItems": 200
                  }
                },
                "required": [
                  "id",
                  "anchor",
                  "layer",
                  "resolved",
                  "messages"
                ],
                "additionalProperties": true
              },
              "message_count": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              },
              "next_message_offset": {
                "anyOf": [
                  {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 9223372036854775807
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "anchor_state": {
                "type": "string",
                "enum": [
                  "visible",
                  "hidden",
                  "orphaned"
                ]
              }
            },
            "required": [
              "thread",
              "message_count",
              "next_message_offset",
              "anchor_state"
            ],
            "additionalProperties": true
          },
          "minItems": 0,
          "maxItems": 200
        },
        "total": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "next_offset": {
          "anyOf": [
            {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "comments",
        "total",
        "next_offset"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_comments\_reopen

Reopen a resolved discussion; idempotent if already open. Preserves messages and anchor; undoable.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `thread_id`         | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "thread_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "thread_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_comments\_reply

Append an agent-attributed plain-text reply to a thread, resolved or open. One undoable edit; returns message ID.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `thread_id`         | Yes      | string                                                                                                                               |
| `text`              | Yes      | string; maxLength: 4096                                                                                                              |
| `message_id`        | Optional | string                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "thread_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "text": {
      "type": "string",
      "minLength": 1,
      "maxLength": 4096
    },
    "message_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "thread_id",
    "text"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        },
        "thread_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "message_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability",
        "thread_id",
        "message_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_comments\_resolve

Resolve an anchored discussion; idempotent if already resolved. Preserves messages and anchor; undoable.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `thread_id`         | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "thread_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "thread_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_document\_create

Create a revision-zero document and visible unlocked Base layer. Default bounds \[0,0,0\]..\[32,32,32\]. Returns model/layer IDs. Up to eight documents per session; never replaces an open model.

| Argument         | Required | Type / constraints                                                                                                                     |
| ---------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch` | Yes      | string                                                                                                                                 |
| `operation_id`   | Yes      | string                                                                                                                                 |
| `label`          | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `name`           | Yes      | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `bounds`         | Optional | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.          |
| `model_id`       | Optional | string                                                                                                                                 |
| `layer_id`       | Optional | string                                                                                                                                 |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "bounds": {
      "type": "object",
      "properties": {
        "min": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32768,
            "maximum": 32767
          },
          "minItems": 3,
          "maxItems": 3
        },
        "max": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32767,
            "maximum": 32768
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "min",
        "max"
      ],
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "layer_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "name"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "name": {
          "type": "string",
          "minLength": 1,
          "maxLength": 128,
          "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
          "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
        },
        "bounds": {
          "type": "object",
          "properties": {
            "min": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            },
            "max": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32767,
                "maximum": 32768
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "min",
            "max"
          ],
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
        },
        "layers": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "id": {
                "type": "string",
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
              },
              "name": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
              },
              "visible": {
                "type": "boolean"
              },
              "locked": {
                "type": "boolean"
              },
              "voxel_count": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              }
            },
            "required": [
              "id",
              "name",
              "visible",
              "locked",
              "voxel_count"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 64
        },
        "dirty": {
          "type": "boolean"
        },
        "voxel_count": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "palette_slots": {
          "type": "integer",
          "minimum": 1,
          "maximum": 256
        },
        "comments": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "versions": {
          "type": "integer",
          "minimum": 0,
          "maximum": 64
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "redo_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "name",
        "bounds",
        "layers",
        "dirty",
        "voxel_count",
        "palette_slots",
        "comments",
        "versions",
        "head_id",
        "redo_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_document\_info

Read document identity, bounds, layers, counts, dirty state and current revision. Optional revision must equal the live revision; no UI state is used.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    }
  },
  "required": [
    "model_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "name": {
          "type": "string",
          "minLength": 1,
          "maxLength": 128,
          "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
          "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
        },
        "bounds": {
          "type": "object",
          "properties": {
            "min": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            },
            "max": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32767,
                "maximum": 32768
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "min",
            "max"
          ],
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
        },
        "layers": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "id": {
                "type": "string",
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
              },
              "name": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
              },
              "visible": {
                "type": "boolean"
              },
              "locked": {
                "type": "boolean"
              },
              "voxel_count": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              }
            },
            "required": [
              "id",
              "name",
              "visible",
              "locked",
              "voxel_count"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 64
        },
        "dirty": {
          "type": "boolean"
        },
        "voxel_count": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "palette_slots": {
          "type": "integer",
          "minimum": 1,
          "maximum": 256
        },
        "comments": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "versions": {
          "type": "integer",
          "minimum": 0,
          "maximum": 64
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "redo_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "name",
        "bounds",
        "layers",
        "dirty",
        "voxel_count",
        "palette_slots",
        "comments",
        "versions",
        "head_id",
        "redo_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_document\_open

Open a canonical .voxl file relative to the configured project root, preserving IDs, comments, versions and unknown metadata. Validation finishes before installation. Duplicate model IDs reject.

| Argument         | Required | Type / constraints                                                                                                                   |
| ---------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch` | Yes      | string                                                                                                                               |
| `operation_id`   | Yes      | string                                                                                                                               |
| `label`          | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `path`           | Yes      | string; maxLength: 1024                                                                                                              |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "path": {
      "type": "string",
      "minLength": 1,
      "maxLength": 1024
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "path"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "name": {
          "type": "string",
          "minLength": 1,
          "maxLength": 128,
          "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
          "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
        },
        "bounds": {
          "type": "object",
          "properties": {
            "min": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            },
            "max": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32767,
                "maximum": 32768
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "min",
            "max"
          ],
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
        },
        "layers": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "id": {
                "type": "string",
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
              },
              "name": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
              },
              "visible": {
                "type": "boolean"
              },
              "locked": {
                "type": "boolean"
              },
              "voxel_count": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              }
            },
            "required": [
              "id",
              "name",
              "visible",
              "locked",
              "voxel_count"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 64
        },
        "dirty": {
          "type": "boolean"
        },
        "voxel_count": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "palette_slots": {
          "type": "integer",
          "minimum": 1,
          "maximum": 256
        },
        "comments": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "versions": {
          "type": "integer",
          "minimum": 0,
          "maximum": 64
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "redo_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "name",
        "bounds",
        "layers",
        "dirty",
        "voxel_count",
        "palette_slots",
        "comments",
        "versions",
        "head_id",
        "redo_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_document\_save

Atomically save .voxl under the project root. New paths must not exist; overwriting the same opened/saved path checks its last SHA-256. Does not create an undo entry. Changes stay in memory until saved.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `path`              | Yes      | string; maxLength: 1024                                                                                                              |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "path": {
      "type": "string",
      "minLength": 1,
      "maxLength": 1024
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "path"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "path": {
          "type": "string",
          "minLength": 1,
          "maxLength": 1024
        },
        "bytes": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "sha256": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$"
        },
        "dirty": {
          "const": false
        }
      },
      "required": [
        "model_id",
        "revision",
        "path",
        "bytes",
        "sha256",
        "dirty"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_export\_glb

Export deterministic GLB of the highest visible layer at each occupied coordinate. Atomically write a new path under project root; existing paths reject. Returns size and SHA-256; document/history unchanged.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |
| `path`     | Yes      | string; maxLength: 1024                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "path": {
      "type": "string",
      "minLength": 1,
      "maxLength": 1024
    }
  },
  "required": [
    "model_id",
    "path"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "path": {
          "type": "string",
          "minLength": 1,
          "maxLength": 1024
        },
        "bytes": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "sha256": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$"
        },
        "mime_type": {
          "const": "model/gltf-binary"
        }
      },
      "required": [
        "model_id",
        "revision",
        "path",
        "bytes",
        "sha256",
        "mime_type"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_export\_png

Render and atomically write one PNG under project root. Default contact sheet; for individual image use one view. Existing paths reject. Returns size, SHA-256 and camera metadata; document/history unchanged.

| Argument                     | Required | Type / constraints                                                                                                                                                              |
| ---------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `model_id`                   | Yes      | string                                                                                                                                                                          |
| `revision`                   | Optional | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                                                           |
| `views`                      | Optional | array; minItems: 1; maxItems: 6                                                                                                                                                 |
| `camera`                     | Optional | object                                                                                                                                                                          |
| `width`                      | Optional | integer; minimum: 1; maximum: 1024                                                                                                                                              |
| `height`                     | Optional | integer; minimum: 1; maximum: 1024                                                                                                                                              |
| `perspective`                | Optional | boolean                                                                                                                                                                         |
| `background`                 | Optional | array; minItems: 4; maxItems: 4                                                                                                                                                 |
| `ambient_occlusion_strength` | Optional | number; default: 1; minimum: 0; maximum: 2. Local voxel contact shading: 0 disables AO, 1 preserves the default look, 2 doubles darkening. Independent of viewport preferences. |
| `image_mode`                 | Optional | string; enum: \["contact\_sheet","individual"\]                                                                                                                                 |
| `path`                       | Yes      | string; maxLength: 1024                                                                                                                                                         |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "views": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "iso",
          "front",
          "back",
          "left",
          "right",
          "top"
        ]
      },
      "minItems": 1,
      "maxItems": 6
    },
    "camera": {
      "type": "object",
      "properties": {
        "eye": {
          "type": "array",
          "items": {
            "type": "number",
            "minimum": -1000000000.0,
            "maximum": 1000000000.0
          },
          "minItems": 3,
          "maxItems": 3
        },
        "target": {
          "type": "array",
          "items": {
            "type": "number",
            "minimum": -1000000000.0,
            "maximum": 1000000000.0
          },
          "minItems": 3,
          "maxItems": 3
        },
        "up": {
          "type": "array",
          "items": {
            "type": "number",
            "minimum": -1000000000.0,
            "maximum": 1000000000.0
          },
          "minItems": 3,
          "maxItems": 3
        },
        "projection": {
          "type": "string",
          "enum": [
            "orthographic",
            "perspective"
          ]
        },
        "ortho_height": {
          "type": "number",
          "exclusiveMinimum": 0,
          "maximum": 1000000000.0
        },
        "fov_y_degrees": {
          "type": "number",
          "exclusiveMinimum": 1,
          "exclusiveMaximum": 179
        }
      },
      "required": [
        "eye",
        "target",
        "up",
        "projection"
      ],
      "additionalProperties": false,
      "oneOf": [
        {
          "properties": {
            "projection": {
              "const": "orthographic"
            }
          },
          "required": [
            "ortho_height"
          ],
          "not": {
            "required": [
              "fov_y_degrees"
            ]
          }
        },
        {
          "properties": {
            "projection": {
              "const": "perspective"
            }
          },
          "required": [
            "fov_y_degrees"
          ],
          "not": {
            "required": [
              "ortho_height"
            ]
          }
        }
      ]
    },
    "width": {
      "type": "integer",
      "minimum": 1,
      "maximum": 1024
    },
    "height": {
      "type": "integer",
      "minimum": 1,
      "maximum": 1024
    },
    "perspective": {
      "type": "boolean"
    },
    "background": {
      "type": "array",
      "items": {
        "type": "integer",
        "minimum": 0,
        "maximum": 255
      },
      "minItems": 4,
      "maxItems": 4
    },
    "ambient_occlusion_strength": {
      "type": "number",
      "minimum": 0,
      "maximum": 2,
      "default": 1,
      "description": "Local voxel contact shading: 0 disables AO, 1 preserves the default look, 2 doubles darkening. Independent of viewport preferences."
    },
    "image_mode": {
      "type": "string",
      "enum": [
        "contact_sheet",
        "individual"
      ]
    },
    "path": {
      "type": "string",
      "minLength": 1,
      "maxLength": 1024
    }
  },
  "required": [
    "model_id",
    "path"
  ],
  "additionalProperties": false,
  "not": {
    "required": [
      "camera"
    ],
    "anyOf": [
      {
        "required": [
          "views"
        ]
      },
      {
        "required": [
          "perspective"
        ]
      }
    ]
  },
  "if": {
    "properties": {
      "image_mode": {
        "const": "individual"
      }
    },
    "required": [
      "image_mode"
    ]
  },
  "then": {
    "anyOf": [
      {
        "required": [
          "camera"
        ]
      },
      {
        "required": [
          "views"
        ],
        "properties": {
          "views": {
            "maxItems": 1
          }
        }
      }
    ]
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "path": {
          "type": "string",
          "minLength": 1,
          "maxLength": 1024
        },
        "bytes": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "sha256": {
          "type": "string",
          "pattern": "^[0-9a-f]{64}$"
        },
        "backend": {
          "const": "cpu"
        },
        "bounds": {
          "type": "object",
          "properties": {
            "min": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            },
            "max": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32767,
                "maximum": 32768
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "min",
            "max"
          ],
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
        },
        "views": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "view": {
                "type": "string",
                "enum": [
                  "iso",
                  "front",
                  "back",
                  "left",
                  "right",
                  "top",
                  "camera",
                  "contact_sheet"
                ]
              },
              "width": {
                "type": "integer",
                "minimum": 1,
                "maximum": 2048
              },
              "height": {
                "type": "integer",
                "minimum": 1,
                "maximum": 2048
              },
              "sha256": {
                "type": "string",
                "pattern": "^[0-9a-f]{64}$"
              },
              "bytes": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              },
              "camera": {
                "type": "object",
                "properties": {
                  "eye": {
                    "type": "array",
                    "items": {
                      "type": "number",
                      "minimum": -1000000000.0,
                      "maximum": 1000000000.0
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "target": {
                    "type": "array",
                    "items": {
                      "type": "number",
                      "minimum": -1000000000.0,
                      "maximum": 1000000000.0
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "up": {
                    "type": "array",
                    "items": {
                      "type": "number",
                      "minimum": -1000000000.0,
                      "maximum": 1000000000.0
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "projection": {
                    "type": "string",
                    "enum": [
                      "orthographic",
                      "perspective"
                    ]
                  },
                  "ortho_height": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "maximum": 1000000000.0
                  },
                  "fov_y_degrees": {
                    "type": "number",
                    "exclusiveMinimum": 1,
                    "exclusiveMaximum": 179
                  }
                },
                "required": [
                  "eye",
                  "target",
                  "up",
                  "projection"
                ],
                "additionalProperties": true,
                "oneOf": [
                  {
                    "properties": {
                      "projection": {
                        "const": "orthographic"
                      }
                    },
                    "required": [
                      "ortho_height"
                    ],
                    "not": {
                      "required": [
                        "fov_y_degrees"
                      ]
                    }
                  },
                  {
                    "properties": {
                      "projection": {
                        "const": "perspective"
                      }
                    },
                    "required": [
                      "fov_y_degrees"
                    ],
                    "not": {
                      "required": [
                        "ortho_height"
                      ]
                    }
                  }
                ]
              }
            },
            "required": [
              "view",
              "width",
              "height",
              "sha256",
              "bytes",
              "camera"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 6
        },
        "images": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "view": {
                "type": "string",
                "enum": [
                  "iso",
                  "front",
                  "back",
                  "left",
                  "right",
                  "top",
                  "camera",
                  "contact_sheet"
                ]
              },
              "width": {
                "type": "integer",
                "minimum": 1,
                "maximum": 2048
              },
              "height": {
                "type": "integer",
                "minimum": 1,
                "maximum": 2048
              },
              "sha256": {
                "type": "string",
                "pattern": "^[0-9a-f]{64}$"
              },
              "bytes": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              }
            },
            "required": [
              "view",
              "width",
              "height",
              "sha256",
              "bytes"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 7
        },
        "image_mode": {
          "type": "string",
          "enum": [
            "contact_sheet",
            "individual",
            "both"
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "path",
        "bytes",
        "sha256",
        "backend",
        "bounds",
        "views",
        "images",
        "image_mode"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_history\_read

Read chronological applied history with agent origins, tool names, labels and change summaries. Includes exact undo/redo entry IDs. Default page 50, max 200; history is bounded and resets on reopen.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |
| `offset`   | Optional | integer; minimum: 0; maximum: 1048576                                 |
| `limit`    | Optional | integer; minimum: 1; maximum: 200                                     |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "offset": {
      "type": "integer",
      "minimum": 0,
      "maximum": 1048576
    },
    "limit": {
      "type": "integer",
      "minimum": 1,
      "maximum": 200
    }
  },
  "required": [
    "model_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "entries": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "entry_id": {
                "type": "string",
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
              },
              "origin": {
                "type": "string",
                "enum": [
                  "agent",
                  "human"
                ]
              },
              "tool": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
              },
              "label": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
              },
              "base_revision": {
                "type": "string",
                "pattern": "^(0|[1-9][0-9]{0,19})$",
                "description": "Captured monotonic revision, encoded as a decimal u64 string."
              },
              "result_revision": {
                "type": "string",
                "pattern": "^(0|[1-9][0-9]{0,19})$",
                "description": "Captured monotonic revision, encoded as a decimal u64 string."
              },
              "added": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              },
              "removed": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              },
              "recolored": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              },
              "changed_bounds": {
                "anyOf": [
                  {
                    "type": "object",
                    "properties": {
                      "min": {
                        "type": "array",
                        "items": {
                          "type": "integer",
                          "minimum": -32768,
                          "maximum": 32767
                        },
                        "minItems": 3,
                        "maxItems": 3
                      },
                      "max": {
                        "type": "array",
                        "items": {
                          "type": "integer",
                          "minimum": -32767,
                          "maximum": 32768
                        },
                        "minItems": 3,
                        "maxItems": 3
                      }
                    },
                    "required": [
                      "min",
                      "max"
                    ],
                    "additionalProperties": true,
                    "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
                  },
                  {
                    "type": "null"
                  }
                ]
              }
            },
            "required": [
              "entry_id",
              "origin",
              "tool",
              "label",
              "base_revision",
              "result_revision",
              "added",
              "removed",
              "recolored",
              "changed_bounds"
            ],
            "additionalProperties": true
          },
          "minItems": 0,
          "maxItems": 200
        },
        "total": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "next_offset": {
          "anyOf": [
            {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "redo_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "initial_retained": {
          "type": "boolean"
        },
        "history_bytes": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        }
      },
      "required": [
        "model_id",
        "revision",
        "entries",
        "total",
        "next_offset",
        "head_id",
        "redo_id",
        "initial_retained",
        "history_bytes"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_history\_redo

Redo exactly the expected redo entry. Requires captured epoch/revision/entry\_id. A new edit clears the redo branch; stale IDs reject.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `entry_id`          | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "entry_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "entry_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        },
        "redo_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability",
        "redo_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_history\_undo

Undo exactly the expected top entry. Requires captured epoch/revision/entry\_id; intervening edits reject. Revision increases monotonically; original entry retains its agent attribution.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `entry_id`          | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "entry_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "entry_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        },
        "redo_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability",
        "redo_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_layers\_add

Insert an empty layer at bottom-to-top index (default append). Optional layer\_id supports deterministic references. Maximum 64 layers.

| Argument            | Required | Type / constraints                                                                                                                     |
| ------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch`    | Yes      | string                                                                                                                                 |
| `operation_id`      | Yes      | string                                                                                                                                 |
| `model_id`          | Yes      | string                                                                                                                                 |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                  |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `name`              | Yes      | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer_id`          | Optional | string                                                                                                                                 |
| `index`             | Optional | integer; minimum: 0; maximum: 64                                                                                                       |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "index": {
      "type": "integer",
      "minimum": 0,
      "maximum": 64
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "name"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        },
        "layer_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability",
        "layer_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_layers\_list

List stable layer IDs, exact names, visibility, lock state and stored voxel counts in bottom-to-top order.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    }
  },
  "required": [
    "model_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "layers": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "id": {
                "type": "string",
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
              },
              "name": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
              },
              "visible": {
                "type": "boolean"
              },
              "locked": {
                "type": "boolean"
              },
              "voxel_count": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              }
            },
            "required": [
              "id",
              "name",
              "visible",
              "locked",
              "voxel_count"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 64
        }
      },
      "required": [
        "model_id",
        "revision",
        "layers"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_layers\_lock

Set lock explicitly. Clear the lock in a separate operation before voxel editing; all mutations share the core lock rules and history.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `locked`            | Yes      | boolean                                                                                                                              |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "locked": {
      "type": "boolean"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer",
    "locked"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_layers\_remove

Delete one unlocked layer and its voxels as one undoable edit. Final-layer deletion rejects; anchored comments remain and may become orphaned.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_layers\_rename

Rename a layer without changing its stable ID. Duplicate names are allowed, but future name-based access must be unambiguous.

| Argument            | Required | Type / constraints                                                                                                                     |
| ------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch`    | Yes      | string                                                                                                                                 |
| `operation_id`      | Yes      | string                                                                                                                                 |
| `model_id`          | Yes      | string                                                                                                                                 |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                  |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                             |
| `name`              | Yes      | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer",
    "name"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_layers\_reorder

Set an exact bottom-to-top permutation of all layer IDs/names. The highest visible layer wins at overlaps; omitted or duplicate layers reject.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `order`             | Yes      | array; minItems: 1; maxItems: 64                                                                                                     |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "order": {
      "type": "array",
      "items": {
        "type": "string",
        "minLength": 1,
        "maxLength": 128,
        "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
        "description": "Layer UUID or unique exact name. Never inferred from UI selection."
      },
      "minItems": 1,
      "maxItems": 64
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "order"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_layers\_visibility

Set visibility explicitly. Hidden cells stay stored and cannot be edited until made visible. Metadata changes are undoable.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `visible`           | Yes      | boolean                                                                                                                              |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "visible": {
      "type": "boolean"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer",
    "visible"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_palette\_apply\_preset

Append exact preset colors into unused slots without recoloring existing voxels. Returns source-to-new slot mapping. Rejects atomically if capacity would exceed 256; originals are immutable.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `preset`            | Yes      | string; enum: \["pico8","endesga32","sweetie16","grayscale","neutral","warm","vivid"\]                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "preset": {
      "type": "string",
      "enum": [
        "pico8",
        "endesga32",
        "sweetie16",
        "grayscale",
        "neutral",
        "warm",
        "vivid"
      ]
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "preset"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        },
        "slot_mapping": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "source": {
                "type": "integer",
                "minimum": 1,
                "maximum": 256
              },
              "slot": {
                "type": "integer",
                "minimum": 1,
                "maximum": 256
              }
            },
            "required": [
              "source",
              "slot"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 256
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability",
        "slot_mapping"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_palette\_export

Return palette as .hex, .gpl or JASC-PAL .pal text in display order; GPL preserves representable names. Read-only, no filesystem side effects.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |
| `format`   | Yes      | string; enum: \["hex","gpl","pal"\]                                   |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "format": {
      "type": "string",
      "enum": [
        "hex",
        "gpl",
        "pal"
      ]
    }
  },
  "required": [
    "model_id",
    "format"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "text": {
          "type": "string",
          "minLength": 1,
          "maxLength": 65536
        },
        "format": {
          "type": "string",
          "enum": [
            "hex",
            "gpl",
            "pal"
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "text",
        "format"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_palette\_import

Append colors from bounded .hex, .gpl or JASC-PAL .pal text into unused slots; no implicit geometry recoloring. Returns source-to-new mapping; malformed input or insufficient capacity leaves palette unchanged.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `format`            | Yes      | string; enum: \["hex","gpl","pal"\]                                                                                                  |
| `text`              | Yes      | string; maxLength: 65536                                                                                                             |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "format": {
      "type": "string",
      "enum": [
        "hex",
        "gpl",
        "pal"
      ]
    },
    "text": {
      "type": "string",
      "minLength": 1,
      "maxLength": 65536
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "format",
    "text"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        },
        "slot_mapping": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "source": {
                "type": "integer",
                "minimum": 1,
                "maximum": 256
              },
              "slot": {
                "type": "integer",
                "minimum": 1,
                "maximum": 256
              }
            },
            "required": [
              "source",
              "slot"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 256
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability",
        "slot_mapping"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_palette\_list

List stable palette slots, names, RGBA colors, display order and exact preset names. Slot 0 is reserved for air.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    }
  },
  "required": [
    "model_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "palette": {
          "type": "object",
          "properties": {
            "slots": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "index": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 256
                  },
                  "rgba": {
                    "type": "array",
                    "prefixItems": [
                      {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 255
                      },
                      {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 255
                      },
                      {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 255
                      },
                      {
                        "const": 255
                      }
                    ],
                    "items": false,
                    "minItems": 4,
                    "maxItems": 4,
                    "description": "Opaque sRGB RGBA8; alpha must equal 255."
                  },
                  "name": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 128,
                    "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                    "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
                  }
                },
                "required": [
                  "index",
                  "rgba"
                ],
                "additionalProperties": true
              },
              "minItems": 1,
              "maxItems": 256
            },
            "order": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": 1,
                "maximum": 256
              },
              "minItems": 1,
              "maxItems": 256
            }
          },
          "required": [
            "slots",
            "order"
          ],
          "additionalProperties": true
        },
        "presets": {
          "type": "array",
          "items": {
            "type": "string",
            "minLength": 1,
            "maxLength": 32
          },
          "minItems": 1,
          "maxItems": 7
        }
      },
      "required": [
        "model_id",
        "revision",
        "palette",
        "presets"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_palette\_set\_slot

Add or replace stable slot 1..256 with opaque RGBA. Existing references change appearance, including locked layers. Omitted name creates an unnamed swatch. One undoable edit.

| Argument            | Required | Type / constraints                                                                                                                     |
| ------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch`    | Yes      | string                                                                                                                                 |
| `operation_id`      | Yes      | string                                                                                                                                 |
| `model_id`          | Yes      | string                                                                                                                                 |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                  |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `slot`              | Yes      | integer; minimum: 1; maximum: 256                                                                                                      |
| `rgba`              | Yes      | array; minItems: 4; maxItems: 4. Opaque sRGB RGBA8; alpha must equal 255.                                                              |
| `name`              | Optional | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "slot": {
      "type": "integer",
      "minimum": 1,
      "maximum": 256
    },
    "rgba": {
      "type": "array",
      "prefixItems": [
        {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        {
          "const": 255
        }
      ],
      "items": false,
      "minItems": 4,
      "maxItems": 4,
      "description": "Opaque sRGB RGBA8; alpha must equal 255."
    },
    "name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "slot",
    "rgba"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_query

Query bounds, counts or paged occupied voxels within a half-open region. view=layers (default) counts overlaps separately; composite uses highest visible layer. Optional layer restricts stored-layer queries. limit defaults 1000, max 10000. Use returned revision with next\_offset to prevent mixed pages.

| Argument   | Required | Type / constraints                                                                                                            |
| ---------- | -------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                                                                        |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string.                                                         |
| `kind`     | Yes      | string; enum: \["bounds","counts","voxels"\]                                                                                  |
| `region`   | Optional | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime. |
| `layer`    | Optional | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                    |
| `view`     | Optional | string; enum: \["layers","composite"\]                                                                                        |
| `offset`   | Optional | integer; minimum: 0; maximum: 1048576                                                                                         |
| `limit`    | Optional | integer; minimum: 1; maximum: 10000                                                                                           |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "kind": {
      "type": "string",
      "enum": [
        "bounds",
        "counts",
        "voxels"
      ]
    },
    "region": {
      "type": "object",
      "properties": {
        "min": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32768,
            "maximum": 32767
          },
          "minItems": 3,
          "maxItems": 3
        },
        "max": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32767,
            "maximum": 32768
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "min",
        "max"
      ],
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "view": {
      "type": "string",
      "enum": [
        "layers",
        "composite"
      ]
    },
    "offset": {
      "type": "integer",
      "minimum": 0,
      "maximum": 1048576
    },
    "limit": {
      "type": "integer",
      "minimum": 1,
      "maximum": 10000
    }
  },
  "required": [
    "model_id",
    "kind"
  ],
  "additionalProperties": false,
  "not": {
    "required": [
      "view",
      "layer"
    ],
    "properties": {
      "view": {
        "const": "composite"
      }
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "bounds": {
          "type": "object",
          "properties": {
            "min": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            },
            "max": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32767,
                "maximum": 32768
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "min",
            "max"
          ],
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
        },
        "region": {
          "type": "object",
          "properties": {
            "min": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            },
            "max": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32767,
                "maximum": 32768
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "min",
            "max"
          ],
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
        },
        "occupied_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "voxel_count": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "view": {
          "type": "string",
          "enum": [
            "layers",
            "composite"
          ]
        },
        "slots": {
          "type": "object",
          "propertyNames": {
            "pattern": "^[0-9]{1,3}$"
          },
          "additionalProperties": {
            "type": "integer",
            "minimum": 0,
            "maximum": 9223372036854775807
          }
        },
        "next_offset": {
          "anyOf": [
            {
              "type": "integer",
              "minimum": 0,
              "maximum": 9223372036854775807
            },
            {
              "type": "null"
            }
          ]
        },
        "voxels": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "layer_id": {
                "anyOf": [
                  {
                    "type": "string",
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "at": {
                "type": "array",
                "items": {
                  "type": "integer",
                  "minimum": -32768,
                  "maximum": 32767
                },
                "minItems": 3,
                "maxItems": 3
              },
              "slot": {
                "type": "integer",
                "minimum": 1,
                "maximum": 256
              }
            },
            "required": [
              "layer_id",
              "at",
              "slot"
            ],
            "additionalProperties": true
          },
          "minItems": 0,
          "maxItems": 10000
        }
      },
      "required": [
        "model_id",
        "revision",
        "bounds",
        "region",
        "occupied_bounds",
        "voxel_count",
        "view"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_render

Render CPU PNG image content from one immutable revision. Default four views \[iso,front,right,back\] at 512², transparent background, orthographic, 2x AA, labeled contact sheet. Up to six views or one explicit camera; <=4,194,304 combined image/sheet pixels. image\_mode=contact\_sheet|individual|both. Returns camera metadata and hashes; no GPU/display required.

| Argument                     | Required | Type / constraints                                                                                                                                                              |
| ---------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `model_id`                   | Yes      | string                                                                                                                                                                          |
| `revision`                   | Optional | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                                                           |
| `views`                      | Optional | array; minItems: 1; maxItems: 6                                                                                                                                                 |
| `camera`                     | Optional | object                                                                                                                                                                          |
| `width`                      | Optional | integer; minimum: 1; maximum: 1024                                                                                                                                              |
| `height`                     | Optional | integer; minimum: 1; maximum: 1024                                                                                                                                              |
| `perspective`                | Optional | boolean                                                                                                                                                                         |
| `background`                 | Optional | array; minItems: 4; maxItems: 4                                                                                                                                                 |
| `ambient_occlusion_strength` | Optional | number; default: 1; minimum: 0; maximum: 2. Local voxel contact shading: 0 disables AO, 1 preserves the default look, 2 doubles darkening. Independent of viewport preferences. |
| `image_mode`                 | Optional | string; enum: \["contact\_sheet","individual","both"\]                                                                                                                          |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "views": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "iso",
          "front",
          "back",
          "left",
          "right",
          "top"
        ]
      },
      "minItems": 1,
      "maxItems": 6
    },
    "camera": {
      "type": "object",
      "properties": {
        "eye": {
          "type": "array",
          "items": {
            "type": "number",
            "minimum": -1000000000.0,
            "maximum": 1000000000.0
          },
          "minItems": 3,
          "maxItems": 3
        },
        "target": {
          "type": "array",
          "items": {
            "type": "number",
            "minimum": -1000000000.0,
            "maximum": 1000000000.0
          },
          "minItems": 3,
          "maxItems": 3
        },
        "up": {
          "type": "array",
          "items": {
            "type": "number",
            "minimum": -1000000000.0,
            "maximum": 1000000000.0
          },
          "minItems": 3,
          "maxItems": 3
        },
        "projection": {
          "type": "string",
          "enum": [
            "orthographic",
            "perspective"
          ]
        },
        "ortho_height": {
          "type": "number",
          "exclusiveMinimum": 0,
          "maximum": 1000000000.0
        },
        "fov_y_degrees": {
          "type": "number",
          "exclusiveMinimum": 1,
          "exclusiveMaximum": 179
        }
      },
      "required": [
        "eye",
        "target",
        "up",
        "projection"
      ],
      "additionalProperties": false,
      "oneOf": [
        {
          "properties": {
            "projection": {
              "const": "orthographic"
            }
          },
          "required": [
            "ortho_height"
          ],
          "not": {
            "required": [
              "fov_y_degrees"
            ]
          }
        },
        {
          "properties": {
            "projection": {
              "const": "perspective"
            }
          },
          "required": [
            "fov_y_degrees"
          ],
          "not": {
            "required": [
              "ortho_height"
            ]
          }
        }
      ]
    },
    "width": {
      "type": "integer",
      "minimum": 1,
      "maximum": 1024
    },
    "height": {
      "type": "integer",
      "minimum": 1,
      "maximum": 1024
    },
    "perspective": {
      "type": "boolean"
    },
    "background": {
      "type": "array",
      "items": {
        "type": "integer",
        "minimum": 0,
        "maximum": 255
      },
      "minItems": 4,
      "maxItems": 4
    },
    "ambient_occlusion_strength": {
      "type": "number",
      "minimum": 0,
      "maximum": 2,
      "default": 1,
      "description": "Local voxel contact shading: 0 disables AO, 1 preserves the default look, 2 doubles darkening. Independent of viewport preferences."
    },
    "image_mode": {
      "type": "string",
      "enum": [
        "contact_sheet",
        "individual",
        "both"
      ]
    }
  },
  "required": [
    "model_id"
  ],
  "additionalProperties": false,
  "not": {
    "required": [
      "camera"
    ],
    "anyOf": [
      {
        "required": [
          "views"
        ]
      },
      {
        "required": [
          "perspective"
        ]
      }
    ]
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "backend": {
          "const": "cpu"
        },
        "bounds": {
          "type": "object",
          "properties": {
            "min": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32768,
                "maximum": 32767
              },
              "minItems": 3,
              "maxItems": 3
            },
            "max": {
              "type": "array",
              "items": {
                "type": "integer",
                "minimum": -32767,
                "maximum": 32768
              },
              "minItems": 3,
              "maxItems": 3
            }
          },
          "required": [
            "min",
            "max"
          ],
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
        },
        "views": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "view": {
                "type": "string",
                "enum": [
                  "iso",
                  "front",
                  "back",
                  "left",
                  "right",
                  "top",
                  "camera",
                  "contact_sheet"
                ]
              },
              "width": {
                "type": "integer",
                "minimum": 1,
                "maximum": 2048
              },
              "height": {
                "type": "integer",
                "minimum": 1,
                "maximum": 2048
              },
              "sha256": {
                "type": "string",
                "pattern": "^[0-9a-f]{64}$"
              },
              "bytes": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              },
              "camera": {
                "type": "object",
                "properties": {
                  "eye": {
                    "type": "array",
                    "items": {
                      "type": "number",
                      "minimum": -1000000000.0,
                      "maximum": 1000000000.0
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "target": {
                    "type": "array",
                    "items": {
                      "type": "number",
                      "minimum": -1000000000.0,
                      "maximum": 1000000000.0
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "up": {
                    "type": "array",
                    "items": {
                      "type": "number",
                      "minimum": -1000000000.0,
                      "maximum": 1000000000.0
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "projection": {
                    "type": "string",
                    "enum": [
                      "orthographic",
                      "perspective"
                    ]
                  },
                  "ortho_height": {
                    "type": "number",
                    "exclusiveMinimum": 0,
                    "maximum": 1000000000.0
                  },
                  "fov_y_degrees": {
                    "type": "number",
                    "exclusiveMinimum": 1,
                    "exclusiveMaximum": 179
                  }
                },
                "required": [
                  "eye",
                  "target",
                  "up",
                  "projection"
                ],
                "additionalProperties": true,
                "oneOf": [
                  {
                    "properties": {
                      "projection": {
                        "const": "orthographic"
                      }
                    },
                    "required": [
                      "ortho_height"
                    ],
                    "not": {
                      "required": [
                        "fov_y_degrees"
                      ]
                    }
                  },
                  {
                    "properties": {
                      "projection": {
                        "const": "perspective"
                      }
                    },
                    "required": [
                      "fov_y_degrees"
                    ],
                    "not": {
                      "required": [
                        "ortho_height"
                      ]
                    }
                  }
                ]
              }
            },
            "required": [
              "view",
              "width",
              "height",
              "sha256",
              "bytes",
              "camera"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 6
        },
        "images": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "view": {
                "type": "string",
                "enum": [
                  "iso",
                  "front",
                  "back",
                  "left",
                  "right",
                  "top",
                  "camera",
                  "contact_sheet"
                ]
              },
              "width": {
                "type": "integer",
                "minimum": 1,
                "maximum": 2048
              },
              "height": {
                "type": "integer",
                "minimum": 1,
                "maximum": 2048
              },
              "sha256": {
                "type": "string",
                "pattern": "^[0-9a-f]{64}$"
              },
              "bytes": {
                "type": "integer",
                "minimum": 0,
                "maximum": 9223372036854775807
              }
            },
            "required": [
              "view",
              "width",
              "height",
              "sha256",
              "bytes"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 7
        },
        "image_mode": {
          "type": "string",
          "enum": [
            "contact_sheet",
            "individual",
            "both"
          ]
        }
      },
      "required": [
        "model_id",
        "revision",
        "backend",
        "bounds",
        "views",
        "images",
        "image_mode"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_versions\_list

List named version IDs, names, parent IDs, origin and immutable content hashes. Persistent across save/open; independent of undo retention. Never includes full voxel snapshots.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    }
  },
  "required": [
    "model_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "versions": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "id": {
                "type": "string",
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
              },
              "name": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
              },
              "parent_id": {
                "anyOf": [
                  {
                    "type": "string",
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "origin": {
                "type": "string",
                "enum": [
                  "agent",
                  "human"
                ]
              },
              "content_hash": {
                "type": "string",
                "pattern": "^[0-9a-f]{64}$"
              }
            },
            "required": [
              "id",
              "name",
              "parent_id",
              "origin",
              "content_hash"
            ],
            "additionalProperties": true
          },
          "minItems": 0,
          "maxItems": 64
        }
      },
      "required": [
        "model_id",
        "revision",
        "versions"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_versions\_restore

Restore named content as a new undoable edit, keeping the version library. Requires expected\_head\_id (null for empty history) in addition to revision/epoch. Core validates current locks before installation.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `version_id`        | Yes      | string                                                                                                                               |
| `expected_head_id`  | Yes      | See schema                                                                                                                           |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "version_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_head_id": {
      "anyOf": [
        {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "version_id",
    "expected_head_id"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_versions\_save

Persist current content as a named immutable snapshot with agent attribution and content hash. Up to 64 versions, no recursive libraries. One undoable library edit; save document to retain across process restarts.

| Argument            | Required | Type / constraints                                                                                                                     |
| ------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch`    | Yes      | string                                                                                                                                 |
| `operation_id`      | Yes      | string                                                                                                                                 |
| `model_id`          | Yes      | string                                                                                                                                 |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                  |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `name`              | Yes      | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `version_id`        | Optional | string                                                                                                                                 |
| `parent_id`         | Optional | string                                                                                                                                 |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "name": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "version_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "parent_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "name"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        },
        "version_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability",
        "version_id"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_voxels\_batch

Apply 1..256 ordered voxel operations atomically to one model: set, erase\_region, fill, box. Each operation names its own layer/mirror; nesting and I/O are forbidden. Later operations see earlier edits; any failure rolls back all; one revision and undo entry.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `operations`        | Yes      | array; minItems: 1; maxItems: 256                                                                                                    |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "operations": {
      "type": "array",
      "items": {
        "oneOf": [
          {
            "type": "object",
            "properties": {
              "layer": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Layer UUID or unique exact name. Never inferred from UI selection."
              },
              "voxels": {
                "type": "array",
                "items": {
                  "type": "object",
                  "properties": {
                    "at": {
                      "type": "array",
                      "items": {
                        "type": "integer",
                        "minimum": -32768,
                        "maximum": 32767
                      },
                      "minItems": 3,
                      "maxItems": 3
                    },
                    "slot": {
                      "anyOf": [
                        {
                          "type": "integer",
                          "minimum": 0,
                          "maximum": 256
                        },
                        {
                          "type": "string",
                          "minLength": 1,
                          "maxLength": 128
                        }
                      ],
                      "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
                    }
                  },
                  "required": [
                    "at",
                    "slot"
                  ],
                  "additionalProperties": false
                },
                "minItems": 1,
                "maxItems": 65536
              },
              "mode": {
                "type": "string",
                "enum": [
                  "place",
                  "paint"
                ]
              },
              "mirror": {
                "type": "object",
                "properties": {
                  "axes": {
                    "type": "array",
                    "items": {
                      "type": "string",
                      "enum": [
                        "x",
                        "y",
                        "z"
                      ]
                    },
                    "minItems": 0,
                    "maxItems": 3,
                    "uniqueItems": true
                  },
                  "plane2": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -65536,
                      "maximum": 65536
                    },
                    "minItems": 3,
                    "maxItems": 3
                  }
                },
                "required": [
                  "axes",
                  "plane2"
                ],
                "additionalProperties": false
              },
              "kind": {
                "const": "set"
              }
            },
            "required": [
              "layer",
              "voxels",
              "kind"
            ],
            "additionalProperties": false
          },
          {
            "type": "object",
            "properties": {
              "layer": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Layer UUID or unique exact name. Never inferred from UI selection."
              },
              "region": {
                "type": "object",
                "properties": {
                  "min": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -32768,
                      "maximum": 32767
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "max": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -32767,
                      "maximum": 32768
                    },
                    "minItems": 3,
                    "maxItems": 3
                  }
                },
                "required": [
                  "min",
                  "max"
                ],
                "additionalProperties": false,
                "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
              },
              "mirror": {
                "type": "object",
                "properties": {
                  "axes": {
                    "type": "array",
                    "items": {
                      "type": "string",
                      "enum": [
                        "x",
                        "y",
                        "z"
                      ]
                    },
                    "minItems": 0,
                    "maxItems": 3,
                    "uniqueItems": true
                  },
                  "plane2": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -65536,
                      "maximum": 65536
                    },
                    "minItems": 3,
                    "maxItems": 3
                  }
                },
                "required": [
                  "axes",
                  "plane2"
                ],
                "additionalProperties": false
              },
              "kind": {
                "const": "erase_region"
              }
            },
            "required": [
              "layer",
              "region",
              "kind"
            ],
            "additionalProperties": false
          },
          {
            "type": "object",
            "properties": {
              "layer": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Layer UUID or unique exact name. Never inferred from UI selection."
              },
              "seed": {
                "type": "array",
                "items": {
                  "type": "integer",
                  "minimum": -32768,
                  "maximum": 32767
                },
                "minItems": 3,
                "maxItems": 3
              },
              "slot": {
                "anyOf": [
                  {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 256
                  },
                  {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 128
                  }
                ],
                "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
              },
              "region": {
                "type": "object",
                "properties": {
                  "min": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -32768,
                      "maximum": 32767
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "max": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -32767,
                      "maximum": 32768
                    },
                    "minItems": 3,
                    "maxItems": 3
                  }
                },
                "required": [
                  "min",
                  "max"
                ],
                "additionalProperties": false,
                "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
              },
              "cap": {
                "type": "integer",
                "minimum": 1,
                "maximum": 1048576
              },
              "mirror": {
                "type": "object",
                "properties": {
                  "axes": {
                    "type": "array",
                    "items": {
                      "type": "string",
                      "enum": [
                        "x",
                        "y",
                        "z"
                      ]
                    },
                    "minItems": 0,
                    "maxItems": 3,
                    "uniqueItems": true
                  },
                  "plane2": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -65536,
                      "maximum": 65536
                    },
                    "minItems": 3,
                    "maxItems": 3
                  }
                },
                "required": [
                  "axes",
                  "plane2"
                ],
                "additionalProperties": false
              },
              "kind": {
                "const": "fill"
              }
            },
            "required": [
              "layer",
              "seed",
              "slot",
              "kind"
            ],
            "additionalProperties": false
          },
          {
            "type": "object",
            "properties": {
              "layer": {
                "type": "string",
                "minLength": 1,
                "maxLength": 128,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "description": "Layer UUID or unique exact name. Never inferred from UI selection."
              },
              "region": {
                "type": "object",
                "properties": {
                  "min": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -32768,
                      "maximum": 32767
                    },
                    "minItems": 3,
                    "maxItems": 3
                  },
                  "max": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -32767,
                      "maximum": 32768
                    },
                    "minItems": 3,
                    "maxItems": 3
                  }
                },
                "required": [
                  "min",
                  "max"
                ],
                "additionalProperties": false,
                "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
              },
              "slot": {
                "anyOf": [
                  {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 256
                  },
                  {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 128
                  }
                ],
                "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
              },
              "hollow": {
                "type": "boolean"
              },
              "mode": {
                "type": "string",
                "enum": [
                  "place",
                  "paint",
                  "erase"
                ]
              },
              "mirror": {
                "type": "object",
                "properties": {
                  "axes": {
                    "type": "array",
                    "items": {
                      "type": "string",
                      "enum": [
                        "x",
                        "y",
                        "z"
                      ]
                    },
                    "minItems": 0,
                    "maxItems": 3,
                    "uniqueItems": true
                  },
                  "plane2": {
                    "type": "array",
                    "items": {
                      "type": "integer",
                      "minimum": -65536,
                      "maximum": 65536
                    },
                    "minItems": 3,
                    "maxItems": 3
                  }
                },
                "required": [
                  "axes",
                  "plane2"
                ],
                "additionalProperties": false
              },
              "kind": {
                "const": "box"
              }
            },
            "required": [
              "layer",
              "region",
              "kind"
            ],
            "additionalProperties": false,
            "if": {
              "properties": {
                "mode": {
                  "const": "erase"
                }
              },
              "required": [
                "mode"
              ]
            },
            "then": {
              "not": {
                "required": [
                  "slot"
                ]
              }
            },
            "else": {
              "required": [
                "slot"
              ]
            }
          }
        ]
      },
      "minItems": 1,
      "maxItems": 256
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "operations"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_voxels\_box

Build an integer half-open solid box or one-cell shell (hollow=true). Default mode=place, hollow=false, mirror=none. slot required for place/paint, forbidden for erase. One atomic undo entry.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `region`            | Yes      | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.        |
| `slot`              | Optional | See schema. Stable slot number or unique swatch name; zero is air for explicit writes only.                                          |
| `hollow`            | Optional | boolean                                                                                                                              |
| `mode`              | Optional | string; enum: \["place","paint","erase"\]                                                                                            |
| `mirror`            | Optional | object                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "region": {
      "type": "object",
      "properties": {
        "min": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32768,
            "maximum": 32767
          },
          "minItems": 3,
          "maxItems": 3
        },
        "max": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32767,
            "maximum": 32768
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "min",
        "max"
      ],
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
    },
    "slot": {
      "anyOf": [
        {
          "type": "integer",
          "minimum": 1,
          "maximum": 256
        },
        {
          "type": "string",
          "minLength": 1,
          "maxLength": 128
        }
      ],
      "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
    },
    "hollow": {
      "type": "boolean"
    },
    "mode": {
      "type": "string",
      "enum": [
        "place",
        "paint",
        "erase"
      ]
    },
    "mirror": {
      "type": "object",
      "properties": {
        "axes": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": [
              "x",
              "y",
              "z"
            ]
          },
          "minItems": 0,
          "maxItems": 3,
          "uniqueItems": true
        },
        "plane2": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -65536,
            "maximum": 65536
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "axes",
        "plane2"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer",
    "region"
  ],
  "additionalProperties": false,
  "if": {
    "properties": {
      "mode": {
        "const": "erase"
      }
    },
    "required": [
      "mode"
    ]
  },
  "then": {
    "not": {
      "required": [
        "slot"
      ]
    }
  },
  "else": {
    "required": [
      "slot"
    ]
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_voxels\_erase\_region

Erase occupied cells in min-inclusive/max-exclusive region. Mirror is explicit per call; omitted means none. Invalid/hidden/locked targets reject without partial edits.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `region`            | Yes      | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.        |
| `mirror`            | Optional | object                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "region": {
      "type": "object",
      "properties": {
        "min": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32768,
            "maximum": 32767
          },
          "minItems": 3,
          "maxItems": 3
        },
        "max": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32767,
            "maximum": 32768
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "min",
        "max"
      ],
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
    },
    "mirror": {
      "type": "object",
      "properties": {
        "axes": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": [
              "x",
              "y",
              "z"
            ]
          },
          "minItems": 0,
          "maxItems": 3,
          "uniqueItems": true
        },
        "plane2": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -65536,
            "maximum": 65536
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "axes",
        "plane2"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer",
    "region"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_voxels\_fill

Six-connected flood fill of the seed's slot, including air. Region defaults to document bounds; cap defaults to 1,048,576 visited cells. Cap exhaustion rejects atomically. Mirror never inherits UI state.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `seed`              | Yes      | array; minItems: 3; maxItems: 3                                                                                                      |
| `slot`              | Yes      | See schema. Stable slot number or unique swatch name; zero is air for explicit writes only.                                          |
| `region`            | Optional | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.        |
| `cap`               | Optional | integer; minimum: 1; maximum: 1048576                                                                                                |
| `mirror`            | Optional | object                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "seed": {
      "type": "array",
      "items": {
        "type": "integer",
        "minimum": -32768,
        "maximum": 32767
      },
      "minItems": 3,
      "maxItems": 3
    },
    "slot": {
      "anyOf": [
        {
          "type": "integer",
          "minimum": 1,
          "maximum": 256
        },
        {
          "type": "string",
          "minLength": 1,
          "maxLength": 128
        }
      ],
      "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
    },
    "region": {
      "type": "object",
      "properties": {
        "min": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32768,
            "maximum": 32767
          },
          "minItems": 3,
          "maxItems": 3
        },
        "max": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -32767,
            "maximum": 32768
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "min",
        "max"
      ],
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
    },
    "cap": {
      "type": "integer",
      "minimum": 1,
      "maximum": 1048576
    },
    "mirror": {
      "type": "object",
      "properties": {
        "axes": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": [
              "x",
              "y",
              "z"
            ]
          },
          "minItems": 0,
          "maxItems": 3,
          "uniqueItems": true
        },
        "plane2": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -65536,
            "maximum": 65536
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "axes",
        "plane2"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer",
    "seed",
    "slot"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_voxels\_get

Read explicit integer coordinates on one layer, including air as slot 0. Returns values in request order. Out-of-bounds coordinates reject the entire read.

| Argument    | Required | Type / constraints                                                                         |
| ----------- | -------- | ------------------------------------------------------------------------------------------ |
| `model_id`  | Yes      | string                                                                                     |
| `revision`  | Optional | string. Captured monotonic revision, encoded as a decimal u64 string.                      |
| `layer`     | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection. |
| `positions` | Yes      | array; minItems: 1; maxItems: 10000                                                        |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "positions": {
      "type": "array",
      "items": {
        "type": "array",
        "items": {
          "type": "integer",
          "minimum": -32768,
          "maximum": 32767
        },
        "minItems": 3,
        "maxItems": 3
      },
      "minItems": 1,
      "maxItems": 10000
    }
  },
  "required": [
    "model_id",
    "layer",
    "positions"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "layer_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "voxels": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "at": {
                "type": "array",
                "items": {
                  "type": "integer",
                  "minimum": -32768,
                  "maximum": 32767
                },
                "minItems": 3,
                "maxItems": 3
              },
              "slot": {
                "type": "integer",
                "minimum": 0,
                "maximum": 256
              }
            },
            "required": [
              "at",
              "slot"
            ],
            "additionalProperties": true
          },
          "minItems": 1,
          "maxItems": 10000
        }
      },
      "required": [
        "model_id",
        "revision",
        "layer_id",
        "voxels"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```

### quantum\_voxels\_set

Atomically place or paint explicit cells, at most 65,536. Slot 0 erases; paint affects occupied cells only. Last duplicate wins before mirror expansion; mirror conflicts reject. Omitted mode=place, mirror=none. One undo entry.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `voxels`            | Yes      | array; minItems: 1; maxItems: 65536                                                                                                  |
| `mode`              | Optional | string; enum: \["place","paint"\]                                                                                                    |
| `mirror`            | Optional | object                                                                                                                               |

Complete input schema

```json
{
  "type": "object",
  "properties": {
    "expected_epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "operation_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "model_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "expected_revision": {
      "type": "string",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "description": "Captured monotonic revision, encoded as a decimal u64 string."
    },
    "label": {
      "type": "string",
      "minLength": 1,
      "maxLength": 80,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime."
    },
    "layer": {
      "type": "string",
      "minLength": 1,
      "maxLength": 128,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "description": "Layer UUID or unique exact name. Never inferred from UI selection."
    },
    "voxels": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "at": {
            "type": "array",
            "items": {
              "type": "integer",
              "minimum": -32768,
              "maximum": 32767
            },
            "minItems": 3,
            "maxItems": 3
          },
          "slot": {
            "anyOf": [
              {
                "type": "integer",
                "minimum": 0,
                "maximum": 256
              },
              {
                "type": "string",
                "minLength": 1,
                "maxLength": 128
              }
            ],
            "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
          }
        },
        "required": [
          "at",
          "slot"
        ],
        "additionalProperties": false
      },
      "minItems": 1,
      "maxItems": 65536
    },
    "mode": {
      "type": "string",
      "enum": [
        "place",
        "paint"
      ]
    },
    "mirror": {
      "type": "object",
      "properties": {
        "axes": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": [
              "x",
              "y",
              "z"
            ]
          },
          "minItems": 0,
          "maxItems": 3,
          "uniqueItems": true
        },
        "plane2": {
          "type": "array",
          "items": {
            "type": "integer",
            "minimum": -65536,
            "maximum": 65536
          },
          "minItems": 3,
          "maxItems": 3
        }
      },
      "required": [
        "axes",
        "plane2"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer",
    "voxels"
  ],
  "additionalProperties": false,
  "$schema": "https://json-schema.org/draft/2020-12/schema"
}
```

Complete output schema

```json
{
  "type": "object",
  "properties": {
    "api_version": {
      "const": "quantum.v1"
    },
    "session_id": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "epoch": {
      "type": "string",
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    },
    "ok": {
      "type": "boolean"
    },
    "data": {
      "type": "object",
      "properties": {
        "model_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        },
        "operation_id": {
          "type": "string",
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "head_id": {
          "anyOf": [
            {
              "type": "string",
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_voxels": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "added": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "removed": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "recolored": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "changed_bounds": {
          "anyOf": [
            {
              "type": "object",
              "properties": {
                "min": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32768,
                    "maximum": 32767
                  },
                  "minItems": 3,
                  "maxItems": 3
                },
                "max": {
                  "type": "array",
                  "items": {
                    "type": "integer",
                    "minimum": -32767,
                    "maximum": 32768
                  },
                  "minItems": 3,
                  "maxItems": 3
                }
              },
              "required": [
                "min",
                "max"
              ],
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime."
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "type": "array",
          "items": {
            "type": "string",
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
          },
          "minItems": 0,
          "maxItems": 64
        },
        "palette_changed": {
          "type": "boolean"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "work_units": {
          "type": "integer",
          "minimum": 0,
          "maximum": 9223372036854775807
        },
        "durability": {
          "const": "memory"
        }
      },
      "required": [
        "model_id",
        "revision",
        "operation_id",
        "undo_entry_id",
        "head_id",
        "changed_voxels",
        "added",
        "removed",
        "recolored",
        "changed_bounds",
        "changed_layers",
        "palette_changed",
        "metadata_changed",
        "work_units",
        "durability"
      ],
      "additionalProperties": true
    },
    "error": {
      "type": "object",
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1,
          "maxLength": 80
        },
        "message": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "retryable": {
          "type": "boolean"
        },
        "recovery": {
          "type": "string",
          "minLength": 1,
          "maxLength": 4096
        },
        "path": {
          "type": "string"
        },
        "command_index": {
          "type": "integer",
          "minimum": 0,
          "maximum": 255
        },
        "current_revision": {
          "type": "string",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "description": "Captured monotonic revision, encoded as a decimal u64 string."
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "additionalProperties": false,
  "oneOf": [
    {
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ],
      "not": {
        "required": [
          "error"
        ]
      }
    },
    {
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ],
      "not": {
        "required": [
          "data"
        ]
      }
    }
  ]
}
```
