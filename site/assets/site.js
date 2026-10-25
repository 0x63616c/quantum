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
