# 0001: Record architecture decisions

Date: 2026-10-26

Status: Accepted for documentation; the refactors below remain recommendations.

## Context

The Voxelite architecture review of commit `09b29d3` identified overlapping app
and MCP lifecycle responsibilities and a large `EditorView`, with no glossary or
ADR directory to record shared terminology and decisions.

## Decision

Keep domain language in [GLOSSARY.md](../../GLOSSARY.md) and record architecture
decisions as numbered Markdown files in `docs/adr/`. Distinguish existing code
from proposed seams so naming a boundary does not imply it has been implemented.

Record the review's recommended order, retaining its candidate numbers:

1. **5 — Glossary:** establish shared names first.
2. **2 — DocumentSession:** share document lifecycle and save safety between app
   and MCP, guarded by the existing editor save and MCP contract tests.
3. **1 — Split EditorView:** extract Viewport, then CommandPalette, then SaveFlow.
4. **3 — Command registry:** unify app command metadata, enablement and execution.
5. **4 — Attachment polling seam:** likely absorb this into DocumentSession so
   the session owns the engine and emits events.

## Consequences

This decision adds documentation only; it does not implement the proposed
refactors or change runtime behavior. Future work should preserve the review's
healthy boundaries: the pure core engine, portable format and renderer's CPU/GPU
split.
