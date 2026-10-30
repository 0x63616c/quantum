// GPU acceptance checks on the real application; build with VOXELITE_PERF=1 first.
import assert from 'node:assert/strict';
import {chromium} from 'playwright';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
const server=createServer(async(req,res)=>{try{const path=new URL(req.url,'http://local').pathname;const bytes=await readFile(`web/dist${path==='/'?'/index.html':path}`);res.setHeader('Content-Type',path.endsWith('.wasm')?'application/wasm':path.endsWith('.js')?'text/javascript':'text/html');res.end(bytes);}catch{res.writeHead(404).end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const report=[];
try {
for(const backend of ['webgl','webgpu']) {
 const browser=await chromium.launch({headless:true,args:['--no-sandbox','--enable-unsafe-webgpu','--enable-unsafe-swiftshader',...(backend==='webgl'?['--use-angle=swiftshader']:['--use-angle=vulkan','--use-vulkan=swiftshader','--enable-features=Vulkan','--disable-vulkan-surface'])]});
 try {
 const page=await browser.newPage({viewport:{width:1440,height:900},deviceScaleFactor:1});
 const errors=[],messages=[];
 page.on('pageerror',e=>errors.push(String(e)));page.on('console',m=>{if(/Voxelite GPU|error/i.test(m.text()))messages.push(m.text());});
 await page.addInitScript(()=>{globalThis.__voxelitePerf={events:[],event(name,a,b){this.events.push({name,a,b,t:performance.now()});}};});
 await page.goto(`http://127.0.0.1:${server.address().port}/?theme=dark&backend=${backend}&viewport=gpu`);
 const idle=()=>page.waitForFunction(()=>{const p=__voxelitePerf.events;const r=p.findLast(e=>e.name==='render_start');const ready=p.findLast(e=>e.name==='image_ready');const paint=p.findLast(e=>e.name==='paint');const edit=p.findLast(e=>e.name==='edit_result'&&e.b===1);return ready&&paint&&paint.a===ready.a&&ready.t>=r.t&&(!edit||ready.b>=edit.a)&&performance.now()-ready.t>300;},{},{timeout:120000});
 await idle();await page.mouse.move(70,160);await page.mouse.move(1200,60);await page.waitForTimeout(200);
 assert(!await page.evaluate(()=>__voxelitePerf.events.some(e=>e.name==='gpu_fallback')),'GPU must not silently fall back');
 const save=async()=>{const pending=page.waitForEvent('download');await page.keyboard.press('Control+s');const d=await pending;return JSON.parse(await readFile(await d.path(),'utf8'));};
 const clip={x:56,y:150,width:1032,height:640};
 const initial=await save();await idle();const initialImage=await page.screenshot({clip});
 await page.mouse.click(550,610);
 await page.waitForFunction(()=>__voxelitePerf.events.some(e=>e.name==='edit_result'&&e.b===1));
 await idle();await page.mouse.move(70,160);await page.mouse.move(1200,60);await page.waitForTimeout(200);
 const edited=await save();
 const count=p=>p.layers.reduce((n,l)=>n+l.voxels.length,0);
 assert.equal(count(edited),count(initial)+1);
 const editedImage=await page.screenshot({clip});assert.notDeepEqual(editedImage,initialImage,'placed block must change viewport pixels');
 const next=async()=>{const revision=await page.evaluate(()=>__voxelitePerf.events.findLast(e=>e.name==='image_ready').b);return revision;};
 let revision=await next();await page.keyboard.press('Control+z');
 await page.waitForFunction(rev=>__voxelitePerf.events.findLast(e=>e.name==='image_ready').b>rev,revision,{timeout:120000});await idle();
 const undone=await save();assert.deepEqual(undone.layers,initial.layers);const undoImage=await page.screenshot({clip});await writeFile('/tmp/gpu-check-initial.png',initialImage);await writeFile('/tmp/gpu-check-undo.png',undoImage);assert.deepEqual(undoImage,initialImage,'undo must restore identical viewport pixels');
 revision=await next();await page.keyboard.press('Control+Shift+z');
 await page.waitForFunction(rev=>__voxelitePerf.events.findLast(e=>e.name==='image_ready').b>rev,revision,{timeout:120000});await idle();
 const redone=await save();assert.deepEqual(redone.layers,edited.layers);assert.deepEqual(await page.screenshot({clip}),editedImage,'redo must restore identical viewport pixels');
 // Camera-only redraws must retain mesh and vertex buffers.
 await page.evaluate(()=>__voxelitePerf.events=[]);
 await page.mouse.move(550,450);await page.mouse.down({button:'right'});await page.mouse.move(650,480,{steps:10});await page.mouse.up({button:'right'});await idle();
 const events=await page.evaluate(()=>__voxelitePerf.events);
 assert(events.some(e=>e.name==='render_submission'));
 assert(!events.some(e=>e.name==='mesh'||e.name==='gpu_upload_bytes'),'camera must not remesh or re-upload vertices');
 await page.setViewportSize({width:1280,height:800});await idle();
 assert.deepEqual(errors,[]);assert(!messages.some(m=>m.includes('ERROR')));
 report.push({backend,checks:['GPU initialized','visible placement pixel change','save voxel count','undo identical pixels and voxels','redo identical pixels and voxels','camera retains meshes/buffers','resize'],errors,messages});
 console.log(`${backend}: all GPU viewport checks passed`);
 }finally{await browser.close();}
}
await mkdir('docs/perf',{recursive:true});await writeFile('docs/perf/gpu-verification.json',JSON.stringify(report,null,2)+'\n');
}finally{server.close();}
