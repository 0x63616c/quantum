// Local browser smoke/capture. Run after build-web.sh and start a static server.
import { chromium } from 'playwright';
import { mkdir, writeFile } from 'node:fs/promises';
const base = process.env.VOXELITE_WEB_URL || 'http://127.0.0.1:8080';
const browser = await chromium.launch({
  headless: true,
  args: ['--enable-unsafe-webgpu', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--no-sandbox'],
});
await mkdir('screenshots', { recursive: true });
const report = { browser: browser.version(), captures: [] };
try {
  for (const [theme, backend] of [['dark', 'auto'], ['light', 'auto'], ['dark', 'webgl']]) {
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 });
    const errors = [];
    const messages = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { messages.push(`${message.type()}: ${message.text()}`); });
    await page.goto(`${base}/?theme=${theme}&backend=${backend}`);
    await page.waitForSelector('canvas', { timeout: 120_000 });
    await page.waitForTimeout(10_000);
    const suffix = backend === 'webgl' ? '-webgl' : '';
    await page.screenshot({ path: `screenshots/web-${theme}${suffix}.png` });
    report.captures.push({ theme, backend, errors, messages });
    console.log(JSON.stringify(report.captures.at(-1)));
    await page.close();
  }
  await writeFile('screenshots/web-verification.json', JSON.stringify(report, null, 2) + '\n');
  if (report.captures.some(c => c.errors.length || c.messages.some(m => /^error:/.test(m)))) {
    throw new Error('Browser errors: see screenshots/web-verification.json');
  }
} finally { await browser.close(); }
