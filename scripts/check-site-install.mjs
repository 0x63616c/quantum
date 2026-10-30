// Serve site/ first. Exercise host tabs, copied commands and cold font loading.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const base = process.env.VOXELITE_SITE_URL || 'http://127.0.0.1:8090';
const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
const server = {
  command: '/Applications/Voxelite.app/Contents/MacOS/voxelite-mcp',
  args: ['--root', '${workspaceFolder}']
};
try {
  for (const width of [1440, 390, 320]) {
    for (const theme of ['dark', 'light']) {
      const page = await browser.newPage({ viewport: { width, height: 900 }, colorScheme: theme });
      await page.context().grantPermissions(['clipboard-read', 'clipboard-write']);
      const errors = [], external = [];
      page.on('pageerror', error => errors.push(error.message));
      page.on('request', request => {
        if (!request.url().startsWith(base)) external.push(request.url());
      });
      for (const route of ['/docs/mcp/', '/docs/skills/']) {
        await page.goto(base + route);
        const tabs = page.getByRole('tab');
        assert.deepEqual(await tabs.allTextContents(), ['Cursor', 'Claude Code', 'Codex', 'VS Code', 'Other']);
        assert.equal(await page.getByRole('tabpanel').count(), 1);
        const cursor = new URL(await page.locator('.cursor-install').getAttribute('href'));
        assert.equal(cursor.searchParams.get('name'), 'voxelite');
        assert.equal(Buffer.from(cursor.searchParams.get('config'), 'base64').toString(), JSON.stringify(server));
        assert.equal(await page.locator(`.badge-${theme}`).isVisible(), true);
        assert.equal(await page.locator(`.badge-${theme === 'dark' ? 'light' : 'dark'}`).isVisible(), false);
        const vscode = await page.locator('#install-vscode a.button').getAttribute('href');
        assert.equal(vscode, 'vscode:mcp/install?' + encodeURIComponent(JSON.stringify({ name: 'voxelite', ...server })));
        await tabs.first().focus();
        await page.keyboard.press('ArrowRight');
        assert.equal(await page.locator(':focus').textContent(), 'Claude Code');
        assert.equal(await page.getByRole('tab', { name: 'Claude Code' }).getAttribute('aria-selected'), 'true');
        const panel = page.getByRole('tabpanel');
        const command = await panel.locator('pre').textContent();
        assert.ok(command.startsWith('claude mcp add voxelite -- '));
        assert.ok(command.endsWith('--root "$PWD"'));
        await panel.getByRole('button', { name: 'Copy code example' }).click();
        assert.equal(await page.evaluate(() => navigator.clipboard.readText()), command);
        await page.getByRole('tab', { name: 'Claude Code' }).focus();
        await page.keyboard.press('End');
        assert.equal(await page.locator(':focus').textContent(), 'Other');
        assert.deepEqual(JSON.parse(await page.getByRole('tabpanel').locator('pre').textContent()), {
          mcpServers: { voxelite: { ...server, args: ['--root', '<project dir>'] } }
        });
        await page.keyboard.press('ArrowRight');
        assert.equal(await page.locator(':focus').textContent(), 'Cursor');
        await page.keyboard.press('ArrowLeft');
        assert.equal(await page.locator(':focus').textContent(), 'Other');
        await page.keyboard.press('Home');
        assert.equal(await page.locator(':focus').textContent(), 'Cursor');
        for (const tab of await tabs.all()) {
          await tab.click();
          assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
        }
        if (route === '/docs/skills/') {
          for (const name of ['voxelite-workflow', 'voxelite-palettes', 'voxelite-primitives', 'voxelite-render-review']) {
            const section = page.locator(`section:has(> h3#${name})`);
            await section.getByRole('button', { name: 'Copy code example' }).click();
            assert.equal(await page.evaluate(() => navigator.clipboard.readText()), `npx skills add 0x63616c/voxelite --skill ${name}`);
          }
        }
      }
      await page.goto(base + '/docs/mcp/#install-codex');
      assert.equal(await page.getByRole('tab', { name: 'Codex', exact: true }).getAttribute('aria-selected'), 'true');
      await page.evaluate(() => Object.defineProperty(navigator, 'clipboard', { value: undefined }));
      const copy = page.getByRole('tabpanel').getByRole('button', { name: 'Copy code example' }).first();
      await copy.click();
      assert.equal(await copy.textContent(), 'Select code to copy');
      assert.deepEqual(errors, []);
      assert.deepEqual(external, []);
      await page.close();
    }
  }
  const plain = await browser.newPage({ javaScriptEnabled: false, viewport: { width: 390, height: 900 } });
  for (const route of ['/docs/mcp/', '/docs/skills/']) {
    await plain.goto(base + route);
    for (const panel of await plain.locator('[data-install-panel]').all()) assert.equal(await panel.isVisible(), true);
    assert.equal(await plain.locator('.copy-code, [role=tab]').count(), 0);
    assert.equal(await plain.locator('.cursor-install').isVisible(), true);
    assert.equal(await plain.getByRole('link', { name: 'Install in VS Code', exact: true }).isVisible(), true);
  }
  await plain.close();

  // A fresh context and intercepted fonts simulate a cold load on a slow link.
  for (const route of ['/', '/download/', '/docs/mcp/', '/docs/skills/', '/404.html']) {
    const page = await browser.newPage();
    let release;
    const gate = new Promise(resolve => { release = resolve; });
    await page.route('**/assets/fonts/*.woff2', async request => { await gate; await request.continue(); });
    await page.goto(base + route, { waitUntil: 'domcontentloaded' });
    await page.waitForTimeout(250); // Let optional fonts' brief display period expire.
    const heading = page.locator('h1');
    const before = await heading.boundingBox();
    release();
    await page.waitForLoadState('networkidle');
    await page.evaluate(() => document.fonts.ready);
    assert.deepEqual(await heading.boundingBox(), before, `No late font reflow on ${route}`);
    const fonts = await page.evaluate(() => [...document.fonts].filter(font => font.family === 'Geist' || font.family === 'Geist Mono').map(font => font.display));
    assert.deepEqual(fonts, ['optional', 'optional']);
    await page.close();
  }
  console.log('Install tabs, keyboard navigation, both themes, 320–1440px widths, exact deeplinks, clipboard success/failure, no-JS instructions and delayed fonts passed.');
} finally {
  await browser.close();
}
