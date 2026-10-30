// Static cards work as file downloads without JavaScript. Everything else is enhancement.
(() => {
  const models = JSON.parse(document.querySelector('#gallery-data').textContent);
  const cards = [...document.querySelectorAll('.gallery-card')];
  const dialog = document.querySelector('.lightbox');
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  let visible = models, active = 0, opener = null;
  const theme = () => document.documentElement.dataset.theme || 'dark';
  const syncPictures = () => {
    document.querySelectorAll('[data-theme-source]').forEach(source => {
      source.media = source.dataset.themeSource === theme() ? 'all' : 'not all';
    });
    document.querySelectorAll('.remix-model').forEach(link => {
      const url = new URL(link.href); url.searchParams.set('theme', theme()); link.href = url;
    });
  };
  new MutationObserver(syncPictures).observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
  syncPictures();
  document.querySelector('.filters').hidden = false;
  const largePicture = model => {
    const picture = document.createElement('picture');
    for (const appearance of ['light', 'dark']) for (const format of [...model.formats].reverse()) {
      const source = document.createElement('source');
      source.type = `image/${format}`; source.dataset.themeSource = appearance;
      source.srcset = `/gallery/images/${model.id}-${appearance}-large.${format}`;
      picture.append(source);
    }
    const img = document.createElement('img'); img.src = `/gallery/images/${model.id}-${theme()}-large.webp`;
    img.alt = model.title; img.width = 1024; img.height = 1024;
    picture.append(img); return picture;
  };
  function show(index) {
    active = (index + visible.length) % visible.length;
    const model = visible[active];
    document.querySelector('.model-picture').replaceChildren(largePicture(model));
    document.querySelector('#model-title').textContent = model.title;
    document.querySelector('#model-description').textContent = model.description;
    document.querySelector('.model-category').textContent = model.category;
    document.querySelector('.voxel-count').textContent = model.voxel_count.toLocaleString('en-US');
    document.querySelector('.model-dims').textContent = model.dims.join(' × ');
    document.querySelector('.lightbox-position').textContent = `${String(active + 1).padStart(2, '0')} / ${visible.length}`;
    document.querySelector('.download-model').href = model.source;
    document.querySelector('.remix-model').href = `/app/?model=${encodeURIComponent(model.source)}&theme=${theme()}`;
    document.querySelector('.model-palette').replaceChildren(...model.palette.map(color => {
      const swatch = document.createElement('li'); swatch.style.setProperty('--swatch', color.hex);
      swatch.title = `${color.name} · ${color.hex}`; swatch.setAttribute('aria-label', swatch.title); return swatch;
    }));
    syncPictures();
  }
  for (const card of cards) {
    card.setAttribute('aria-haspopup', 'dialog');
    card.setAttribute('aria-label', `View ${models.find(model => model.id === card.dataset.id).title}`);
    card.addEventListener('click', event => {
      if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      event.preventDefault(); opener = card;
      show(visible.findIndex(model => model.id === card.dataset.id)); dialog.showModal();
      document.querySelector('.close-lightbox').focus();
    });
    // A second real Voxelite camera appears only on hover-capable devices.
    const turn = enabled => {
      if (reduced.matches || !matchMedia('(hover: hover)').matches) return;
      card.querySelectorAll('source').forEach(source => { source.srcset = source.srcset.replace(enabled ? '-tile.' : '-turn.', enabled ? '-turn.' : '-tile.'); });
    };
    card.addEventListener('pointerenter', () => turn(true));
    card.addEventListener('pointerleave', () => turn(false));
    card.addEventListener('blur', () => turn(false));
  }
  for (const chip of document.querySelectorAll('.filter-chip')) chip.addEventListener('click', () => {
    document.querySelectorAll('.filter-chip').forEach(button => button.setAttribute('aria-pressed', String(button === chip)));
    const category = chip.dataset.filter;
    visible = models.filter(model => category === 'All' || model.category === category);
    for (const card of cards) card.hidden = category !== 'All' && card.dataset.category !== category;
    document.querySelector('.model-count').textContent = `${visible.length} models`;
  });
  document.querySelector('.close-lightbox').addEventListener('click', () => dialog.close());
  document.querySelector('.previous-model').addEventListener('click', () => show(active - 1));
  document.querySelector('.next-model').addEventListener('click', () => show(active + 1));
  dialog.addEventListener('click', event => { if (event.target === dialog) { const box = dialog.getBoundingClientRect(); if (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom) dialog.close(); } });
  dialog.addEventListener('close', () => opener?.focus({ preventScroll: true }));
  dialog.addEventListener('keydown', event => {
    if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') { event.preventDefault(); show(active + (event.key === 'ArrowRight' ? 1 : -1)); }
    if (event.key === 'Tab') {
      const focusable = [...dialog.querySelectorAll('a[href], button:not([disabled])')];
      const first = focusable[0], last = focusable.at(-1);
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    }
  });
})();
