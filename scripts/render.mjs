import { chromium } from 'playwright';
import { mkdir, stat } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

async function captureHashes(page) {
  const captures = await page.evaluate(() => [...document.querySelectorAll('[data-thumbnail]')].map(canvas => canvas.toDataURL()));
  return captures.map(value => createHash('sha256').update(value).digest('hex'));
}

// Each HTML file is standalone; no HTTP server or network assets are needed.
const root = fileURLToPath(new URL('../', import.meta.url));
const output = `${root}/design/concepts/png`;
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true });
const rendered = [];
try {
  for (const concept of ['a', 'b', 'c']) {
    const page = await browser.newPage({
      viewport: { width: 1600, height: 1000 },
      deviceScaleFactor: 2,
      reducedMotion: 'reduce',
    });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.route(/^https?:/, route => {
      errors.push(`Unexpected network request: ${route.request().url()}`);
      return route.abort();
    });
    await page.goto(pathToFileURL(`${root}/design/concepts/concept-${concept}/index.html`).href);
    await page.waitForFunction(() => window.__voxelSceneReady === true);
    await page.evaluate(() => document.fonts.ready);
    for (const theme of ['dark', 'light']) {
      await page.evaluate(theme => document.documentElement.dataset.theme = theme, theme);
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      const metrics = await page.evaluate(() => ({
        width: document.documentElement.scrollWidth,
        height: document.documentElement.scrollHeight,
        theme: document.documentElement.dataset.theme,
        scene: window.__voxelSceneStats,
        tools: [...document.querySelectorAll('[data-tool]')].map(el => el.dataset.tool),
        mirror: document.querySelectorAll('[data-mirror]').length,
        layers: document.querySelectorAll('[data-layer]').length,
        connected: !!document.querySelector('.agent-status'),
        session: !!document.querySelector('.agent-session, .c-console'),
        panelsFit: [...document.querySelectorAll('.a-inspector, .b-inspector, .c-inspector')]
          .every(el => el.scrollHeight <= el.clientHeight + 1),
        viewportFits: document.querySelector('.viewport').getBoundingClientRect().bottom <=
          document.querySelector('.statusbar').getBoundingClientRect().top + 1,
        sessionFits: [...document.querySelectorAll('.agent-session,.b-session,.b-material-dock')].every(el => el.scrollHeight <= el.clientHeight + 1),
        black: getComputedStyle(document.body).backgroundColor,
        panel: getComputedStyle(document.querySelector('.agent-session')).backgroundColor,
        fonts: document.fonts.check('13px Geist') && document.fonts.check('12px "Geist Mono"'),
        minimumText: Math.min(...[...document.querySelectorAll('body *')].filter(el =>
          el.getClientRects().length && [...el.childNodes].some(n => n.nodeType === Node.TEXT_NODE && n.textContent.trim())
        ).map(el => parseFloat(getComputedStyle(el).fontSize))),
        contrast: (() => {
          const style = getComputedStyle(document.body);
          const rgb = token => {
            const probe = document.createElement('span');
            probe.style.color = style.getPropertyValue(token); document.body.append(probe);
            const components = getComputedStyle(probe).color.match(/[\d.]+/g).map(Number); probe.remove();
            return components;
          };
          const luminance = color => color.slice(0,3).map(v => v / 255).map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4)
            .reduce((sum,v,i) => sum + v * [.2126,.7152,.0722][i], 0);
          const ratio = (a,b) => (Math.max(luminance(a),luminance(b)) + .05) / (Math.min(luminance(a),luminance(b)) + .05);
          const base = rgb('--bg'), tint = rgb('--accent-soft');
          const soft = tint.slice(0,3).map((v,i) => v * (tint[3] ?? 1) + base[i] * (1 - (tint[3] ?? 1)));
          return {
            text: Math.min(...['--text','--strong','--muted','--accent'].flatMap(t => [ratio(rgb(t), base), ratio(rgb(t), soft)])),
            controls: Math.min(ratio(rgb('--control'),base), ratio(rgb('--accent'),soft)),
          };
        })(),
        tableAligned: (() => {
          const header = document.querySelector('.table-labels');
          if (!header) return true;
          const row = document.querySelector('.transaction-log [data-step="18"]');
          const cells = ['.step-number','.step-inspect','.diff','.step-meta>code','.step-meta>.muted','.step-undo'];
          return cells.every((s,i) => Math.abs(row.querySelector(s).getBoundingClientRect().left - header.children[i].getBoundingClientRect().left) < 1);
        })(),
        distinctThumbs: new Set([...document.querySelectorAll('[data-thumbnail]')].map(el => el.toDataURL())).size,
        hintCopy: !!document.querySelector('.navigation-hints'),
        annotation: (() => {
          const a = document.querySelector('.edit-annotation').getBoundingClientRect();
          const v = document.querySelector('.viewport').getBoundingClientRect();
          return a.left >= v.left && a.right <= v.right && a.top >= v.top && a.bottom <= v.bottom;
        })(),
      }));
      assert.equal(metrics.width, 1600, 'Unexpected horizontal overflow');
      assert.equal(metrics.height, 1000, 'Unexpected vertical overflow');
      assert.equal(metrics.scene.dpr, 2);
      assert.ok(metrics.scene.cubes > 1500 && metrics.scene.faces > 700, 'Voxel scene did not render');
      assert.equal(metrics.tools.length, 6);
      assert.equal(metrics.mirror, 3);
      assert.equal(metrics.layers, 4);
      assert.ok(metrics.connected && metrics.annotation);
      assert.ok(metrics.session && metrics.panelsFit && metrics.viewportFits && metrics.sessionFits,
        `Panel layout overflow: ${JSON.stringify(metrics)}`);
      assert.equal(metrics.black, theme === 'dark' ? 'rgb(0, 0, 0)' : 'rgb(255, 255, 255)');
      assert.equal(metrics.panel, metrics.black);
      assert.ok(metrics.fonts && !metrics.hintCopy);
      assert.ok(metrics.minimumText >= 12, `Unreadable text: ${metrics.minimumText}px`);
      assert.ok(metrics.contrast.text >= 4.5 && metrics.contrast.controls >= 3, `Contrast failure: ${JSON.stringify(metrics.contrast)}`);
      assert.ok(metrics.tableAligned, 'Transaction cells must align with their headings');
      assert.equal(metrics.distinctThumbs, 3, 'Agent render angles must differ');
      assert.deepEqual(errors, [], 'Browser errors');
      const path = `${output}/concept-${concept}-${theme}.png`;
      await page.screenshot({ path, fullPage: false, animations: 'disabled' });
      assert.ok((await stat(path)).size > 100_000, 'Screenshot appears empty');
      rendered.push(path);
      console.log(`Rendered concept-${concept}-${theme}.png · 3200 × 2000 · ${metrics.scene.cubes} voxels · text ${metrics.contrast.text.toFixed(2)}:1 / controls ${metrics.contrast.controls.toFixed(2)}:1`);
    }
    // Exercise the functional preview controls after capturing the default state.
    await page.keyboard.press('e');
    await page.locator('[data-tool="erase"][aria-pressed="true"]').waitFor();
    await page.locator('[data-mirror="Y"]').click();
    assert.equal(await page.locator('[data-mirror="Y"]').getAttribute('aria-pressed'), 'true');
    const initialCaptures = await captureHashes(page);
    await page.locator('[data-layer-eye="Canopy"]').click();
    assert.equal(await page.locator('[data-layer-eye="Canopy"]').getAttribute('aria-pressed'), 'false');
    assert.deepEqual(await captureHashes(page), initialCaptures, 'Visibility must not overwrite captured evidence');
    assert.equal(await page.locator('[data-render-stale]').isVisible(), true);
    await page.locator('[data-layer-eye="Canopy"]').click();
    assert.equal(await page.locator('[data-render-stale]').isVisible(), false);
    const lock = page.locator('[data-layer="Canopy"] [data-layer-lock]');
    const unlockedIcon = await lock.locator('svg').innerHTML();
    await lock.click();
    assert.equal(await lock.getAttribute('aria-pressed'), 'true');
    assert.notEqual(await lock.locator('svg').innerHTML(), unlockedIcon);
    await lock.click();
    const swatch = page.locator('[data-color="#829a63"]');
    await swatch.focus();
    await page.keyboard.press('ArrowRight');
    assert.equal(await page.locator('[data-color][aria-pressed="true"]').getAttribute('data-color'), '#6e8657');
    assert.equal(await page.locator('[data-color][tabindex="0"]').count(), 1);
    await page.keyboard.press('ArrowLeft');
    if (concept === 'c') {
      await page.locator('[data-select-layer="Roof"]').click();
      assert.equal(await page.locator('[data-selected-layer]').textContent(), 'Roof');
      assert.equal(await page.locator('.selection-inspector [data-inspect-step]').isVisible(), false);
      assert.notEqual(await page.locator('[data-selection-count]').textContent(), '263 voxels');
      await page.locator('[data-select-layer="Canopy"]').click();
      assert.equal(await page.locator('[data-selection-count]').textContent(), '263 voxels');
    }
    const before = await page.evaluate(() => window.__voxelSceneStats.cubes);
    await page.locator('[data-inspect-step="18"]:visible').first().click();
    const inspected = JSON.parse(await page.locator('[data-history-diff]').textContent());
    assert.deepEqual(inspected.added.map(p => p.join(',')).sort(),
      [[8,13,-1],[8,14,-2],[7,15,-2],[7,15,-3],[8,14,-3],[8,14,-1]].map(p => p.join(',')).sort());
    assert.equal(inspected.state, 'applied');
    assert.equal(inspected.revision, 18);
    await page.locator('[data-history-revert]').click();
    assert.equal(await page.evaluate(() => window.__voxelSceneStats.cubes), before - 6);
    const reverted = JSON.parse(await page.locator('[data-history-diff]').textContent());
    assert.equal(reverted.state, 'undone');
    assert.equal(reverted.revision, 18, 'Original change revision must be immutable');
    assert.equal(reverted.document_revision, 19);
    assert.equal(await page.locator('[data-change-evidence]').isVisible(), false);
    if (concept === 'c') assert.equal(await page.locator('[data-session-total]').textContent(), '+24 −0');
    await page.locator('[data-history-revert]').click();
    assert.equal(await page.evaluate(() => window.__voxelSceneStats.cubes), before);
    if (concept === 'c') assert.equal(await page.locator('[data-session-total]').textContent(), '+30 −0');
    await page.keyboard.press('Escape');
    assert.equal(await page.locator('.history-dialog').evaluate(el => el.open), false);
    assert.equal(await page.evaluate(() => document.activeElement.hasAttribute('data-inspect-step')), true);
    await page.locator('.session-step [data-step-undo="18"]').click();
    assert.equal(await page.evaluate(() => window.__voxelSceneStats.cubes), before - 6);
    assert.deepEqual(await captureHashes(page), initialCaptures, 'Undo must not overwrite captured evidence');
    assert.equal(await page.locator('[data-render-revision]').textContent(), 'r018');
    assert.equal(await page.locator('[data-render-stale]').isVisible(), true);
    await page.locator('[data-refresh-renders]').click();
    const undoneCaptures = await captureHashes(page);
    assert.notDeepEqual(undoneCaptures, initialCaptures, 'Refresh must capture the edited model');
    assert.equal(await page.locator('[data-render-revision]').textContent(), 'r021');
    assert.equal(await page.locator('[data-render-stale]').isVisible(), false);
    await page.locator('.session-step [data-step-undo="18"]').click();
    assert.equal(await page.evaluate(() => window.__voxelSceneStats.cubes), before);
    assert.deepEqual(await captureHashes(page), undoneCaptures, 'Redo must preserve the last explicit capture');
    assert.equal(await page.locator('[data-render-stale]').isVisible(), true);
    assert.equal(await page.locator('[data-change-evidence]').isVisible(), true);
    await page.locator('.session-step [data-step-undo="17"]').click();
    assert.equal(await page.evaluate(() => window.__voxelSceneStats.cubes), before - 24);
    await page.locator('.session-step [data-step-undo="17"]').click();
    assert.equal(await page.evaluate(() => window.__voxelSceneStats.cubes), before);
    await page.locator('[data-history-undo]').click();
    await page.locator('[data-history-undo]').click();
    assert.equal(await page.evaluate(() => window.__voxelSceneStats.cubes), before - 30);
    await page.locator('[data-history-redo]').click();
    await page.locator('[data-history-redo]').click();
    assert.equal(await page.evaluate(() => window.__voxelSceneStats.cubes), before);
    await page.locator('[data-refresh-renders]').click();
    const refreshedCaptures = await captureHashes(page);
    assert.equal(await page.locator('[data-render-stale]').isVisible(), false);
    const initialAxis = await page.locator('[data-axis="x"] circle').getAttribute('cx');
    await page.locator('.viewport > canvas').focus();
    await page.keyboard.press('ArrowRight');
    assert.notEqual(await page.locator('[data-axis="x"] circle').getAttribute('cx'), initialAxis);
    assert.equal(await page.locator('[data-follow]').getAttribute('aria-pressed'), 'false');
    assert.equal(await page.locator('[data-view-label]').textContent(), 'Custom perspective');
    assert.equal(await page.locator('[data-view-angle][aria-pressed="true"]').count(), 0);
    assert.deepEqual(await captureHashes(page), refreshedCaptures, 'Camera motion must not change evidence');
    await page.locator('[data-zoom-select]').selectOption('125');
    assert.equal(await page.locator('[data-zoom-select]').inputValue(), '125');
    await page.locator('.viewport > canvas').focus();
    await page.keyboard.press('-');
    assert.equal(await page.locator('[data-zoom-select]').inputValue(), '115');
    await page.keyboard.press('Home');
    assert.equal(await page.locator('[data-zoom-select]').inputValue(), '100');
    assert.equal(await page.locator('[data-view-label]').textContent(), 'Front perspective');
    await page.locator('[data-toggle-grid]').click();
    assert.equal(await page.locator('[data-toggle-grid]').getAttribute('aria-pressed'), 'false');
    await page.locator('[data-toggle-grid]').click();
    assert.equal(await page.locator('[data-toggle-grid]').evaluate(el => getComputedStyle(el, '::after').height), '2px');
    await page.locator('[data-follow]').click();
    assert.equal(await page.locator('[data-follow]').getAttribute('aria-pressed'), 'true');
    await page.locator('[data-follow]').click();
    assert.equal(await page.locator('[data-follow]').getAttribute('aria-pressed'), 'false');
    await page.locator('[data-view-angle="back"]').click();
    assert.equal(await page.locator('[data-view-angle="back"]').getAttribute('aria-pressed'), 'true');
    await page.locator('[data-follow]').click();
    assert.equal(await page.locator('[data-view-angle="front"]').getAttribute('aria-pressed'), 'true');
    const viewport = await page.locator('.viewport > canvas').boundingBox();
    await page.mouse.move(viewport.x + 100, viewport.y + 100);
    await page.mouse.down();
    await page.mouse.move(viewport.x + 180, viewport.y + 120);
    await page.mouse.up();
    assert.equal(await page.locator('[data-view-angle][aria-pressed="true"]').count(), 0);
    assert.equal(await page.locator('[data-follow]').getAttribute('aria-pressed'), 'false');
    await page.locator('[data-reset-view]').first().click();
    assert.equal(await page.locator('[data-view-angle="front"]').getAttribute('aria-pressed'), 'true');
    await page.locator('[data-pause]').click();
    assert.equal(await page.locator('[data-running-label]').textContent(), 'Paused');
    assert.equal(await page.locator('[data-session-state]').textContent(), 'Paused');
    await page.keyboard.press('Meta+k');
    await page.locator('.command-dialog[open]').waitFor();
    await page.locator('.command-search input').fill('no such command');
    assert.equal(await page.locator('.command-empty').isVisible(), true);
    await page.locator('.command-search input').fill('');
    await page.keyboard.press('ArrowDown');
    assert.equal(await page.locator('.command-search input').evaluate(el => el === document.activeElement), true);
    await page.keyboard.type('Paint');
    await page.keyboard.press('Enter');
    assert.equal(await page.locator('[data-tool="paint"]').getAttribute('aria-pressed'), 'true');
    assert.equal(await page.locator('.command-dialog').evaluate(el => el.open), false);
    await page.locator('[data-pause]').click();
    assert.equal(await page.locator('[data-session-state]').textContent(), 'Live');
    if (concept === 'b') {
      assert.equal(await page.locator('[data-copy-instruction]').isDisabled(), true);
      await page.evaluate(() => Object.defineProperty(navigator, 'clipboard', {configurable:true, value:{writeText: async text => {window.__copiedDraft = text;}}}));
      await page.locator('#instruction').fill('Add a mossy stone beside the path.');
      await page.locator('[data-copy-instruction]').click();
      assert.equal(await page.evaluate(() => window.__copiedDraft), 'Add a mossy stone beside the path.');
      assert.equal(await page.locator('.toast').textContent(), 'Instruction copied');
      await page.evaluate(() => navigator.clipboard.writeText = async () => {throw new Error('Clipboard unavailable');});
      await page.locator('[data-copy-instruction]').click();
      assert.equal(await page.locator('#instruction').evaluate(el => el === document.activeElement && el.selectionEnd - el.selectionStart === el.value.length), true);
    }
    await page.locator('[data-theme-toggle]').click();
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'dark');
    assert.deepEqual(errors, []);
    await page.close();
  }
} finally {
  await browser.close();
}
assert.equal(rendered.length, 6);
console.log('Verified all 6 PNGs and preview controls.');
