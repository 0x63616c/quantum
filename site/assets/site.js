// Progressive enhancement only: all navigation and documentation remain static.
for (const pre of document.querySelectorAll('.docs-content pre')) {
  const button = document.createElement('button');
  button.type = 'button'; button.className = 'copy-code'; button.textContent = 'Copy code';
  button.setAttribute('aria-label', 'Copy preceding code example');
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(pre.textContent);
      button.textContent = 'Copied';
      setTimeout(() => { button.textContent = 'Copy code'; }, 1800);
    } catch { button.textContent = 'Select code to copy'; }
  });
  pre.after(button);
}
