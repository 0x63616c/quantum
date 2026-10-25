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
      await page.close();
    }
  }
  const page = await browser.newPage({ viewport: { width: 390, height: 900 }, javaScriptEnabled: false });
  await page.goto(base);
  assert.equal(await page.getByRole('heading', { level: 1 }).textContent(), 'The voxel editorbuilt for AI agents.');
  assert.equal(await page.getByRole('link', { name: 'Open the desktop web editor', exact: false }).count(), 2);
  report.checks.push('No horizontal overflow, one h1, decoded images, theme toggle and persistence, no page errors, no-JavaScript navigation');
  await page.close();
} finally {
  await browser.close();
  await writeFile(`${directory}/verification.json`, JSON.stringify(report, null, 2) + '\n');
}
console.log(`Captured and verified ${report.captures.length} pages in ${directory}`);
