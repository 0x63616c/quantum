// Progressive enhancement only: all navigation and documentation remain static.
for (const block of document.querySelectorAll('[data-install]')) {
  const panels = [...block.querySelectorAll('[data-install-panel]')];
  const tabs = document.createElement('div');
  tabs.className = 'install-tabs';
  tabs.setAttribute('role', 'tablist');
  tabs.setAttribute('aria-label', 'MCP host');
  const buttons = panels.map(panel => {
    const button = document.createElement('button');
    button.type = 'button';
    button.id = `${panel.id}-tab`;
    button.textContent = panel.dataset.installPanel;
    button.setAttribute('role', 'tab');
    button.setAttribute('aria-controls', panel.id);
    panel.setAttribute('role', 'tabpanel');
    panel.setAttribute('aria-labelledby', button.id);
    panel.tabIndex = 0;
    button.addEventListener('click', () => select(panel));
    tabs.append(button);
    return button;
  });
  function select(selected) {
    panels.forEach((panel, index) => {
      const active = panel === selected;
      panel.hidden = !active;
      buttons[index].setAttribute('aria-selected', String(active));
      buttons[index].tabIndex = active ? 0 : -1;
    });
  }
  tabs.addEventListener('keydown', event => {
    const index = buttons.indexOf(event.target);
    if (index < 0) return;
    let next;
    if (event.key === 'ArrowRight') next = (index + 1) % buttons.length;
    else if (event.key === 'ArrowLeft') next = (index + buttons.length - 1) % buttons.length;
    else if (event.key === 'Home') next = 0;
    else if (event.key === 'End') next = buttons.length - 1;
    else return;
    event.preventDefault();
    select(panels[next]);
    buttons[next].focus();
  });
  const selectFromHash = () => {
    const panel = panels.find(panel => `#${panel.id}` === location.hash);
    if (panel) select(panel);
  };
  block.prepend(tabs);
  block.classList.add('install-enhanced');
  select(panels[0]);
  selectFromHash();
  window.addEventListener('hashchange', selectFromHash);
}

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
