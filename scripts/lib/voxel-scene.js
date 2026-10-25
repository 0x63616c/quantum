/* Embedded in every concept. A tiny, dependency-free, perspective voxel renderer. */
(() => {
  const mainCanvas = document.querySelector('.viewport > canvas');
  let canvas = mainCanvas;
  let ctx = canvas.getContext('2d');
  const viewport = canvas.parentElement;
  const cubes = new Map();
  const key = (x, y, z) => `${x},${y},${z}`;
  const random = (x, y, z) => {
    const n = Math.sin(x * 127.1 + y * 311.7 + z * 74.7) * 43758.5453;
    return n - Math.floor(n);
  };
  const rgb = hex => hex.match(/[a-f\d]{2}/gi).map(v => parseInt(v, 16));
  const shade = (color, value) => `rgb(${color.map(v => Math.min(255, Math.round(v * value))).join(',')})`;
  const put = (x, y, z, color, layer = 'Garden', agent = false) => {
    const noise = .96 + random(x, y, z) * .08;
    cubes.set(key(x, y, z), { x, y, z, color: rgb(color).map(v => v * noise), layer, agent });
  };
  const box = (x1, x2, y1, y2, z1, z2, color, layer) => {
    for (let x = x1; x <= x2; x++) for (let y = y1; y <= y2; y++) for (let z = z1; z <= z2; z++) put(x, y, z, color, layer);
  };

  // A small hand-composed woodland diorama, built entirely from real cubes.
  for (let x = -10; x <= 10; x++) for (let z = -8; z <= 9; z++) {
    if ((Math.abs(x) > 8 && (z < -6 || z > 7)) || (Math.abs(x) > 9 && (z < -5 || z > 6))) continue;
    put(x, 0, z, '#687359');
    put(x, 1, z, '#88916a');
    put(x, 2, z, ['#a3ad79', '#a1aa76', '#a9b17d', '#9da774'][Math.floor(random(x, 2, z) * 4)]);
  }
  // Foundation, limewashed walls, oak beams and a steep tiled roof.
  box(-7, 1, 3, 3, -5, 3, '#b6ad94', 'Cottage');
  for (let y = 4; y <= 10; y++) for (let x = -7; x <= 1; x++) for (let z = -5; z <= 3; z++) {
    if (x === -7 || x === 1 || z === -5 || z === 3) {
      const beam = ((x === -7 || x === 1) && (z === -5 || z === 3)) || y === 10;
      put(x, y, z, beam ? '#8c7253' : '#e9dbb5', 'Cottage');
    }
  }
  for (let x = -8; x <= 2; x++) {
    const roofY = 16 - Math.abs(x + 3);
    for (let z = -6; z <= 4; z++) {
      if (x > -8 && x < 2 && z > -6 && z < 4) {
        for (let y = 11; y < roofY; y++) put(x, y, z, z === 3 || z === -5 ? '#e3d2a8' : '#a68160', 'Cottage');
      }
      put(x, roofY, z, ['#b75f43', '#c26b4c', '#b86247', '#c16a4a'][Math.floor(random(x, roofY, z) * 4)], 'Roof');
      if (z === 4 || z === -6) put(x, roofY - 1, z, '#8e4d39', 'Roof');
    }
  }
  box(-1, 0, 14, 18, -4, -3, '#ad9c84', 'Roof');
  box(-1, 0, 19, 19, -4, -3, '#c9b79b', 'Roof');
  put(-1, 19, -3, '#817363', 'Roof');
  // Door, clerestory and warm glass with wooden window ledges.
  box(-3, -2, 4, 7, 3, 3, '#496969', 'Cottage');
  box(-3, -2, 8, 8, 3, 3, '#a18a61', 'Cottage');
  put(-2, 5, 3, '#688075', 'Cottage');
  box(-3, -2, 3, 3, 4, 5, '#c3b69a', 'Cottage');
  box(-6, -5, 7, 8, 3, 3, '#e4b96c', 'Cottage');
  box(-6, -5, 6, 6, 4, 4, '#80634a', 'Cottage');
  box(-6, -5, 7, 7, 4, 4, '#607655', 'Garden');
  put(-6, 8, 4, '#d3917e');
  put(-5, 8, 4, '#d7b680');
  box(-4, -2, 11, 11, 3, 3, '#8a6c4e', 'Cottage');
  put(-3, 12, 3, '#e3b772', 'Cottage');
  for (const z of [-3, 1]) {
    box(1, 1, 6, 8, z - 1, z, '#917a57', 'Cottage');
    box(1, 1, 7, 8, z - 1, z, '#c8ab75', 'Cottage');
    box(2, 2, 6, 6, z - 1, z, '#9f8762', 'Cottage');
  }
  // Stepping stones leading to the front door.
  for (const [x, z] of [[-3, 6], [-2, 6], [-2, 8], [-1, 8], [0, 9]]) put(x, 3, z, '#c5bca2');
  // Tree trunk and several overlapping cube-volume boughs.
  box(6, 6, 3, 11, -3, -3, '#8b7352', 'Canopy');
  box(5, 7, 8, 9, -3, -3, '#8b7352', 'Canopy');
  const foliage = [[6, 12, -3, 3.4, 3.5, 3.3], [4, 11, -2, 2.5, 2.4, 2.6], [8, 11, -3, 2.7, 2.8, 2.8], [6, 14, -4, 2.3, 2.5, 2.6]];
  for (const [cx, cy, cz, rx, ry, rz] of foliage) {
    for (let x = Math.floor(cx - rx); x <= cx + rx; x++) for (let y = Math.floor(cy - ry); y <= cy + ry; y++) for (let z = Math.floor(cz - rz); z <= cz + rz; z++) {
      const d = ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 + ((z - cz) / rz) ** 2;
      if (d < 1.04) put(x, y, z, ['#7f9660', '#829a63', '#738d59', '#8da36b'][Math.floor(random(x, y, z) * 4)], 'Canopy');
    }
  }
  for (const [x, y, z] of [[8, 13, -1], [8, 14, -2], [7, 15, -2], [7, 15, -3], [8, 14, -3], [8, 14, -1]]) put(x, y, z, '#9eae77', 'Canopy', true);
  // Low garden beds, a fence and tiny ochre flowers.
  for (const [x, z] of [[5, 4], [6, 4], [7, 5], [5, 5], [-8, 5], [-8, 6], [7, 0]]) {
    put(x, 3, z, '#6e8657');
    if (random(x, 0, z) > .25) put(x, 4, z, '#839a5d');
  }
  for (const [x, z] of [[5, 4], [7, 5], [-8, 6]]) put(x, 5, z, '#e4bd6b');
  for (const z of [1, 4, 7]) box(9, 9, 3, 5, z, z, '#c3af87');
  box(9, 9, 4, 4, 1, 7, '#c3af87');
  put(6, 3, 7, '#acaa91');
  put(7, 3, 7, '#b5b298');

  const dot = (a, b) => a.reduce((sum, n, i) => sum + n * b[i], 0);
  const subtract = (a, b) => a.map((n, i) => n - b[i]);
  const unit = a => a.map(n => n / Math.hypot(...a));
  const cross = (a, b) => [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]];
  let width, height, camera, forward, right, up, focal, center;
  let azimuth = .68, elevation = .46, zoom = 1, pan = [0, 0];
  const hidden = new Set();
  let grid = true;
  let revision = 18;
  const undone = new Set();
  const historyGroups = {
    '18': [...cubes.values()].filter(c => c.agent),
    '17': [...cubes.values()].filter(c => c.layer === 'Canopy' && !c.agent && c.y >= 14).slice(0, 24),
  };
  let thumbnailKey = '';
  const views = { front: [.68, .46], right: [2.25, .42], back: [3.8, .46] };
  function selectCamera(name) {
    document.querySelectorAll('[data-view-angle]').forEach(el => el.setAttribute('aria-pressed', String(el.dataset.viewAngle === name)));
  }
  const bounds = items => [0,1,2].map((axis) => {
    const values = items.map(c => [c.x,c.y,c.z][axis]);
    return `${'xyz'[axis]} ${Math.min(...values)}…${Math.max(...values)}`;
  }).join(' · ').replaceAll('-', '−');
  window.__getChange = id => ({
    step: Number(id), tool: id === '18' ? 'set_voxels' : 'fill_box', layer: 'Canopy',
    revision, state: undone.has(id) ? 'undone' : 'applied',
    added: historyGroups[id].map(c => [c.x,c.y,c.z]), removed: [],
  });
  window.__updateSelection = () => {
    const selected = document.querySelector('[data-select-layer][aria-pressed=true]')?.dataset.selectLayer;
    const items = [...cubes.values()].filter(c => c.layer === selected);
    document.querySelectorAll('[data-selected-layer]').forEach(el => el.textContent = selected);
    document.querySelectorAll('[data-selection-count]').forEach(el => el.textContent = `${items.length} voxels`);
    document.querySelectorAll('[data-selection-bounds]').forEach(el => el.textContent = bounds(items));
    document.querySelectorAll('.selection-inspector [data-inspect-step]').forEach(el => el.hidden = selected !== 'Canopy');
  };
  const project = p => {
    const offset = subtract(p, camera);
    const depth = dot(offset, forward);
    return [center[0] + focal * dot(offset, right) / depth, center[1] - focal * dot(offset, up) / depth, depth];
  };
  const path = points => {
    ctx.beginPath();
    points.forEach((p, i) => i ? ctx.lineTo(p[0], p[1]) : ctx.moveTo(p[0], p[1]));
    ctx.closePath();
  };
  const faceDefinitions = [
    { n: [0,1,0], vertices: [[0,1,0],[0,1,1],[1,1,1],[1,1,0]], light: 1.10 },
    { n: [0,0,1], vertices: [[0,0,1],[1,0,1],[1,1,1],[0,1,1]], light: .91 },
    { n: [1,0,0], vertices: [[1,0,0],[1,1,0],[1,1,1],[1,0,1]], light: .72 },
    { n: [0,0,-1], vertices: [[1,0,0],[0,0,0],[0,1,0],[1,1,0]], light: .74 },
    { n: [-1,0,0], vertices: [[0,0,1],[0,1,1],[0,1,0],[0,0,0]], light: .94 }
  ];
  function paint(thumbnail = false) {
    width = canvas.clientWidth; height = canvas.clientHeight;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(width * dpr); canvas.height = Math.round(height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.imageSmoothingEnabled = true;
    const light = document.documentElement.dataset.theme === 'light';
    const styles = getComputedStyle(document.body);
    const accent = styles.getPropertyValue('--accent').trim();
    const radius = 65;
    camera = [Math.sin(azimuth) * Math.cos(elevation) * radius, 7 + Math.sin(elevation) * radius, Math.cos(azimuth) * Math.cos(elevation) * radius];
    forward = unit(subtract([0,7,0], camera));
    right = unit(cross(forward, [0,1,0])); up = cross(right, forward);
    focal = thumbnail ? Math.min(width * 2.15, height * 2) : Math.min(width * 1.3, height * (document.body.classList.contains('concept-c') ? 1.67 : 1.43)) * zoom;
    center = thumbnail ? [width * .5, height * .51] : [width * .48 + pan[0], height * .49 + pan[1]];
    ctx.clearRect(0, 0, width, height);
    // Perspective grid, gently fading at the edges of the work plane.
    if (!thumbnail && grid) for (let i = -50; i <= 50; i += 2) {
      ctx.lineWidth = i % 10 === 0 ? .85 : .6;
      for (let j = -50; j < 50; j += 5) {
        const alpha = Math.max(0, 1 - Math.hypot(i, j + 2.5) / 54) ** 1.4 * .10;
        ctx.strokeStyle = light ? `rgba(100,108,115,${alpha})` : `rgba(163,174,177,${alpha})`;
        for (const points of [[[i,-.12,j],[i,-.12,j+5]], [[j,-.12,i],[j+5,-.12,i]]]) {
          const a = project(points[0]), b = project(points[1]);
          ctx.beginPath(); ctx.moveTo(a[0],a[1]); ctx.lineTo(b[0],b[1]); ctx.stroke();
        }
      }
    }
    // Soft ground shadow. Rendered in screen space under the voxel geometry.
    const ground = project([1, 0, 1]);
    ctx.save();
    const shadowScale = thumbnail ? width / 800 : 1;
    ctx.translate(ground[0] + 16 * shadowScale, ground[1] + 10 * shadowScale);
    ctx.scale(shadowScale, shadowScale);
    ctx.scale(1, .38);
    const shadow = ctx.createRadialGradient(0,0,15,0,0,260 * zoom);
    shadow.addColorStop(0, light ? 'rgba(44,47,36,.28)' : 'rgba(0,0,0,.36)');
    shadow.addColorStop(.65, light ? 'rgba(44,47,36,.12)' : 'rgba(0,0,0,.2)');
    shadow.addColorStop(1,'rgba(0,0,0,0)');
    ctx.fillStyle = shadow; ctx.fillRect(-350 * zoom,-350 * zoom,700 * zoom,700 * zoom); ctx.restore();
    const faces = [];
    for (const cube of cubes.values()) {
      if (hidden.has(cube.layer)) continue;
      for (const face of faceDefinitions) {
        const neighbor = cubes.get(key(cube.x + face.n[0],cube.y + face.n[1],cube.z + face.n[2]));
        if (neighbor && !hidden.has(neighbor.layer)) continue;
        const faceCenter = [cube.x + .5 + face.n[0] * .5, cube.y + .5 + face.n[1] * .5, cube.z + .5 + face.n[2] * .5];
        if (dot(face.n, subtract(camera, faceCenter)) <= 0) continue;
        const points = face.vertices.map(v => project([cube.x + v[0],cube.y + v[1],cube.z + v[2]]));
        faces.push({ cube, face, points, depth: project(faceCenter)[2] });
      }
    }
    faces.sort((a,b) => b.depth - a.depth);
    for (const {cube, face, points} of faces) {
      let ao = 1;
      if (face.n[1] === 1) {
        for (const [dx,dz] of [[-1,0],[1,0],[0,1],[0,-1]]) {
          const next = cubes.get(key(cube.x + dx, cube.y + 1, cube.z + dz));
          if (next && !hidden.has(next.layer)) ao -= .055;
        }
      } else if (cube.y === 3 || cube.y === 4) ao = .96;
      path(points);
      ctx.fillStyle = shade(cube.color, face.light * ao);
      ctx.strokeStyle = ctx.fillStyle;
      ctx.lineWidth = .45;
      ctx.fill(); ctx.stroke();
      if (cube.agent) {
        ctx.globalAlpha = .13; ctx.fillStyle = accent; ctx.fill(); ctx.globalAlpha = 1;
        ctx.strokeStyle = accent; ctx.lineWidth = .85; ctx.stroke();
      }
    }
    if (thumbnail) return;
    const anchor = historyGroups['18'].map(c => project([c.x + .5, c.y + .5, c.z + .5]))
      .reduce((a,b) => a[0] > b[0] ? a : b);
    const annotation = document.querySelector('.edit-annotation');
    annotation.hidden = false;
    const tagWidth = annotation.offsetWidth;
    const flipped = anchor[0] + tagWidth > width - 12;
    const tagLeft = flipped ? anchor[0] - tagWidth : anchor[0];
    annotation.classList.toggle('flipped', flipped);
    annotation.style.left = `${tagLeft}px`;
    annotation.style.top = `${anchor[1] - 15}px`;
    annotation.hidden = hidden.has('Canopy') || undone.has('18') || tagLeft < 12 ||
      tagLeft + tagWidth > width - 12 || anchor[1] < 42 || anchor[1] > height - 42;
    const count = document.querySelectorAll('[data-voxel-count]');
    count.forEach(el => el.textContent = cubes.size.toLocaleString('en-US'));
    document.querySelectorAll('[data-zoom]').forEach(el => el.textContent = `${Math.round(100 * zoom)}%`);
    window.__voxelSceneReady = true;
    window.__voxelSceneStats = { cubes: cubes.size, faces: faces.length, width, height, dpr };
  }
  function draw() {
    paint();
    const nextKey = [document.documentElement.dataset.theme, [...hidden].join(), [...undone].join()].join('/');
    if (nextKey !== thumbnailKey) {
      const saved = [azimuth, elevation, zoom, pan];
      document.querySelectorAll('[data-thumbnail]').forEach(thumbnail => {
        canvas = thumbnail; ctx = canvas.getContext('2d');
        [azimuth, elevation] = views[thumbnail.dataset.thumbnail];
        zoom = 1; pan = [0, 0]; paint(true);
      });
      [azimuth, elevation, zoom, pan] = saved;
      canvas = mainCanvas; ctx = canvas.getContext('2d');
      thumbnailKey = nextKey;
    }
  }
  window.__setViewAngle = name => {
    [azimuth, elevation] = views[name] || views.front;
    zoom = 1; pan = [0, 0]; selectCamera(name); draw();
  };
  window.__toggleStep = id => {
    const group = historyGroups[id];
    if (!group) return;
    if (undone.has(id)) {
      group.forEach(cube => cubes.set(key(cube.x, cube.y, cube.z), cube));
      undone.delete(id);
    } else {
      group.forEach(cube => cubes.delete(key(cube.x, cube.y, cube.z)));
      undone.add(id);
    }
    revision++;
    const netAdded = Object.entries(historyGroups).filter(([step]) => !undone.has(step)).reduce((sum,[,items]) => sum + items.length, 0);
    document.querySelectorAll('[data-session-total]').forEach(el => el.innerHTML = `+${netAdded} <span>−0</span>`);
    document.querySelectorAll('[data-revision]').forEach(el => el.textContent = 'r' + String(revision).padStart(3, '0'));
    document.querySelectorAll('[data-render-revision]').forEach(el => el.textContent = 'r' + String(revision).padStart(3, '0'));
    window.__updateSelection();
    draw();
    return undone.has(id);
  };
  document.querySelectorAll('[data-toggle-grid]').forEach(el => el.addEventListener('click', () => {
    grid = !grid; el.setAttribute('aria-pressed', String(grid)); draw();
  }));
  // A few real viewport interactions make the standalone concepts explorable.
  let drag;
  canvas.addEventListener('pointerdown', e => {
    if (e.button === 2) return;
    document.dispatchEvent(new Event('manual-camera'));
    selectCamera(null);
    drag = { x:e.clientX, y:e.clientY, pan: e.shiftKey || e.button === 1 };
    canvas.setPointerCapture(e.pointerId); canvas.classList.add('dragging');
  });
  canvas.addEventListener('pointermove', e => {
    if (!drag) return;
    const dx = e.clientX - drag.x, dy = e.clientY - drag.y;
    if (drag.pan) { pan[0] += dx; pan[1] += dy; }
    else { azimuth += dx * .006; elevation = Math.max(.16,Math.min(1.2,elevation + dy * .004)); }
    drag.x = e.clientX; drag.y = e.clientY; draw();
  });
  const stop = () => { drag = null; canvas.classList.remove('dragging'); };
  canvas.addEventListener('pointerup', stop); canvas.addEventListener('pointercancel', stop);
  canvas.addEventListener('wheel', e => { e.preventDefault(); document.dispatchEvent(new Event('manual-camera')); zoom = Math.max(.55,Math.min(1.8,zoom - e.deltaY * .001)); draw(); }, { passive:false });
  document.querySelectorAll('[data-reset-view]').forEach(el => el.addEventListener('click', () => {
    document.dispatchEvent(new Event('manual-camera'));
    window.__setViewAngle('front');
  }));
  document.querySelectorAll('[data-layer-eye]').forEach(el => el.addEventListener('click', () => {
    const name = el.dataset.layerEye;
    if (hidden.has(name)) hidden.delete(name); else hidden.add(name);
    el.setAttribute('aria-pressed', String(!hidden.has(name)));
    el.querySelector('svg').innerHTML = '<path d="M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6-10-6-10-6Z"/><circle cx="12" cy="12" r="2.5"/>' +
      (hidden.has(name) ? '<path d="m3 3 18 18"/>' : '');
    el.closest('.layer-row').classList.toggle('layer-hidden',hidden.has(name)); draw();
  }));
  new ResizeObserver(draw).observe(viewport);
  new MutationObserver(draw).observe(document.documentElement, { attributes:true, attributeFilter:['data-theme'] });
  window.__drawVoxelScene = draw;
  window.__updateSelection();
  document.querySelectorAll('[data-change-bounds]').forEach(el => el.textContent = bounds(historyGroups['18']));
  draw();
})();
