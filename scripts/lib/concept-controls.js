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
    $$('[data-current-tool-icon]').forEach(t => t.replaceChildren(el.querySelector('svg').cloneNode(true)));
    $$('.current-tool>kbd').forEach(t => t.textContent = el.dataset.shortcut.toUpperCase());
  }
  $$('[data-tool]').forEach(el => el.addEventListener('click', () => chooseTool(el)));
  $$('[data-select-picker]').forEach(el => el.addEventListener('click', () => chooseTool($('[data-tool="picker"]'))));
  $$('[data-mirror]').forEach(el => el.addEventListener('click', () => {el.classList.toggle('active');el.setAttribute('aria-pressed', String(el.classList.contains('active')));}));
  $$('[data-color]').forEach(el => el.addEventListener('click', () => {
    $$('[data-color]').forEach(s => {s.classList.toggle('selected', s === el);s.setAttribute('aria-pressed', String(s === el));s.tabIndex = s === el ? 0 : -1;});
    $$('.current-swatch,.foreground').forEach(s => s.style.background = el.dataset.color);
    $$('[data-color-label]').forEach(s => s.textContent = el.dataset.color.toUpperCase());
  }));
  $$('[data-color]').forEach(el => {
    el.tabIndex = el.classList.contains('selected') ? 0 : -1;
    el.addEventListener('keydown', e => {
      const swatches = $$('[data-color]');
      const columns = getComputedStyle(el.parentElement).gridTemplateColumns.split(' ').length;
      const moves = {ArrowRight: 1, ArrowLeft: -1, ArrowDown: columns, ArrowUp: -columns};
      if (!(e.key in moves) && e.key !== 'Home' && e.key !== 'End') return;
      e.preventDefault(); e.stopPropagation();
      const index = e.key === 'Home' ? 0 : e.key === 'End' ? swatches.length - 1 :
        (swatches.indexOf(el) + moves[e.key] + swatches.length) % swatches.length;
      swatches[index].click(); swatches[index].focus();
    });
  });
  $$('[data-layer-lock]').forEach(el => el.addEventListener('click', () => {
    const locked = el.getAttribute('aria-pressed') !== 'true'; el.setAttribute('aria-pressed', String(locked));
    el.querySelector('svg').innerHTML = '<rect x="6" y="10" width="12" height="10" rx="2"/>' +
      (locked ? '<path d="M8 10V7a4 4 0 0 1 8 0v3M12 14v2"/>' : '<path d="M8 10V7a4 4 0 0 1 7-2M12 14v2"/>');
    el.classList.toggle('is-locked', locked); notify(el.closest('.layer-row').dataset.layer + (locked ? ' locked' : ' unlocked'));
  }));
  $$('[data-select-layer]').forEach(el => el.addEventListener('click', () => {
    $$('.layer-row').forEach(row => row.classList.toggle('selected', row === el.closest('.layer-row')));
    $$('[data-select-layer]').forEach(t => t.setAttribute('aria-pressed', String(t === el)));
    window.__updateSelection();
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
    $('[data-session-state]').textContent = paused ? 'Paused' : 'Live';
    $('[data-agent-status]').classList.toggle('is-paused', paused);
  });
  const reverted = [];
  function toggleStep(id) {
    const undone = window.__toggleStep(id);
    const previous = reverted.indexOf(id);
    if (previous >= 0) reverted.splice(previous, 1);
    if (undone) reverted.push(id);
    const row = $('[data-step="' + id + '"]'); row.classList.toggle('is-undone', undone);
    const el = row.querySelector('[data-step-undo]');el.querySelector('span').textContent = undone ? 'Redo' : 'Undo';el.setAttribute('aria-label', (undone ? 'Redo' : 'Undo') + ' step ' + id);
    if ($('.history-dialog').open) showHistoryChange(id);
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
  if (form) {
    const instruction = form.querySelector('input'), copy = form.querySelector('[data-copy-instruction]');
    instruction.addEventListener('input', () => copy.disabled = !instruction.value.trim());
    form.addEventListener('submit', async e => {
      e.preventDefault();
      if (!instruction.value.trim()) return;
      try {
        await navigator.clipboard.writeText(instruction.value.trim());
        notify('Instruction copied');
      } catch {
        instruction.focus(); instruction.select();
        notify('Press ⌘C or Ctrl+C to copy the selected draft');
      }
    });
  }
  $('[data-refresh-renders]').addEventListener('click', () => notify('Captured 3 views · ' + $('[data-render-revision]').textContent));
  const historyDialog = $('.history-dialog');
  let historyFocus, inspectedStep = '18';
  function showHistoryChange(id) {
    inspectedStep = id;
    const change = window.__getChange(id);
    $$('[data-inspect-step]').forEach(el => {
      if (el.closest('.history-tabs')) el.setAttribute('aria-pressed', String(el.dataset.inspectStep === id));
    });
    $('[data-history-description]').textContent = `Step ${id} · ${change.tool} · ${change.layer} · ${change.state}. Change r${String(change.revision).padStart(3, '0')} · document r${String(change.document_revision).padStart(3, '0')}.`;
    $('[data-history-diff]').textContent = JSON.stringify(change, null, 2)
      .replace(/\[\n\s+(-?\d+),\n\s+(-?\d+),\n\s+(-?\d+)\n\s+\]/g, '[$1, $2, $3]');
    $('[data-history-revert] span').textContent = change.state === 'undone' ? 'Redo change' : 'Undo change';
  }
  function openHistory(id = '18') {
    if (!historyDialog.open) {historyFocus = document.activeElement; historyDialog.showModal();}
    showHistoryChange(id);
  }
  $$('[data-inspect-step]').forEach(el => el.addEventListener('click', () => openHistory(el.dataset.inspectStep)));
  $$('[data-history-open]').forEach(el => el.addEventListener('click', () => openHistory()));
  $('[data-close-history]').addEventListener('click', () => historyDialog.close());
  historyDialog.addEventListener('close', () => historyFocus?.focus());
  $('[data-history-revert]').addEventListener('click', () => toggleStep(inspectedStep));
  const dialog = $('.command-dialog'), search = dialog.querySelector('input');
  let previousFocus, activeCommand = 0;
  const visibleCommands = () => $$('[data-command]').filter(el => !el.hidden);
  function selectCommand(index) {
    const visible = visibleCommands();
    activeCommand = visible.length ? (index + visible.length) % visible.length : -1;
    $$('[data-command]').forEach(el => el.setAttribute('aria-selected', String(el === visible[activeCommand])));
    if (visible[activeCommand]) search.setAttribute('aria-activedescendant', visible[activeCommand].id);
    else search.removeAttribute('aria-activedescendant');
    $('.command-empty').hidden = visible.length > 0;
  }
  function openCommands() {previousFocus = document.activeElement;search.value = '';$$('[data-command]').forEach(el => el.hidden = false);selectCommand(0);dialog.showModal();search.focus();}
  function closeCommands() {dialog.close();previousFocus?.focus();}
  $('[data-commands]').addEventListener('click', openCommands);
  dialog.addEventListener('close', () => previousFocus?.focus());
  search.addEventListener('input', () => {
    $$('[data-command]').forEach(el => el.hidden = !el.textContent.toLowerCase().includes(search.value.toLowerCase()));
    selectCommand(0);
  });
  $$('[data-command]').forEach(el => el.addEventListener('click', () => {
    closeCommands();
    const action = el.dataset.command;
    if (action === 'frame') $('[data-reset-view]').click();
    else if (action === 'follow') follow.click();
    else if (action === 'theme') theme();
    else chooseTool($('[data-tool="' + action + '"]'));
  }));
  dialog.addEventListener('keydown', e => {
    const visible = visibleCommands();
    if (!visible.length) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault(); selectCommand(activeCommand + (e.key === 'ArrowDown' ? 1 : -1));
    } else if (e.key === 'Enter' && document.activeElement === search) {e.preventDefault();visible[activeCommand].click();}
  });
  document.addEventListener('keydown', e => {
    if (historyDialog.open) return;
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
