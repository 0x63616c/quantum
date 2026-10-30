# MCP tools

Coordinates in. Models out. Real visual feedback.

This reference is generated from the Rust server's live discovery catalog, API `voxelite.v1`. Download the [complete JSON catalog](https://voxelite.worldwidewebb.co/assets/mcp-tools.json). Run `voxelite-mcp --schema` to inspect the catalog for your own build.

## Shared contract

*   Stdio uses one JSON-RPC message per line. Stdout contains protocol messages only; diagnostics go to stderr.
*   Supported protocol profiles: `2025-11-25` (initialize and initialized notification) and `2026-07-28` (per-request protocol metadata).
*   Discover the epoch with `voxelite_capabilities_get`. Writes use captured `expected_epoch`, a decimal-string `expected_revision` where required, and a fresh UUID `operation_id`.
*   Coordinate order is `[x, y, z]`, right-handed, Y up. Regions include `min` and exclude `max`. Slots are stable IDs, not display positions; zero means air only where the schema allows it.
*   Prefer layer UUIDs and stable slot numbers. Names must resolve uniquely. Unknown fields are rejected, including nested fields.
*   Atomic batches commit one undo entry or nothing. Content-edit results report revision, changed bounds/counts and history IDs. Check the result before updating your local state.
*   The same success JSON appears in `structuredContent` and the first text block. It contains `api_version`, `session_id`, `epoch`, `ok`, and `data` or `error`.

The attached native catalog omits `document_create`, `document_open`, and `document_save`. Use the native UI for file lifecycle. No screenshot-named tool or ellipsoid/cylinder tool is currently exposed: use `voxelite_render` for model images and explicit voxel batches for other shapes.

## Render feedback

Default views are iso/front/right/back at 512 × 512 each, returned as a 1024 × 1024 contact sheet. `image_mode` can return individual images, a sheet, or both. Images are actual PNG MCP image blocks, never a simulated preview.

At most six views, 1024 × 1024 per view and 4,194,304 combined view/sheet pixels are allowed, with a 12 MiB encoded response budget. Front looks from +Z, right from +X, top from +Y with −Z up. Explicit cameras and named views are mutually exclusive. Lower dimensions or view count after a budget error.

## Errors, retries and files

| Code                                    | Next step                                                                         |
| --------------------------------------- | --------------------------------------------------------------------------------- |
| `revision_conflict`, `epoch_conflict`   | Read the current model/session, rebase the intended edit, use a new operation ID. |
| `operation_id_reused`                   | Recover the original request. A retry must have identical tool, arguments and ID. |
| `layer_blocked`                         | Confirm the target layer; show or unlock it explicitly if intended.               |
| `history_conflict`                      | Read history and capture the exact current head/redo ID.                          |
| `invalid_argument`, `out_of_bounds`     | Correct the indicated field or half-open bounds.                                  |
| `limit_exceeded`                        | Reduce batch/page/render size. If receipts are exhausted, save and restart.       |
| `external_file_conflict`, `file_exists` | Preserve externally changed content; choose a new output path.                    |
| `path_outside_root`                     | Use a relative path within the configured file root. Parents must exist.          |
| `outcome_unknown`                       | Inspect the destination: rename may have succeeded before directory sync failed.  |

A domain error returns `isError: true` and `ok: false`, with a recovery hint. Transport success alone does not mean an edit succeeded. Cancellation does not undo an already committed edit. Exact retries replay successful receipts only within the current process.

File tools are confined to `--root`. New files are create-only. Saving over a previously opened/saved file requires its recorded hash to match. Save uses a sibling temporary file, sync and atomic installation. Export does not mark a document saved. Undo/retry state is session-local; comments and named versions persist in `.voxl`.

## All 38 tools

Generated directly from `VoxeliteServer::tools()`. Expand a schema for nested properties, defaults, limits, conditional requirements and the output envelope. A property marked optional may still be required by a schema condition.

