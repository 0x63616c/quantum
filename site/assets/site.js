// Progressive enhancement only: all navigation and documentation remain static.
for (const link of document.querySelectorAll('[data-markdown]')) {
  const button = document.createElement('button');
  button.type = 'button';
  button.textContent = 'Copy page as Markdown';
  const status = link.parentElement.querySelector('[role="status"]');
  button.addEventListener('click', async () => {
    button.disabled = true;
    status.textContent = 'Copying…';
    try {
      if (!navigator.clipboard) throw new Error('Clipboard unavailable');
      const markdown = fetch(link.href).then(async response => {
        if (!response.ok) throw new Error('Markdown unavailable');
        const text = await response.text();
        if (!text.startsWith('# ')) throw new Error('Unexpected response');
        return text;
      });
      // A promised ClipboardItem keeps the write within the click's user
      // activation in browsers that require it, even while fetching the page.
      if (window.ClipboardItem && navigator.clipboard.write) {
        const content = markdown.then(text => new Blob([text], { type: 'text/plain' }));
        // A denied write can reject before it consumes the promised content.
        // Observe a later download failure even when the clipboard ignores it;
        // the original promise still rejects for writes that do consume it.
        void content.catch(() => {});
        await navigator.clipboard.write([new ClipboardItem({
          'text/plain': content
        })]);
      } else {
        await navigator.clipboard.writeText(await markdown);
      }
      status.textContent = 'Copied';
    } catch {
      status.textContent = 'Could not copy. Use View .md.';
    } finally {
      button.disabled = false;
    }
  });
  link.before(button);
}

for (const pre of document.querySelectorAll('.docs-content pre')) {
  const button = document.createElement('button');
  button.type = 'button'; button.className = 'copy-code'; button.textContent = 'Copy code';
  button.setAttribute('aria-label', 'Copy code example');
  button.setAttribute('aria-live', 'polite');
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(pre.textContent);
      button.textContent = 'Copied';
      setTimeout(() => { button.textContent = 'Copy code'; }, 1800);
    } catch { button.textContent = 'Select code to copy'; }
  });
  const wrapper = document.createElement('div');
  wrapper.className = 'code-example';
  const bar = document.createElement('div');
  bar.className = 'code-bar';
  const language = document.createElement('span');
  const type = pre.querySelector('code')?.className.replace('language-', '') || 'json';
  language.textContent = type === 'sh' ? 'Shell' : type.toUpperCase();
  bar.append(language, button);
  pre.before(wrapper);
  wrapper.append(bar, pre);
}

// Native disclosures keep both menus usable without JavaScript. Enhancement
// dismisses them on navigation, outside clicks and Escape without trapping focus.
const docsMenus = document.querySelectorAll('.docs-mobile-nav details');
for (const menu of docsMenus) {
  menu.addEventListener('click', event => {
    const anchor = event.target.closest('a');
    if (!anchor) return;
    menu.open = false;
    if (anchor.hash && anchor.pathname === location.pathname) {
      const target = document.getElementById(decodeURIComponent(anchor.hash.slice(1)));
      if (target) {
        target.tabIndex = -1;
        target.focus({ preventScroll: true });
      }
    }
  });
}
document.addEventListener('click', event => {
  for (const menu of docsMenus) {
    if (!menu.contains(event.target)) menu.open = false;
  }
});
document.addEventListener('keydown', event => {
  if (event.key !== 'Escape') return;
  for (const menu of docsMenus) {
    if (!menu.open) continue;
    menu.open = false;
    menu.querySelector('summary').focus();
  }
});
