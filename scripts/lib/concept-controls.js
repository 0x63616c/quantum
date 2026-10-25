/* Local prototype state only. The session is a deterministic demonstration fixture. */
(() => {
  const $ = selector => document.querySelector(selector);
  const $$ = selector => [...document.querySelectorAll(selector)];
  let toastTimer;
  function notify(message) {
    const toast = $('.toast'); toast.textContent = message; toast.hidden = false;
    clearTimeout(toastTimer); toastTimer = setTimeout(() => toast.hidden = true, 2400);
  }
  const theme = () => document.documentElement.dataset.theme = document.documentElement.dataset.theme === 'light' ? 'dark' : 'light';
  $('[data-theme-toggle]').addEventListener('click', theme);
  function chooseTool(el) {
    if (!el) return;
    $$('[data-tool]').forEach(t => { const active = t === el; t.classList.toggle('active', active); t.setAttribute('aria-pressed', String(active)); });
    $$('[data-current-tool]').forEach(t => t.textContent = el.getAttribute('aria-label').split(' (')[0]);
    $$('.current-tool>kbd').forEach(t => t.textContent = el.dataset.shortcut.toUpperCase());
  }
  $$('[data-tool]').forEach(el => el.addEventListener('click', () => chooseTool(el)));
  $$('[data-select-picker]').forEach(el => el.addEventListener('click', () => chooseTool($('[data-tool="picker"]'))));
  $$('[data-mirror]').forEach(el => el.addEventListener('click', () => {el.classList.toggle('active');el.setAttribute('aria-pressed', String(el.classList.contains('active')));}));
  $$('[data-color]').forEach(el => el.addEventListener('click', () => {
    $$('[data-color]').forEach(s => {s.classList.toggle('selected', s === el);s.setAttribute('aria-pressed', String(s === el));});
    $$('.current-swatch,.foreground').forEach(s => s.style.background = el.dataset.color);
    $$('[data-color-label]').forEach(s => s.textContent = el.dataset.color.toUpperCase());
  }));
  $$('[data-layer-lock]').forEach(el => el.addEventListener('click', () => {
    const locked = el.getAttribute('aria-pressed') !== 'true'; el.setAttribute('aria-pressed', String(locked));
    el.classList.toggle('is-locked', locked); notify(el.closest('.layer-row').dataset.layer + (locked ? ' locked' : ' unlocked'));
  }));
  $$('[data-select-layer]').forEach(el => el.addEventListener('click', () => {
    $$('.layer-row').forEach(row => row.classList.toggle('selected', row === el.closest('.layer-row')));
    $$('[data-select-layer]').forEach(t => t.setAttribute('aria-pressed', String(t === el)));
  }));
  $$('[data-feedback]').forEach(el => el.addEventListener('click', () => notify(el.dataset.feedback)));
  for (const [selector, message] of [['[data-add-layer]','Add layer · concept preview'],['[data-delete-layer]','Delete layer · concept preview'],['[data-import]','Import palette · .hex / .gpl / .png'],['[data-save]','Save woodland-cottage.voxl · concept preview']]) {
    $$(selector).forEach(el => el.addEventListener('click', () => notify(message)));
  }
  $('[data-png]').addEventListener('click', () => {
    const canvas = $('.viewport > canvas');
    const link = document.createElement('a');link.download = 'woodland-cottage.png';link.href = canvas.toDataURL('image/png');link.click();
    notify('PNG exported');
  });
  const follow = $('[data-follow]');
  function setFollow(enabled) { follow.setAttribute('aria-pressed', String(enabled)); }
  follow.addEventListener('click', () => {const enabled = follow.getAttribute('aria-pressed') !== 'true';setFollow(enabled);if (enabled) window.__setViewAngle('front');});
  document.addEventListener('manual-camera', () => setFollow(false));
  $$('[data-view-angle]').forEach(el => el.addEventListener('click', () => {setFollow(false);window.__setViewAngle(el.dataset.viewAngle);}));
  $('[data-pause]').addEventListener('click', e => {
    const el = e.currentTarget, paused = el.getAttribute('aria-pressed') !== 'true';
    el.setAttribute('aria-pressed', String(paused));el.querySelector('svg').innerHTML = paused ? '<path d="m8 4 12 8-12 8V4Z"/>' : '<path d="M8 5v14M16 5v14"/>';el.querySelector('span').textContent = paused ? 'Resume' : 'Pause';
    $('[data-running-label]').textContent = paused ? 'Paused' : 'Running';
    $('[data-running-title]').textContent = paused ? 'Render check paused' : 'Checking silhouettes';
  });
  const reverted = [];
  function toggleStep(id) {
    const undone = window.__toggleStep(id);
    const previous = reverted.indexOf(id);
    if (previous >= 0) reverted.splice(previous, 1);
    if (undone) reverted.push(id);
    const row = $('[data-step="' + id + '"]'); row.classList.toggle('is-undone', undone);
    const el = row.querySelector('[data-step-undo]');el.querySelector('span').textContent = undone ? 'Redo' : 'Undo';el.setAttribute('aria-label', (undone ? 'Redo' : 'Undo') + ' step ' + id);
    notify((undone ? 'Undid' : 'Restored') + ' step ' + id + ' · local preview');
  }
  $$('[data-step-undo]').forEach(el => el.addEventListener('click', () => toggleStep(el.dataset.stepUndo)));
  function undo() {
    const row = $$('.session-step[data-step]:not(.is-undone)').at(-1);
    if (row) toggleStep(row.dataset.step); else notify('Nothing to undo');
  }
  function redo() {
    const id = reverted.at(-1); if (id) toggleStep(id); else notify('Nothing to redo');
  }
  $$('[data-history-undo]').forEach(el => el.addEventListener('click', undo));
  $$('[data-history-redo]').forEach(el => el.addEventListener('click', redo));
  const form = $('.session-composer');
  if (form) form.addEventListener('submit', e => {e.preventDefault();notify('Instruction preview · connect an MCP host in the app');});
  const dialog = $('.command-dialog'), search = dialog.querySelector('input');
  let previousFocus;
  function openCommands() {previousFocus = document.activeElement;search.value = '';$$('[data-command]').forEach(el => el.hidden = false);dialog.showModal();search.focus();}
  function closeCommands() {dialog.close();previousFocus?.focus();}
  $('[data-commands]').addEventListener('click', openCommands);
  dialog.addEventListener('close', () => previousFocus?.focus());
  search.addEventListener('input', () => $$('[data-command]').forEach(el => el.hidden = !el.textContent.toLowerCase().includes(search.value.toLowerCase())));
  $$('[data-command]').forEach(el => el.addEventListener('click', () => {
    closeCommands();
    const action = el.dataset.command;
    if (action === 'frame') $('[data-reset-view]').click();
    else if (action === 'follow') follow.click();
    else if (action === 'theme') theme();
    else chooseTool($('[data-tool="' + action + '"]'));
  }));
  dialog.addEventListener('keydown', e => {
    const visible = $$('[data-command]').filter(el => !el.hidden);
    if (!visible.length) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault(); const direction = e.key === 'ArrowDown' ? 1 : -1;
      const index = visible.indexOf(document.activeElement);
      visible[(index + direction + visible.length) % visible.length].focus();
    } else if (e.key === 'Enter' && document.activeElement === search) {e.preventDefault();visible[0].click();}
  });
  document.addEventListener('keydown', e => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {e.preventDefault();dialog.open ? closeCommands() : openCommands();return;}
    if (dialog.open || /INPUT|SELECT|TEXTAREA/.test(e.target.tagName)) return;
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'z') {e.preventDefault();e.shiftKey ? redo() : undo();return;}
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.shiftKey && e.key.toLowerCase() === 'f') {e.preventDefault();follow.click();return;}
    const tool = $('[data-shortcut="' + e.key.toLowerCase() + '"]');
    if (tool) {e.preventDefault();chooseTool(tool);}
    if (e.key === 'Home') {e.preventDefault();$('[data-reset-view]').click();}
  });
})();
