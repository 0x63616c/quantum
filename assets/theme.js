// Resolve appearance before first paint; storage is optional, never required.
(() => {
  let saved;
  try { saved = localStorage.getItem('voxelite-theme'); } catch { /* private storage */ }
  const system = matchMedia('(prefers-color-scheme: light)');
  const apply = theme => {
    document.documentElement.dataset.theme = theme;
    document.querySelector('meta[name="theme-color"]')?.setAttribute('content', theme === 'light' ? '#ffffff' : '#000000');
    document.querySelectorAll('.theme-image').forEach(img => { img.src = img.dataset[theme]; });
    document.querySelectorAll('.workspace-detail').forEach(link => { link.href = `/assets/session-${theme}.png`; });
    document.querySelectorAll('.theme-toggle').forEach(button => {
      const label = `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`;
      button.setAttribute('aria-label', label); button.title = label;
      button.querySelector('svg').innerHTML = theme === 'light'
        ? '<path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5Z"/>'
        : '<circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.4 1.4m11.2 11.2L19 19M5 19l1.4-1.4M17.6 6.4 19 5"/>';
    });
  };
  apply(saved === 'light' || saved === 'dark' ? saved : system.matches ? 'light' : 'dark');
  system.addEventListener('change', event => { if (!saved) apply(event.matches ? 'light' : 'dark'); });
  document.addEventListener('DOMContentLoaded', () => {
    apply(document.documentElement.dataset.theme);
    document.querySelectorAll('.theme-toggle').forEach(button => button.addEventListener('click', () => {
      saved = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
      try { localStorage.setItem('voxelite-theme', saved); } catch { /* appearance still works */ }
      apply(saved);
    }));
  });
})();
