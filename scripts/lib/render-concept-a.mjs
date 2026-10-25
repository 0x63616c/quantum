import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";

const settle = (page) =>
  page.evaluate(
    () =>
      new Promise((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(resolve)),
      ),
  );
const count = (page) => page.evaluate(() => window.__voxelSceneStats.cubes);
const hashes = async (page) =>
  (
    await page.evaluate(() =>
      [...document.querySelectorAll("[data-thumbnail]")].map((c) =>
        c.toDataURL(),
      ),
    )
  ).map((s) => createHash("sha256").update(s).digest("hex"));
const menu = async (page, name) => {
  await page.locator(`.menubar [data-menu="${name}"]`).click();
};
const menuItem = (page, name) =>
  page.getByRole("menuitem", { name, exact: true })
    .or(page.getByRole("menuitemcheckbox", {name, exact:true}))
    .or(page.getByRole("menuitemradio", {name, exact:true}));
const paletteAction = async (page, label) => {
  await page.locator('[data-menu="PaletteManage"]').click();
  await menuItem(page, label).click();
};
const swatchAction = async (page, label) => {
  await page.locator('[data-menu="SwatchEdit"]').click();
  await menuItem(page, label).click();
};
const versionAction = async (page, label, id = null) => {
  await (id ? page.locator(`[data-menu="Version:${id}"]`) : page.locator('.version-item [data-menu]').first()).click();
  await menuItem(page, label).click();
};

