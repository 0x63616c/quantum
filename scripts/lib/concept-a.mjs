// Concept A has an independent component surface; B/C remain archived design studies.
export function conceptA({ icon: baseIcon, gizmo, colors }) {
  const extras = {
    comment:
      '<path d="M20 11.5a8 8 0 0 1-8 8H4v-4a8 8 0 1 1 16-4Z"/><path d="M8 10h8M8 14h5"/>',
    clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
    branch:
      '<circle cx="6" cy="5" r="2"/><circle cx="6" cy="19" r="2"/><circle cx="18" cy="5" r="2"/><path d="M6 7v10M18 7a8 8 0 0 1-8 8H6"/>',
    link: '<path d="m10 13 4-4M8 16l-1 1a4 4 0 0 1-6-6l5-5a4 4 0 0 1 6 0M16 8l1-1a4 4 0 0 1 6 6l-5 5a4 4 0 0 1-6 0" transform="translate(0 1) scale(.95)"/>',
  };
  const icon = (name) =>
    extras[name]
      ? `<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${extras[name]}</svg>`
      : baseIcon(name);
  const btn = (name, label, attrs = "", text = "") =>
    `<button ${attrs} aria-label="${label}" title="${label}" class="${text ? "" : "icon-button"}">${icon(name)}${text}</button>`;
  const close = (id, label) =>
    btn("close", `Close ${label}`, `data-close="${id}"`);
  const steps = [
    ["17", "Shape the canopy", "fill_box", 24],
    ["18", "Add leaf detail", "set_voxels", 6],
  ];
  const views = ["front", "right", "back"];
  const markup = `<main class="app">
    <nav class="menubar" role="menubar" aria-label="Application menu"><strong>Voxelite</strong>${["File", "Edit", "View", "Tools", "Layers", "Palette", "Agent", "Extensions", "Help"].map((name) => `<button role="menuitem" aria-haspopup="menu" aria-expanded="false" data-menu="${name}">${name}</button>`).join("")}</nav>
    <header class="unified-toolbar"><div class="window-controls" aria-hidden="true"><i></i><i></i><i></i></div><div class="document-name">${icon("cube")}<span>Woodland cottage<small>.voxl</small></span></div><span class="toolbar-divider"></span><button data-menu="ToolPicker" aria-haspopup="menu" aria-expanded="false" class="tool-picker"><span data-tool-icon>${icon("place")}</span><span data-current-tool>Place</span>${icon("down")}</button><div class="toolbar-spacer"></div><button class="command-trigger" data-action="commands">${icon("search")}<span>Search commands</span><kbd>⌘K</kbd></button><div class="presence" aria-label="Artist 01 editing, Designer 02 commenting, Canopy agent connected"><span class="avatar online" title="Artist 01 · Editing now">A1</span><span class="avatar online" title="Designer 02 · Commenting now">D2</span><span class="avatar agent-avatar online" title="Canopy agent · Connected">${icon("agent")}</span></div>${btn("comment", "Toggle comments", 'data-action="comments" aria-pressed="false"')}${btn("clock", "Version history", 'data-action="history" aria-expanded="false"')}<button class="share-button" data-action="share" aria-expanded="false">Share</button></header>
    <div class="workspace"><section class="viewport" aria-label="3D voxel viewport"><canvas tabindex="0" aria-label="Cottage viewport. Arrow keys orbit, Shift arrows pan, plus and minus zoom."></canvas><div class="view-heading"><span data-view-label>Front perspective</span></div>${gizmo}<div class="edit-annotation" hidden></div><div class="history-preview-banner" role="status" hidden><span></span><button data-action="history-live">Return to latest</button></div><div class="comment-pins" aria-label="3D comments"></div><div class="compare-badge floating" hidden></div><button class="viewport-zoom" data-menu="Zoom" aria-haspopup="menu" aria-expanded="false"><span data-zoom-label>100%</span>${icon("down")}</button></section>
    <div class="sidebar-resizer" role="separator" aria-label="Resize sidebar" aria-orientation="vertical" tabindex="0" aria-valuemin="320" aria-valuemax="440" aria-valuenow="360"></div><aside class="sidebar" aria-label="Editor sidebar"><div class="sidebar-tabs" role="tablist" aria-label="Activity"><button role="tab" aria-selected="true" aria-controls="session-panel" id="session-tab" data-tab="session">${icon("agent")}Session</button><button role="tab" aria-selected="false" aria-controls="comments-panel" id="comments-tab" data-tab="comments" tabindex="-1">${icon("comment")}Comments <span data-comment-count>3</span></button></div>
<section class="tool-settings" hidden aria-label="Tool settings"><div class="tool-settings-heading"><h2>Scatter moss</h2><span>Canopy surface</span></div><label>Density<input type="range" min="1" max="100" value="24" data-tool-density aria-label="Scatter density"><output>24%</output></label><label>Seed<input type="number" value="42" min="0" max="9999" data-tool-seed aria-label="Scatter seed"></label></section>    <section id="session-panel" class="agent-session" role="tabpanel" aria-labelledby="session-tab"><div class="session-identity"><span class="avatar agent-avatar">${icon("agent")}</span><div><h1>Refine the woodland cottage</h1><span>Canopy agent <span class="identity-dot">·</span> <span data-session-state>Live</span></span></div><button class="icon-button" data-menu="Agent" aria-label="Session options" aria-haspopup="menu" aria-expanded="false">${icon("more")}</button></div><div class="current-activity"><div><span data-running-title>Checking silhouettes</span><code data-running-label>render_views · 3 angles</code></div>${btn("pause", "Pause agent", 'data-action="pause" data-pause aria-pressed="false"', '<span>Pause</span>')}</div><div class="session-subline"><button class="follow-button" data-action="follow" data-follow aria-pressed="true"><span>Follow agent camera</span><span class="follow-switch" aria-hidden="true"></span><span data-follow-state>On</span></button></div><div class="recent-label">Recent changes</div><div class="session-steps">${steps.map(([id, name, tool, delta]) => `<details class="session-step" data-step="${id}" ${id === "18" ? "open" : ""}><summary><span class="step-status">${icon("check")}</span><span>${name}</span><span class="diff">+${delta} <span>−0</span></span>${icon("down")}</summary><div class="step-detail"><div><code>${tool}</code><button data-step-undo="${id}">${icon("undo")}<span>Undo step</span></button></div><details class="coordinate-detail"><summary>Canopy · ${delta} added cells ${icon("down")}</summary><pre data-step-coordinates="${id}"></pre></details></div></details>`).join("")}</div><section class="captures" aria-label="Multi-angle captures"><div class="section-title"><span>Captured views</span><span class="capture-meta" data-render-stale hidden>Outdated</span><code data-render-revision>r018</code><button class="icon-button" data-menu="Captures" aria-label="Capture options" aria-haspopup="menu" aria-expanded="false">${icon("more")}</button></div><div class="capture-angles segmented" role="group" aria-label="Captured angle">${views.map((v, i) => `<button data-capture-angle="${v}" aria-pressed="${i === 0}">${v[0].toUpperCase() + v.slice(1)}</button>`).join("")}</div><div class="render-grid">${views.map((v, i) => `<button class="render-preview" data-view-angle="${v}" data-capture-visible="${i === 0}" aria-label="View ${v} render" aria-pressed="${i === 0}"><canvas data-thumbnail="${v}" aria-label="${v} render"></canvas></button>`).join("")}</div></section></section>
    <section id="comments-panel" role="tabpanel" aria-labelledby="comments-tab" hidden><div class="section-title"><select data-comment-filter aria-label="Filter comments"><option value="open">Open</option><option value="resolved">Resolved</option><option value="all">All threads</option></select></div><div class="pin-thread" hidden aria-label="Pinned comment thread"></div><div class="comment-list"></div></section>
    <div class="sidebar-bottom"><section class="layers-section" aria-label="Layers"><div class="section-title"><h2>Layers</h2><button class="icon-button" data-menu="Layers" aria-label="Layer options" aria-haspopup="menu" aria-expanded="false">${icon("more")}</button></div><div class="layer-list">${["Canopy", "Roof", "Cottage", "Garden"].map((name, i) => `<div class="layer-row ${i === 0 ? "selected" : ""}" data-layer="${name}"><button class="layer-select" data-select-layer="${name}" aria-pressed="${i === 0}">${icon("layers")}<span>${name}</span></button>${btn("eye", `Toggle ${name} visibility`, `data-layer-eye="${name}" aria-pressed="true"`)}${btn(name === "Garden" ? "lock" : "unlock", `Toggle ${name} lock`, `data-layer-lock="${name}" aria-pressed="${name === "Garden"}"`)}</div>`).join("")}</div></section>
    <section class="palette-section" aria-label="Active palette"><div class="section-title"><button class="palette-switch" data-menu="PaletteSwitcher" aria-haspopup="menu" aria-expanded="false"><span data-palette-name>Woodland</span>${icon("down")}</button><button class="icon-button" data-menu="Palette" aria-label="Palette options" aria-haspopup="menu" aria-expanded="false">${icon("more")}</button></div><div class="swatch-grid" data-active-swatches></div><div class="palette-caption"><code data-color-label>#829A63</code><span data-palette-count>16 colors</span></div></section></div></aside></div>
    <footer class="statusbar"><span><span data-voxel-count>2,079</span> voxels <span class="footer-dot">·</span> 32 × 32 × 32</span><span>Concept</span></footer>
    <div class="menu-root"></div><div class="toast" role="status" hidden></div>
    <section class="share-popover floating" hidden role="dialog" aria-labelledby="share-title"><div class="floating-heading"><h2 id="share-title">Share this project</h2>${close("share", "sharing")}</div><div class="share-link-group"><label class="share-scope"><strong>Link access</strong><select data-access-scope aria-label="Link access"><option value="link">Anyone with the link</option><option value="restricted">Restricted</option></select></label><div class="segmented" aria-label="Link permission" role="group">${["view", "comment", "remix"].map((p) => `<button data-permission="${p}" aria-pressed="${p === "comment"}">Can ${p}</button>`).join("")}</div><p class="permission-description">View and comment. No voxel edits.</p><div class="link-field"><input aria-label="Share link" readonly value="https://voxelite.example/s/woodland?access=comment"><button class="primary copy-link" data-action="copy-link">${icon("copy")}<span>Copy link</span></button></div><div class="share-feedback" role="status" aria-live="polite"></div></div><div class="share-members"><h3>People & agents</h3><p class="access-rule">Link or individual permission, whichever is higher.</p><div class="share-member"><span class="avatar">A1</span><div>Artist 01</div><span class="muted">Owner</span></div><div class="share-member"><span class="avatar">D2</span><div>Designer 02</div><select aria-label="Designer 02 permission" data-member-permission><option>Can comment</option><option>Can view</option><option>Can remix</option></select></div><div class="share-member"><span class="avatar agent-avatar">${icon("agent")}</span><div>Canopy agent</div><select aria-label="Canopy agent permission" data-member-permission><option>Can edit</option><option>Can comment</option><option>Can view</option></select></div></div></section>
    <section class="history-panel" hidden role="dialog" aria-labelledby="versions-title"><div class="floating-heading"><h2 id="versions-title">Version history</h2>${close("history", "history")}</div><div class="history-branch">${icon("branch")}<span data-branch-name>main</span><span class="muted">Local history</span></div><div class="history-preview-title"><strong data-history-step-title>Leaf detail</strong><span data-history-position>Step 8 / 8</span></div><div class="history-navigation"><button data-action="history-prev" aria-label="Previous edit">← Previous</button><button data-action="history-next" aria-label="Next edit">Next →</button></div><div class="history-scrubber"><div class="timeline-marks" aria-hidden="true">${Array.from({length:9},(_,i)=>`<i class="${i === 0 ? "initial-marker" : i <= 4 ? "human-marker" : "agent-marker"}"></i>`).join("")}</div><input type="range" min="0" max="8" value="8" step="1" data-timeline aria-label="Scrub edit history"></div><div class="timeline-endpoints"><span>0 · Initial</span><span>8 · Latest edit</span></div><div class="timeline-legend"><span><i class="human-marker"></i>Human</span><span><i class="agent-marker"></i>Agent</span></div><div class="history-list-label">Edit history <span>9 states</span></div><div class="history-event-list" tabindex="0" aria-label="All nine history states"></div><div class="versions-heading"><h3>Named versions</h3><button data-action="save-version">${icon("plus")}Name version</button></div><form class="version-form" hidden><p data-version-save-context></p><input aria-label="Version name" placeholder="Version name" maxlength="60" required><button type="submit">Save</button></form><div class="version-list"></div></section>
    <dialog class="palette-dialog" aria-labelledby="palette-title"><div class="floating-heading"><h2 id="palette-title">Palettes</h2>${close("palettes", "palettes")}</div><div class="palette-manager-body"><nav class="palette-library" aria-label="Palette library"><div><div class="library-label">Your palettes</div><div data-custom-palettes></div><button class="create-palette" data-action="create-palette">${icon("plus")}New palette</button></div><div><div class="library-label">Preset library</div><div data-preset-palettes></div></div></nav><section class="palette-editor"><div class="palette-editor-heading"><div><input data-palette-rename aria-label="Palette name" maxlength="60"><h3 data-editor-title hidden></h3><p class="muted" data-editor-subtitle></p></div><button class="icon-button" data-menu="PaletteManage" aria-label="Manage this palette" aria-haspopup="menu" aria-expanded="false">${icon("more")}</button></div><button class="duplicate-preset" data-action="duplicate-palette">${icon("copy")}Duplicate to edit</button><div class="manager-swatches" role="group" aria-label="Edit swatches"></div><div class="swatch-editor"><span class="color-preview"></span><label><span data-swatch-position>Color 11</span><input data-hex-edit aria-label="Swatch hex" maxlength="7" spellcheck="false"></label><button class="icon-button" data-menu="SwatchEdit" aria-label="Swatch actions" aria-haspopup="menu" aria-expanded="false">${icon("more")}</button><button data-action="add-swatch">${icon("plus")}Add color</button></div><div class="palette-use"><span class="muted" data-editor-status>Applies to new voxels</span><button class="primary" data-action="use-palette">Use palette</button></div></section></div><input type="file" accept=".hex,.gpl" data-palette-file hidden></dialog>
    <dialog class="command-dialog" aria-label="Command palette"><div class="command-search">${icon("search")}<input aria-label="Search commands" role="combobox" aria-expanded="true" aria-autocomplete="list" aria-controls="command-results" placeholder="Search commands…"><kbd>esc</kbd></div><div id="command-results" role="listbox" class="command-results"></div><div class="command-footer">↑ ↓ Navigate <span>↵ Run command</span></div></dialog>
    <div hidden aria-hidden="true"><button data-refresh-renders></button><button data-toggle-grid aria-pressed="true"></button><button data-reset-view></button><select data-zoom-select>${[75, 100, 125, 150, 175].map((n) => `<option value="${n}">${n}</option>`).join("")}<option data-custom-zoom hidden></option></select></div>
  </main>`;
  return {
    markup,
    icons: Object.fromEntries(
      [
        "place",
        "erase",
        "paint",
        "fill",
        "box",
        "picker",
        "agent",
        "check",
        "down",
        "chevron",
        "undo",
        "redo",
        "comment",
        "clock",
        "layers",
        "swatches",
        "copy",
        "branch",
        "trash",
        "eye",
        "lock",
        "unlock",
        "link",
        "plus",
        "close",
        "camera",
        "grid",
        "sun",
        "focus",
        "search",
        "import",
        "export",
        "refresh",
        "more",
      ].map((n) => [n, icon(n)]),
    ),
    woodland: [
      colors[0],
      colors[1],
      colors[4],
      colors[5],
      colors[6],
      colors[7],
      colors[9],
      colors[18],
      colors[13],
      colors[14],
      colors[15],
      colors[16],
      colors[17],
      colors[22],
      colors[23],
      colors[30],
    ],
  };
}
