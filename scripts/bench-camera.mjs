// Real GPUI/wasm build, real CDP input, wall-clock scheduling independent of page FPS.
// VOXELITE_PERF=1 bash scripts/build-web.sh
// node scripts/bench-camera.mjs --out docs/perf/camera-before.json --runs 3
import { chromium } from 'playwright';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createServer } from 'node:http';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import os from 'node:os';
const arg = (k, d) => process.argv.includes(k) ? process.argv[process.argv.indexOf(k) + 1] : d;
const output = arg('--out', 'docs/perf/camera-before.json');
const runs = Number(arg('--runs', '3'));
const duration = Number(arg('--duration', '6000'));
const backend = arg('--backend', 'webgl');
if(!Number.isInteger(runs)||runs<1||runs>20||!Number.isFinite(duration)||duration<1000) throw Error('Use 1–20 runs and a duration of at least 1000 ms');
if(!['all','typical','large'].includes(arg('--scene','all'))) throw Error('--scene must be all, typical, or large');
if(!['webgl','webgpu'].includes(backend)) throw Error('--backend must be webgl or webgpu');
const extraQuery = arg('--query', '');
const sleep = ms => new Promise(r => setTimeout(r, ms));
import { summarize, median, markdown } from './lib/camera-metrics.mjs';
const original = JSON.parse(await readFile('examples/autumn-workshop.voxl', 'utf8'));
const scenes = [structuredClone(original), structuredClone(original)];
scenes[1].document.name = 'Camera benchmark gallery';
scenes[1].document.bounds = { min: [-80,0,-80], max: [80,64,80] };
scenes[1].comments = [];
scenes[1].versions = [];
for (const layer of scenes[1].layers) layer.voxels = [-40,40].flatMap(x => [-40,40].flatMap(z => layer.voxels.map(([a,b,c,s]) => [a+x,b,c+z,s])));
const server = createServer(async (req,res) => {
  try {
    const path = new URL(req.url, 'http://local').pathname;
    if (path.includes('..')) throw Error('bad path');
    const bytes = await readFile(`web/dist${path === '/' ? '/index.html' : path}`);
    res.setHeader('Content-Type', path.endsWith('.wasm') ? 'application/wasm' : path.endsWith('.js') ? 'text/javascript' : 'text/html');
    res.end(bytes);
  } catch { res.writeHead(404).end(); }
});
await new Promise(r => server.listen(0, '127.0.0.1', r));
const flags = ['--no-sandbox', '--enable-unsafe-webgpu', '--enable-unsafe-swiftshader', ...(backend === 'webgpu' ? ['--use-angle=vulkan','--use-vulkan=swiftshader','--enable-features=Vulkan','--disable-vulkan-surface'] : ['--use-angle=swiftshader'])];
const browser = await chromium.launch({headless:true, args:flags});
const report = { schema:1, date:new Date().toISOString(), commit:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(), wasmSha256:createHash('sha256').update(await readFile('web/dist/voxelite_app_bg.wasm')).digest('hex'), environment:{ browser:browser.version(), platform:os.platform(), cpu:os.cpus()[0].model, cores:os.cpus().length, flags, backend, viewportQuery:extraQuery, viewport:[1440,900], dpr:1 }, protocol:{durationMs:duration, inputHz:60, runs, placementSingle:10, placementSeries:50, seriesIntervalMs:33.333, presentation:'First requestAnimationFrame after unique viewport paint. Submission-to-presentation upper-bound proxy, not a physical display photon measurement.', frameMs:'Intervals between changed viewport images; excludes overlay-only/UI paints. RAF FPS is separately reported.', cpu:'Inclusive wasm spans; nested intervals must not be added. JS/wasm boundary overhead is not separately measurable and is reported null.', cameraLatency:'Browser capture-phase event to first changed viewport paint after it (next RAF); this can include a previously in-flight image.', causalCameraLatency:'Injection timestamp to the first painted image whose render began after the associated navigation event; includes dispatch backlog. Intermediate coalesced states may not have their own frame.', placementLatency:'Capture-phase left mousedown to first painted revision >= successful edit revision (next RAF). Rejections/no-ops separately counted.'}, scenes:[] };
await mkdir('docs/perf', {recursive:true});
try {
for (let si=0;si<scenes.length;si++) {
 if(arg('--scene','all')!=='all'&&arg('--scene')!==(si?'large':'typical')) continue;
 const scene=scenes[si];
 const result={name:si?'large':'typical', title:scene.document.name, voxels:scene.layers.reduce((n,l)=>n+l.voxels.length,0), sha256:createHash('sha256').update(JSON.stringify(scene)).digest('hex'), runs:[]};
 report.scenes.push(result);
 for(let run=0;run<runs;run++) {
  console.log(`${result.name} run ${run+1}/${runs}`);
  const page=await browser.newPage({viewport:{width:1440,height:900},deviceScaleFactor:1});
  const errors=[]; const diagnostics=[];
  page.on('pageerror',e=>{errors.push(String(e));console.error(String(e));});
  page.on('console',m=>{if (/successfully with|Voxelite GPU|error/i.test(m.text())) {const text=m.text().split(', limits=')[0];diagnostics.push(text);console.log(text);}});
  await page.addInitScript(() => {
   const p=globalThis.__voxelitePerf={events:[],rafs:[],inputs:[],presented:[],phase:'load',lastFrame:-1,
    event(name,a,b){const t=performance.now();this.events.push({name,a,b,t,phase:this.phase});if(name==='paint'&&a!==this.lastFrame){this.lastFrame=a;const phase=this.phase;requestAnimationFrame(()=>this.presented.push({id:a,revision:b,t:performance.now(),paint:t,phase}));}}};
   for(const type of ['pointermove','pointerdown','pointerup','wheel']) addEventListener(type,e=>p.inputs.push({type,t:performance.now(),timestamp:e.timeStamp,buttons:e.buttons,button:e.button,x:e.clientX,y:e.clientY,phase:p.phase}),true);
   const tick=t=>{p.rafs.push({t,phase:p.phase});requestAnimationFrame(tick);};requestAnimationFrame(tick);
  });
  await page.goto(`http://127.0.0.1:${server.address().port}/?theme=dark&backend=${backend}${extraQuery ? '&'+extraQuery : ''}`);
  if(process.env.BENCH_DEBUG) setTimeout(async()=>{try{console.log(await page.evaluate(()=>__voxelitePerf.events.slice(-20)));await page.screenshot({path:'/tmp/voxelite-debug.png'});}catch{}},10000).unref();
  const idle=async()=>{await page.waitForFunction(()=>{const p=__voxelitePerf;const a=p.events.filter(e=>e.name==='render_start').at(-1);const b=p.events.filter(e=>e.name==='image_ready').at(-1);const edit=p.events.findLast(e=>e.name==='edit_result'&&e.b===1);const nav=p.events.findLast(e=>e.name==='navigation');return b&&(!a||b.t>=a.t)&&(!edit||b.b>=edit.a)&&(!nav||b.t>=nav.t)&&p.presented.some(f=>f.id===b.a)&&performance.now()-b.t>400;},{},{timeout:120000});};
  await idle();
  if(si){const chooser=page.waitForEvent('filechooser');await page.keyboard.press('Control+o');await(await chooser).setFiles({name:'gallery.voxl',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(scene))});await page.waitForFunction(()=>document.title.includes('Camera benchmark gallery'),{},{timeout:120000});await idle();}
  if(run===0) await page.screenshot({path:output.replace('.json',`-${result.name}.png`)});
  const cdp=await page.context().newCDPSession(page);
  const phase=async name=>page.evaluate(n=>{__voxelitePerf.phase=n;__voxelitePerf.events.push({name:'phase',t:performance.now(),phase:n});},name);
  const timed=async(ms,fn)=>{const start=performance.now(), pending=[];for(let i=0;i<ms/1000*60;i++){await sleep(Math.max(0,start+i*1000/60-performance.now()));pending.push(fn(i));}await Promise.all(pending);};
  for(const kind of ['orbit','pan','zoom','preview']){
   await phase(kind);
   const button=kind==='orbit'?'right':'middle';
   if(kind==='orbit'||kind==='pan'){await page.mouse.move(560,450);await page.mouse.down({button});}
   await timed(duration, i=>cdp.send('Input.dispatchMouseEvent',kind==='zoom'?{timestamp:Date.now()/1000,type:'mouseWheel',x:560,y:450,deltaX:0,deltaY:Math.sin(i/20)*4}:{timestamp:Date.now()/1000,type:'mouseMoved',x:560+Math.sin(i/45)*140,y:450+Math.sin(i/33)*60,button:kind==='preview'?'none':button,buttons:kind==='preview'?0:kind==='orbit'?2:4}));
   if(kind==='orbit'||kind==='pan') await page.mouse.up({button});
   await phase('settle'); await idle();
  }
  // Reset camera through the real Frame command before placement.
  await page.keyboard.press('f');await idle();await page.keyboard.press('b');
  await phase('single');
  for(let i=0;i<10;i++){await page.mouse.click(550+(i%5)*7,610+Math.floor(i/5)*7);await idle();}
  await phase('series');
  const seriesStart=performance.now();
  const pending=[];
  for(let i=0;i<50;i++){
   await sleep(Math.max(0,seriesStart+i*1000/30-performance.now()));
   const x=520+(i%10)*7,y=630+Math.floor(i/10)*7;
   // Repeated placement is supported by the existing editor; held-left drag only hovers.
   pending.push(cdp.send('Input.dispatchMouseEvent',{timestamp:Date.now()/1000,type:'mousePressed',x,y,button:'left',buttons:1,clickCount:1}));
   pending.push(cdp.send('Input.dispatchMouseEvent',{timestamp:Date.now()/1000,type:'mouseReleased',x,y,button:'left',buttons:0,clickCount:1}));
  }
  await Promise.all(pending);await idle();
  const seriesWallMs=performance.now()-seriesStart;
  await phase('done');
  const raw=await page.evaluate(()=>({events:__voxelitePerf.events,inputs:__voxelitePerf.inputs,presented:__voxelitePerf.presented,rafs:__voxelitePerf.rafs}));
  await writeFile(output.replace('.json',`-${result.name}-${run+1}.trace.json`),JSON.stringify(raw));
  const item={...summarize(raw,duration),errors,diagnostics};
  item.placement.series.wallIncludingIdleWaitMs=seriesWallMs;
  result.runs.push(item);
  await writeFile(output,JSON.stringify(report,null,2)+'\n');
  if(errors.length) throw Error(errors.join('\n'));
  if(extraQuery.split('&').includes('viewport=gpu')&&(raw.events.some(e=>e.name==='gpu_fallback')||!diagnostics.some(m=>m.includes('Voxelite GPU viewport:')))) throw Error('GPU benchmark fell back to CPU');
  await page.close();
 }
 // Coordinate-wise median of all numeric measurements; retain raw runs above.
 result.median=median(result.runs.map(({camera,placement})=>({camera,placement})));
}
const table=markdown(report);
await writeFile(output,JSON.stringify(report,null,2)+'\n');
await writeFile(output.replace('.json','.md'),table);console.log(table);
}finally{await browser.close();server.close();}
