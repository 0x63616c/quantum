import assert from 'node:assert/strict';
import { readFile, stat } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';

const settle = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
const count = page => page.evaluate(() => window.__voxelSceneStats.cubes);
const hashes = async page => (await page.evaluate(() => [...document.querySelectorAll('[data-thumbnail]')].map(c=>c.toDataURL()))).map(s=>createHash('sha256').update(s).digest('hex'));
const menu = async (page,name) => { await page.locator(`.menubar [data-menu="${name}"]`).click(); };
const menuItem = (page,name) => page.getByRole('menuitem',{name,exact:true});

export async function renderConceptA(browser,root,output) {
  const files=[];
  for(const theme of ['dark','light']) {
    const page=await browser.newPage({viewport:{width:1600,height:1000},deviceScaleFactor:2,reducedMotion:'reduce'});
    const errors=[];
    page.on('pageerror',e=>errors.push(e.message));
    await page.route(/^https?:/,r=>{errors.push('Unexpected request: '+r.request().url());return r.abort();});
    await page.goto(pathToFileURL(`${root}/design/concepts/concept-a/index.html`).href);
    await page.waitForFunction(()=>window.__voxelSceneReady && window.__aPreview);
    await page.evaluate(async theme=>{document.documentElement.dataset.theme=theme;await document.fonts.ready;},theme);await settle(page);
    const capture=async suffix=>{
      await settle(page);
      const audit=await page.evaluate(()=>({
        width:document.documentElement.scrollWidth,height:document.documentElement.scrollHeight,
        fonts:document.fonts.check('13px Geist')&&document.fonts.check('12px "Geist Mono"'),
        minimumText:Math.min(...[...document.querySelectorAll('body *')].filter(el=>el.getClientRects().length&&[...el.childNodes].some(n=>n.nodeType===3&&n.textContent.trim())).map(el=>parseFloat(getComputedStyle(el).fontSize))),
        floatingFit:[...document.querySelectorAll('.floating:not([hidden]),dialog[open],.dropdown-menu')].every(el=>{const r=el.getBoundingClientRect();return r.left>=0&&r.right<=1600&&r.top>=0&&r.bottom<=1000;}),
        base:getComputedStyle(document.body).backgroundColor,
        blur:getComputedStyle(document.querySelector('.sidebar')).backdropFilter,
      }));
      assert.equal(audit.width,1600);assert.equal(audit.height,1000);assert.ok(audit.fonts&&audit.minimumText>=12,JSON.stringify(audit));assert.ok(audit.floatingFit,`Floating layer clipped (${suffix})`);
      assert.equal(audit.base,theme==='dark'?'rgb(0, 0, 0)':'rgb(255, 255, 255)');assert.ok(audit.blur.includes('blur'));
      const path=`${output}/concept-a-${theme}${suffix?'-'+suffix:''}.png`;await page.screenshot({path,animations:'disabled'});assert.ok((await stat(path)).size>100000);files.push(path);console.log(`Rendered concept-a-${theme}${suffix?'-'+suffix:''}.png · ${await count(page)} voxels`);
    };
    assert.equal(await count(page),2079);assert.equal(new Set(await hashes(page)).size,3);
    assert.equal(await page.locator('[data-pin]:visible').count(),3);
    assert.ok(await page.locator('.sidebar').evaluate(el=>el.scrollHeight<=el.clientHeight+1),'Default sidebar overflow');
    assert.equal(await page.locator('details[open]').count(),0);
    await capture('');
    await menu(page,'Palette');assert.equal(await page.getByRole('menu',{name:'Palette',exact:true}).isVisible(),true);await capture('menu');
    await menuItem(page,'Manage palettes…').click();assert.ok(await page.locator('.palette-dialog').evaluate(el=>el.open));await capture('palettes');
    await page.keyboard.press('Escape');
    await page.locator('[data-pin="1"]').click();assert.equal(await page.locator('#comments-panel').isVisible(),true);assert.equal(await page.locator('.mcp-note').textContent(),'MCP · comments.reply');await capture('comments');
    await page.locator('[data-tab="session"]').click();await page.locator('[data-action="share"]').click();await capture('share');
    await page.locator('[data-close="share"]').click();await page.locator('[data-action="history"]').click();
    await page.locator('[data-timeline]').fill('4');assert.ok(await count(page)<2079);await capture('history');
    await page.locator('[data-close="history"]').click();assert.equal(await count(page),2079,'History close restores live scene');

    // Native-style menu traversal, submenus, dismissal and context menus.
    await page.locator('.menubar [data-menu="File"]').focus();await page.keyboard.press('ArrowDown');
    assert.equal(await page.evaluate(()=>document.activeElement.textContent.trim()),'New project⌘N');
    await page.keyboard.press('End');await page.keyboard.press('ArrowRight');
    assert.equal(await page.getByRole('menu',{name:'Export',exact:true}).isVisible(),true);
    await page.keyboard.press('ArrowLeft');await page.keyboard.press('Escape');
    assert.equal(await page.evaluate(()=>document.activeElement.dataset.menu),'File');
    for(const [selector,expected] of [['.viewport>canvas','ViewportContext'],['[data-layer="Roof"]','Layers'],['.palette-section','Palette']]){
      await page.locator(selector).click({button:'right',position:{x:15,y:15}});
      assert.ok(await page.getByRole('menu',{name:expected,exact:true}).isVisible());await page.keyboard.press('Escape');
    }
    await page.locator('.tool-picker').click();assert.equal(await menuItem(page,'Scatter moss Agent').count(),1);await menuItem(page,'Scatter moss Agent').click();assert.equal(await page.locator('[data-current-tool]').textContent(),'Scatter moss');
    await page.keyboard.press('p');assert.equal(await page.locator('[data-current-tool]').textContent(),'Paint');
    await menu(page,'View');await menuItem(page,'Show grid').click();assert.equal(await page.locator('[data-toggle-grid]').getAttribute('aria-pressed'),'false');await menu(page,'View');await menuItem(page,'Show grid').click();
    await menu(page,'Tools');await menuItem(page,'Mirror').click();await menuItem(page,'Mirror Y').click();assert.deepEqual(await page.evaluate(()=>window.__aPreview.mirrors),['Y']);

    // Palette switching, keyboard selection, accurate export bytes and round trips.
    await page.locator('.palette-switch').click();await menuItem(page,'PICO-8').click();assert.equal(await page.locator('[data-active-swatches] button').count(),16);assert.equal(await page.locator('[data-swatch="1"]').getAttribute('data-color'),'#1d2b53');
    await page.locator('[data-swatch="0"]').focus();await page.keyboard.press('End');assert.equal(await page.locator('[data-swatch][aria-pressed="true"]').getAttribute('data-color'),'#ffccaa');
    await page.locator('.palette-switch').click();await menuItem(page,'Endesga 32').click();assert.equal(await page.locator('[data-active-swatches] button').count(),32);assert.equal(await page.locator('[data-swatch="31"]').getAttribute('data-color'),'#c28569');
    await menu(page,'Palette');await menuItem(page,'Manage palettes…').click();assert.ok(await page.locator('[data-hex-edit]').isDisabled());
    for(const format of ['hex','gpl']){
      const dl=page.waitForEvent('download');await page.locator(`[data-action="export-${format}"]`).click();const download=await dl;const text=await readFile(await download.path(),'utf8');
      assert.equal(text,await page.evaluate(format=>window.__aPreview.exportText(format),format));
      assert.ok(text.endsWith('\n'));if(format==='hex'){assert.equal(text.split('\n')[0],'BE4A2F');assert.equal(text.trim().split('\n').length,32);}else{assert.ok(text.startsWith('GIMP Palette\nName: Endesga 32\nColumns: 8\n#\n'));assert.ok(text.includes('190  74  47\t#BE4A2F'));}
      assert.deepEqual(await page.evaluate(({text,format})=>window.__aPreview.parsePalette(text,format).colors,{text,format}),await page.evaluate(()=>window.__aPreview.palettes.find(p=>p.id==='endesga32').colors));
    }
    await page.locator('[data-action="duplicate-palette"]').click();await page.locator('[data-palette-rename]').fill('Forest experiment');await page.locator('[data-palette-rename]').press('Tab');
    await page.locator('[data-hex-edit]').fill('#123456');await page.locator('[data-hex-edit]').press('Tab');
    await page.locator('[data-action="swatch-right"]').click();assert.equal(await page.locator('[data-edit-swatch="1"]').getAttribute('data-color'),'#123456');
    await page.locator('[data-action="add-swatch"]').click();assert.equal(await page.locator('[data-edit-swatch]').count(),33);await page.locator('[data-action="remove-swatch"]').click();assert.equal(await page.locator('[data-edit-swatch]').count(),32);
    await page.locator('[data-action="use-palette"]').click();assert.equal(await page.locator('[data-palette-name]').textContent(),'Forest experiment');
    await menu(page,'Palette');await menuItem(page,'Manage palettes…').click();await page.locator('[data-action="delete-palette"]').click();assert.equal(await page.locator('[data-palette-name]').textContent(),'Woodland');
    await page.locator('[data-action="create-palette"]').click();assert.equal(await page.locator('[data-edit-swatch]').count(),1);await page.locator('[data-action="delete-palette"]').click();
    await page.locator('[data-palette-file]').setInputFiles({name:'sample.gpl',mimeType:'text/plain',buffer:Buffer.from('GIMP Palette\nName: Imported study\nColumns: 2\n#\n  1   2   3\tInk\n255 128   0\tGold\n')});
    await page.waitForFunction(()=>document.querySelector('[data-editor-title]').textContent==='Imported study');assert.equal(await page.locator('[data-edit-swatch="0"]').getAttribute('data-color'),'#010203');await page.locator('[data-action="delete-palette"]').click();
    const palettesBefore=await page.evaluate(()=>window.__aPreview.palettes.length);
    await page.locator('[data-palette-file]').setInputFiles({name:'invalid.hex',mimeType:'text/plain',buffer:Buffer.from('not a color')});await page.waitForFunction(()=>document.querySelector('.toast').textContent.includes('six-digit'));assert.equal(await page.evaluate(()=>window.__aPreview.palettes.length),palettesBefore);
    await page.keyboard.press('Escape');

    // Progressive disclosure and immutable captures across edits/camera changes.
    const initialCaptures=await hashes(page);
    await page.locator('[data-step="18"] summary').click();assert.equal((await page.locator('[data-step-coordinates="18"]').textContent()).split('\n').length,6);
    await page.locator('[data-step-undo="18"]').click();assert.equal(await count(page),2073);assert.deepEqual(await hashes(page),initialCaptures);assert.equal(await page.locator('[data-render-stale]').isVisible(),true);
    await page.locator('[data-step-undo="18"]').click();assert.equal(await count(page),2079);
    await page.keyboard.press('Meta+z');await page.keyboard.press('Meta+z');assert.equal(await count(page),2049);await page.keyboard.press('Meta+Shift+z');await page.keyboard.press('Meta+Shift+z');assert.equal(await count(page),2079);
    await page.locator('[data-step="18"] summary').click();
    await page.locator('[data-layer="Canopy"]').hover();await page.locator('[data-layer-eye="Canopy"]').click();assert.deepEqual(await hashes(page),initialCaptures);assert.equal(await page.locator('[data-pin="1"]').isVisible(),false);await page.locator('[data-layer-eye="Canopy"]').click();
    await menu(page,'Agent');await menuItem(page,'Refresh captured views').click();const fresh=await hashes(page);assert.equal(await page.locator('[data-render-stale]').isVisible(),false);
    const pin=await page.locator('[data-pin="1"]').getAttribute('style');await page.locator('.viewport>canvas').focus();await page.keyboard.press('ArrowRight');assert.notEqual(await page.locator('[data-pin="1"]').getAttribute('style'),pin);assert.deepEqual(await hashes(page),fresh);assert.equal(await page.locator('[data-follow]').getAttribute('aria-pressed'),'false');
    await page.keyboard.press('Home');await page.locator('[data-follow]').click();assert.equal(await page.locator('[data-follow]').getAttribute('aria-pressed'),'true');
    await page.locator('[data-pause]').click();assert.equal(await page.locator('[data-session-state]').textContent(),'Paused');await page.locator('[data-pause]').click();

    // Comment reply, resolve/reopen, filters and share permission/clipboard fallback.
    await page.locator('[data-pin="1"]').click();await page.locator('.reply-form input').fill('The softer silhouette works.');await page.locator('.reply-form button').click();assert.ok((await page.locator('.pin-thread').textContent()).includes('The softer silhouette works.'));
    await page.locator('[data-resolve="1"]').click();assert.equal(await page.locator('[data-pin="1"]').count(),0);await page.locator('[data-comment-filter]').selectOption('resolved');await page.locator('[data-open-thread="1"]').click();await page.locator('[data-resolve="1"]').click();await page.locator('[data-comment-filter]').selectOption('open');assert.equal(await page.locator('[data-open-thread="1"]').count(),1);
    await page.locator('[data-tab="session"]').click();await page.locator('[data-action="share"]').click();await page.locator('[data-permission="remix"]').click();assert.ok((await page.locator('.link-field input').inputValue()).endsWith('access=remix'));
    await page.evaluate(()=>Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text=>{window.__copiedLink=text;}}}));await page.locator('[data-action="copy-link"]').click();assert.equal(await page.evaluate(()=>window.__copiedLink),await page.locator('.link-field input').inputValue());
    await page.evaluate(()=>navigator.clipboard.writeText=async()=>{throw Error('Unavailable');});await page.locator('[data-action="copy-link"]').click();assert.ok(await page.locator('.link-field input').evaluate(el=>el.selectionEnd-el.selectionStart===el.value.length));await page.keyboard.press('Escape');

    // Geometry-backed history, save/restore/compare and branch actions.
    await menu(page,'View');await menuItem(page,'History').click();const latest=await count(page);await page.locator('[data-timeline]').fill('4');const midway=await count(page);assert.ok(midway<latest&&midway>1000);
    await page.locator('[data-action="save-version"]').click();await page.locator('.version-form input').fill('Garden checkpoint');await page.locator('.version-form button').click();assert.equal(await page.locator('.version-item h3').first().textContent(),'Garden checkpoint');
    await page.locator('[data-action="history-live"]').click();assert.equal(await count(page),latest);
    await page.locator('.version-item').first().locator('[data-version-action="compare"]').click();assert.ok((await page.locator('.compare-badge').textContent()).includes(`+${latest-midway} / −0`));await page.locator('[data-action="clear-compare"]').click();
    await page.locator('.version-item').first().locator('[data-version-action="restore"]').click();assert.equal(await count(page),midway);await page.locator('[data-close="history"]').click();assert.equal(await count(page),midway,'Restore must persist after closing history');
    await page.locator('[data-action="history"]').click();await page.locator('[data-version-id="v2"][data-version-action="branch"]').click();assert.equal(await count(page),latest);assert.equal(await page.locator('[data-branch-name]').textContent(),'canopy-pass-branch');await page.locator('[data-close="history"]').click();
    await page.keyboard.press('Meta+k');await page.locator('.command-search input').fill('nonsense');assert.ok(await page.locator('.command-empty').isVisible());await page.locator('.command-search input').fill('');await page.keyboard.press('ArrowDown');assert.ok(await page.locator('.command-search input').evaluate(el=>el===document.activeElement));await page.keyboard.type('Paint');await page.keyboard.press('Enter');assert.equal(await page.locator('[data-current-tool]').textContent(),'Paint');
    assert.deepEqual(errors,[],'Concept A browser errors');
    console.log(`Verified A ${theme}: menus, palettes/files, comments/share, history geometry, captures and keyboard controls.`);
    await page.close();
  }
  return files;
}
