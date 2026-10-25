/* Offline design fixture. No network session, sharing service or plugin host is implied. */
(() => {
  const $ = s => document.querySelector(s), $$ = s => [...document.querySelectorAll(s)];
  const {icons, woodland} = window.__conceptA;
  const esc = value => String(value).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  let toastTimer, menuTrigger, menuChain=[], modalFocus, follow=true, paused=false, tool='place', selectedLayer='Canopy';
  const mirrors = new Set(), lockedLayers = new Set(['Garden']);
  const notify = message => { $('.toast').textContent=message;$('.toast').hidden=false;clearTimeout(toastTimer);toastTimer=setTimeout(()=>$('.toast').hidden=true,2600); };
  const download=(name,text,type='text/plain')=>{const url=URL.createObjectURL(new Blob([text],{type}));const a=document.createElement('a');a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
  const presets = [
    {id:'pico8',name:'PICO-8',colors:'000000 1d2b53 7e2553 008751 ab5236 5f574f c2c3c7 fff1e8 ff004d ffa300 ffec27 00e436 29adff 83769c ff77a8 ffccaa'},
    {id:'endesga32',name:'Endesga 32',colors:'be4a2f d77643 ead4aa e4a672 b86f50 733e39 3e2731 a22633 e43b44 f77622 feae34 fee761 63c74d 3e8948 265c42 193c3e 124e89 0099db 2ce8f5 ffffff c0cbdc 8b9bb4 5a6988 3a4466 262b44 181425 ff0044 68386c b55088 f6757a e8b796 c28569'},
    {id:'sweetie16',name:'Sweetie 16',colors:'1a1c2c 5d275d b13e53 ef7d57 ffcd75 a7f070 38b764 257179 29366f 3b5dc9 41a6f6 73eff7 f4f4f4 94b0c2 566c86 333c57'},
    {id:'grayscale',name:'Grayscale',colors:Array.from({length:16},(_,i)=>(i*17).toString(16).padStart(2,'0').repeat(3)).join(' ')}
  ].map(p=>({...p,preset:true,colors:p.colors.split(' ').map(c=>'#'+c)}));
  let custom=[{id:'woodland',name:'Woodland',colors:woodland}], activePalette='woodland', editingPalette='woodland', swatchIndex=10, editorIndex=10;
  try { const saved=JSON.parse(localStorage.getItem('voxelite-palettes-v1')); if(Array.isArray(saved?.custom) && saved.custom.every(p=>typeof p.id==='string' && typeof p.name==='string' && p.name.trim() && p.colors.length && p.colors.length<=256 && p.colors.every(c=>/^#[\da-f]{6}$/i.test(c)))){custom=saved.custom;activePalette=saved.activePalette;}}catch{}
  const palettes=()=>[...custom,...presets], palette=id=>palettes().find(p=>p.id===id)||custom[0]||presets[0], active=()=>palette(activePalette), editing=()=>palette(editingPalette);
  if(!palettes().some(p=>p.id===activePalette))activePalette=palettes()[0].id;
  editingPalette=activePalette;
  const persist=()=>{try{localStorage.setItem('voxelite-palettes-v1',JSON.stringify({custom,activePalette}));}catch{notify('Palette storage unavailable; export to keep your colors.');}};
  const uid=prefix=>`${prefix}-${Date.now()}-${Math.random().toString(36).slice(2,7)}`;
  const swatchButton=(color,index,selected,editor=false)=>`<button class="swatch" style="--swatch:${color}" ${editor?'data-edit-swatch':'data-swatch'}="${index}" data-color="${color}" aria-label="${editor?'Edit':'Select'} color ${color.toUpperCase()}" aria-pressed="${index===selected}" tabindex="${index===selected?0:-1}"></button>`;
  function renderPalette(){const p=active();swatchIndex=Math.min(swatchIndex,p.colors.length-1);$('[data-palette-name]').textContent=p.name;$('[data-palette-count]').textContent=`${p.colors.length} colors`;$('[data-color-label]').textContent=p.colors[swatchIndex].toUpperCase();$('[data-active-swatches]').innerHTML=p.colors.map((c,i)=>swatchButton(c,i,swatchIndex)).join('');}
  function renderManager(){
    const p=editing();editorIndex=Math.max(0,Math.min(editorIndex,p.colors.length-1));
    for(const [selector,items] of [['[data-custom-palettes]',custom],['[data-preset-palettes]',presets]]) $(selector).innerHTML=items.map(q=>`<button data-edit-palette="${esc(q.id)}" aria-current="${q.id===p.id}">${esc(q.name)}<small>${q.colors.length}</small></button>`).join('');
    $('[data-editor-title]').textContent=p.name;$('[data-editor-subtitle]').textContent=`${p.colors.length} colors · ${p.preset?'Preset':'Custom palette'}`;
    $('[data-palette-rename]').value=p.name;$('[data-palette-rename]').disabled=!!p.preset;
    $('.manager-swatches').innerHTML=p.colors.map((c,i)=>swatchButton(c,i,editorIndex,true)).join('');
    $('.color-preview').style.background=p.colors[editorIndex];$('[data-hex-edit]').value=p.colors[editorIndex].toUpperCase();$('[data-hex-edit]').disabled=!!p.preset;
    for(const action of ['swatch-left','swatch-right','remove-swatch','add-swatch','delete-palette'])$(`[data-action="${action}"]`).disabled=!!p.preset || (action==='remove-swatch'&&p.colors.length===1) || (action==='add-swatch'&&p.colors.length>=256) || (action==='swatch-left'&&editorIndex===0) || (action==='swatch-right'&&editorIndex===p.colors.length-1);
    $('.palette-edit-note').textContent=p.preset?'Duplicate this preset to make it your own.':'Select a swatch to edit. Use the arrows to reorder.';
    $('[data-editor-status]').textContent=p.id===activePalette?'Active palette':'';
  }
  function openManager(){closeFloating();editingPalette=activePalette;editorIndex=swatchIndex;renderManager();openDialog($('.palette-dialog'));}
  function updatePalette(){persist();renderPalette();renderManager();}
  function createPalette(copy=false){const source=editing();const p={id:uid('palette'),name:copy?source.name+' copy':'Untitled palette',colors:copy?[...source.colors]:['#829a63']};custom.push(p);editingPalette=p.id;editorIndex=0;updatePalette();$('[data-palette-rename]').focus();$('[data-palette-rename]').select();}
  function exportText(format,p=editing()){if(format==='hex')return p.colors.map(c=>c.slice(1).toUpperCase()).join('\n')+'\n';return `GIMP Palette\nName: ${p.name.replace(/[\r\n]/g,' ')}\nColumns: 8\n#\n`+p.colors.map(c=>c.slice(1).match(/../g).map(n=>String(parseInt(n,16)).padStart(3,' ')).join(' ')+'\t'+c.toUpperCase()).join('\n')+'\n';}
  function exportPalette(format){const p=$('.palette-dialog').open?editing():active();const text=exportText(format,p);window.__lastPaletteExport={format,text};download(p.name.replace(/[^a-z0-9_-]/gi,'-').toLowerCase()+'.'+format,text);notify(`Exported ${p.name}.${format}`);}
  function parsePalette(text,extension){
    const lines=text.replace(/^\uFEFF/,'').split(/\r?\n/);let name='Imported palette', colors=[];
    if(extension==='gpl'){
      if(lines.shift()?.trim()!=='GIMP Palette')throw Error('Expected a GIMP Palette header.');
      for(const line of lines){const value=line.trim();if(!value||value.startsWith('#')||value.startsWith('Columns:'))continue;if(value.startsWith('Name:')){name=value.slice(5).trim()||name;continue;}const m=value.match(/^(\d+)\s+(\d+)\s+(\d+)(?:\s|$)/);if(!m || m.slice(1).some(c=>Number(c)>255))throw Error('Invalid RGB row in .gpl file.');colors.push('#'+m.slice(1).map(c=>Number(c).toString(16).padStart(2,'0')).join(''));}
    }else{for(const line of lines){const value=line.trim();if(!value||value.startsWith(';')||value.startsWith('//'))continue;if(!/^#?[a-f\d]{6}$/i.test(value))throw Error('Use one six-digit hex color per line.');colors.push('#'+value.replace(/^#/,'').toLowerCase());}}
    if(!colors.length||colors.length>256)throw Error('Palettes need 1–256 colors.');return {id:uid('palette'),name:name.slice(0,60),colors};
  }
  $('[data-palette-file]').addEventListener('change',async e=>{const file=e.target.files[0];if(!file)return;try{if(file.size>65536)throw Error('Palette file is too large.');const ext=file.name.split('.').pop().toLowerCase();if(!['hex','gpl'].includes(ext))throw Error('Choose a .hex or .gpl file.');const p=parsePalette(await file.text(),ext);if(ext==='hex')p.name=file.name.replace(/\.hex$/i,'').slice(0,60);custom.push(p);editingPalette=p.id;editorIndex=0;updatePalette();notify(`Imported ${p.colors.length} colors`);}catch(error){notify(error.message);}finally{e.target.value='';}});
  $('[data-palette-rename]').addEventListener('change',e=>{if(editing().preset)return;const name=e.target.value.trim();if(!name){notify('Give this palette a name.');renderManager();return;}editing().name=name;updatePalette();});
  $('[data-hex-edit]').addEventListener('change',e=>{if(editing().preset)return;const value=e.target.value.trim();if(!/^#?[a-f\d]{6}$/i.test(value)){notify('Enter a six-digit hex color.');e.target.value=editing().colors[editorIndex];return;}editing().colors[editorIndex]='#'+value.replace(/^#/,'').toLowerCase();updatePalette();});

  const tools=[['place','Place','B'],['erase','Erase','E'],['paint','Paint','P'],['fill','Fill','G'],['box','Box','U'],['picker','Picker','I']];
  function chooseTool(id){tool=id;const name=tools.find(t=>t[0]===id)?.[1]||'Scatter moss';$('[data-current-tool]').textContent=name;$('[data-tool-icon]').innerHTML=icons[id]||icons.agent;$('.tool-picker').setAttribute('data-selected-tool',id);if(id==='scatter')notify('Scatter moss selected · agent-made tool preview');}
  function setFollow(value){follow=value;$('[data-follow]').setAttribute('aria-pressed',String(value));if(value)window.__setViewAngle('front');}
  document.addEventListener('manual-camera',()=>{follow=false;$('[data-follow]').setAttribute('aria-pressed','false');});
  function pause(){paused=!paused;$('[data-pause]').setAttribute('aria-pressed',String(paused));$('[data-pause]').setAttribute('aria-label',paused?'Resume agent':'Pause agent');$('[data-session-state]').textContent=paused?'Paused':'Live';$('[data-running-title]').textContent=paused?'Session paused':'Checking silhouettes';$('[data-running-label]').textContent=paused?'Ready to resume':'render_views · 3 angles';$('[data-pause]').innerHTML=paused?'<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="m8 4 12 8-12 8Z"/></svg>':'<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor"><path d="M8 5v14M16 5v14"/></svg>';}
  const reverted=[];
  function syncSteps(){for(const id of ['17','18']){const c=window.__getChange(id),el=$(`[data-step="${id}"]`);el.classList.toggle('is-undone',c.state==='undone');el.querySelector('[data-step-undo] span').textContent=c.state==='undone'?'Redo step':'Undo step';$(`[data-step-coordinates="${id}"]`).textContent=c.added.map(p=>p.join(', ')).join('\n');}}
  function toggleStep(id){if(!$('.history-panel').hidden)closeHistory();const undone=window.__toggleStep(id);const i=reverted.indexOf(id);if(i>=0)reverted.splice(i,1);if(undone)reverted.push(id);syncSteps();}
  function undo(){const id=['18','17'].find(id=>window.__getChange(id).state==='applied');if(id)toggleStep(id);else notify('No earlier reversible session step.');}
  function redo(){const id=reverted.at(-1);if(id)toggleStep(id);else notify('Nothing to redo.');}

  // DOM menus share one command registry, including context menus and command search.
  const item=(label,action,shortcut='',iconName='')=>({label,action,shortcut,icon:iconName});
  const sub=(label,menu)=>({label,menu});
  const preview=label=>()=>notify(label+' · concept preview');
  const menuItems=name=>({
    File:[item('New project',preview('New project'),'⌘N'),item('Open…',preview('Open .voxl'),'⌘O'),null,item('Save',preview('Save .voxl'),'⌘S'),item('Save as…',preview('Save as .voxl'),'⇧⌘S'),null,sub('Export','Export')],
    Export:[item('PNG image',()=>{const a=document.createElement('a');a.download='woodland-cottage.png';a.href=$('.viewport>canvas').toDataURL();a.click();}),item('glTF binary (.glb)',preview('GLB export'))],
    Edit:[item('Undo',undo,'⌘Z','undo'),item('Redo',redo,'⇧⌘Z','redo'),null,item('Search commands','commands','⌘K','search')],
    View:[item('Frame model','frame','Home','focus'),sub('Zoom','Zoom'),item('Show grid','grid','','grid'),null,item('Comments','comments','','comment'),item(`${pinsVisible?'✓  ':''}Comment pins`,()=>{pinsVisible=!pinsVisible;window.__drawVoxelScene();}),item('History','history','','clock'),null,item('Toggle light / dark','theme','','sun')],
    Zoom:[75,100,125,150,175].map(n=>item(`${n}%`,()=>{const sel=$('[data-zoom-select]');sel.value=n;sel.dispatchEvent(new Event('change'));})),
    Tools:[...tools.map(([id,label,key])=>item(label,()=>chooseTool(id),key,id)),null,sub('Mirror','Mirror'),null,{label:'Custom',heading:true}, {...item('Scatter moss',()=>chooseTool('scatter'),'','agent'),badge:'Agent'}],
    ToolPicker:[...tools.map(([id,label,key])=>item(label,()=>chooseTool(id),key,id)),null,{label:'Custom',heading:true},{...item('Scatter moss',()=>chooseTool('scatter'),'','agent'),badge:'Agent'}],
    Mirror:['X','Y','Z'].map(axis=>item(`${mirrors.has(axis)?'✓  ':''}Mirror ${axis}`,()=>{mirrors.has(axis)?mirrors.delete(axis):mirrors.add(axis);notify(`Mirror ${axis} ${mirrors.has(axis)?'on':'off'} · preview`);})),
    Layers:[item('New layer',preview('New layer')),item('Rename layer',preview('Rename '+selectedLayer)),item('Duplicate layer',preview('Duplicate '+selectedLayer)),null,item('Toggle visibility',()=> $(`[data-layer-eye="${selectedLayer}"]`).click(),'','eye'),item(`${lockedLayers.has(selectedLayer)?'Unlock':'Lock'} layer`,()=>{lockedLayers.has(selectedLayer)?lockedLayers.delete(selectedLayer):lockedLayers.add(selectedLayer);notify(`${selectedLayer} ${lockedLayers.has(selectedLayer)?'locked':'unlocked'}`);},'','lock'),null,item('Delete layer',preview('Delete '+selectedLayer),'','trash')],
    Palette:[item('Manage palettes…','palettes','','swatches'),item('New palette…',()=>{openManager();createPalette();}),sub('Switch palette','PaletteSwitcher'),null,item('Import .hex / .gpl…','import-palette','','import'),sub('Export palette','PaletteExport')],
    PaletteExport:[item('Hex colors (.hex)','export-hex'),item('GIMP palette (.gpl)','export-gpl')],
    PaletteSwitcher:[{label:'Your palettes',heading:true},...custom.map(p=>({...item(p.name,()=>selectPalette(p.id)),ramp:p.colors})),null,{label:'Preset library',heading:true},...presets.map(p=>({...item(p.name,()=>selectPalette(p.id)),ramp:p.colors})),null,item('Manage palettes…','palettes')],
    Agent:[item(paused?'Resume session':'Pause session','pause'),item(`${follow?'✓  ':''}Follow agent`,'follow','⇧F','camera'),null,item('Refresh captured views','refresh','','refresh'),item('Inspect latest step',()=>{$('[data-tab="session"]').click();$('[data-step="18"]').open=true;}),item('History','history','','clock')],
    Captures:[item('Refresh captures','refresh','','refresh'),...['front','right','back'].map(v=>item(`View ${v}`,()=>{setFollow(false);window.__setViewAngle(v);} ))],
    Extensions:[{label:'Installed',heading:true},item('Scatter moss',()=>chooseTool('scatter'),'','agent'),null,item('Manage extensions…',preview('Extensions · local tool plugins')),item('Create a tool via MCP…',preview('tools.register · runtime tool authoring'))],
    Help:[item('Search commands','commands','⌘K','search'),item('Keyboard shortcuts',()=>notify('B Place · E Erase · P Paint · G Fill · U Box · I Picker · ⇧F Follow · ⌘K Commands')),item('About this preview',()=>notify('Offline design fixture · no connected MCP or sharing service'))],
    ViewportContext:[item('Add comment here',()=>addCommentAtContext(),'','comment'),item('Frame model','frame','Home'),sub('Camera','Captures'),sub('Zoom','Zoom'),null,item('Show grid','grid'),item('History','history')]
  })[name]||[];
  function selectPalette(id){activePalette=id;swatchIndex=0;persist();renderPalette();notify(`Using ${active().name}`);}
  function closeMenus(restore=false){$('.menu-root').replaceChildren();menuChain=[];$$('[data-menu][aria-expanded=true]').forEach(el=>el.setAttribute('aria-expanded','false'));if(restore)menuTrigger?.focus();}
  function showMenu(name,trigger,point=null,level=0){
    if(level===0){closeMenus();menuTrigger=trigger;trigger?.setAttribute('aria-expanded','true');}else{menuChain.splice(level).forEach(m=>m.element.remove());}
    const entries=menuItems(name),element=document.createElement('div');element.className='dropdown-menu';element.role='menu';element.setAttribute('aria-label',name);element.dataset.menuName=name;
    element.innerHTML=entries.map((it,i)=>!it?'<div role="separator" class="separator"></div>':it.heading?`<div class="menu-label">${esc(it.label)}</div>`:`<button role="menuitem" tabindex="-1" data-menu-index="${i}" ${it.menu?'aria-haspopup="menu" aria-expanded="false"':''}>${it.icon?icons[it.icon]:''}<span>${esc(it.label)}</span>${it.shortcut?`<kbd>${it.shortcut}</kbd>`:''}${it.badge?`<span class="menu-agent-badge">${it.badge}</span>`:''}${it.ramp?`<span class="mini-ramp">${it.ramp.slice(0,8).map(c=>`<i style="background:${c}"></i>`).join('')}</span>`:''}${it.menu?`<span class="menu-tail">${icons.chevron}</span>`:''}</button>`).join('');
    $('.menu-root').append(element);const r=trigger?.getBoundingClientRect();let x=point?.x??(level?r.right+4:r.left),y=point?.y??(level?r.top:r.bottom+5);const bounds=element.getBoundingClientRect();if(x+bounds.width>innerWidth-8)x=level?r.left-bounds.width-4:innerWidth-bounds.width-8;y=Math.max(8,Math.min(y,innerHeight-bounds.height-8));element.style.left=x+'px';element.style.top=y+'px';menuChain.push({name,element,trigger});
    element.addEventListener('click',e=>{const el=e.target.closest('[data-menu-index]');if(!el)return;const it=entries[el.dataset.menuIndex];if(it.menu){el.setAttribute('aria-expanded','true');showMenu(it.menu,el,null,level+1);}else{closeMenus();run(it.action);}});
    element.addEventListener('pointerover',e=>{const el=e.target.closest('[data-menu-index]');if(!el)return;const it=entries[el.dataset.menuIndex];if(it.menu){if(menuChain[level+1]?.name!==it.menu){el.setAttribute('aria-expanded','true');showMenu(it.menu,el,null,level+1,false);}}else{menuChain.splice(level+1).forEach(m=>m.element.remove());}});
    element.addEventListener('keydown',e=>{const buttons=[...element.querySelectorAll('button')],index=buttons.indexOf(document.activeElement);if(['ArrowDown','ArrowUp','Home','End'].includes(e.key)){e.preventDefault();const next=e.key==='Home'?0:e.key==='End'?buttons.length-1:(index+(e.key==='ArrowDown'?1:-1)+buttons.length)%buttons.length;buttons[next].focus();}if(e.key==='ArrowRight'){e.preventDefault();const it=entries[document.activeElement.dataset.menuIndex];if(it?.menu){document.activeElement.click();menuChain.at(-1).element.querySelector('button')?.focus();}else if(level===0)cycleMenubar(1);}if(e.key==='ArrowLeft'){e.preventDefault();if(level){menuChain.splice(level).forEach(m=>m.element.remove());trigger.focus();trigger.setAttribute('aria-expanded','false');}else cycleMenubar(-1);}if(e.key==='Escape'){e.preventDefault();e.stopPropagation();closeMenus(true);}if(e.key==='Tab')closeMenus();});
  }
  function cycleMenubar(direction){const buttons=$$('.menubar>[data-menu]'),index=buttons.indexOf(menuTrigger);if(index>=0){const el=buttons[(index+direction+buttons.length)%buttons.length];showMenu(el.dataset.menu,el);menuChain[0].element.querySelector('button')?.focus();}}

  // Comment fixtures use model-space anchors, projected on every camera draw.
  let comments=[
    {id:1,anchor:[8,14,-1],layer:'Canopy',author:'AL',text:'Could the canopy feel a little less symmetrical?',resolved:false,replies:[{author:'Agent',text:'I softened the outer branches and added six lighter leaves. Check the right-side capture.',mcp:true}]},
    {id:2,anchor:[-3,16,3],layer:'Roof',author:'JD',text:'Keep this warm roof color. It balances the greens nicely.',resolved:false,replies:[]},
    {id:3,anchor:[-2,3,8],layer:'Garden',author:'AL',text:'A few mossy stones along this path?',resolved:false,replies:[]}
  ],activeThread=null,pinsVisible=true,contextPoint=null;
  function setTab(tab){$$('[data-tab]').forEach(el=>el.setAttribute('aria-selected',String(el.dataset.tab===tab)));$$('[data-tab]').forEach(el=>el.tabIndex=el.dataset.tab===tab?0:-1);$('#session-panel').hidden=tab!=='session';$('#comments-panel').hidden=tab!=='comments';$('[data-action="comments"]').setAttribute('aria-pressed',String(tab==='comments'));if(tab==='session'){activeThread=null;$('.pin-thread').hidden=true;}renderComments();window.__drawVoxelScene();}
  const messageHTML=(author,text,mcp=false)=>`<div class="thread-message"><div class="thread-meta"><span class="avatar ${mcp?'agent-avatar':''}">${mcp?icons.agent:esc(author)}</span><strong>${esc(author)}</strong>${mcp?'':'<span>Just now</span>'}</div><p>${esc(text)}</p>${mcp?'<div class="mcp-note">MCP · comments.reply</div>':''}</div>`;
  function renderComments(){
    $('[data-comment-count]').textContent=comments.filter(c=>!c.resolved).length;
    $('.comment-pins').innerHTML=comments.filter(c=>!c.resolved || $('[data-comment-filter]').value!=='open').map(c=>`<button class="comment-pin ${c.resolved?'resolved':''}" data-pin="${c.id}" aria-label="Comment ${c.id}: ${esc(c.layer)}" aria-pressed="${c.id===activeThread}">${c.id}</button>`).join('');
    const filter=$('[data-comment-filter]').value, visible=comments.filter(c=>filter==='all'||c.resolved===(filter==='resolved'));
    $('.comment-list').innerHTML=visible.map(c=>`<article class="comment-card ${c.id===activeThread?'selected':''}"><button data-open-thread="${c.id}"><div class="thread-meta"><span class="avatar">${esc(c.author)}</span><strong>${esc(c.author)}</strong><span>· ${esc(c.layer)}</span></div><p>${esc(c.text)}</p><span class="comment-location">${c.replies.length?`${c.replies.length} ${c.replies.length===1?'reply':'replies'}`:'No replies'}${c.replies.some(r=>r.mcp)?' · Agent replied':''}${c.resolved?' · Resolved':''}</span></button></article>`).join('')||'<p class="muted">No threads here.</p>';
    const c=comments.find(c=>c.id===activeThread);$('.pin-thread').hidden=!c;
    if(c){$('.pin-thread').innerHTML=`<div class="thread-heading"><strong>${esc(c.layer)}</strong><span class="muted">#${c.id}</span><button class="icon-button" data-action="close-thread" aria-label="Close thread">${icons.close}</button></div>${messageHTML(c.author,c.text)}${c.replies.map(r=>messageHTML(r.author,r.text,r.mcp)).join('')}<div class="thread-actions"><span class="muted">${c.resolved?'Resolved':'Open thread'}</span><button data-resolve="${c.id}">${icons.check}${c.resolved?'Reopen':'Resolve'}</button></div><form class="reply-form"><input aria-label="Reply to comment" placeholder="Reply…" required maxlength="500"><button aria-label="Send reply" type="submit">↵</button></form>`;}
  }
  function openThread(id){setTab('comments');activeThread=Number(id);renderComments();window.__drawVoxelScene();}
  $('[data-comment-filter]').addEventListener('change',()=>{activeThread=null;renderComments();window.__drawVoxelScene();});
  function addCommentAtContext(){const hit=window.__pickVoxel?.(contextPoint);if(!hit){notify('Right-click a voxel to anchor a comment.');return;}comments.push({id:Math.max(...comments.map(c=>c.id))+1,anchor:[hit.x,hit.y,hit.z],layer:hit.layer,author:'You',text:'New comment',resolved:false,replies:[]});openThread(comments.at(-1).id);$('.reply-form input').focus();}
  window.__projectComments=(project,cubes,hidden)=>{
    for(const el of $$('[data-pin]')){const c=comments.find(c=>c.id===Number(el.dataset.pin));const [x,y,depth]=project(c.anchor.map(v=>v+.5));const anchorExists=cubes.has(c.anchor.join(','));el.hidden=!pinsVisible||hidden.has(c.layer)||!anchorExists||depth<0||x<20||y<36||x>$('.viewport').clientWidth-20||y>$('.viewport').clientHeight-20;el.style.left=x+'px';el.style.top=y+'px';if(c.id===activeThread){const thread=$('.pin-thread');thread.hidden=el.hidden;const left=Math.min($('.viewport').clientWidth-312,Math.max(16,x+25));const top=Math.min($('.viewport').clientHeight-thread.offsetHeight-16,Math.max(18,y-65));thread.style.left=left+'px';thread.style.top=top+'px';}}
    $('[data-zoom-label]').textContent=$('[data-zoom-select]').value+'%';
  };
  document.addEventListener('submit',e=>{if(e.target.matches('.reply-form')){e.preventDefault();const input=e.target.querySelector('input'),text=input.value.trim();if(!text)return;comments.find(c=>c.id===activeThread).replies.push({author:'You',text});renderComments();window.__drawVoxelScene();$('.reply-form input').focus();}});

  let permission='comment';
  function openShare(){closeFloating();$('.share-popover').hidden=false;$('[data-action="share"]').setAttribute('aria-expanded','true');$(`[data-permission="${permission}"]`).focus();}
  async function copyLink(){const input=$('.link-field input');try{await navigator.clipboard.writeText(input.value);notify('Preview link copied');}catch{input.focus();input.select();notify('Press ⌘C or Ctrl+C to copy the preview link.');}}

  // Timeline snapshots are reconstructed from the procedural cottage's actual cells.
  const historyNames=['Empty project','Ground plane','Cottage walls','Warm tiled roof','Garden & path','Tree structure','Full canopy','Shape the canopy','Leaf detail'];
  let liveSnapshot, timelineStep=8, branch='main';
  const versions=[{id:'v1',name:'Cottage study',step:4,snapshot:window.__historySnapshot(4)},{id:'v2',name:'Canopy pass',step:8,snapshot:window.__sceneSnapshot()}];
  function renderHistory(){
    $('[data-history-position]').textContent=`${timelineStep} of 8`;$('[data-history-step-title]').textContent=historyNames[timelineStep];$('[data-timeline]').value=timelineStep;$('[data-branch-name]').textContent=branch;
    const start=Math.max(1,Math.min(timelineStep-1,5));$('.history-event-list').innerHTML=Array.from({length:4},(_,i)=>start+i).map(step=>`<button class="history-event ${step===timelineStep?'active':''}" data-history-tick="${step}"><span class="event-origin">${step<=4?'<i class="human-marker"></i>':icons.agent}</span><div><span>${historyNames[step]}</span><small>${step<=4?'Human':'Agent'} · Step ${step}</small></div></button>`).join('');
    $('.version-list').innerHTML=versions.map(v=>`<article class="version-item"><h3>${esc(v.name)}</h3><p class="version-meta">${v.snapshot.length.toLocaleString()} voxels · Step ${v.step}</p><div class="version-actions"><button data-version-action="restore" data-version-id="${v.id}">Restore</button><button data-version-action="compare" data-version-id="${v.id}">Compare</button><button data-version-action="branch" data-version-id="${v.id}">Branch from here</button></div></article>`).join('');
  }
  function openHistory(){closeFloating();liveSnapshot=window.__sceneSnapshot();timelineStep=window.__currentHistoryStep??8;$('.history-panel').hidden=false;$('[data-action="history"]').setAttribute('aria-expanded','true');renderHistory();$('[data-timeline]').focus();}
  function scrub(step){timelineStep=Number(step);window.__setHistory(timelineStep);syncSteps();renderHistory();}
  function closeHistory(){if(!$('.history-panel').hidden&&liveSnapshot){window.__loadSnapshot(liveSnapshot);syncSteps();}$('.history-panel').hidden=true;$('[data-action="history"]').setAttribute('aria-expanded','false');window.__compareScene(null);$('.compare-badge').hidden=true;$('.version-form').hidden=true;}
  $('[data-timeline]').addEventListener('input',e=>scrub(e.target.value));
  $('.version-form').addEventListener('submit',e=>{e.preventDefault();const input=e.target.querySelector('input'),name=input.value.trim();if(!name)return;versions.unshift({id:uid('version'),name,step:timelineStep,snapshot:window.__sceneSnapshot()});input.value='';e.target.hidden=true;renderHistory();notify('Named version saved in this preview.');});
  function versionAction(action,id){const v=versions.find(v=>v.id===id);if(action==='compare'){const counts=window.__compareScene(v.snapshot);$('.compare-badge').innerHTML=`<span>Compared with ${esc(v.name)}</span><code>+${counts.added} / −${counts.removed}</code><span class="muted">Solid added · dashed removed</span><button data-action="clear-compare">Done</button>`;$('.compare-badge').hidden=false;return;}window.__loadSnapshot(v.snapshot,true);timelineStep=v.step;liveSnapshot=window.__sceneSnapshot();if(action==='branch')branch=v.name.toLowerCase().replace(/[^a-z0-9]+/g,'-')+'-branch';renderHistory();syncSteps();notify(action==='branch'?'Created local preview branch':'Version restored in this preview');}
  function closeFloating(){closeMenus();$('.share-popover').hidden=true;$('[data-action="share"]').setAttribute('aria-expanded','false');closeHistory();}
  function openDialog(dialog){modalFocus=document.activeElement===document.body?menuTrigger:document.activeElement;closeMenus();dialog.showModal();}
  for(const dialog of $$('dialog'))dialog.addEventListener('close',()=>modalFocus?.focus());

  const actions={
    commands:openCommands,palettes:openManager,comments:()=>setTab($('#comments-panel').hidden?'comments':'session'),history:()=>$('.history-panel').hidden?openHistory():closeHistory(),share:()=>$('.share-popover').hidden?openShare():closeFloating(),
    'copy-link':copyLink,follow:()=>setFollow(!follow),pause,frame:()=>{$('[data-reset-view]').click();},grid:()=>$('[data-toggle-grid]').click(),theme:()=>document.documentElement.dataset.theme=document.documentElement.dataset.theme==='light'?'dark':'light',refresh:()=>{window.__captureViews();notify('Captured 3 views');},
    'create-palette':()=>createPalette(),'duplicate-palette':()=>createPalette(true),'use-palette':()=>{selectPalette(editingPalette);$('.palette-dialog').close();},
    'delete-palette':()=>{if(editing().preset)return;const id=editingPalette;custom=custom.filter(p=>p.id!==id);if(activePalette===id)activePalette=palettes()[0].id;editingPalette=activePalette;editorIndex=0;updatePalette();},
    'swatch-left':()=>moveSwatch(-1),'swatch-right':()=>moveSwatch(1),'remove-swatch':()=>{const p=editing();if(p.preset||p.colors.length===1)return;p.colors.splice(editorIndex,1);updatePalette();},'add-swatch':()=>{const p=editing();if(p.preset||p.colors.length>=256)return;p.colors.push('#829a63');editorIndex=p.colors.length-1;updatePalette();},
    'export-hex':()=>exportPalette('hex'),'export-gpl':()=>exportPalette('gpl'),'import-palette':()=>{if(!$('.palette-dialog').open)openManager();$('[data-palette-file]').click();},
    'close-thread':()=>{activeThread=null;renderComments();window.__drawVoxelScene();},'history-live':()=>{window.__loadSnapshot(liveSnapshot);timelineStep=window.__currentHistoryStep??8;renderHistory();syncSteps();},
    'save-version':()=>{$('.version-form').hidden=false;$('.version-form input').focus();},'clear-compare':()=>{window.__compareScene(null);$('.compare-badge').hidden=true;}
  };
  function moveSwatch(direction){const p=editing(),next=editorIndex+direction;if(p.preset||next<0||next>=p.colors.length)return;[p.colors[editorIndex],p.colors[next]]=[p.colors[next],p.colors[editorIndex]];editorIndex=next;updatePalette();}
  function run(action){if(typeof action==='function')action();else actions[action]?.();}

  const commands=[...tools.map(([id,label,key])=>item(label,()=>chooseTool(id),key,id)),item('Manage palettes','palettes','','swatches'),item('Version history','history','','clock'),item('Comments','comments','','comment'),item('Share project','share','','link'),item('Frame model','frame','Home','focus'),item('Follow agent','follow','⇧F','camera'),item('Refresh captures','refresh','','refresh'),item('Toggle theme','theme','','sun'),item('Scatter moss',()=>chooseTool('scatter'),'','agent')];
  let commandIndex=0,filteredCommands=[];
  function renderCommands(){const value=$('.command-search input').value.toLowerCase();filteredCommands=commands.filter(c=>c.label.toLowerCase().includes(value));commandIndex=Math.max(0,Math.min(commandIndex,filteredCommands.length-1));$('.command-results').innerHTML=filteredCommands.map((c,i)=>`<button id="cmd-${i}" role="option" aria-selected="${i===commandIndex}" tabindex="-1" data-command="${i}">${icons[c.icon]||''}<span>${c.label}</span><kbd>${c.shortcut}</kbd></button>`).join('')||'<p class="command-empty">No commands found.</p>';if(filteredCommands.length)$('.command-search input').setAttribute('aria-activedescendant',`cmd-${commandIndex}`);else $('.command-search input').removeAttribute('aria-activedescendant');}
  function openCommands(){closeFloating();commandIndex=0;$('.command-search input').value='';renderCommands();openDialog($('.command-dialog'));$('.command-search input').focus();}
  $('.command-search input').addEventListener('input',()=>{commandIndex=0;renderCommands();});
  $('.command-dialog').addEventListener('keydown',e=>{if(['ArrowUp','ArrowDown'].includes(e.key)){e.preventDefault();if(filteredCommands.length)commandIndex=(commandIndex+(e.key==='ArrowDown'?1:-1)+filteredCommands.length)%filteredCommands.length;renderCommands();$(`[data-command="${commandIndex}"]`)?.scrollIntoView({block:'nearest'});}if(e.key==='Enter'){e.preventDefault();$(`[data-command="${commandIndex}"]`)?.click();}});
  document.addEventListener('click',e=>{
    const el=e.target.closest('button');if(!el)return;
    if(el.dataset.menu){const open=el.getAttribute('aria-expanded')==='true';if(open)closeMenus(true);else showMenu(el.dataset.menu,el);return;}
    if(el.dataset.action){run(el.dataset.action);return;}
    if(el.dataset.close){const id=el.dataset.close;if(id==='palettes')$('.palette-dialog').close();else if(id==='history'){closeHistory();$('[data-action="history"]').focus();}else {closeFloating();$('[data-action="share"]').focus();}return;}
    if(el.dataset.tab){setTab(el.dataset.tab);return;}
    if(el.dataset.editPalette){editingPalette=el.dataset.editPalette;editorIndex=0;renderManager();return;}
    if(el.hasAttribute('data-swatch')){swatchIndex=Number(el.dataset.swatch);renderPalette();$(`[data-swatch="${swatchIndex}"]`).focus();return;}
    if(el.hasAttribute('data-edit-swatch')){editorIndex=Number(el.dataset.editSwatch);renderManager();$(`[data-edit-swatch="${editorIndex}"]`).focus();return;}
    if(el.dataset.stepUndo){toggleStep(el.dataset.stepUndo);return;}
    if(el.dataset.viewAngle){setFollow(false);window.__setViewAngle(el.dataset.viewAngle);return;}
    if(el.dataset.selectLayer){selectedLayer=el.dataset.selectLayer;$$('[data-select-layer]').forEach(b=>{b.setAttribute('aria-pressed',String(b===el));b.closest('.layer-row').classList.toggle('selected',b===el);});window.__updateSelection();return;}
    if(el.dataset.pin||el.dataset.openThread){openThread(el.dataset.pin||el.dataset.openThread);return;}
    if(el.dataset.resolve){const c=comments.find(c=>c.id===Number(el.dataset.resolve));c.resolved=!c.resolved;if(c.resolved){activeThread=null;}renderComments();window.__drawVoxelScene();return;}
    if(el.dataset.permission){permission=el.dataset.permission;$$('[data-permission]').forEach(b=>b.setAttribute('aria-pressed',String(b===el)));$('.link-field input').value=`https://voxelite.example/s/woodland?access=${permission}`;return;}
    if(el.hasAttribute('data-history-tick')){scrub(el.dataset.historyTick);return;}
    if(el.dataset.versionAction){versionAction(el.dataset.versionAction,el.dataset.versionId);return;}
    if(el.hasAttribute('data-command')){const c=filteredCommands[Number(el.dataset.command)];$('.command-dialog').close();run(c.action);}
  });
  document.addEventListener('pointerdown',e=>{if(!e.target.closest('.menu-root,[data-menu]'))closeMenus();if(!$('.share-popover').hidden&&!e.target.closest('.share-popover,[data-action="share"]')){$('.share-popover').hidden=true;$('[data-action="share"]').setAttribute('aria-expanded','false');}});
  $$('.menubar>[data-menu]').forEach(el=>{el.addEventListener('pointerenter',()=>{if(menuChain.length&&menuTrigger?.closest('.menubar'))showMenu(el.dataset.menu,el);});el.addEventListener('keydown',e=>{if(['ArrowDown','ArrowUp'].includes(e.key)){e.preventDefault();showMenu(el.dataset.menu,el);menuChain[0].element.querySelector('button')?.focus();}if(['ArrowLeft','ArrowRight'].includes(e.key)){e.preventDefault();const buttons=$$('.menubar>[data-menu]'),i=buttons.indexOf(el);buttons[(i+(e.key==='ArrowRight'?1:-1)+buttons.length)%buttons.length].focus();}});});
  document.addEventListener('contextmenu',e=>{const layer=e.target.closest('[data-layer]'),paletteArea=e.target.closest('.palette-section'),view=e.target.closest('.viewport');if(!layer&&!paletteArea&&!view)return;e.preventDefault();if(layer)layer.querySelector('[data-select-layer]').click();if(view){const r=$('.viewport').getBoundingClientRect();contextPoint=[e.clientX-r.left,e.clientY-r.top];}showMenu(layer?'Layers':paletteArea?'Palette':'ViewportContext',null,{x:e.clientX,y:e.clientY});});
  document.addEventListener('keydown',e=>{
    if(e.target.hasAttribute('data-tab') && ['ArrowLeft','ArrowRight'].includes(e.key)){e.preventDefault();const tab=e.target.dataset.tab==='session'?'comments':'session';setTab(tab);$(`[data-tab="${tab}"]`).focus();return;}
    const swatch=e.target.closest('[data-swatch],[data-edit-swatch]');if(swatch&&['ArrowRight','ArrowLeft','ArrowUp','ArrowDown','Home','End'].includes(e.key)){e.preventDefault();e.stopPropagation();const editor=swatch.hasAttribute('data-edit-swatch'),items=[...swatch.parentElement.children],i=items.indexOf(swatch),next=e.key==='Home'?0:e.key==='End'?items.length-1:(i+({ArrowRight:1,ArrowLeft:-1,ArrowUp:-8,ArrowDown:8}[e.key])+items.length)%items.length;items[next].click();return;}
    if(e.key==='Escape'){closeMenus(true);closeFloating();activeThread=null;renderComments();window.__drawVoxelScene();return;}
    if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='k'){e.preventDefault();if($('.palette-dialog').open)return;$('.command-dialog').open?$('.command-dialog').close():openCommands();return;}
    if($('dialog[open]')||menuChain.length||/INPUT|TEXTAREA|SELECT/.test(e.target.tagName))return;
    if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='z'){e.preventDefault();e.shiftKey?redo():undo();return;}
    if(e.metaKey||e.ctrlKey){const key=e.key.toLowerCase();if(['n','o','s'].includes(key)){e.preventDefault();notify(({n:'New project',o:'Open .voxl',s:e.shiftKey?'Save as .voxl':'Save .voxl'})[key]+' · concept preview');}return;}
    if(e.shiftKey&&e.key.toLowerCase()==='f'){e.preventDefault();setFollow(!follow);return;}const choice=tools.find(t=>t[2].toLowerCase()===e.key.toLowerCase());if(choice){e.preventDefault();chooseTool(choice[0]);}if(e.key==='Home'){e.preventDefault();actions.frame();}
  });
  renderPalette();renderComments();syncSteps();window.__drawVoxelScene();
  window.__aPreview={exportText,parsePalette,get palettes(){return palettes();},get comments(){return comments;},get versions(){return versions;},get tool(){return tool;},get mirrors(){return [...mirrors];}};
})();
