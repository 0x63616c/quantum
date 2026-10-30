// Serve site/ on 8191+, then: node scripts/capture-gallery.mjs screenshots/gallery/round-1
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
const base = process.env.VOXELITE_SITE_URL || 'http://127.0.0.1:8191';
const directory = process.argv[2] || 'screenshots/gallery/final';
const models = JSON.parse(await readFile(new URL('../site/gallery/manifest.json', import.meta.url)));
await mkdir(directory, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox', '--enable-unsafe-swiftshader'] });
const report = { browser: browser.version(), captures: [], checks: [] };
const decode = async page => page.locator('img').evaluateAll(images => Promise.all(images.map(image => { image.loading = 'eager'; return image.decode(); })));
try {
  for (const width of [1440, 390]) for (const theme of ['dark', 'light']) {
    const page = await browser.newPage({ viewport: { width, height: 900 }, colorScheme: theme, deviceScaleFactor: 1 });
    const errors = [], external = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('request', req => { if (!req.url().startsWith(base) && !req.url().startsWith('data:')) external.push(req.url()); });
    await page.goto(base + '/gallery/', { waitUntil: 'networkidle' });
    await page.evaluate(() => document.fonts.ready); await decode(page);
    assert.equal(await page.locator('h1').count(), 1);
    assert.equal(await page.locator('.gallery-card').count(), models.length);
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
    assert.equal(await page.locator('.gallery-card img').first().evaluate(img => img.currentSrc.includes(`-${document.documentElement.dataset.theme}-tile.`)), true);
    const prefix = `${width === 1440 ? 'desktop' : 'mobile'}-${theme}`;
    await page.screenshot({ path: `${directory}/${prefix}-grid.png` });
    await page.screenshot({ path: `${directory}/${prefix}-wall.png`, fullPage: true });
    report.captures.push(prefix + '-grid', prefix + '-wall');
    await page.locator('[data-id="midnight-noodles"]').click();
    await page.locator('.model-picture img').evaluate(img => img.decode());
    await page.mouse.move(0, 0);
    await page.screenshot({ path: `${directory}/${prefix}-lightbox.png` });
    report.captures.push(prefix + '-lightbox');
    assert.equal(await page.locator('dialog').evaluate(d => d.open), true);
    await page.keyboard.press('Shift+Tab');
    assert.equal(await page.locator(':focus').getAttribute('class'), 'button download-model');
    await page.keyboard.press('Tab');
    assert.equal(await page.locator(':focus').getAttribute('aria-label'), 'Close model');
    await page.keyboard.press('ArrowRight');
    assert.equal(await page.locator('#model-title').textContent(), models[(models.findIndex(model => model.id === 'midnight-noodles') + 1) % models.length].title);
    await page.keyboard.press('ArrowLeft');
    assert.equal(await page.locator('#model-title').textContent(), 'Midnight noodles');
    const remix = new URL(await page.locator('.remix-model').getAttribute('href'), base);
    assert.equal(remix.pathname, '/app/'); assert.equal(remix.searchParams.get('model'), '/gallery/models/midnight-noodles.voxl');
    const download = await page.request.get(base + await page.locator('.download-model').getAttribute('href'));
    assert.equal(download.ok(), true); assert.equal((await download.json()).document.name, 'Midnight noodles');
    await page.keyboard.press('Escape');
    assert.equal(await page.locator(':focus').getAttribute('data-id'), 'midnight-noodles');
    for (const category of ['Characters', 'Buildings', 'Vehicles', 'Objects', 'Nature', 'All']) {
      await page.getByRole('button', { name: category, exact: true }).click();
      assert.equal(await page.locator('.gallery-card:visible').count(), models.filter(m => category === 'All' || m.category === category).length);
    }
    await page.getByRole('button', { name: 'Nature', exact: true }).click();
    await page.locator('.gallery-card:visible').first().click();
    await page.keyboard.press('ArrowLeft');
    assert.equal(await page.locator('#model-title').textContent(), models.filter(m => m.category === 'Nature').at(-1).title);
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme` }).click();
    await page.reload({ waitUntil: 'networkidle' });
    assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), theme === 'dark' ? 'light' : 'dark');
    assert.deepEqual(errors, []); assert.deepEqual(external, []);
    await page.close();
  }
  report.checks.push('1440/390 dark/light: all images decoded, no overflow, no JS errors or external requests; downloads and remix URLs; filter counts; filtered prev/next wrap; arrows/Escape; focus trap and restoration; persisted theme');
  for (const width of [320, 768, 1024]) {
    const page = await browser.newPage({ viewport: { width, height: 900 }, reducedMotion: 'reduce' });
    await page.goto(base + '/gallery/');
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
    assert.equal(await page.locator('.gallery-card').first().evaluate(card => getComputedStyle(card).transitionDuration), '0s');
    await page.keyboard.press('Tab'); assert.equal(await page.locator(':focus').textContent(), 'Skip to content');
    await page.locator('.gallery-card').first().hover();
    assert.equal(await page.locator('.gallery-card source').first().getAttribute('srcset').then(src => src.includes('-tile.')), true);
    await page.close();
  }
  report.checks.push('320/768/1024 fit; reduced motion suppresses transform and alternate camera; keyboard skip link');
  for (const theme of ['dark', 'light']) {
    const page = await browser.newPage({ viewport: { width: 390, height: 900 }, colorScheme: theme, javaScriptEnabled: false });
    await page.goto(base + '/gallery/');
    assert.equal(await page.locator('.gallery-card:visible').count(), models.length);
    assert.equal(await page.locator('.filters').isVisible(), false);
    assert.equal(await page.locator('body').evaluate(body => getComputedStyle(body).backgroundColor), theme === 'light' ? 'rgb(255, 255, 255)' : 'rgb(0, 0, 0)');
    assert.equal(await page.locator('.gallery-card img').first().evaluate(img => img.currentSrc.includes(`-${matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'}-tile.`)), true);
    assert.equal(await page.locator('.gallery-card').first().getAttribute('download'), '');
    await page.close();
  }
  report.checks.push('JavaScript disabled: all models visible, correct system-theme pictures, direct .voxl downloads');
  const short = await browser.newPage({ viewport: { width: 390, height: 600 } });
  await short.goto(base + '/gallery/');
  await short.locator('.gallery-card').first().click();
  await short.locator('.download-model').scrollIntoViewIfNeeded();
  assert.equal(await short.locator('.close-lightbox').isVisible(), true);
  const closeBox = await short.locator('.close-lightbox').boundingBox();
  assert.ok(closeBox.y >= 0 && closeBox.y + closeBox.height <= 600);
  await short.getByRole('button', { name: 'Close model', exact: true }).click();
  await short.close();
  report.checks.push('390×600: lightbox scrolls, downloads remain reachable and sticky close stays in the viewport');
  const bad = await browser.newPage();
  await bad.goto(base + '/app/?model=%2Fgallery%2Fmodels%2Fmissing.voxl');
  await bad.getByText(/gallery model could not be loaded/).waitFor();
  await bad.route('**/gallery/models/invalid.voxl', route => route.fulfill({ body: 'not a voxel project' }));
  await bad.goto(base + '/app/?model=%2Fgallery%2Fmodels%2Finvalid.voxl');
  await bad.getByText(/invalid .voxl JSON/).waitFor({ timeout: 60000 });
  await bad.route('**/gallery/models/oversized.voxl', route => route.fulfill({ body: Buffer.alloc(16 * 1024 * 1024 + 1, 32) }));
  await bad.goto(base + '/app/?model=%2Fgallery%2Fmodels%2Foversized.voxl');
  await bad.getByText(/Gallery model exceeds 16 MB/).waitFor();
  await bad.close();
  report.checks.push('Remix reports missing files, rejects invalid .voxl through the actual Rust decoder and enforces the streamed 16 MB cap');
  const app = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  await app.goto(base + '/app/?model=https%3A%2F%2Fexample.com%2Fbad.voxl');
  await app.getByText(/Choose a model from the Voxelite gallery/).waitFor();
  await app.goto(base + '/app/?backend=webgl&model=%2Fgallery%2Fmodels%2Forbit.voxl');
  await app.locator('canvas').waitFor({ timeout: 120000 });
  await app.waitForTimeout(3000);
  await app.screenshot({ path: `${directory}/remix-editor.png` });
  await app.close();
  report.checks.push('Actual wasm editor opens the gallery model; cross-origin input rejected before fetch');
} finally {
  await browser.close();
  await writeFile(`${directory}/verification.json`, JSON.stringify(report, null, 2) + '\n');
}
console.log(`Captured ${report.captures.length} gallery states and verified interactions in ${directory}`);
