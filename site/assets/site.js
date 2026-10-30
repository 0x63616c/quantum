// Progressive enhancement only: all navigation and documentation remain static.
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
