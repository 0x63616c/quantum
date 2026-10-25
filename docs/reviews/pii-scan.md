# Repository and Git history personal-info scan

Scanned on 2026-10-25. Values are deliberately redacted; locations and categories are sufficient for follow-up. No history was rewritten and no Git identity was changed.

Scope: 471 working-tree/generated files; 120 commits and 2839 unique historical blobs reachable through all local refs and reflogs. Includes `design/` read-only, source, documentation, lockfiles, assets and the generated wasm bundles. Scans commit messages plus every author/committer header. Git metadata pointer files are checked separately below.

Excluded dependency caches and build intermediates (`node_modules/`, `target/`, `target-webdeploy/`): these are not repository deliverables. Binary assets were byte-scanned; this is not OCR, a full secret entropy detector, or proof that every possible personal name/secret was found. No remote access or credential validation was attempted.

## Git metadata

- 4 distinct author/committer identity pairs are present across 120 commits. Names, addresses and account identifiers are redacted here. These immutable headers remain as requested; inspect locally with `git log --all --format=fuller` if authorized.
- `.git:1` is a worktree pointer containing a local machine path. It is untracked Git plumbing, not a distributable source file; changing it would break the checkout.

## Working tree findings

| File:line | Category | Disposition |
| --- | --- | --- |
| `LICENSE:3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `PLAN.md:1,27,55,139,144,198,262,286,290,309,350,351,360,392,414,415,434,641,642,645,647,693,697,700,729,742,748,830,835` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/Cargo.toml:17` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/README.md:1,3,18,19,104,167,199,228` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/assets/fonts/Geist.ttf:793` | email | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/assets/icons/LICENSE:3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/src/actions.rs:96` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/src/main.rs:1,64,131,140` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/src/palettes.rs:33` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/src/persistence.rs:86` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/src/share.rs:58,166` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/src/ui.rs:1198,2735` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-app/src/ui/agent.rs:104` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-core/src/lib.rs:1` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-core/src/palette.rs:225,227,229` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-export/src/lib.rs:70,103` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-format/src/lib.rs:21,179,206,208` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-format/tests/roundtrip.rs:40` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-mcp/Cargo.toml:7` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-mcp/src/catalog.rs:213,331,466` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-mcp/src/lib.rs:1,27,95` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-mcp/src/main.rs:26` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-mcp/tests/stdio.rs:92` | Git identity name/username | Inspect; do not reproduce matched value. |
| `crates/voxelite-site/src/main.rs:61,63,64,167` | Git identity name/username | Inspect; do not reproduce matched value. |
| `design/ART-CHECKLIST.md:1,27` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/assets/fonts/Geist.ttf:793` | email | Read-only design ownership; retained and reported. |
| `design/concepts/README.md:1` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/concept-a/index.html:2,1916,1926,3295` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/concept-b/index.html:2,599` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/concept-c/index.html:2,599` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v2/concept-c-dark.png:815` | email | Read-only design ownership; retained and reported. |
| `design/concepts/history/v5/README.md:1` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v5/concept-a/index.html:2,599` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v5/concept-b/index.html:2,599` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v5/concept-c/index.html:2,599` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v6/README.md:1` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v6/concept-a/index.html:2,1479,1489,3231` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v6/concept-b/index.html:2,599` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v6/concept-c/index.html:2,599` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/history/v7/concept-a/index.html:2,1677,1687,3044` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/concepts/png/concept-a-dark-history.png:2353` | email | Read-only design ownership; retained and reported. |
| `design/plan-history/PLAN-v1.md:1,3,55,125,129,139,309,365,371` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/plan-history/PLAN-v2.md:1,27,91,96,191,234,235,264,285,439,440,443,487,491,494,519,538,611,616` | Git identity name/username | Read-only design ownership; retained and reported. |
| `design/plan-history/PLAN-v3.md:1,27,94,99,204,249,250,259,289,311,312,493,494,497,499,544,548,551,580,593,599,679,684` | Git identity name/username | Read-only design ownership; retained and reported. |
| `docs/architecture/core.md:16` | Git identity name/username | Inspect; do not reproduce matched value. |
| `docs/mcp.md:1,121,150,263` | Git identity name/username | Inspect; do not reproduce matched value. |
| `docs/reviews/app-polish-2.md:166` | Git identity name/username | Inspect; do not reproduce matched value. |
| `docs/screenshots/autumn-workshop.glb:1` | Git identity name/username | Inspect; do not reproduce matched value. |
| `docs/verification/native-app.md:11,18` | Git identity name/username | Inspect; do not reproduce matched value. |
| `examples/autumn-workshop.voxl:28` | Git identity name/username | Inspect; do not reproduce matched value. |
| `integrations/README.md:17,22,29` | Git identity name/username | Inspect; do not reproduce matched value. |
| `integrations/claude/mcp.json:3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `integrations/codex/config.toml:1` | Git identity name/username | Inspect; do not reproduce matched value. |
| `integrations/cursor/mcp.json:3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `package-lock.json:2,8` | Git identity name/username | Inspect; do not reproduce matched value. |
| `package.json:2,4` | Git identity name/username | Inspect; do not reproduce matched value. |
| `scripts/build-concepts.mjs:58,105` | Git identity name/username | Inspect; do not reproduce matched value. |
| `scripts/capture-native-scales.sh:33,37` | Git identity name/username | Inspect; do not reproduce matched value. |
| `scripts/capture-native.sh:35,39,63` | Git identity name/username | Inspect; do not reproduce matched value. |
| `scripts/lib/capture-share.sh:13` | Git identity name/username | Inspect; do not reproduce matched value. |
| `scripts/lib/concept-a-controls.js:937` | Git identity name/username | Inspect; do not reproduce matched value. |
| `scripts/lib/concept-a.mjs:25,35` | Git identity name/username | Inspect; do not reproduce matched value. |
| `scripts/review-native.sh:6,28,32,75,80,85,89,96,101,104,108,124,128,142,144,146,148,161` | Git identity name/username | Inspect; do not reproduce matched value. |
| `scripts/review-polish-2.sh:28,32` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/404.html:3,5,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/app/index.html:7,8,19` | Git identity name/username | Ignored generated bundle; evaluate reproducible-build path remapping separately. |
| `site/app/voxelite_app_bg.wasm:130885` | email | Ignored generated bundle; evaluate reproducible-build path remapping separately. |
| `site/app/voxelite_app_bg.wasm:72619` | absolute machine path | Ignored generated bundle; evaluate reproducible-build path remapping separately. |
| `site/app/voxelite_app_bg.wasm:72575,72627,72749,72772,73886` | Git identity name/username | Ignored generated bundle; evaluate reproducible-build path remapping separately. |
| `site/assets/LICENSE:3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/fonts/Geist.ttf:793` | email | Inspect; do not reproduce matched value. |
| `site/assets/integrations/README.md:17,22,29` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/integrations/claude/mcp.json:3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/integrations/codex/config.toml:1` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/integrations/cursor/mcp.json:3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/mcp-tools.json:9,54,60,561,868,1241,1543,1845,2168,2450,2745,3051,3227,3586,4001,4338,4642,4958,5232,5452,5747,6050,6350,6649,6949,7252,7447,7742,8036,8380,8866,9254,9490,9793,10534,10943,11304,11696,11989,12282` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/minimal.voxl:10` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/skills/README.md:1,3,36,54` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/skills/voxelite-palettes/SKILL.md:3,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/skills/voxelite-primitives/SKILL.md:3,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/skills/voxelite-render-review/SKILL.md:3,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/assets/skills/voxelite-workflow/SKILL.md:3,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/content/format.html:1,2,15` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/content/getting-started.html:4,14,19,21,22` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/content/mcp.html:1` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/content/overview.html:1,4` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/content/skills.html:1,3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/docs/format/index.html:3,5,6,7,20,33` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/docs/getting-started/index.html:3,5,6,9,19,24,26,27,65` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/docs/index.html:3,5,6,9,10` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/docs/mcp/index.html:3,5,6,11,53,59,548,843,1204,1494,1784,2095,2365,2648,2942,3106,3453,3856,4181,4473,4777,5039,5247,5530,5821,6109,6396,6684,6975,7158,7441,7723,8055,8529,8905,9129,9420,10149,10546,10895,11275,11556,11837,12040` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/docs/skills/index.html:3,5,6,8,9` | Git identity name/username | Inspect; do not reproduce matched value. |
| `site/index.html:5,7,12,15,29,30` | Git identity name/username | Inspect; do not reproduce matched value. |
| `skills/README.md:1,3,36,54` | Git identity name/username | Inspect; do not reproduce matched value. |
| `skills/voxelite-palettes/SKILL.md:3,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `skills/voxelite-primitives/SKILL.md:3,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `skills/voxelite-render-review/SKILL.md:3,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `skills/voxelite-workflow/SKILL.md:3,6` | Git identity name/username | Inspect; do not reproduce matched value. |
| `spec/mcp/v1/compatibility.json:2` | Git identity name/username | Inspect; do not reproduce matched value. |
| `spec/mcp/v1/contract.md:1,12` | Git identity name/username | Inspect; do not reproduce matched value. |
| `spec/mcp/v1/tools.json:4,48,54,555,862,1235,1537,1839,2162,2444,2739,3045,3221,3580,3995,4332,4636,4952,5226,5446,5741,6044,6344,6643,6943,7246,7441,7736,8030,8374,8860,9248,9484,9787,10528,10937,11298,11690,11983,12276` | Git identity name/username | Inspect; do not reproduce matched value. |
| `spec/voxl/1.0/README.md:3` | Git identity name/username | Inspect; do not reproduce matched value. |
| `spec/voxl/1.0/fixtures/populated.canonical.voxl:10` | Git identity name/username | Inspect; do not reproduce matched value. |
| `spec/voxl/1.0/fixtures/populated.input.voxl:1` | Git identity name/username | Inspect; do not reproduce matched value. |
| `spec/voxl/1.0/schema.json:3,15` | Git identity name/username | Inspect; do not reproduce matched value. |
| `web/dist/index.html:7,8,19` | Git identity name/username | Ignored generated bundle; evaluate reproducible-build path remapping separately. |
| `web/dist/voxelite_app_bg.wasm:130885` | email | Ignored generated bundle; evaluate reproducible-build path remapping separately. |
| `web/dist/voxelite_app_bg.wasm:72619` | absolute machine path | Ignored generated bundle; evaluate reproducible-build path remapping separately. |
| `web/dist/voxelite_app_bg.wasm:72575,72627,72749,72772,73886` | Git identity name/username | Ignored generated bundle; evaluate reproducible-build path remapping separately. |
| `web/index.html:7,8,19` | Git identity name/username | Inspect; do not reproduce matched value. |

## Historical blob findings

One row per unique blob/category; a blob can occur in many commits. Use `git show <blob>` locally to inspect a finding. No matched values are copied into this report.

| Blob · file:line | Category |
| --- | --- |
| `d31807356d9a` · `LICENSE:3` | Git identity name/username |
| `0a8274603156` · `PLAN.md:1,27,55,139,144,198,262,286,290,309,350,351,360,392,414,415,434,641,642,645,647,693,697,700,729,742,748,830,835` | Git identity name/username |
| `5fcc0fe8b92c` · `crates/voxelite-app/Cargo.toml:17` | Git identity name/username |
| `0077396158ef` · `crates/voxelite-app/README.md:1,3,18,19,104,167,199,228` | Git identity name/username |
| `99be836ad05d` · `crates/voxelite-app/assets/fonts/Geist.ttf:793` | email |
| `325e8ff0aae2` · `crates/voxelite-app/assets/icons/LICENSE:3` | Git identity name/username |
| `35c7d20b9e6a` · `crates/voxelite-app/src/actions.rs:96` | Git identity name/username |
| `e53ab9bb4344` · `crates/voxelite-app/src/main.rs:1,64,131,140` | Git identity name/username |
| `2f9e1574ca86` · `crates/voxelite-app/src/palettes.rs:33` | Git identity name/username |
| `c530e246b4d6` · `crates/voxelite-app/src/persistence.rs:86` | Git identity name/username |
| `ef4cefb25f3d` · `crates/voxelite-app/src/share.rs:58,166` | Git identity name/username |
| `76872e23529d` · `crates/voxelite-app/src/ui.rs:1198,2735` | Git identity name/username |
| `7450d4c1a155` · `crates/voxelite-app/src/ui/agent.rs:104` | Git identity name/username |
| `d287402847fa` · `crates/voxelite-core/src/lib.rs:1` | Git identity name/username |
| `53f842fa9337` · `crates/voxelite-core/src/palette.rs:225,227,229` | Git identity name/username |
| `ca85f8785b41` · `crates/voxelite-export/src/lib.rs:70,103` | Git identity name/username |
| `10a4b41c2665` · `crates/voxelite-format/src/lib.rs:21,179,206,208` | Git identity name/username |
| `9e41faf732b7` · `crates/voxelite-format/tests/roundtrip.rs:40` | Git identity name/username |
| `44fb5c1fbf87` · `crates/voxelite-mcp/Cargo.toml:7` | Git identity name/username |
| `3772532e9661` · `crates/voxelite-mcp/src/catalog.rs:213,331,466` | Git identity name/username |
| `103ef2da7eac` · `crates/voxelite-mcp/src/lib.rs:1,27,95` | Git identity name/username |
| `3fef042b140e` · `crates/voxelite-mcp/src/main.rs:26` | Git identity name/username |
| `6e5b25793899` · `crates/voxelite-mcp/tests/stdio.rs:92` | Git identity name/username |
| `8d8e43a9e70b` · `crates/voxelite-site/src/main.rs:61,63,64,167` | Git identity name/username |
| `5ba95d573b4c` · `design/ART-CHECKLIST.md:1,27` | Git identity name/username |
| `86750f9cc5d2` · `design/concepts/README.md:1` | Git identity name/username |
| `5a40d1ae64fe` · `design/concepts/concept-a/index.html:2,1916,1926,3295` | Git identity name/username |
| `654d96961085` · `design/concepts/concept-b/index.html:2,599` | Git identity name/username |
| `ff8633508919` · `design/concepts/concept-c/index.html:2,599` | Git identity name/username |
| `78be1e3e9d08` · `design/concepts/history/v2/concept-c-dark.png:815` | email |
| `a50361c85767` · `design/concepts/history/v5/README.md:1` | Git identity name/username |
| `371681bd8589` · `design/concepts/history/v5/concept-a/index.html:2,599` | Git identity name/username |
| `38def5993ac0` · `design/concepts/history/v6/README.md:1` | Git identity name/username |
| `9f529724f478` · `design/concepts/history/v6/concept-a/index.html:2,1479,1489,3231` | Git identity name/username |
| `7f659033dbbc` · `design/concepts/history/v7/concept-a/index.html:2,1677,1687,3044` | Git identity name/username |
| `4e44bfa3e485` · `design/concepts/png/concept-a-dark-history.png:2353` | email |
| `d6834508408e` · `design/plan-history/PLAN-v1.md:1,3,55,125,129,139,309,365,371` | Git identity name/username |
| `4824c75e2577` · `design/plan-history/PLAN-v2.md:1,27,91,96,191,234,235,264,285,439,440,443,487,491,494,519,538,611,616` | Git identity name/username |
| `0f782c6e80be` · `design/plan-history/PLAN-v3.md:1,27,94,99,204,249,250,259,289,311,312,493,494,497,499,544,548,551,580,593,599,679,684` | Git identity name/username |
| `a0973168db20` · `docs/architecture/core.md:16` | Git identity name/username |
| `13bcbb53527b` · `docs/mcp.md:1,121,150,263` | Git identity name/username |
| `01f260380d03` · `docs/reviews/app-polish-2.md:166` | Git identity name/username |
| `c4798230e3b5` · `docs/screenshots/autumn-workshop.glb:1` | Git identity name/username |
| `dab1d36addbe` · `docs/verification/native-app.md:11,18` | Git identity name/username |
| `4624b700a9fd` · `examples/autumn-workshop.voxl:28` | Git identity name/username |
| `9f4eef6ab599` · `integrations/README.md:17,22,29` | Git identity name/username |
| `a023fe987a25` · `integrations/claude/mcp.json:3` | Git identity name/username |
| `f20dfe0ff361` · `integrations/codex/config.toml:1` | Git identity name/username |
| `d31752eeba22` · `integrations/cursor/mcp.json:3` | Git identity name/username |
| `9607eee17a0f` · `package-lock.json:2,8` | Git identity name/username |
| `0ee4d23b9585` · `package.json:2,4` | Git identity name/username |
| `b20fac740515` · `scripts/build-concepts.mjs:58,105` | Git identity name/username |
| `9cc2426adad3` · `scripts/capture-native-scales.sh:33,37` | Git identity name/username |
| `1989881c03b2` · `scripts/capture-native.sh:35,39,63` | Git identity name/username |
| `8678779ef4cc` · `scripts/lib/capture-share.sh:13` | Git identity name/username |
| `b3dd2ef5cbe8` · `scripts/lib/concept-a-controls.js:937` | Git identity name/username |
| `271d91ecbfd2` · `scripts/lib/concept-a.mjs:25,35` | Git identity name/username |
| `7017eb815c98` · `scripts/review-native.sh:6,28,32,75,80,85,89,96,101,104,108,124,128,142,144,146,148,161` | Git identity name/username |
| `9030ec1bce23` · `scripts/review-polish-2.sh:28,32` | Git identity name/username |
| `047f7b296941` · `site/404.html:3,5,6` | Git identity name/username |
| `132c1e40d610` · `site/assets/mcp-tools.json:9,54,60,561,868,1241,1543,1845,2168,2450,2745,3051,3227,3586,4001,4338,4642,4958,5232,5452,5747,6050,6350,6649,6949,7252,7447,7742,8036,8380,8866,9254,9490,9793,10534,10943,11304,11696,11989,12282` | Git identity name/username |
| `716eef91a7c1` · `site/assets/minimal.voxl:10` | Git identity name/username |
| `425336a2648f` · `site/assets/skills/README.md:1,3,36,54` | Git identity name/username |
| `e363a582b898` · `site/assets/skills/voxelite-palettes/SKILL.md:3,6` | Git identity name/username |
| `8442ade327a2` · `site/assets/skills/voxelite-primitives/SKILL.md:3,6` | Git identity name/username |
| `d1bb88573085` · `site/assets/skills/voxelite-render-review/SKILL.md:3,6` | Git identity name/username |
| `90c69550f4d0` · `site/assets/skills/voxelite-workflow/SKILL.md:3,6` | Git identity name/username |
| `d592682e25ef` · `site/content/format.html:1,2,15` | Git identity name/username |
| `0675afcc7964` · `site/content/getting-started.html:4,14,19,21,22` | Git identity name/username |
| `dfcfb2ed581b` · `site/content/mcp.html:1` | Git identity name/username |
| `bb9c6f45e5b2` · `site/content/overview.html:1,4` | Git identity name/username |
| `d4b3e0dc44f2` · `site/content/skills.html:1,3` | Git identity name/username |
| `8ab12205ce5f` · `site/docs/format/index.html:3,5,6,7,20,33` | Git identity name/username |
| `8989966cd7e1` · `site/docs/getting-started/index.html:3,5,6,9,19,24,26,27,65` | Git identity name/username |
| `a8519eacd62e` · `site/docs/index.html:3,5,6,9,10` | Git identity name/username |
| `5e7b0b1196cd` · `site/docs/mcp/index.html:3,5,6,11,53,59,548,843,1204,1494,1784,2095,2365,2648,2942,3106,3453,3856,4181,4473,4777,5039,5247,5530,5821,6109,6396,6684,6975,7158,7441,7723,8055,8529,8905,9129,9420,10149,10546,10895,11275,11556,11837,12040` | Git identity name/username |
| `5af91b7beb52` · `site/docs/skills/index.html:3,5,6,8,9` | Git identity name/username |
| `6d834237bab1` · `site/index.html:5,7,12,15,29,30` | Git identity name/username |
| `40059a519e2a` · `spec/mcp/v1/compatibility.json:2` | Git identity name/username |
| `df247710e094` · `spec/mcp/v1/contract.md:1,12` | Git identity name/username |
| `92cd38b8a814` · `spec/mcp/v1/tools.json:4,48,54,555,862,1235,1537,1839,2162,2444,2739,3045,3221,3580,3995,4332,4636,4952,5226,5446,5741,6044,6344,6643,6943,7246,7441,7736,8030,8374,8860,9248,9484,9787,10528,10937,11298,11690,11983,12276` | Git identity name/username |
| `9564be32b76f` · `spec/voxl/1.0/README.md:3` | Git identity name/username |
| `bbb3fcdfada8` · `spec/voxl/1.0/fixtures/populated.canonical.voxl:10` | Git identity name/username |
| `beb7334975c1` · `spec/voxl/1.0/fixtures/populated.input.voxl:1` | Git identity name/username |
| `b34f1a608663` · `spec/voxl/1.0/schema.json:3,15` | Git identity name/username |
| `b8d07805a356` · `web/index.html:7,8,19` | Git identity name/username |
| `1d3a6f5f0326` · `site/index.html:5,7,12,15,29,30` | Git identity name/username |
| `23eca4103fe6` · `site/assets/skills/voxelite-workflow/SKILL.md:3,6` | Git identity name/username |
| `747170834966` · `site/content/getting-started.html:4,14,19,21,22` | Git identity name/username |
| `f37da3193efe` · `site/docs/getting-started/index.html:3,5,6,9,19,24,26,27,60` | Git identity name/username |
| `c6276d759369` · `crates/voxelite-site/src/main.rs:32,34,35,87` | Git identity name/username |
| `1ea123ba489b` · `site/index.html:5,7,12,15,29,30` | Git identity name/username |
| `fe7e86da348e` · `target-webdeploy/.rustc_info.json:1` | absolute machine path |
| `7106f3e7d6ba` · `target-webdeploy/wasm32-unknown-unknown/web/.fingerprint/voxelite-app-a1e0bf24d878e85a/dep-lib-voxelite_app:1` | absolute machine path |
| `1598c5aa6ac3` · `target-webdeploy/wasm32-unknown-unknown/web/build/anyhow-fe5737645ec3b66a/root-output:1` | absolute machine path |
| `859e5eb5b70c` · `target-webdeploy/wasm32-unknown-unknown/web/build/crc32fast-7b4c4b46f7edd3e9/root-output:1` | absolute machine path |
| `7bf4409ffa99` · `target-webdeploy/wasm32-unknown-unknown/web/build/crossbeam-deque-b1eceae6df7d937f/root-output:1` | absolute machine path |
| `221d3a860850` · `target-webdeploy/wasm32-unknown-unknown/web/build/crossbeam-epoch-ae29aa7827304038/root-output:1` | absolute machine path |
| `40a92f9d3d03` · `target-webdeploy/wasm32-unknown-unknown/web/build/crossbeam-utils-acb71a5b4cbfca65/root-output:1` | absolute machine path |
| `f4165d3583a8` · `target-webdeploy/wasm32-unknown-unknown/web/build/generic-array-baa558690f83cafc/root-output:1` | absolute machine path |
| `6cf5940e37f2` · `target-webdeploy/wasm32-unknown-unknown/web/build/getrandom-8d0ca61f15f95976/root-output:1` | absolute machine path |
| `65064480c4c3` · `target-webdeploy/wasm32-unknown-unknown/web/build/gpui-unofficial-95368884aeaf6788/root-output:1` | absolute machine path |
| `25e74c0c24bf` · `target-webdeploy/wasm32-unknown-unknown/web/build/heapless-616b38219f869e34/root-output:1` | absolute machine path |
| `2e48b58bfe11` · `target-webdeploy/wasm32-unknown-unknown/web/build/icu_normalizer_data-cf414be5173b7094/root-output:1` | absolute machine path |
| `af69749b603d` · `target-webdeploy/wasm32-unknown-unknown/web/build/icu_properties_data-8fc6dad3cc699248/root-output:1` | absolute machine path |
| `588ec121dc12` · `target-webdeploy/wasm32-unknown-unknown/web/build/libc-f8a4d1b122d2e2fd/root-output:1` | absolute machine path |
| `c9dca0d70a78` · `target-webdeploy/wasm32-unknown-unknown/web/build/libm-d36ee378f7a8a7f0/root-output:1` | absolute machine path |
| `ea2feedab776` · `target-webdeploy/wasm32-unknown-unknown/web/build/naga-c365b84d685a650b/root-output:1` | absolute machine path |
| `f116f0cb30dd` · `target-webdeploy/wasm32-unknown-unknown/web/build/num-traits-f9b9a9b854900dac/root-output:1` | absolute machine path |
| `deb6e0438e53` · `target-webdeploy/wasm32-unknown-unknown/web/build/object-5ac9a3b7350eb375/root-output:1` | absolute machine path |
| `c6f320366747` · `target-webdeploy/wasm32-unknown-unknown/web/build/parking_lot_core-5bcd52db8123e8ca/root-output:1` | absolute machine path |
| `71047d0234b1` · `target-webdeploy/wasm32-unknown-unknown/web/build/pulp-6f7bbf3ba0cb1fa6/root-output:1` | absolute machine path |
| `fbfa224c8ec3` · `target-webdeploy/wasm32-unknown-unknown/web/build/rayon-core-9b57e40e6df25bb6/root-output:1` | absolute machine path |
| `31a2a17556db` · `target-webdeploy/wasm32-unknown-unknown/web/build/ref-cast-7147267ab1df321f/root-output:1` | absolute machine path |
| `12985fd6a746` · `target-webdeploy/wasm32-unknown-unknown/web/build/serde-d78a58b099a0d120/root-output:1` | absolute machine path |
| `a6e47e19241e` · `target-webdeploy/wasm32-unknown-unknown/web/build/serde_core-ef5baecbf9e74c48/root-output:1` | absolute machine path |
| `7b762415a036` · `target-webdeploy/wasm32-unknown-unknown/web/build/serde_json-5d1f673e0518f1ec/root-output:1` | absolute machine path |
| `de13ecab51ab` · `target-webdeploy/wasm32-unknown-unknown/web/build/slotmap-3a215d6d9f1e1203/root-output:1` | absolute machine path |
| `c435428ea139` · `target-webdeploy/wasm32-unknown-unknown/web/build/thiserror-d46690f9eb748d83/root-output:1` | absolute machine path |
| `4dd0e29fe9c6` · `target-webdeploy/wasm32-unknown-unknown/web/build/wasm-bindgen-8d1725b4dee1a809/root-output:1` | absolute machine path |
| `f9a61fdd34a4` · `target-webdeploy/wasm32-unknown-unknown/web/build/wasm-bindgen-shared-466f0de4b14f4870/output:1,2` | absolute machine path |
| `c95275c60a47` · `target-webdeploy/wasm32-unknown-unknown/web/build/wasm-bindgen-shared-466f0de4b14f4870/root-output:1` | absolute machine path |
| `14864d5e6c98` · `target-webdeploy/wasm32-unknown-unknown/web/build/wgpu-7d8bd54ca7ca75cb/root-output:1` | absolute machine path |
| `b6801e4f37be` · `target-webdeploy/wasm32-unknown-unknown/web/build/wgpu-core-e94e7dbc2624b3aa/root-output:1` | absolute machine path |
| `03ed90f6a91e` · `target-webdeploy/wasm32-unknown-unknown/web/build/wgpu-hal-86a28ec8f10d7dcf/root-output:1` | absolute machine path |
| `3d5261726a78` · `target-webdeploy/wasm32-unknown-unknown/web/build/zerocopy-c940bd2093e07e77/root-output:1` | absolute machine path |
| `525aac062693` · `target-webdeploy/wasm32-unknown-unknown/web/build/zmij-dc8f4dd456ccffa2/root-output:1` | absolute machine path |
| `121711d6a849` · `target-webdeploy/wasm32-unknown-unknown/web/build/ztracing-gpui-unofficial-f920d7056ff382dc/root-output:1` | absolute machine path |
| `c56ee3ad18cd` · `target-webdeploy/wasm32-unknown-unknown/web/deps/accesskit-72962d940fa0bc82.d:1,3,5,7,8` | absolute machine path |
| `1a277ea9a956` · `target-webdeploy/wasm32-unknown-unknown/web/deps/addr2line-f36ecd4e1c36b4ed.d:1,3,5,7,8,9,10,11,12` | absolute machine path |
| `a407d880ab07` · `target-webdeploy/wasm32-unknown-unknown/web/deps/adler2-d9205a46a951c58c.d:1,3,5,7,8` | absolute machine path |
| `8d675dd037f0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/aho_corasick-acfbd24cf9c9e03a.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35` | absolute machine path |
| `9a11d4d0e4dd` · `target-webdeploy/wasm32-unknown-unknown/web/deps/anyhow-a37f2e30000ee41d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17` | absolute machine path |
| `e58c0e3cdf97` · `target-webdeploy/wasm32-unknown-unknown/web/deps/arrayref-def321366f2a3221.d:1,3,5,7` | absolute machine path |
| `e220d6e26559` · `target-webdeploy/wasm32-unknown-unknown/web/deps/arrayvec-8b60ec0a1445ed20.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `3334035aaf61` · `target-webdeploy/wasm32-unknown-unknown/web/deps/async_channel-12f318aa9ba67c68.d:1,3,5,7` | absolute machine path |
| `60e6440a7a37` · `target-webdeploy/wasm32-unknown-unknown/web/deps/async_task-f7c105b2bd547895.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `ef235ee48055` · `target-webdeploy/wasm32-unknown-unknown/web/deps/backtrace-195a9198bd526d48.d:1,3,5,7,8,9,10,11,12,13,14` | absolute machine path |
| `dc0f9057661a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/base64-dba26f02de0206ba.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22` | absolute machine path |
| `4e92a7ef5f7e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/bit_field-c733a6e606f75551.d:1,3,5,7` | absolute machine path |
| `c784dcb343b0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/bit_set-fe498fa5e45c6f03.d:1,3,5,7` | absolute machine path |
| `844927c88713` · `target-webdeploy/wasm32-unknown-unknown/web/deps/bit_vec-e491c65703108950.d:1,3,5,7` | absolute machine path |
| `640c236ff782` · `target-webdeploy/wasm32-unknown-unknown/web/deps/bitflags-04483e19dcbf9a0b.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `9ee652b4d8ca` · `target-webdeploy/wasm32-unknown-unknown/web/deps/bitflags-709308348910a8af.d:1,3,5,7` | absolute machine path |
| `8912a879b9a8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/block_buffer-ea0698cec20ff4a7.d:1,3,5,7,8` | absolute machine path |
| `624a953fdd6e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/bytemuck-668636f59b42b24a.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20` | absolute machine path |
| `ddc14f1fc6e8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/byteorder-dfe442aa1b32ddcd.d:1,3,5,7` | absolute machine path |
| `83c9a4528afe` · `target-webdeploy/wasm32-unknown-unknown/web/deps/byteorder_lite-a65faadf68e4c69d.d:1,3,5,7,8` | absolute machine path |
| `9cea13e8e0a1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/bytes-f2de1e383f5ee818.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24` | absolute machine path |
| `664f30d48311` · `target-webdeploy/wasm32-unknown-unknown/web/deps/cfg_if-47a91e697e93cb5f.d:1,3,5,7` | absolute machine path |
| `71c01047e9a7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/chrono-3f3991832ab6cbb0.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35` | absolute machine path |
| `1cba70c3e864` · `target-webdeploy/wasm32-unknown-unknown/web/deps/codespan_reporting-f54da9b4bc7044cc.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `bc25bb480dbe` · `target-webdeploy/wasm32-unknown-unknown/web/deps/collections_gpui_unofficial-315ad8e45f4c2306.d:1,3,5,7,8` | absolute machine path |
| `3b757c64deb8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/color_quant-e96b99a910223afc.d:1,3,5,7,8` | absolute machine path |
| `e980c366671b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/concurrent_queue-356951378fe623c0.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `974c05bebb6c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/console_error_panic_hook-53ce67e276f687c3.d:1,3,5,7` | absolute machine path |
| `1b57915f3cf7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/core_maths-8f13727a91ca4bfe.d:1,3,5,7` | absolute machine path |
| `639fcb585c29` · `target-webdeploy/wasm32-unknown-unknown/web/deps/cosmic_text-231f460ab99d89a4.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27` | absolute machine path |
| `de7f4b0e8225` · `target-webdeploy/wasm32-unknown-unknown/web/deps/crc32fast-aea6d1dfda07dd9f.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `c6a074534bbe` · `target-webdeploy/wasm32-unknown-unknown/web/deps/crossbeam_deque-f629a30110b1f469.d:1,3,5,7,8` | absolute machine path |
| `bff6023ae2ac` · `target-webdeploy/wasm32-unknown-unknown/web/deps/crossbeam_epoch-b42d69dd984bb638.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18` | absolute machine path |
| `7e0fdfc23855` · `target-webdeploy/wasm32-unknown-unknown/web/deps/crossbeam_utils-411613110992b82d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `05b2abece67d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/crypto_common-962e10aa5c60551f.d:1,3,5,7` | absolute machine path |
| `9530da580545` · `target-webdeploy/wasm32-unknown-unknown/web/deps/ctor-5ab4a2b02c1822b9.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18` | absolute machine path |
| `6b9aa490499e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/data_url-39162354f93fb2aa.d:1,3,5,7,8,9` | absolute machine path |
| `b064c72eff7c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/derive_more-393cf85328c0a8ea.d:1,3,5,7,8,9,10` | absolute machine path |
| `df9afd348cf6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/digest-f995b97431ce9e4b.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `2731cc94bb05` · `target-webdeploy/wasm32-unknown-unknown/web/deps/dyn_clone-a718b5527d5f42b6.d:1,3,5,7,8` | absolute machine path |
| `74fbfcdbcb9e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/either-34e716935586d135.d:1,3,5,7,8,9` | absolute machine path |
| `a592d5dd2ac5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/equivalent-577ff8398ac685d5.d:1,3,5,7` | absolute machine path |
| `d2e054e7b456` · `target-webdeploy/wasm32-unknown-unknown/web/deps/etagere-01684ac51f9da86b.d:1,3,5,7,8,9` | absolute machine path |
| `d304936e1738` · `target-webdeploy/wasm32-unknown-unknown/web/deps/euclid-e1a2cf8cb84d4232.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28` | absolute machine path |
| `f346132bb696` · `target-webdeploy/wasm32-unknown-unknown/web/deps/event_listener-ae2197d196c1faf3.d:1,3,5,7,8,9` | absolute machine path |
| `fee45849eb66` · `target-webdeploy/wasm32-unknown-unknown/web/deps/event_listener_strategy-5b11c6b155384e91.d:1,3,5,7` | absolute machine path |
| `181a2b1f6eee` · `target-webdeploy/wasm32-unknown-unknown/web/deps/exr-052ac3b52b7b79b0.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53` | absolute machine path |
| `8e602ff1283c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/fax-b0d42ec9ced48de2.d:1,3,5,7,8,9,10,11,12` | absolute machine path |
| `d89eb9b30245` · `target-webdeploy/wasm32-unknown-unknown/web/deps/fdeflate-637e035e9a274e55.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `e063df20e05b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/flate2-b096a72acf57387a.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26` | absolute machine path |
| `85d1a2652698` · `target-webdeploy/wasm32-unknown-unknown/web/deps/float_cmp-89f1b69b9036bbf4.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `a71e8c121103` · `target-webdeploy/wasm32-unknown-unknown/web/deps/float_next_after-02fd77e18483a5c6.d:1,3,5,7` | absolute machine path |
| `8bad63b69811` · `target-webdeploy/wasm32-unknown-unknown/web/deps/flume-dab6467cfefd6408.d:1,3,5,7,8,9` | absolute machine path |
| `8be9177aa22f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/foldhash-cbdb1f643612d5fc.d:1,3,5,7,8,9,10` | absolute machine path |
| `1a5108c1305a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/font_types-4e8facf3dfb9604d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22` | absolute machine path |
| `30effc6200ad` · `target-webdeploy/wasm32-unknown-unknown/web/deps/font_types-eb67a18a8f4e7a4d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21` | absolute machine path |
| `d7a45ebe2642` · `target-webdeploy/wasm32-unknown-unknown/web/deps/fontdb-8eb86dd40f4a099e.d:1,3,5,7` | absolute machine path |
| `d210c80c01c0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/form_urlencoded-1d998a2b8e72bd5c.d:1,3,5,7` | absolute machine path |
| `7ba9610dd71b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/futures-e6c4e9e219cc8f8e.d:1,3,5,7` | absolute machine path |
| `66cc136b4f33` · `target-webdeploy/wasm32-unknown-unknown/web/deps/futures_channel-33b7d20424f85d38.d:1,3,5,7,8,9,10,11,12` | absolute machine path |
| `e70c5a9d3f2d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/futures_core-99ecb25813b5e77b.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `90c27f2122d7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/futures_executor-1000b14f4ed90363.d:1,3,5,7,8,9` | absolute machine path |
| `f2dede20b14c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/futures_io-7399d0d83c0e453a.d:1,3,5,7` | absolute machine path |
| `cbfc09628700` · `target-webdeploy/wasm32-unknown-unknown/web/deps/futures_sink-accb1e2730709709.d:1,3,5,7` | absolute machine path |
| `7d86b7d8c4b5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/futures_task-201cca4148e7b727.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `73caa1938ef1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/futures_util-4fabbc897642c0e3.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,181,182,183` | absolute machine path |
| `e37c8b8d5119` · `target-webdeploy/wasm32-unknown-unknown/web/deps/generic_array-05e943d6b826e76e.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `1811dc322070` · `target-webdeploy/wasm32-unknown-unknown/web/deps/getrandom-d9eac8cfb620c88e.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `59a04acd55d2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/gif-1b0f8ed19d571860.d:1,3,5,7,8,9,10,11,12,13,14` | absolute machine path |
| `2241b25f01d8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/gimli-36a4f7d64e10a6e5.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35` | absolute machine path |
| `8388c2a7d14c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/glam-55fa7a0b48eca2af.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120` | absolute machine path |
| `412718514dd6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/glow-b98221d0a62b757e.d:1,3,5,7,8,9` | absolute machine path |
| `f2386baf6cac` · `target-webdeploy/wasm32-unknown-unknown/web/deps/gpui-bc07bef071e9ae75.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77` | absolute machine path |
| `74a728835d39` · `target-webdeploy/wasm32-unknown-unknown/web/deps/gpui_shared_string_gpui_unofficial-f9608ed24d1423c6.d:1,3,5,7` | absolute machine path |
| `23f2e9f74c93` · `target-webdeploy/wasm32-unknown-unknown/web/deps/gpui_util_gpui_unofficial-4a4b8c98b51431ae.d:1,3,5,7,8` | absolute machine path |
| `2bf2a7231f0d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/gpui_web_gpui_unofficial-9b29a8b146cffba0.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22` | absolute machine path |
| `b8258a0ed85d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/gpui_wgpu_gpui_unofficial-41354d151d8a36c6.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `e3d34850bca0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/half-f4fcb665e236790b.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `4b6ed75808fa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/harfrust-9a57bd105c69d522.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75` | absolute machine path |
| `91fd161e5c88` · `target-webdeploy/wasm32-unknown-unknown/web/deps/hash32-6060ecbc024a4b25.d:1,3,5,7,8,9` | absolute machine path |
| `85c2d9c6670b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/hashbrown-cf0ce1937955ee9a.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22` | absolute machine path |
| `0eb43186c535` · `target-webdeploy/wasm32-unknown-unknown/web/deps/heapless-83aa0dc6382dffee.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30` | absolute machine path |
| `3f0349130cdc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/http-e8757ef0e4590cc3.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26` | absolute machine path |
| `9fd05b3d02b3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/http_body-86b0eedde9c8d90b.d:1,3,5,7,8,9` | absolute machine path |
| `bf878b0a5177` · `target-webdeploy/wasm32-unknown-unknown/web/deps/http_client_gpui_unofficial-90d27b8161b7be77.d:1,3,5,7,8` | absolute machine path |
| `b7fc2dad7705` · `target-webdeploy/wasm32-unknown-unknown/web/deps/icu_collections-e14c955852b3fd6a.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `e5b550d53e29` · `target-webdeploy/wasm32-unknown-unknown/web/deps/icu_locale_core-db2e9959c4a56369.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66` | absolute machine path |
| `a678b0708bc2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/icu_normalizer-56e32d6a47915628.d:1,3,5,7,8,9,10` | absolute machine path |
| `ebb743901f2d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/icu_normalizer_data-7a52c40c3b23286e.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `60eeb00257ce` · `target-webdeploy/wasm32-unknown-unknown/web/deps/icu_properties-2acc8a9a49a83fb9.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `2232fc959e8a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/icu_properties_data-68a40bd8768b91a3.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149` | absolute machine path |
| `44d915613a03` · `target-webdeploy/wasm32-unknown-unknown/web/deps/icu_provider-7ff6b176f883b7d3.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `2f7f7101dbf3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/idna-5f30f4f7b5fb1259.d:1,3,5,7,8,9,10` | absolute machine path |
| `3d43c2cacff5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/idna_adapter-a87e0fc20e5ff473.d:1,3,5,7` | absolute machine path |
| `238d8eedfca1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/image-c7776a87e39cabf0.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73` | absolute machine path |
| `4fd4cc34162b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/image_webp-0cb710f3bbfa2ea1.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `352168fede30` · `target-webdeploy/wasm32-unknown-unknown/web/deps/imagesize-9aee81c9f6e6ec86.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31` | absolute machine path |
| `3e47179b68b3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/indexmap-e1fa959837c21661.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26` | absolute machine path |
| `18db7f0c4b9a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/inventory-2e602b72739f50e9.d:1,3,5,7` | absolute machine path |
| `c952e73e1a32` · `target-webdeploy/wasm32-unknown-unknown/web/deps/itertools-6481b6772170f6a3.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56` | absolute machine path |
| `61ddb2c84844` · `target-webdeploy/wasm32-unknown-unknown/web/deps/itoa-b871a4945f476c59.d:1,3,5,7,8` | absolute machine path |
| `e3e76295bff9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/js_sys-5f0837a473ec879a.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `8a8d269b2be2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/kurbo-6602e7737e61791b.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37` | absolute machine path |
| `390aa44c7432` · `target-webdeploy/wasm32-unknown-unknown/web/deps/lebe-b952d7044526dd74.d:1,3,5,7` | absolute machine path |
| `890dffbc3248` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libaccesskit-72962d940fa0bc82.rlib:3988,4071,4104` | absolute machine path |
| `bd0a377be851` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libaccesskit-72962d940fa0bc82.rmeta:3966,4049` | absolute machine path |
| `cb31711cb547` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libaddr2line-f36ecd4e1c36b4ed.rlib:1751,1761,1762,1769,1781,1789,1821` | absolute machine path |
| `3e0caa55016a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libaddr2line-f36ecd4e1c36b4ed.rmeta:1742,1752,1753,1760,1772,1780` | absolute machine path |
| `36d39b57433d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libadler2-d9205a46a951c58c.rlib:54,57` | absolute machine path |
| `2a1447b8ed2f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libadler2-d9205a46a951c58c.rmeta:49,52` | absolute machine path |
| `2df43ae06daf` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libaho_corasick-acfbd24cf9c9e03a.rlib:12262,12284,12291,12308,12310,12434,12442,12461,12467,12503,12517,12554,12583,12584,12616,12617,12653,12656,12670,12676,12714,12716,12718,12721,12800,12873,13054,13116,13255,13328,13413,13430,13448,13463,13539,13569,13595,13596,13728,13765` | absolute machine path |
| `98999589886f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libaho_corasick-acfbd24cf9c9e03a.rmeta:12242,12264,12271,12288,12290,12414,12422,12441,12447,12483,12497,12534,12563,12564,12596,12597,12633,12636,12650,12656,12694,12696,12698,12701` | absolute machine path |
| `42ebd7475e37` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libanyhow-a37f2e30000ee41d.rlib:1778,1779,1784,1798,1843,1852,1893,1894,1906,1917,1919,1939` | absolute machine path |
| `d3f20987c66a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libanyhow-a37f2e30000ee41d.rmeta:1758,1759,1764,1778,1823,1832,1873,1874,1886,1897,1899` | absolute machine path |
| `e68c118142d7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libarrayref-def321366f2a3221.rlib:33` | absolute machine path |
| `44b000a643d4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libarrayref-def321366f2a3221.rmeta:28` | absolute machine path |
| `c29a8cc5e351` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libarrayvec-8b60ec0a1445ed20.rlib:1334,1337,1370,1384,1390,1391,1394` | absolute machine path |
| `e4ad52ab86b8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libarrayvec-8b60ec0a1445ed20.rmeta:1329,1332,1365,1379,1385,1386,1389` | absolute machine path |
| `7ec4fce61630` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libasync_channel-12f318aa9ba67c68.rlib:700,737` | absolute machine path |
| `95ca49badd4e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libasync_channel-12f318aa9ba67c68.rmeta:695` | absolute machine path |
| `a17f7500e2c0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libasync_task-f7c105b2bd547895.rlib:958,964,979,984,987,1012,1044` | absolute machine path |
| `e0c4bcc607b5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libasync_task-f7c105b2bd547895.rmeta:952,958,973,978,981,1006` | absolute machine path |
| `aade5fb6488a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbacktrace-195a9198bd526d48.rlib:303,308,321,325,344,361,459,480,487` | absolute machine path |
| `628c3706bc65` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbacktrace-195a9198bd526d48.rmeta:283,288,301,305,324,341` | absolute machine path |
| `f65609de50de` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbase64-dba26f02de0206ba.rlib:1468,1473,1475,1478,1484,1501,1507,1516,1524,1528,1537,1549,1555,1588,1601,1624,1631,1649` | absolute machine path |
| `730e6ad24938` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbase64-dba26f02de0206ba.rmeta:1450,1455,1457,1460,1466,1483,1489,1498,1506,1510,1519,1531,1537` | absolute machine path |
| `ccabfce950b0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbit_field-c733a6e606f75551.rlib:365` | absolute machine path |
| `460aa435e88d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbit_field-c733a6e606f75551.rmeta:360` | absolute machine path |
| `c84cd8bfa479` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbit_set-fe498fa5e45c6f03.rlib:450` | absolute machine path |
| `562bb821af14` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbit_set-fe498fa5e45c6f03.rmeta:445` | absolute machine path |
| `4f2008f97a45` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbit_vec-e491c65703108950.rlib:1286,1378` | absolute machine path |
| `611ef01fb727` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbit_vec-e491c65703108950.rmeta:1280` | absolute machine path |
| `2160063328bc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbitflags-04483e19dcbf9a0b.rlib:1345,1354,1373,1401,1411,1427,1450` | absolute machine path |
| `1dd152c3cf0a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbitflags-04483e19dcbf9a0b.rmeta:1339,1348,1367,1395,1405,1421,1444` | absolute machine path |
| `a0b6b3922be3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbitflags-709308348910a8af.rlib:87` | absolute machine path |
| `17fd973e809c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbitflags-709308348910a8af.rmeta:82` | absolute machine path |
| `93ebee19932a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libblock_buffer-ea0698cec20ff4a7.rlib:232,235` | absolute machine path |
| `abf070139e34` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libblock_buffer-ea0698cec20ff4a7.rmeta:227,230` | absolute machine path |
| `17b23e7010e8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbytemuck-668636f59b42b24a.rlib:2365,2391,2413,2431,2449,2450` | absolute machine path |
| `df8ac380c5a7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbytemuck-668636f59b42b24a.rmeta:2358,2384,2406,2424,2442,2443` | absolute machine path |
| `fe97142092e6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbyteorder-dfe442aa1b32ddcd.rlib:1072,1126` | absolute machine path |
| `168f86e9fd8d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbyteorder-dfe442aa1b32ddcd.rmeta:1067` | absolute machine path |
| `ce2c25a22a59` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbyteorder_lite-a65faadf68e4c69d.rlib:1818,1855,1864` | absolute machine path |
| `a199de07b880` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbyteorder_lite-a65faadf68e4c69d.rmeta:1812,1849` | absolute machine path |
| `cec9ba5b0a61` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbytes-f2de1e383f5ee818.rlib:9203,9235,9281,9283,9286,9392,9395,9403,9469,9471,9472,9476,9477,9479,9482,9503,9580,9669` | absolute machine path |
| `00d3e4d5e97c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libbytes-f2de1e383f5ee818.rmeta:9185,9217,9263,9265,9268,9374,9377,9385,9451,9453,9454,9458,9459,9461,9464` | absolute machine path |
| `9fd39450d03c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libc-0a0ff83f482c6190.d:1,3,5,7,8,9,10` | absolute machine path |
| `d82ec2798295` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcfg_if-47a91e697e93cb5f.rlib:47` | absolute machine path |
| `9d5460570eee` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcfg_if-47a91e697e93cb5f.rmeta:41` | absolute machine path |
| `615e6d4654ff` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libchrono-3f3991832ab6cbb0.rlib:10094,10112,10120,10123,10133,10232,10242,10259,10288,10298,10309,10324,10326,10377,10381,10473,10488,10490,10504,10507,10512,10516,10521,10543,10553,10569,10584,10602,10732,10791,10885,10950,11003,11018,11105,11163,11202,11237,11305,11317,11337,11346` | absolute machine path |
| `d22a6b59feb0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libchrono-3f3991832ab6cbb0.rmeta:10074,10092,10100,10103,10113,10212,10222,10239,10268,10278,10289,10304,10306,10357,10361,10453,10468,10470,10484,10487,10492,10496,10501,10523,10533,10549,10564,10582` | absolute machine path |
| `41dc73a4486a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcodespan_reporting-f54da9b4bc7044cc.rlib:997,1001,1008,1030,1049,1050,1062,1120,1124,1169` | absolute machine path |
| `245d8774edca` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcodespan_reporting-f54da9b4bc7044cc.rmeta:981,985,992,1014,1033,1034,1046` | absolute machine path |
| `c7d670123d59` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcollections_gpui_unofficial-315ad8e45f4c2306.rlib:219` | absolute machine path |
| `69099b9b7858` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcollections_gpui_unofficial-315ad8e45f4c2306.rmeta:214` | absolute machine path |
| `fefec3af4e86` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcolor_quant-e96b99a910223afc.rlib:78,92,143` | absolute machine path |
| `f48436c9ed77` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcolor_quant-e96b99a910223afc.rmeta:69,83` | absolute machine path |
| `e841f58c0802` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libconcurrent_queue-356951378fe623c0.rlib:1032,1056,1064,1071,1075` | absolute machine path |
| `8f2e5aabea71` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libconcurrent_queue-356951378fe623c0.rmeta:1027,1051,1059,1066,1070` | absolute machine path |
| `3d700694d517` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libconsole_error_panic_hook-53ce67e276f687c3.rlib:124` | absolute machine path |
| `3d998fd1cf3a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libconsole_error_panic_hook-53ce67e276f687c3.rmeta:116` | absolute machine path |
| `d34681cf40e8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcore_maths-8f13727a91ca4bfe.rlib:661` | absolute machine path |
| `8603504869d3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcore_maths-8f13727a91ca4bfe.rmeta:656` | absolute machine path |
| `b56f796f8ab6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcosmic_text-231f460ab99d89a4.rlib:15617,15633,15681,15692,15699,15711,15712,15725,15731,15733,15789,15791,15795,15798,15802,15808,15810,15819,15825,15832,15833,17120,17121,17162,17375,17414,17986,17989,18028,18615,18720,18910,19306,19631,19713,19944,20040` | absolute machine path |
| `ac6e19e4fd2e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcosmic_text-231f460ab99d89a4.rmeta:15597,15613,15661,15672,15679,15691,15692,15705,15711,15713,15769,15771,15775,15778,15782,15788,15790,15799,15805,15812,15813` | absolute machine path |
| `0f4fdd2b3cd2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrc32fast-aea6d1dfda07dd9f.rlib:151,153,156,158,159,166` | absolute machine path |
| `2e2f1581d9fa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrc32fast-aea6d1dfda07dd9f.rmeta:143,145,148,150,151` | absolute machine path |
| `9b6db2e8f066` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrossbeam_deque-f629a30110b1f469.rlib:1062,1122` | absolute machine path |
| `8c681326b2ee` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrossbeam_deque-f629a30110b1f469.rmeta:1057,1117` | absolute machine path |
| `3b4a7e74162e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrossbeam_epoch-b42d69dd984bb638.rlib:1150,1156,1191,1217,1219,1225,1247,1263,1284,1290,1314,1319,1343,1390,1423,1445` | absolute machine path |
| `ea17ee65c068` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrossbeam_epoch-b42d69dd984bb638.rmeta:1134,1140,1175,1201,1203,1209,1231,1247,1268,1274,1298,1303` | absolute machine path |
| `89bcec043481` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrossbeam_utils-411613110992b82d.rlib:4575,4578,4586,4590,4600,4611,4640,4641,4646,4657,4673,4678,4681,4703,4733` | absolute machine path |
| `53b26d6499e9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrossbeam_utils-411613110992b82d.rmeta:4555,4558,4566,4570,4580,4591,4620,4621,4626,4637,4653,4658,4661` | absolute machine path |
| `ec5ca111b635` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrypto_common-962e10aa5c60551f.rlib:130` | absolute machine path |
| `faf3be7032f2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libcrypto_common-962e10aa5c60551f.rmeta:125` | absolute machine path |
| `79a657134610` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libctor-5ab4a2b02c1822b9.rlib:2366,2368,2382,2412,2420,2421,2422,2427,2428,2429` | absolute machine path |
| `60701b1a7cad` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libctor-5ab4a2b02c1822b9.rmeta:2360,2362,2376,2406,2414,2415,2416,2421,2422,2423` | absolute machine path |
| `f39b697c269d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libdata_url-39162354f93fb2aa.rlib:363,370,374,412,466` | absolute machine path |
| `3af8830f0251` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libdata_url-39162354f93fb2aa.rmeta:350,357,361` | absolute machine path |
| `e4629320445b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libderive_more-393cf85328c0a8ea.rlib:104,106,109,110` | absolute machine path |
| `d1d764cbbd54` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libderive_more-393cf85328c0a8ea.rmeta:96,98,101,102` | absolute machine path |
| `33985bcb70fa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libdigest-f995b97431ce9e4b.rlib:615,617,620,622,624,625` | absolute machine path |
| `a115103516a4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libdigest-f995b97431ce9e4b.rmeta:609,611,614,616,618,619` | absolute machine path |
| `286b482d33b5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libdyn_clone-a718b5527d5f42b6.rlib:283,286` | absolute machine path |
| `49a82d8f1e67` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libdyn_clone-a718b5527d5f42b6.rmeta:275,278` | absolute machine path |
| `623e9ea52980` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libeither-34e716935586d135.rlib:790,883,910` | absolute machine path |
| `dad8cd34a7c5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libeither-34e716935586d135.rmeta:785,878,905` | absolute machine path |
| `250ac0195ebc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libequivalent-577ff8398ac685d5.rlib:19` | absolute machine path |
| `573a3d965339` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libequivalent-577ff8398ac685d5.rmeta:14` | absolute machine path |
| `d5c3aa72b754` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libetagere-01684ac51f9da86b.rlib:279,303,387,476` | absolute machine path |
| `0838a0a2c596` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libetagere-01684ac51f9da86b.rmeta:266,290` | absolute machine path |
| `5e454db5aa65` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libeuclid-e1a2cf8cb84d4232.rlib:7419,7460,7464,7488,7512,7518,7523,7620,7633,7649,7706,7721,7750,7834,7924,7984,8031,8135,8136,8139,8140,8143` | absolute machine path |
| `9232fcb661ff` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libeuclid-e1a2cf8cb84d4232.rmeta:7414,7455,7459,7483,7507,7513,7518,7615,7628,7644,7701,7716,7745,7829,7919,7979,8026,8130,8131,8134,8135,8138` | absolute machine path |
| `afa38a13985a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libevent_listener-ae2197d196c1faf3.rlib:657,722,889,932,956,973` | absolute machine path |
| `560a125bf485` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libevent_listener-ae2197d196c1faf3.rmeta:650,715,882,925,949` | absolute machine path |
| `1b633fead5d0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libevent_listener_strategy-5b11c6b155384e91.rlib:94` | absolute machine path |
| `43ccc1f1c284` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libevent_listener_strategy-5b11c6b155384e91.rmeta:88` | absolute machine path |
| `42193938b581` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libexr-052ac3b52b7b79b0.rlib:18772,18797,18824,18836,18851,18858,18869,18882,18902,18911,18912,18915,18946,18976,18991,19017,19035,19049,19063,19283,19487,19506,19525,19528,19543,19551,19556,19565,19581,19754,19763,19773,19783,19793,19802,19806,19816,19827,19843,19858,19876,19899,19907,19912,19933,19941,19950,20027,20212,20314,20376,20424,20526,20527,20528,20620,20700,20792,20807,20808,20918,21106,21522,21617,21618,21706,21854,21856` | absolute machine path |
| `1fa39921db8d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libexr-052ac3b52b7b79b0.rmeta:18752,18777,18804,18816,18831,18838,18849,18862,18882,18891,18892,18895,18926,18956,18971,18997,19015,19029,19043,19263,19467,19486,19505,19508,19523,19531,19536,19545,19561,19734,19743,19753,19763,19773,19782,19786,19796,19807,19823,19838,19856,19879,19887,19892,19913,19921,19930` | absolute machine path |
| `9aa487374c16` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfax-b0d42ec9ced48de2.rlib:790,813,815,824,826,847` | absolute machine path |
| `8a5e685e05ee` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfax-b0d42ec9ced48de2.rmeta:777,800,802,811,813` | absolute machine path |
| `c3a24f11a295` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfdeflate-637e035e9a274e55.rlib:603,613,649,652,1079,1135,1188,1218` | absolute machine path |
| `ddc10be9fdaa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfdeflate-637e035e9a274e55.rmeta:588,598,634,637` | absolute machine path |
| `8ecafc27a334` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libflate2-b096a72acf57387a.rlib:2805,2809,2823,2825,2828,2838,2845,2856,2871,2892,2915,2919,2924,2935,2938,2946,2948,2957,3045,3151` | absolute machine path |
| `7a6db2083e02` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libflate2-b096a72acf57387a.rmeta:2786,2790,2804,2806,2809,2819,2826,2837,2852,2873,2896,2900,2905,2916,2919,2927,2929,2938` | absolute machine path |
| `9172c5e43b24` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfloat_cmp-89f1b69b9036bbf4.rlib:270,274,289,295` | absolute machine path |
| `b897ed5afc05` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfloat_cmp-89f1b69b9036bbf4.rmeta:265,269,284,290` | absolute machine path |
| `d4cf30492af0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfloat_next_after-02fd77e18483a5c6.rlib:35` | absolute machine path |
| `d2b8803928c9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfloat_next_after-02fd77e18483a5c6.rmeta:29` | absolute machine path |
| `3cf43fcb2ce6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libflume-dab6467cfefd6408.rlib:1181,1220,1231` | absolute machine path |
| `77a5a9f7af60` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libflume-dab6467cfefd6408.rmeta:1173,1212,1223` | absolute machine path |
| `ba34624b2575` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfoldhash-cbdb1f643612d5fc.rlib:296,306,310,320` | absolute machine path |
| `4ad4ee852e20` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfoldhash-cbdb1f643612d5fc.rmeta:289,299,303,313` | absolute machine path |
| `06386bae9a41` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfont_types-4e8facf3dfb9604d.rlib:3869,3870,3873,3924,3927,3931,3940,3943,3954,3955,3957,3958,3959,3988` | absolute machine path |
| `23e69bda9dac` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfont_types-4e8facf3dfb9604d.rmeta:3855,3856,3859,3910,3913,3917,3926,3929,3940,3941,3943,3944,3945` | absolute machine path |
| `95f5d9e6cdcf` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfont_types-eb67a18a8f4e7a4d.rlib:3721,3722,3739,3742,3746,3755,3758,3769,3770,3771,3772,3810` | absolute machine path |
| `422edee9e43a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfont_types-eb67a18a8f4e7a4d.rmeta:3697,3698,3715,3718,3722,3731,3734,3745,3746,3747,3748` | absolute machine path |
| `2afec55f2f8e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfontdb-8eb86dd40f4a099e.rlib:700,875,910,928,957,982` | absolute machine path |
| `c7eec9f9315b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfontdb-8eb86dd40f4a099e.rmeta:664` | absolute machine path |
| `384c36e6a9ec` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libform_urlencoded-1d998a2b8e72bd5c.rlib:119` | absolute machine path |
| `7f31de7cc045` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libform_urlencoded-1d998a2b8e72bd5c.rmeta:112` | absolute machine path |
| `aea75c2b6c91` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures-e6c4e9e219cc8f8e.rlib:63` | absolute machine path |
| `85e7a54dfc9f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures-e6c4e9e219cc8f8e.rmeta:58` | absolute machine path |
| `be787c80b1ff` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_channel-33b7d20424f85d38.rlib:1230,1231,1235,1236,1288,1302` | absolute machine path |
| `5e256cfadcf6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_channel-33b7d20424f85d38.rmeta:1224,1225,1229,1230,1282,1296` | absolute machine path |
| `86ef23982520` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_core-99ecb25813b5e77b.rlib:139,148,150,155,156,157,159,163` | absolute machine path |
| `5635d1c61063` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_core-99ecb25813b5e77b.rmeta:134,143,145,150,151,152,154` | absolute machine path |
| `fbb870e4b791` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_executor-1000b14f4ed90363.rlib:283,284,292,313,327,357` | absolute machine path |
| `7dd709fcbf4a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_executor-1000b14f4ed90363.rmeta:263,264,272` | absolute machine path |
| `2007bf779bf0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_io-7399d0d83c0e453a.rlib:228` | absolute machine path |
| `ce9f3bff654e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_io-7399d0d83c0e453a.rmeta:220` | absolute machine path |
| `975b30ae6a3b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_sink-accb1e2730709709.rlib:91` | absolute machine path |
| `21b28baaee04` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_sink-accb1e2730709709.rmeta:85` | absolute machine path |
| `9aa183480964` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_task-201cca4148e7b727.rlib:404,407,419,420,421,444` | absolute machine path |
| `6a17cc19c13f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_task-201cca4148e7b727.rmeta:397,400,412,413,414,437` | absolute machine path |
| `41d12361ff33` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_util-4fabbc897642c0e3.rlib:19285,19300,19317,19325,19361,19376,19384,19405,19410,19440,19473,19474,19479,19502,19503,19506,19508,19513,19515,19517,19538,19540,19553,19556,19558,19560,19565,19566,19569,19570,19577,19583,19585,19589,19594,19597,19598,19608,19612,19613,19672,19677,19679,19692,19698,19700,19701,19702,19729,19730,19733,19735,19736,19737,19739,19743,19744,19745,19748,19750,19751,19752,19761,19762,19764,19766,19768,19772,19775,19778,19779,19782,19787,19791,19797,19804,19806,19808,19810,19811,19812,19817,19819,19821,19822,19827,19828,19831,19835,19840,19843,19849,19850,19853,19855,19858,19861,19862,19873,19878,19880,19886,19889,19891,19894,19896,19898,19899,19902,19903,19905,19907,19911,19912,19918,19926,19931,19935,19936,19940,19944,19945,19946,19950,19951,19952,19954,19957,19958,19960,19961,19963,19964,19968,19969,19980,20097` | absolute machine path |
| `5de7a908f0a5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libfutures_util-4fabbc897642c0e3.rmeta:19265,19280,19297,19305,19341,19356,19364,19385,19390,19420,19453,19454,19459,19482,19483,19486,19488,19493,19495,19497,19518,19520,19533,19536,19538,19540,19545,19546,19549,19550,19557,19563,19565,19569,19574,19577,19578,19588,19592,19593,19652,19657,19659,19672,19678,19680,19681,19682,19709,19710,19713,19715,19716,19717,19719,19723,19724,19725,19728,19730,19731,19732,19741,19742,19744,19746,19748,19752,19755,19758,19759,19762,19767,19771,19777,19784,19786,19788,19790,19791,19792,19797,19799,19801,19802,19807,19808,19811,19815,19820,19823,19829,19830,19833,19835,19838,19841,19842,19853,19858,19860,19866,19869,19871,19874,19876,19878,19879,19882,19883,19885,19887,19891,19892,19898,19906,19911,19915,19916,19920,19924,19925,19926,19930,19931,19932,19934,19937,19938,19940,19941,19943,19944,19948,19949` | absolute machine path |
| `aff149d33074` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgeneric_array-05e943d6b826e76e.rlib:7189,7217,7233,7240,7249,7256,7274,7294` | absolute machine path |
| `a198fd855bbb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgeneric_array-05e943d6b826e76e.rmeta:7183,7211,7227,7234,7243,7250,7268` | absolute machine path |
| `b3c3e8503cde` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgetrandom-d9eac8cfb620c88e.rlib:588,591,599,602,607,609,610` | absolute machine path |
| `729b681825f2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgetrandom-d9eac8cfb620c88e.rmeta:577,580,588,591,596,598,599` | absolute machine path |
| `fa2f894959f6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgif-1b0f8ed19d571860.rlib:1399,1405,1420,1441,1458,1476,1913,1932,1948` | absolute machine path |
| `c822d4414320` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgif-1b0f8ed19d571860.rmeta:1379,1385,1400,1421,1438,1456` | absolute machine path |
| `f119912f175e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgimli-36a4f7d64e10a6e5.rlib:19817` | email |
| `f119912f175e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgimli-36a4f7d64e10a6e5.rlib:21610,21613,21622,21630,21634,21663,21676,21687,21835,21876,21885,21895,21931,21947,21961,22048,22066,22076,22124,22141,22146,22306,22348,22355,22362,22363,22364,22370,22372,22769` | absolute machine path |
| `401d3b3fec13` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgimli-36a4f7d64e10a6e5.rmeta:19797` | email |
| `401d3b3fec13` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgimli-36a4f7d64e10a6e5.rmeta:21590,21593,21602,21610,21614,21643,21656,21667,21815,21856,21865,21875,21911,21927,21941,22028,22046,22056,22104,22121,22126,22286,22328,22335,22342,22343,22344,22350,22352` | absolute machine path |
| `c0d9fc4d4384` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libglam-55fa7a0b48eca2af.rlib:52854` | email |
| `c0d9fc4d4384` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libglam-55fa7a0b48eca2af.rlib:68696,68698,68700,68705,68710,68779,68783,68785,68794,68800,68810,68819,68830,68840,68861,68875,68925,68934,68943,68993,69006,69025,69088,69124,69188,69226,69250,69271,69334,69370,69394,69445,69461,69532,69596,69667,69731,69803,69867,69938,70002,70064,70142,70213,70291,70362,70440,70511,70589,70661,70730,70802,70853,70874,70960,71039,71129,71210,71301,71383,71476,71559,71638,71647,71663,71672,71688,71697,71713,71722,71738,71747,71763,71764,71773,71789,71798,71814,71823,71839,71849,71865,71874,71890,71893,71896,71899,71902,71905,71908,71912,71915,71918,71923,71926,71927,71928` | absolute machine path |
| `656410dd7e10` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libglam-55fa7a0b48eca2af.rmeta:52834` | email |
| `656410dd7e10` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libglam-55fa7a0b48eca2af.rmeta:68676,68678,68680,68685,68690,68759,68763,68765,68774,68780,68790,68799,68810,68820,68841,68855,68905,68914,68923,68973,68986,69005,69068,69104,69168,69206,69230,69251,69314,69350,69374,69425,69441,69512,69576,69647,69711,69783,69847,69918,69982,70044,70122,70193,70271,70342,70420,70491,70569,70641,70710,70782,70833,70854,70940,71019,71109,71190,71281,71363,71456,71539,71618,71627,71643,71652,71668,71677,71693,71702,71718,71727,71743,71744,71753,71769,71778,71794,71803,71819,71829,71845,71854,71870,71873,71876,71879,71882,71885,71888,71892,71895,71898,71903,71906,71907,71908` | absolute machine path |
| `3ade33e3ed96` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libglow-b98221d0a62b757e.rlib:2446,2455,2592,3715,3985,4066,4073,4092` | absolute machine path |
| `5dfdc1597d30` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libglow-b98221d0a62b757e.rmeta:2426,2435,2572` | absolute machine path |
| `942a3224c229` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui-bc07bef071e9ae75.rlib:48676` | email |
| `942a3224c229` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui-bc07bef071e9ae75.rlib:51112,51206,51240,51256,51263,51264,51272,51295,51314,51332,51342,51356,51358,51490,51504,51582,51583,51602,51621,51639,51649,51762,51804,51816,51832,51856,51917,51949,51959,51977,52053,52105,52110,52133,52153,52205,52211,52220,52246,52274,52303,52344,52374,52398,52416,52668,52672,52730,52745,52773,52802,52814,52819,52821,52868,52874,52882,52886,52890,52900,52903,52922,52927,52928,52930,52932,55050,55051,55064,55068,55069,57236,57237,57240,57241,58198,58201,58202,58206,58207,58214,60166,60167,60170,60172,60175,60207,61569,61578,61579,61583,63129,63131,63136,63908,63909,63913,63916,63921,64049,64052,67300,67304,67308,67309,67319,67323,68901,68903,68904,68907,68908,68909,68914,70514,70551,70554,70555,70557,72003,73100,73101,73102,73108,73312,74193,74194,74204,74205,75427,75428,75436,75438,75443,75462,75463,75465,76854,76862,76863,76864,76865` | absolute machine path |
| `734f31cad764` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui-bc07bef071e9ae75.rmeta:48656` | email |
| `734f31cad764` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui-bc07bef071e9ae75.rmeta:51092,51186,51220,51236,51243,51244,51252,51275,51294,51312,51322,51336,51338,51470,51484,51562,51563,51582,51601,51619,51629,51742,51784,51796,51812,51836,51897,51929,51939,51957,52033,52085,52090,52113,52133,52185,52191,52200,52226,52254,52283,52324,52354,52378,52396,52648,52652,52710,52725,52753,52782,52794,52799,52801,52848,52854,52862,52866,52870,52880,52883,52902,52907,52908,52910,52912` | absolute machine path |
| `0032fe815ab5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui_shared_string_gpui_unofficial-f9608ed24d1423c6.rlib:310` | absolute machine path |
| `7ace5de7f8da` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui_shared_string_gpui_unofficial-f9608ed24d1423c6.rmeta:304` | absolute machine path |
| `266cf84dc44d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui_util_gpui_unofficial-4a4b8c98b51431ae.rlib:524,553` | absolute machine path |
| `2cf958774b7a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui_util_gpui_unofficial-4a4b8c98b51431ae.rmeta:510,539` | absolute machine path |
| `41654ec369a7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui_web_gpui_unofficial-9b29a8b146cffba0.rlib:3876,3890,3912,3934,3937,3950,3958,3964,3971,3973,3991,4015,4035,4036,4043,4175,4294,4301,4302,4417,4424,4425,4648,4727,4792,4796,4880,5151,5154,5461,5591,5677,5716,5778,5817,5882,5883,5965` | absolute machine path |
| `acf4e232d5fb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui_web_gpui_unofficial-9b29a8b146cffba0.rmeta:3856,3870,3892,3914,3917,3930,3938,3944,3951,3953,3971,3995,4015,4016,4023` | absolute machine path |
| `09c6779608d6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui_wgpu_gpui_unofficial-41354d151d8a36c6.rlib:6089,6131,6141,6162,6223,6399,6735,11382,11717,11756,11803,11805,11887` | absolute machine path |
| `9d9dac2dca1f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libgpui_wgpu_gpui_unofficial-41354d151d8a36c6.rmeta:6069,6111,6121,6142,6203` | absolute machine path |
| `b4a1cd259c8e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhalf-f4fcb665e236790b.rlib:3779,3936,4091,4170,4180,4196,4198,4201,4211,4378` | absolute machine path |
| `8f3c189c8df1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhalf-f4fcb665e236790b.rmeta:3767,3924,4079,4158,4168,4184,4186,4189,4199` | absolute machine path |
| `4ba6202f4e7a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libharfrust-9a57bd105c69d522.rlib:10355,10365,10381,10396,10400,10403,10413,10431,10432,10439,10546,10549,10589,10612,10643,10652,10656,10659,10665,10668,10681,10689,10692,10715,10722,10747,10781,10799,10809,10872,10885,10901,10905,10927,10929,10932,10979,10982,10986,10992,10997,11009,11012,11022,11037,11042,11043,11052,11057,11067,11072,11074,11077,11078,11082,11083,11122,11137,11139,11155,11170,11182,11187,11189,11191,11486,11491,11639,12119,12318,12321,12327,12340,12583,12672,12673,12674,12839,12938,12946,12957,13049,13333,13444,14010,14011,14014,14019,14645,14647,14877,14880,15021,15131,15595` | absolute machine path |
| `f04d2f986cde` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libharfrust-9a57bd105c69d522.rmeta:10335,10345,10361,10376,10380,10383,10393,10411,10412,10419,10526,10529,10569,10592,10623,10632,10636,10639,10645,10648,10661,10669,10672,10695,10702,10727,10761,10779,10789,10852,10865,10881,10885,10907,10909,10912,10959,10962,10966,10972,10977,10989,10992,11002,11017,11022,11023,11032,11037,11047,11052,11054,11057,11058,11062,11063,11102,11117,11119,11135,11150,11162,11167,11169,11171` | absolute machine path |
| `52246c6f2e97` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhash32-6060ecbc024a4b25.rlib:202,203,209` | absolute machine path |
| `a4a9957d349d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhash32-6060ecbc024a4b25.rmeta:197,198,204` | absolute machine path |
| `765ed0add1e5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhashbrown-cf0ce1937955ee9a.rlib:10639,10640,10642,10647,10782,11065,11186,11290,11294,11299,11303,11304,11306,11310` | absolute machine path |
| `8eb52c1444ce` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhashbrown-cf0ce1937955ee9a.rmeta:10631,10632,10634,10639,10774,11057,11178,11282,11286,11291,11295,11296,11298,11302` | absolute machine path |
| `1fd354831760` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libheapless-83aa0dc6382dffee.rlib:7650,7812,9034,9040,9051,9131,9177,9261,9289,9298,9349,9379,9380,9486,9490,9519,9530,9535,9537,9546,9561,9567,9597,9632,9646` | absolute machine path |
| `2b3cfbc0e2fa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libheapless-83aa0dc6382dffee.rmeta:7640,7802,9024,9030,9041,9121,9167,9251,9279,9288,9339,9369,9370,9476,9480,9509,9520,9525,9527,9536,9551,9557,9587,9622,9636` | absolute machine path |
| `f46c66d0ca37` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhttp-e8757ef0e4590cc3.rlib:4565` | email |
| `f46c66d0ca37` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhttp-e8757ef0e4590cc3.rlib:6740,6742,6767,6781,6793,6817,6831,6839,6848,6860,6873,6911,6921,7081,7086,7087,7094,7095,7099,7212,7215,7244,7295,7317,7343,7363,7428,7455,7497` | absolute machine path |
| `6be72df32a85` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhttp-e8757ef0e4590cc3.rmeta:4545` | email |
| `6be72df32a85` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhttp-e8757ef0e4590cc3.rmeta:6720,6722,6747,6761,6773,6797,6811,6819,6828,6840,6853,6891,6901,7061,7066,7067,7074,7075,7079` | absolute machine path |
| `c6a74c01cc02` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhttp_body-86b0eedde9c8d90b.rlib:164,167,177` | absolute machine path |
| `e98718906818` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhttp_body-86b0eedde9c8d90b.rmeta:159,162,172` | absolute machine path |
| `a7b734f5f971` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhttp_client_gpui_unofficial-90d27b8161b7be77.rlib:367,379,400,448,541,573` | absolute machine path |
| `5a1321490ca8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libhttp_client_gpui_unofficial-90d27b8161b7be77.rmeta:347,359` | absolute machine path |
| `9dd98c2918b8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_collections-e14c955852b3fd6a.rlib:1659,1660,1678,1708,1710,1721,1757,1762,1789,1813,1836,1877,1905,1913` | absolute machine path |
| `a61857e550ed` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_collections-e14c955852b3fd6a.rmeta:1639,1640,1658,1688,1690,1701,1737,1742` | absolute machine path |
| `bf2e4dd96952` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_locale_core-db2e9959c4a56369.rlib:9081,9083,9108,9129,9132,9138,9139,9140,9146,9157,9166,9175,9181,9188,9205,9208,9219,9220,9229,9239,9253,9256,9260,9262,9264,9273,9281,9294,9309,9327,9328,9332,9334,9337,9338,9339,9340,9363,9525,9558` | absolute machine path |
| `ebcb998b840d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_locale_core-db2e9959c4a56369.rmeta:9061,9063,9088,9109,9112,9118,9119,9120,9126,9137,9146,9155,9161,9168,9185,9188,9199,9200,9209,9219,9233,9236,9240,9242,9244,9253,9261,9274,9289,9307,9308,9312,9314,9317,9318,9319,9320,9343` | absolute machine path |
| `86a4a538d1b2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_normalizer-56e32d6a47915628.rlib:2641,2733,2761,2908,2934,3083` | absolute machine path |
| `1d205e768777` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_normalizer-56e32d6a47915628.rmeta:2621,2713,2741` | absolute machine path |
| `f2db23c59352` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_normalizer_data-7a52c40c3b23286e.rlib:124,126,132,138,144,150,156,162` | absolute machine path |
| `a14274b6d074` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_normalizer_data-7a52c40c3b23286e.rmeta:119,121,127,133,139,145,151,157` | absolute machine path |
| `5eba34061577` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_properties-2acc8a9a49a83fb9.rlib:17328,17330,17335,17345,17347,17368,17377,17381,17386,17390,17409,17410,23061,23079` | absolute machine path |
| `45b53dbc8c47` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_properties-2acc8a9a49a83fb9.rmeta:17306,17308,17313,17323,17325,17346,17355,17359,17364,17368,17387,17388` | absolute machine path |
| `a88dcb10ef2d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_properties_data-68a40bd8768b91a3.rlib:2058,2060,2066,2072,2078,2084,2090,2097,2104,2111,2117,2124,2130,2136,2142,2148,2154,2160,2166,2172,2178,2185,2191,2197,2203,2209,2216,2223,2229,2235,2241,2247,2253,2259,2266,2272,2278,2284,2291,2297,2303,2309,2315,2321,2327,2333,2339,2345,2351,2357,2363,2369,2375,2381,2387,2393,2399,2405,2411,2417,2423,2429,2435,2441,2447,2453,2459,2465,2471,2477,2483,2489,2495,2501,2507,2513,2519,2525,2531,2537,2543,2550,2556,2563,2569,2575,2581,2587,2593,2599,2605,2611,2617,2623,2629,2635,2641,2647,2654,2660,2666,2672,2678,2684,2690,2696,2702,2708,2714,2720,2726,2733,2739,2745,2751,2757,2763,2769,2775,2781,2787,2793,2800,2806,2812,2818,2824,2830,2837,2843,2849,2855,2861,2868,2874,2880,2886,2892,2898,2905,2912,2919` | absolute machine path |
| `3fe46e5efefb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_properties_data-68a40bd8768b91a3.rmeta:2053,2055,2061,2067,2073,2079,2085,2092,2099,2106,2112,2119,2125,2131,2137,2143,2149,2155,2161,2167,2173,2180,2186,2192,2198,2204,2211,2218,2224,2230,2236,2242,2248,2254,2261,2267,2273,2279,2286,2292,2298,2304,2310,2316,2322,2328,2334,2340,2346,2352,2358,2364,2370,2376,2382,2388,2394,2400,2406,2412,2418,2424,2430,2436,2442,2448,2454,2460,2466,2472,2478,2484,2490,2496,2502,2508,2514,2520,2526,2532,2538,2545,2551,2558,2564,2570,2576,2582,2588,2594,2600,2606,2612,2618,2624,2630,2636,2642,2649,2655,2661,2667,2673,2679,2685,2691,2697,2703,2709,2715,2721,2728,2734,2740,2746,2752,2758,2764,2770,2776,2782,2788,2795,2801,2807,2813,2819,2825,2832,2838,2844,2850,2856,2863,2869,2875,2881,2887,2893,2900,2907,2914` | absolute machine path |
| `32d1293a0fa1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_provider-7ff6b176f883b7d3.rlib:2432,2433,2436,2439,2448,2466,2503,2515,2530,2542,2543,2548,2553,2591,2673,2682` | absolute machine path |
| `741cecd74a4c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libicu_provider-7ff6b176f883b7d3.rmeta:2412,2413,2416,2419,2428,2446,2483,2495,2510,2522,2523,2528,2533` | absolute machine path |
| `2f74b4a40f1e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libidna-5f30f4f7b5fb1259.rlib:725,738,744,786,1045,1893,2006,2082,2393,2408` | absolute machine path |
| `3d13336c3330` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libidna-5f30f4f7b5fb1259.rmeta:701,714,720,762` | absolute machine path |
| `5f6f6a482f84` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libidna_adapter-a87e0fc20e5ff473.rlib:146` | absolute machine path |
| `484c185ba60d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libidna_adapter-a87e0fc20e5ff473.rmeta:141` | absolute machine path |
| `f78b6d7eb0cf` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libimage-c7776a87e39cabf0.rlib:16326` | email |
| `f78b6d7eb0cf` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libimage-c7776a87e39cabf0.rlib:17852,17853,17872,17879,17889,17901,17923,17963,17968,18011,18024,18039,18050,18077,18092,18101,18105,18142,18157,18184,18194,18232,18238,18240,18265,18271,18273,18282,18308,18314,18382,18391,18468,18486,18555,18560,18565,18585,18600,18626,18632,18639,18647,18653,18671,18688,18700,18707,18709,18725,18744,18760,18764,18781,18787,18788,18817,18818,18819,18820,18821,20548,21331,21678,21682,22392,22393,22406,23612,23614,23619,23620,24205,24211,25032,25036,25038,25039,25317,25323,25884,25889,25890,25909,26551,26552,26553,26556,26558,26561,27495,28723,28724,28728,28731,29356,29358,29395,30008,30009,30011,30021,30328,30334,31008,31012,31015,31018` | absolute machine path |
| `d6a019b1d867` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libimage-c7776a87e39cabf0.rmeta:16306` | email |
| `d6a019b1d867` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libimage-c7776a87e39cabf0.rmeta:17832,17833,17852,17859,17869,17881,17903,17943,17948,17991,18004,18019,18030,18057,18072,18081,18085,18122,18137,18164,18174,18212,18218,18220,18245,18251,18253,18262,18288,18294,18362,18371,18448,18466,18535,18540,18545,18565,18580,18606,18612,18619,18627,18633,18651,18668,18680,18687,18689,18705,18724,18740,18744,18761,18767,18768,18797,18798,18799,18800,18801` | absolute machine path |
| `77f7ef8d6cd4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libimage_webp-0cb710f3bbfa2ea1.rlib:4190,4194,4250,4285,4297,4316,4330,4352,4366,4496,4507,4514,4619,4738,4818,4932,4964,5012,5040,5061,5130,5149` | absolute machine path |
| `ae58ab007e4d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libimage_webp-0cb710f3bbfa2ea1.rmeta:4170,4174,4230,4265,4277,4296,4310,4332,4346,4476,4487,4494` | absolute machine path |
| `4c88fa66c9cc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libimagesize-9aee81c9f6e6ec86.rlib:1789,1802,1816,1817,1819,1820,1827,1828,1830,1833,1835,1839,1840,1843,1854,1864,1868,1905,1922,1971` | absolute machine path |
| `00b02c38020f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libimagesize-9aee81c9f6e6ec86.rmeta:1769,1782,1796,1797,1799,1800,1807,1808,1810,1813,1815,1819,1820,1823,1834,1844,1848` | absolute machine path |
| `9094bf44bdcf` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libindexmap-e1fa959837c21661.rlib:7582,7594,7616,7621,7659,7724,7741,7796,7805,7845,7850,7866,7873,7881,7884,7909,7915,7938,7942,7967` | absolute machine path |
| `30bd4088973b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libindexmap-e1fa959837c21661.rmeta:7577,7589,7611,7616,7654,7719,7736,7791,7800,7840,7845,7861,7868,7876,7879,7904,7910,7933,7937,7962` | absolute machine path |
| `705609596eef` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libinventory-2e602b72739f50e9.rlib:82` | absolute machine path |
| `21de32b08350` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libinventory-2e602b72739f50e9.rmeta:77` | absolute machine path |
| `f5a87a7b258b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libitertools-6481b6772170f6a3.rlib:13215,13392,13402,13405,13419,13422,13425,13432,13439,13442,13448,13504,13545,13547,13562,13565,13570,13571,13575,13594,13600,13601,13607,13615,13616,13635,13654,13658,13665,13668,13670,13674,13679,13692,13694,13697,13703,13704,13709,13712,13717,13720,13728,13729,13730,13735,13738,13744,13771` | absolute machine path |
| `eae4ea4ca744` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libitertools-6481b6772170f6a3.rmeta:13204,13381,13391,13394,13408,13411,13414,13421,13428,13431,13437,13493,13534,13536,13551,13554,13559,13560,13564,13583,13589,13590,13596,13604,13605,13624,13643,13647,13654,13657,13659,13663,13668,13681,13683,13686,13692,13693,13698,13701,13706,13709,13717,13718,13719,13724,13727,13733` | absolute machine path |
| `70f438b3c83c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libitoa-b871a4945f476c59.rlib:337,338,365` | absolute machine path |
| `3f0461ee9fb3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libitoa-b871a4945f476c59.rmeta:332,333` | absolute machine path |
| `a06453a2ac61` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libjs_sys-5f0837a473ec879a.rlib:24376` | email |
| `a06453a2ac61` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libjs_sys-5f0837a473ec879a.rlib:28049,28191,28196,28200,28208,28338,29802,30143,30192,30201,30222` | absolute machine path |
| `8dd57068ab82` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libjs_sys-5f0837a473ec879a.rmeta:24356` | email |
| `8dd57068ab82` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libjs_sys-5f0837a473ec879a.rmeta:28029,28171,28176,28180,28188` | absolute machine path |
| `1e29337257c3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libkurbo-6602e7737e61791b.rlib:7620,7628,7635,7645,7689,7707,7731,7769,7787,7798,7821,7828,7832,7834,7851,7855,7859,7884,7887,7906,7910,7916,7927,7937,7960,7980,7987,7998,8010,8013,8019,8204,8324,8481,8560,8631,8699,8814,8921,8980,9092,9244,9309` | absolute machine path |
| `da48eb2c428a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libkurbo-6602e7737e61791b.rmeta:7595,7603,7610,7620,7664,7682,7706,7744,7762,7773,7796,7803,7807,7809,7826,7830,7834,7859,7862,7881,7885,7891,7902,7912,7935,7955,7962,7973,7985,7988,7994` | absolute machine path |
| `4b8716e6922e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblebe-b952d7044526dd74.rlib:999` | absolute machine path |
| `af6ca90bf5c9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblebe-b952d7044526dd74.rmeta:994` | absolute machine path |
| `16b16924ffbb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblibc-0a0ff83f482c6190.rlib:205,279,280,291` | absolute machine path |
| `d4d4a736437e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblibc-0a0ff83f482c6190.rmeta:200,274,275,286` | absolute machine path |
| `cdb7514f89d6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblibm-d3f925135440fa41.rlib:3797,3833,3840,3849,3853,3862,3894,3899,3908,3909,3914,3926,3933,3934,3938,3945,3946,3947,3949,3951,3953,3955,3959,3978,3984,3985,3986,3987,3989,3991,3996,3998,4006,4015,4021,4023,4040,4045,4049,4050,4051,4077,4079,4087,4090,4091,4095,4101,4103,4104,4107,4111,4115,4118,4120,4122,4124,4127,4129,4131,4134,4135,4137,4138,4139,4151,4154,4157,4164,4165,4166,4169,4172,4175,4178,4179,4181,4183,4184,4185,4189,4194,4198,4205,4206,4207,4216,4224,4234,4244,4246,4250,4254,4256,4262,4265,4267,4269,4272,4274,4282,4283,4285,4286,4291,4297,4298,4303,4308,4311,4560,4561,4607,4644,4811` | absolute machine path |
| `a433297bef86` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblibm-d3f925135440fa41.rmeta:3767,3803,3810,3819,3823,3832,3864,3869,3878,3879,3884,3896,3903,3904,3908,3915,3916,3917,3919,3921,3923,3925,3929,3948,3954,3955,3956,3957,3959,3961,3966,3968,3976,3985,3991,3993,4010,4015,4019,4020,4021,4047,4049,4057,4060,4061,4065,4071,4073,4074,4077,4081,4085,4088,4090,4092,4094,4097,4099,4101,4104,4105,4107,4108,4109,4121,4124,4127,4134,4135,4136,4139,4142,4145,4148,4149,4151,4153,4154,4155,4159,4164,4168,4175,4176,4177,4186,4194,4204,4214,4216,4220,4224,4226,4232,4235,4237,4239,4242,4244,4252,4253,4255,4256,4261,4267,4268,4273,4278,4281` | absolute machine path |
| `416bad58ad61` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblinebender_resource_handle-240e79f74b69368b.rlib:179,185,187` | absolute machine path |
| `41bead9d844b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblinebender_resource_handle-240e79f74b69368b.rmeta:174,180,182` | absolute machine path |
| `f8854b478808` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblink_section-f3ce6caa8c85c5cd.rlib:10309,10310,10315,10338,10347,10371,10377,10386,10392,10422,10423,10432,10433,10434,10467,10485` | absolute machine path |
| `93c7faffe537` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblink_section-f3ce6caa8c85c5cd.rmeta:10292,10293,10298,10321,10330,10354,10360,10369,10375,10405,10406,10415,10416,10417` | absolute machine path |
| `88e23bae3fde` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblitemap-7e29ee619ce51ab6.rlib:1317,1318,1387,1395` | absolute machine path |
| `4d7336b27996` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblitemap-7e29ee619ce51ab6.rmeta:1312,1313,1382,1390` | absolute machine path |
| `dbc0f56eb9a1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblock_api-4c759158cd368f4f.rlib:731,734,774,819` | absolute machine path |
| `c3ce9ba85627` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblock_api-4c759158cd368f4f.rmeta:725,728,768,813` | absolute machine path |
| `6374a5ce6141` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblog-576ffd46ce09ef54.rlib:860,899,909,939,951` | absolute machine path |
| `67103f231d13` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblog-576ffd46ce09ef54.rmeta:853,892,902,932` | absolute machine path |
| `190554168f86` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon-9547470d76e3c736.rlib:20` | absolute machine path |
| `efc8ff66563e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon-9547470d76e3c736.rmeta:15` | absolute machine path |
| `a9f06910b922` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon_algorithms-fdceae7e0f17945f.rlib:1547,1550,1591,1610,1612,1615,1633,1634,1636,1638,1647,1765,2124,2152,2183,2348,2373,2393` | absolute machine path |
| `0fe9fea0fb84` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon_algorithms-fdceae7e0f17945f.rmeta:1527,1530,1571,1590,1592,1595,1613,1614,1616,1618,1627` | absolute machine path |
| `e0c0298dbe43` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon_geom-9199d3f9321b7b6d.rlib:10953,10972,11052,11075,11154,11168,11216,11225,11301` | absolute machine path |
| `37239c898a0a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon_geom-9199d3f9321b7b6d.rmeta:10948,10967,11047,11070,11149,11163,11211,11220,11296` | absolute machine path |
| `8359e0abf447` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon_path-06c73d2e08d2a3c6.rlib:2992,2998,3050,3081,3123,3127,3136,3147,3157,3236,3280,3306,3346` | absolute machine path |
| `265aa3fa2224` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon_path-06c73d2e08d2a3c6.rmeta:2976,2982,3034,3065,3107,3111,3120,3131,3141` | absolute machine path |
| `beb5648de3bd` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon_tessellation-a8a0fe4992dbf4e2.rlib:1885,1888,1892,1984,1996,1997,2077,2081,2109,2506,2774,2908,2951,3034,3175,3293,3361,3396,3430` | absolute machine path |
| `aa553631e487` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liblyon_tessellation-a8a0fe4992dbf4e2.rmeta:1828,1831,1835,1927,1939,1940,2020,2024,2052` | absolute machine path |
| `8119e3c56291` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libm-d3f925135440fa41.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145` | absolute machine path |
| `2dbc53469810` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libmemchr-28a1ea40f6f490d0.rlib:3346,3389,3398,3411,3415,3438,3442,3452,3498,3523,3535,3578,3614,3620,3622,3713,3756,3769` | absolute machine path |
| `8c0ad8e45f13` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libmemchr-28a1ea40f6f490d0.rmeta:3336,3379,3388,3401,3405,3428,3432,3442,3488,3513,3525,3568,3604,3610,3612` | absolute machine path |
| `ab592716bb47` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libmemmap2-c402c63e463fdfda.rlib:352,374` | absolute machine path |
| `6a8fcbbc0f77` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libmemmap2-c402c63e463fdfda.rmeta:342,364` | absolute machine path |
| `b318d56aa423` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libminiz_oxide-960495ed1d6a3ad6.rlib:1317,1384,1386,1395,1399,1409,1457,1466,1471,1474,1754,1874,1875,1911,1936,1945,1977` | absolute machine path |
| `8f2d41a4ad4d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libminiz_oxide-960495ed1d6a3ad6.rmeta:1291,1358,1360,1369,1373,1383,1431,1440,1445,1448` | absolute machine path |
| `c6c1601a52ad` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libminiz_oxide-f59bee9a216466c3.rlib:1578,1652,1654,1662,1668,1678,1728,1737,1742,1745,1747,2063,2176,2177,2218,2227,2278,2292` | absolute machine path |
| `d76b45b41959` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libminiz_oxide-f59bee9a216466c3.rmeta:1558,1632,1634,1642,1648,1658,1708,1717,1722,1725,1727` | absolute machine path |
| `4b86dfd23b66` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libmoxcms-05adb9b5194d6a86.rlib:10776,10781,10783,10797,10808,10818,10831,10839,10874,10884,10889,10906,10943,10954,10964,10969,11012,11016,11022,11026,11029,11044,11052,11056,11058,11106,11153,11228,11315,11316,11320,11322,11324,11326,11328,11332,11391,11396,11420,11444,11449,11454,11463,11469,11477,11479,11488,11490,11493,11495,11533,11597,11600,11607,11623,11627,11638,11649,11660,12059,12766,13558,13674,13894,13962,14014,14061,14170,14283,14349,14584` | absolute machine path |
| `117a5e764751` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libmoxcms-05adb9b5194d6a86.rmeta:10756,10761,10763,10777,10788,10798,10811,10819,10854,10864,10869,10886,10923,10934,10944,10949,10992,10996,11002,11006,11009,11024,11032,11036,11038,11086,11133,11208,11295,11296,11300,11302,11304,11306,11308,11312,11371,11376,11400,11424,11429,11434,11443,11449,11457,11459,11468,11470,11473,11475,11513,11577,11580,11587,11603,11607,11618,11629,11640` | absolute machine path |
| `9407196788e5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnaga-6a55535d1711f3da.rlib:42857,42862,42871,42881,42891,42938,42943,42947,42957,42961,42964,42965,42989,42992,43017,43109,43121,43124,43135,43140,43142,43146,43149,43157,43166,43170,43203,43210,43246,43265,43276,43296,43299,43375,43416,43434,43438,43441,43500,43507,43514,43517,43525,43549,43559,43564,43570,43573,43576,43591,43663,43678,43696,43737,43742,43747,43749,43753,43763,43770,43772,43779,43782,43786,43792,43798,43803,43826,43845,43875,43882,43887,43889,43894,43900,43901,43902,43908,43914,43917,43923,44718,44723,44733,45998,46004,46694,46695,46704,46705,47303,47840,47877,47993,47994,48534,48538,48540,48784,48786,48791,48792,49222,49704,49705,49706,49712,49714,50351,50353,50605,50626,50843,50848,51231,51233,51234,51593,51620,51741,51745` | absolute machine path |
| `4a7fe8ca2f0d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnaga-6a55535d1711f3da.rmeta:42837,42842,42851,42861,42871,42918,42923,42927,42937,42941,42944,42945,42969,42972,42997,43089,43101,43104,43115,43120,43122,43126,43129,43137,43146,43150,43183,43190,43226,43245,43256,43276,43279,43355,43396,43414,43418,43421,43480,43487,43494,43497,43505,43529,43539,43544,43550,43553,43556,43571,43643,43658,43676,43717,43722,43727,43729,43733,43743,43750,43752,43759,43762,43766,43772,43778,43783,43806,43825,43855,43862,43867,43869,43874,43880,43881,43882,43888,43894,43897,43903` | absolute machine path |
| `d5ce4a9f0ad1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnaga_types-4d301bbf9d4157b8.rlib:671,682,692,705` | absolute machine path |
| `8c0a5bbdf5bc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnaga_types-4d301bbf9d4157b8.rmeta:665,676,686` | absolute machine path |
| `bb37c052c979` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnum_complex-f3b4f46b0c16b153.rlib:4084,4198,4200,4203` | absolute machine path |
| `158e01547549` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnum_complex-f3b4f46b0c16b153.rmeta:4076,4190,4192,4195` | absolute machine path |
| `8faac0f46a98` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnum_cpus-d8364c06903d6185.rlib:20` | absolute machine path |
| `9c4e5de3b0df` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnum_cpus-d8364c06903d6185.rmeta:15` | absolute machine path |
| `ceba47b078f0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnum_traits-bc9fbf49325f1a55.rlib:9199,9267,9275,9282,9294,9298,9307,9310,9328,9330,9335,9338,9339,9341,9345,9353,9356,9407` | absolute machine path |
| `e3e1cd47f26e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libnum_traits-bc9fbf49325f1a55.rmeta:9192,9260,9268,9275,9287,9291,9300,9303,9321,9323,9328,9331,9332,9334,9338,9346,9349` | absolute machine path |
| `01c1f684a452` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libobject-3fd5ca6ac4cb9920.rlib:26330,26390,26400,26425,26433,26444,26487,26517,26521,26534,26539,26542,26556,26560,26564,26588,26590,26612,26624,26646,26667,26697,26735,26744,26752,26778,26796,26807,26819,26824,26827,26831,26842,26883,26886,26893,26898,26905,26926,26933,26954,26956,26965,26974,27002,27010,27012,27013,27014,27022,27031,27035,27042,27043` | absolute machine path |
| `c4a4d8585fd2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libobject-3fd5ca6ac4cb9920.rmeta:26310,26370,26380,26405,26413,26424,26467,26497,26501,26514,26519,26522,26536,26540,26544,26568,26570,26592,26604,26626,26647,26677,26715,26724,26732,26758,26776,26787,26799,26804,26807,26811,26822,26863,26866,26873,26878,26885,26906,26913,26934,26936,26945,26954,26982,26990,26992,26993,26994,27002,27011,27015,27022,27023` | absolute machine path |
| `9679722053a0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libonce_cell-673dc8c2b6897805.rlib:887,953,966,1002` | absolute machine path |
| `33a4b20ee178` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libonce_cell-673dc8c2b6897805.rmeta:882,948,961` | absolute machine path |
| `721f3a1a80b9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libparking-9479e98cacfb6f2d.rlib:95,109` | absolute machine path |
| `7ab3b9cf71a2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libparking-9479e98cacfb6f2d.rmeta:88` | absolute machine path |
| `7012646c870c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libparking_lot-bb14e5daef146f73.rlib:627,657,666,675,689,693,731,733,734,737,740,755,913,937,952,973,1011` | absolute machine path |
| `39b44b27ddc7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libparking_lot-bb14e5daef146f73.rmeta:611,641,650,659,673,677,715,717,718,721,724,739` | absolute machine path |
| `392896cd8648` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libparking_lot_core-17753d02173ceedb.rlib:727,790,801,803,808,809,819,847` | absolute machine path |
| `e10165150ac3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libparking_lot_core-17753d02173ceedb.rmeta:718,781,792,794,799,800` | absolute machine path |
| `d7fcd14793d0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpercent_encoding-0b7cab864b941d26.rlib:225,226,251` | absolute machine path |
| `60c07ddb77af` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpercent_encoding-0b7cab864b941d26.rmeta:217,218` | absolute machine path |
| `d624305ffa7a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpico_args-6491726215c01f7e.rlib:373,411` | absolute machine path |
| `0802e11b1952` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpico_args-6491726215c01f7e.rmeta:363` | absolute machine path |
| `573ba00f33f2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpin_project-8823898d1a81d98b.rlib:139` | absolute machine path |
| `04f9d24654d1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpin_project-8823898d1a81d98b.rmeta:134` | absolute machine path |
| `bbd96836a42e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpin_project_lite-6ac52b036886392f.rlib:178` | absolute machine path |
| `669409830b7d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpin_project_lite-6ac52b036886392f.rmeta:173` | absolute machine path |
| `a75acf4aa7e8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpng-361490b173c5e408.rlib:11075,11108,11112,11186,11193,11286,11302,11322,11323,11324,11346,11353,11372,11382,11395,11414,11417,11505,11572,11624,11648,11675,11725,11787,11826,11842,11873,11874,11926` | absolute machine path |
| `51ea0f1b616f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpng-361490b173c5e408.rmeta:11040,11073,11077,11151,11158,11251,11267,11287,11288,11289,11311,11318,11337,11347,11360,11379,11382` | absolute machine path |
| `0a2c2a277ecc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpng-ee9d781577d7ea37.rlib:10805,10815,10819,10877,10884,10967,10994,10995,10996,11023,11031,11051,11060,11061,11080,11082,11146,11270,11310,11340,11387,11388,11423,11442,11490,11508,11537` | absolute machine path |
| `778e2d180268` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpng-ee9d781577d7ea37.rmeta:10769,10779,10783,10841,10848,10931,10958,10959,10960,10987,10995,11015,11024,11025,11044,11046` | absolute machine path |
| `238be3575d87` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpolycool-c5e54fa897797165.rlib:405,412,413,422,432,435,559,604,616` | absolute machine path |
| `b0d087bdb3b1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpolycool-c5e54fa897797165.rmeta:389,396,397,406,416,419` | absolute machine path |
| `fee7b400eb49` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpotential_utf-b8b2102fc25c6921.rlib:423,425,433` | absolute machine path |
| `19f6edf00953` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpotential_utf-b8b2102fc25c6921.rmeta:413,415,423` | absolute machine path |
| `1f63b97e9796` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libppv_lite86-46352b6533f60de9.rlib:1633,1636,1652,1661` | absolute machine path |
| `2a9b4df893c6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libppv_lite86-46352b6533f60de9.rmeta:1628,1631,1647,1656` | absolute machine path |
| `e5a595af685a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libprofiling-060301a67beb4e1c.rlib:38,46` | absolute machine path |
| `5fc64a493ad3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libprofiling-060301a67beb4e1c.rmeta:33,41` | absolute machine path |
| `2e27c71b8722` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpulp-31b2423eea4ce9d1.rlib:19096,19447,19448` | absolute machine path |
| `5903f352ee81` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpulp-31b2423eea4ce9d1.rmeta:19089,19440,19441` | absolute machine path |
| `0c797dc43376` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpulp_wasm_simd_flag-d4f2f9af727346e1.rlib:36` | absolute machine path |
| `6b5f59275005` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpulp_wasm_simd_flag-d4f2f9af727346e1.rmeta:31` | absolute machine path |
| `2b3c4c5e4f45` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpxfm-1654d5d94a250952.rlib:8119,8122,8128,8135,8138,8147,8152,8160,8161,8162,8174,8183,8195,8206,8214,8221,8227,8233,8238,8244,8254,8269,8277,8289,8297,8304,8310,8313,8318,8323,8328,8331,8337,8343,8350,8363,8373,8386,8396,8404,8426,8447,8457,8469,8477,8486,8489,8491,8498,8502,8509,8564,8599,8604,8614,8619,8626,8628,8633,8638,8643,8647,8651,8657,8658,8667,8675,8680,8690,8695,8702,8708,8712,8717,8718,8725,8733,8737,8741,8747,8750,8753,8763,8773,8780,8786,8796,8807,8819,8827,8843,8848,8854,8859,8862,8865,8882,8893,8899,8904,8908,8910,8915,8919,8924,8927,8934,8938,8945,8948,8953,8959,8965,8969,8970,8974,8979,8986,8987,8997,9004,9012,9016,9020,9022,9030,9044,9050,9051,9054,9055,9062,9065,9068,9089,9098,9113,9125,9137,9139,9141,9146,9157,9158,9160,9162,9169,9172,9173,9180,9183,9186,9187,9190,9197,9201,9204,9206,9212,9214,9216,9236,9252,9255,9260,9264,9270,9274,9275,9279,9289,9297,9307,9313,9315,9324,9327,9329,9331,9336,9340,9344,9347,9352,9357,9361,9363,9371,9373,9376,9380,9384,9388,9394,9399,9406,9410,9414,9434,9435,9437,9439,9440,9445,9450,9454,9455,9468,9469,9475,9482,9483,9487,9490,9492,9497,9501,9503,9505,9507,9508,9513,9515,9516,9520,9521,9524,9527,9529,9530,9533,9536,9537,9538,9539,9540,9541,9543,9544,9546,9548,9552,9553,9556,9846,9861,9870,10636,11103,11613,11619,11627,12195,12202,12701,12703,12997,13007,13659,13661,14021,14023,14026,14407,14413,14416,14588,14598,15186,15295,15633,15641,15720,16121,16244,16975,17025,17388,17477,17488,17493` | absolute machine path |
| `14cb8fc9b2fa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libpxfm-1654d5d94a250952.rmeta:8099,8102,8108,8115,8118,8127,8132,8140,8141,8142,8154,8163,8175,8186,8194,8201,8207,8213,8218,8224,8234,8249,8257,8269,8277,8284,8290,8293,8298,8303,8308,8311,8317,8323,8330,8343,8353,8366,8376,8384,8406,8427,8437,8449,8457,8466,8469,8471,8478,8482,8489,8544,8579,8584,8594,8599,8606,8608,8613,8618,8623,8627,8631,8637,8638,8647,8655,8660,8670,8675,8682,8688,8692,8697,8698,8705,8713,8717,8721,8727,8730,8733,8743,8753,8760,8766,8776,8787,8799,8807,8823,8828,8834,8839,8842,8845,8862,8873,8879,8884,8888,8890,8895,8899,8904,8907,8914,8918,8925,8928,8933,8939,8945,8949,8950,8954,8959,8966,8967,8977,8984,8992,8996,9000,9002,9010,9024,9030,9031,9034,9035,9042,9045,9048,9069,9078,9093,9105,9117,9119,9121,9126,9137,9138,9140,9142,9149,9152,9153,9160,9163,9166,9167,9170,9177,9181,9184,9186,9192,9194,9196,9216,9232,9235,9240,9244,9250,9254,9255,9259,9269,9277,9287,9293,9295,9304,9307,9309,9311,9316,9320,9324,9327,9332,9337,9341,9343,9351,9353,9356,9360,9364,9368,9374,9379,9386,9390,9394,9414,9415,9417,9419,9420,9425,9430,9434,9435,9448,9449,9455,9462,9463,9467,9470,9472,9477,9481,9483,9485,9487,9488,9493,9495,9496,9500,9501,9504,9507,9509,9510,9513,9516,9517,9518,9519,9520,9521,9523,9524,9526,9528,9532,9533,9536` | absolute machine path |
| `7cf874af1a18` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libqoi-5eb53c08cdd21e4b.rlib:1249,1250,1262,1264,1273,1274,1278,1280,1288,1293` | absolute machine path |
| `c0a51d78df8f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libqoi-5eb53c08cdd21e4b.rmeta:1240,1241,1253,1255,1264,1265,1269,1271,1279` | absolute machine path |
| `c043cf12e437` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libquick_error-9f3f27b54b511427.rlib:170` | absolute machine path |
| `80a328a7aa0f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libquick_error-9f3f27b54b511427.rmeta:165` | absolute machine path |
| `91682b04c78c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librand-7a1425fd14f774cb.rlib:10672,10681,10691,10698,10706,10715,10725,10727,10737,10764,10782,10802,10828,10845,10850,10851,10852,10856,10857,10878,10912,10939,10952,10954,10955,10959,10962,10965,10978` | absolute machine path |
| `5323cd06d154` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librand-7a1425fd14f774cb.rmeta:10656,10665,10675,10682,10690,10699,10709,10711,10721,10748,10766,10786,10812,10829,10834,10835,10836,10840,10841,10862,10896,10923,10936,10938,10939,10943,10946,10949` | absolute machine path |
| `f39807ee2e78` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librand_chacha-a1f31bafc6606475.rlib:622,648,651,699` | absolute machine path |
| `7d23c69bda3a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librand_chacha-a1f31bafc6606475.rmeta:609,635,638` | absolute machine path |
| `2a5630799c32` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librand_core-54e8c7a08a94ef1a.rlib:582,596,598,627,629` | absolute machine path |
| `f2dacfcacbc8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librand_core-54e8c7a08a94ef1a.rmeta:575,589,591,620,622` | absolute machine path |
| `8f5563d65fc7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librangemap-4493a95bc35fb835.rlib:2235,2284,2319,2357,2384,2389,2391` | absolute machine path |
| `f9daa3d9cded` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librangemap-4493a95bc35fb835.rmeta:2230,2279,2314,2352,2379,2384,2386` | absolute machine path |
| `136d58ee8c13` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libraw_window_handle-1a32a61e9ce556b9.rlib:1251,1258,1259,1261,1264,1266,1267,1268,1272,1284,1285` | absolute machine path |
| `3b8c81392fa9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libraw_window_handle-1a32a61e9ce556b9.rmeta:1231,1238,1239,1241,1244,1246,1247,1248,1252,1264,1265` | absolute machine path |
| `a672d6bbfc6c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librayon-0f7e8d487ddc663c.rlib:16096,16227,16228,16230,16243,16253,16262,16267,16281,16293,16351,16395,16397,16404,16409,16419,16426,16429,16435,16436,16437,16439,16442,16445,16453,16470,16485,16488,16500,16514,16520,16548,16562,16568,16575,16611,16616,16621,16626,16632,16637,16643,16648,16665,16666,16685,16699,16719,16723,16741,16757,16771,16801,16803,16818,16819,16823,16835,16855,16861,16868,16872,16886,16893,16898,16905,16913,16916,16923,16928,16933,16941,16949,16963,16968,16973,16990,17004,17012,17016,17017,17024,17034,17056,17078,17079,17087,17140,17143,17152,17153,17173,17236` | absolute machine path |
| `0225a4a2b3cf` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librayon-0f7e8d487ddc663c.rmeta:16078,16209,16210,16212,16225,16235,16244,16249,16263,16275,16333,16377,16379,16386,16391,16401,16408,16411,16417,16418,16419,16421,16424,16427,16435,16452,16467,16470,16482,16496,16502,16530,16544,16550,16557,16593,16598,16603,16608,16614,16619,16625,16630,16647,16648,16667,16681,16701,16705,16723,16739,16753,16783,16785,16800,16801,16805,16817,16837,16843,16850,16854,16868,16875,16880,16887,16895,16898,16905,16910,16915,16923,16931,16945,16950,16955,16972,16986,16994,16998,16999,17006,17016,17038,17060,17061,17069,17122,17125,17134,17135` | absolute machine path |
| `9bfe2d3289c7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librayon_core-ca691849154bce37.rlib:2968,2970,2974,2979,2994,2999,3000,3018,3024,3049,3050,3085,3100,3110,3121,3122,3123,3191,3223,3259,3341,3389,3428,3463` | absolute machine path |
| `47e0e450414b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librayon_core-ca691849154bce37.rmeta:2853,2855,2859,2864,2879,2884,2885,2903,2909,2934,2935,2970,2985,2995,3006,3007,3008` | absolute machine path |
| `2e732055f2ce` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libread_fonts-e1b9c91a5a20b3b0.rlib:26321,26338,26340,26368,26411,26413,26418,26442,26518,26526,26530,26531,26556,26557,26571,26572,26574,26578,26622,26669,26672,26673,26678,26680,26682,26690,26696,26701,26728,26748,26752,26806,26807,26818,26821,26823,26827,26841,26857,26867,26910,26918,26921,26925,26932,26936,26939,26961,26963,26976,26993,27005,27020,27060,27061,27065,27068,27090,27140,27143,27149,27154,27156,27174,27177,27180,27200,27225,27244,27245,27246,27247,27269,27270,27295,27364,27378,27379,27395,27396,27397,27401,27404,27418,27433,27443,27480,27550,27579,27606,27621,27622,27647,27648,27650,27651,27658,27662,27718,27719,27801,27803,27804,27806,27813,27814,27816,27820,27833,27836,27837,27852,27880,27883,27886,27905,27906,27907,27909,27910,27914,27915,27925,27926,28476,28478,28486,28509,28511,29575,29911,30139,30360,30460,30542,30723,31266,31399,31523,31620,31673,31701` | absolute machine path |
| `3ca1f0cece43` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libread_fonts-e1b9c91a5a20b3b0.rmeta:26301,26318,26320,26348,26391,26393,26398,26422,26498,26506,26510,26511,26536,26537,26551,26552,26554,26558,26602,26649,26652,26653,26658,26660,26662,26670,26676,26681,26708,26728,26732,26786,26787,26798,26801,26803,26807,26821,26837,26847,26890,26898,26901,26905,26912,26916,26919,26941,26943,26956,26973,26985,27000,27040,27041,27045,27048,27070,27120,27123,27129,27134,27136,27154,27157,27160,27180,27205,27224,27225,27226,27227,27249,27250,27275,27344,27358,27359,27375,27376,27377,27381,27384,27398,27413,27423,27460,27530,27559,27586,27601,27602,27627,27628,27630,27631,27638,27642,27698,27699,27781,27783,27784,27786,27793,27794,27796,27800,27813,27816,27817,27832,27860,27863,27866,27885,27886,27887,27889,27890,27894,27895,27905,27906` | absolute machine path |
| `2f1ef18d2bbb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libread_fonts-fcbcf0d6d95637b5.rlib:30871,30889,30895,30922,30966,30968,30974,30998,31074,31082,31088,31091,31094,31097,31119,31136,31145,31147,31160,31181,31184,31210,31215,31270,31271,31288,31300,31317,31393,31396,31397,31422,31423,31438,31440,31484,31531,31534,31535,31540,31542,31544,31557,31563,31568,31595,31615,31619,31692,31693,31704,31707,31709,31713,31727,31743,31744,31755,31798,31806,31809,31813,31820,31824,31825,31829,31832,31855,31922,31925,31929,31932,31972,31975,31981,32022,32024,32028,32032,32035,32056,32092,32100,32107,32145,32178,32198,32201,32204,32246,32367,32383,32387,32406,32409,32412,32420,32427,32443,32475,32485,32530,32636,32661,32814,32864,32886,32913,32916,32919,32940,32950,33006,33007,33011,33014,33019,33031,33032,33036,33041,33056,33059,33097,33104,33111,33143,33146,33150,33153,33160,33161,33162,33163,33164,33445,33895,34074,34309,34310,34393,34769,34816,35195,35197,35428,35550,35571,35689,35691,35789,35792,35794,35839,36062,36067,36069,36383,36385,36393,36617,36621,36740` | absolute machine path |
| `10ecb55ccbdd` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libread_fonts-fcbcf0d6d95637b5.rmeta:30703,30721,30727,30754,30798,30800,30806,30830,30906,30914,30920,30923,30926,30929,30951,30968,30977,30979,30992,31013,31016,31042,31047,31102,31103,31120,31132,31149,31225,31228,31229,31254,31255,31270,31272,31316,31363,31366,31367,31372,31374,31376,31389,31395,31400,31427,31447,31451,31524,31525,31536,31539,31541,31545,31559,31575,31576,31587,31630,31638,31641,31645,31652,31656,31657,31661,31664,31687,31754,31757,31761,31764,31804,31807,31813,31854,31856,31860,31864,31867,31888,31924,31932,31939,31977,32010,32030,32033,32036,32078,32199,32215,32219,32238,32241,32244,32252,32259,32275,32307,32317,32362,32468,32493,32646,32696,32718,32745,32748,32751,32772,32782,32838,32839,32843,32846,32851,32863,32864,32868,32873,32888,32891,32929,32936,32943,32975,32978,32982,32985,32992,32993,32994,32995,32996` | absolute machine path |
| `7ef5364bd082` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libreborrow-072db6232540a65b.rlib:72` | absolute machine path |
| `3023bdcd3e42` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libreborrow-072db6232540a65b.rmeta:67` | absolute machine path |
| `cc1965e3fb0c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libref_cast-48044fb5660284b2.rlib:79,80,85,86` | absolute machine path |
| `97ea9921b87e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libref_cast-48044fb5660284b2.rmeta:74,75,80,81` | absolute machine path |
| `04011eeeba22` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librefineable_gpui_unofficial-c31cf0061b740733.rlib:129` | absolute machine path |
| `cd24a67619e0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librefineable_gpui_unofficial-c31cf0061b740733.rmeta:124` | absolute machine path |
| `e943e2d0b145` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libregex-b343816b272c865f.rlib:2983,2989,3121,3123,3130` | email |
| `e943e2d0b145` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libregex-b343816b272c865f.rlib:3389,3395,3410,3470,3474,3490,3495,3500,3525,3555,3607,3679,3687,3704` | absolute machine path |
| `80d42ef64e16` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libregex-b343816b272c865f.rmeta:2963,2969,3101,3103,3110` | email |
| `80d42ef64e16` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libregex-b343816b272c865f.rmeta:3369,3375,3390,3450,3454,3470,3475,3480` | absolute machine path |
| `0a03b535d2d6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libregex_automata-53e8e8011c06df59.rlib:16242,16313,16323,16370,16380,16432,16440,16446,16522,16688,16707,16718,16753,16764,16783,16808,16819,16846,16868,16874,16886,16920,16962,16972,16978,16997,17014,17045,17073,17076,17087,17093,17096,17099,17102,17119,17126,17133,17170,17173,17180,17200,17207,17237,17257,17277,17281,17283,17284,17286,17306,17307,17319,17320,17786,17787,17792,17794,17969,17970,18151,18274,18440,18447,18448,18567,18568,18680,18681,18806,19038,19040,19171,19172,19174,19325,19336,19337,19338,19772,19774,19928,19936,19944,20050,20051,20057,20304,20305,20311` | absolute machine path |
| `e6c12519a4a2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libregex_automata-53e8e8011c06df59.rmeta:16222,16293,16303,16350,16360,16412,16420,16426,16502,16668,16687,16698,16733,16744,16763,16788,16799,16826,16848,16854,16866,16900,16942,16952,16958,16977,16994,17025,17053,17056,17067,17073,17076,17079,17082,17099,17106,17113,17150,17153,17160,17180,17187,17217,17237,17257,17261,17263,17264,17266,17286,17287,17299,17300` | absolute machine path |
| `6f1e2678b3c1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libregex_syntax-ef5c4727eab52c09.rlib:14766,14889,14907,14951,14965,15020,15028,15077,15138,15166,15181,15184,15194,15222,15226,15231,15242,15250,16011,16012,16025,16039,16471,16605,16695,17042,17329,17448,17897,18414,18537,21387,21404` | absolute machine path |
| `8f7ed0d95222` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libregex_syntax-ef5c4727eab52c09.rmeta:14746,14869,14887,14931,14945,15000,15008,15057,15118,15146,15161,15164,15174,15202,15206,15211,15222,15230,15991,15992,16005,16019` | absolute machine path |
| `e9639e746ff5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libresvg-fac76453d1bfc570.rlib:644,655,668,684,688,689,708,714,720,723,727,729,731,734,741,743,751,1029,1035,1235,1493,1834,1910,1979,2026,2028,2121,2322,2325,2330,2368,2438,2439,2519,2715,2862,2920` | absolute machine path |
| `9f5d69dec089` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libresvg-fac76453d1bfc570.rmeta:624,635,648,664,668,669,688,694,700,703,707,709,711,714,721,723,731` | absolute machine path |
| `e582091e546d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librgb-40177f9787752466.rlib:5071,5072,5073,5080,5115,5116,5118,5119,5124,5130,5137,5150,5153,5159` | absolute machine path |
| `ebd805cd8dc6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librgb-40177f9787752466.rmeta:5066,5067,5068,5075,5110,5111,5113,5114,5119,5125,5132,5145,5148,5154` | absolute machine path |
| `1d4b6580daf6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libroxmltree-d188a8cd2e829371.rlib:2163,2202,2241,2607,2793,2854` | absolute machine path |
| `bbbee4502e9b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libroxmltree-d188a8cd2e829371.rmeta:2063,2102,2141` | absolute machine path |
| `7d95671c665c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librustc_demangle-747047182eea2180.rlib:236,246,253,409,489,524` | absolute machine path |
| `b2f229972967` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librustc_demangle-747047182eea2180.rmeta:223,233,240` | absolute machine path |
| `f18edeaa9684` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librustc_hash-129e8a99a60a8990.rlib:307,317` | absolute machine path |
| `d9a5220bedde` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librustc_hash-129e8a99a60a8990.rmeta:302,312` | absolute machine path |
| `d1ef4de8c6c3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librustc_hash-220228dd22b1fe2d.rlib:77` | absolute machine path |
| `531acadfd27b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librustc_hash-220228dd22b1fe2d.rmeta:72` | absolute machine path |
| `7bc29cd9e7c4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librustybuzz-93352d13fa1168a8.rlib:8956,8957,8970,8986,9001,9013,9014,9031,9041,9048,9144,9147,9165,9192,9201,9207,9214,9215,9216,9267,9270,9329,9343,9385,9389,9410,9417,9419,9422,9469,9474,9480,9486,9491,9503,9506,9515,9531,9536,9537,9549,9565,9566,9582,9584,9589,9590,9593,9597,9600,9601,9602,9604,9605,9606,9609,9610,9612,9627,9643,9644,9645,9661,9665,9815,9937,10028,10137,10139,10245,10360,10361,10362,10563,10564,10650,10658,10805,10952,11490,11492,11629,11887,11955,11960,11968,12531,12538,12753,12754,12762,12926,13324` | absolute machine path |
| `f04bb8876403` · `target-webdeploy/wasm32-unknown-unknown/web/deps/librustybuzz-93352d13fa1168a8.rmeta:8936,8937,8950,8966,8981,8993,8994,9011,9021,9028,9124,9127,9145,9172,9181,9187,9194,9195,9196,9247,9250,9309,9323,9365,9369,9390,9397,9399,9402,9449,9454,9460,9466,9471,9483,9486,9495,9511,9516,9517,9529,9545,9546,9562,9564,9569,9570,9573,9577,9580,9581,9582,9584,9585,9586,9589,9590,9592,9607,9623,9624,9625,9641,9645` | absolute machine path |
| `f548a569a9ce` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libryu-4d683bce611b26ef.rlib:153,157,177,178,182,185,194,197,209` | absolute machine path |
| `5d80c16a9f95` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libryu-4d683bce611b26ef.rmeta:143,147,167,168,172,175,184,187,199` | absolute machine path |
| `bfa35f18b070` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libscheduler_gpui_unofficial-c792f47be59ca487.rlib:1106,1112,1145,1173,1292,1595,1667,1681,1700,1847,1870,1907` | absolute machine path |
| `6ece44b630c6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libscheduler_gpui_unofficial-c792f47be59ca487.rmeta:1086,1092,1125,1153` | absolute machine path |
| `a74e37bfbbae` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libschemars-45de1fa0030cb63d.rlib:10361,10364,10367,10372,10385,10403,10404,10420,10426,10430,10439,10461,10463,10467,10488,10497,10512,10513,11222,11336,11457,11458,11524,11646,11712,11743,11776,11795,11823,11824,11873,11912,11936,11937,11959,11992` | absolute machine path |
| `7f9fe782137a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libschemars-45de1fa0030cb63d.rmeta:10316,10319,10322,10327,10340,10358,10359,10375,10381,10385,10394,10416,10418,10422,10443,10452,10467,10468` | absolute machine path |
| `1bcdcd5d65f2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libscopeguard-620383605d5536b2.rlib:55` | absolute machine path |
| `a601f9f66b91` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libscopeguard-620383605d5536b2.rmeta:50` | absolute machine path |
| `546ba130a437` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libseahash-20fa559f4a4c1721.rlib:130,137,139,142,146,166` | absolute machine path |
| `cbcc106ce651` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libseahash-20fa559f4a4c1721.rmeta:122,129,131,134,138` | absolute machine path |
| `e2c36d95b666` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libself_cell-b5e95521fa0e62c1.rlib:179,188` | absolute machine path |
| `4c1130ee1194` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libself_cell-b5e95521fa0e62c1.rmeta:174,183` | absolute machine path |
| `770f60124df9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde-5b2fbec1f44be650.rlib:3491,3495,3970,4067` | absolute machine path |
| `d4355c954f93` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde-5b2fbec1f44be650.rmeta:3479,3483,3958,4055` | absolute machine path |
| `26099da44a16` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde_core-ff5d10784170d596.rlib:25311,25365,25618` | email |
| `26099da44a16` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde_core-ff5d10784170d596.rlib:27374,27377,27483,27668,27821,27974,27980,27982,28000,28001,28007,28073,28082,28083,28087,28089,28090,28091,28103,28148,28187` | absolute machine path |
| `d23e2ad67b0f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde_core-ff5d10784170d596.rmeta:25293,25347,25600` | email |
| `d23e2ad67b0f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde_core-ff5d10784170d596.rmeta:27356,27359,27465,27650,27803,27956,27962,27964,27982,27983,27989,28055,28064,28065,28069,28071,28072,28073` | absolute machine path |
| `cef1295d7819` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde_json-7dbef7da475981d9.rlib:7623,7770,7835,7936,7943,8082,8125,8168,8205,8207,8224,8287,8342,8345,8355,8395,8570,8621,8660,8720,8735,8776,8804,8891,8929,8968` | absolute machine path |
| `2f27f38a1e80` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde_json-7dbef7da475981d9.rmeta:7603,7750,7815,7916,7923,8062,8105,8148,8185,8187,8204,8267,8322,8325,8335` | absolute machine path |
| `c773f6ef6be3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde_urlencoded-222a9e4582623b1f.rlib:818,832,835,840,843,851` | absolute machine path |
| `fec87b3e6291` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libserde_urlencoded-222a9e4582623b1f.rmeta:809,823,826,831,834,842` | absolute machine path |
| `73ba22aacc3e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsha2-e8fb2a681ef22e27.rlib:406,408,409,410,411` | absolute machine path |
| `4489644dc861` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsha2-e8fb2a681ef22e27.rmeta:395,397,398,399,400` | absolute machine path |
| `e8fa80378984` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsimd_adler32-6e8328c876d5ee51.rlib:196,204,212,217,224,232,241,248,261` | absolute machine path |
| `21adc82f00a7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsimd_adler32-6e8328c876d5ee51.rmeta:189,197,205,210,217,225,234,241,254` | absolute machine path |
| `b4e2c038e926` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsimplecss-d84bdb51be40f843.rlib:610,626,646,719,850,907,1696` | absolute machine path |
| `03fd1fd5297a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsimplecss-d84bdb51be40f843.rmeta:589,605,625` | absolute machine path |
| `35fae32f60b5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsiphasher-abea6e109e7fe029.rlib:632,634,641,650` | absolute machine path |
| `4b231abf3b41` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsiphasher-abea6e109e7fe029.rmeta:625,627,634,643` | absolute machine path |
| `cab9cec7baac` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libskrifa-4d7e1edd58cb1bc7.rlib:9761,9765,9775,9783,9812,9818,9847,9856,9860,9879,9909,9924,9937,9946,9947,9958,9980,10005,10023,10037,10057,10119,10150,10163,10167,10176,10193,10197,10214,10220,10223,10228,10240,10258,10262,10272,10319,10320,10323,10334,10338,10344,10349,10358,10362,10374,10395,10397,10436,10453,10457,10471,10484,10541,10544,10554,10570,10575,10590,10599,10603,10608,10624,10626,10628,10631,10637,10644,10645,10649,10650,10652,10654,11617,11698,11885,12082,12287,12422,12920,12921,12922,12925,13067,13312,13459,13740,14114,14305,14354,14415` | absolute machine path |
| `fcf8c06a6363` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libskrifa-4d7e1edd58cb1bc7.rmeta:9741,9745,9755,9763,9792,9798,9827,9836,9840,9859,9889,9904,9917,9926,9927,9938,9960,9985,10003,10017,10037,10099,10130,10143,10147,10156,10173,10177,10194,10200,10203,10208,10220,10238,10242,10252,10299,10300,10303,10314,10318,10324,10329,10338,10342,10354,10375,10377,10416,10433,10437,10451,10464,10521,10524,10534,10550,10555,10570,10579,10583,10588,10604,10606,10608,10611,10617,10624,10625,10629,10630,10632,10634` | absolute machine path |
| `f02dca1eb779` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libskrifa-60b8304193cf70c9.rlib:9261,9265,9278,9286,9315,9343,9354,9358,9377,9409,9424,9438,9448,9451,9462,9485,9511,9530,9545,9565,9628,9645,9658,9662,9671,9687,9691,9708,9714,9717,9722,9734,9752,9756,9766,9813,9814,9817,9828,9832,9838,9842,9852,9856,9868,9889,9891,9931,9949,9953,9967,9980,10026,10089,10099,10116,10121,10137,10146,10151,10156,10158,10162,10179,10181,10183,10186,10192,10193,10200,10201,10206,10208,10209,10211,11553,12106,12468,12712,12752,12753,13162,13467,13468,13913,14044,14086,14087,14158,14159,14397,14398,14438,14487,14547,14548,14740,14908,15021,15425` | absolute machine path |
| `2a89d8f428ba` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libskrifa-60b8304193cf70c9.rmeta:9241,9245,9258,9266,9295,9323,9334,9338,9357,9389,9404,9418,9428,9431,9442,9465,9491,9510,9525,9545,9608,9625,9638,9642,9651,9667,9671,9688,9694,9697,9702,9714,9732,9736,9746,9793,9794,9797,9808,9812,9818,9822,9832,9836,9848,9869,9871,9911,9929,9933,9947,9960,10006,10069,10079,10096,10101,10117,10126,10131,10136,10138,10142,10159,10161,10163,10166,10172,10173,10180,10181,10186,10188,10189,10191` | absolute machine path |
| `c42a7b4705f7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libslab-0e8f67bfda3a8b52.rlib:653,711` | absolute machine path |
| `c6192cdf136b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libslab-0e8f67bfda3a8b52.rmeta:648,706` | absolute machine path |
| `2470419f13af` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libslotmap-2efa131d275a99bb.rlib:10257,10270,10325,10372,10427,10495,10548,10559` | absolute machine path |
| `d6518b445ecc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libslotmap-2efa131d275a99bb.rmeta:10250,10263,10318,10365,10420,10488,10541` | absolute machine path |
| `d9796762a50b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsmallvec-7f9c67a493f3cf7e.rlib:1058` | absolute machine path |
| `f51480f128ea` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsmallvec-7f9c67a493f3cf7e.rmeta:1051` | absolute machine path |
| `940cf433de73` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsmol_str-332b5043805fb49c.rlib:1268,1321,1382` | absolute machine path |
| `9f83a51dcaf9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsmol_str-332b5043805fb49c.rmeta:1258,1311` | absolute machine path |
| `6990ef413795` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libspin-8dd9c9ee29c686a3.rlib:413,414,435,440` | absolute machine path |
| `eec2b65870ff` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libspin-8dd9c9ee29c686a3.rmeta:408,409,430,435` | absolute machine path |
| `2879d972d680` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libspin-989f6c4f5a7e152c.rlib:871,872,880,901,906,938,983` | absolute machine path |
| `8d433239b88f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libspin-989f6c4f5a7e152c.rmeta:866,867,875,896,901,933,978` | absolute machine path |
| `fb62abe9e003` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libstable_deref_trait-660c9f4b32bcd903.rlib:127` | absolute machine path |
| `56ad992dee0c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libstable_deref_trait-660c9f4b32bcd903.rmeta:122` | absolute machine path |
| `73382071d55b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libstatic_assertions-e3adf4772183ad05.rlib:177,179,180,181,182` | absolute machine path |
| `adfd67518e6b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libstatic_assertions-e3adf4772183ad05.rmeta:172,174,175,176,177` | absolute machine path |
| `d4589218478d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libstrict_num-caccc63f04300133.rlib:564` | absolute machine path |
| `8b9e5819c45e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libstrict_num-caccc63f04300133.rmeta:559` | absolute machine path |
| `5f87cc4bb863` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libstrum-7dcf42c15ea6101e.rlib:66,70` | absolute machine path |
| `edd076e8b2e5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libstrum-7dcf42c15ea6101e.rmeta:61,65` | absolute machine path |
| `0a50a8fd275a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsum_tree_gpui_unofficial-de64db781cc3f71c.rlib:4306,4368,4375` | absolute machine path |
| `33de0b279420` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsum_tree_gpui_unofficial-de64db781cc3f71c.rmeta:4301,4363,4370` | absolute machine path |
| `48471ead9166` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsvg_fmt-b81f9a2bc11a0af0.rlib:469,470,499` | absolute machine path |
| `57ccc7b51ba7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsvg_fmt-b81f9a2bc11a0af0.rmeta:461,462,491` | absolute machine path |
| `bbebb07eaad9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsvgtypes-abb48c9ca42ce4fb.rlib:2238,2242,2247,2256,2261,2263,2271,2283,2291,2301,2312,2318,2326,2361,2362,2371,2382,2387,2409,2414,2418,2512,2603,2665,2692,2729,2879,2940,2959,2989,2998` | absolute machine path |
| `621262b3099e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsvgtypes-abb48c9ca42ce4fb.rmeta:2218,2222,2227,2236,2241,2243,2251,2263,2271,2281,2292,2298,2306,2341,2342,2351,2362,2367,2389,2394,2398` | absolute machine path |
| `95df4c068bd0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libswash-5dbb02fc3075bbc3.rlib:41434,41465,41471,41497,41516,41542,41546,41550,41552,41566,41568,41580,41589,41669,41692,41730,41745,41746,41769,41779,41780,41787,41801,41810,41812,41825,41838,41846,41847,41859,41862,41865,41866,41867,41874,41944,41978,41986,42004,42026,42048,42057,42067,42077,42097,42100,42105,42125,42126,42141,42143,42151,42156,42158,42166,42509,42768,43255,43329,43400,43491,43514,43517,43746,44414,44758,44909,45042,45043,45060,45278,45279,45568,45845,45999,46452,46480,46651,46652,46724,46727,47168,47207,47208,47458` | absolute machine path |
| `dc42aa6dec41` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libswash-5dbb02fc3075bbc3.rmeta:41347,41378,41384,41410,41429,41455,41459,41463,41465,41479,41481,41493,41502,41582,41605,41643,41658,41659,41682,41692,41693,41700,41714,41723,41725,41738,41751,41759,41760,41772,41775,41778,41779,41780,41787,41857,41891,41899,41917,41939,41961,41970,41980,41990,42010,42013,42018,42038,42039,42054,42056,42064,42069,42071,42079` | absolute machine path |
| `1d630f0a753b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsys_locale-811cdc35daf0d4a4.rlib:31` | absolute machine path |
| `53f393825d26` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libsys_locale-811cdc35daf0d4a4.rmeta:26` | absolute machine path |
| `603a8e354a13` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtaffy-661d1a1b65274898.rlib:12061,12079,12087,12095,12118,12148,12163,12166,12174,12184,12216,12229,12235,12292,12315,12318,12329,12343,12377,12378,12412,12438,12444,12446,12449,12451,12453,12492,12519,12534,12540,12597,12610,12614,12617,12649,12660,12662,12670,12672,12679,12735,12816,12849,12867,12901,12942,12943,12969,12999,13005` | absolute machine path |
| `e9e036386e27` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtaffy-661d1a1b65274898.rmeta:12027,12045,12053,12061,12084,12114,12129,12132,12140,12150,12182,12195,12201,12258,12281,12284,12295,12309,12343,12344,12378,12404,12410,12412,12415,12417,12419,12458,12485,12500,12506,12563,12576,12580,12583,12615,12626,12628,12636,12638,12645` | absolute machine path |
| `085e93c0b761` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libthiserror-f7b42c3dfd36eb62.rlib:75,77,94` | absolute machine path |
| `87087b357ee1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libthiserror-f7b42c3dfd36eb62.rmeta:70,72,89` | absolute machine path |
| `ddadcbb03e82` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtiff-3eff5afc36e81231.rlib:9825,11989,11999,12065,12068,12069,12077,12078,12080,12115,12130,12149,12151,12202,12203,12205,12228,12235,12324,12476,12511,12659,12765` | absolute machine path |
| `9677867c3739` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtiff-3eff5afc36e81231.rmeta:9805,11969,11979,12045,12048,12049,12057,12058,12060,12095,12110,12129,12131,12182,12183,12185,12208,12215` | absolute machine path |
| `c21588bf5136` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtiny_skia-eb82e62852cf162d.rlib:4661,4667,4698,4701,4714,4715,4733,4736,4743,4757,4775,4785,4791,4796,4802,4828,4836,4847,4850,4851,4871,4881,4882,4884,4898,4907,4913,4917,4936,4981,4990,4994,4995,5020,5024,5034,5044,5050,5056,5063,5072,5076,5082,5626,5907,6066,6067,6129,6130,6166,6168,6169,6293,6296,6404,6499,6606,6609,6746,6747,6816,6951,6990,7066,7179,7302,7372` | absolute machine path |
| `f7e9002181b3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtiny_skia-eb82e62852cf162d.rmeta:4641,4647,4678,4681,4694,4695,4713,4716,4723,4737,4755,4765,4771,4776,4782,4808,4816,4827,4830,4831,4851,4861,4862,4864,4878,4887,4893,4897,4916,4961,4970,4974,4975,5000,5004,5014,5024,5030,5036,5043,5052,5056,5062` | absolute machine path |
| `e7db169a6a2a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtiny_skia_path-c485c5e9997d1fca.rlib:2198,2231,2235,2237,2248,2278,2299,2301,2386,2399,2405,2523,2607,2674,2697,2722,2750,2783` | absolute machine path |
| `a27176a16b7e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtiny_skia_path-c485c5e9997d1fca.rmeta:2178,2211,2215,2217,2228,2258,2279,2281,2366,2379,2385` | absolute machine path |
| `46b7d0b8d990` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtinystr-0c616a8f6617c05f.rlib:1732,1776,1777,1780,1781,1783` | absolute machine path |
| `bf670963d7fc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtinystr-0c616a8f6617c05f.rmeta:1726,1770,1771,1774,1775,1777` | absolute machine path |
| `04bdf6bf2fb7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtinyvec-ef3e589fa0c738f8.rlib:6432,6433,6481,6503,6547,6548,6549` | absolute machine path |
| `daf1da480574` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtinyvec-ef3e589fa0c738f8.rmeta:6427,6428,6476,6498,6542,6543,6544` | absolute machine path |
| `ff31088c9c94` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtracing-702315dd849ad322.rlib:319` | email |
| `ff31088c9c94` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtracing-702315dd849ad322.rlib:1264,1382,1605,1633,1641,1643,1648,1649,1656,1704` | absolute machine path |
| `d385ed14d1a6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtracing-702315dd849ad322.rmeta:308` | email |
| `d385ed14d1a6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtracing-702315dd849ad322.rmeta:1253,1371,1594,1622,1630,1632,1637,1638` | absolute machine path |
| `73d6fe0b7048` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtracing_core-d75813d2a3553c6f.rlib:2092,2097,2101,2120,2139,2170,2192,2201,2204,2205,2225,2250,2304,2384,2409,2460` | absolute machine path |
| `1789e1dd52b4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtracing_core-d75813d2a3553c6f.rmeta:1969,1974,1978,1997,2016,2047,2069,2078,2081,2082` | absolute machine path |
| `52a650f4a320` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libttf_parser-feeb9c904f8e481f.rlib:19059,19142,19161,19181,19190,19197,19279,19293,19328,19331,19333,19335,19340,19348,19352,19356,19357,19358,19363,19367,19379,19392,19398,19407,19409,19411,19412,19414,19416,19419,19422,19423,19428,19438,19473,19475,19500,19503,19504,19508,19516,19522,19523,19541,19558,19565,19578,19580,19581,19582,19594,19622,19630,19634,19635,19638,19647,19653,19657,19662,19664,19665,19669,19671,19672,19676,19932,19995,20073,20181,20301,20365,20510,20593,20869,21032,21099,21351` | absolute machine path |
| `d5d674ec3065` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libttf_parser-feeb9c904f8e481f.rmeta:19039,19122,19141,19161,19170,19177,19259,19273,19308,19311,19313,19315,19320,19328,19332,19336,19337,19338,19343,19347,19359,19372,19378,19387,19389,19391,19392,19394,19396,19399,19402,19403,19408,19418,19453,19455,19480,19483,19484,19488,19496,19502,19503,19521,19538,19545,19558,19560,19561,19562,19574,19602,19610,19614,19615,19618,19627,19633,19637,19642,19644,19645,19649,19651,19652,19656` | absolute machine path |
| `85661baaef7c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtypenum-9239e4b41a691e69.rlib:9570,9578,9606,9659,9662,9663,9674,9687,9689,9690` | absolute machine path |
| `fbe78d4e75af` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libtypenum-9239e4b41a691e69.rmeta:9564,9572,9600,9653,9656,9657,9668,9681,9683,9684` | absolute machine path |
| `d4faccd1a7ab` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_bidi-a483b0bb212b6ed8.rlib:1951,1993,2001,2005,2008,2016,2024,2044,2046,2144,2184,2215,2239,2276,2300,2437,2442,2453` | absolute machine path |
| `9125675ccee5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_bidi-a483b0bb212b6ed8.rmeta:1871,1913,1921,1925,1928,1936,1944,1964,1966` | absolute machine path |
| `370afd869cdb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_bidi_mirroring-99194fddb7d8aa8a.rlib:57,70` | absolute machine path |
| `8123c492a86d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_bidi_mirroring-99194fddb7d8aa8a.rmeta:52` | absolute machine path |
| `d90a0ee6ad6f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_ccc-441f0b204fad2838.rlib:152` | absolute machine path |
| `1dbbdabc1421` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_ccc-441f0b204fad2838.rmeta:147` | absolute machine path |
| `4194aeb6a6a0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_ident-0c9875476da5e782.rlib:51,52` | absolute machine path |
| `ff1b69746ff4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_ident-0c9875476da5e782.rmeta:46,47` | absolute machine path |
| `5d4d1d61265e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_linebreak-765dd9a34a92fff2.rlib:344,363` | absolute machine path |
| `fb727d961d6f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_linebreak-765dd9a34a92fff2.rmeta:339` | absolute machine path |
| `b0b1dd62e263` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_properties-b24c242ae0fa020d.rlib:1875,1877,1889,2141` | absolute machine path |
| `28d94488468d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_properties-b24c242ae0fa020d.rmeta:1868,1870` | absolute machine path |
| `a34812fe7d0a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_script-9553be170227da46.rlib:2718,2733,2752,2818` | absolute machine path |
| `3bfeee1ae157` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_script-9553be170227da46.rmeta:2689,2704` | absolute machine path |
| `19e69d684b71` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_segmentation-a5c098667e98782e.rlib:4641,4642,4680,4714,4720,4873,4934,4944,4953,5051,5178` | absolute machine path |
| `32db9309ac53` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_segmentation-a5c098667e98782e.rmeta:4628,4629,4667,4701,4707` | absolute machine path |
| `88e83e4a9ae7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_vo-a6b4215d7084a06d.rlib:76` | absolute machine path |
| `dd92f8713460` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_vo-a6b4215d7084a06d.rmeta:71` | absolute machine path |
| `f071e3e8acfd` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_width-39815f21c027b8c0.rlib:290,1845,1855` | absolute machine path |
| `dfe8fe24038e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libunicode_width-39815f21c027b8c0.rmeta:285,1840` | absolute machine path |
| `4bc2544e82d2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liburl-8dcb9a89f6ee93c4.rlib:713,724,733,737,738,739,741,742,743,795` | email |
| `4bc2544e82d2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liburl-8dcb9a89f6ee93c4.rlib:1042,1129,1155,1169,1172,1254,1259,1620,1743,1919,1943,1963,1996,1997,2053` | absolute machine path |
| `4bc2544e82d2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liburl-8dcb9a89f6ee93c4.rlib:713` | URL credential |
| `a07e13be9fb7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liburl-8dcb9a89f6ee93c4.rmeta:693,704,713,717,718,719,721,722,723,775` | email |
| `a07e13be9fb7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liburl-8dcb9a89f6ee93c4.rmeta:1022,1109,1135,1149,1152,1234,1239` | absolute machine path |
| `a07e13be9fb7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/liburl-8dcb9a89f6ee93c4.rmeta:693` | URL credential |
| `c5e7708b50e8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libusvg-934f987afeb09662.rlib:17101,17145,17186,17237,17270,17279,17293,17295,17306,17333,17345,17367,17373,17378,17384,17427,17434,17440,17446,17495,17526,17549,17551,17553,17556,17561,17570,17573,17577,17631,17842,18154,18155,18435,18732,18735,18736,18903,19072,19247,19750,19751,19905,19908,19974,19975,19979,20238,20239,20256,20453,20455,20758,21058,21059` | absolute machine path |
| `17d8bbfea1ee` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libusvg-934f987afeb09662.rmeta:17081,17125,17166,17217,17250,17259,17273,17275,17286,17313,17325,17347,17353,17358,17364,17407,17414,17420,17426,17475,17506,17529,17531,17533,17536,17541,17550,17553,17557` | absolute machine path |
| `8e5a88544f82` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libutf8_iter-21d76da726885b21.rlib:242,253,255` | absolute machine path |
| `ab1202486b79` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libutf8_iter-21d76da726885b21.rmeta:236,247,249` | absolute machine path |
| `9554061204de` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libuuid-150872a78aed0f8d.rlib:3613,3640,3643,3659,3693,3739,3740,3760,3830,3832,3864,3894,3939,3945` | absolute machine path |
| `ef20c743e101` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libuuid-150872a78aed0f8d.rmeta:3551,3578,3581,3597,3631,3677,3678,3698,3768,3770` | absolute machine path |
| `6765a74ced75` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_app.rlib:69709` | email |
| `6765a74ced75` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_app.rlib:3743,3747,3795,3797,3802,3810,3817,3886,3900,3901,3903,3910,3913,3917,3920,7698,7704,9508,9510,10016,10339,11915,11916,11922,12130,68798,68799,68914,68916,73114,73600,74403,74410,74411,74518,74519,74725,74727,74849,75196` | absolute machine path |
| `6765a74ced75` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_app.rlib:7720,7732,7750,9534,10339,12167,68801,74412,74729,74843` | Git identity name/username |
| `b9e0de479fcc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_core-387de1f7042374a7.rlib:6910,6937,6941,6953,6999,7017,7032,10363,10482,10539` | absolute machine path |
| `b9e0de479fcc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_core-387de1f7042374a7.rlib:4381,4895` | Git identity name/username |
| `08ef5f15237a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_core-387de1f7042374a7.rmeta:6890,6917,6921,6933,6979,6997,7012` | absolute machine path |
| `08ef5f15237a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_core-387de1f7042374a7.rmeta:4361,4875` | Git identity name/username |
| `af46ae4b9c8b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_export-3e8142faad86f79a.rlib:1660,1765,2160,2251,2346,2366,2452,2467,2476,2492` | absolute machine path |
| `af46ae4b9c8b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_export-3e8142faad86f79a.rlib:1604` | Git identity name/username |
| `adb78b1fd52f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_export-3e8142faad86f79a.rmeta:1640` | absolute machine path |
| `adb78b1fd52f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_export-3e8142faad86f79a.rmeta:1584` | Git identity name/username |
| `e55f1b58d71d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_format-faeccfc19e3fb839.rlib:1217,1228,1665,1673,2060,2154,2178,2712,3243,3244,3245` | absolute machine path |
| `e55f1b58d71d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_format-faeccfc19e3fb839.rlib:477,1104,1105,1665` | Git identity name/username |
| `159d9c342fc3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_format-faeccfc19e3fb839.rmeta:1086,1097` | absolute machine path |
| `159d9c342fc3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_format-faeccfc19e3fb839.rmeta:346,973,974` | Git identity name/username |
| `3f619ea7d4dc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_render-3d6572dd7d1e40ca.rlib:356,360,370,396` | absolute machine path |
| `20602aa2ed90` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libvoxelite_render-3d6572dd7d1e40ca.rmeta:336,340,350,376` | absolute machine path |
| `9e598fbdcb3b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwaker_fn-cb27adc71c5bdf85.rlib:90` | absolute machine path |
| `d2d929e7e74f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwaker_fn-cb27adc71c5bdf85.rmeta:85` | absolute machine path |
| `efd829d0d975` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwasm_bindgen-5cac80edcdf2bb6a.rlib:11857,11893,11898,11905,11948,11974,12001,12009,12029,12040,12060,12070,12086,12088,12096,12164` | absolute machine path |
| `86c8e7202e13` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwasm_bindgen-5cac80edcdf2bb6a.rmeta:11837,11873,11878,11885,11928,11954,11981,11989,12009,12020,12040,12050,12066,12068,12076` | absolute machine path |
| `b0c149c33c26` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwasm_bindgen_futures-da3ed5422fe3e0de.rlib:20` | absolute machine path |
| `9f8c6b7e7219` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwasm_bindgen_futures-da3ed5422fe3e0de.rmeta:15` | absolute machine path |
| `9a7d7008bc21` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwasm_bindgen_shared-76ddab7b8a31480b.rlib:442,471,475` | absolute machine path |
| `e47445bf6fe8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwasm_bindgen_shared-76ddab7b8a31480b.rmeta:426,455,459` | absolute machine path |
| `82c04c4311ad` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libweb_sys-051ed307d996014c.rlib:38123` | email |
| `82c04c4311ad` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libweb_sys-051ed307d996014c.rlib:49765,49766,49767,49768,49770,49771,49772,49773,49774,49775,49776,49777,49778,49779,49780,49782,49784,51710,51786,51823` | absolute machine path |
| `74f6d4c7a986` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libweb_sys-051ed307d996014c.rmeta:38103` | email |
| `74f6d4c7a986` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libweb_sys-051ed307d996014c.rmeta:49745,49746,49747,49748,49750,49751,49752,49753,49754,49755,49756,49757,49758,49759,49760,49762,49764` | absolute machine path |
| `d36fdfeae787` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libweb_time-36cfabbe70dec486.rlib:264,266,268,269,276,295,307` | absolute machine path |
| `9e802d479a79` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libweb_time-36cfabbe70dec486.rmeta:253,255,257,258` | absolute machine path |
| `adab7cd24815` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libweezl-7092b48b4db555a4.rlib:858,859,901,933,1098,1172,1199,1225` | absolute machine path |
| `7b54c8489ad4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libweezl-7092b48b4db555a4.rmeta:840,841,883,915` | absolute machine path |
| `1c4c8817d96d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu-905c5cef53916ed4.rlib:40313,40327,40347,40355,40372,40381,40385,40386,40394,40395,40397,40399,40524,40706,40708,40712,40715,40717,40719,40721,40724,40739,40741,40743,40748,40756,40762,40769,40772,40775,40781,40783,40784,40785,40786,40787,40788,40789,40791,40792,40793,40794,40795,40796,40797,40798,40799,40800,40810,40813,40815,40819,41205,41206,41217,41466,41573,41581,41746,41751,41752,41753,41775,41965,41967,41970,42374,42485,42561,42562,42563,42795,42810,42811,43063,43066,43067,43068,43199,43200,43201,43434,43463,43464,43604,43608` | absolute machine path |
| `08fba369b8b8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu-905c5cef53916ed4.rmeta:40293,40307,40327,40335,40352,40361,40365,40366,40374,40375,40377,40379,40504,40686,40688,40692,40695,40697,40699,40701,40704,40719,40721,40723,40728,40736,40742,40749,40752,40755,40761,40763,40764,40765,40766,40767,40768,40769,40771,40772,40773,40774,40775,40776,40777,40778,40779,40780,40790,40793,40795,40799` | absolute machine path |
| `a1d3299b132a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_core-1e635a8c1ad1d4c2.rlib:23617,23656,23699,23700,23795,23796,23851,23873,23940,23955,24111,24115,24135,24136,24140,24144,24171,24178,24208,24216,24217,24221,24229,24239,24246,24249,24335,24344,24348,24353,24362,24368,24376,24378,24380,24395,24419,24420,24431,24465,24477,24478,24494,24533,24540,24558,24582,24585,24598,24601,24605,24610,24612,24615,24616,24622,24632,24650,24653,24679,24684,24687,24688,24732,24735,24737,24740,24938,25398,25402,25404,26027,26028,26704,26710,26711,27312,27313,27320,27321,27641,27645,28056,28061,28369,28373,28583,28584,29233,29241,30888,31412,31416,31635,31636,31642,32301,32305,32306,32329,32421,32797,32803,32812,33160,33163` | absolute machine path |
| `969de5776083` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_core-1e635a8c1ad1d4c2.rmeta:23597,23636,23679,23680,23775,23776,23831,23853,23920,23935,24091,24095,24115,24116,24120,24124,24151,24158,24188,24196,24197,24201,24209,24219,24226,24229,24315,24324,24328,24333,24342,24348,24356,24358,24360,24375,24399,24400,24411,24445,24457,24458,24474,24513,24520,24538,24562,24565,24578,24581,24585,24590,24592,24595,24596,24602,24612,24630,24633,24659,24664,24667,24668,24712,24715,24717,24720` | absolute machine path |
| `1fed1ede1593` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_core_deps_wasm-493cbd2a00c6013d.rlib:10` | absolute machine path |
| `c7101698316d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_core_deps_wasm-493cbd2a00c6013d.rmeta:5` | absolute machine path |
| `7d871593d28f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_hal-d063d53d1190bf35.rlib:10815,10845,10855,10884,10886,10892,10906,10969,11019,11029,11035,11039,11040,11047,11048,11064,11065,11068,11069,11582,11663,11683,12284,12290,12312,12318,12622,12690,12795,12835,13008,13026,13082,13133,13179,13202,13266,13367,13386` | absolute machine path |
| `7d871593d28f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_hal-d063d53d1190bf35.rlib:5658` | Git identity name/username |
| `737f10370f06` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_hal-d063d53d1190bf35.rmeta:10795,10825,10835,10864,10866,10872,10886,10949,10999,11009,11015,11019,11020,11027,11028,11044,11045,11048,11049` | absolute machine path |
| `737f10370f06` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_hal-d063d53d1190bf35.rmeta:5638` | Git identity name/username |
| `d556a978ce99` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_naga_bridge-4ae6a95c9ff6fdcb.rlib:59` | absolute machine path |
| `9508199b5e42` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_naga_bridge-4ae6a95c9ff6fdcb.rmeta:53` | absolute machine path |
| `11c7f97d1d22` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_types-1a876d57c819325d.rlib:22831` | email |
| `11c7f97d1d22` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_types-1a876d57c819325d.rlib:23691,23702,23707,23753,23755,23756,23761,23762,23787,23802,23819,23825,23827,23840,23857,23875,23881,23916,23917,23943,23944,23949,23954,24089,24121,24260,24484,24573` | absolute machine path |
| `e9fa027b4b5d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_types-1a876d57c819325d.rmeta:22811` | email |
| `e9fa027b4b5d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwgpu_types-1a876d57c819325d.rmeta:23671,23682,23687,23733,23735,23736,23741,23742,23767,23782,23799,23805,23807,23820,23837,23855,23861,23896,23897,23923,23924,23929,23934` | absolute machine path |
| `3b4400feac52` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwriteable-28733e6ab8792f7f.rlib:1077,1099,1106,1111,1118,1134,1138` | absolute machine path |
| `05098c647bcc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libwriteable-28733e6ab8792f7f.rmeta:1069,1091,1098,1103,1110,1126,1130` | absolute machine path |
| `ab27f5877f70` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libxmlwriter-e2485812164083e6.rlib:207,268` | absolute machine path |
| `e39581c463f6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libxmlwriter-e2485812164083e6.rmeta:189` | absolute machine path |
| `c1714b4d9f29` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libyazi-f1c7ff53562006a6.rlib:1135,1146,1260,1681,1965,2067` | absolute machine path |
| `8dc0c76f54aa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libyazi-f1c7ff53562006a6.rmeta:1121,1132,1246` | absolute machine path |
| `05ab0af045af` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libyoke-c6832570f3254543.rlib:461,463,483,555,576,577,580,591,592` | absolute machine path |
| `5830612379a6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libyoke-c6832570f3254543.rmeta:456,458,478,550,571,572,575,586,587` | absolute machine path |
| `c41bd552e3d2` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzeno-8a19d53d1525b3dd.rlib:4067,4075,4084,4107,4117,4130,4168,4182,4195,4229,4245,4253,4261,4307,4364,4439,4501` | absolute machine path |
| `7f8d1940b120` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzeno-8a19d53d1525b3dd.rmeta:4047,4055,4064,4087,4097,4110,4148,4162,4175,4209,4225,4233,4241,4287` | absolute machine path |
| `8cf48931a926` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerocopy-96d60c44bc5b5380.rlib:39768,45603` | email |
| `8cf48931a926` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerocopy-96d60c44bc5b5380.rlib:45660,45948,45983,46006,46035,46042,46101,46143,46162,46268,46331,46342,46439,46531,46603,46628,46644` | absolute machine path |
| `8cf48931a926` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerocopy-96d60c44bc5b5380.rlib:20174,33602` | Git identity name/username |
| `4c43a2078e5d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerocopy-96d60c44bc5b5380.rmeta:39763,45598` | email |
| `4c43a2078e5d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerocopy-96d60c44bc5b5380.rmeta:45655,45943,45978,46001,46030,46037,46096,46138,46157,46263,46326,46337,46434,46526,46598,46623,46639` | absolute machine path |
| `4c43a2078e5d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerocopy-96d60c44bc5b5380.rmeta:20169,33597` | Git identity name/username |
| `7cfa3f1d5bf1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerofrom-62f7362069a4c5ee.rlib:404,405,407` | absolute machine path |
| `c30ebc97c351` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerofrom-62f7362069a4c5ee.rmeta:399,400,402` | absolute machine path |
| `625fe7469138` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerotrie-31a94bc6aaa25a55.rlib:1866,1868,1869,1878,1888,1903,1919,1964,1965,1971,1987,1991,2000,2004,2048,2072,2078` | absolute machine path |
| `a7bd901c14d7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerotrie-31a94bc6aaa25a55.rmeta:1856,1858,1859,1868,1878,1893,1909,1954,1955,1961,1977,1981,1990,1994` | absolute machine path |
| `57d566cb7496` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerovec-657954b246df0b13.rlib:8431,8432,8445,8462,8466,8476,8504,8512,8515,8519,8520,8522,8527,8537,8539,8557,8564,8579,8582,8590,8591,8593,8596` | absolute machine path |
| `c5cc6eb8e3a4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzerovec-657954b246df0b13.rmeta:8424,8425,8438,8455,8459,8469,8497,8505,8508,8512,8513,8515,8520,8530,8532,8550,8557,8572,8575,8583,8584,8586,8589` | absolute machine path |
| `18a535066335` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzmij-f3342f52a38aada8.rlib:482,535` | absolute machine path |
| `74c6eaf04435` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzmij-f3342f52a38aada8.rmeta:476,529` | absolute machine path |
| `ad774635d5d1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libztracing_gpui_unofficial-501669480131ceac.rlib:42` | absolute machine path |
| `3eb3f24740fb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libztracing_gpui_unofficial-501669480131ceac.rmeta:37` | absolute machine path |
| `04c16e1fee0f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzune_core-bf5944db8230fce5.rlib:1177,1181,1193,1198,1201,1205,1212,1218,1220,1229,1248,1252,1254,1272,1280` | absolute machine path |
| `dea3d1ebbaeb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzune_core-bf5944db8230fce5.rmeta:1160,1164,1176,1181,1184,1188,1195,1201,1203,1212,1231,1235,1237` | absolute machine path |
| `6ad1c4fbd6bf` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzune_inflate-312e139591b2c4ec.rlib:243,245,308,318,319,329,331,420,475,493` | absolute machine path |
| `1fd946ed585a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzune_inflate-312e139591b2c4ec.rmeta:227,229,292,302,303,313,315` | absolute machine path |
| `9411b0f47ef0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzune_jpeg-c488f84a10839a23.rlib:4105,4109,4114,4116,4122,4125,4133,4136,4153,4165,4175,4186,4213,4218,4222,4229,4249,4261,4276,4282,4286,4287,4307,4308,4457,4557,4586,4635,4661,4704` | absolute machine path |
| `0cff43748f21` · `target-webdeploy/wasm32-unknown-unknown/web/deps/libzune_jpeg-c488f84a10839a23.rmeta:4085,4089,4094,4096,4102,4105,4113,4116,4133,4145,4155,4166,4193,4198,4202,4209,4229,4241,4256,4262,4266,4267,4287,4288` | absolute machine path |
| `3df714517df4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/linebender_resource_handle-240e79f74b69368b.d:1,3,5,7,8,9` | absolute machine path |
| `f6cd62aac67c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/link_section-f3ce6caa8c85c5cd.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24` | absolute machine path |
| `2a919ac26457` · `target-webdeploy/wasm32-unknown-unknown/web/deps/litemap-7e29ee619ce51ab6.d:1,3,5,7,8,9,10` | absolute machine path |
| `798f7f0d258e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/lock_api-4c759158cd368f4f.d:1,3,5,7,8,9,10` | absolute machine path |
| `2d3a2bc4bd48` · `target-webdeploy/wasm32-unknown-unknown/web/deps/log-576ffd46ce09ef54.d:1,3,5,7,8,9,10` | absolute machine path |
| `7c02ce8ca0dc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/lyon-9547470d76e3c736.d:1,3,5,7` | absolute machine path |
| `826ac2478001` · `target-webdeploy/wasm32-unknown-unknown/web/deps/lyon_algorithms-fdceae7e0f17945f.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `f9dc77a49f22` · `target-webdeploy/wasm32-unknown-unknown/web/deps/lyon_geom-9199d3f9321b7b6d.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `bb61afda2c18` · `target-webdeploy/wasm32-unknown-unknown/web/deps/lyon_path-06c73d2e08d2a3c6.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `4e09f445d284` · `target-webdeploy/wasm32-unknown-unknown/web/deps/lyon_tessellation-a8a0fe4992dbf4e2.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `3bba7626e5c1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/memchr-28a1ea40f6f490d0.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25` | absolute machine path |
| `f999edd3205b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/memmap2-c402c63e463fdfda.d:1,3,5,7,8` | absolute machine path |
| `334dd1e7388d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/miniz_oxide-960495ed1d6a3ad6.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18` | absolute machine path |
| `adb54f3de128` · `target-webdeploy/wasm32-unknown-unknown/web/deps/miniz_oxide-f59bee9a216466c3.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18` | absolute machine path |
| `623fea42dbeb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/moxcms-05adb9b5194d6a86.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68` | absolute machine path |
| `812e423c4f9b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/naga-6a55535d1711f3da.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91` | absolute machine path |
| `2c9d23a83ec3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/naga_types-4d301bbf9d4157b8.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `c7d2297562b1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/num_complex-f3b4f46b0c16b153.d:1,3,5,7,8,9,10` | absolute machine path |
| `399c2972bcc6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/num_cpus-d8364c06903d6185.d:1,3,5,7` | absolute machine path |
| `bd35832ea9d5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/num_traits-bc9fbf49325f1a55.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25` | absolute machine path |
| `e69a0faa4b4c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/object-3fd5ca6ac4cb9920.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68` | absolute machine path |
| `8c7e58045610` · `target-webdeploy/wasm32-unknown-unknown/web/deps/once_cell-673dc8c2b6897805.d:1,3,5,7,8,9` | absolute machine path |
| `6dd8041a9f9b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/parking-9479e98cacfb6f2d.d:1,3,5,7` | absolute machine path |
| `9fa2a05f47f4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/parking_lot-bb14e5daef146f73.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `cf3cad55bd89` · `target-webdeploy/wasm32-unknown-unknown/web/deps/parking_lot_core-17753d02173ceedb.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `23abdd91782e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/percent_encoding-0b7cab864b941d26.d:1,3,5,7,8` | absolute machine path |
| `eb9bf7e09414` · `target-webdeploy/wasm32-unknown-unknown/web/deps/pico_args-6491726215c01f7e.d:1,3,5,7` | absolute machine path |
| `26f3ae499410` · `target-webdeploy/wasm32-unknown-unknown/web/deps/pin_project-8823898d1a81d98b.d:1,3,5,7` | absolute machine path |
| `f710e8bb4032` · `target-webdeploy/wasm32-unknown-unknown/web/deps/pin_project_lite-6ac52b036886392f.d:1,3,5,7` | absolute machine path |
| `b18f9ca81dd3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/png-361490b173c5e408.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24` | absolute machine path |
| `454fde19ae19` · `target-webdeploy/wasm32-unknown-unknown/web/deps/png-ee9d781577d7ea37.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23` | absolute machine path |
| `b9b307df13d8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/polycool-c5e54fa897797165.d:1,3,5,7,8,9,10,11,12` | absolute machine path |
| `bddefefd1d09` · `target-webdeploy/wasm32-unknown-unknown/web/deps/potential_utf-b8b2102fc25c6921.d:1,3,5,7,8,9` | absolute machine path |
| `1cefcafb9b65` · `target-webdeploy/wasm32-unknown-unknown/web/deps/ppv_lite86-46352b6533f60de9.d:1,3,5,7,8,9,10` | absolute machine path |
| `82eea3647156` · `target-webdeploy/wasm32-unknown-unknown/web/deps/profiling-060301a67beb4e1c.d:1,3,5,7,8` | absolute machine path |
| `7698a04233f4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/pulp-31b2423eea4ce9d1.d:1,3,5,7,8,9,10` | absolute machine path |
| `e133f9f6d80b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/pulp_wasm_simd_flag-d4f2f9af727346e1.d:1,3,5,7` | absolute machine path |
| `977d1f637708` · `target-webdeploy/wasm32-unknown-unknown/web/deps/pxfm-1654d5d94a250952.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,181,182,183,184,185,186,187,188,189,190,191,192,193,194,195,196,197,198,199,200,201,202,203,204,205,206,207,208,209,210,211,212,213,214,215,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230,231,232,233,234,235,236,237,238,239,240,241,242,243,244,245,246,247,248,249,250,251,252,253,254,255,256,257,258,259,260,261,262,263,264,265` | absolute machine path |
| `c4c523c2235f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/qoi-5eb53c08cdd21e4b.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `ac387d14a3de` · `target-webdeploy/wasm32-unknown-unknown/web/deps/quick_error-9f3f27b54b511427.d:1,3,5,7` | absolute machine path |
| `a2f7c002b139` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rand-7a1425fd14f774cb.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36` | absolute machine path |
| `bf519916adc4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rand_chacha-a1f31bafc6606475.d:1,3,5,7,8,9` | absolute machine path |
| `e0f159ab5c94` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rand_core-54e8c7a08a94ef1a.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `8affab4b2168` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rangemap-4493a95bc35fb835.d:1,3,5,7,8,9,10,11,12,13,14` | absolute machine path |
| `c6063b3aeb50` · `target-webdeploy/wasm32-unknown-unknown/web/deps/raw_window_handle-1a32a61e9ce556b9.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17` | absolute machine path |
| `9f6f7052a933` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rayon-0f7e8d487ddc663c.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104` | absolute machine path |
| `50b522d73f87` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rayon_core-ca691849154bce37.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29` | absolute machine path |
| `5fecac58e87b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/read_fonts-e1b9c91a5a20b3b0.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149` | absolute machine path |
| `d414fa23c7dc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/read_fonts-fcbcf0d6d95637b5.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159,160,161` | absolute machine path |
| `c67307c015fc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/reborrow-072db6232540a65b.d:1,3,5,7` | absolute machine path |
| `7503c7142ad6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/ref_cast-48044fb5660284b2.d:1,3,5,7,8,9,10,11,12,14` | absolute machine path |
| `3dc33c9e40fa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/refineable_gpui_unofficial-c31cf0061b740733.d:1,3,5,7` | absolute machine path |
| `4ed37f7b01d0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/regex-b343816b272c865f.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17` | absolute machine path |
| `fe5a8ec19dd6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/regex_automata-53e8e8011c06df59.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67` | absolute machine path |
| `74851b36cfd6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/regex_syntax-ef5c4727eab52c09.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37` | absolute machine path |
| `bd5b75cbe3e1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/resvg-fac76453d1bfc570.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24` | absolute machine path |
| `879d5c09ea00` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rgb-40177f9787752466.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27` | absolute machine path |
| `d0978b176a84` · `target-webdeploy/wasm32-unknown-unknown/web/deps/roxmltree-d188a8cd2e829371.d:1,3,5,7,8,9` | absolute machine path |
| `2930413579f9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rustc_demangle-747047182eea2180.d:1,3,5,7,8,9` | absolute machine path |
| `19e5fb8f089c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rustc_hash-129e8a99a60a8990.d:1,3,5,7,8` | absolute machine path |
| `7182c4e29685` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rustc_hash-220228dd22b1fe2d.d:1,3,5,7` | absolute machine path |
| `f4e5a2519d86` · `target-webdeploy/wasm32-unknown-unknown/web/deps/rustybuzz-93352d13fa1168a8.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78` | absolute machine path |
| `6c0c67121e7c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/ryu-4d683bce611b26ef.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18` | absolute machine path |
| `2850b59088f4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/scheduler_gpui_unofficial-c792f47be59ca487.d:1,3,5,7,8,9,10` | absolute machine path |
| `17410e87b4c6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/schemars-45de1fa0030cb63d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32` | absolute machine path |
| `6c3369947810` · `target-webdeploy/wasm32-unknown-unknown/web/deps/scopeguard-620383605d5536b2.d:1,3,5,7` | absolute machine path |
| `096098d34867` · `target-webdeploy/wasm32-unknown-unknown/web/deps/seahash-20fa559f4a4c1721.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `5cae3595fbdd` · `target-webdeploy/wasm32-unknown-unknown/web/deps/self_cell-b5e95521fa0e62c1.d:1,3,5,7,8` | absolute machine path |
| `f67e6c618e02` · `target-webdeploy/wasm32-unknown-unknown/web/deps/serde-5b2fbec1f44be650.d:1,3,5,7,8,9,10,11,12,14` | absolute machine path |
| `3988b24d35ab` · `target-webdeploy/wasm32-unknown-unknown/web/deps/serde_core-ff5d10784170d596.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,27` | absolute machine path |
| `69eb29e6b035` · `target-webdeploy/wasm32-unknown-unknown/web/deps/serde_json-7dbef7da475981d9.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23` | absolute machine path |
| `5f94fd297ac5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/serde_urlencoded-222a9e4582623b1f.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `5ed152b6737b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/sha2-e8fb2a681ef22e27.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `41461cef8ebc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/simd_adler32-6e8328c876d5ee51.d:1,3,5,7,8,9,10,11,12,13,14,15,16` | absolute machine path |
| `d1ef4bee1d69` · `target-webdeploy/wasm32-unknown-unknown/web/deps/simplecss-d84bdb51be40f843.d:1,3,5,7,8,9` | absolute machine path |
| `a0be16c9289a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/siphasher-abea6e109e7fe029.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `ccc78ac13490` · `target-webdeploy/wasm32-unknown-unknown/web/deps/skrifa-4d7e1edd58cb1bc7.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85` | absolute machine path |
| `d9beb97b1367` · `target-webdeploy/wasm32-unknown-unknown/web/deps/skrifa-60b8304193cf70c9.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87` | absolute machine path |
| `cb53eab5bb51` · `target-webdeploy/wasm32-unknown-unknown/web/deps/slab-0e8f67bfda3a8b52.d:1,3,5,7,8` | absolute machine path |
| `c497d389784b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/slotmap-2efa131d275a99bb.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `71892c405886` · `target-webdeploy/wasm32-unknown-unknown/web/deps/smallvec-7f9c67a493f3cf7e.d:1,3,5,7` | absolute machine path |
| `c4f6199c544a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/smol_str-332b5043805fb49c.d:1,3,5,7,8,9` | absolute machine path |
| `1e9c9ff16ba4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/spin-8dd9c9ee29c686a3.d:1,3,5,7,8,9,10` | absolute machine path |
| `82681cd0beed` · `target-webdeploy/wasm32-unknown-unknown/web/deps/spin-989f6c4f5a7e152c.d:1,3,5,7,8,9,10,11,12,13,14` | absolute machine path |
| `8b9a2c22e1c0` · `target-webdeploy/wasm32-unknown-unknown/web/deps/stable_deref_trait-660c9f4b32bcd903.d:1,3,5,7` | absolute machine path |
| `37028a7f8182` · `target-webdeploy/wasm32-unknown-unknown/web/deps/static_assertions-e3adf4772183ad05.d:1,3,5,7,8,9,10,11,12,13,14,15,16` | absolute machine path |
| `03195a28e441` · `target-webdeploy/wasm32-unknown-unknown/web/deps/strict_num-caccc63f04300133.d:1,3,5,7` | absolute machine path |
| `3ace859b4fb3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/strum-7dcf42c15ea6101e.d:1,3,5,7,8` | absolute machine path |
| `4ba5fea23dac` · `target-webdeploy/wasm32-unknown-unknown/web/deps/sum_tree_gpui_unofficial-de64db781cc3f71c.d:1,3,5,7,8,9` | absolute machine path |
| `e536fd77d178` · `target-webdeploy/wasm32-unknown-unknown/web/deps/svg_fmt-b81f9a2bc11a0af0.d:1,3,5,7,8,9` | absolute machine path |
| `36d7d0550072` · `target-webdeploy/wasm32-unknown-unknown/web/deps/svgtypes-abb48c9ca42ce4fb.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27` | absolute machine path |
| `ebad257b7f12` · `target-webdeploy/wasm32-unknown-unknown/web/deps/swash-5dbb02fc3075bbc3.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67` | absolute machine path |
| `b5e9bb9e0cfe` · `target-webdeploy/wasm32-unknown-unknown/web/deps/sys_locale-811cdc35daf0d4a4.d:1,3,5,7` | absolute machine path |
| `2b7b76f69500` · `target-webdeploy/wasm32-unknown-unknown/web/deps/taffy-661d1a1b65274898.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54` | absolute machine path |
| `28097eebc9dc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/thiserror-f7b42c3dfd36eb62.d:1,3,5,7,8,9,10,11,12,14` | absolute machine path |
| `bd669255145b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/tiff-3eff5afc36e81231.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26` | absolute machine path |
| `2be9e2c4e2b5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/tiny_skia-eb82e62852cf162d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50` | absolute machine path |
| `4e4c6a68a3bc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/tiny_skia_path-c485c5e9997d1fca.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `5c0856829045` · `target-webdeploy/wasm32-unknown-unknown/web/deps/tinystr-0c616a8f6617c05f.d:1,3,5,7,8,9,10,11,12,13,14` | absolute machine path |
| `cdc3813b1d7a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/tinyvec-ef3e589fa0c738f8.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `c77c1e99e52d` · `target-webdeploy/wasm32-unknown-unknown/web/deps/tracing-702315dd849ad322.d:1,3,5,7,8,9,10,11,12,13,14` | absolute machine path |
| `baf29d4b8693` · `target-webdeploy/wasm32-unknown-unknown/web/deps/tracing_core-d75813d2a3553c6f.d:1,3,5,7,8,9,10,11,12,13,14,15,16` | absolute machine path |
| `dfcc34737169` · `target-webdeploy/wasm32-unknown-unknown/web/deps/ttf_parser-feeb9c904f8e481f.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72` | absolute machine path |
| `d082e641dd10` · `target-webdeploy/wasm32-unknown-unknown/web/deps/typenum-9239e4b41a691e69.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `f9644f10f24c` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_bidi-a483b0bb212b6ed8.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17` | absolute machine path |
| `c73f5918e6af` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_bidi_mirroring-99194fddb7d8aa8a.d:1,3,5,7` | absolute machine path |
| `2c55dc3805e6` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_ccc-441f0b204fad2838.d:1,3,5,7` | absolute machine path |
| `2d7e0780f7c8` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_ident-0c9875476da5e782.d:1,3,5,7,8` | absolute machine path |
| `f32834dc5633` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_linebreak-765dd9a34a92fff2.d:1,3,5,7,8,9` | absolute machine path |
| `4d207c49317a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_properties-b24c242ae0fa020d.d:1,3,5,7,8` | absolute machine path |
| `cd2b0172440e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_script-9553be170227da46.d:1,3,5,7,8` | absolute machine path |
| `8a92aa38c492` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_segmentation-a5c098667e98782e.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `a4f5ded6eb7e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_vo-a6b4215d7084a06d.d:1,3,5,7` | absolute machine path |
| `31a538a60100` · `target-webdeploy/wasm32-unknown-unknown/web/deps/unicode_width-39815f21c027b8c0.d:1,3,5,7,8` | absolute machine path |
| `dc14d2b7809a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/url-8dcb9a89f6ee93c4.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `9fb951cc48bc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/usvg-934f987afeb09662.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35` | absolute machine path |
| `21a357164be7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/utf8_iter-21d76da726885b21.d:1,3,5,7,8,9` | absolute machine path |
| `060a2bf4a114` · `target-webdeploy/wasm32-unknown-unknown/web/deps/uuid-150872a78aed0f8d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18` | absolute machine path |
| `8af9d5799589` · `target-webdeploy/wasm32-unknown-unknown/web/deps/voxelite_app.d:1,3,5` | absolute machine path |
| `03c5df36a387` · `target-webdeploy/wasm32-unknown-unknown/web/deps/voxelite_app.wasm:129011` | email |
| `03c5df36a387` · `target-webdeploy/wasm32-unknown-unknown/web/deps/voxelite_app.wasm:70745` | absolute machine path |
| `03c5df36a387` · `target-webdeploy/wasm32-unknown-unknown/web/deps/voxelite_app.wasm:70701,70753,70875,70898,72012` | Git identity name/username |
| `0f63b3f40b71` · `target-webdeploy/wasm32-unknown-unknown/web/deps/voxelite_core-387de1f7042374a7.d:1,3,5` | absolute machine path |
| `4d4389f265b9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/voxelite_export-3e8142faad86f79a.d:1,3,5` | absolute machine path |
| `5751aef40661` · `target-webdeploy/wasm32-unknown-unknown/web/deps/voxelite_format-faeccfc19e3fb839.d:1,3,5` | absolute machine path |
| `31f3d9c960a3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/voxelite_render-3d6572dd7d1e40ca.d:1,3,5` | absolute machine path |
| `4a65864b0be9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/waker_fn-cb27adc71c5bdf85.d:1,3,5,7` | absolute machine path |
| `013c911865f7` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wasm_bindgen-5cac80edcdf2bb6a.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24` | absolute machine path |
| `ada1913dcaaa` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wasm_bindgen_futures-da3ed5422fe3e0de.d:1,3,5,7` | absolute machine path |
| `5086371c0690` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wasm_bindgen_shared-76ddab7b8a31480b.d:1,3,5,7,8,9` | absolute machine path |
| `b0970772d9bc` · `target-webdeploy/wasm32-unknown-unknown/web/deps/web_sys-051ed307d996014c.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123` | absolute machine path |
| `6e805459739a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/web_time-36cfabbe70dec486.d:1,3,5,7,8,9,10,11,12` | absolute machine path |
| `d0e17c56c9d4` · `target-webdeploy/wasm32-unknown-unknown/web/deps/weezl-7092b48b4db555a4.d:1,3,5,7,8,9,10` | absolute machine path |
| `65984a4eb40a` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wgpu-905c5cef53916ed4.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127,128,129,130,131,132,133,134,135,136,137,138,139,140,141,142,143,144,145,146,147,148,149,150,151,152,153,154,155,156,157,158,159,160,161,162,163,164,165,166,167,168,169,170,171,172,173,174,175,176,177,178,179,180,181,182,183,184,185` | absolute machine path |
| `6e26ff5de1ba` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wgpu_core-1e635a8c1ad1d4c2.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81` | absolute machine path |
| `a458728dc4bb` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wgpu_core_deps_wasm-493cbd2a00c6013d.d:1,3,5,7` | absolute machine path |
| `764cb041b5c5` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wgpu_hal-d063d53d1190bf35.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30` | absolute machine path |
| `747d75274724` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wgpu_naga_bridge-4ae6a95c9ff6fdcb.d:1,3,5,7` | absolute machine path |
| `badea1a13375` · `target-webdeploy/wasm32-unknown-unknown/web/deps/wgpu_types-1a876d57c819325d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37` | absolute machine path |
| `93a25afaefb9` · `target-webdeploy/wasm32-unknown-unknown/web/deps/writeable-28733e6ab8792f7f.d:1,3,5,7,8,9,10,11,12,13,14` | absolute machine path |
| `d4d09fb68748` · `target-webdeploy/wasm32-unknown-unknown/web/deps/xmlwriter-e2485812164083e6.d:1,3,5,7` | absolute machine path |
| `93f243dcdc72` · `target-webdeploy/wasm32-unknown-unknown/web/deps/yazi-f1c7ff53562006a6.d:1,3,5,7,8,9` | absolute machine path |
| `44507d1d76f1` · `target-webdeploy/wasm32-unknown-unknown/web/deps/yoke-c6832570f3254543.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `c4a2467d846e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zeno-8a19d53d1525b3dd.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20` | absolute machine path |
| `b2c8b149a3e3` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zerocopy-96d60c44bc5b5380.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25` | absolute machine path |
| `27b1f2477c82` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zerofrom-62f7362069a4c5ee.d:1,3,5,7,8,9` | absolute machine path |
| `50205ead794e` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zerotrie-31a94bc6aaa25a55.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21` | absolute machine path |
| `71c79ee37c12` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zerovec-657954b246df0b13.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30` | absolute machine path |
| `1ec47d371a23` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zmij-f3342f52a38aada8.d:1,3,5,7,8` | absolute machine path |
| `978f2db6b169` · `target-webdeploy/wasm32-unknown-unknown/web/deps/ztracing_gpui_unofficial-501669480131ceac.d:1,3,5,7` | absolute machine path |
| `466abab4e224` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zune_core-bf5944db8230fce5.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24` | absolute machine path |
| `9398a315301b` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zune_inflate-312e139591b2c4ec.d:1,3,5,7,8,9,10,11,12,13,14,15,16` | absolute machine path |
| `d12f3002117f` · `target-webdeploy/wasm32-unknown-unknown/web/deps/zune_jpeg-c488f84a10839a23.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31` | absolute machine path |
| `3987814ca17a` · `target-webdeploy/wasm32-unknown-unknown/web/libvoxelite_app.d:1` | absolute machine path |
| `4a59eeba51cb` · `target-webdeploy/wasm32-unknown-unknown/web/voxelite_app.d:1` | absolute machine path |
| `93d0bbdac8ad` · `target-webdeploy/web/build/anyhow-c133e46889c468bb/build-script-build:173` | absolute machine path |
| `c8bbdcc7c5eb` · `target-webdeploy/web/build/anyhow-c133e46889c468bb/build_script_build-c133e46889c468bb.d:1,3,5` | absolute machine path |
| `65a94384fd30` · `target-webdeploy/web/build/crc32fast-465b5d165116b989/build_script_build-465b5d165116b989.d:1,3,5` | absolute machine path |
| `abb1d9414453` · `target-webdeploy/web/build/crossbeam-deque-e1955f67cf9b20db/build-script-build:147` | absolute machine path |
| `39a022f0b80c` · `target-webdeploy/web/build/crossbeam-deque-e1955f67cf9b20db/build_script_build-e1955f67cf9b20db.d:1,3,5` | absolute machine path |
| `c24df9972edb` · `target-webdeploy/web/build/crossbeam-epoch-dd3bd9e18c79c031/build-script-build:154` | absolute machine path |
| `02cababb990b` · `target-webdeploy/web/build/crossbeam-epoch-dd3bd9e18c79c031/build_script_build-dd3bd9e18c79c031.d:1,3,5` | absolute machine path |
| `c93d702af0bd` · `target-webdeploy/web/build/crossbeam-utils-d60d63eb60bf7c48/build-script-build:19,26` | absolute machine path |
| `3ce340db83ce` · `target-webdeploy/web/build/crossbeam-utils-d60d63eb60bf7c48/build_script_build-d60d63eb60bf7c48.d:1,3,5,6,7` | absolute machine path |
| `4513bc761635` · `target-webdeploy/web/build/generic-array-5cc2767dc22e2756/build-script-build:22,27` | absolute machine path |
| `c796047ae98e` · `target-webdeploy/web/build/generic-array-5cc2767dc22e2756/build_script_build-5cc2767dc22e2756.d:1,3,5` | absolute machine path |
| `6b7b2b3cec12` · `target-webdeploy/web/build/getrandom-06d4315af6de370c/build-script-build:143` | absolute machine path |
| `70e191235b56` · `target-webdeploy/web/build/getrandom-06d4315af6de370c/build_script_build-06d4315af6de370c.d:1,3,5` | absolute machine path |
| `5dc56939b0e0` · `target-webdeploy/web/build/gpui-unofficial-ea297e89192bea1c/build-script-build:672,673,675,676,678,681,682,683,685,686,687,688,694,695,703,704,706,707,711` | absolute machine path |
| `bb395af9d240` · `target-webdeploy/web/build/gpui-unofficial-ea297e89192bea1c/build_script_build-ea297e89192bea1c.d:1,3,5` | absolute machine path |
| `f810c2200678` · `target-webdeploy/web/build/heapless-817c13f742418f32/build-script-build:149` | absolute machine path |
| `84c5d78b291f` · `target-webdeploy/web/build/heapless-817c13f742418f32/build_script_build-817c13f742418f32.d:1,3,5` | absolute machine path |
| `0a835851bfc9` · `target-webdeploy/web/build/icu_normalizer_data-5dbf9304a30d2d52/build_script_build-5dbf9304a30d2d52.d:1,3,5` | absolute machine path |
| `2461e6d8eecf` · `target-webdeploy/web/build/icu_properties_data-a5233eab4ebb730b/build_script_build-a5233eab4ebb730b.d:1,3,5` | absolute machine path |
| `2f5ce6855d10` · `target-webdeploy/web/build/libc-cda79ff38737910e/build-script-build:172` | absolute machine path |
| `3861fd4aca8a` · `target-webdeploy/web/build/libc-cda79ff38737910e/build_script_build-cda79ff38737910e.d:1,3,5` | absolute machine path |
| `2a6b0359b978` · `target-webdeploy/web/build/libm-0eaa3a21ce902dcb/build-script-build:169` | absolute machine path |
| `1bb510961f15` · `target-webdeploy/web/build/libm-0eaa3a21ce902dcb/build_script_build-0eaa3a21ce902dcb.d:1,3,5,6` | absolute machine path |
| `f85155c82a98` · `target-webdeploy/web/build/naga-c0b66ea5a5b57210/build_script_build-c0b66ea5a5b57210.d:1,3,5` | absolute machine path |
| `6291f4029489` · `target-webdeploy/web/build/num-traits-278da1b462e148db/build-script-build:147,162,165` | absolute machine path |
| `79c8f33293b3` · `target-webdeploy/web/build/num-traits-278da1b462e148db/build_script_build-278da1b462e148db.d:1,3,5` | absolute machine path |
| `633c60d08954` · `target-webdeploy/web/build/object-ab5233b02ebacb66/build-script-build:150` | absolute machine path |
| `ef53cbc5dc44` · `target-webdeploy/web/build/object-ab5233b02ebacb66/build_script_build-ab5233b02ebacb66.d:1,3,5` | absolute machine path |
| `94ce5530cbe4` · `target-webdeploy/web/build/parking_lot_core-a973fbb4f88943de/build_script_build-a973fbb4f88943de.d:1,3,5` | absolute machine path |
| `235a60ef4b2a` · `target-webdeploy/web/build/paste-3fe37c2f0dbe8869/build_script_build-3fe37c2f0dbe8869.d:1,3,5` | absolute machine path |
| `cad5a2429782` · `target-webdeploy/web/build/paste-d3c79a5c55cd3d81/root-output:1` | absolute machine path |
| `d0b1e50afb21` · `target-webdeploy/web/build/proc-macro2-68f51d9242c7a0fa/root-output:1` | absolute machine path |
| `1ac793a01639` · `target-webdeploy/web/build/proc-macro2-ec1fa8d2755b1122/build-script-build:180` | absolute machine path |
| `4e86348283b5` · `target-webdeploy/web/build/proc-macro2-ec1fa8d2755b1122/build_script_build-ec1fa8d2755b1122.d:1,3,5` | absolute machine path |
| `f008e13f8380` · `target-webdeploy/web/build/pulp-91676ad3ed4d4901/build-script-build:175,187,198` | absolute machine path |
| `6d312ef1684a` · `target-webdeploy/web/build/pulp-91676ad3ed4d4901/build_script_build-91676ad3ed4d4901.d:1,3,5` | absolute machine path |
| `27845ac67b1a` · `target-webdeploy/web/build/quote-2165c0071fef8d8b/build_script_build-2165c0071fef8d8b.d:1,3,5` | absolute machine path |
| `f5f96172c332` · `target-webdeploy/web/build/quote-86148c8a4d375b21/root-output:1` | absolute machine path |
| `6dc0cc0ba71f` · `target-webdeploy/web/build/rayon-core-8fac0e817c725ea3/build_script_build-8fac0e817c725ea3.d:1,3,5` | absolute machine path |
| `48d8fac05d62` · `target-webdeploy/web/build/ref-cast-94fb125c981276dc/build-script-build:164` | absolute machine path |
| `45fda6f018fa` · `target-webdeploy/web/build/ref-cast-94fb125c981276dc/build_script_build-94fb125c981276dc.d:1,3,5` | absolute machine path |
| `3cabc57ce0e2` · `target-webdeploy/web/build/rustversion-2c0c5f4841a48194/build-script-build:151,160` | absolute machine path |
| `40287655f926` · `target-webdeploy/web/build/rustversion-2c0c5f4841a48194/build_script_build-2c0c5f4841a48194.d:1,3,5,6` | absolute machine path |
| `dea05511935a` · `target-webdeploy/web/build/rustversion-91bbd7863d8cbf05/root-output:1` | absolute machine path |
| `18894159630f` · `target-webdeploy/web/build/serde-0fd8d349c9f7424d/build-script-build:162` | absolute machine path |
| `e108211b2e91` · `target-webdeploy/web/build/serde-0fd8d349c9f7424d/build_script_build-0fd8d349c9f7424d.d:1,3,5` | absolute machine path |
| `e5d28535d788` · `target-webdeploy/web/build/serde_core-0ce1ae5685907101/root-output:1` | absolute machine path |
| `469cd0fd0182` · `target-webdeploy/web/build/serde_core-64e409c200bdf234/build-script-build:172` | absolute machine path |
| `3153a75eb6d7` · `target-webdeploy/web/build/serde_core-64e409c200bdf234/build_script_build-64e409c200bdf234.d:1,3,5` | absolute machine path |
| `fac3d5068cb4` · `target-webdeploy/web/build/serde_core-b715ca05d6ce3cff/build-script-build:172` | absolute machine path |
| `2b5a27e064e6` · `target-webdeploy/web/build/serde_core-b715ca05d6ce3cff/build_script_build-b715ca05d6ce3cff.d:1,3,5` | absolute machine path |
| `951344ef2d44` · `target-webdeploy/web/build/serde_json-6f29c0fb42ec24bf/build-script-build:147` | absolute machine path |
| `061f3b57c311` · `target-webdeploy/web/build/serde_json-6f29c0fb42ec24bf/build_script_build-6f29c0fb42ec24bf.d:1,3,5` | absolute machine path |
| `74d11093f8d5` · `target-webdeploy/web/build/slotmap-806ed9f714718bfc/build-script-build:29,34` | absolute machine path |
| `7e98a6d74854` · `target-webdeploy/web/build/slotmap-806ed9f714718bfc/build_script_build-806ed9f714718bfc.d:1,3,5` | absolute machine path |
| `b33cf43bfd20` · `target-webdeploy/web/build/thiserror-5c9142bad2887cbd/build-script-build:174` | absolute machine path |
| `aa706a4c9e5f` · `target-webdeploy/web/build/thiserror-5c9142bad2887cbd/build_script_build-5c9142bad2887cbd.d:1,3,5` | absolute machine path |
| `e73135bc68a3` · `target-webdeploy/web/build/wasm-bindgen-046a11438fea3c92/build-script-build:142` | absolute machine path |
| `b37a34a41e74` · `target-webdeploy/web/build/wasm-bindgen-046a11438fea3c92/build_script_build-046a11438fea3c92.d:1,3,5` | absolute machine path |
| `6f39acfca4e5` · `target-webdeploy/web/build/wasm-bindgen-shared-3218bb89d2ebd00a/root-output:1` | absolute machine path |
| `dfde2b6dbff7` · `target-webdeploy/web/build/wasm-bindgen-shared-f2e9293ba4062c07/build-script-build:175` | absolute machine path |
| `fd2e84f0b3c3` · `target-webdeploy/web/build/wasm-bindgen-shared-f2e9293ba4062c07/build_script_build-f2e9293ba4062c07.d:1,3,5` | absolute machine path |
| `80e61fd95b25` · `target-webdeploy/web/build/wgpu-2a9c17e6752284b2/build_script_build-2a9c17e6752284b2.d:1,3,5` | absolute machine path |
| `d6320485e9c9` · `target-webdeploy/web/build/wgpu-core-cefc93153fcc96c3/build_script_build-cefc93153fcc96c3.d:1,3,5` | absolute machine path |
| `147e5647752b` · `target-webdeploy/web/build/wgpu-hal-11368e886ee55832/build_script_build-11368e886ee55832.d:1,3,5` | absolute machine path |
| `4ce4cbcf89c7` · `target-webdeploy/web/build/zerocopy-fc0886eb7471bb11/build-script-build:175` | absolute machine path |
| `ac21607621d1` · `target-webdeploy/web/build/zerocopy-fc0886eb7471bb11/build_script_build-fc0886eb7471bb11.d:1,3,5` | absolute machine path |
| `5436431e4486` · `target-webdeploy/web/build/zmij-a75537cd80fe769f/build-script-build:171` | absolute machine path |
| `153a15dff6c2` · `target-webdeploy/web/build/zmij-a75537cd80fe769f/build_script_build-a75537cd80fe769f.d:1,3,5` | absolute machine path |
| `5ea3452af05e` · `target-webdeploy/web/build/ztracing-gpui-unofficial-094d7b26a357e0e5/build_script_build-094d7b26a357e0e5.d:1,3,5` | absolute machine path |
| `7043c4815edc` · `target-webdeploy/web/deps/autocfg-a37330238752ec3b.d:1,3,5,7,8,9,10` | absolute machine path |
| `e7d16c2f1830` · `target-webdeploy/web/deps/bumpalo-cb071fbb24974cb2.d:1,3,5,7,8,9` | absolute machine path |
| `33f297f75d94` · `target-webdeploy/web/deps/bytemuck_derive-a2909b09ba6745c6.d:1,3,5,6` | absolute machine path |
| `d8c4cfdfccb4` · `target-webdeploy/web/deps/cc-7f59e279e5c2dfda.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21` | absolute machine path |
| `239cb500ee6a` · `target-webdeploy/web/deps/cfg_aliases-8bca756f5a0ff1d4.d:1,3,5,7` | absolute machine path |
| `1f698c4600dd` · `target-webdeploy/web/deps/convert_case-62b333b2f99549c8.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `cf69f9522a56` · `target-webdeploy/web/deps/derive_more_impl-68bc53b31508a79d.d:1,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24` | absolute machine path |
| `7605f758e171` · `target-webdeploy/web/deps/derive_refineable_gpui_unofficial-242cc09f15431da3.d:1,3,5` | absolute machine path |
| `22fc6905ff43` · `target-webdeploy/web/deps/displaydoc-10e4bca1867b9bcc.d:1,3,5,6,7,8` | absolute machine path |
| `aa9ee04a5274` · `target-webdeploy/web/deps/document_features-e48cc9f5f0863f98.d:1,3,5` | absolute machine path |
| `d2c24e3abc45` · `target-webdeploy/web/deps/embed_resource-b6aff67b0f102dc0.d:1,3,5,7,8` | absolute machine path |
| `cd6fa36c13bc` · `target-webdeploy/web/deps/enumn-ab2629ada41ddabb.d:1,3,5` | absolute machine path |
| `f088dd553b3c` · `target-webdeploy/web/deps/find_msvc_tools-d4d852696ef06732.d:1,3,5,7,8,9` | absolute machine path |
| `9b463b368f57` · `target-webdeploy/web/deps/futures_macro-43f4327c69348bca.d:1,3,5,6,7,8,9` | absolute machine path |
| `d442f5233c86` · `target-webdeploy/web/deps/gpui_macros_gpui_unofficial-393de4c3c6f1e46f.d:1,3,5,6,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `b83f2538401c` · `target-webdeploy/web/deps/heck-819adfa2f80b239e.d:1,3,5,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `74c0aba9e287` · `target-webdeploy/web/deps/libautocfg-a37330238752ec3b.rlib:360,377,379,385,401,455,620` | absolute machine path |
| `6578f80eb985` · `target-webdeploy/web/deps/libautocfg-a37330238752ec3b.rmeta:352,369,371,377` | absolute machine path |
| `027a73248993` · `target-webdeploy/web/deps/libbumpalo-cb071fbb24974cb2.rlib:887,966,983,995` | absolute machine path |
| `5c2452683898` · `target-webdeploy/web/deps/libbumpalo-cb071fbb24974cb2.rmeta:882,961,978` | absolute machine path |
| `fc92c148a5b3` · `target-webdeploy/web/deps/libbytemuck_derive-a2909b09ba6745c6.so:832,835,836,839,840,842,845,5223,5254` | absolute machine path |
| `a728695c7b43` · `target-webdeploy/web/deps/libcc-7f59e279e5c2dfda.rlib:1540,1725,1729,1757,1781,1783,1789,1817,1832,1845,1848,1851,2268,2784,2785,2888,3209,3280,3391,3478` | absolute machine path |
| `d315be224a3d` · `target-webdeploy/web/deps/libcc-7f59e279e5c2dfda.rmeta:1518,1703,1707,1735,1759,1761,1767,1795,1810,1823,1826,1829` | absolute machine path |
| `d44287ddb8e2` · `target-webdeploy/web/deps/libcfg_aliases-8bca756f5a0ff1d4.rlib:18` | absolute machine path |
| `5b10a7abf1be` · `target-webdeploy/web/deps/libcfg_aliases-8bca756f5a0ff1d4.rmeta:13` | absolute machine path |
| `93103f8a9d55` · `target-webdeploy/web/deps/libconvert_case-62b333b2f99549c8.rlib:375,385,389,392,415,441,586` | absolute machine path |
| `ed3ca66912d6` · `target-webdeploy/web/deps/libconvert_case-62b333b2f99549c8.rmeta:369,379,383,386,409` | absolute machine path |
| `98896d901934` · `target-webdeploy/web/deps/libderive_more_impl-68bc53b31508a79d.so:1125,1126,1128,1130,1134,1135,1136,1139,1140,1142,1145,8650,8774,8780,8797,8800,8803,8813,8819,8827,8828,8832,8834,8836,8838,8840,8851,8858` | absolute machine path |
| `40ba3cec6948` · `target-webdeploy/web/deps/libderive_refineable_gpui_unofficial-242cc09f15431da3.so:983,984,986,988,991,992,996,997,999,1002,5767` | absolute machine path |
| `1291c4fbc66a` · `target-webdeploy/web/deps/libdisplaydoc-10e4bca1867b9bcc.so:703,706,707,710,711,713,716,4680,4689,4694` | absolute machine path |
| `2ee5a1be88e4` · `target-webdeploy/web/deps/libdocument_features-e48cc9f5f0863f98.so:242,246,251,255,256,258,262,1070` | absolute machine path |
| `8423b87d7a9d` · `target-webdeploy/web/deps/libembed_resource-b6aff67b0f102dc0.rlib:335,351,373` | absolute machine path |
| `6a006f03b730` · `target-webdeploy/web/deps/libembed_resource-b6aff67b0f102dc0.rmeta:330,346` | absolute machine path |
| `03235949dfc1` · `target-webdeploy/web/deps/libenumn-ab2629ada41ddabb.so:660,661,663,665,668,669,672,673,675,678,4595` | absolute machine path |
| `75ce0ac684ab` · `target-webdeploy/web/deps/libfind_msvc_tools-d4d852696ef06732.rlib:146,190,191,250` | absolute machine path |
| `128f9f0b5035` · `target-webdeploy/web/deps/libfind_msvc_tools-d4d852696ef06732.rmeta:141,185,186` | absolute machine path |
| `7fdf04aa874c` · `target-webdeploy/web/deps/libfutures_macro-43f4327c69348bca.so:757,760,761,764,765,767,770,5505` | absolute machine path |
| `1c0686802eda` · `target-webdeploy/web/deps/libgpui_macros_gpui_unofficial-393de4c3c6f1e46f.so:1017,1018,1020,1022,1025,1026,1028,1031,1032,1034,1037,6321` | absolute machine path |
| `58b96fa9a53b` · `target-webdeploy/web/deps/libheck-819adfa2f80b239e.rlib:225,226,227,266` | absolute machine path |
| `58e569eee1fc` · `target-webdeploy/web/deps/libheck-819adfa2f80b239e.rmeta:220,221,222` | absolute machine path |
| `852a28273975` · `target-webdeploy/web/deps/liblinktime_proc_macro-7db3c545d6ff6780.so:311,313,316,325,1119,1120` | absolute machine path |
| `ca5b10c4abb9` · `target-webdeploy/web/deps/liblitrs-178e78623daadbd5.rlib:1518,1527,1531,1535,1539,1542,1545,1554,1565,1575,1578,1581,1585,1636,1713,1770,1833` | absolute machine path |
| `47845dc12d58` · `target-webdeploy/web/deps/liblitrs-178e78623daadbd5.rmeta:1510,1519,1523,1527,1531,1534,1537,1546,1557,1567,1570,1573,1577` | absolute machine path |
| `60fc15af7034` · `target-webdeploy/web/deps/libmemchr-8c4a70787283027b.rlib:4670,4713,4722,4735,4739,4762,4766,4812,4822,4864,4873,4883,4930,4955,4967,5010,5046,5052,5058,5060,5061,5113,5269,5484` | absolute machine path |
| `2b655b510a0c` · `target-webdeploy/web/deps/libmemchr-8c4a70787283027b.rmeta:4663,4706,4715,4728,4732,4755,4759,4805,4815,4857,4866,4876,4923,4948,4960,5003,5039,5045,5051,5053,5054` | absolute machine path |
| `e3015843642b` · `target-webdeploy/web/deps/libpaste-7ec7dd41f1956530.so:284,290,870` | absolute machine path |
| `518fc895ee05` · `target-webdeploy/web/deps/libpin_project_internal-f8eb4520b8a0f8c1.so:755,756,758,760,763,764,765,766,769,770,772,776,5858` | absolute machine path |
| `4e15bc492657` · `target-webdeploy/web/deps/libproc_macro2-dae466aba027a5dd.rlib:1022,1023,1074,1079,1112,1185,1218,1226,1266,1394,1587,1642` | absolute machine path |
| `1a066a9159bf` · `target-webdeploy/web/deps/libproc_macro2-dae466aba027a5dd.rmeta:1013,1014,1065,1070,1103,1176,1209,1217` | absolute machine path |
| `4ed3306b9433` · `target-webdeploy/web/deps/libprofiling_procmacros-70cf941ed954bfd0.so:1038,1039,1041,1043,1046,1047,1050,1051,1053,1056,4776` | absolute machine path |
| `816af307d520` · `target-webdeploy/web/deps/libquote-0b8b9fbcbb9ecc80.rlib:450,466,467,473,497,511,526` | absolute machine path |
| `bab03ae87041` · `target-webdeploy/web/deps/libquote-0b8b9fbcbb9ecc80.rmeta:445,461,462,468,492,506` | absolute machine path |
| `d33ff575d1dd` · `target-webdeploy/web/deps/libref_cast_impl-d169f6b0101a4cc7.so:1095,1098,1099,1102,1103,1105,1108,5467` | absolute machine path |
| `49c07be24ea4` · `target-webdeploy/web/deps/librustc_version-3a9b1ec13ac738c1.rlib:263,395` | absolute machine path |
| `089d3ddb8f54` · `target-webdeploy/web/deps/librustc_version-3a9b1ec13ac738c1.rmeta:257` | absolute machine path |
| `a270fb069b36` · `target-webdeploy/web/deps/librustversion-25a1e7662977fed7.so:367,374,375,378,384,955` | absolute machine path |
| `dedfff5452a2` · `target-webdeploy/web/deps/libschemars_derive-e6bd5fb6a5a986b5.so:1036,1039,1041,1044,1045,1048,1049,1051,1054,6279,6287,6297,6307,6308` | absolute machine path |
| `750436d46575` · `target-webdeploy/web/deps/libsemver-4b30861a3f6738a7.rlib:294,301,303,310,318,338,344,390` | absolute machine path |
| `bd4d8e914be7` · `target-webdeploy/web/deps/libsemver-4b30861a3f6738a7.rmeta:289,296,298,305,313,333,339` | absolute machine path |
| `7ae9e208098f` · `target-webdeploy/web/deps/libserde_core-3d7696788ebb1453.rlib:10461` | email |
| `7ae9e208098f` · `target-webdeploy/web/deps/libserde_core-3d7696788ebb1453.rlib:11160,11163,11270,11423,11575,11581,11599,11784,11789,11855,11864,11866,11870,11872,11874,11875,11876,11877,11892` | absolute machine path |
| `c6fb9aba2fa1` · `target-webdeploy/web/deps/libserde_core-3d7696788ebb1453.rmeta:10456` | email |
| `c6fb9aba2fa1` · `target-webdeploy/web/deps/libserde_core-3d7696788ebb1453.rmeta:11155,11158,11265,11418,11570,11576,11594,11779,11784,11850,11859,11861,11865,11867,11869,11870,11871,11872` | absolute machine path |
| `31fdd748f72e` · `target-webdeploy/web/deps/libserde_derive-872be7438da10bf2.so:1198,1201,1203,1206,1207,1210,1211,1213,1216,6289,6304,6319,6360` | absolute machine path |
| `0cb34569eb73` · `target-webdeploy/web/deps/libserde_derive_internals-2268b7449fd916e4.rlib:834,838,900,904,914,917,919,936,939,955,1184,1317,1393,1450,1510,1571,1644,1692` | absolute machine path |
| `384bd9814804` · `target-webdeploy/web/deps/libserde_derive_internals-2268b7449fd916e4.rmeta:821,825,887,891,901,904,906,923,926,942` | absolute machine path |
| `52af90661827` · `target-webdeploy/web/deps/libserde_spanned-cd7afb22bcd0c9ba.rlib:144,145,152` | absolute machine path |
| `25bed7d6873e` · `target-webdeploy/web/deps/libserde_spanned-cd7afb22bcd0c9ba.rmeta:138,139,146` | absolute machine path |
| `d0567e36b453` · `target-webdeploy/web/deps/libshlex-bf8bee87fecfaea1.rlib:156,159,184` | absolute machine path |
| `758a65f10ce5` · `target-webdeploy/web/deps/libshlex-bf8bee87fecfaea1.rmeta:151,154` | absolute machine path |
| `56c288716d37` · `target-webdeploy/web/deps/libstrum_macros-bb03b173acf251e2.so:1229,1230,1234,1235,1237,1240,1241,1244,1246,1247,1249,1252,6770,6772` | absolute machine path |
| `90fe2d544c86` · `target-webdeploy/web/deps/libsyn-9e21a8eabb853136.rlib:28140,28147,28157,28168,28184,28199,28359,28403,28459,28494,28553,28751,28950,28963,28989,28990,28995,29010,29018,29110,29118,29128,29138,29146,29198,29202,29210,29211,29214,29218,29424,29445,29449,29557,29561,29596,29601,29616,29672,29676,29680,29689,29693,29699,29704,29705,29706,29708,29879,29911,30277,30651,30918,31071,31208,31343,31408,31675,31906,32491,32799,32800,32980,33150,33601,33721` | absolute machine path |
| `066da99edb8b` · `target-webdeploy/web/deps/libsyn-9e21a8eabb853136.rmeta:28120,28127,28137,28148,28164,28179,28339,28383,28439,28474,28533,28731,28930,28943,28969,28970,28975,28990,28998,29090,29098,29108,29118,29126,29178,29182,29190,29191,29194,29198,29404,29425,29429,29537,29541,29576,29581,29596,29652,29656,29660,29669,29673,29679,29684,29685,29686,29688,29859,29891` | absolute machine path |
| `ff3513e641da` · `target-webdeploy/web/deps/libsyn-f6c11f27e79961f8.rlib:29526,29531,29541,29552,29569,29587,29746,29790,29825,29882,30009,30211,30413,30426,30452,30453,30458,30473,30481,30570,30577,30587,30598,30606,30658,30662,30670,30672,30730,30733,30737,30956,30979,30983,31102,31106,31144,31149,31164,31223,31227,31231,31240,31244,31250,31255,31256,31258,31436,31469,31771,32191,32421,32568,32777,32778,32896,33113,33272,33568,33964,34137,34490,34656,34943,35157` | absolute machine path |
| `1380203d16e0` · `target-webdeploy/web/deps/libsyn-f6c11f27e79961f8.rmeta:29002,29007,29017,29028,29045,29063,29222,29266,29301,29358,29485,29687,29889,29902,29928,29929,29934,29949,29957,30046,30053,30063,30074,30082,30134,30138,30146,30148,30206,30209,30213,30432,30455,30459,30578,30582,30620,30625,30640,30699,30703,30707,30716,30720,30726,30731,30732,30734,30912,30945` | absolute machine path |
| `88f904520ed1` · `target-webdeploy/web/deps/libsynstructure-f09698470bf6a345.rlib:520,596,683,739,835,1012,1013` | absolute machine path |
| `e11ae4bca9cc` · `target-webdeploy/web/deps/libsynstructure-f09698470bf6a345.rmeta:510,586` | absolute machine path |
| `0ce46b4e3cbd` · `target-webdeploy/web/deps/libthiserror_impl-866b8b5169c4adf1.so:808,810,812,815,816,819,820,822,825,5475` | absolute machine path |
| `103cbe49e0db` · `target-webdeploy/web/deps/libtoml-fb369889cdd0e09d.rlib:3736,3782,3792,3793,3854,3860,3883,3896,3902,3915,3921,3923,3926,3944,3950,3954,3959,3967,3970,3976,3991,3996,4004,4011,4014,4042,4048,4068,4071,4072,4075,4160,4329,4401,4404,4481,4569,4645,4754` | absolute machine path |
| `dcac5ea59bc7` · `target-webdeploy/web/deps/libtoml-fb369889cdd0e09d.rmeta:3724,3770,3780,3781,3842,3848,3871,3884,3890,3903,3909,3911,3914,3932,3938,3942,3947,3955,3958,3964,3979,3984,3992,3999,4002,4030,4036,4056,4059,4060,4063` | absolute machine path |
| `fa7907c43dcb` · `target-webdeploy/web/deps/libtoml_datetime-119d0274b7876f09.rlib:393,434,446,524` | absolute machine path |
| `194ffd3f2167` · `target-webdeploy/web/deps/libtoml_datetime-119d0274b7876f09.rmeta:388,429,441` | absolute machine path |
| `16e81d62bb1a` · `target-webdeploy/web/deps/libtoml_parser-08af305d50c8d42a.rlib:2301,2304,2332,2359,2370,2401,2406,2407,2411,2414,2419,2494,2607,2808,2888` | absolute machine path |
| `78b905ed1b9f` · `target-webdeploy/web/deps/libtoml_parser-08af305d50c8d42a.rmeta:2262,2265,2293,2320,2331,2362,2367,2368,2372,2375,2380` | absolute machine path |
| `991196ffdb39` · `target-webdeploy/web/deps/libtoml_writer-a38a7e86ab9aceee.rlib:449,453,455,476,480,498` | absolute machine path |
| `177a84711929` · `target-webdeploy/web/deps/libtoml_writer-a38a7e86ab9aceee.rmeta:444,448,450,471,475` | absolute machine path |
| `720e20d8157e` · `target-webdeploy/web/deps/libtracing_attributes-6524c0c513ed633c.so:784,785,787,789,792,793,796,797,799,802,5657` | absolute machine path |
| `2b10b3735a33` · `target-webdeploy/web/deps/libunicode_ident-82952c4e5edf73c0.rlib:29,30,35` | absolute machine path |
| `e826684747af` · `target-webdeploy/web/deps/libunicode_ident-82952c4e5edf73c0.rmeta:24,25` | absolute machine path |
| `ed8f849224a3` · `target-webdeploy/web/deps/libunicode_segmentation-52243cfece65f3b5.rlib:7375,7376,7414,7448,7466,7490` | absolute machine path |
| `dff2f81ff2e5` · `target-webdeploy/web/deps/libunicode_segmentation-52243cfece65f3b5.rmeta:7370,7371,7409,7443,7461` | absolute machine path |
| `479489c370c8` · `target-webdeploy/web/deps/libversion_check-bb21ec76f396fc8b.rlib:159,162,165,176,203,317` | absolute machine path |
| `a8968a61ca52` · `target-webdeploy/web/deps/libversion_check-bb21ec76f396fc8b.rmeta:153,156,159,170` | absolute machine path |
| `38c1c047ccfb` · `target-webdeploy/web/deps/libwasm_bindgen_macro-6ec03b942e279ee9.so:3025,3026,3030,3033,3034,3037,3038,3040,3043,9095` | absolute machine path |
| `57b8d147e794` · `target-webdeploy/web/deps/libwasm_bindgen_macro_support-2d1ad74348de5174.rlib:1796,1906,1926,2029,2036,2044,2064,2065,4007,5029,5524,7324,8208,9103,9816` | absolute machine path |
| `d0efb9ce78c1` · `target-webdeploy/web/deps/libwasm_bindgen_macro_support-2d1ad74348de5174.rmeta:1776,1886,1906,2009,2016,2024,2044,2045` | absolute machine path |
| `2f7942bb9097` · `target-webdeploy/web/deps/libwasm_bindgen_shared-05d1a8e8ebaa6b05.rlib:392,421,425` | absolute machine path |
| `726b0ca677f0` · `target-webdeploy/web/deps/libwasm_bindgen_shared-05d1a8e8ebaa6b05.rmeta:386,415,419` | absolute machine path |
| `a52434d1add9` · `target-webdeploy/web/deps/libwinnow-50a29f8afb993434.rlib:2487,2493,2497,2544,2551,2557,2558,2560,2562,2565,2568,2571,2582,2598,2640` | absolute machine path |
| `2adc5510364c` · `target-webdeploy/web/deps/libwinnow-50a29f8afb993434.rmeta:2427,2433,2437,2484,2491,2497,2498,2500,2502,2505,2508,2511,2522,2538` | absolute machine path |
| `a12266af3fea` · `target-webdeploy/web/deps/libyoke_derive-799a5defedd5f39a.so:765,769,772,773,776,777,779,782,783,5743,5745,5750` | absolute machine path |
| `95e779598f14` · `target-webdeploy/web/deps/libzerocopy_derive-7199598f9492a86b.so:1129,1130,1132,1134,1137,1138,1141,1142,1144,1147,5905,5910,5922,5927,5968` | absolute machine path |
| `519ae0bc18f8` · `target-webdeploy/web/deps/libzerofrom_derive-6804d03aa2a25beb.so:847,850,851,854,855,857,860,861,5716,5718` | absolute machine path |
| `a828dd48b1a2` · `target-webdeploy/web/deps/libzerovec_derive-396cd6c74f100cc8.so:802,805,807,810,811,814,815,817,820,5541` | absolute machine path |
| `0172bc8b26c9` · `target-webdeploy/web/deps/libztracing_macro_gpui_unofficial-c43c5e0342d41974.so:626` | absolute machine path |
| `3bcc30818af2` · `target-webdeploy/web/deps/linktime_proc_macro-7db3c545d6ff6780.d:1,3,5,6,7,8,9,10,11,12` | absolute machine path |
| `80e1d6e0de9e` · `target-webdeploy/web/deps/litrs-178e78623daadbd5.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `974ad1057c07` · `target-webdeploy/web/deps/memchr-8c4a70787283027b.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33` | absolute machine path |
| `6fa7f66072b2` · `target-webdeploy/web/deps/paste-7ec7dd41f1956530.d:1,3,5,6,7,8` | absolute machine path |
| `96100cdd61f5` · `target-webdeploy/web/deps/pin_project_internal-f8eb4520b8a0f8c1.d:1,3,5,6,7,8,9,10,11,12` | absolute machine path |
| `b60ca0689c64` · `target-webdeploy/web/deps/proc_macro2-dae466aba027a5dd.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17` | absolute machine path |
| `a3b1ac654b2d` · `target-webdeploy/web/deps/profiling_procmacros-70cf941ed954bfd0.d:1,3,5` | absolute machine path |
| `38ec86804a36` · `target-webdeploy/web/deps/quote-0b8b9fbcbb9ecc80.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `2798fb532f07` · `target-webdeploy/web/deps/ref_cast_impl-d169f6b0101a4cc7.d:1,3,5` | absolute machine path |
| `660c97758c04` · `target-webdeploy/web/deps/rustc_version-3a9b1ec13ac738c1.d:1,3,5,7` | absolute machine path |
| `54d420302af2` · `target-webdeploy/web/deps/rustversion-25a1e7662977fed7.d:1,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,20` | absolute machine path |
| `6f8b4ea53e57` · `target-webdeploy/web/deps/schemars_derive-e6bd5fb6a5a986b5.d:1,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `0384e88efb37` · `target-webdeploy/web/deps/semver-4b30861a3f6738a7.d:1,3,5,7,8,9,10,11,12,13` | absolute machine path |
| `ec528ea386d9` · `target-webdeploy/web/deps/serde_core-3d7696788ebb1453.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,27` | absolute machine path |
| `f2aa8c2e888f` · `target-webdeploy/web/deps/serde_derive-872be7438da10bf2.d:1,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32` | absolute machine path |
| `09e7b1f57108` · `target-webdeploy/web/deps/serde_derive_internals-2268b7449fd916e4.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17` | absolute machine path |
| `b748a759b1b1` · `target-webdeploy/web/deps/serde_spanned-cd7afb22bcd0c9ba.d:1,3,5,7,8,9` | absolute machine path |
| `c016e0c939ca` · `target-webdeploy/web/deps/shlex-bf8bee87fecfaea1.d:1,3,5,7,8` | absolute machine path |
| `fb41d3ee9de7` · `target-webdeploy/web/deps/strum_macros-bb03b173acf251e2.d:1,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28` | absolute machine path |
| `de4fa2edc492` · `target-webdeploy/web/deps/syn-9e21a8eabb853136.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59` | absolute machine path |
| `96b9f3ed2099` · `target-webdeploy/web/deps/syn-f6c11f27e79961f8.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60` | absolute machine path |
| `2c69f2f57b7e` · `target-webdeploy/web/deps/synstructure-f09698470bf6a345.d:1,3,5,7,8` | absolute machine path |
| `0bc3dd5fd4f3` · `target-webdeploy/web/deps/thiserror_impl-866b8b5169c4adf1.d:1,3,5,6,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `46d1480e576e` · `target-webdeploy/web/deps/toml-fb369889cdd0e09d.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41` | absolute machine path |
| `e39f71fa7434` · `target-webdeploy/web/deps/toml_datetime-119d0274b7876f09.d:1,3,5,7,8,9,10` | absolute machine path |
| `c0464270d13d` · `target-webdeploy/web/deps/toml_parser-08af305d50c8d42a.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19` | absolute machine path |
| `09f20bf51b46` · `target-webdeploy/web/deps/toml_writer-a38a7e86ab9aceee.d:1,3,5,7,8,9,10,11,12` | absolute machine path |
| `86d986162c8e` · `target-webdeploy/web/deps/tracing_attributes-6524c0c513ed633c.d:1,3,5,6,7` | absolute machine path |
| `7da3c78a9b0b` · `target-webdeploy/web/deps/unicode_ident-82952c4e5edf73c0.d:1,3,5,7,8` | absolute machine path |
| `eebb11f7f4b0` · `target-webdeploy/web/deps/unicode_segmentation-52243cfece65f3b5.d:1,3,5,7,8,9,10,11` | absolute machine path |
| `7e4ee25954b3` · `target-webdeploy/web/deps/version_check-bb21ec76f396fc8b.d:1,3,5,7,8,9,10` | absolute machine path |
| `e57e6ff2e4d5` · `target-webdeploy/web/deps/wasm_bindgen_macro-6ec03b942e279ee9.d:1,3,5` | absolute machine path |
| `a12573e64b6a` · `target-webdeploy/web/deps/wasm_bindgen_macro_support-2d1ad74348de5174.d:1,3,5,7,8,9,10,11,12,13,14` | absolute machine path |
| `21d2431e5748` · `target-webdeploy/web/deps/wasm_bindgen_shared-05d1a8e8ebaa6b05.d:1,3,5,7,8,9` | absolute machine path |
| `b49bd46d0d86` · `target-webdeploy/web/deps/winnow-50a29f8afb993434.d:1,3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20` | absolute machine path |
| `bbfb86eeea3c` · `target-webdeploy/web/deps/yoke_derive-799a5defedd5f39a.d:1,3,5,6,7` | absolute machine path |
| `a5429c23bdc2` · `target-webdeploy/web/deps/zerocopy_derive-7199598f9492a86b.d:1,3,5,6,7,8,9,10,11,12,13,14,15` | absolute machine path |
| `8584d8b38d5f` · `target-webdeploy/web/deps/zerofrom_derive-6804d03aa2a25beb.d:1,3,5,6` | absolute machine path |
| `e4a64685e2c6` · `target-webdeploy/web/deps/zerovec_derive-396cd6c74f100cc8.d:1,3,5,6,7,8,9,10` | absolute machine path |
| `bb30aa0a6ced` · `target-webdeploy/web/deps/ztracing_macro_gpui_unofficial-c43c5e0342d41974.d:1,3,5` | absolute machine path |
| `49f5d04c37d0` · `crates/voxelite-app/Cargo.toml:17` | Git identity name/username |
| `9d55667d1fe8` · `crates/voxelite-app/src/actions.rs:96` | Git identity name/username |
| `10823a203ac2` · `crates/voxelite-app/src/palettes.rs:33` | Git identity name/username |
| `244398dbe8a5` · `crates/voxelite-app/src/ui.rs:1198,2733` | Git identity name/username |
| `2c9f681d8c7e` · `web/index.html:7,8,18` | Git identity name/username |
| `93c81839463f` · `crates/voxelite-app/Cargo.toml:13` | Git identity name/username |
| `88256749e8f8` · `crates/voxelite-app/src/actions.rs:86` | Git identity name/username |
| `7f9f320c4f86` · `crates/voxelite-app/src/palettes.rs:36` | Git identity name/username |
| `51954440730b` · `crates/voxelite-app/src/persistence.rs:84` | Git identity name/username |
| `54cb64fd71d0` · `crates/voxelite-app/src/share.rs:58,169` | Git identity name/username |
| `74ec0419c111` · `crates/voxelite-app/src/ui.rs:1190,2725` | Git identity name/username |
| `c001c22425cc` · `crates/voxelite-app/src/ui/agent.rs:103` | Git identity name/username |
| `4d958637c9c1` · `PLAN.md:1,27,55,64,139,144,198,262,286,290,309,350,351,360,392,414,415,434,641,642,645,647,693,697,700,729,742,748,830,835` | Git identity name/username |
| `23768d4dc8d6` · `crates/voxelite-app/src/ui.rs:1188,2651` | Git identity name/username |
| `ccb6fe467c82` · `crates/voxelite-app/src/ui/agent.rs:103` | Git identity name/username |
| `2caaec1bab4e` · `scripts/capture-native.sh:34,38,62` | Git identity name/username |
| `b35719407a90` · `screenshots/polish-2/agent-light.png:582` | email |
| `e7c85840461f` · `crates/voxelite-app/README.md:1,3,18,19,104,165,196,225` | Git identity name/username |
| `bcc9b195c58a` · `PLAN.md:1,27,55,64,139,144,198,262,286,290,309,350,351,360,392,414,415,434,641,642,645,647,693,697,700,729,742,748,830,835` | Git identity name/username |
| `c02b2c9945e8` · `scripts/capture-native.sh:34,38,62` | Git identity name/username |
| `69610e6b531e` · `scripts/review-polish-2.sh:28,32` | Git identity name/username |
| `c286d465829d` · `PLAN.md:1,27,63,138,143,197,261,285,289,308,349,350,359,391,413,414,433,640,641,644,646,692,696,699,728,741,747,829,834` | Git identity name/username |
| `5ec3eeab697d` · `crates/voxelite-app/README.md:1,3,18,19,104,190,219` | Git identity name/username |
| `a0d6401346c1` · `crates/voxelite-app/src/ui.rs:1181,2641` | Git identity name/username |
| `8190eb607611` · `crates/voxelite-app/README.md:1,3,18,19,104,188,217` | Git identity name/username |
| `5e329e850164` · `crates/voxelite-app/src/ui.rs:1180,2628` | Git identity name/username |
| `89d6faacb963` · `crates/voxelite-app/README.md:1,3,18,19,104,186,215` | Git identity name/username |
| `42242bfacf05` · `crates/voxelite-app/src/ui.rs:1165,2613` | Git identity name/username |
| `8556bf34ce9c` · `scripts/review-polish-2.sh:28,32` | Git identity name/username |
| `f8cf03999de1` · `scripts/capture-native.sh:29,33,57` | Git identity name/username |
| `972f932162c8` · `crates/voxelite-app/src/ui.rs:1138,2578` | Git identity name/username |
| `d5000bf548f0` · `crates/voxelite-app/README.md:1,3,18,19,104,186,209` | Git identity name/username |
| `28bab98ef69e` · `crates/voxelite-app/src/ui.rs:1138,2582` | Git identity name/username |
| `907e75214280` · `crates/voxelite-app/src/ui/agent.rs:103` | Git identity name/username |
| `4bd33da4e8de` · `scripts/review-polish-2.sh:28,32` | Git identity name/username |
| `04ef24362d6b` · `crates/voxelite-app/src/ui.rs:1137,2565` | Git identity name/username |
| `1faa10728c59` · `crates/voxelite-app/src/ui/agent.rs:103` | Git identity name/username |
| `5a7cf9ba9e50` · `crates/voxelite-app/src/ui.rs:1137,2565` | Git identity name/username |
| `0903032f5ee7` · `crates/voxelite-app/src/ui/agent.rs:103` | Git identity name/username |
| `d54ac02da932` · `crates/voxelite-app/src/ui.rs:1117,2555` | Git identity name/username |
| `307c61fb8cd6` · `crates/voxelite-app/src/ui/agent.rs:79` | Git identity name/username |
| `48e2751cb944` · `crates/voxelite-app/src/actions.rs:49` | Git identity name/username |
| `088a78f076f0` · `crates/voxelite-app/src/ui.rs:1115,2540` | Git identity name/username |
| `542713318c51` · `crates/voxelite-app/src/ui/agent.rs:79` | Git identity name/username |
| `976d3810d331` · `crates/voxelite-app/README.md:1,3,18,19,104,188,211` | Git identity name/username |
| `1ea551e4c868` · `crates/voxelite-app/src/main.rs:1,64,131,140` | Git identity name/username |
| `7d3aaaeb0cf8` · `crates/voxelite-app/src/ui.rs:1080,2505` | Git identity name/username |
| `4d8f4b822378` · `docs/mcp.md:1,121,150` | Git identity name/username |
| `38ea412b2b4e` · `scripts/capture-native.sh:25,29,53` | Git identity name/username |
| `dc4ac43bc654` · `crates/voxelite-app/README.md:1,3,18,19,107` | Git identity name/username |
| `f21ee004175a` · `crates/voxelite-app/src/palettes.rs:36` | Git identity name/username |
| `523cbb032367` · `crates/voxelite-app/src/ui.rs:1054,2479` | Git identity name/username |
| `0e9fea607fcf` · `crates/voxelite-app/src/ui/agent.rs:79` | Git identity name/username |
| `166023d00c80` · `scripts/capture-native.sh:6,24,28,51` | Git identity name/username |
| `04529277c441` · `crates/voxelite-app/src/ui.rs:1054,2477` | Git identity name/username |
| `8728cbe0be4b` · `crates/voxelite-app/src/ui/agent.rs:79` | Git identity name/username |
| `4317fd9f0811` · `crates/voxelite-app/Cargo.toml:13` | Git identity name/username |
| `a28bcb32523a` · `crates/voxelite-app/src/main.rs:1,55,122,127` | Git identity name/username |
| `d7bdec37f291` · `crates/voxelite-app/src/ui.rs:1014,2426` | Git identity name/username |
| `93ec15a372aa` · `crates/voxelite-mcp/Cargo.toml:7` | Git identity name/username |
| `192fef512bcd` · `crates/voxelite-mcp/src/lib.rs:1,24,92` | Git identity name/username |
| `2c669372ce64` · `crates/voxelite-mcp/src/main.rs:20` | Git identity name/username |
| `ecb3195311a1` · `crates/voxelite-app/src/ui.rs:1006,2377` | Git identity name/username |
| `d24a166c2edc` · `crates/voxelite-app/src/actions.rs:47` | Git identity name/username |
| `10e0e138df7c` · `crates/voxelite-app/src/ui.rs:991,2325` | Git identity name/username |
| `6856147e9364` · `crates/voxelite-app/src/actions.rs:45` | Git identity name/username |
| `a0a42db7a407` · `crates/voxelite-app/src/ui.rs:985,2308` | Git identity name/username |
| `6f8b36d5c206` · `crates/voxelite-app/src/actions.rs:44` | Git identity name/username |
| `2ec8c894b035` · `crates/voxelite-app/src/main.rs:1,54,121,126` | Git identity name/username |
| `99780d6c9136` · `crates/voxelite-app/src/ui.rs:953,2253` | Git identity name/username |
| `a0bd42cd4b4a` · `crates/voxelite-app/src/actions.rs:44` | Git identity name/username |
| `e46293b8c8f8` · `crates/voxelite-app/src/ui.rs:953,2270` | Git identity name/username |
| `74069021eddc` · `crates/voxelite-app/README.md:1,3,12,13,98` | Git identity name/username |
| `9184b64e136b` · `docs/verification/native-app.md:8,15` | Git identity name/username |
| `f3199fa34752` · `scripts/review-native.sh:6,28,32,75,80,85,88,92,108,112,126,128,130,132` | Git identity name/username |
| `ecc6abdcf2eb` · `crates/voxelite-app/src/ui.rs:953,2270` | Git identity name/username |
| `cd32f695899f` · `scripts/review-native.sh:6,27,31,74,79,84,87,91,107,111,125,127,129,131` | Git identity name/username |
| `834622594828` · `crates/voxelite-app/src/main.rs:1,54,117,122` | Git identity name/username |
| `446b60b3ac3e` · `crates/voxelite-app/src/ui.rs:941,2146` | Git identity name/username |
| `4d58bd1d8f99` · `crates/voxelite-app/src/ui.rs:887,1993` | Git identity name/username |
| `56391baf19c1` · `scripts/capture-native.sh:6,24,28,48` | Git identity name/username |
| `294fb928eb42` · `scripts/review-native.sh:6,27,31,72,77,82,85,89,105,109,123,125,127,129` | Git identity name/username |
| `aa51c949e0bb` · `crates/voxelite-app/src/ui.rs:887,1991` | Git identity name/username |
| `5ad25ea27a0f` · `docs/verification/native-app.md:4,11` | Git identity name/username |
| `2886e9c657ad` · `crates/voxelite-app/README.md:1,3,10,11,80` | Git identity name/username |
| `81906b8e7230` · `crates/voxelite-app/src/ui.rs:882,1986` | Git identity name/username |
| `e4e99012a5ea` · `scripts/review-native.sh:6,27,31,71,76,81,84,88,104,108,116,118,120,122` | Git identity name/username |
| `480a08ed629a` · `crates/voxelite-app/src/persistence.rs:80` | Git identity name/username |
| `e5f87d0f802a` · `crates/voxelite-app/src/ui.rs:878,1972` | Git identity name/username |
| `68552cde9348` · `scripts/capture-native.sh:6,24,28,45` | Git identity name/username |
| `7518de1939a1` · `crates/voxelite-app/src/actions.rs:44` | Git identity name/username |
| `7a4f88ed8cbe` · `crates/voxelite-app/src/ui.rs:792,1750` | Git identity name/username |
| `8c78cf8e9ca0` · `crates/voxelite-app/src/actions.rs:43` | Git identity name/username |
| `99eb55f8f808` · `crates/voxelite-app/src/ui.rs:423,670,1628` | Git identity name/username |
| `31842e9bed1d` · `crates/voxelite-app/Cargo.toml:13` | Git identity name/username |
| `c794e9256012` · `docs/verification/native-app.md:4,11` | Git identity name/username |
| `62030e756fb6` · `crates/voxelite-app/Cargo.toml:13` | Git identity name/username |
| `ffb0021d3d2c` · `crates/voxelite-app/src/main.rs:1,54,116,121` | Git identity name/username |
| `4e347a084c82` · `crates/voxelite-app/src/ui.rs:423,670,1628` | Git identity name/username |
| `57f9602bb346` · `crates/voxelite-app/src/main.rs:1,54,115,120` | Git identity name/username |
| `c8d8200eadb1` · `crates/voxelite-app/src/ui.rs:419,666,1618` | Git identity name/username |
| `4beac0219d4c` · `crates/voxelite-app/src/ui.rs:415,662,1611` | Git identity name/username |
| `e25798899bd8` · `crates/voxelite-app/src/ui.rs:359,595,1533` | Git identity name/username |
| `de9bd5d078a8` · `crates/voxelite-app/src/ui.rs:353,583,1509` | Git identity name/username |
| `489798319a83` · `crates/voxelite-app/src/actions.rs:9` | Git identity name/username |
| `8408f09df382` · `crates/voxelite-app/src/main.rs:1` | Git identity name/username |
| `fef62d317d60` · `crates/voxelite-app/src/ui.rs:130,202` | Git identity name/username |
| `0d6449236737` · `crates/voxelite-app/Cargo.toml:13` | Git identity name/username |
| `7db670596be0` · `docs/mcp.md:1,116,145` | Git identity name/username |
| `10bb54f39b3e` · `spec/mcp/v1/contract.md:1,12` | Git identity name/username |
| `24502cbc34b8` · `crates/voxelite-mcp/src/catalog.rs:206,234,366` | Git identity name/username |
| `68fa416d3cbd` · `spec/mcp/v1/tools.json:4,48,54,314,608,804,1091,1378,1683,1910,2148,2397,2569,2847,3023,3253,3530,3817,4078,4261,4540,4825,5109,5392,5677,5948,6141,6404,6595,6913,7282,7489,7678,7964,8661,9031,9375,9750,10029,10295` | Git identity name/username |
| `40239328a68c` · `crates/voxelite-mcp/src/catalog.rs:183,211,328` | Git identity name/username |
| `052758066162` · `crates/voxelite-mcp/tests/stdio.rs:92` | Git identity name/username |
| `58cd44fb2df5` · `docs/mcp.md:1,116,145` | Git identity name/username |
| `8af61be693bd` · `spec/mcp/v1/compatibility.json:2` | Git identity name/username |
| `4cff4c014ebf` · `crates/voxelite-mcp/src/catalog.rs:183,211,328` | Git identity name/username |
| `f17114a94679` · `crates/voxelite-mcp/tests/stdio.rs:92` | Git identity name/username |
| `b7b44980f045` · `spec/mcp/v1/tools.json:4,48,54,314,608,804,1091,1378,1683,1910,2148,2397,2569,2847,3023,3253,3530,3817,4078,4261,4540,4825,5109,5392,5677,5948,6141,6404,6595,6913,7282,7489,7678,7964,8661,9031,9375,9750,10029,10295` | Git identity name/username |
| `510d550e8cbe` · `crates/voxelite-mcp/src/catalog.rs:182,224` | Git identity name/username |
| `4964c81b9e7b` · `docs/mcp.md:1,114,143` | Git identity name/username |
| `57cd646d754d` · `spec/mcp/v1/contract.md:1,12` | Git identity name/username |
| `9673fd4c3210` · `spec/mcp/v1/tools.json:4,48,279,430,584,748,902,1084,1217,1361,1516,1655,1900,2043,2197,2351,2515,2648,2808,2964,3126,3287,3447,3609,3750,3913,4046,4217,4412,4651,4784,4950,5113,5682,5929,6150,6402,6558,6796` | Git identity name/username |
| `c92319427e06` · `crates/voxelite-mcp/src/lib.rs:1,24,90` | Git identity name/username |
| `1498a8d2959e` · `crates/voxelite-mcp/src/lib.rs:1,23,76` | Git identity name/username |
| `a8c2efcdb2f9` · `crates/voxelite-mcp/src/main.rs:21` | Git identity name/username |
| `d73ec5b4630a` · `crates/voxelite-mcp/src/catalog.rs:52,67` | Git identity name/username |
| `3c7fb3c530f2` · `crates/voxelite-mcp/src/lib.rs:1,11,21` | Git identity name/username |
| `c30a6a5c3d71` · `crates/voxelite-export/src/lib.rs:70,103` | Git identity name/username |
| `c4b57df8dde2` · `crates/voxelite-export/src/lib.rs:71,104` | Git identity name/username |
| `0642b6e2c657` · `crates/voxelite-export/src/lib.rs:1` | Git identity name/username |
| `d232e7950a3a` · `crates/voxelite-format/src/lib.rs:21,179,206,208` | Git identity name/username |
| `70f00c75f772` · `crates/voxelite-render/src/lib.rs:1` | Git identity name/username |
| `ee68e94a4002` · `crates/voxelite-format/src/lib.rs:1` | Git identity name/username |
| `41ff19b68c8f` · `PLAN.md:1,3,18,65,135,137,148,168,204,318,414,420` | Git identity name/username |
| `94ff0e955ccf` · `design/REVISIONS.md:423` | absolute machine path |
| `94ff0e955ccf` · `design/REVISIONS.md:1,252,407` | Git identity name/username |
| `8ec2cd4aa90f` · `design/REVISIONS.md:1,252` | Git identity name/username |
| `2a3d0a9ed864` · `design/concepts/concept-a/index.html:2,1904,1914,3283` | Git identity name/username |
| `1dec35a40938` · `scripts/lib/concept-a-controls.js:937` | Git identity name/username |
| `8dc2852aea63` · `scripts/lib/concept-a.mjs:25,35` | Git identity name/username |
| `1cda23b27a34` · `design/REVISIONS.md:3,254,381,399,405,441` | Git identity name/username |
| `ad6225d0dde5` · `design/concepts/README.md:1` | Git identity name/username |
| `14661d2606ec` · `design/concepts/concept-a/index.html:2,1901,1911,3280` | Git identity name/username |
| `eb55ee4a2df9` · `scripts/lib/concept-a-controls.js:937` | Git identity name/username |
| `7b0553d0ac9a` · `PLAN.md:1,3,18,65,135,137,148,168,204,318,414,420` | Git identity name/username |
| `baaf5c77ebe9` · `design/REVISIONS.md:1,252` | Git identity name/username |
| `b200879acdee` · `design/concepts/concept-a/index.html:2,1904,1914,3283` | Git identity name/username |
| `8a104869e900` · `scripts/lib/concept-a-controls.js:937` | Git identity name/username |
| `58021dd59b6e` · `design/concepts/concept-a/index.html:2,1845,1855,3191` | Git identity name/username |
| `8f5374522776` · `scripts/lib/concept-a-controls.js:904` | Git identity name/username |
| `5bbde93bf692` · `scripts/lib/concept-a.mjs:25,35` | Git identity name/username |
| `4401f08cd702` · `design/concepts/concept-a/index.html:2,1798,1808,3144` | Git identity name/username |
| `58019df8a99e` · `scripts/lib/concept-a-controls.js:904` | Git identity name/username |
| `38f92d90616d` · `scripts/lib/concept-a.mjs:25,35` | Git identity name/username |
| `62ef15896d2b` · `scripts/lib/concept-a-controls.js:925` | Git identity name/username |
| `297064f0d58b` · `scripts/lib/concept-a.mjs:25,35` | Git identity name/username |
| `42a95de53d8b` · `design/REVISIONS.md:1,252` | Git identity name/username |
| `53bf48876353` · `design/concepts/concept-a/index.html:2,1673,1683,3039` | Git identity name/username |
| `9736d51fc766` · `scripts/lib/concept-a-controls.js:924` | Git identity name/username |
| `63ff81bf62a1` · `scripts/lib/concept-a-controls.js:1310` | Git identity name/username |
| `c526c7a79a70` · `scripts/lib/concept-a.mjs:25,35` | Git identity name/username |
| `a481c722002f` · `design/REVISIONS.md:1,252` | Git identity name/username |
| `74803a202262` · `design/concepts/concept-a/index.html:2,10,20,633` | Git identity name/username |
| `902e3ea936d9` · `scripts/build-concepts.mjs:58,105` | Git identity name/username |
| `210a7f863c29` · `scripts/lib/concept-a-controls.js:181` | Git identity name/username |
| `d2e39a391911` · `scripts/lib/concept-a.mjs:15,25` | Git identity name/username |
| `257ec5f4583a` · `PLAN.md:1,3,57,127,129,140,160,196,310,406,412` | Git identity name/username |
| `750f2996b69b` · `design/ART-CHECKLIST.md:1,24` | Git identity name/username |
| `06ad6c6fd8dd` · `design/REVISIONS.md:1` | Git identity name/username |
| `fef49cc44c06` · `design/concepts/concept-a/index.html:2,9,19,631` | Git identity name/username |
| `be40bca4b83b` · `scripts/lib/concept-a-controls.js:181` | Git identity name/username |
| `b479712caef2` · `scripts/lib/concept-a.mjs:15,25` | Git identity name/username |
| `948016b39e96` · `scripts/build-concepts.mjs:57,99` | Git identity name/username |
| `0726ebb62c1b` · `scripts/lib/concept-a.mjs:15,25` | Git identity name/username |
| `5c3674b835e9` · `design/ART-CHECKLIST.md:1,24` | Git identity name/username |
| `2548f0b0f651` · `design/REVISIONS.md:14,32,38,74` | Git identity name/username |
| `d08c4437d373` · `design/concepts/README.md:1` | Git identity name/username |
| `a929978d3c18` · `design/concepts/concept-a/index.html:7,39` | Git identity name/username |
| `7fa61c8c0a12` · `design/concepts/concept-b/index.html:7,39` | Git identity name/username |
| `a18a1d2ac971` · `design/concepts/concept-c/index.html:7,39` | Git identity name/username |
| `6d28bf0f070c` · `scripts/build-concepts.mjs:47,127` | Git identity name/username |
| `8066c9d551a2` · `design/REVISIONS.md:1` | Git identity name/username |
| `63b22f06fce8` · `PLAN.md:1,3,57,127,129,140,160,196,306,402,408` | Git identity name/username |
| `00c6600db2cb` · `design/REVISIONS.md:1` | Git identity name/username |
| `42da1072b39a` · `design/concepts/README.md:1` | Git identity name/username |
| `0b51fa1be252` · `design/concepts/concept-a/index.html:2,555` | Git identity name/username |
| `fa82a8cd05c9` · `design/concepts/concept-b/index.html:2,555` | Git identity name/username |
| `102c5d78cf4a` · `design/concepts/concept-c/index.html:2,555` | Git identity name/username |
| `1fe9642f1f13` · `scripts/build-concepts.mjs:55,97` | Git identity name/username |
| `a40b8352ecb9` · `design/REVISIONS.md:14,32,38` | Git identity name/username |
| `7b1126e512b9` · `PLAN.md:1,3,57,127,129,140,160,196,304,400,406` | Git identity name/username |
| `19f7e5b07e65` · `design/REVISIONS.md:1` | Git identity name/username |
| `3c7bf6e5c4fd` · `design/concepts/README.md:1` | Git identity name/username |
| `f23988864125` · `design/concepts/concept-a/index.html:2,27` | Git identity name/username |
| `9db4c019d2f1` · `design/concepts/concept-b/index.html:2,27` | Git identity name/username |
| `c7b4894c93f5` · `design/concepts/concept-c/index.html:2,27` | Git identity name/username |
| `e41cb2d9ac2c` · `scripts/build-concepts.mjs:55,96` | Git identity name/username |
| `815505bdd86f` · `design/ART-CHECKLIST.md:1` | Git identity name/username |
| `beb8dfa8be90` · `design/concepts/concept-a/index.html:7,39` | Git identity name/username |
| `0e2755e8abe2` · `design/concepts/concept-b/index.html:7,39` | Git identity name/username |
| `1953aa0cd59c` · `design/concepts/concept-c/index.html:7,39` | Git identity name/username |
| `0ad4c7eb97be` · `scripts/build-concepts.mjs:47,127` | Git identity name/username |

## Commit messages

- `1e6f8aaa1462:message:1` — Git identity name/username.
- `1240ffb9115b:message:1` — Git identity name/username.
- `e6a9adf1b364:message:1` — Git identity name/username.
- `60388338a09d:message:1` — Git identity name/username.

## Outcome

No credential-shaped strings or URL credentials were found unless explicitly listed above. Historical identity headers remain. License attributions remain. No trivially safe authored working-tree path leak was found outside the read-only design boundary; Git plumbing and generated compiler paths must not be blindly replaced. This report does not certify the repository as free of personal information.
