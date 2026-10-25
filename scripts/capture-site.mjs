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
      await page.locator('.theme-image').evaluateAll(images => Promise.all(images.map(image => image.decode())));
      assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), theme);
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, 'No horizontal overflow');
      assert.equal(await page.locator('h1').count(), 1);
      const name = `${width === 1440 ? 'desktop' : 'mobile'}-${theme}`;
      await page.screenshot({ path: `${directory}/${name}.png`, fullPage: true });
      report.captures.push({ name, width, height: 900, fullPage: true });
      await page.getByRole('button', { name: `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme` }).click();
      await page.reload({ waitUntil: 'networkidle' });
      assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), theme === 'dark' ? 'light' : 'dark');
      assert.deepEqual(errors, []);
      assert.deepEqual(external, [], 'No third-party requests');
      await page.close();
    }
  }
  for (const width of [1440, 390, 320]) {
    for (const theme of ['dark', 'light']) {
      const page = await browser.newPage({ viewport: { width, height: 900 }, colorScheme: theme, reducedMotion: 'reduce' });
      for (const route of ['/docs/', '/docs/getting-started/', '/docs/mcp/', '/docs/format/', '/docs/skills/']) {
        const response = await page.goto(base + route);
        assert.equal(response.status(), 200);
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `${route} fits ${width}`);
        assert.equal(await page.getByRole('heading', { level: 1 }).count(), 1);
        if (route === '/docs/getting-started/' && width !== 320) {
          await page.evaluate(() => document.fonts.ready);
          await page.screenshot({ path: `${directory}/docs-${width === 1440 ? 'desktop' : 'mobile'}-${theme}.png` });
        }
      }
      await page.goto(base);
      await page.keyboard.press('Tab');
      assert.equal(await page.locator(':focus').textContent(), 'Skip to content');
      assert.equal(await page.evaluate(() => getComputedStyle(document.documentElement).scrollBehavior), 'auto');
      await page.close();
    }
  }
  report.checks.push('All five docs routes fit 1440, 390 and 320px in both themes; keyboard skip link; reduced motion; no third-party homepage requests');
  const clipboard = await browser.newPage();
  await clipboard.context().grantPermissions(['clipboard-read', 'clipboard-write']);
  await clipboard.goto(base + '/docs/getting-started/');
  const code = await clipboard.locator('.docs-content pre').first().textContent();
  await clipboard.getByRole('button', { name: 'Copy preceding code example' }).first().click();
  assert.equal(await clipboard.evaluate(() => navigator.clipboard.readText()), code);
  report.checks.push('Documentation Copy code writes the complete example to clipboard');
  await clipboard.close();
  const page = await browser.newPage({ viewport: { width: 390, height: 900 }, javaScriptEnabled: false });
  await page.goto(base);
  assert.equal(await page.getByRole('heading', { level: 1 }).textContent(), 'The voxel editorbuilt for AI agents.');
  assert.equal(await page.getByRole('link', { name: 'Open the desktop web editor', exact: false }).count(), 1);
  report.checks.push('No horizontal overflow, one h1, decoded images, theme toggle and persistence, no page errors, no-JavaScript navigation');
  await page.close();
} finally {
  await browser.close();
  await writeFile(`${directory}/verification.json`, JSON.stringify(report, null, 2) + '\n');
}
console.log(`Captured and verified ${report.captures.length} pages in ${directory}`);