- [capabilities\_get](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_capabilities_get)
- [comments\_add](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_comments_add)
- [comments\_list](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_comments_list)
- [comments\_reopen](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_comments_reopen)
- [comments\_reply](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_comments_reply)
- [comments\_resolve](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_comments_resolve)
- [document\_create](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_document_create)
- [document\_info](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_document_info)
- [document\_open](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_document_open)
- [document\_save](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_document_save)
- [export\_glb](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_export_glb)
- [export\_png](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_export_png)
- [history\_read](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_history_read)
- [history\_redo](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_history_redo)
- [history\_undo](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_history_undo)
- [layers\_add](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_layers_add)
- [layers\_list](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_layers_list)
- [layers\_lock](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_layers_lock)
- [layers\_remove](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_layers_remove)
- [layers\_rename](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_layers_rename)
- [layers\_reorder](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_layers_reorder)
- [layers\_visibility](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_layers_visibility)
- [palette\_apply\_preset](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_palette_apply_preset)
- [palette\_export](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_palette_export)
- [palette\_import](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_palette_import)
- [palette\_list](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_palette_list)
- [palette\_set\_slot](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_palette_set_slot)
- [query](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_query)
- [render](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_render)
- [versions\_list](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_versions_list)
- [versions\_restore](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_versions_restore)
- [versions\_save](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_versions_save)
- [voxels\_batch](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_voxels_batch)
- [voxels\_box](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_voxels_box)
- [voxels\_erase\_region](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_voxels_erase_region)
- [voxels\_fill](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_voxels_fill)
- [voxels\_get](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_voxels_get)
- [voxels\_set](https://voxelite.worldwidewebb.co/docs/mcp/#voxelite_voxels_set)

### voxelite\_capabilities\_get

Discover voxelite.v1, session epoch, limits, CPU renderer, exact palette presets and open model IDs. Start here; use returned epoch and revisions for edits.

| Argument | Required | Type / constraints |
| -------- | -------- | ------------------ |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {},
  "required": [],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "api_version": {
          "const": "voxelite.v1"
        },
        "documents": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "bounds": {
                "additionalProperties": true,
                "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
                "properties": {
                  "max": {
                    "items": {
                      "maximum": 32768,
                      "minimum": -32767,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  },
                  "min": {
                    "items": {
                      "maximum": 32767,
                      "minimum": -32768,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "min",
                  "max"
                ],
                "type": "object"
              },
              "comments": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "dirty": {
                "type": "boolean"
              },
              "head_id": {
                "anyOf": [
                  {
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                    "type": "string"
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "layers": {
                "items": {
                  "additionalProperties": true,
                  "properties": {
                    "id": {
                      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                      "type": "string"
                    },
                    "locked": {
                      "type": "boolean"
                    },
                    "name": {
                      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                      "maxLength": 128,
                      "minLength": 1,
                      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                      "type": "string"
                    },
                    "visible": {
                      "type": "boolean"
                    },
                    "voxel_count": {
                      "maximum": 9223372036854775807,
                      "minimum": 0,
                      "type": "integer"
                    }
                  },
                  "required": [
                    "id",
                    "name",
                    "visible",
                    "locked",
                    "voxel_count"
                  ],
                  "type": "object"
                },
                "maxItems": 64,
                "minItems": 1,
                "type": "array"
              },
              "model_id": {
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                "type": "string"
              },
              "name": {
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "palette_slots": {
                "maximum": 256,
                "minimum": 1,
                "type": "integer"
              },
              "redo_id": {
                "anyOf": [
                  {
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                    "type": "string"
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "revision": {
                "description": "Captured monotonic revision, encoded as a decimal u64 string.",
                "pattern": "^(0|[1-9][0-9]{0,19})$",
                "type": "string"
              },
              "versions": {
                "maximum": 64,
                "minimum": 0,
                "type": "integer"
              },
              "voxel_count": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
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
            "type": "object"
          },
          "maxItems": 8,
          "minItems": 0,
          "type": "array"
        },
        "durability": {
          "const": "memory"
        },
        "formats": {
          "items": {
            "maxLength": 16,
            "minLength": 1,
            "type": "string"
          },
          "maxItems": 5,
          "minItems": 1,
          "type": "array"
        },
        "limits": {
          "additionalProperties": true,
          "properties": {
            "batch_operations": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            "explicit_voxels": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            "open_documents": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            "operation_receipts": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            "read_voxels": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            "receipt_bytes": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            "request_bytes": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            "result_bytes": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            "work_units": {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
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
          "type": "object"
        },
        "presets": {
          "items": {
            "maxLength": 32,
            "minLength": 1,
            "type": "string"
          },
          "maxItems": 7,
          "minItems": 1,
          "type": "array"
        },
        "protocol_versions": {
          "items": {
            "enum": [
              "2025-11-25",
              "2026-07-28"
            ],
            "type": "string"
          },
          "maxItems": 2,
          "minItems": 2,
          "type": "array"
        },
        "renderer": {
          "const": "cpu"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_comments\_add

Create a persistent voxel/region anchored thread. Author is always agent, never caller-supplied. Text is plain text, max 4096 UTF-8 bytes. Returns thread/message IDs; undoable and saved in .voxl.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `anchor`            | Yes      | See schema                                                                                                                           |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Optional | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `message_id`        | Optional | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `text`              | Yes      | string; maxLength: 4096                                                                                                              |
| `thread_id`         | Optional | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "anchor": {
      "oneOf": [
        {
          "additionalProperties": false,
          "properties": {
            "at": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "kind": {
              "const": "voxel"
            }
          },
          "required": [
            "kind",
            "at"
          ],
          "type": "object"
        },
        {
          "additionalProperties": false,
          "properties": {
            "kind": {
              "const": "region"
            },
            "max": {
              "items": {
                "maximum": 32768,
                "minimum": -32767,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "min": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }
          },
          "required": [
            "kind",
            "min",
            "max"
          ],
          "type": "object"
        }
      ]
    },
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "message_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "text": {
      "maxLength": 4096,
      "minLength": 1,
      "type": "string"
    },
    "thread_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "message_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "thread_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_comments\_list

List anchored threads including messages, agent/human roles, resolved state and anchor status. Optional state=open|resolved|all (default all). Thread and message pages default to 50, max 200. message\_offset applies per thread; bind pages with revision.

| Argument         | Required | Type / constraints                                                    |
| ---------------- | -------- | --------------------------------------------------------------------- |
| `limit`          | Optional | integer; minimum: 1; maximum: 200                                     |
| `message_limit`  | Optional | integer; minimum: 1; maximum: 200                                     |
| `message_offset` | Optional | integer; minimum: 0; maximum: 10000                                   |
| `model_id`       | Yes      | string                                                                |
| `offset`         | Optional | integer; minimum: 0; maximum: 1024                                    |
| `revision`       | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |
| `state`          | Optional | string; enum: \["open","resolved","all"\]                             |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "limit": {
      "maximum": 200,
      "minimum": 1,
      "type": "integer"
    },
    "message_limit": {
      "maximum": 200,
      "minimum": 1,
      "type": "integer"
    },
    "message_offset": {
      "maximum": 10000,
      "minimum": 0,
      "type": "integer"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "offset": {
      "maximum": 1024,
      "minimum": 0,
      "type": "integer"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "state": {
      "enum": [
        "open",
        "resolved",
        "all"
      ],
      "type": "string"
    }
  },
  "required": [
    "model_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "comments": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "anchor_state": {
                "enum": [
                  "visible",
                  "hidden",
                  "orphaned"
                ],
                "type": "string"
              },
              "message_count": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "next_message_offset": {
                "anyOf": [
                  {
                    "maximum": 9223372036854775807,
                    "minimum": 0,
                    "type": "integer"
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "thread": {
                "additionalProperties": true,
                "properties": {
                  "anchor": {
                    "oneOf": [
                      {
                        "additionalProperties": true,
                        "properties": {
                          "at": {
                            "items": {
                              "maximum": 32767,
                              "minimum": -32768,
                              "type": "integer"
                            },
                            "maxItems": 3,
                            "minItems": 3,
                            "type": "array"
                          },
                          "kind": {
                            "const": "voxel"
                          }
                        },
                        "required": [
                          "kind",
                          "at"
                        ],
                        "type": "object"
                      },
                      {
                        "additionalProperties": true,
                        "properties": {
                          "kind": {
                            "const": "region"
                          },
                          "max": {
                            "items": {
                              "maximum": 32768,
                              "minimum": -32767,
                              "type": "integer"
                            },
                            "maxItems": 3,
                            "minItems": 3,
                            "type": "array"
                          },
                          "min": {
                            "items": {
                              "maximum": 32767,
                              "minimum": -32768,
                              "type": "integer"
                            },
                            "maxItems": 3,
                            "minItems": 3,
                            "type": "array"
                          }
                        },
                        "required": [
                          "kind",
                          "min",
                          "max"
                        ],
                        "type": "object"
                      }
                    ]
                  },
                  "id": {
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                    "type": "string"
                  },
                  "layer": {
                    "anyOf": [
                      {
                        "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                        "type": "string"
                      },
                      {
                        "type": "null"
                      }
                    ]
                  },
                  "messages": {
                    "items": {
                      "additionalProperties": true,
                      "properties": {
                        "id": {
                          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                          "type": "string"
                        },
                        "origin": {
                          "enum": [
                            "agent",
                            "human"
                          ],
                          "type": "string"
                        },
                        "text": {
                          "maxLength": 8192,
                          "minLength": 1,
                          "type": "string"
                        },
                        "tool": {
                          "anyOf": [
                            {
                              "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                              "maxLength": 128,
                              "minLength": 1,
                              "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                              "type": "string"
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
                      "type": "object"
                    },
                    "maxItems": 200,
                    "minItems": 0,
                    "type": "array"
                  },
                  "resolved": {
                    "type": "boolean"
                  }
                },
                "required": [
                  "id",
                  "anchor",
                  "layer",
                  "resolved",
                  "messages"
                ],
                "type": "object"
              }
            },
            "required": [
              "thread",
              "message_count",
              "next_message_offset",
              "anchor_state"
            ],
            "type": "object"
          },
          "maxItems": 200,
          "minItems": 0,
          "type": "array"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "next_offset": {
          "anyOf": [
            {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            {
              "type": "null"
            }
          ]
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "total": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "model_id",
        "revision",
        "comments",
        "total",
        "next_offset"
      ],
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_comments\_reopen

Reopen a resolved discussion; idempotent if already open. Preserves messages and anchor; undoable.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `thread_id`         | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "thread_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "thread_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_comments\_reply

Append an agent-attributed plain-text reply to a thread, resolved or open. One undoable edit; returns message ID.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `message_id`        | Optional | string                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `text`              | Yes      | string; maxLength: 4096                                                                                                              |
| `thread_id`         | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "message_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "text": {
      "maxLength": 4096,
      "minLength": 1,
      "type": "string"
    },
    "thread_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "message_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "thread_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_comments\_resolve

Resolve an anchored discussion; idempotent if already resolved. Preserves messages and anchor; undoable.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `thread_id`         | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "thread_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "thread_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_document\_create

Create a revision-zero document and visible unlocked Base layer. Default bounds \[0,0,0\]..\[32,32,32\]. Returns model/layer IDs. Up to eight documents per session; never replaces an open model.

| Argument         | Required | Type / constraints                                                                                                                     |
| ---------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `bounds`         | Optional | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.          |
| `expected_epoch` | Yes      | string                                                                                                                                 |
| `label`          | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `layer_id`       | Optional | string                                                                                                                                 |
| `model_id`       | Optional | string                                                                                                                                 |
| `name`           | Yes      | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `operation_id`   | Yes      | string                                                                                                                                 |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "bounds": {
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
      "properties": {
        "max": {
          "items": {
            "maximum": 32768,
            "minimum": -32767,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "min": {
          "items": {
            "maximum": 32767,
            "minimum": -32768,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "min",
        "max"
      ],
      "type": "object"
    },
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "name": {
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "name"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "bounds": {
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
          "properties": {
            "max": {
              "items": {
                "maximum": 32768,
                "minimum": -32767,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "min": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }
          },
          "required": [
            "min",
            "max"
          ],
          "type": "object"
        },
        "comments": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "dirty": {
          "type": "boolean"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "layers": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "id": {
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                "type": "string"
              },
              "locked": {
                "type": "boolean"
              },
              "name": {
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "visible": {
                "type": "boolean"
              },
              "voxel_count": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              }
            },
            "required": [
              "id",
              "name",
              "visible",
              "locked",
              "voxel_count"
            ],
            "type": "object"
          },
          "maxItems": 64,
          "minItems": 1,
          "type": "array"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "name": {
          "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
          "maxLength": 128,
          "minLength": 1,
          "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
          "type": "string"
        },
        "palette_slots": {
          "maximum": 256,
          "minimum": 1,
          "type": "integer"
        },
        "redo_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "versions": {
          "maximum": 64,
          "minimum": 0,
          "type": "integer"
        },
        "voxel_count": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_document\_info

Read document identity, bounds, layers, counts, dirty state and current revision. Optional revision must equal the live revision; no UI state is used.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    }
  },
  "required": [
    "model_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "bounds": {
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
          "properties": {
            "max": {
              "items": {
                "maximum": 32768,
                "minimum": -32767,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "min": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }
          },
          "required": [
            "min",
            "max"
          ],
          "type": "object"
        },
        "comments": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "dirty": {
          "type": "boolean"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "layers": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "id": {
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                "type": "string"
              },
              "locked": {
                "type": "boolean"
              },
              "name": {
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "visible": {
                "type": "boolean"
              },
              "voxel_count": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              }
            },
            "required": [
              "id",
              "name",
              "visible",
              "locked",
              "voxel_count"
            ],
            "type": "object"
          },
          "maxItems": 64,
          "minItems": 1,
          "type": "array"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "name": {
          "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
          "maxLength": 128,
          "minLength": 1,
          "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
          "type": "string"
        },
        "palette_slots": {
          "maximum": 256,
          "minimum": 1,
          "type": "integer"
        },
        "redo_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "versions": {
          "maximum": 64,
          "minimum": 0,
          "type": "integer"
        },
        "voxel_count": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_document\_open

Open a canonical .voxl file relative to the configured project root, preserving IDs, comments, versions and unknown metadata. Validation finishes before installation. Duplicate model IDs reject.

| Argument         | Required | Type / constraints                                                                                                                   |
| ---------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch` | Yes      | string                                                                                                                               |
| `label`          | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `operation_id`   | Yes      | string                                                                                                                               |
| `path`           | Yes      | string; maxLength: 1024                                                                                                              |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "path": {
      "maxLength": 1024,
      "minLength": 1,
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "path"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "bounds": {
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
          "properties": {
            "max": {
              "items": {
                "maximum": 32768,
                "minimum": -32767,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "min": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }
          },
          "required": [
            "min",
            "max"
          ],
          "type": "object"
        },
        "comments": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "dirty": {
          "type": "boolean"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "layers": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "id": {
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                "type": "string"
              },
              "locked": {
                "type": "boolean"
              },
              "name": {
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "visible": {
                "type": "boolean"
              },
              "voxel_count": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              }
            },
            "required": [
              "id",
              "name",
              "visible",
              "locked",
              "voxel_count"
            ],
            "type": "object"
          },
          "maxItems": 64,
          "minItems": 1,
          "type": "array"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "name": {
          "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
          "maxLength": 128,
          "minLength": 1,
          "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
          "type": "string"
        },
        "palette_slots": {
          "maximum": 256,
          "minimum": 1,
          "type": "integer"
        },
        "redo_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "versions": {
          "maximum": 64,
          "minimum": 0,
          "type": "integer"
        },
        "voxel_count": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_document\_save

Atomically save .voxl under the project root. New paths must not exist; overwriting the same opened/saved path checks its last SHA-256. Does not create an undo entry. Changes stay in memory until saved.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `path`              | Yes      | string; maxLength: 1024                                                                                                              |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "path": {
      "maxLength": 1024,
      "minLength": 1,
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "path"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "bytes": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "dirty": {
          "const": false
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "path": {
          "maxLength": 1024,
          "minLength": 1,
          "type": "string"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "sha256": {
          "pattern": "^[0-9a-f]{64}$",
          "type": "string"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_export\_glb

Export deterministic GLB of the highest visible layer at each occupied coordinate. Atomically write a new path under project root; existing paths reject. Returns size and SHA-256; document/history unchanged.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `path`     | Yes      | string; maxLength: 1024                                               |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "path": {
      "maxLength": 1024,
      "minLength": 1,
      "type": "string"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    }
  },
  "required": [
    "model_id",
    "path"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "bytes": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "mime_type": {
          "const": "model/gltf-binary"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "path": {
          "maxLength": 1024,
          "minLength": 1,
          "type": "string"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "sha256": {
          "pattern": "^[0-9a-f]{64}$",
          "type": "string"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_export\_png

Render and atomically write one PNG under project root. Default contact sheet; for individual image use one view. Existing paths reject. Returns size, SHA-256 and camera metadata; document/history unchanged.

| Argument      | Required | Type / constraints                                                    |
| ------------- | -------- | --------------------------------------------------------------------- |
| `background`  | Optional | array; minItems: 4; maxItems: 4                                       |
| `camera`      | Optional | object                                                                |
| `height`      | Optional | integer; minimum: 1; maximum: 1024                                    |
| `image_mode`  | Optional | string; enum: \["contact\_sheet","individual"\]                       |
| `model_id`    | Yes      | string                                                                |
| `path`        | Yes      | string; maxLength: 1024                                               |
| `perspective` | Optional | boolean                                                               |
| `revision`    | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |
| `views`       | Optional | array; minItems: 1; maxItems: 6                                       |
| `width`       | Optional | integer; minimum: 1; maximum: 1024                                    |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
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
  "not": {
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
    ],
    "required": [
      "camera"
    ]
  },
  "properties": {
    "background": {
      "items": {
        "maximum": 255,
        "minimum": 0,
        "type": "integer"
      },
      "maxItems": 4,
      "minItems": 4,
      "type": "array"
    },
    "camera": {
      "additionalProperties": false,
      "oneOf": [
        {
          "not": {
            "required": [
              "fov_y_degrees"
            ]
          },
          "properties": {
            "projection": {
              "const": "orthographic"
            }
          },
          "required": [
            "ortho_height"
          ]
        },
        {
          "not": {
            "required": [
              "ortho_height"
            ]
          },
          "properties": {
            "projection": {
              "const": "perspective"
            }
          },
          "required": [
            "fov_y_degrees"
          ]
        }
      ],
      "properties": {
        "eye": {
          "items": {
            "maximum": 1000000000.0,
            "minimum": -1000000000.0,
            "type": "number"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "fov_y_degrees": {
          "exclusiveMaximum": 179,
          "exclusiveMinimum": 1,
          "type": "number"
        },
        "ortho_height": {
          "exclusiveMinimum": 0,
          "maximum": 1000000000.0,
          "type": "number"
        },
        "projection": {
          "enum": [
            "orthographic",
            "perspective"
          ],
          "type": "string"
        },
        "target": {
          "items": {
            "maximum": 1000000000.0,
            "minimum": -1000000000.0,
            "type": "number"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "up": {
          "items": {
            "maximum": 1000000000.0,
            "minimum": -1000000000.0,
            "type": "number"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "eye",
        "target",
        "up",
        "projection"
      ],
      "type": "object"
    },
    "height": {
      "maximum": 1024,
      "minimum": 1,
      "type": "integer"
    },
    "image_mode": {
      "enum": [
        "contact_sheet",
        "individual"
      ],
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "path": {
      "maxLength": 1024,
      "minLength": 1,
      "type": "string"
    },
    "perspective": {
      "type": "boolean"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "views": {
      "items": {
        "enum": [
          "iso",
          "front",
          "back",
          "left",
          "right",
          "top"
        ],
        "type": "string"
      },
      "maxItems": 6,
      "minItems": 1,
      "type": "array"
    },
    "width": {
      "maximum": 1024,
      "minimum": 1,
      "type": "integer"
    }
  },
  "required": [
    "model_id",
    "path"
  ],
  "then": {
    "anyOf": [
      {
        "required": [
          "camera"
        ]
      },
      {
        "properties": {
          "views": {
            "maxItems": 1
          }
        },
        "required": [
          "views"
        ]
      }
    ]
  },
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "backend": {
          "const": "cpu"
        },
        "bounds": {
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
          "properties": {
            "max": {
              "items": {
                "maximum": 32768,
                "minimum": -32767,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "min": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }
          },
          "required": [
            "min",
            "max"
          ],
          "type": "object"
        },
        "bytes": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "image_mode": {
          "enum": [
            "contact_sheet",
            "individual",
            "both"
          ],
          "type": "string"
        },
        "images": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "bytes": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "height": {
                "maximum": 2048,
                "minimum": 1,
                "type": "integer"
              },
              "sha256": {
                "pattern": "^[0-9a-f]{64}$",
                "type": "string"
              },
              "view": {
                "enum": [
                  "iso",
                  "front",
                  "back",
                  "left",
                  "right",
                  "top",
                  "camera",
                  "contact_sheet"
                ],
                "type": "string"
              },
              "width": {
                "maximum": 2048,
                "minimum": 1,
                "type": "integer"
              }
            },
            "required": [
              "view",
              "width",
              "height",
              "sha256",
              "bytes"
            ],
            "type": "object"
          },
          "maxItems": 7,
          "minItems": 1,
          "type": "array"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "path": {
          "maxLength": 1024,
          "minLength": 1,
          "type": "string"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "sha256": {
          "pattern": "^[0-9a-f]{64}$",
          "type": "string"
        },
        "views": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "bytes": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "camera": {
                "additionalProperties": true,
                "oneOf": [
                  {
                    "not": {
                      "required": [
                        "fov_y_degrees"
                      ]
                    },
                    "properties": {
                      "projection": {
                        "const": "orthographic"
                      }
                    },
                    "required": [
                      "ortho_height"
                    ]
                  },
                  {
                    "not": {
                      "required": [
                        "ortho_height"
                      ]
                    },
                    "properties": {
                      "projection": {
                        "const": "perspective"
                      }
                    },
                    "required": [
                      "fov_y_degrees"
                    ]
                  }
                ],
                "properties": {
                  "eye": {
                    "items": {
                      "maximum": 1000000000.0,
                      "minimum": -1000000000.0,
                      "type": "number"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  },
                  "fov_y_degrees": {
                    "exclusiveMaximum": 179,
                    "exclusiveMinimum": 1,
                    "type": "number"
                  },
                  "ortho_height": {
                    "exclusiveMinimum": 0,
                    "maximum": 1000000000.0,
                    "type": "number"
                  },
                  "projection": {
                    "enum": [
                      "orthographic",
                      "perspective"
                    ],
                    "type": "string"
                  },
                  "target": {
                    "items": {
                      "maximum": 1000000000.0,
                      "minimum": -1000000000.0,
                      "type": "number"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  },
                  "up": {
                    "items": {
                      "maximum": 1000000000.0,
                      "minimum": -1000000000.0,
                      "type": "number"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "eye",
                  "target",
                  "up",
                  "projection"
                ],
                "type": "object"
              },
              "height": {
                "maximum": 2048,
                "minimum": 1,
                "type": "integer"
              },
              "sha256": {
                "pattern": "^[0-9a-f]{64}$",
                "type": "string"
              },
              "view": {
                "enum": [
                  "iso",
                  "front",
                  "back",
                  "left",
                  "right",
                  "top",
                  "camera",
                  "contact_sheet"
                ],
                "type": "string"
              },
              "width": {
                "maximum": 2048,
                "minimum": 1,
                "type": "integer"
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
            "type": "object"
          },
          "maxItems": 6,
          "minItems": 1,
          "type": "array"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_history\_read

Read chronological applied history with agent origins, tool names, labels and change summaries. Includes exact undo/redo entry IDs. Default page 50, max 200; history is bounded and resets on reopen.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `limit`    | Optional | integer; minimum: 1; maximum: 200                                     |
| `model_id` | Yes      | string                                                                |
| `offset`   | Optional | integer; minimum: 0; maximum: 1048576                                 |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "limit": {
      "maximum": 200,
      "minimum": 1,
      "type": "integer"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "offset": {
      "maximum": 1048576,
      "minimum": 0,
      "type": "integer"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    }
  },
  "required": [
    "model_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "entries": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "added": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "base_revision": {
                "description": "Captured monotonic revision, encoded as a decimal u64 string.",
                "pattern": "^(0|[1-9][0-9]{0,19})$",
                "type": "string"
              },
              "changed_bounds": {
                "anyOf": [
                  {
                    "additionalProperties": true,
                    "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
                    "properties": {
                      "max": {
                        "items": {
                          "maximum": 32768,
                          "minimum": -32767,
                          "type": "integer"
                        },
                        "maxItems": 3,
                        "minItems": 3,
                        "type": "array"
                      },
                      "min": {
                        "items": {
                          "maximum": 32767,
                          "minimum": -32768,
                          "type": "integer"
                        },
                        "maxItems": 3,
                        "minItems": 3,
                        "type": "array"
                      }
                    },
                    "required": [
                      "min",
                      "max"
                    ],
                    "type": "object"
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "entry_id": {
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                "type": "string"
              },
              "label": {
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "origin": {
                "enum": [
                  "agent",
                  "human"
                ],
                "type": "string"
              },
              "recolored": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "removed": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "result_revision": {
                "description": "Captured monotonic revision, encoded as a decimal u64 string.",
                "pattern": "^(0|[1-9][0-9]{0,19})$",
                "type": "string"
              },
              "tool": {
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
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
            "type": "object"
          },
          "maxItems": 200,
          "minItems": 0,
          "type": "array"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "history_bytes": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "initial_retained": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "next_offset": {
          "anyOf": [
            {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            {
              "type": "null"
            }
          ]
        },
        "redo_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "total": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_history\_redo

Redo exactly the expected redo entry. Requires captured epoch/revision/entry\_id. A new edit clears the redo branch; stale IDs reject.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `entry_id`          | Yes      | string                                                                                                                               |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "entry_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "entry_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "redo_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_history\_undo

Undo exactly the expected top entry. Requires captured epoch/revision/entry\_id; intervening edits reject. Revision increases monotonically; original entry retains its agent attribution.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `entry_id`          | Yes      | string                                                                                                                               |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "entry_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "entry_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "redo_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_layers\_add

Insert an empty layer at bottom-to-top index (default append). Optional layer\_id supports deterministic references. Maximum 64 layers.

| Argument            | Required | Type / constraints                                                                                                                     |
| ------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch`    | Yes      | string                                                                                                                                 |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                  |
| `index`             | Optional | integer; minimum: 0; maximum: 64                                                                                                       |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `layer_id`          | Optional | string                                                                                                                                 |
| `model_id`          | Yes      | string                                                                                                                                 |
| `name`              | Yes      | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `operation_id`      | Yes      | string                                                                                                                                 |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "index": {
      "maximum": 64,
      "minimum": 0,
      "type": "integer"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "name": {
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "name"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "layer_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_layers\_list

List stable layer IDs, exact names, visibility, lock state and stored voxel counts in bottom-to-top order.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    }
  },
  "required": [
    "model_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "layers": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "id": {
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                "type": "string"
              },
              "locked": {
                "type": "boolean"
              },
              "name": {
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "visible": {
                "type": "boolean"
              },
              "voxel_count": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              }
            },
            "required": [
              "id",
              "name",
              "visible",
              "locked",
              "voxel_count"
            ],
            "type": "object"
          },
          "maxItems": 64,
          "minItems": 1,
          "type": "array"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        }
      },
      "required": [
        "model_id",
        "revision",
        "layers"
      ],
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_layers\_lock

Set lock explicitly. Unlock in a separate operation before voxel editing; all mutations share the core lock rules and history.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `locked`            | Yes      | boolean                                                                                                                              |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "locked": {
      "type": "boolean"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_layers\_remove

Delete one unlocked layer and its voxels as one undoable edit. Final-layer deletion rejects; anchored comments remain and may become orphaned.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "layer"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_layers\_rename

Rename a layer without changing its stable ID. Duplicate names are allowed, but future name-based access must be unambiguous.

| Argument            | Required | Type / constraints                                                                                                                     |
| ------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch`    | Yes      | string                                                                                                                                 |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                  |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                             |
| `model_id`          | Yes      | string                                                                                                                                 |
| `name`              | Yes      | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `operation_id`      | Yes      | string                                                                                                                                 |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "name": {
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_layers\_reorder

Set an exact bottom-to-top permutation of all layer IDs/names. The highest visible layer wins at overlaps; omitted or duplicate layers reject.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `order`             | Yes      | array; minItems: 1; maxItems: 64                                                                                                     |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "order": {
      "items": {
        "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
        "maxLength": 128,
        "minLength": 1,
        "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
        "type": "string"
      },
      "maxItems": 64,
      "minItems": 1,
      "type": "array"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "order"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_layers\_visibility

Set visibility explicitly. Hidden cells stay stored and cannot be edited until made visible. Metadata changes are undoable.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `visible`           | Yes      | boolean                                                                                                                              |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_palette\_apply\_preset

Append exact preset colors into unused slots without recoloring existing voxels. Returns source-to-new slot mapping. Rejects atomically if capacity would exceed 256; originals are immutable.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `preset`            | Yes      | string; enum: \["pico8","endesga32","sweetie16","grayscale","neutral","warm","vivid"\]                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "preset": {
      "enum": [
        "pico8",
        "endesga32",
        "sweetie16",
        "grayscale",
        "neutral",
        "warm",
        "vivid"
      ],
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "preset"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "slot_mapping": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "slot": {
                "maximum": 256,
                "minimum": 1,
                "type": "integer"
              },
              "source": {
                "maximum": 256,
                "minimum": 1,
                "type": "integer"
              }
            },
            "required": [
              "source",
              "slot"
            ],
            "type": "object"
          },
          "maxItems": 256,
          "minItems": 1,
          "type": "array"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_palette\_export

Return palette as .hex or .gpl text in display order; GPL preserves representable names. Read-only, no filesystem side effects.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `format`   | Yes      | string; enum: \["hex","gpl"\]                                         |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "format": {
      "enum": [
        "hex",
        "gpl"
      ],
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    }
  },
  "required": [
    "model_id",
    "format"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "format": {
          "enum": [
            "hex",
            "gpl"
          ],
          "type": "string"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "text": {
          "maxLength": 65536,
          "minLength": 1,
          "type": "string"
        }
      },
      "required": [
        "model_id",
        "revision",
        "text",
        "format"
      ],
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_palette\_import

Append colors from bounded .hex or .gpl text into unused slots; no implicit geometry recoloring. Returns source-to-new mapping; malformed input or insufficient capacity leaves palette unchanged.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `format`            | Yes      | string; enum: \["hex","gpl"\]                                                                                                        |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `text`              | Yes      | string; maxLength: 65536                                                                                                             |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "format": {
      "enum": [
        "hex",
        "gpl"
      ],
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "text": {
      "maxLength": 65536,
      "minLength": 1,
      "type": "string"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "slot_mapping": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "slot": {
                "maximum": 256,
                "minimum": 1,
                "type": "integer"
              },
              "source": {
                "maximum": 256,
                "minimum": 1,
                "type": "integer"
              }
            },
            "required": [
              "source",
              "slot"
            ],
            "type": "object"
          },
          "maxItems": 256,
          "minItems": 1,
          "type": "array"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_palette\_list

List stable palette slots, names, RGBA colors, display order and exact preset names. Slot 0 is reserved for air.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    }
  },
  "required": [
    "model_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette": {
          "additionalProperties": true,
          "properties": {
            "order": {
              "items": {
                "maximum": 256,
                "minimum": 1,
                "type": "integer"
              },
              "maxItems": 256,
              "minItems": 1,
              "type": "array"
            },
            "slots": {
              "items": {
                "additionalProperties": true,
                "properties": {
                  "index": {
                    "maximum": 256,
                    "minimum": 1,
                    "type": "integer"
                  },
                  "name": {
                    "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                    "maxLength": 128,
                    "minLength": 1,
                    "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                    "type": "string"
                  },
                  "rgba": {
                    "description": "Opaque sRGB RGBA8; alpha must equal 255.",
                    "items": false,
                    "maxItems": 4,
                    "minItems": 4,
                    "prefixItems": [
                      {
                        "maximum": 255,
                        "minimum": 0,
                        "type": "integer"
                      },
                      {
                        "maximum": 255,
                        "minimum": 0,
                        "type": "integer"
                      },
                      {
                        "maximum": 255,
                        "minimum": 0,
                        "type": "integer"
                      },
                      {
                        "const": 255
                      }
                    ],
                    "type": "array"
                  }
                },
                "required": [
                  "index",
                  "rgba"
                ],
                "type": "object"
              },
              "maxItems": 256,
              "minItems": 1,
              "type": "array"
            }
          },
          "required": [
            "slots",
            "order"
          ],
          "type": "object"
        },
        "presets": {
          "items": {
            "maxLength": 32,
            "minLength": 1,
            "type": "string"
          },
          "maxItems": 7,
          "minItems": 1,
          "type": "array"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        }
      },
      "required": [
        "model_id",
        "revision",
        "palette",
        "presets"
      ],
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_palette\_set\_slot

Add or replace stable slot 1..256 with opaque RGBA. Existing references change appearance, including locked layers. Omitted name creates an unnamed swatch. One undoable edit.

| Argument            | Required | Type / constraints                                                                                                                     |
| ------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch`    | Yes      | string                                                                                                                                 |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                  |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `model_id`          | Yes      | string                                                                                                                                 |
| `name`              | Optional | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `operation_id`      | Yes      | string                                                                                                                                 |
| `rgba`              | Yes      | array; minItems: 4; maxItems: 4. Opaque sRGB RGBA8; alpha must equal 255.                                                              |
| `slot`              | Yes      | integer; minimum: 1; maximum: 256                                                                                                      |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "name": {
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "rgba": {
      "description": "Opaque sRGB RGBA8; alpha must equal 255.",
      "items": false,
      "maxItems": 4,
      "minItems": 4,
      "prefixItems": [
        {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        {
          "const": 255
        }
      ],
      "type": "array"
    },
    "slot": {
      "maximum": 256,
      "minimum": 1,
      "type": "integer"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_query

Query bounds, counts or paged occupied voxels within a half-open region. view=layers (default) counts overlaps separately; composite uses highest visible layer. Optional layer restricts stored-layer queries. limit defaults 1000, max 10000. Use returned revision with next\_offset to prevent mixed pages.

| Argument   | Required | Type / constraints                                                                                                            |
| ---------- | -------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `kind`     | Yes      | string; enum: \["bounds","counts","voxels"\]                                                                                  |
| `layer`    | Optional | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                    |
| `limit`    | Optional | integer; minimum: 1; maximum: 10000                                                                                           |
| `model_id` | Yes      | string                                                                                                                        |
| `offset`   | Optional | integer; minimum: 0; maximum: 1048576                                                                                         |
| `region`   | Optional | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime. |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string.                                                         |
| `view`     | Optional | string; enum: \["layers","composite"\]                                                                                        |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "not": {
    "properties": {
      "view": {
        "const": "composite"
      }
    },
    "required": [
      "view",
      "layer"
    ]
  },
  "properties": {
    "kind": {
      "enum": [
        "bounds",
        "counts",
        "voxels"
      ],
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "limit": {
      "maximum": 10000,
      "minimum": 1,
      "type": "integer"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "offset": {
      "maximum": 1048576,
      "minimum": 0,
      "type": "integer"
    },
    "region": {
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
      "properties": {
        "max": {
          "items": {
            "maximum": 32768,
            "minimum": -32767,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "min": {
          "items": {
            "maximum": 32767,
            "minimum": -32768,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "min",
        "max"
      ],
      "type": "object"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "view": {
      "enum": [
        "layers",
        "composite"
      ],
      "type": "string"
    }
  },
  "required": [
    "model_id",
    "kind"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "bounds": {
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
          "properties": {
            "max": {
              "items": {
                "maximum": 32768,
                "minimum": -32767,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "min": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }
          },
          "required": [
            "min",
            "max"
          ],
          "type": "object"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "next_offset": {
          "anyOf": [
            {
              "maximum": 9223372036854775807,
              "minimum": 0,
              "type": "integer"
            },
            {
              "type": "null"
            }
          ]
        },
        "occupied_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "region": {
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
          "properties": {
            "max": {
              "items": {
                "maximum": 32768,
                "minimum": -32767,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "min": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }
          },
          "required": [
            "min",
            "max"
          ],
          "type": "object"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "slots": {
          "additionalProperties": {
            "maximum": 9223372036854775807,
            "minimum": 0,
            "type": "integer"
          },
          "propertyNames": {
            "pattern": "^[0-9]{1,3}$"
          },
          "type": "object"
        },
        "view": {
          "enum": [
            "layers",
            "composite"
          ],
          "type": "string"
        },
        "voxel_count": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "voxels": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "at": {
                "items": {
                  "maximum": 32767,
                  "minimum": -32768,
                  "type": "integer"
                },
                "maxItems": 3,
                "minItems": 3,
                "type": "array"
              },
              "layer_id": {
                "anyOf": [
                  {
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                    "type": "string"
                  },
                  {
                    "type": "null"
                  }
                ]
              },
              "slot": {
                "maximum": 256,
                "minimum": 1,
                "type": "integer"
              }
            },
            "required": [
              "layer_id",
              "at",
              "slot"
            ],
            "type": "object"
          },
          "maxItems": 10000,
          "minItems": 0,
          "type": "array"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_render

Render CPU PNG image content from one immutable revision. Default four views \[iso,front,right,back\] at 512², transparent background, orthographic, 2x AA, labeled contact sheet. Up to six views or one explicit camera; <=4,194,304 combined image/sheet pixels. image\_mode=contact\_sheet|individual|both. Returns camera metadata and hashes; no GPU/display required.

| Argument      | Required | Type / constraints                                                    |
| ------------- | -------- | --------------------------------------------------------------------- |
| `background`  | Optional | array; minItems: 4; maxItems: 4                                       |
| `camera`      | Optional | object                                                                |
| `height`      | Optional | integer; minimum: 1; maximum: 1024                                    |
| `image_mode`  | Optional | string; enum: \["contact\_sheet","individual","both"\]                |
| `model_id`    | Yes      | string                                                                |
| `perspective` | Optional | boolean                                                               |
| `revision`    | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |
| `views`       | Optional | array; minItems: 1; maxItems: 6                                       |
| `width`       | Optional | integer; minimum: 1; maximum: 1024                                    |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "not": {
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
    ],
    "required": [
      "camera"
    ]
  },
  "properties": {
    "background": {
      "items": {
        "maximum": 255,
        "minimum": 0,
        "type": "integer"
      },
      "maxItems": 4,
      "minItems": 4,
      "type": "array"
    },
    "camera": {
      "additionalProperties": false,
      "oneOf": [
        {
          "not": {
            "required": [
              "fov_y_degrees"
            ]
          },
          "properties": {
            "projection": {
              "const": "orthographic"
            }
          },
          "required": [
            "ortho_height"
          ]
        },
        {
          "not": {
            "required": [
              "ortho_height"
            ]
          },
          "properties": {
            "projection": {
              "const": "perspective"
            }
          },
          "required": [
            "fov_y_degrees"
          ]
        }
      ],
      "properties": {
        "eye": {
          "items": {
            "maximum": 1000000000.0,
            "minimum": -1000000000.0,
            "type": "number"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "fov_y_degrees": {
          "exclusiveMaximum": 179,
          "exclusiveMinimum": 1,
          "type": "number"
        },
        "ortho_height": {
          "exclusiveMinimum": 0,
          "maximum": 1000000000.0,
          "type": "number"
        },
        "projection": {
          "enum": [
            "orthographic",
            "perspective"
          ],
          "type": "string"
        },
        "target": {
          "items": {
            "maximum": 1000000000.0,
            "minimum": -1000000000.0,
            "type": "number"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "up": {
          "items": {
            "maximum": 1000000000.0,
            "minimum": -1000000000.0,
            "type": "number"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "eye",
        "target",
        "up",
        "projection"
      ],
      "type": "object"
    },
    "height": {
      "maximum": 1024,
      "minimum": 1,
      "type": "integer"
    },
    "image_mode": {
      "enum": [
        "contact_sheet",
        "individual",
        "both"
      ],
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "perspective": {
      "type": "boolean"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "views": {
      "items": {
        "enum": [
          "iso",
          "front",
          "back",
          "left",
          "right",
          "top"
        ],
        "type": "string"
      },
      "maxItems": 6,
      "minItems": 1,
      "type": "array"
    },
    "width": {
      "maximum": 1024,
      "minimum": 1,
      "type": "integer"
    }
  },
  "required": [
    "model_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "backend": {
          "const": "cpu"
        },
        "bounds": {
          "additionalProperties": true,
          "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
          "properties": {
            "max": {
              "items": {
                "maximum": 32768,
                "minimum": -32767,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            },
            "min": {
              "items": {
                "maximum": 32767,
                "minimum": -32768,
                "type": "integer"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }
          },
          "required": [
            "min",
            "max"
          ],
          "type": "object"
        },
        "image_mode": {
          "enum": [
            "contact_sheet",
            "individual",
            "both"
          ],
          "type": "string"
        },
        "images": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "bytes": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "height": {
                "maximum": 2048,
                "minimum": 1,
                "type": "integer"
              },
              "sha256": {
                "pattern": "^[0-9a-f]{64}$",
                "type": "string"
              },
              "view": {
                "enum": [
                  "iso",
                  "front",
                  "back",
                  "left",
                  "right",
                  "top",
                  "camera",
                  "contact_sheet"
                ],
                "type": "string"
              },
              "width": {
                "maximum": 2048,
                "minimum": 1,
                "type": "integer"
              }
            },
            "required": [
              "view",
              "width",
              "height",
              "sha256",
              "bytes"
            ],
            "type": "object"
          },
          "maxItems": 7,
          "minItems": 1,
          "type": "array"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "views": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "bytes": {
                "maximum": 9223372036854775807,
                "minimum": 0,
                "type": "integer"
              },
              "camera": {
                "additionalProperties": true,
                "oneOf": [
                  {
                    "not": {
                      "required": [
                        "fov_y_degrees"
                      ]
                    },
                    "properties": {
                      "projection": {
                        "const": "orthographic"
                      }
                    },
                    "required": [
                      "ortho_height"
                    ]
                  },
                  {
                    "not": {
                      "required": [
                        "ortho_height"
                      ]
                    },
                    "properties": {
                      "projection": {
                        "const": "perspective"
                      }
                    },
                    "required": [
                      "fov_y_degrees"
                    ]
                  }
                ],
                "properties": {
                  "eye": {
                    "items": {
                      "maximum": 1000000000.0,
                      "minimum": -1000000000.0,
                      "type": "number"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  },
                  "fov_y_degrees": {
                    "exclusiveMaximum": 179,
                    "exclusiveMinimum": 1,
                    "type": "number"
                  },
                  "ortho_height": {
                    "exclusiveMinimum": 0,
                    "maximum": 1000000000.0,
                    "type": "number"
                  },
                  "projection": {
                    "enum": [
                      "orthographic",
                      "perspective"
                    ],
                    "type": "string"
                  },
                  "target": {
                    "items": {
                      "maximum": 1000000000.0,
                      "minimum": -1000000000.0,
                      "type": "number"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  },
                  "up": {
                    "items": {
                      "maximum": 1000000000.0,
                      "minimum": -1000000000.0,
                      "type": "number"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "eye",
                  "target",
                  "up",
                  "projection"
                ],
                "type": "object"
              },
              "height": {
                "maximum": 2048,
                "minimum": 1,
                "type": "integer"
              },
              "sha256": {
                "pattern": "^[0-9a-f]{64}$",
                "type": "string"
              },
              "view": {
                "enum": [
                  "iso",
                  "front",
                  "back",
                  "left",
                  "right",
                  "top",
                  "camera",
                  "contact_sheet"
                ],
                "type": "string"
              },
              "width": {
                "maximum": 2048,
                "minimum": 1,
                "type": "integer"
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
            "type": "object"
          },
          "maxItems": 6,
          "minItems": 1,
          "type": "array"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_versions\_list

List named version IDs, names, parent IDs, origin and immutable content hashes. Persistent across save/open; independent of undo retention. Never includes full voxel snapshots.

| Argument   | Required | Type / constraints                                                    |
| ---------- | -------- | --------------------------------------------------------------------- |
| `model_id` | Yes      | string                                                                |
| `revision` | Optional | string. Captured monotonic revision, encoded as a decimal u64 string. |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    }
  },
  "required": [
    "model_id"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "versions": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "content_hash": {
                "pattern": "^[0-9a-f]{64}$",
                "type": "string"
              },
              "id": {
                "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                "type": "string"
              },
              "name": {
                "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "origin": {
                "enum": [
                  "agent",
                  "human"
                ],
                "type": "string"
              },
              "parent_id": {
                "anyOf": [
                  {
                    "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                    "type": "string"
                  },
                  {
                    "type": "null"
                  }
                ]
              }
            },
            "required": [
              "id",
              "name",
              "parent_id",
              "origin",
              "content_hash"
            ],
            "type": "object"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        }
      },
      "required": [
        "model_id",
        "revision",
        "versions"
      ],
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_versions\_restore

Restore named content as a new undoable edit, keeping the version library. Requires expected\_head\_id (null for empty history) in addition to revision/epoch. Core validates current locks before installation.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_head_id`  | Yes      | See schema                                                                                                                           |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `version_id`        | Yes      | string                                                                                                                               |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_head_id": {
      "anyOf": [
        {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        {
          "type": "null"
        }
      ]
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "version_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_versions\_save

Persist current content as a named immutable snapshot with agent attribution and content hash. Up to 64 versions, no recursive libraries. One undoable library edit; save document to retain across process restarts.

| Argument            | Required | Type / constraints                                                                                                                     |
| ------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `expected_epoch`    | Yes      | string                                                                                                                                 |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                  |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.   |
| `model_id`          | Yes      | string                                                                                                                                 |
| `name`              | Yes      | string; maxLength: 128. Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `operation_id`      | Yes      | string                                                                                                                                 |
| `parent_id`         | Optional | string                                                                                                                                 |
| `version_id`        | Optional | string                                                                                                                                 |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "name": {
      "description": "Nonempty text, at most 128 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "parent_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "version_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "name"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "version_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_voxels\_batch

Apply 1..256 ordered voxel operations atomically to one model: set, erase\_region, fill, box. Each operation names its own layer/mirror; nesting and I/O are forbidden. Later operations see earlier edits; any failure rolls back all; one revision and undo entry.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `operations`        | Yes      | array; minItems: 1; maxItems: 256                                                                                                    |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operations": {
      "items": {
        "oneOf": [
          {
            "additionalProperties": false,
            "properties": {
              "kind": {
                "const": "set"
              },
              "layer": {
                "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "mirror": {
                "additionalProperties": false,
                "properties": {
                  "axes": {
                    "items": {
                      "enum": [
                        "x",
                        "y",
                        "z"
                      ],
                      "type": "string"
                    },
                    "maxItems": 3,
                    "minItems": 0,
                    "type": "array",
                    "uniqueItems": true
                  },
                  "plane2": {
                    "items": {
                      "maximum": 65536,
                      "minimum": -65536,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "axes",
                  "plane2"
                ],
                "type": "object"
              },
              "mode": {
                "enum": [
                  "place",
                  "paint"
                ],
                "type": "string"
              },
              "voxels": {
                "items": {
                  "additionalProperties": false,
                  "properties": {
                    "at": {
                      "items": {
                        "maximum": 32767,
                        "minimum": -32768,
                        "type": "integer"
                      },
                      "maxItems": 3,
                      "minItems": 3,
                      "type": "array"
                    },
                    "slot": {
                      "anyOf": [
                        {
                          "maximum": 256,
                          "minimum": 0,
                          "type": "integer"
                        },
                        {
                          "maxLength": 128,
                          "minLength": 1,
                          "type": "string"
                        }
                      ],
                      "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
                    }
                  },
                  "required": [
                    "at",
                    "slot"
                  ],
                  "type": "object"
                },
                "maxItems": 65536,
                "minItems": 1,
                "type": "array"
              }
            },
            "required": [
              "layer",
              "voxels",
              "kind"
            ],
            "type": "object"
          },
          {
            "additionalProperties": false,
            "properties": {
              "kind": {
                "const": "erase_region"
              },
              "layer": {
                "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "mirror": {
                "additionalProperties": false,
                "properties": {
                  "axes": {
                    "items": {
                      "enum": [
                        "x",
                        "y",
                        "z"
                      ],
                      "type": "string"
                    },
                    "maxItems": 3,
                    "minItems": 0,
                    "type": "array",
                    "uniqueItems": true
                  },
                  "plane2": {
                    "items": {
                      "maximum": 65536,
                      "minimum": -65536,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "axes",
                  "plane2"
                ],
                "type": "object"
              },
              "region": {
                "additionalProperties": false,
                "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
                "properties": {
                  "max": {
                    "items": {
                      "maximum": 32768,
                      "minimum": -32767,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  },
                  "min": {
                    "items": {
                      "maximum": 32767,
                      "minimum": -32768,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "min",
                  "max"
                ],
                "type": "object"
              }
            },
            "required": [
              "layer",
              "region",
              "kind"
            ],
            "type": "object"
          },
          {
            "additionalProperties": false,
            "properties": {
              "cap": {
                "maximum": 1048576,
                "minimum": 1,
                "type": "integer"
              },
              "kind": {
                "const": "fill"
              },
              "layer": {
                "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "mirror": {
                "additionalProperties": false,
                "properties": {
                  "axes": {
                    "items": {
                      "enum": [
                        "x",
                        "y",
                        "z"
                      ],
                      "type": "string"
                    },
                    "maxItems": 3,
                    "minItems": 0,
                    "type": "array",
                    "uniqueItems": true
                  },
                  "plane2": {
                    "items": {
                      "maximum": 65536,
                      "minimum": -65536,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "axes",
                  "plane2"
                ],
                "type": "object"
              },
              "region": {
                "additionalProperties": false,
                "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
                "properties": {
                  "max": {
                    "items": {
                      "maximum": 32768,
                      "minimum": -32767,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  },
                  "min": {
                    "items": {
                      "maximum": 32767,
                      "minimum": -32768,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "min",
                  "max"
                ],
                "type": "object"
              },
              "seed": {
                "items": {
                  "maximum": 32767,
                  "minimum": -32768,
                  "type": "integer"
                },
                "maxItems": 3,
                "minItems": 3,
                "type": "array"
              },
              "slot": {
                "anyOf": [
                  {
                    "maximum": 256,
                    "minimum": 1,
                    "type": "integer"
                  },
                  {
                    "maxLength": 128,
                    "minLength": 1,
                    "type": "string"
                  }
                ],
                "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
              }
            },
            "required": [
              "layer",
              "seed",
              "slot",
              "kind"
            ],
            "type": "object"
          },
          {
            "additionalProperties": false,
            "else": {
              "required": [
                "slot"
              ]
            },
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
            "properties": {
              "hollow": {
                "type": "boolean"
              },
              "kind": {
                "const": "box"
              },
              "layer": {
                "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
                "maxLength": 128,
                "minLength": 1,
                "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
                "type": "string"
              },
              "mirror": {
                "additionalProperties": false,
                "properties": {
                  "axes": {
                    "items": {
                      "enum": [
                        "x",
                        "y",
                        "z"
                      ],
                      "type": "string"
                    },
                    "maxItems": 3,
                    "minItems": 0,
                    "type": "array",
                    "uniqueItems": true
                  },
                  "plane2": {
                    "items": {
                      "maximum": 65536,
                      "minimum": -65536,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "axes",
                  "plane2"
                ],
                "type": "object"
              },
              "mode": {
                "enum": [
                  "place",
                  "paint",
                  "erase"
                ],
                "type": "string"
              },
              "region": {
                "additionalProperties": false,
                "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
                "properties": {
                  "max": {
                    "items": {
                      "maximum": 32768,
                      "minimum": -32767,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  },
                  "min": {
                    "items": {
                      "maximum": 32767,
                      "minimum": -32768,
                      "type": "integer"
                    },
                    "maxItems": 3,
                    "minItems": 3,
                    "type": "array"
                  }
                },
                "required": [
                  "min",
                  "max"
                ],
                "type": "object"
              },
              "slot": {
                "anyOf": [
                  {
                    "maximum": 256,
                    "minimum": 1,
                    "type": "integer"
                  },
                  {
                    "maxLength": 128,
                    "minLength": 1,
                    "type": "string"
                  }
                ],
                "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
              }
            },
            "required": [
              "layer",
              "region",
              "kind"
            ],
            "then": {
              "not": {
                "required": [
                  "slot"
                ]
              }
            },
            "type": "object"
          }
        ]
      },
      "maxItems": 256,
      "minItems": 1,
      "type": "array"
    }
  },
  "required": [
    "expected_epoch",
    "operation_id",
    "model_id",
    "expected_revision",
    "operations"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_voxels\_box

Build an integer half-open solid box or one-cell shell (hollow=true). Default mode=place, hollow=false, mirror=none. slot required for place/paint, forbidden for erase. One atomic undo entry.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `hollow`            | Optional | boolean                                                                                                                              |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `mirror`            | Optional | object                                                                                                                               |
| `mode`              | Optional | string; enum: \["place","paint","erase"\]                                                                                            |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `region`            | Yes      | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.        |
| `slot`              | Optional | See schema. Stable slot number or unique swatch name; zero is air for explicit writes only.                                          |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "else": {
    "required": [
      "slot"
    ]
  },
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
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "hollow": {
      "type": "boolean"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "mirror": {
      "additionalProperties": false,
      "properties": {
        "axes": {
          "items": {
            "enum": [
              "x",
              "y",
              "z"
            ],
            "type": "string"
          },
          "maxItems": 3,
          "minItems": 0,
          "type": "array",
          "uniqueItems": true
        },
        "plane2": {
          "items": {
            "maximum": 65536,
            "minimum": -65536,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "axes",
        "plane2"
      ],
      "type": "object"
    },
    "mode": {
      "enum": [
        "place",
        "paint",
        "erase"
      ],
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "region": {
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
      "properties": {
        "max": {
          "items": {
            "maximum": 32768,
            "minimum": -32767,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "min": {
          "items": {
            "maximum": 32767,
            "minimum": -32768,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "min",
        "max"
      ],
      "type": "object"
    },
    "slot": {
      "anyOf": [
        {
          "maximum": 256,
          "minimum": 1,
          "type": "integer"
        },
        {
          "maxLength": 128,
          "minLength": 1,
          "type": "string"
        }
      ],
      "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
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
  "then": {
    "not": {
      "required": [
        "slot"
      ]
    }
  },
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_voxels\_erase\_region

Erase occupied cells in min-inclusive/max-exclusive region. Mirror is explicit per call; omitted means none. Invalid/hidden/locked targets reject without partial edits.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `mirror`            | Optional | object                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `region`            | Yes      | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.        |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "mirror": {
      "additionalProperties": false,
      "properties": {
        "axes": {
          "items": {
            "enum": [
              "x",
              "y",
              "z"
            ],
            "type": "string"
          },
          "maxItems": 3,
          "minItems": 0,
          "type": "array",
          "uniqueItems": true
        },
        "plane2": {
          "items": {
            "maximum": 65536,
            "minimum": -65536,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "axes",
        "plane2"
      ],
      "type": "object"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "region": {
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
      "properties": {
        "max": {
          "items": {
            "maximum": 32768,
            "minimum": -32767,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "min": {
          "items": {
            "maximum": 32767,
            "minimum": -32768,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "min",
        "max"
      ],
      "type": "object"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_voxels\_fill

Six-connected flood fill of the seed's slot, including air. Region defaults to document bounds; cap defaults to 1,048,576 visited cells. Cap exhaustion rejects atomically. Mirror never inherits UI state.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `cap`               | Optional | integer; minimum: 1; maximum: 1048576                                                                                                |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `mirror`            | Optional | object                                                                                                                               |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `region`            | Optional | object. Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.        |
| `seed`              | Yes      | array; minItems: 3; maxItems: 3                                                                                                      |
| `slot`              | Yes      | See schema. Stable slot number or unique swatch name; zero is air for explicit writes only.                                          |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "cap": {
      "maximum": 1048576,
      "minimum": 1,
      "type": "integer"
    },
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "mirror": {
      "additionalProperties": false,
      "properties": {
        "axes": {
          "items": {
            "enum": [
              "x",
              "y",
              "z"
            ],
            "type": "string"
          },
          "maxItems": 3,
          "minItems": 0,
          "type": "array",
          "uniqueItems": true
        },
        "plane2": {
          "items": {
            "maximum": 65536,
            "minimum": -65536,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "axes",
        "plane2"
      ],
      "type": "object"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "region": {
      "additionalProperties": false,
      "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
      "properties": {
        "max": {
          "items": {
            "maximum": 32768,
            "minimum": -32767,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        },
        "min": {
          "items": {
            "maximum": 32767,
            "minimum": -32768,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "min",
        "max"
      ],
      "type": "object"
    },
    "seed": {
      "items": {
        "maximum": 32767,
        "minimum": -32768,
        "type": "integer"
      },
      "maxItems": 3,
      "minItems": 3,
      "type": "array"
    },
    "slot": {
      "anyOf": [
        {
          "maximum": 256,
          "minimum": 1,
          "type": "integer"
        },
        {
          "maxLength": 128,
          "minLength": 1,
          "type": "string"
        }
      ],
      "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_voxels\_get

Read explicit integer coordinates on one layer, including air as slot 0. Returns values in request order. Out-of-bounds coordinates reject the entire read.

| Argument    | Required | Type / constraints                                                                         |
| ----------- | -------- | ------------------------------------------------------------------------------------------ |
| `layer`     | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection. |
| `model_id`  | Yes      | string                                                                                     |
| `positions` | Yes      | array; minItems: 1; maxItems: 10000                                                        |
| `revision`  | Optional | string. Captured monotonic revision, encoded as a decimal u64 string.                      |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "positions": {
      "items": {
        "items": {
          "maximum": 32767,
          "minimum": -32768,
          "type": "integer"
        },
        "maxItems": 3,
        "minItems": 3,
        "type": "array"
      },
      "maxItems": 10000,
      "minItems": 1,
      "type": "array"
    },
    "revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    }
  },
  "required": [
    "model_id",
    "layer",
    "positions"
  ],
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "layer_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "voxels": {
          "items": {
            "additionalProperties": true,
            "properties": {
              "at": {
                "items": {
                  "maximum": 32767,
                  "minimum": -32768,
                  "type": "integer"
                },
                "maxItems": 3,
                "minItems": 3,
                "type": "array"
              },
              "slot": {
                "maximum": 256,
                "minimum": 0,
                "type": "integer"
              }
            },
            "required": [
              "at",
              "slot"
            ],
            "type": "object"
          },
          "maxItems": 10000,
          "minItems": 1,
          "type": "array"
        }
      },
      "required": [
        "model_id",
        "revision",
        "layer_id",
        "voxels"
      ],
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```

### voxelite\_voxels\_set

Atomically place or paint explicit cells, at most 65,536. Slot 0 erases; paint affects occupied cells only. Last duplicate wins before mirror expansion; mirror conflicts reject. Omitted mode=place, mirror=none. One undo entry.

| Argument            | Required | Type / constraints                                                                                                                   |
| ------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `expected_epoch`    | Yes      | string                                                                                                                               |
| `expected_revision` | Yes      | string. Captured monotonic revision, encoded as a decimal u64 string.                                                                |
| `label`             | Optional | string; maxLength: 80. Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime. |
| `layer`             | Yes      | string; maxLength: 128. Layer UUID or unique exact name. Never inferred from UI selection.                                           |
| `mirror`            | Optional | object                                                                                                                               |
| `mode`              | Optional | string; enum: \["place","paint"\]                                                                                                    |
| `model_id`          | Yes      | string                                                                                                                               |
| `operation_id`      | Yes      | string                                                                                                                               |
| `voxels`            | Yes      | array; minItems: 1; maxItems: 65536                                                                                                  |

Complete input schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "expected_epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "expected_revision": {
      "description": "Captured monotonic revision, encoded as a decimal u64 string.",
      "pattern": "^(0|[1-9][0-9]{0,19})$",
      "type": "string"
    },
    "label": {
      "description": "Nonempty text, at most 80 UTF-8 bytes, without control characters. The byte limit is also checked at runtime.",
      "maxLength": 80,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "layer": {
      "description": "Layer UUID or unique exact name. Never inferred from UI selection.",
      "maxLength": 128,
      "minLength": 1,
      "pattern": "^[^\\u0000-\\u001f\\u007f-\\u009f]+$",
      "type": "string"
    },
    "mirror": {
      "additionalProperties": false,
      "properties": {
        "axes": {
          "items": {
            "enum": [
              "x",
              "y",
              "z"
            ],
            "type": "string"
          },
          "maxItems": 3,
          "minItems": 0,
          "type": "array",
          "uniqueItems": true
        },
        "plane2": {
          "items": {
            "maximum": 65536,
            "minimum": -65536,
            "type": "integer"
          },
          "maxItems": 3,
          "minItems": 3,
          "type": "array"
        }
      },
      "required": [
        "axes",
        "plane2"
      ],
      "type": "object"
    },
    "mode": {
      "enum": [
        "place",
        "paint"
      ],
      "type": "string"
    },
    "model_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "operation_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "voxels": {
      "items": {
        "additionalProperties": false,
        "properties": {
          "at": {
            "items": {
              "maximum": 32767,
              "minimum": -32768,
              "type": "integer"
            },
            "maxItems": 3,
            "minItems": 3,
            "type": "array"
          },
          "slot": {
            "anyOf": [
              {
                "maximum": 256,
                "minimum": 0,
                "type": "integer"
              },
              {
                "maxLength": 128,
                "minLength": 1,
                "type": "string"
              }
            ],
            "description": "Stable slot number or unique swatch name; zero is air for explicit writes only."
          }
        },
        "required": [
          "at",
          "slot"
        ],
        "type": "object"
      },
      "maxItems": 65536,
      "minItems": 1,
      "type": "array"
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
  "type": "object"
}
```

Complete output schema

```json
{
  "additionalProperties": false,
  "oneOf": [
    {
      "not": {
        "required": [
          "error"
        ]
      },
      "properties": {
        "ok": {
          "const": true
        }
      },
      "required": [
        "data"
      ]
    },
    {
      "not": {
        "required": [
          "data"
        ]
      },
      "properties": {
        "ok": {
          "const": false
        }
      },
      "required": [
        "error"
      ]
    }
  ],
  "properties": {
    "api_version": {
      "const": "voxelite.v1"
    },
    "data": {
      "additionalProperties": true,
      "properties": {
        "added": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "changed_bounds": {
          "anyOf": [
            {
              "additionalProperties": true,
              "description": "Integer region: min inclusive, max exclusive. Each extent must be positive and at most 256 cells; checked at runtime.",
              "properties": {
                "max": {
                  "items": {
                    "maximum": 32768,
                    "minimum": -32767,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                },
                "min": {
                  "items": {
                    "maximum": 32767,
                    "minimum": -32768,
                    "type": "integer"
                  },
                  "maxItems": 3,
                  "minItems": 3,
                  "type": "array"
                }
              },
              "required": [
                "min",
                "max"
              ],
              "type": "object"
            },
            {
              "type": "null"
            }
          ]
        },
        "changed_layers": {
          "items": {
            "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            "type": "string"
          },
          "maxItems": 64,
          "minItems": 0,
          "type": "array"
        },
        "changed_voxels": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "durability": {
          "const": "memory"
        },
        "head_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "metadata_changed": {
          "type": "boolean"
        },
        "model_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "operation_id": {
          "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
          "type": "string"
        },
        "palette_changed": {
          "type": "boolean"
        },
        "recolored": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "removed": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
        },
        "revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "undo_entry_id": {
          "anyOf": [
            {
              "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
              "type": "string"
            },
            {
              "type": "null"
            }
          ]
        },
        "work_units": {
          "maximum": 9223372036854775807,
          "minimum": 0,
          "type": "integer"
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
      "type": "object"
    },
    "epoch": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    },
    "error": {
      "additionalProperties": false,
      "properties": {
        "code": {
          "maxLength": 80,
          "minLength": 1,
          "type": "string"
        },
        "command_index": {
          "maximum": 255,
          "minimum": 0,
          "type": "integer"
        },
        "current_revision": {
          "description": "Captured monotonic revision, encoded as a decimal u64 string.",
          "pattern": "^(0|[1-9][0-9]{0,19})$",
          "type": "string"
        },
        "message": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "path": {
          "type": "string"
        },
        "recovery": {
          "maxLength": 4096,
          "minLength": 1,
          "type": "string"
        },
        "retryable": {
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "message",
        "retryable",
        "recovery"
      ],
      "type": "object"
    },
    "ok": {
      "type": "boolean"
    },
    "session_id": {
      "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
      "type": "string"
    }
  },
  "required": [
    "api_version",
    "session_id",
    "epoch",
    "ok"
  ],
  "type": "object"
}
```
