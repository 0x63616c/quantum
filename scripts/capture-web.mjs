// Local browser smoke/capture. Run after build-web.sh and start a static server.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
const base = process.env.VOXELITE_WEB_URL || 'http://127.0.0.1:8080';
await mkdir('screenshots', { recursive: true });
const report = { captures: [], workflows: [] };
const common = ['--no-sandbox', '--enable-unsafe-webgpu', '--enable-unsafe-swiftshader'];
const configurations = [
  { backend: 'auto', args: ['--use-angle=vulkan', '--use-vulkan=swiftshader', '--enable-features=Vulkan', '--disable-vulkan-surface'] },
  { backend: 'webgl', args: ['--use-angle=swiftshader'] },
  { backend: 'fallback', args: ['--use-angle=swiftshader'] },
];
for (const configuration of configurations) {
  const browser = await chromium.launch({ headless: true, args: [...common, ...configuration.args] });
  try {
    report.browser = browser.version();
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 });
    const errors = [];
    const messages = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => {
      // Keep diagnostics useful without serializing every GPU limit.
      messages.push(`${message.type()}: ${message.text().split(', limits=')[0]}`);
    });
    if (configuration.backend === 'fallback') {
      await page.addInitScript(() => Object.defineProperty(navigator, 'gpu', { value: undefined }));
    }
    const backend = configuration.backend === 'webgl' ? 'webgl' : 'auto';
    await page.goto(`${base}/?theme=dark&backend=${backend}`);
    await page.waitForFunction(() => document.title.includes('Autumn Workshop'), { timeout: 120_000 });
    await page.waitForTimeout(3000);
    const suffix = configuration.backend === 'auto' ? '' : `-${configuration.backend}`;
    const capture = async theme => {
      const bytes = await page.screenshot({ path: `screenshots/web-${theme}${suffix}.png` });
      assert.ok(bytes.length > 100_000, 'Capture must contain rendered artwork, not a blank canvas');
      report.captures.push({ theme, backend: configuration.backend, bytes: bytes.length });
    };
    await capture('dark');
    // Toggle the actual shared GPUI toolbar control, without reloading.
    await page.mouse.click(1190, 63);
    await page.mouse.move(700, 80);
    await page.waitForTimeout(800);
    await capture('light');
    if (configuration.backend === 'auto') {
      const downloadFile = async action => {
        const pending = page.waitForEvent('download');
        await action();
        const download = await pending;
        assert.equal(await download.failure(), null);
        const data = await readFile(await download.path());
        return { name: download.suggestedFilename(), data };
      };
      const saved = await downloadFile(() => page.keyboard.press('Control+s'));
      const project = JSON.parse(saved.data.toString());
      assert.ok(saved.name.endsWith('.voxl'));
      assert.ok(saved.data.length > 100_000);
      report.workflows.push({ action: 'save', name: saved.name, bytes: saved.data.length });
      const command = async text => {
        await page.keyboard.press('Control+k');
        await page.waitForTimeout(100);
        await page.keyboard.type(text, { delay: 15 });
        await page.keyboard.press('Enter');
      };
      const glb = await downloadFile(() => command('Export GLB'));
      assert.equal(glb.data.subarray(0, 4).toString(), 'glTF');
      report.workflows.push({ action: 'export-glb', name: glb.name, bytes: glb.data.length });
      const png = await downloadFile(() => command('Export PNG'));
      assert.equal(png.data.subarray(0, 8).toString('hex'), '89504e470d0a1a0a');
      report.workflows.push({ action: 'export-png', name: png.name, bytes: png.data.length });
      const choose = page.waitForEvent('filechooser');
      await page.keyboard.press('Control+o');
      await (await choose).setFiles({ name: saved.name, mimeType: 'application/json', buffer: saved.data });
      await page.waitForTimeout(1200);
      const reopened = await downloadFile(() => page.keyboard.press('Control+s'));
      assert.deepEqual(JSON.parse(reopened.data.toString()), project);
      report.workflows.push({ action: 'upload-save-roundtrip', identical: true });
      const invalid = page.waitForEvent('filechooser');
      await page.keyboard.press('Control+o');
      await (await invalid).setFiles({ name: 'broken.voxl', mimeType: 'application/json', buffer: Buffer.from('{broken') });
      await page.waitForTimeout(300);
      const retained = await downloadFile(() => page.keyboard.press('Control+s'));
      assert.deepEqual(JSON.parse(retained.data.toString()), project);
      report.workflows.push({ action: 'invalid-upload-retains-project', identical: true });
    }
    const expected = configuration.backend === 'auto' ? 'BrowserWebGpu' : 'Gl';
    assert.ok(messages.some(m => m.includes(`successfully with ${expected}`)), `Expected ${expected} backend`);
    report.captures.at(-1).diagnostics = { errors, messages };
    assert.deepEqual(errors, []);
    assert.ok(!messages.some(m => m.startsWith('error:')), messages.join('\n'));
    console.log(`${configuration.backend}: rendered both themes and passed browser checks`);
  } finally {
    await browser.close();
    await writeFile('screenshots/web-verification.json', JSON.stringify(report, null, 2) + '\n');
  }
}