export async function renderConceptA(browser, root, output) {
  const files = [];
  for (const theme of ["dark", "light"]) {
    const page = await browser.newPage({
      viewport: { width: 1600, height: 1000 },
      deviceScaleFactor: 2,
      reducedMotion: "reduce",
    });
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.message));
    await page.route(/^https?:/, (r) => {
      errors.push("Unexpected request: " + r.request().url());
      return r.abort();
    });
    await page.goto(
      pathToFileURL(`${root}/design/concepts/concept-a/index.html`).href,
    );
    await page.waitForFunction(
      () => window.__voxelSceneReady && window.__aPreview,
    );
    await page.evaluate(async (theme) => {
      document.documentElement.dataset.theme = theme;
      await document.fonts.ready;
    }, theme);
    await settle(page);
    const capture = async (suffix) => {
      await settle(page);
      const audit = await page.evaluate(() => ({
        width: document.documentElement.scrollWidth,
        height: document.documentElement.scrollHeight,
        fonts:
          document.fonts.check("13px Geist") &&
          document.fonts.check('12px "Geist Mono"'),
        minimumText: Math.min(
          ...[...document.querySelectorAll("body *")]
            .filter(
              (el) =>
                el.getClientRects().length &&
                [...el.childNodes].some(
                  (n) => n.nodeType === 3 && n.textContent.trim(),
                ),
            )
            .map((el) => parseFloat(getComputedStyle(el).fontSize)),
        ),
        floatingFit: [
          ...document.querySelectorAll(
            ".floating:not([hidden]),dialog[open],.dropdown-menu",
          ),
        ].every((el) => {
          const r = el.getBoundingClientRect();
          return (
            r.left >= 0 && r.right <= 1600 && r.top >= 0 && r.bottom <= 1000
          );
        }),
        base: getComputedStyle(document.body).backgroundColor,
        material: ['.sidebar', '.unified-toolbar'].map(selector => {
          const style = getComputedStyle(document.querySelector(selector));
          return {alpha: Number(style.backgroundColor.match(/[\d.]+/g)[3]), blur: style.backdropFilter};
        }),
        transparentSession: getComputedStyle(document.querySelector('.agent-session')).backgroundColor === 'rgba(0, 0, 0, 0)',
        ambient: getComputedStyle(document.querySelector('.app'), '::before').backgroundImage,
        blur: getComputedStyle(document.querySelector(".sidebar"))
          .backdropFilter,
        contrast: (() => {
          const style = getComputedStyle(document.body),
            rgb = (token) => {
              const p = document.createElement("span");
              p.style.color = style.getPropertyValue(token);
              document.body.append(p);
              const c = getComputedStyle(p)
                .color.match(/[\d.]+/g)
                .map(Number);
              p.remove();
              return c;
            };
          const luminance = (c) =>
            c
              .slice(0, 3)
              .map((v) => v / 255)
              .map((v) =>
                v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4,
              )
              .reduce((sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i], 0);
          const ratio = (a, b) =>
            (Math.max(luminance(a), luminance(b)) + 0.05) /
            (Math.min(luminance(a), luminance(b)) + 0.05);
          const bg = rgb("--bg"),
            float = rgb("--float"),
            surface = float
              .slice(0, 3)
              .map(
                (v, i) => v * (float[3] ?? 1) + bg[i] * (1 - (float[3] ?? 1)),
              );
          return Math.min(
            ...["--text", "--strong", "--muted", "--accent"].flatMap((t) => [
              ratio(rgb(t), bg),
              ratio(rgb(t), surface),
            ]),
          );
        })(),
      }));
      assert.ok(audit.contrast >= 4.5, `Text contrast ${audit.contrast}`);
      assert.equal(audit.width, 1600);
      assert.equal(audit.height, 1000);
      assert.ok(audit.fonts && audit.minimumText >= 12, JSON.stringify(audit));
      assert.ok(audit.floatingFit, `Floating layer clipped (${suffix})`);
      assert.equal(
        audit.base,
        theme === "dark" ? "rgb(0, 0, 0)" : "rgb(255, 255, 255)",
      );
      assert.ok(audit.blur.includes("blur"));
      assert.ok(audit.material.every(m => m.alpha > 0 && m.alpha < .75 && m.blur.includes('blur')));
      assert.ok(audit.transparentSession && audit.ambient.includes('radial-gradient'));
      const path = `${output}/concept-a-${theme}${suffix ? "-" + suffix : ""}.png`;
      await page.screenshot({ path, animations: "disabled" });
      assert.ok((await stat(path)).size > 100000);
      files.push(path);
      console.log(
        `Rendered concept-a-${theme}${suffix ? "-" + suffix : ""}.png · ${await count(page)} voxels`,
      );
    };
    assert.equal(await count(page), 2079);
    assert.equal(new Set(await hashes(page)).size, 3);
    assert.equal(await page.locator("[data-pin]:visible").count(), 3);
    assert.ok(
      await page
        .locator(".sidebar")
        .evaluate((el) => el.scrollHeight <= el.clientHeight + 1),
      "Default sidebar overflow",
    );
    assert.equal(await page.locator("details[open]").count(), 1);
    assert.equal(await page.locator("[data-follow-state]").textContent(), "On");
    await capture("");
    await menu(page, "File");
    assert.equal(
      await page.getByRole("menu", { name: "File", exact: true }).isVisible(),
      true,
    );
    await menuItem(page, 'Export').focus();
    await page.keyboard.press('ArrowRight');
    assert.equal(await menuItem(page, 'Export').getAttribute('aria-expanded'), 'true');
    await capture("menu");
    await page.keyboard.press("Escape");
    await menu(page, "Palette");
    await menuItem(page, "Manage palettes…").click();
    assert.ok(await page.locator(".palette-dialog").evaluate((el) => el.open));
    assert.ok(await page.locator('[data-action="use-palette"]').isDisabled());
    await page.locator('[data-edit-palette="pico8"]').click();
    assert.ok(await page.locator('[data-hex-edit]').isDisabled());
    assert.ok(await page.locator('.duplicate-preset').isVisible());
    assert.ok(await page.locator('[data-menu="SwatchEdit"]').isDisabled());
    assert.ok(await page.locator('[data-action="add-swatch"]').isDisabled());
    assert.equal(await page.locator('[data-editor-status]').textContent(), 'Applies to new voxels');
    const sheet = await page.locator('.palette-dialog').boundingBox();
    assert.ok(sheet.x >= 1200 && sheet.x + sheet.width === 1600, 'Palette manager is a right sheet');
    await capture("palettes");
    await page.keyboard.press("Escape");
    await page.locator('[data-pin="1"]').click();
    assert.equal(await page.locator("#comments-panel").isVisible(), true);
    assert.equal(await page.locator(".mcp-note").count(), 0);
    assert.ok(await page.locator('.thread-meta .agent-avatar').isVisible());
    assert.ok(
      await page
        .locator(".sidebar")
        .evaluate((el) => el.scrollHeight <= el.clientHeight + 1),
      "Comments sidebar overflow",
    );
    assert.equal(await page.locator('.viewport .pin-thread,.comment-anchor').count(), 0);
    assert.equal(await page.getByText('Could the canopy feel a little less symmetrical?', {exact:true}).count(), 1);
    const threadBox = await page.locator('.pin-thread').boundingBox();
    const viewBox = await page.locator('.viewport').boundingBox();
    assert.ok(threadBox.x >= viewBox.x + viewBox.width, 'Discussion must never overlap artwork');
    await capture("comments");
    await page.locator('[data-tab="session"]').click();
    await page.locator('[data-action="share"]').click();
    await page.locator('[data-access-scope]').selectOption('restricted');
    assert.ok(await page.locator('[data-permission="view"]').isDisabled());
    assert.ok((await page.locator('.link-field input').inputValue()).includes('scope=restricted'));
    await page.locator('[data-access-scope]').selectOption('link');
    await page.locator('[data-permission="remix"]').click();
    await page.evaluate(() => Object.defineProperty(navigator, 'clipboard', {configurable:true, value:{writeText:async text => window.__copiedLink = text}}));
    await page.locator('[data-action="copy-link"]').click();
    assert.equal(await page.locator('.copy-link span').textContent(), 'Copied');
    assert.equal(await page.locator('.share-feedback').textContent(), '');
    assert.ok((await page.locator('.permission-description').textContent()).includes('comment'));
    assert.ok((await page.locator('.access-rule').textContent()).includes('whichever is higher'));
    assert.ok((await page.locator('.share-popover').boundingBox()).height < 430);
    await capture("share");
    await page.locator('[data-close="share"]').click();
    await page.locator('[data-action="history"]').click();
    await page.locator("[data-timeline]").fill("4");
    assert.ok((await count(page)) < 2079);
    assert.ok(await page.locator('.tool-picker').isDisabled());
    assert.equal(await page.locator('[data-action="save-version"]').textContent(), 'Name version');
    const previewTool = await page.locator('[data-current-tool]').textContent();
    const previewCount = await count(page);
    await page.keyboard.press('p');
    await page.keyboard.press('Meta+z');
    assert.equal(await count(page), previewCount);
    assert.equal(await page.locator('[data-current-tool]').textContent(), previewTool);
    await menu(page, 'Tools');
    assert.ok(await menuItem(page, 'Paint').isDisabled());
    await page.keyboard.press('Escape');
    // Menu Escape dismisses only the menu, retaining the read-only preview.
    assert.equal(await count(page), previewCount);
    await menu(page, 'Edit');
    assert.ok(await menuItem(page, 'Undo').isDisabled());
    await page.keyboard.press('Escape');
    await page.keyboard.press('Meta+k');
    await page.locator('.command-search input').fill('Paint');
    assert.ok(await page.locator('.command-empty').isVisible());
    await page.keyboard.press('Escape');
    // Closing command search must preserve the preview as well.
    assert.equal(await count(page), previewCount);
    assert.equal(await page.locator('.timeline-ticks').count(), 0);
    assert.equal(await page.locator('.history-event small').first().textContent(), 'Artist 01');
    const rows = await page.locator('.history-event-list').evaluate(el => {
      const outer = el.getBoundingClientRect();
      return [...el.children].filter(row => {
        const r = row.getBoundingClientRect(); return r.top >= outer.top && r.bottom <= outer.bottom;
      }).length;
    });
    assert.equal(rows, 9, 'All nine fixture states should fit');
    await page.locator('.history-event[data-history-tick="4"]').click();
    await capture("history");
    await page.locator('[data-close="history"]').click();
    assert.equal(await count(page), 2079, "History close restores live scene");

    await page.locator('.tool-picker').click();
    assert.ok(await page.getByRole('textbox', {name:'Search tools',exact:true}).isVisible());
    assert.equal(await page.locator('.dropdown-menu .tool-configuration').count(), 0);
    await capture('tools');
    await page.keyboard.press('Escape');
    await page.keyboard.press('b');

    // Native-style menu traversal, submenus, dismissal and context menus.
    await page.locator('.menubar [data-menu="File"]').focus();
    await page.keyboard.press("ArrowDown");
    assert.equal(
      await page.evaluate(() => document.activeElement.textContent.trim()),
      "New project⌘N",
    );
    await page.keyboard.press("End");
    await page.keyboard.press("ArrowRight");
    assert.equal(
      await page.getByRole("menu", { name: "Export", exact: true }).isVisible(),
      true,
    );
    await page.keyboard.press("ArrowLeft");
    await page.keyboard.press("Escape");
    assert.equal(
      await page.evaluate(() => document.activeElement.dataset.menu),
      "File",
    );
    for (const [selector, expected] of [
      [".viewport>canvas", "ViewportContext"],
      ['[data-layer="Roof"]', "Layers"],
      [".palette-section", "Palette"],
    ]) {
      await page
        .locator(selector)
        .click({ button: "right", position: { x: 15, y: 15 } });
      assert.ok(
        await page
          .getByRole("menu", { name: expected, exact: true })
          .isVisible(),
      );
      await page.keyboard.press("Escape");
    }
    await page.locator(".tool-picker").click();
    assert.equal(await menuItem(page, "Scatter moss Agent").count(), 1);
    await menuItem(page, "Scatter moss Agent").click();
    assert.equal(
      await page.locator("[data-current-tool]").textContent(),
      "Scatter moss",
    );
    await page.keyboard.press("p");
    assert.equal(
      await page.locator("[data-current-tool]").textContent(),
      "Paint",
    );
    await menu(page, "View");
    assert.equal(await menuItem(page, "Show grid").getAttribute('role'), 'menuitemcheckbox');
    await menuItem(page, "Show grid").click();
    assert.equal(
      await page.locator("[data-toggle-grid]").getAttribute("aria-pressed"),
      "false",
    );
    await menu(page, "View");
    assert.equal(await menuItem(page, "Show grid").getAttribute('role'), 'menuitemcheckbox');
    await menuItem(page, "Show grid").click();
    await menu(page, "Tools");
    await menuItem(page, "Mirror").click();
    await menuItem(page, "Mirror Y").click();
    assert.deepEqual(await page.evaluate(() => window.__aPreview.mirrors), [
      "Y",
    ]);

    await menu(page, 'Edit');
    assert.ok(await menuItem(page, 'Redo').isDisabled());
    await menuItem(page, 'Undo').focus();
    await page.keyboard.press('ArrowDown');
    assert.equal(await page.evaluate(() => document.activeElement.getAttribute('aria-label')), 'Search commands');
    await page.keyboard.press('Escape');
    await menu(page, 'Tools');
    assert.equal(await menuItem(page, 'Paint').getAttribute('aria-checked'), 'true');
    await page.keyboard.press('Escape');
    await page.locator('.tool-picker').click();
    await page.getByRole('textbox', {name:'Search tools',exact:true}).fill('paint');
    assert.equal(await page.locator('.dropdown-menu [data-menu-index]:visible').count(), 1);
    await page.keyboard.press('ArrowDown');
    assert.equal(await page.evaluate(() => document.activeElement.getAttribute('aria-label')), 'Paint');
    await page.keyboard.press('Enter');
    assert.equal(await page.locator('[data-current-tool]').textContent(), 'Paint');
    await menu(page, 'Extensions');
    await page.getByRole('textbox', {name:'Search extensions',exact:true}).fill('no-match');
    assert.ok(await page.locator('.menu-empty').isVisible());
    await page.getByRole('textbox', {name:'Search extensions',exact:true}).fill('scatter');
    assert.equal(await page.locator('.dropdown-menu [data-menu-index]:visible').count(), 1);
    await page.keyboard.press('Enter');
    assert.ok(await page.locator('.tool-settings').isVisible());
    await menu(page, 'Extensions');
    assert.equal(await menuItem(page, 'Scatter moss Agent').getAttribute('aria-checked'), 'true');
    await page.keyboard.press('Escape');
    await page.locator('[data-tool-density]').fill('36');
    await page.locator('[data-tool-seed]').fill('7');
    assert.deepEqual(await page.evaluate(() => window.__aPreview.toolConfig), {density:36,seed:7});
    await page.keyboard.press('Escape');
    await page.locator('.tool-picker').click();
    assert.equal(await page.locator('[data-tool-density]').inputValue(), '36');
    await page.keyboard.press('Escape');

    // Palette switching, keyboard selection, accurate export bytes and round trips.
    const beforePaletteGeometry = await page.evaluate(() => JSON.stringify(window.__sceneSnapshot()));
    await page.locator(".palette-switch").click();
    await menuItem(page, "PICO-8").click();
    assert.equal(
      await page.locator("[data-active-swatches] button").count(),
      16,
    );
    assert.equal(
      await page.locator('[data-swatch="1"]').getAttribute("data-color"),
      "#1d2b53",
    );
    await page.locator('[data-swatch="0"]').focus();
    await page.keyboard.press("End");
    assert.equal(
      await page
        .locator('[data-swatch][aria-pressed="true"]')
        .getAttribute("data-color"),
      "#ffccaa",
    );
    await page.locator(".palette-switch").click();
    await menuItem(page, "Endesga 32").click();
    assert.equal(
      await page.locator("[data-active-swatches] button").count(),
      32,
    );
    assert.equal(
      await page.locator('[data-swatch="31"]').getAttribute("data-color"),
      "#c28569",
    );
    assert.equal(await page.evaluate(() => JSON.stringify(window.__sceneSnapshot())), beforePaletteGeometry, 'Palette switch must not recolor geometry');
    await menu(page, "Palette");
    await menuItem(page, "Manage palettes…").click();
    assert.ok(await page.locator("[data-hex-edit]").isDisabled());
    for (const format of ["hex", "gpl"]) {
      const dl = page.waitForEvent("download");
      await page.locator('[data-menu="PaletteManage"]').click();
      await menuItem(page, 'Export palette').click();
      await menuItem(page, format === 'hex' ? 'Hex colors (.hex)' : 'GIMP palette (.gpl)').click();
      const download = await dl;
      const text = await readFile(await download.path(), "utf8");
      assert.equal(
        text,
        await page.evaluate(
          (format) => window.__aPreview.exportText(format),
          format,
        ),
      );
      assert.ok(text.endsWith("\n"));
      if (format === "hex") {
        assert.equal(text.split("\n")[0], "BE4A2F");
        assert.equal(text.trim().split("\n").length, 32);
      } else {
        assert.ok(
          text.startsWith("GIMP Palette\nName: Endesga 32\nColumns: 8\n#\n"),
        );
        assert.ok(text.includes("190  74  47\t#BE4A2F"));
      }
      assert.deepEqual(
        await page.evaluate(
          ({ text, format }) =>
            window.__aPreview.parsePalette(text, format).colors,
          { text, format },
        ),
        await page.evaluate(
          () =>
            window.__aPreview.palettes.find((p) => p.id === "endesga32").colors,
        ),
      );
    }
    await page.locator('[data-action="duplicate-palette"]').click();
    await page.locator("[data-palette-rename]").fill("Forest experiment");
    await page.locator("[data-palette-rename]").press("Tab");
    await page.locator("[data-hex-edit]").fill("#123456");
    await page.locator("[data-hex-edit]").press("Tab");
    await swatchAction(page, 'Move right');
    assert.equal(
      await page.locator('[data-edit-swatch="1"]').getAttribute("data-color"),
      "#123456",
    );
    await page.locator('[data-action="add-swatch"]').click();
    assert.equal(await page.locator("[data-edit-swatch]").count(), 33);
    await swatchAction(page, 'Remove color');
    assert.equal(await page.locator("[data-edit-swatch]").count(), 32);
    await page.locator('[data-action="use-palette"]').click();
    assert.equal(
      await page.locator("[data-palette-name]").textContent(),
      "Forest experiment",
    );
    await menu(page, "Palette");
    await menuItem(page, "Manage palettes…").click();
    await paletteAction(page, 'Delete palette');
    assert.equal(
      await page.locator("[data-palette-name]").textContent(),
      "Woodland",
    );
    await page.locator('[data-action="create-palette"]').click();
    assert.equal(await page.locator("[data-edit-swatch]").count(), 1);
    await paletteAction(page, 'Delete palette');
    await page.locator("[data-palette-file]").setInputFiles({
      name: "sample.gpl",
      mimeType: "text/plain",
      buffer: Buffer.from(
        "GIMP Palette\nName: Imported study\nColumns: 2\n#\n  1   2   3\tInk\n255 128   0\tGold\n",
      ),
    });
    await page.waitForFunction(
      () =>
        document.querySelector("[data-editor-title]").textContent ===
        "Imported study",
    );
    assert.equal(
      await page.locator('[data-edit-swatch="0"]').getAttribute("data-color"),
      "#010203",
    );
    await paletteAction(page, 'Delete palette');
    await page.locator("[data-palette-file]").setInputFiles({
      name: "three.hex",
      mimeType: "text/plain",
      buffer: Buffer.from("123456\nABCDEF\n000000\n"),
    });
    await page.waitForFunction(
      () =>
        document.querySelector("[data-editor-title]").textContent === "three",
    );
    await page.locator('[data-edit-swatch="0"]').focus();
    await page.keyboard.press("ArrowUp");
    assert.equal(
      await page
        .locator('[data-edit-swatch][aria-pressed="true"]')
        .getAttribute("data-edit-swatch"),
      "1",
    );
    await paletteAction(page, 'Delete palette');
    const palettesBefore = await page.evaluate(
      () => window.__aPreview.palettes.length,
    );
    await page.locator("[data-palette-file]").setInputFiles({
      name: "invalid.hex",
      mimeType: "text/plain",
      buffer: Buffer.from("not a color"),
    });
    await page.waitForFunction(() =>
      document.querySelector(".toast").textContent.includes("six-digit"),
    );
    assert.equal(
      await page.evaluate(() => window.__aPreview.palettes.length),
      palettesBefore,
    );
    await page.keyboard.press("Escape");

    // Progressive disclosure and immutable captures across edits/camera changes.
    await page.keyboard.press('b');
    const initialCaptures = await hashes(page);
    assert.ok((await page.locator('[data-thumbnail="front"]').boundingBox()).height >= 140);
    for (const angle of ['right','back','front']) {
      await page.locator('[data-capture-angle="' + angle + '"]').click();
      assert.ok(await page.locator('[data-view-angle="' + angle + '"]').isVisible());
      assert.equal(await page.locator('.render-preview:visible').count(), 1);
      assert.deepEqual(await hashes(page), initialCaptures);
    }
    for (const layer of ['Canopy','Roof','Cottage','Garden']) {
      for (const control of ['eye','lock']) {
        const button = page.locator('[data-layer-' + control + '="' + layer + '"]');
        assert.ok(await button.isVisible());
        assert.equal(await button.evaluate(el => getComputedStyle(el).opacity), '1');
      }
    }
    const lock = page.locator('[data-layer-lock="Canopy"]');
    await lock.click();
    assert.equal(await lock.getAttribute('aria-pressed'), 'true');
    await lock.click();
    assert.equal(await lock.getAttribute('aria-pressed'), 'false');
    await page.locator('[data-step="18"]').evaluate(el => el.open = true);
    assert.equal(
      (await page.locator('[data-step-coordinates="18"]').textContent()).split(
        "\n",
      ).length,
      6,
    );
    await page.locator('[data-step="18"] .coordinate-detail summary').click();
    assert.ok(await page.locator('[data-step-coordinates="18"]').isVisible());
    await page.locator('[data-step="18"] .coordinate-detail summary').click();
    await page.locator('[data-step-undo="18"]').click();
    assert.equal(await count(page), 2073);
    assert.deepEqual(await hashes(page), initialCaptures);
    assert.equal(await page.locator("[data-render-stale]").isVisible(), true);
    await page.locator('[data-step-undo="18"]').click();
    assert.equal(await count(page), 2079);
    await page.keyboard.press("Meta+z");
    await page.keyboard.press("Meta+z");
    assert.equal(await count(page), 2049);
    await page.keyboard.press("Meta+Shift+z");
    await page.keyboard.press("Meta+Shift+z");
    assert.equal(await count(page), 2079);
    await page.locator('[data-step="18"] > summary').click();
    await page.locator('[data-layer="Canopy"]').hover();
    await page.locator('[data-layer-eye="Canopy"]').click();
    assert.deepEqual(await hashes(page), initialCaptures);
    assert.equal(await page.locator('[data-pin="1"]').isVisible(), false);
    await page.locator('[data-layer-eye="Canopy"]').click();
    await menu(page, "Agent");
    await menuItem(page, "Refresh captured views").click();
    const fresh = await hashes(page);
    assert.equal(await page.locator("[data-render-stale]").isVisible(), false);
    const pin = await page.locator('[data-pin="1"]').getAttribute("style");
    await page.locator(".viewport>canvas").focus();
    await page.keyboard.press("ArrowRight");
    assert.notEqual(
      await page.locator('[data-pin="1"]').getAttribute("style"),
      pin,
    );
    assert.deepEqual(await hashes(page), fresh);
    assert.equal(
      await page.locator("[data-follow]").getAttribute("aria-pressed"),
      "false",
    );
    await page.keyboard.press("Home");
    await page.locator("[data-follow]").click();
    assert.equal(
      await page.locator("[data-follow]").getAttribute("aria-pressed"),
      "true",
    );
    await page.locator("[data-pause]").click();
    assert.equal(
      await page.locator("[data-session-state]").textContent(),
      "Paused",
    );
    await page.locator("[data-pause]").click();

    // Camera changes cannot collide the docked thread with the artwork.
    await page.locator('[data-pin="1"]').click();
    for (const angle of ['right','back','front']) {
      await page.evaluate(angle => window.__setViewAngle(angle), angle);
      const dock = await page.locator('.pin-thread').boundingBox();
      const canvas = await page.locator('.viewport').boundingBox();
      assert.ok(dock.x >= canvas.x + canvas.width);
      assert.equal(await page.locator('.viewport .pin-thread,.comment-anchor').count(), 0);
    }
    // Comment reply, resolve/reopen, filters and share permission/clipboard fallback.

    await page
      .locator(".reply-form textarea")
      .fill("The softer silhouette works.");
    await page.locator(".reply-form button").click();
    assert.ok(
      (await page.locator(".pin-thread").textContent()).includes(
        "The softer silhouette works.",
      ),
    );
    await page.locator('[data-resolve="1"]').click();
    assert.equal(await page.locator('[data-pin="1"]').count(), 0);
    await page.locator("[data-comment-filter]").selectOption("resolved");
    await page.locator('[data-open-thread="1"]').click();
    await page.locator('[data-resolve="1"]').click();
    await page.locator("[data-comment-filter]").selectOption("open");
    assert.equal(await page.locator('[data-open-thread="1"]').count(), 1);
    const anchor = await page.locator('[data-pin="2"]').evaluate((el) => ({
      x: parseFloat(el.style.left),
      y: parseFloat(el.style.top) + 5,
    }));
    await page
      .locator(".viewport>canvas")
      .click({ button: "right", position: anchor });
    await menuItem(page, "Add comment here").click();
    await page
      .locator(".reply-form textarea")
      .fill("Keep the chimney silhouette.");
    await page.locator(".reply-form button").click();
    assert.equal(
      await page.evaluate(() => window.__aPreview.comments.at(-1).text),
      "Keep the chimney silhouette.",
    );
    await page.locator('[data-tab="session"]').click();
    await page.locator('[data-action="share"]').click();
    await page.locator('[data-access-scope]').selectOption('restricted');
    assert.ok(await page.locator('[data-permission="view"]').isDisabled());
    assert.ok((await page.locator('.link-field input').inputValue()).includes('scope=restricted'));
    await page.locator('[data-access-scope]').selectOption('link');
    await page.locator('[data-permission="remix"]').click();
    assert.ok(
      (await page.locator(".link-field input").inputValue()).endsWith(
        "access=remix",
      ),
    );
    await page.evaluate(() =>
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          writeText: async (text) => {
            window.__copiedLink = text;
          },
        },
      }),
    );
    await page.locator('[data-action="copy-link"]').click();
    assert.equal(
      await page.evaluate(() => window.__copiedLink),
      await page.locator(".link-field input").inputValue(),
    );
    await page.evaluate(
      () =>
        (navigator.clipboard.writeText = async () => {
          throw Error("Unavailable");
        }),
    );
    await page.locator('[data-action="copy-link"]').click();
    assert.ok(
      await page
        .locator(".link-field input")
        .evaluate(
          (el) => el.selectionEnd - el.selectionStart === el.value.length,
        ),
    );
    await page.keyboard.press("Escape");

    // Geometry-backed history, save/restore/compare and branch actions.
    await menu(page, "View");
    await menuItem(page, "History").click();
    const latest = await count(page);
    await page.locator("[data-timeline]").fill("4");
    const midway = await count(page);
    assert.ok(await page.locator('.history-preview-banner').isVisible());
    assert.equal(await page.locator('.history-event-list button').count(), 9);
    await page.locator('[data-timeline]').fill('0');
    assert.equal(await count(page), 0);
    assert.equal(await page.locator('[data-history-position]').textContent(), 'Initial state');
    assert.ok(await page.locator('[data-action="history-prev"]').isDisabled());
    await page.locator('[data-timeline]').fill('4');
    assert.ok(midway < latest && midway > 1000);
    await page.locator('[data-action="save-version"]').click();
    assert.equal(await page.locator('[data-version-save-context]').textContent(), 'Saves the previewed geometry at step 4.');
    await page.locator(".version-form input").fill("Garden checkpoint");
    await page.locator(".version-form button").click();
    assert.equal(
      await page.locator(".version-item h3").first().textContent(),
      "Garden checkpoint",
    );
    assert.equal(await page.evaluate(() => window.__aPreview.versions[0].snapshot.length), midway);
    await page.locator('[data-action="history-live"]').click();
    assert.equal(await count(page), latest);
    assert.ok(await page.locator('.tool-picker').isEnabled());
    await versionAction(page, 'Compare with current');
    assert.ok(
      (await page.locator(".compare-badge").textContent()).includes(
        `+${latest - midway} / −0`,
      ),
    );
    await page.locator('[data-action="clear-compare"]').click();
    await versionAction(page, 'Restore version');
    assert.equal(await count(page), midway);
    await page.locator('[data-close="history"]').click();
    assert.equal(
      await count(page),
      midway,
      "Restore must persist after closing history",
    );
    await page.locator('[data-action="history"]').click();
    await versionAction(page, 'Branch from version', 'v2');
    assert.equal(await count(page), latest);
    assert.equal(
      await page.locator("[data-branch-name]").textContent(),
      "canopy-pass-branch",
    );
    assert.deepEqual(
      await page.evaluate(() =>
        window.__aPreview.branches.map((b) => ({
          name: b.name,
          count: b.snapshot.length,
          parent: b.parent,
        })),
      ),
      [
        { name: "main", count: midway, parent: null },
        { name: "canopy-pass-branch", count: latest, parent: "main" },
      ],
    );
    await page.locator('[data-close="history"]').click();
    await page.keyboard.press("Meta+k");
    await page.locator(".command-search input").fill("nonsense");
    assert.ok(await page.locator(".command-empty").isVisible());
    await page.locator(".command-search input").fill("");
    await page.keyboard.press("ArrowDown");
    assert.ok(
      await page
        .locator(".command-search input")
        .evaluate((el) => el === document.activeElement),
    );
    await page.keyboard.type("Paint");
    await page.keyboard.press("Enter");
    assert.equal(
      await page.locator("[data-current-tool]").textContent(),
      "Paint",
    );
    // Deleting the final custom palette falls back to a preset and survives reload.
    await menu(page, "Palette");
    await menuItem(page, "Manage palettes…").click();
    await paletteAction(page, 'Delete palette');
    assert.equal(
      await page.locator("[data-palette-name]").textContent(),
      "PICO-8",
    );
    await page.keyboard.press("Escape");
    await page.reload();
    await page.waitForFunction(() => window.__aPreview);
    assert.equal(
      await page.locator("[data-palette-name]").textContent(),
      "PICO-8",
    );
    assert.equal(
      await page.evaluate(
        () => window.__aPreview.palettes.filter((p) => !p.preset).length,
      ),
      0,
    );
    // A smaller window keeps controls reachable through a bounded, resizable sidebar.
    await page.setViewportSize({width:1100,height:760});
    await settle(page);
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth), 1100);
    assert.equal(await page.evaluate(() => document.documentElement.scrollHeight), 760);
    await page.locator('.sidebar-resizer').focus();
    await page.keyboard.press('ArrowLeft');
    assert.equal(await page.locator('.sidebar-resizer').getAttribute('aria-valuenow'), '376');
    await page.locator('.palette-switch').scrollIntoViewIfNeeded();
    assert.ok(await page.locator('.palette-switch').isVisible());
    await page.locator('[data-action="history"]').click();
    await page.locator('[data-timeline]').fill('4');
    assert.ok(await page.locator('.history-preview-banner').isVisible());
    assert.ok(await page.locator('.history-event-list').evaluate(el => el.scrollHeight > el.clientHeight));
    await page.locator('.history-event-list [data-history-tick="8"]').scrollIntoViewIfNeeded();
    assert.ok(await page.locator('.history-event-list [data-history-tick="8"]').isVisible());
    await page.locator('[data-close="history"]').click();
    await page.keyboard.press('Meta+k');
    await page.keyboard.type('Scatter');
    await page.keyboard.press('Enter');
    assert.equal(await page.locator('[data-current-tool]').textContent(), 'Scatter moss');
    assert.deepEqual(errors, [], "Concept A browser errors");
    console.log(
      `Verified A ${theme}: menus, palettes/files, comments/share, history geometry, captures and keyboard controls.`,
    );
    await page.close();
  }
  return files;
}
