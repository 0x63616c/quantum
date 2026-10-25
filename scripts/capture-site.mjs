// Reproducible full-page site captures and interaction checks. Serve site/ first.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';
const base = process.env.VOXELITE_SITE_URL || 'http://127.0.0.1:8090';
const directory = process.argv[2] || 'screenshots/site';
await mkdir(directory, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
const report = { browser: browser.version(), captures: [], checks: [] };
try {
  for (const width of [1440, 390]) {
    for (const theme of ['dark', 'light']) {
      const page = await browser.newPage({ viewport: { width, height: 900 }, colorScheme: theme, deviceScaleFactor: 1 });
      const errors = [];
      const external = [];
      page.on('request', request => { if (!request.url().startsWith(base) && !request.url().startsWith('data:')) external.push(request.url()); });
      page.on('pageerror', error => errors.push(error.message));
      await page.goto(base, { waitUntil: 'networkidle' });
      await page.evaluate(() => document.fonts.ready);
      await page.locator('.theme-image').evaluateAll(images => Promise.all(images.map(image => {
        // Hidden comparison panels can defer lazy loads indefinitely. Decode
        // every source before capturing either state, including below the fold.
        image.loading = 'eager';
        return image.decode();
      })));
      assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), theme);
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, 'No horizontal overflow');
      assert.equal(await page.locator('h1').count(), 1);
      const name = `${width === 1440 ? 'desktop' : 'mobile'}-${theme}`;
      await page.screenshot({ path: `${directory}/${name}.png`, fullPage: true });
      await page.screenshot({ path: `${directory}/${name}-viewport.png` });
      report.captures.push({ name: `${name}-viewport`, width, height: 900, fullPage: false });
      if (width === 390) {
        const before = page.getByRole('radio', { name: 'Before', exact: true });
        const after = page.getByRole('radio', { name: 'After +3 voxels', exact: true });
        assert.equal(await after.isChecked(), true);
        assert.equal(await page.locator('.proof-before').isVisible(), false);
        await after.focus();
        await page.keyboard.press('ArrowLeft');
        assert.equal(await before.isChecked(), true);
        assert.equal(await page.locator('.proof-before').isVisible(), true);
        assert.equal(await page.locator('.proof-after').isVisible(), false);
        await before.blur();
        await page.locator('.edit-proof').screenshot({ path: `${directory}/proof-before-${theme}.png` });
        await page.getByText('After +3 voxels', { exact: true }).click();
        assert.equal(await after.isChecked(), true);
        assert.equal(await page.locator('.proof-after').isVisible(), true);
        await after.blur();
        await page.locator('.edit-proof').screenshot({ path: `${directory}/proof-after-${theme}.png` });
        for (const state of ['before', 'after']) {
          report.captures.push({ name: `proof-${state}-${theme}`, width, element: '.edit-proof', state });
        }
      }
      report.captures.push({ name, width, height: 900, fullPage: true, pageHeight: await page.locator('body').evaluate(body => body.scrollHeight) });
      await page.getByRole('button', { name: `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme` }).click();
      await page.reload({ waitUntil: 'networkidle' });
      assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), theme === 'dark' ? 'light' : 'dark');
      assert.deepEqual(errors, []);
      assert.deepEqual(external, [], 'No third-party requests');
      await page.close();
    }
  }
  for (const width of [1440, 1024, 768, 390, 320]) {
    for (const theme of ['dark', 'light']) {
      const page = await browser.newPage({ viewport: { width, height: 900 }, colorScheme: theme, reducedMotion: 'reduce' });
      for (const route of ['/docs/', '/docs/getting-started/', '/docs/mcp/', '/docs/format/', '/docs/skills/']) {
        const response = await page.goto(base + route);
        assert.equal(response.status(), 200);
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `${route} fits ${width}`);
        assert.equal(await page.getByRole('heading', { level: 1 }).count(), 1);
        const sections = await page.locator('.docs-content h2[id]').evaluateAll(headings => headings.map(heading => `#${heading.id}`));
        if (sections.length) {
          const index = page.locator(width < 1100 ? '.docs-mobile-nav nav[aria-label="Page sections"]' : '.docs-toc nav');
          assert.deepEqual(await index.locator('a').evaluateAll(links => links.map(link => link.getAttribute('href'))), sections, 'Section index matches the article');
        }
        if (route === '/docs/getting-started/' && (width === 1440 || width === 390)) {
          await page.evaluate(() => document.fonts.ready);
          const name = `docs-${width === 1440 ? 'desktop' : 'mobile'}-${theme}`;
          await page.screenshot({ path: `${directory}/${name}.png` });
          report.captures.push({ name, width, height: 900, fullPage: false });
          if (width === 390) {
            const example = await page.locator('.code-example').first().boundingBox();
            assert.ok(example.y + example.height <= 900, 'First build example fits the phone viewport');
            assert.equal(await page.locator('.docs-content pre').first().evaluate(pre => pre.scrollWidth <= pre.clientWidth), true, 'First build command needs no horizontal scrolling');
            const menus = page.locator('.docs-mobile-nav');
            await menus.getByText('Browse docs', { exact: true }).click();
            await page.screenshot({ path: `${directory}/${name}-navigation.png` });
            await menus.getByText('On this page', { exact: true }).click();
            assert.equal(await menus.locator('details[open]').count(), 1, 'Only one navigation menu opens at a time');
            await page.screenshot({ path: `${directory}/${name}-sections.png` });
            for (const state of ['navigation', 'sections']) report.captures.push({ name: `${name}-${state}`, width, height: 900, fullPage: false });
            await page.keyboard.press('Escape');
            assert.equal(await menus.locator('details[open]').count(), 0);
            assert.equal(await page.locator(':focus').textContent(), 'On this page');
            await page.keyboard.press('Enter');
            await menus.getByRole('link', { name: 'Your first model through MCP', exact: true }).click();
            assert.equal(new URL(page.url()).hash, '#first-model');
            assert.equal(await page.locator(':focus').getAttribute('id'), 'first-model');
            assert.equal(await menus.locator('details[open]').count(), 0);
            await menus.getByText('Browse docs', { exact: true }).click();
            await page.locator('.header').click({ position: { x: 4, y: 4 } });
            assert.equal(await menus.locator('details[open]').count(), 0, 'Click outside dismisses navigation');
          }
        }
      }
      await page.goto(base);
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `Homepage fits ${width}`);
      await page.keyboard.press('Tab');
      assert.equal(await page.locator(':focus').textContent(), 'Skip to content');
      assert.equal(await page.evaluate(() => getComputedStyle(document.documentElement).scrollBehavior), 'auto');
      await page.close();
    }
  }
  report.checks.push('Homepage and all five docs routes fit 1440, 1024, 768, 390 and 320px in both themes; keyboard skip link; reduced motion; no third-party homepage requests');
  report.checks.push('Generated section indexes match article headings; mobile docs menus support mutual exclusion, keyboard, Escape with focus restoration, outside dismissal and anchor focus; first mobile build example fits without scrolling');
  const clipboard = await browser.newPage();
  await clipboard.context().grantPermissions(['clipboard-read', 'clipboard-write']);
  await clipboard.goto(base + '/docs/getting-started/');
  const code = await clipboard.locator('.docs-content pre').first().textContent();
  await clipboard.getByRole('button', { name: 'Copy code example' }).first().click();
  assert.equal(await clipboard.evaluate(() => navigator.clipboard.readText()), code);
  report.checks.push('Documentation Copy code writes the complete example to clipboard');
  await clipboard.close();
  const page = await browser.newPage({ viewport: { width: 390, height: 900 }, javaScriptEnabled: false });
  await page.goto(base);
  assert.equal(await page.getByRole('heading', { level: 1 }).textContent(), 'The voxel editorbuilt for AI agents.');
  assert.equal(await page.getByRole('link', { name: 'Open the desktop web editor', exact: false }).count(), 1);
  await page.getByText('Before', { exact: true }).click();
  assert.equal(await page.locator('.proof-before').isVisible(), true);
  assert.equal(await page.locator('.proof-after').isVisible(), false);
  await page.getByText('After +3 voxels', { exact: true }).click();
  assert.equal(await page.locator('.proof-after').isVisible(), true);
  await page.goto(base + '/docs/getting-started/');
  await page.getByText('On this page', { exact: true }).click();
  await page.locator('.docs-mobile-nav').getByRole('link', { name: 'Your first model through MCP', exact: true }).click();
  assert.equal(new URL(page.url()).hash, '#first-model');
  await page.goto(base + '/docs/getting-started/');
  await page.getByText('Browse docs', { exact: true }).click();
  await page.locator('.docs-mobile-nav').getByRole('link', { name: 'The .voxl format', exact: true }).click();
  assert.equal(new URL(page.url()).pathname, '/docs/format/');
  await page.goto(base + '/docs/getting-started/');
  await page.getByRole('link', { name: 'Connect MCP', exact: false }).click();
  assert.equal(new URL(page.url()).hash, '#connect');
  report.checks.push('Mobile comparison switches with keyboard, pointer and JavaScript disabled; no-JavaScript setup navigation');
  report.checks.push('No horizontal overflow, one h1, decoded images, theme toggle and persistence, no page errors, no-JavaScript navigation');
  await page.close();
} finally {
  await browser.close();
  await writeFile(`${directory}/verification.json`, JSON.stringify(report, null, 2) + '\n');
}
console.log(`Captured and verified ${report.captures.length} pages in ${directory}`);
