// Resolve appearance before first paint; storage is optional, never required.
(() => {
  let saved;
  try { saved = localStorage.getItem('voxelite-theme'); } catch { /* private storage */ }
  const system = matchMedia('(prefers-color-scheme: light)');
  const apply = theme => {
    document.documentElement.dataset.theme = theme;
    document.querySelector('meta[name="theme-color"]')?.setAttribute('content', theme === 'light' ? '#ffffff' : '#000000');
    document.querySelectorAll('.theme-image').forEach(img => { img.src = img.dataset[theme]; });
    document.querySelectorAll('.theme-toggle').forEach(button => {
      const label = `Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`;
      button.setAttribute('aria-label', label); button.title = label;
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
