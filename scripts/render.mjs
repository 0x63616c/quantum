import { chromium } from 'playwright';
import { mkdir, stat } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';

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
        connected: document.body.textContent.includes('Agent connected'),
        session: !!document.querySelector('.agent-session, .c-console'),
        panelsFit: [...document.querySelectorAll('.a-inspector, .b-inspector, .c-inspector')]
          .every(el => el.scrollHeight <= el.clientHeight + 1),
        viewportFits: document.querySelector('.viewport').getBoundingClientRect().bottom <=
          document.querySelector('.statusbar').getBoundingClientRect().top + 1,
        sessionFits: (() => {
          const card = document.querySelector('.b-agent-card');
          if (!card) return true;
          const rect = card.getBoundingClientRect();
          return rect.top >= document.querySelector('.b-inspector').getBoundingClientRect().bottom + 8 &&
            rect.bottom <= document.querySelector('.statusbar').getBoundingClientRect().top;
        })(),
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
      assert.deepEqual(errors, [], 'Browser errors');
      const path = `${output}/concept-${concept}-${theme}.png`;
      await page.screenshot({ path, fullPage: false, animations: 'disabled' });
      assert.ok((await stat(path)).size > 100_000, 'Screenshot appears empty');
      rendered.push(path);
      console.log(`Rendered concept-${concept}-${theme}.png · 3200 × 2000 · ${metrics.scene.cubes} voxels`);
    }
    // Exercise the functional preview controls after capturing the default state.
    await page.keyboard.press('e');
    await page.locator('[data-tool="erase"][aria-pressed="true"]').waitFor();
    await page.locator('[data-mirror="Y"]').click();
    assert.equal(await page.locator('[data-mirror="Y"]').getAttribute('aria-pressed'), 'true');
    await page.locator('[data-layer-eye="Canopy"]').click();
    assert.equal(await page.locator('[data-layer-eye="Canopy"]').getAttribute('aria-pressed'), 'false');
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
