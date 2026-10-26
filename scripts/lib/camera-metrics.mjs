// Shared offline analysis: existing raw traces can be reanalyzed without rerunning.
export const quantile = (xs, q) => xs.length ? [...xs].sort((a,b)=>a-b)[Math.min(xs.length-1,Math.floor(q*(xs.length-1)))] : null;
export const stats = xs => ({count:xs.length,p50:quantile(xs,.5),p95:quantile(xs,.95),p99:quantile(xs,.99),mean:xs.length?xs.reduce((a,b)=>a+b,0)/xs.length:null});
export function summarize(raw,duration) {
 const item={camera:{},placement:{}};
 // One render is in flight at a time. Bind every installed image to its exact
 // preceding render-start event by event order (timestamps can tie).
 const captures=new Map();let capture;
 for(const e of raw.events){if(e.name==='render_start')capture=e;if(e.name==='image_ready'&&capture)captures.set(e.a,capture);}

 for(const kind of ['orbit','pan','zoom','preview']) {
  const start=raw.events.find(e=>e.name==='phase'&&e.phase===kind).t;
  const end=raw.events.find(e=>e.name==='phase'&&e.t>start).t;
  const frames=raw.presented.filter(f=>f.phase===kind),raf=raw.rafs.filter(f=>f.phase===kind);
  const inputs=raw.inputs.filter(e=>e.phase===kind&&(e.type==='pointermove'||e.type==='wheel'));
  // Hover changes only an overlay. Do not attribute the next unrelated camera
  // reset several seconds later as its "input to frame" latency.
  const latency=kind==='preview'?[]:inputs.map(e=>raw.presented.find(f=>f.t>=e.t&&f.paint>=e.t&&(f.phase===kind||f.phase==='settle')&&f.paint<=end+5000)?.t-e.t).filter(Number.isFinite);

  const causalLatency=kind==='preview'?[]:inputs.map(e=>{
   const navigation=raw.events.find(n=>n.name==='navigation'&&n.phase===kind&&n.t>=e.t);
   return navigation?raw.presented.find(f=>captures.get(f.id)?.t>=navigation.t&&f.paint<=end+5000)?.t-e.timestamp:null;
  }).filter(Number.isFinite);
  const paints=raw.events.filter(e=>e.phase===kind&&e.name==='paint');
  const overlayLatency=inputs.map(e=>{const paint=paints.find(p=>p.t>=e.t);return paint?raw.rafs.find(r=>r.t>=paint.t)?.t-e.t:null;}).filter(Number.isFinite);
  const cpu={};for(const e of raw.events.filter(e=>e.phase===kind&&e.b===-1&&e.name!=='gpu_wait_readback'))(cpu[e.name]??=[]).push(e.a);
  item.camera[kind]={measuredWallMs:end-start,backlogDrainMs:Math.max(0,end-start-duration),avgFps:frames.length/((end-start)/1000),frameMs:stats(frames.slice(1).map((f,i)=>f.t-frames[i].t)),rafFps:raf.length/((end-start)/1000),overlayPaintFps:paints.length/((end-start)/1000),inputToOverlayFrameMs:stats(overlayLatency),inputToNextFrameMs:stats(latency),inputToCausalFrameMs:stats(causalLatency),dispatchDelayMs:stats(inputs.map(e=>e.t-e.timestamp)),inputEvents:inputs.length,offeredEvents:duration/1000*60,cpuMs:Object.fromEntries(Object.entries(cpu).map(([k,v])=>[k,{...stats(v),perChangedFrame:frames.length?v.reduce((a,b)=>a+b,0)/frames.length:null}])),gpuWaitReadbackMs:stats(raw.events.filter(e=>e.phase===kind&&e.name==='gpu_wait_readback').map(e=>e.a)),gpuUploadedBytes:raw.events.filter(e=>e.phase===kind&&e.name==='gpu_upload_bytes').reduce((s,e)=>s+e.a,0),jsWasmBoundaryMs:null};
 }
 for(const kind of ['single','series']) {
  const inputs=raw.inputs.filter(e=>e.phase===kind&&e.type==='pointerdown'&&e.button===0);
  const edits=raw.events.filter(e=>e.phase===kind&&e.name==='edit_result'&&e.b===1);
  const latencies=[],offered=[];
  for(const e of edits){const click=inputs.findLast(i=>i.t<=e.t);const frame=raw.presented.find(f=>f.paint>=e.t&&f.revision>=e.a);if(frame&&click){latencies.push(frame.t-click.t);offered.push(frame.t-click.timestamp);}}
  const last=edits.at(-1);
  item.placement[kind]={attempted:inputs.length,accepted:edits.length,visible:latencies.length,clickToVisibleMs:stats(latencies),eventTimestampToVisibleMs:stats(offered),dispatchDelayMs:stats(inputs.map(i=>i.t-i.timestamp)),editCpuMs:stats(raw.events.filter(e=>e.phase===kind&&e.name==='edit'&&e.b===-1).map(e=>e.a)),meshCpuMs:stats(raw.events.filter(e=>e.phase===kind&&e.name==='mesh'&&e.b===-1).map(e=>e.a)),updatedChunks:stats(raw.events.filter(e=>e.phase===kind&&e.name==='mesh_chunks').map(e=>e.a)),totalToLastVisibleMs:last?raw.presented.find(f=>f.revision>=last.a&&f.paint>=last.t)?.t-inputs[0]?.t:null};
 }
 return item;
}
export const median=values=>{if(values.every(v=>typeof v==='number'))return quantile(values,.5);if(values[0]&&typeof values[0]==='object'&&!Array.isArray(values[0]))return Object.fromEntries(Object.keys(values[0]).map(k=>[k,median(values.map(v=>v[k]))]));return values[0];};
export function markdown(report) {
 const rows=['| Scene | Action | FPS | Frame p50 / p95 / p99 ms | Causal input / click p50 / p95 ms | Accepted |','|---|---|---:|---|---|---:|'];
 const f=n=>typeof n==='number'?n.toFixed(1):'n/a';
 for(const s of report.scenes){for(const [k,v] of Object.entries(s.median.camera))rows.push(`| ${s.name} | ${k} | ${f(v.avgFps)} | ${f(v.frameMs.p50)} / ${f(v.frameMs.p95)} / ${f(v.frameMs.p99)} | ${f(v.inputToCausalFrameMs.p50)} / ${f(v.inputToCausalFrameMs.p95)} | |`);for(const [k,v] of Object.entries(s.median.placement))rows.push(`| ${s.name} | ${k} placement | | | ${f(v.clickToVisibleMs.p50)} / ${f(v.clickToVisibleMs.p95)} | ${v.accepted}/${v.attempted} |`);}
 return rows.join('\n')+'\n';
}
