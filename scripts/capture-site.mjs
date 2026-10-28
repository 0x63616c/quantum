// Reproducible full-page site captures and interaction checks. Serve site/ first.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';
const base = process.env.QUANTUM_SITE_URL || 'http://127.0.0.1:8090';
const directory = process.argv[2] || 'screenshots/site';
await mkdir(directory, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
const report = { browser: browser.version(), captures: [], checks: [] };
async function checkHeader(page) {
  const controls = await page.locator('.header a, .header button').evaluateAll(elements =>
    elements.filter(element => element.checkVisibility()).map(element => {
      const box = element.getBoundingClientRect();
      return { label: element.textContent || element.ariaLabel, fits: box.left >= 0 && box.right <= innerWidth };
    }));
  assert.ok(controls.length >= 4, 'Brand, Docs, Download and theme control remain visible');
  for (const control of controls) assert.ok(control.fits, `${page.url()}: header ${control.label} fits viewport`);
  for (const href of ['/docs/', '/download/']) {
    assert.equal(await page.locator(`.header nav a[href="${href}"]`).isVisible(), true);
  }
  assert.equal((await page.locator('.theme-toggle').boundingBox()).width, 44, 'Reserve the full theme target');
}

async function checkTextContrast(control) {
  const contrast = await control.evaluate(element => {
    const rgb = color => color.match(/[\d.]+/g).slice(0, 3).map(Number);
    const luminance = color => rgb(color).map(value => {
      const channel = value / 255;
      return channel <= .04045 ? channel / 12.92 : ((channel + .055) / 1.055) ** 2.4;
    }).reduce((sum, channel, i) => sum + channel * [.2126, .7152, .0722][i], 0);
    const style = getComputedStyle(element);
    let background = element;
    while (getComputedStyle(background).backgroundColor === 'rgba(0, 0, 0, 0)') background = background.parentElement;
    const values = [luminance(style.color), luminance(getComputedStyle(background).backgroundColor)].sort((a, b) => b - a);
    return { ratio: (values[0] + .05) / (values[1] + .05), opacity: style.opacity };
  });
  assert.equal(contrast.opacity, '1', 'Enabled text controls keep full opacity');
  assert.ok(contrast.ratio >= 4.5, `Text contrast ${contrast.ratio.toFixed(2)}:1 meets AA`);
}

async function checkPressed(page, control) {
  await control.hover();
  await page.mouse.down();
  try {
    assert.equal(await control.evaluate(element => element.matches(':active')), true);
    await checkTextContrast(control);
  } finally {
    // Release away from the target so a link does not navigate.
    await page.mouse.move(0, 0);
    await page.mouse.up();
  }
}
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
        const session = await page.locator('.mobile-session').boundingBox();
        assert.ok(session.y + session.height <= 900, 'Artwork and complete native session fit the first phone viewport');
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
  for (const width of [1440, 1101, 1100, 1024, 1001, 1000, 851, 850, 768, 701, 700, 390, 320]) {
    for (const theme of ['dark', 'light']) {
      const page = await browser.newPage({ viewport: { width, height: 900 }, colorScheme: theme, reducedMotion: 'reduce' });
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      for (const route of ['/download/', '/404.html', '/docs/', '/docs/getting-started/', '/docs/mcp/', '/docs/format/', '/docs/skills/', '/docs/plugins/', '/changelog/', '/blog/', '/blog/2026-10-28-rename/']) {
        const response = await page.goto(base + route);
        assert.equal(response.status(), 200);
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `${route} fits ${width}`);
        assert.equal(await page.getByRole('heading', { level: 1 }).count(), 1);
        await checkHeader(page);
        assert.deepEqual(await page.locator('.docs-content code').evaluateAll(elements =>
          elements.filter(element => element.checkVisibility() && parseFloat(getComputedStyle(element).fontSize) < 12)
            .map(element => element.textContent)), [], `${route}: docs code is at least 12px`);
        if (route === '/docs/mcp/') {
          assert.equal(await page.locator('.tool th:nth-child(-n+2), .tool td:nth-child(-n+2)').evaluateAll(cells =>
            cells.every(cell => getComputedStyle(cell).whiteSpace === 'nowrap' && getComputedStyle(cell).overflowWrap === 'normal')), true);
          const table = page.locator('#quantum_comments_add + p + .table-scroll');
          assert.equal(await table.evaluate(element => getComputedStyle(element).overflowX), 'auto');
          if (width <= 390) {
            assert.equal(await table.evaluate(element => {
              element.scrollLeft = 100;
              return element.scrollWidth > element.clientWidth && element.scrollLeft > 0;
            }), true, 'Narrow MCP tables scroll inside their wrapper');
          }
        }
        const sections = await page.locator('.documentation .docs-content h2[id]').evaluateAll(headings => headings.map(heading => `#${heading.id}`));
        if (sections.length) {
          const index = page.locator(width < 1100 ? '.docs-mobile-nav nav[aria-label="Page sections"]' : '.docs-toc nav');
          assert.deepEqual(await index.locator('a').evaluateAll(links => links.map(link => link.getAttribute('href'))), sections, 'Section index matches the article');
        }
        if ((route === '/changelog/' || route.startsWith('/blog/')) && (width === 1440 || width === 390)) {
          await page.evaluate(() => document.fonts.ready);
          const slug = route.split('/').filter(Boolean).join('-');
          const name = `${slug}-${width === 1440 ? 'desktop' : 'mobile'}-${theme}`;
          await page.screenshot({ path: `${directory}/${name}.png` });
          report.captures.push({ name, width, height: 900, fullPage: false });
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
            const sectionLinks = menus.locator('nav[aria-label="Page sections"] a');
            for (let i = 0; i < await sectionLinks.count(); i++) {
              await page.keyboard.press('Tab');
              assert.equal(await menus.locator('details[open]').count(), 1, 'Focus inside keeps the menu open');
            }
            await page.keyboard.press('Tab');
            assert.equal(await page.locator(':focus').textContent(), 'Copy page as Markdown');
            assert.equal(await menus.locator('details[open]').count(), 0, 'Tab out closes the menu without moving focus');
            assert.equal(await page.locator(':focus').evaluate(element => {
              const box = element.getBoundingClientRect();
              return element.contains(document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2));
            }), true, 'The newly focused control is unobscured');
            const summary = menus.getByText('On this page', { exact: true });
            await summary.focus();
            await page.keyboard.press('Enter');
            await page.keyboard.press('Shift+Tab');
            assert.equal(await page.locator(':focus').textContent(), 'Browse docs');
            assert.equal(await menus.locator('details[open]').count(), 0, 'Reverse Tab out also closes the menu');
            await checkPressed(page, menus.getByText('Browse docs', { exact: true }));
            await checkPressed(page, page.getByRole('button', { name: 'Copy page as Markdown', exact: true }));
            await summary.focus();
            await page.keyboard.press('Enter');
            await menus.getByRole('link', { name: 'First model through MCP', exact: true }).click();
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
      await checkHeader(page);
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `Homepage fits ${width}`);
      await page.keyboard.press('Tab');
      assert.equal(await page.locator(':focus').textContent(), 'Skip to content');
      await page.locator('.skip').hover();
      await checkTextContrast(page.locator('.skip'));
      await checkPressed(page, page.locator('.skip'));
      await checkPressed(page, page.locator('.header nav a[href="/docs/"]'));
      assert.equal(await page.evaluate(() => getComputedStyle(document.documentElement).scrollBehavior), 'auto');
      assert.deepEqual(errors, [], 'No documentation or homepage runtime errors');
      await page.close();
    }
  }
  report.checks.push('Every page type and all docs routes fit 320–1440px, including 701/768px and both sides of header breakpoints; visible header controls stay in bounds with a full 44px theme target in both themes');
  report.checks.push('Docs code stays at least 12px; MCP identifiers/status labels stay whole and narrow tables scroll; focused/hovered/pressed skip links and pressed navigation/buttons/summaries keep AA contrast');
  report.checks.push('Generated section indexes match article headings; mobile docs menus support mutual exclusion, keyboard, Escape with focus restoration, outside dismissal and anchor focus; first mobile build example fits without scrolling');
  const clipboard = await browser.newPage();
  await clipboard.context().grantPermissions(['clipboard-read', 'clipboard-write']);
  await clipboard.goto(base + '/docs/getting-started/');
  const code = await clipboard.locator('.docs-content pre').first().textContent();
  await clipboard.getByRole('button', { name: 'Copy code example' }).first().click();
  assert.equal(await clipboard.evaluate(() => navigator.clipboard.readText()), code);
  report.checks.push('Documentation Copy code writes the complete example to clipboard');
  const clipboardErrors = [];
  clipboard.on('pageerror', error => clipboardErrors.push(error.message));
  for (const route of ['/docs/', '/docs/getting-started/', '/docs/mcp/', '/docs/format/', '/docs/skills/']) {
    await clipboard.goto(base + route);
    const raw = route.replace(/\/$/, '.md');
    const expected = await (await clipboard.request.get(base + raw)).text();
    const alias = await clipboard.request.get(base + route + 'index.md');
    assert.equal(alias.status(), 200);
    assert.equal(await alias.text(), expected);
    assert.equal(await clipboard.getByRole('link', { name: 'View .md', exact: true }).getAttribute('href'), raw);
    const copy = clipboard.getByRole('button', { name: 'Copy page as Markdown', exact: true });
    await copy.focus();
    await clipboard.keyboard.press('Enter');
    await clipboard.getByRole('status').filter({ hasText: /^Copied$/ }).waitFor();
    assert.equal(await clipboard.evaluate(() => navigator.clipboard.readText()), expected);
    assert.equal(await copy.isEnabled(), true);
  }
  await clipboard.route('**/docs/skills.md', route => route.fulfill({ status: 503, body: 'Unavailable' }));
  await clipboard.getByRole('button', { name: 'Copy page as Markdown', exact: true }).click();
  await clipboard.getByRole('status').filter({ hasText: 'Could not copy. Use View .md.' }).waitFor();
  await clipboard.unroute('**/docs/skills.md');
  await clipboard.context().clearPermissions();
  await clipboard.context().grantPermissions(['clipboard-read']);
  await clipboard.evaluate(() => Object.defineProperty(navigator, 'clipboard', {
    value: { write: async () => { throw new DOMException('Permission denied', 'NotAllowedError'); } }
  }));
  await clipboard.getByRole('button', { name: 'Copy page as Markdown', exact: true }).click();
  await clipboard.getByRole('status').filter({ hasText: 'Could not copy. Use View .md.' }).waitFor();
  assert.equal(await clipboard.getByRole('button', { name: 'Copy page as Markdown', exact: true }).isEnabled(), true);
  // Deny the clipboard before completing the download. A later fetch rejection
  // must still be handled even when the browser never reads the ClipboardItem.
  for (const failure of ['http', 'network', 'unexpected-content']) {
    let receiveDownload;
    const downloadReady = new Promise(resolve => { receiveDownload = resolve; });
    await clipboard.route('**/docs/skills.md', route => receiveDownload(route));
    await clipboard.getByRole('button', { name: 'Copy page as Markdown', exact: true }).click();
    const download = await downloadReady;
    await clipboard.getByRole('status').filter({ hasText: 'Could not copy. Use View .md.' }).waitFor();
    if (failure === 'network') await download.abort();
    else await download.fulfill({ status: failure === 'http' ? 503 : 200, body: '<html>Unavailable</html>' });
    await clipboard.waitForLoadState('networkidle');
    assert.deepEqual(clipboardErrors, [], `No unhandled rejection after delayed ${failure} failure`);
    await clipboard.unroute('**/docs/skills.md');
  }
  await clipboard.context().grantPermissions(['clipboard-read', 'clipboard-write']);
  await clipboard.reload();
  await clipboard.evaluate(() => Object.defineProperty(window, 'ClipboardItem', { value: undefined }));
  const fallbackMarkdown = await (await clipboard.request.get(base + '/docs/skills.md')).text();
  await clipboard.getByRole('button', { name: 'Copy page as Markdown', exact: true }).click();
  await clipboard.getByRole('status').filter({ hasText: /^Copied$/ }).waitFor();
  assert.equal(await clipboard.evaluate(() => navigator.clipboard.readText()), fallbackMarkdown);
  assert.deepEqual(clipboardErrors, []);
  report.checks.push('All doc Markdown aliases return identical content; keyboard copying preserves the complete page including schemas; writeText fallback works; simultaneous clipboard denial and delayed HTTP, network or content failures remain handled');
  await clipboard.close();
  const page = await browser.newPage({ viewport: { width: 390, height: 900 }, javaScriptEnabled: false });
  await page.goto(base);
  assert.equal(await page.getByRole('heading', { level: 1 }).textContent(), 'The voxel editorbuilt for AI agents.');
  assert.equal(await page.locator('.header nav').getByRole('link', { name: 'Download', exact: true }).getAttribute('href'), '/download/');
  await page.locator('label[for="proof-before"]').click();
  assert.equal(await page.locator('.proof-before').isVisible(), true);
  assert.equal(await page.locator('.proof-after').isVisible(), false);
  await page.locator('label[for="proof-after"]').click();
  assert.equal(await page.locator('.proof-after').isVisible(), true);
  await page.goto(base + '/docs/getting-started/');
  await page.locator('.docs-mobile-nav summary').filter({ hasText: 'On this page' }).click();
  await page.locator('.docs-mobile-nav').getByRole('link', { name: 'First model through MCP', exact: true }).click();
  assert.equal(new URL(page.url()).hash, '#first-model');
  await page.goto(base + '/docs/getting-started/');
  await page.getByText('Browse docs', { exact: true }).click();
  await page.locator('.docs-mobile-nav').getByRole('link', { name: 'The .voxl format', exact: true }).click();
  assert.equal(new URL(page.url()).pathname, '/docs/format/');
  await page.goto(base + '/docs/getting-started/');
  await page.getByRole('link', { name: 'Connect MCP', exact: false }).click();
  assert.equal(new URL(page.url()).hash, '#connect');
  assert.equal(await page.getByRole('button', { name: 'Copy page as Markdown', exact: true }).count(), 0);
  const rawResponse = await Promise.all([
    page.waitForResponse(response => response.url() === base + '/docs/getting-started.md'),
    page.getByRole('link', { name: 'View .md', exact: true }).click()
  ]);
  assert.equal(rawResponse[0].status(), 200);
  assert.ok((await rawResponse[0].text()).startsWith('# Getting started\n'));
  report.checks.push('View .md works with JavaScript disabled');
  report.checks.push('Mobile comparison switches with keyboard, pointer and JavaScript disabled; no-JavaScript setup navigation');
  report.checks.push('No horizontal overflow, one h1, decoded images, theme toggle and persistence, no page errors, no-JavaScript navigation');
  await page.close();
} finally {
  await browser.close();
  await writeFile(`${directory}/verification.json`, JSON.stringify(report, null, 2) + '\n');
}
console.log(`Captured and verified ${report.captures.length} pages in ${directory}`);
