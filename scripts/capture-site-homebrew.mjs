// Capture the installation sections; serve the selected revision's site first.
import { chromium } from 'playwright';
import { mkdir } from 'node:fs/promises';
const base = process.env.QUANTUM_SITE_URL || 'http://127.0.0.1:8090';
const output = process.argv[2] || 'screenshots/site/homebrew';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
try {
  for (const theme of ['dark', 'light']) {
    for (const width of [1440, 390]) {
      const page = await browser.newPage({ viewport: { width, height: 3000 }, colorScheme: theme, deviceScaleFactor: 1 });
      for (const [name, path, first, last] of [
        ['download', '/download/', '#install', '#first-launch'],
        ['getting-started', '/docs/getting-started/', '.lede', '#website'],
      ]) {
        await page.goto(base + path, { waitUntil: 'networkidle' });
        await page.evaluate(() => document.fonts.ready);
        const start = await page.locator(first).first().boundingBox();
        await page.locator(last).waitFor();
        const content = await page.locator('.docs-content').first().boundingBox();
        await page.screenshot({ path: `${output}/${name}-${width}-${theme}.png`,
          clip: { x: content.x, y: start.y, width: content.width, height: width === 1440 ? 1000 : 1800 } });
      }
      await page.close();
    }
  }
} finally {
  await browser.close();
}
