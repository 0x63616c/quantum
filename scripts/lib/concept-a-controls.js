/* Offline design fixture. No network session, sharing service or plugin host is implied. */
(() => {
  const $ = (s) => document.querySelector(s),
    $$ = (s) => [...document.querySelectorAll(s)];
  const { icons, woodland } = window.__conceptA;
  const esc = (value) =>
    String(value).replace(
      /[&<>"']/g,
      (c) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          '"': "&quot;",
          "'": "&#39;",
        })[c],
    );
  let toastTimer,
    menuTrigger,
    menuChain = [],
    modalFocus,
    follow = true,
    paused = false,
    historyPreview = false,
    tool = "place",
    selectedLayer = "Canopy";
  const mirrors = new Set(),
    lockedLayers = new Set(["Garden"]);
  const notify = (message) => {
    $(".toast").textContent = message;
    $(".toast").hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => ($(".toast").hidden = true), 2600);
  };
  const download = (name, text, type = "text/plain") => {
    const url = URL.createObjectURL(new Blob([text], { type }));
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  const presets = [
    {
      id: "pico8",
      name: "PICO-8",
      colors:
        "000000 1d2b53 7e2553 008751 ab5236 5f574f c2c3c7 fff1e8 ff004d ffa300 ffec27 00e436 29adff 83769c ff77a8 ffccaa",
    },
    {
      id: "endesga32",
      name: "Endesga 32",
      colors:
        "be4a2f d77643 ead4aa e4a672 b86f50 733e39 3e2731 a22633 e43b44 f77622 feae34 fee761 63c74d 3e8948 265c42 193c3e 124e89 0099db 2ce8f5 ffffff c0cbdc 8b9bb4 5a6988 3a4466 262b44 181425 ff0044 68386c b55088 f6757a e8b796 c28569",
    },
    {
      id: "sweetie16",
      name: "Sweetie 16",
      colors:
        "1a1c2c 5d275d b13e53 ef7d57 ffcd75 a7f070 38b764 257179 29366f 3b5dc9 41a6f6 73eff7 f4f4f4 94b0c2 566c86 333c57",
    },
    {
      id: "grayscale",
      name: "Grayscale",
      colors: Array.from({ length: 16 }, (_, i) =>
        (i * 17).toString(16).padStart(2, "0").repeat(3),
      ).join(" "),
    },
  ].map((p) => ({
    ...p,
    preset: true,
    colors: p.colors.split(" ").map((c) => "#" + c),
  }));
  let custom = [{ id: "woodland", name: "Woodland", colors: woodland }],
    activePalette = "woodland",
    editingPalette = "woodland",
    swatchIndex = 10,
    editorIndex = 10;
  try {
    const saved = JSON.parse(localStorage.getItem("voxelite-palettes-v1"));
    if (
      Array.isArray(saved?.custom) &&
      saved.custom.every(
        (p) =>
          typeof p.id === "string" &&
          typeof p.name === "string" &&
          p.name.trim() &&
          p.colors.length &&
          p.colors.length <= 256 &&
          p.colors.every((c) => /^#[\da-f]{6}$/i.test(c)),
      )
    ) {
      custom = saved.custom;
      activePalette = saved.activePalette;
    }
  } catch {}
  const palettes = () => [...custom, ...presets],
    palette = (id) =>
      palettes().find((p) => p.id === id) || custom[0] || presets[0],
    active = () => palette(activePalette),
    editing = () => palette(editingPalette);
  if (!palettes().some((p) => p.id === activePalette))
    activePalette = palettes()[0].id;
  editingPalette = activePalette;
  const persist = () => {
    try {
      localStorage.setItem(
        "voxelite-palettes-v1",
        JSON.stringify({ custom, activePalette }),
      );
    } catch {
      notify("Palette storage unavailable; export to keep your colors.");
    }
  };
  const uid = (prefix) =>
    `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
  const swatchButton = (color, index, selected, editor = false) =>
    `<button class="swatch" style="--swatch:${color}" ${editor ? "data-edit-swatch" : "data-swatch"}="${index}" data-color="${color}" aria-label="${editor ? "Edit" : "Select"} color ${color.toUpperCase()}" aria-pressed="${index === selected}" tabindex="${index === selected ? 0 : -1}"></button>`;
  function renderPalette() {
    const p = active();
    swatchIndex = Math.min(swatchIndex, p.colors.length - 1);
    $("[data-palette-name]").textContent = p.name;
    $("[data-palette-count]").textContent = `${p.colors.length} colors`;
    $("[data-color-label]").textContent = p.colors[swatchIndex].toUpperCase();
    $("[data-active-swatches]").innerHTML = p.colors
      .map((c, i) => swatchButton(c, i, swatchIndex))
      .join("");
  }
  function renderManager() {
    const p = editing();
    editorIndex = Math.max(0, Math.min(editorIndex, p.colors.length - 1));
    for (const [selector, items] of [
      ["[data-custom-palettes]", custom],
      ["[data-preset-palettes]", presets],
    ])
      $(selector).innerHTML = items
        .map(
          (q) =>
            `<button data-edit-palette="${esc(q.id)}" aria-current="${q.id === p.id}">${esc(q.name)}<small>${q.colors.length}</small></button>`,
        )
        .join("");
    $("[data-editor-title]").textContent = p.name;
    $("[data-editor-subtitle]").textContent =
      `${p.preset ? "Read-only preset" : "Custom palette"} · ${p.colors.length} colors`;
    $("[data-palette-rename]").value = p.name;
    $("[data-palette-rename]").disabled = !!p.preset;
    $("[data-palette-rename]").hidden = !!p.preset;
    $("[data-editor-title]").hidden = !p.preset;
    $(".palette-dialog").classList.toggle("is-preset", !!p.preset);
    $(".manager-swatches").innerHTML = p.colors
      .map((c, i) => swatchButton(c, i, editorIndex, true))
      .join("");
    $(".color-preview").style.background = p.colors[editorIndex];
    $("[data-hex-edit]").value = p.colors[editorIndex].toUpperCase();
    $("[data-hex-edit]").disabled = !!p.preset;
    for (const action of [
      "swatch-left",
      "swatch-right",
      "remove-swatch",
      "add-swatch",
      "delete-palette",
    ])
      $(`[data-action="${action}"]`) && ($(`[data-action="${action}"]`).disabled =
        !!p.preset ||
        (action === "remove-swatch" && p.colors.length === 1) ||
        (action === "add-swatch" && p.colors.length >= 256) ||
        (action === "swatch-left" && editorIndex === 0) ||
        (action === "swatch-right" && editorIndex === p.colors.length - 1));
    $("[data-editor-status]").textContent = "Applies to new voxels";
    const use = $('[data-action="use-palette"]');
    use.disabled = p.id === activePalette;
    use.textContent = use.disabled ? "In use" : "Use palette";
    $('.duplicate-preset').hidden = !p.preset;
    $('[data-swatch-position]').textContent = `Color ${editorIndex + 1}`;
    $('[data-menu="SwatchEdit"]').disabled = !!p.preset;
  }
  function openManager() {
    closeFloating();
    editingPalette = activePalette;
    editorIndex = swatchIndex;
    renderManager();
    $(".app").classList.add("palette-sheet-open");
    openDialog($(".palette-dialog"));
  }
  function updatePalette() {
    persist();
    renderPalette();
    renderManager();
  }
  function createPalette(copy = false) {
    const source = editing();
    const p = {
      id: uid("palette"),
      name: copy ? source.name + " copy" : "Untitled palette",
      colors: copy ? [...source.colors] : ["#829a63"],
    };
    custom.push(p);
    editingPalette = p.id;
    editorIndex = 0;
    updatePalette();
    $("[data-palette-rename]").focus();
    $("[data-palette-rename]").select();
  }
  function exportText(format, p = editing()) {
    if (format === "hex")
      return p.colors.map((c) => c.slice(1).toUpperCase()).join("\n") + "\n";
    return (
      `GIMP Palette\nName: ${p.name.replace(/[\r\n]/g, " ")}\nColumns: 8\n#\n` +
      p.colors
        .map(
          (c) =>
            c
              .slice(1)
              .match(/../g)
              .map((n) => String(parseInt(n, 16)).padStart(3, " "))
              .join(" ") +
            "\t" +
            c.toUpperCase(),
        )
        .join("\n") +
      "\n"
    );
  }
  function exportPalette(format) {
    const p = $(".palette-dialog").open ? editing() : active();
    const text = exportText(format, p);
    window.__lastPaletteExport = { format, text };
    download(
      p.name.replace(/[^a-z0-9_-]/gi, "-").toLowerCase() + "." + format,
      text,
    );
    notify(`Exported ${p.name}.${format}`);
  }
  function parsePalette(text, extension) {
    const lines = text.replace(/^\uFEFF/, "").split(/\r?\n/);
    let name = "Imported palette",
      colors = [];
    if (extension === "gpl") {
      if (lines.shift()?.trim() !== "GIMP Palette")
        throw Error("Expected a GIMP Palette header.");
      for (const line of lines) {
        const value = line.trim();
        if (!value || value.startsWith("#") || value.startsWith("Columns:"))
          continue;
        if (value.startsWith("Name:")) {
          name = value.slice(5).trim() || name;
          continue;
        }
        const m = value.match(/^(\d+)\s+(\d+)\s+(\d+)(?:\s|$)/);
        if (!m || m.slice(1).some((c) => Number(c) > 255))
          throw Error("Invalid RGB row in .gpl file.");
        colors.push(
          "#" +
            m
              .slice(1)
              .map((c) => Number(c).toString(16).padStart(2, "0"))
              .join(""),
        );
      }
    } else {
      for (const line of lines) {
        const value = line.trim();
        if (!value || value.startsWith(";") || value.startsWith("//")) continue;
        if (!/^#?[a-f\d]{6}$/i.test(value))
          throw Error("Use one six-digit hex color per line.");
        colors.push("#" + value.replace(/^#/, "").toLowerCase());
      }
    }
    if (!colors.length || colors.length > 256)
      throw Error("Palettes need 1–256 colors.");
    return { id: uid("palette"), name: name.slice(0, 60), colors };
  }
  $("[data-palette-file]").addEventListener("change", async (e) => {
    const file = e.target.files[0];
    if (!file) return;
    try {
      if (file.size > 65536) throw Error("Palette file is too large.");
      const ext = file.name.split(".").pop().toLowerCase();
      if (!["hex", "gpl"].includes(ext))
        throw Error("Choose a .hex or .gpl file.");
      const p = parsePalette(await file.text(), ext);
      if (ext === "hex") p.name = file.name.replace(/\.hex$/i, "").slice(0, 60);
      custom.push(p);
      editingPalette = p.id;
      editorIndex = 0;
      updatePalette();
      notify(`Imported ${p.colors.length} colors`);
    } catch (error) {
      notify(error.message);
    } finally {
      e.target.value = "";
    }
  });
  $("[data-palette-rename]").addEventListener("change", (e) => {
    if (editing().preset) return;
    const name = e.target.value.trim();
    if (!name) {
      notify("Give this palette a name.");
      renderManager();
      return;
    }
    editing().name = name;
    updatePalette();
  });
  $("[data-hex-edit]").addEventListener("change", (e) => {
    if (editing().preset) return;
    const value = e.target.value.trim();
    if (!/^#?[a-f\d]{6}$/i.test(value)) {
      notify("Enter a six-digit hex color.");
      e.target.value = editing().colors[editorIndex];
      return;
    }
    editing().colors[editorIndex] = "#" + value.replace(/^#/, "").toLowerCase();
    updatePalette();
  });

  const tools = [
    ["place", "Place", "B"],
    ["erase", "Erase", "E"],
    ["paint", "Paint", "P"],
    ["fill", "Fill", "G"],
    ["box", "Box", "U"],
    ["picker", "Picker", "I"],
  ];
  function chooseTool(id) {
    if (historyPreview) return;
    tool = id;
    const name = tools.find((t) => t[0] === id)?.[1] || "Scatter moss";
    $("[data-current-tool]").textContent = name;
    $("[data-tool-icon]").innerHTML = icons[id] || icons.agent;
    $(".tool-picker").setAttribute("data-selected-tool", id);
    $(".tool-settings").hidden = id !== "scatter";
    $('.sidebar').classList.toggle('has-tool-settings', id === 'scatter');

  }
  function setFollow(value) {
    follow = value;
    $("[data-follow]").setAttribute("aria-pressed", String(value));
    $("[data-follow-state]").textContent = value ? "On" : "Off";
    if (value) window.__setViewAngle("front");
  }
  document.addEventListener("manual-camera", () => {
    setFollow(false);
  });
  function pause() {
    paused = !paused;
    $("[data-pause]").setAttribute("aria-pressed", String(paused));
    $("[data-pause]").setAttribute(
      "aria-label",
      paused ? "Resume agent" : "Pause agent",
    );
    $("[data-session-state]").textContent = paused ? "Paused" : "Live";
    $("[data-running-title]").textContent = paused
      ? "Session paused"
      : "Checking silhouettes";
    $("[data-running-label]").textContent = paused
      ? "Ready to resume"
      : "render_views · 3 angles";
    $("[data-pause]").innerHTML = paused
      ? '<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="m8 4 12 8-12 8Z"/></svg><span>Resume</span>'
      : '<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M8 5v14M16 5v14"/></svg><span>Pause</span>';
  }
  const reverted = [];
  function syncSteps() {
    for (const id of ["17", "18"]) {
      const c = window.__getChange(id),
        el = $(`[data-step="${id}"]`);
      el.classList.toggle("is-undone", c.state === "undone");
      el.querySelector(".step-status").innerHTML = c.state === "undone" ? icons.undo : icons.check;
      el.querySelector("[data-step-undo] span").textContent =
        c.state === "undone" ? "Redo step" : "Undo step";
      $(`[data-step-coordinates="${id}"]`).textContent = c.added
        .map((p) => p.join(", "))
        .join("\n");
    }
  }
  function toggleStep(id) {
    if (historyPreview) return;
    if (!$(".history-panel").hidden) closeHistory();
    const undone = window.__toggleStep(id);
    const i = reverted.indexOf(id);
    if (i >= 0) reverted.splice(i, 1);
    if (undone) reverted.push(id);
    syncSteps();
  }
  function undo() {
    if (historyPreview) return;
    const id = ["18", "17"].find(
      (id) => window.__getChange(id).state === "applied",
    );
    if (id) toggleStep(id);
    else notify("No earlier reversible session step.");
  }
  function redo() {
    if (historyPreview) return;
    const id = reverted.at(-1);
    if (id) toggleStep(id);
    else notify("Nothing to redo.");
  }

  // DOM menus share one command registry, including context menus and command search.
  const item = (label, action, shortcut = "", iconName = "") => ({
    label,
    action,
    shortcut,
    icon: iconName,
  });
  const sub = (label, menu) => ({ label, menu });
  const preview = (label) => () => notify(label + " · concept preview");
  const checked = (entry, value, radio = false) => ({...entry, checked: value, radio});
  const toolConfig = {density: 24, seed: 42};
  const menuItems = (name) => {
    if (name.startsWith('Version:')) {
      const id = name.slice(8);
      return [item('Restore version', () => versionAction('restore', id), '', 'undo'),
        item('Compare with current', () => versionAction('compare', id)),
        null, item('Branch from version', () => versionAction('branch', id), '', 'branch')];
    }
    return ({
      File: [
        item("New project", preview("New project"), "⌘N"),
        item("Open…", preview("Open .voxl"), "⌘O"),
        null,
        item("Save", preview("Save .voxl"), "⌘S"),
        item("Save as…", preview("Save as .voxl"), "⇧⌘S"),
        null,
        sub("Export", "Export"),
      ],
      Export: [
        item("PNG image", () => {
          const a = document.createElement("a");
          a.download = "woodland-cottage.png";
          a.href = $(".viewport>canvas").toDataURL();
          a.click();
        }),
        item("glTF binary (.glb)", preview("GLB export")),
      ],
      Edit: [
        item("Undo", undo, "⌘Z", "undo"),
        {...item("Redo", redo, "⇧⌘Z", "redo"), disabled: !reverted.length},
        null,
        item("Search commands", "commands", "⌘K", "search"),
      ],
      View: [
        item("Frame model", "frame", "Home", "focus"),
        sub("Zoom", "Zoom"),
        checked(item("Show grid", "grid", "", "grid"), $("[data-toggle-grid]").getAttribute("aria-pressed") === "true"),
        null,
        item("Comments", "comments", "", "comment"),
        checked(item("Comment pins", () => {
          pinsVisible = !pinsVisible;
          window.__drawVoxelScene();
        }), pinsVisible),
        item("History", "history", "", "clock"),
        null,
        item("Toggle light / dark", "theme", "", "sun"),
      ],
      Zoom: [75, 100, 125, 150, 175].map((n) =>
        item(`${n}%`, () => {
          const sel = $("[data-zoom-select]");
          sel.value = n;
          sel.dispatchEvent(new Event("change"));
        }),
      ),
      Tools: [
        ...tools.map(([id, label, key]) =>
          checked(item(label, () => chooseTool(id), key, id), tool === id, true),
        ),
        null,
        sub("Mirror", "Mirror"),
        null,
        { label: "Custom", heading: true },
        {
          ...checked(item("Scatter moss", () => chooseTool("scatter"), "", "agent"), tool === "scatter", true),
          badge: "Agent",
        },
      ],
      ToolPicker: [
        ...tools.map(([id, label, key]) =>
          checked(item(label, () => chooseTool(id), key, id), tool === id, true),
        ),
        null,
        { label: "Custom", heading: true },
        {
          ...checked(item("Scatter moss", () => chooseTool("scatter"), "", "agent"), tool === "scatter", true),
          badge: "Agent",
        },
      ],
      Mirror: ["X", "Y", "Z"].map((axis) =>
        checked(item(`Mirror ${axis}`, () => {
          mirrors.has(axis) ? mirrors.delete(axis) : mirrors.add(axis);
          notify(
            `Mirror ${axis} ${mirrors.has(axis) ? "on" : "off"} · preview`,
          );
        }), mirrors.has(axis)),
      ),
      Layers: [
        item("New layer", preview("New layer")),
        item("Rename layer", preview("Rename " + selectedLayer)),
        item("Duplicate layer", preview("Duplicate " + selectedLayer)),
        null,
        item(
          "Toggle visibility",
          () => $(`[data-layer-eye="${selectedLayer}"]`).click(),
          "",
          "eye",
        ),
        item(
          `${lockedLayers.has(selectedLayer) ? "Unlock" : "Lock"} layer`,
          () => toggleLayerLock(selectedLayer),
          "",
          "lock",
        ),
        null,
        item("Delete layer", preview("Delete " + selectedLayer), "", "trash"),
      ],
      Palette: [
        item("Manage palettes…", "palettes", "", "swatches"),
        item("New palette…", () => {
          openManager();
          createPalette();
        }),
        sub("Switch palette", "PaletteSwitcher"),
        null,
        item("Import .hex / .gpl…", "import-palette", "", "import"),
        sub("Export palette", "PaletteExport"),
      ],
      PaletteManage: [
        item("Duplicate palette", "duplicate-palette", "", "copy"),
        null, item("Import .hex / .gpl…", "import-palette", "", "import"),
        sub("Export palette", "PaletteExport"),
        null, {...item("Delete palette", "delete-palette", "", "trash"), disabled: !!editing().preset},
      ],
      SwatchEdit: [
        {...item("Move left", "swatch-left"), disabled: editorIndex === 0},
        {...item("Move right", "swatch-right"), disabled: editorIndex === editing().colors.length - 1},
        null, {...item("Remove color", "remove-swatch", "", "trash"), disabled: editing().colors.length === 1},
      ],
      PaletteExport: [
        item("Hex colors (.hex)", "export-hex"),
        item("GIMP palette (.gpl)", "export-gpl"),
      ],
      PaletteSwitcher: [
        { label: "Your palettes", heading: true },
        ...custom.map((p) => ({
          ...checked(item(p.name, () => selectPalette(p.id)), p.id === activePalette, true),
          ramp: p.colors,
        })),
        null,
        { label: "Preset library", heading: true },
        ...presets.map((p) => ({
          ...checked(item(p.name, () => selectPalette(p.id)), p.id === activePalette, true),
          ramp: p.colors,
        })),
        null,
        item("Manage palettes…", "palettes"),
      ],
      Agent: [
        item(paused ? "Resume session" : "Pause session", "pause"),
        checked(item("Follow agent", "follow", "⇧F", "camera"), follow),
        null,
        item("Refresh captured views", "refresh", "", "refresh"),
        item("Inspect latest step", () => {
          $('[data-tab="session"]').click();
          $('[data-step="18"]').open = true;
        }),
        item("History", "history", "", "clock"),
      ],
      Captures: [
        item("Refresh captures", "refresh", "", "refresh"),
        ...["front", "right", "back"].map((v) =>
          item(`View ${v}`, () => {
            setFollow(false);
            window.__setViewAngle(v);
          }),
        ),
      ],
      Extensions: [
        { label: "Installed", heading: true },
        {...checked(item("Scatter moss", () => chooseTool("scatter"), "", "agent"), tool === "scatter", true), badge: "Agent"},
        null,
        item("Manage extensions…", preview("Extensions · local tool plugins")),
        item(
          "Create a tool via MCP…",
          preview("tools.register · runtime tool authoring"),
        ),
      ],
      Help: [
        item("Search commands", "commands", "⌘K", "search"),
        item("Keyboard shortcuts", () =>
          notify(
            "B Place · E Erase · P Paint · G Fill · U Box · I Picker · ⇧F Follow · ⌘K Commands",
          ),
        ),
        item("About this preview", () =>
          notify(
            "Offline design fixture · no connected MCP or sharing service",
          ),
        ),
      ],
      ViewportContext: [
        item("Add comment here", () => addCommentAtContext(), "", "comment"),
        item("Frame model", "frame", "Home"),
        sub("Camera", "Captures"),
        sub("Zoom", "Zoom"),
        null,
        checked(item("Show grid", "grid"), $("[data-toggle-grid]").getAttribute("aria-pressed") === "true"),
        item("History", "history"),
      ],
    })[name] || [];
  };
  function toggleLayerLock(name) {
    if (historyPreview) return;
    lockedLayers.has(name) ? lockedLayers.delete(name) : lockedLayers.add(name);
    const button = $(`[data-layer-lock="${name}"]`);
    button.setAttribute('aria-pressed', String(lockedLayers.has(name)));
    button.innerHTML = icons[lockedLayers.has(name) ? 'lock' : 'unlock'];
  }
  function selectCapture(angle) {
    $$('[data-capture-angle]').forEach(b => b.setAttribute('aria-pressed', String(b.dataset.captureAngle === angle)));
    $$('[data-view-angle]').forEach(b => b.dataset.captureVisible = String(b.dataset.viewAngle === angle));
  }
  function selectPalette(id) {
    activePalette = id;
    swatchIndex = 0;
    persist();
    renderPalette();
    notify(`Using ${active().name} for new voxels`);
  }
  function closeMenus(restore = false) {
    $(".menu-root").replaceChildren();
    menuChain = [];
    $$("[data-menu][aria-expanded=true]").forEach((el) =>
      el.setAttribute("aria-expanded", "false"),
    );
    if (restore) menuTrigger?.focus();
  }
  function showMenu(name, trigger, point = null, level = 0) {
    if (level === 0) {
      closeMenus();
      menuTrigger = trigger;
      trigger?.setAttribute("aria-expanded", "true");
    } else {
      menuChain.splice(level).forEach((m) => {m.trigger?.setAttribute('aria-expanded','false'); m.element.remove();});
      trigger?.setAttribute('aria-expanded','true');
    }
    const entries = menuItems(name).map(entry => {
      if (!entry || entry.heading) return entry;
      const editingMenu = ['Tools','ToolPicker','Mirror','Layers'].includes(name);
      const editingItem = name === 'Edit' && ['Undo','Redo'].includes(entry.label) ||
        name === 'File' && ['New project','Open…','Save','Save as…'].includes(entry.label) ||
        name === 'Extensions' && entry.label === 'Scatter moss';
      return {...entry, disabled: entry.disabled || historyPreview && (editingMenu || editingItem)};
    }),
      element = document.createElement("div");
    element.className = "dropdown-menu";
    element.role = "menu";
    element.setAttribute("aria-label", name);
    element.dataset.menuName = name;
    element.innerHTML = entries
      .map((it, i) =>
        !it
          ? '<div role="separator" class="separator"></div>'
          : it.heading
            ? `<div class="menu-label">${esc(it.label)}</div>`
            : `<button role="${it.checked === undefined ? "menuitem" : it.radio ? "menuitemradio" : "menuitemcheckbox"}" ${it.checked === undefined ? "" : `aria-checked="${it.checked}"`} ${it.disabled ? 'disabled aria-disabled="true"' : ""} aria-label="${esc(it.label)}${it.badge ? ` ${it.badge}` : ""}" tabindex="-1" data-menu-index="${i}" ${it.menu ? 'aria-haspopup="menu" aria-expanded="false"' : ""}>${it.icon ? icons[it.icon] : ""}<span>${esc(it.label)}</span>${it.checked ? `<span class="menu-check">${icons.check}</span>` : ""}${it.shortcut ? `<kbd>${it.shortcut}</kbd>` : ""}${it.badge ? `<span class="menu-agent-badge">${it.badge}</span>` : ""}${
                it.ramp
                  ? `<span class="mini-ramp">${it.ramp
                      .slice(0, 8)
                      .map((c) => `<i style="background:${c}"></i>`)
                      .join("")}</span>`
                  : ""
              }${it.menu ? `<span class="menu-tail">${icons.chevron}</span>` : ""}</button>`,
      )
      .join("");
    const host = $('dialog[open]') || $('.app');
    if ($('.menu-root').parentElement !== host) host.append($('.menu-root'));
    $('.menu-root').append(element);
    if (['ToolPicker','Extensions'].includes(name)) {
      element.insertAdjacentHTML('afterbegin', `<label class="menu-search">${icons.search}<input data-menu-search aria-label="Search ${name === 'Extensions' ? 'extensions' : 'tools'}" placeholder="Search ${name === 'Extensions' ? 'extensions' : 'tools'}…" autocomplete="off"></label>`);
      element.insertAdjacentHTML('beforeend','<p class="menu-empty" hidden>No matches</p>');
      const input = element.querySelector('input');
      input.addEventListener('input', () => {
        const value = input.value.trim().toLowerCase();
        const rows = [...element.querySelectorAll('[data-menu-index]')];
        rows.forEach(row => row.hidden = !row.getAttribute('aria-label').toLowerCase().includes(value));
        element.querySelectorAll('.menu-label').forEach(label => {
          let next = label.nextElementSibling, visible = false;
          while (next && !next.matches('.menu-label,.separator')) {
            if (next.matches('[data-menu-index]') && !next.hidden) visible = true;
            next = next.nextElementSibling;
          }
          label.hidden = !visible;
        });
        element.querySelectorAll('.separator').forEach(el => el.hidden = !!value);
        element.querySelector('.menu-empty').hidden = rows.some(row => !row.hidden);
      });
    }
    const r = trigger?.getBoundingClientRect();
    let x = point?.x ?? (level ? r.right + 4 : r.left),
      y = point?.y ?? (level ? r.top : r.bottom + 5);
    const bounds = element.getBoundingClientRect();
    if (x + bounds.width > innerWidth - 8)
      x = level ? r.left - bounds.width - 4 : innerWidth - bounds.width - 8;
    y = Math.max(8, Math.min(y, innerHeight - bounds.height - 8));
    element.style.left = x + "px";
    element.style.top = y + "px";
    menuChain.push({ name, element, trigger });
    element.querySelector('[data-menu-search]')?.focus();
    element.addEventListener("click", (e) => {
      const el = e.target.closest("[data-menu-index]");
      if (!el || el.disabled) return;
      const it = entries[el.dataset.menuIndex];
      if (it.menu) {
        el.setAttribute("aria-expanded", "true");
        showMenu(it.menu, el, null, level + 1);
      } else {
        closeMenus();
        run(it.action);
      }
    });
    element.addEventListener("pointerover", (e) => {
      const el = e.target.closest("[data-menu-index]");
      if (!el || el.disabled) return;
      const it = entries[el.dataset.menuIndex];
      if (it.menu) {
        if (menuChain[level + 1]?.name !== it.menu) {
          el.setAttribute("aria-expanded", "true");
          showMenu(it.menu, el, null, level + 1, false);
        }
      } else {
        menuChain.splice(level + 1).forEach((m) => {m.trigger?.setAttribute('aria-expanded','false'); m.element.remove();});
      }
    });
    element.addEventListener("keydown", (e) => {
      const inSearch = e.target.matches('[data-menu-search]');
      if (/INPUT|SELECT/.test(e.target.tagName) && !inSearch && e.key !== "Escape") return;
      const buttons = [...element.querySelectorAll("button:not(:disabled):not([hidden])")],
        index = buttons.indexOf(document.activeElement);
      if (inSearch && !['ArrowDown','ArrowUp','Enter','Escape','Tab'].includes(e.key)) return;
      if (inSearch && e.key === 'Enter') { e.preventDefault(); buttons[0]?.click(); return; }
      if (["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) {
        e.preventDefault();
        const next =
          e.key === "Home"
            ? 0
            : e.key === "End"
              ? buttons.length - 1
              : (index + (e.key === "ArrowDown" ? 1 : -1) + buttons.length) %
                buttons.length;
        buttons[next]?.focus();
      }
      if (e.key === "ArrowRight") {
        e.preventDefault();
        const it = entries[document.activeElement.dataset.menuIndex];
        if (it?.menu) {
          document.activeElement.click();
          menuChain.at(-1).element.querySelector("button")?.focus();
        } else if (level === 0) cycleMenubar(1);
      }
      if (e.key === "ArrowLeft") {
        e.preventDefault();
        if (level) {
          menuChain.splice(level).forEach((m) => m.element.remove());
          trigger.focus();
          trigger.setAttribute("aria-expanded", "false");
        } else cycleMenubar(-1);
      }
      if (/INPUT|SELECT/.test(e.target.tagName) && !inSearch) return;
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        closeMenus(true);
      }
      if (e.key === "Tab") closeMenus();
    });
  }
  function cycleMenubar(direction) {
    const buttons = $$(".menubar>[data-menu]"),
      index = buttons.indexOf(menuTrigger);
    if (index >= 0) {
      const el = buttons[(index + direction + buttons.length) % buttons.length];
      showMenu(el.dataset.menu, el);
      menuChain[0].element.querySelector("button")?.focus();
    }
  }

  // Comment fixtures use model-space anchors, projected on every camera draw.
  let comments = [
      {
        id: 1,
        anchor: [8, 14, -1],
        layer: "Canopy",
        author: "Artist 01",
        text: "Could the canopy feel a little less symmetrical?",
        resolved: false,
        replies: [
          {
            author: "Canopy agent",
            text: "I softened the outer branches and added six lighter leaves. Check the right-side capture.",
            mcp: true,
          },
        ],
      },
      {
        id: 2,
        anchor: [-3, 16, 3],
        layer: "Roof",
        author: "Designer 02",
        text: "Keep this warm roof color. It balances the greens nicely.",
        resolved: false,
        replies: [],
      },
      {
        id: 3,
        anchor: [-2, 3, 8],
        layer: "Garden",
        author: "Artist 01",
        text: "A few mossy stones along this path?",
        resolved: false,
        replies: [],
      },
    ],
    activeThread = null,
    pinsVisible = true,
    contextPoint = null;
  function setTab(tab) {
    $$("[data-tab]").forEach((el) =>
      el.setAttribute("aria-selected", String(el.dataset.tab === tab)),
    );
    $$("[data-tab]").forEach(
      (el) => (el.tabIndex = el.dataset.tab === tab ? 0 : -1),
    );
    $("#session-panel").hidden = tab !== "session";
    $("#comments-panel").hidden = tab !== "comments";
    $('[data-action="comments"]').setAttribute(
      "aria-pressed",
      String(tab === "comments"),
    );
    if (tab === "session") {
      activeThread = null;
      $(".pin-thread").hidden = true;
    }
    renderComments();
    window.__drawVoxelScene();
  }
  const initials = author => author === "Artist 01" ? "A1" : author === "Designer 02" ? "D2" : "You";
  const messageHTML = (author, text, mcp = false) =>
    `<div class="thread-message"><div class="thread-meta"><span class="avatar ${mcp ? "agent-avatar" : ""}">${mcp ? icons.agent : esc(initials(author))}</span><strong>${esc(author)}</strong><time>${author === "You" ? "Now" : mcp ? "2m" : "3m"}</time></div><p>${esc(text)}</p>${mcp ? '<div class="mcp-note">MCP · comments.reply</div>' : ""}</div>`;
  function renderComments() {
    $("[data-comment-count]").textContent = comments.filter(c => !c.resolved).length;
    $(".comment-pins").innerHTML = comments
      .filter(c => !c.resolved || $("[data-comment-filter]").value !== "open")
      .map(c => `<button class="comment-pin ${c.resolved ? "resolved" : ""}" data-pin="${c.id}" aria-label="Comment ${c.id}: ${esc(c.layer)}" aria-pressed="${c.id === activeThread}">${c.id}</button>`).join("");
    const filter = $("[data-comment-filter]").value;
    const visible = comments.filter(c => c.id !== activeThread && (filter === "all" || c.resolved === (filter === "resolved")));
    $(".comment-list").innerHTML = visible.map(c => `<article class="comment-card"><button data-open-thread="${c.id}"><span class="comment-summary-heading"><span class="comment-number">${c.id}</span><strong>${esc(c.layer)}</strong><span>${esc(c.author)}</span></span><p>${esc(c.text || "New comment")}</p></button></article>`).join("") || (activeThread ? "" : '<p class="muted">No threads here.</p>');
    const c = comments.find(c => c.id === activeThread);
    $(".pin-thread").hidden = !c;
    $("#comments-panel").classList.toggle("thread-open", !!c);
    if (c) $(".pin-thread").innerHTML = `<div class="thread-heading"><span class="comment-number selected">${c.id}</span><strong>${esc(c.layer)}</strong><button data-resolve="${c.id}">${icons.check}${c.resolved ? "Reopen" : "Resolve"}</button><button class="icon-button" data-action="close-thread" aria-label="Close thread">${icons.close}</button></div>${c.draft ? "" : messageHTML(c.author, c.text)}${c.replies.map(r => messageHTML(r.author, r.text, r.mcp)).join("")}<form class="reply-form"><textarea aria-label="Reply to comment" placeholder="${c.draft ? "Write a comment…" : "Reply or mention…"}" required maxlength="500" rows="2"></textarea><button aria-label="Send reply" type="submit">${c.draft ? "Comment" : "Reply"} ↵</button></form>`;
  }
  function openThread(id) {
    setTab("comments");
    activeThread = Number(id);
    renderComments();
    window.__drawVoxelScene();
  }
  $("[data-comment-filter]").addEventListener("change", () => {
    activeThread = null;
    renderComments();
    window.__drawVoxelScene();
  });
  function addCommentAtContext() {
    const hit = window.__pickVoxel?.(contextPoint);
    if (!hit) {
      notify("Right-click a voxel to anchor a comment.");
      return;
    }
    comments.push({
      id: Math.max(...comments.map((c) => c.id)) + 1,
      anchor: [hit.x, hit.y, hit.z],
      layer: hit.layer,
      author: "You",
      text: "",
      draft: true,
      resolved: false,
      replies: [],
    });
    openThread(comments.at(-1).id);
    $(".reply-form textarea").focus();
  }
  window.__projectComments = (project, cubes, hidden) => {
    for (const el of $$("[data-pin]")) {
      const c = comments.find((c) => c.id === Number(el.dataset.pin));
      const [x, y, depth] = project(c.anchor.map((v) => v + 0.5));
      const anchorExists = cubes.has(c.anchor.join(","));
      el.hidden =
        !pinsVisible ||
        hidden.has(c.layer) ||
        !anchorExists ||
        depth < 0 ||
        x < 20 ||
        y < 36 ||
        x > $(".viewport").clientWidth - 20 ||
        y > $(".viewport").clientHeight - 20;
      el.style.left = x + "px";
      el.style.top = y + "px";
      // The thread is docked outside the canvas. Only the anchored pin is
      // highlighted; no connector or voxel wireframe can cross the artwork.
    }
    $("[data-zoom-label]").textContent = $("[data-zoom-select]").value + "%";
  };
  document.addEventListener("submit", (e) => {
    if (e.target.matches(".reply-form")) {
      e.preventDefault();
      const input = e.target.querySelector("textarea"),
        text = input.value.trim();
      if (!text) return;
      const thread = comments.find((c) => c.id === activeThread);
      if (thread.draft) {
        thread.text = text;
        thread.draft = false;
      } else thread.replies.push({ author: "You", text });
      renderComments();
      window.__drawVoxelScene();
      $(".reply-form textarea").focus();
    }
  });

  let permission = "comment", accessScope = 'link';
  function updateShare() {
    const restricted = accessScope === 'restricted';
    $$('[data-permission]').forEach(b => { b.disabled = restricted; b.setAttribute('aria-pressed', String(b.dataset.permission === permission)); });
    $('.permission-description').textContent = restricted ? 'Only listed people and agents can open this link.' : ({view:'View the model and history. No comments or edits.', comment:'View and comment. No voxel edits.', remix:'View, comment and create an independent copy.'})[permission];
    $('.access-rule').textContent = restricted ? 'Only their individual permission applies.' : 'Link or individual permission, whichever is higher.';
    $('.link-field input').value = `https://voxelite.example/s/woodland?${restricted ? 'scope=restricted' : `access=${permission}`}`;
    $('.copy-link span').textContent = 'Copy link';
  }
  $('[data-access-scope]').addEventListener('change', e => {accessScope = e.target.value; updateShare(); $('.share-feedback').textContent = '';});
  $$('[data-member-permission]').forEach(el => el.addEventListener('change', () => $('.share-feedback').textContent = ''));

  function openShare() {
    closeFloating();
    $(".share-popover").hidden = false;
    $('[data-action="share"]').setAttribute("aria-expanded", "true");
    $(`[data-permission="${permission}"]`).focus();
  }
  async function copyLink() {
    const input = $(".link-field input");
    try {
      await navigator.clipboard.writeText(input.value);
      $(".copy-link span").textContent = "Copied";
      $(".share-feedback").textContent = "";
    } catch {
      input.focus();
      input.select();
      $(".share-feedback").textContent = "Press ⌘C or Ctrl+C to copy the selected link.";
    }
  }

  // Timeline snapshots are reconstructed from the procedural cottage's actual cells.
  const historyNames = [
    "Empty project",
    "Ground plane",
    "Cottage walls",
    "Warm tiled roof",
    "Garden & path",
    "Tree structure",
    "Full canopy",
    "Shape the canopy",
    "Leaf detail",
  ];
  let liveSnapshot,
    timelineStep = 8,
    branch = "main";
  const branches = new Map([
    [
      "main",
      {
        name: "main",
        parent: null,
        sourceVersion: null,
        snapshot: window.__sceneSnapshot(),
      },
    ],
  ]);
  const versions = [
    {
      id: "v1",
      name: "Cottage study",
      step: 4,
      snapshot: window.__historySnapshot(4),
    },
    {
      id: "v2",
      name: "Canopy pass",
      step: 8,
      snapshot: window.__sceneSnapshot(),
    },
  ];
  function setHistoryPreview(value) {
    historyPreview = value;
    $('.tool-picker').disabled = value;
    $('[data-current-tool]').textContent = value ? 'Read only' : tools.find(t => t[0] === tool)?.[1] || 'Scatter moss';
    $('[data-tool-icon]').innerHTML = value ? icons.lock : icons[tool] || icons.agent;
    $('.tool-picker').title = value ? 'Editing is disabled during history preview' : 'Choose a tool';
    $$('[data-step-undo],[data-tool-density],[data-tool-seed],[data-layer-lock]').forEach(b => b.disabled = value);
    $('.app').classList.toggle('previewing-history', value);
  }
  function renderHistory() {
    $('[data-history-position]').textContent = timelineStep === 0 ? 'Initial state' : `Step ${timelineStep} / 8`;
    $('[data-history-step-title]').textContent = historyNames[timelineStep];
    $('[data-timeline]').value = timelineStep;
    $('[data-timeline]').setAttribute('aria-valuetext', `${timelineStep === 0 ? 'Initial state' : `Step ${timelineStep} of 8`}: ${historyNames[timelineStep]}`);
    $('[data-branch-name]').textContent = branch;
    $('[data-action="history-prev"]').disabled = timelineStep === 0;
    $('[data-action="history-next"]').disabled = timelineStep === 8;
    $('.history-event-list').innerHTML = historyNames.map((name, step) => `<button class="history-event ${step === timelineStep ? 'active' : ''}" data-history-tick="${step}" aria-current="${step === timelineStep}"><span class="event-origin"><i class="${step === 0 ? 'initial-marker' : step <= 4 ? 'human-marker' : 'agent-marker'}"></i></span><span class="event-title">${name}</span><small>${step === 0 ? 'Artist 01' : step <= 2 ? 'Artist 01' : step <= 4 ? 'Designer 02' : 'Canopy agent'}</small></button>`).join('');
    $('.history-event.active')?.scrollIntoView({block:'nearest'});
    $('.version-list').innerHTML = versions.map(v => `<article class="version-item"><div><h3>${esc(v.name)}</h3><p class="version-meta">${v.snapshot.length.toLocaleString()} voxels · Step ${v.step}</p></div><button class="icon-button" data-menu="Version:${v.id}" aria-haspopup="menu" aria-expanded="false" aria-label="Actions for ${esc(v.name)}">${icons.more}</button></article>`).join('');
    const isPreview = window.__sceneSnapshot().map(c => `${c.x},${c.y},${c.z}`).join('|') !== liveSnapshot?.map(c => `${c.x},${c.y},${c.z}`).join('|');
    setHistoryPreview(isPreview);
    $('[data-action="save-version"]').innerHTML = icons.plus + (isPreview ? 'Save preview' : 'Save this state');
    $('[data-version-save-context]').textContent = isPreview ? 'Saves the previewed geometry at step ' + timelineStep + '.' : 'Saves the latest document state.';
    $('.history-preview-banner').hidden = !isPreview;
    $('.history-preview-banner span').textContent = `Read-only preview · ${timelineStep === 0 ? 'Initial state' : `Step ${timelineStep} / 8`} · ${historyNames[timelineStep]}`;
    $('.viewport').classList.toggle('historical-preview', isPreview);
  }
  function openHistory() {
    closeFloating();
    liveSnapshot = window.__sceneSnapshot();
    timelineStep = window.__currentHistoryStep ?? 8;
    $(".history-panel").hidden = false;
    $(".workspace").classList.add("history-open");
    $('[data-action="history"]').setAttribute("aria-expanded", "true");
    renderHistory();
    $("[data-timeline]").focus();
  }
  function scrub(step) {
    timelineStep = Number(step);
    window.__setHistory(timelineStep);
    syncSteps();
    renderHistory();
  }
  function closeHistory() {
    if (!$(".history-panel").hidden && liveSnapshot) {
      window.__loadSnapshot(liveSnapshot);
      syncSteps();
    }
    setHistoryPreview(false);
    $(".history-panel").hidden = true;
    $(".workspace").classList.remove("history-open");
    $(".history-preview-banner").hidden = true;
    $(".viewport").classList.remove("historical-preview");
    $('[data-action="history"]').setAttribute("aria-expanded", "false");
    window.__compareScene(null);
    $(".compare-badge").hidden = true;
    $(".version-form").hidden = true;
  }
  $("[data-timeline]").addEventListener("input", (e) => scrub(e.target.value));
  $(".version-form").addEventListener("submit", (e) => {
    e.preventDefault();
    const input = e.target.querySelector("input"),
      name = input.value.trim();
    if (!name) return;
    versions.unshift({
      id: uid("version"),
      name,
      step: timelineStep,
      snapshot: window.__sceneSnapshot(),
    });
    input.value = "";
    e.target.hidden = true;
    renderHistory();
    notify("Named version saved");
  });
  function versionAction(action, id) {
    const v = versions.find((v) => v.id === id);
    if (action === "compare") {
      const counts = window.__compareScene(v.snapshot);
      $(".compare-badge").innerHTML =
        `<span>Compared with ${esc(v.name)}</span><code>+${counts.added} / −${counts.removed}</code><span class="muted">Solid added · dashed removed</span><button data-action="clear-compare">Done</button>`;
      $(".compare-badge").hidden = false;
      return;
    }
    if (action === "branch") {
      branches.get(branch).snapshot = liveSnapshot;
      const parent = branch;
      const base = v.name.toLowerCase().replace(/[^a-z0-9]+/g, "-") + "-branch";
      branch = base;
      for (let suffix = 2; branches.has(branch); suffix++)
        branch = `${base}-${suffix}`;
      branches.set(branch, {
        name: branch,
        parent,
        sourceVersion: v.id,
        snapshot: v.snapshot,
      });
    }
    window.__loadSnapshot(v.snapshot, true);
    timelineStep = v.step;
    liveSnapshot = window.__sceneSnapshot();
    branches.get(branch).snapshot = liveSnapshot;
    renderHistory();
    syncSteps();
    notify(
      action === "branch"
        ? "Created branch from version"
        : "Version restored",
    );
  }
  function closeFloating() {
    closeMenus();
    $(".share-popover").hidden = true;
    $('[data-action="share"]').setAttribute("aria-expanded", "false");
    closeHistory();
  }
  function openDialog(dialog) {
    modalFocus =
      document.activeElement === document.body
        ? menuTrigger
        : document.activeElement;
    closeMenus();
    dialog.showModal();
  }
  for (const dialog of $$("dialog")) {
    dialog.addEventListener("close", () => {if (dialog.matches(".palette-dialog")) $(".app").classList.remove("palette-sheet-open"); closeMenus(); $('.app').append($('.menu-root')); modalFocus?.focus();});
    dialog.addEventListener('cancel', e => {if (menuChain.length) {e.preventDefault(); closeMenus(true);}});
  }

  const actions = {
    commands: openCommands,
    palettes: openManager,
    comments: () =>
      setTab($("#comments-panel").hidden ? "comments" : "session"),
    history: () =>
      $(".history-panel").hidden ? openHistory() : closeHistory(),
    share: () => ($(".share-popover").hidden ? openShare() : closeFloating()),
    "copy-link": copyLink,
    follow: () => setFollow(!follow),
    pause,
    frame: () => {
      $("[data-reset-view]").click();
    },
    grid: () => $("[data-toggle-grid]").click(),
    theme: () =>
      (document.documentElement.dataset.theme =
        document.documentElement.dataset.theme === "light" ? "dark" : "light"),
    refresh: () => {
      window.__captureViews();
      notify("Captured 3 views");
    },
    "create-palette": () => createPalette(),
    "duplicate-palette": () => createPalette(true),
    "use-palette": () => {
      selectPalette(editingPalette);
      $(".palette-dialog").close();
    },
    "delete-palette": () => {
      if (editing().preset) return;
      const id = editingPalette;
      custom = custom.filter((p) => p.id !== id);
      if (activePalette === id) activePalette = palettes()[0].id;
      editingPalette = activePalette;
      editorIndex = 0;
      updatePalette();
    },
    "swatch-left": () => moveSwatch(-1),
    "swatch-right": () => moveSwatch(1),
    "remove-swatch": () => {
      const p = editing();
      if (p.preset || p.colors.length === 1) return;
      p.colors.splice(editorIndex, 1);
      updatePalette();
    },
    "add-swatch": () => {
      const p = editing();
      if (p.preset || p.colors.length >= 256) return;
      p.colors.push("#829a63");
      editorIndex = p.colors.length - 1;
      updatePalette();
    },
    "export-hex": () => exportPalette("hex"),
    "export-gpl": () => exportPalette("gpl"),
    "import-palette": () => {
      if (!$(".palette-dialog").open) openManager();
      $("[data-palette-file]").click();
    },
    "close-thread": () => {
      activeThread = null;
      renderComments();
      window.__drawVoxelScene();
    },
    "history-prev": () => scrub(Math.max(0, timelineStep - 1)),
    "history-next": () => scrub(Math.min(8, timelineStep + 1)),
    "history-live": () => {
      window.__loadSnapshot(liveSnapshot);
      timelineStep = window.__currentHistoryStep ?? 8;
      renderHistory();
      syncSteps();
    },
    "save-version": () => {
      $(".version-form").hidden = false;
      $(".version-form input").focus();
    },
    "clear-compare": () => {
      window.__compareScene(null);
      $(".compare-badge").hidden = true;
    },
  };
  function moveSwatch(direction) {
    const p = editing(),
      next = editorIndex + direction;
    if (p.preset || next < 0 || next >= p.colors.length) return;
    [p.colors[editorIndex], p.colors[next]] = [
      p.colors[next],
      p.colors[editorIndex],
    ];
    editorIndex = next;
    updatePalette();
  }
  function run(action) {
    if (typeof action === "function") action();
    else actions[action]?.();
  }

  const commands = [
    ...tools.map(([id, label, key]) =>
      checked(item(label, () => chooseTool(id), key, id), tool === id, true),
    ),
    item("Manage palettes", "palettes", "", "swatches"),
    item("Version history", "history", "", "clock"),
    item("Comments", "comments", "", "comment"),
    item("Share project", "share", "", "link"),
    item("Frame model", "frame", "Home", "focus"),
    item("Follow agent", "follow", "⇧F", "camera"),
    item("Refresh captures", "refresh", "", "refresh"),
    item("Toggle theme", "theme", "", "sun"),
    item("Scatter moss", () => chooseTool("scatter"), "", "agent"),
  ];
  let commandIndex = 0,
    filteredCommands = [];
  function renderCommands() {
    const value = $(".command-search input").value.toLowerCase();
    filteredCommands = commands.filter((c) =>
      c.label.toLowerCase().includes(value) && !(historyPreview && (tools.some(t => t[1] === c.label) || c.label === 'Scatter moss')),
    );
    commandIndex = Math.max(
      0,
      Math.min(commandIndex, filteredCommands.length - 1),
    );
    $(".command-results").innerHTML =
      filteredCommands
        .map(
          (c, i) =>
            `<button id="cmd-${i}" role="option" aria-selected="${i === commandIndex}" tabindex="-1" data-command="${i}">${icons[c.icon] || ""}<span>${c.label}</span><kbd>${c.shortcut}</kbd></button>`,
        )
        .join("") || '<p class="command-empty">No commands found.</p>';
    if (filteredCommands.length)
      $(".command-search input").setAttribute(
        "aria-activedescendant",
        `cmd-${commandIndex}`,
      );
    else $(".command-search input").removeAttribute("aria-activedescendant");
  }
  function openCommands() {
    if (historyPreview) { closeMenus(); $(".share-popover").hidden = true; }
    else closeFloating();
    commandIndex = 0;
    $(".command-search input").value = "";
    renderCommands();
    openDialog($(".command-dialog"));
    $(".command-search input").focus();
  }
  $(".command-search input").addEventListener("input", () => {
    commandIndex = 0;
    renderCommands();
  });
  $(".command-dialog").addEventListener("keydown", (e) => {
    if (["ArrowUp", "ArrowDown"].includes(e.key)) {
      e.preventDefault();
      if (filteredCommands.length)
        commandIndex =
          (commandIndex +
            (e.key === "ArrowDown" ? 1 : -1) +
            filteredCommands.length) %
          filteredCommands.length;
      renderCommands();
      $(`[data-command="${commandIndex}"]`)?.scrollIntoView({
        block: "nearest",
      });
    }
    if (e.key === "Enter") {
      e.preventDefault();
      $(`[data-command="${commandIndex}"]`)?.click();
    }
  });
  document.addEventListener("click", (e) => {
    const el = e.target.closest("button");
    if (!el) return;
    if (el.dataset.menu) {
      const open = el.getAttribute("aria-expanded") === "true";
      if (open) closeMenus(true);
      else showMenu(el.dataset.menu, el);
      return;
    }
    if (el.dataset.action) {
      run(el.dataset.action);
      return;
    }
    if (el.dataset.close) {
      const id = el.dataset.close;
      if (id === "palettes") $(".palette-dialog").close();
      else if (id === "history") {
        closeHistory();
        $('[data-action="history"]').focus();
      } else {
        closeFloating();
        $('[data-action="share"]').focus();
      }
      return;
    }
    if (el.dataset.tab) {
      setTab(el.dataset.tab);
      return;
    }
    if (el.dataset.editPalette) {
      editingPalette = el.dataset.editPalette;
      editorIndex = 0;
      renderManager();
      return;
    }
    if (el.hasAttribute("data-swatch")) {
      swatchIndex = Number(el.dataset.swatch);
      renderPalette();
      $(`[data-swatch="${swatchIndex}"]`).focus();
      return;
    }
    if (el.hasAttribute("data-edit-swatch")) {
      editorIndex = Number(el.dataset.editSwatch);
      renderManager();
      $(`[data-edit-swatch="${editorIndex}"]`).focus();
      return;
    }
    if (el.dataset.stepUndo) {
      toggleStep(el.dataset.stepUndo);
      return;
    }
    if (el.dataset.viewAngle) {
      setFollow(false);
      window.__setViewAngle(el.dataset.viewAngle);
      return;
    }
    if (el.dataset.captureAngle) { selectCapture(el.dataset.captureAngle); return; }
    if (el.dataset.layerLock) { toggleLayerLock(el.dataset.layerLock); return; }
    if (el.dataset.selectLayer) {
      selectedLayer = el.dataset.selectLayer;
      $$("[data-select-layer]").forEach((b) => {
        b.setAttribute("aria-pressed", String(b === el));
        b.closest(".layer-row").classList.toggle("selected", b === el);
      });
      window.__updateSelection();
      return;
    }
    if (el.dataset.pin || el.dataset.openThread) {
      openThread(el.dataset.pin || el.dataset.openThread);
      return;
    }
    if (el.dataset.resolve) {
      const c = comments.find((c) => c.id === Number(el.dataset.resolve));
      c.resolved = !c.resolved;
      if (c.resolved) {
        activeThread = null;
      }
      renderComments();
      window.__drawVoxelScene();
      return;
    }
    if (el.dataset.permission) {
      permission = el.dataset.permission;
      updateShare();
      $('.share-feedback').textContent = '';
      return;
    }
    if (el.hasAttribute("data-history-tick")) {
      scrub(el.dataset.historyTick);
      return;
    }
    if (el.dataset.versionAction) {
      versionAction(el.dataset.versionAction, el.dataset.versionId);
      return;
    }
    if (el.hasAttribute("data-command")) {
      const c = filteredCommands[Number(el.dataset.command)];
      $(".command-dialog").close();
      run(c.action);
    }
  });
  document.addEventListener("pointerdown", (e) => {
    if (!e.target.closest(".menu-root,[data-menu]")) closeMenus();
    if (
      !$(".share-popover").hidden &&
      !e.target.closest('.share-popover,[data-action="share"]')
    ) {
      $(".share-popover").hidden = true;
      $('[data-action="share"]').setAttribute("aria-expanded", "false");
    }
  });
  $$(".menubar>[data-menu]").forEach((el) => {
    el.addEventListener("pointerenter", () => {
      if (menuChain.length && menuTrigger?.closest(".menubar"))
        showMenu(el.dataset.menu, el);
    });
    el.addEventListener("keydown", (e) => {
      if (["ArrowDown", "ArrowUp"].includes(e.key)) {
        e.preventDefault();
        showMenu(el.dataset.menu, el);
        menuChain[0].element.querySelector("button")?.focus();
      }
      if (["ArrowLeft", "ArrowRight"].includes(e.key)) {
        e.preventDefault();
        const buttons = $$(".menubar>[data-menu]"),
          i = buttons.indexOf(el);
        buttons[
          (i + (e.key === "ArrowRight" ? 1 : -1) + buttons.length) %
            buttons.length
        ].focus();
      }
    });
  });
  document.addEventListener("contextmenu", (e) => {
    const layer = e.target.closest("[data-layer]"),
      paletteArea = e.target.closest(".palette-section"),
      view = e.target.closest(".viewport");
    if (!layer && !paletteArea && !view) return;
    e.preventDefault();
    if (layer) layer.querySelector("[data-select-layer]").click();
    if (view) {
      const r = $(".viewport").getBoundingClientRect();
      contextPoint = [e.clientX - r.left, e.clientY - r.top];
    }
    showMenu(
      layer ? "Layers" : paletteArea ? "Palette" : "ViewportContext",
      null,
      { x: e.clientX, y: e.clientY },
    );
  });
  document.addEventListener("keydown", (e) => {
    if (
      e.target.hasAttribute("data-tab") &&
      ["ArrowLeft", "ArrowRight"].includes(e.key)
    ) {
      e.preventDefault();
      const tab = e.target.dataset.tab === "session" ? "comments" : "session";
      setTab(tab);
      $(`[data-tab="${tab}"]`).focus();
      return;
    }
    const swatch = e.target.closest("[data-swatch],[data-edit-swatch]");
    if (
      swatch &&
      [
        "ArrowRight",
        "ArrowLeft",
        "ArrowUp",
        "ArrowDown",
        "Home",
        "End",
      ].includes(e.key)
    ) {
      e.preventDefault();
      e.stopPropagation();
      const editor = swatch.hasAttribute("data-edit-swatch"),
        items = [...swatch.parentElement.children],
        i = items.indexOf(swatch),
        next =
          e.key === "Home"
            ? 0
            : e.key === "End"
              ? items.length - 1
              : (((i +
                  { ArrowRight: 1, ArrowLeft: -1, ArrowUp: -8, ArrowDown: 8 }[
                    e.key
                  ]) %
                  items.length) +
                  items.length) %
                items.length;
      items[next].click();
      return;
    }
    if (e.key === "Escape") {
      if ($("dialog[open]") && !menuChain.length) return;
      if (menuChain.length) { e.preventDefault(); closeMenus(true); return; }
      closeMenus(true);
      closeFloating();
      activeThread = null;
      renderComments();
      window.__drawVoxelScene();
      return;
    }
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      if ($(".palette-dialog").open) return;
      $(".command-dialog").open ? $(".command-dialog").close() : openCommands();
      return;
    }
    if (
      $("dialog[open]") ||
      menuChain.length ||
      /INPUT|TEXTAREA|SELECT/.test(e.target.tagName)
    )
      return;
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      e.shiftKey ? redo() : undo();
      return;
    }
    if (e.metaKey || e.ctrlKey) {
      const key = e.key.toLowerCase();
      if (["n", "o", "s"].includes(key)) {
        e.preventDefault();
        if (historyPreview) return;
        notify(
          {
            n: "New project",
            o: "Open .voxl",
            s: e.shiftKey ? "Save as .voxl" : "Save .voxl",
          }[key] + " · concept preview",
        );
      }
      return;
    }
    if (e.shiftKey && e.key.toLowerCase() === "f") {
      e.preventDefault();
      setFollow(!follow);
      return;
    }
    const choice = tools.find(
      (t) => t[2].toLowerCase() === e.key.toLowerCase(),
    );
    if (choice) {
      e.preventDefault();
      chooseTool(choice[0]);
    }
    if (e.key === "Home") {
      e.preventDefault();
      actions.frame();
    }
  });
  $('.workspace').append($('.history-panel'));
  // The sidebar scrolls on short windows and supports pointer/keyboard resizing.
  const resizer = $('.sidebar-resizer');
  let resizing = false;
  const resize = width => {width = Math.max(320, Math.min(440, width)); $('.workspace').style.setProperty('--sidebar-width', width + 'px'); resizer.setAttribute('aria-valuenow', width);};
  resizer.addEventListener('pointerdown', e => {resizing = true; resizer.setPointerCapture(e.pointerId);});
  resizer.addEventListener('pointermove', e => {if (resizing) resize(innerWidth - e.clientX);});
  resizer.addEventListener('pointerup', () => resizing = false);
  resizer.addEventListener('keydown', e => {if (['ArrowLeft','ArrowRight'].includes(e.key)) {e.preventDefault(); resize(Number(resizer.getAttribute('aria-valuenow')) + (e.key === 'ArrowLeft' ? 16 : -16));}});
  for (const el of $$('[data-palette-rename],[data-hex-edit]')) el.addEventListener('keydown', e => {if (e.key === 'Enter') {e.preventDefault(); el.blur();}});
  document.addEventListener('input', e => {
    if (e.target.matches('[data-tool-density]')) {toolConfig.density = Number(e.target.value); e.target.nextElementSibling.textContent = toolConfig.density + '%';}
    if (e.target.matches('[data-tool-seed]')) toolConfig.seed = Number(e.target.value);
  });
  renderPalette();
  renderComments();
  syncSteps();
  window.__drawVoxelScene();
  window.__aPreview = {
    exportText,
    parsePalette,
    get palettes() {
      return palettes();
    },
    get comments() {
      return comments;
    },
    get versions() {
      return versions;
    },
    get branches() {
      return [...branches.values()];
    },
    get toolConfig() { return {...toolConfig}; },
    get tool() {
      return tool;
    },
    get mirrors() {
      return [...mirrors];
    },
  };
})();
