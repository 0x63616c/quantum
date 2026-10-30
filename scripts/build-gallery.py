#!/usr/bin/env python3
"""Rebuild the gallery through the real Voxelite stdio MCP server.

CARGO_TARGET_DIR=target-gallery cargo build --release -p voxelite-mcp
python3 scripts/build-gallery.py --draft     # inspect blockouts first
python3 scripts/build-gallery.py             # sources, four-view evidence, assets
python3 scripts/build-gallery.py --only orbit # refine one sculpture

Requires Pillow with WebP (and optionally AVIF). CPU rendering needs no display.
The UUID receipts vary per run; geometry, palette and cameras are deterministic.
"""
import argparse, base64, hashlib, importlib.util, io, json, math, os, subprocess, sys, uuid
sys.dont_write_bytecode = True
from pathlib import Path
from PIL import Image, ImageDraw, features

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'site/gallery'; EVIDENCE=ROOT/'screenshots/gallery/models'
spec=importlib.util.spec_from_file_location('models',ROOT/'scripts/gallery-models.py')
models=importlib.util.module_from_spec(spec);spec.loader.exec_module(models)

class MCP:
    def __init__(self):
        target=Path(os.environ.get('CARGO_TARGET_DIR',ROOT/'target-gallery'))
        binary=target/'release/voxelite-mcp'
        if not binary.exists():binary=target/'debug/voxelite-mcp'
        self.p=subprocess.Popen([str(binary),'--root',str(ROOT)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
        self.n=0;self.receipts=[]
        self.rpc('initialize',dict(protocolVersion='2025-11-25',capabilities={},clientInfo=dict(name='gallery-builder',version='1')))
        self.p.stdin.write(json.dumps(dict(jsonrpc='2.0',method='notifications/initialized'))+'\n');self.p.stdin.flush()
        self.catalog=self.rpc('tools/list',{})['tools']
        self.epoch=self.call('capabilities_get',{})['structuredContent']['epoch']
    def rpc(self,method,params):
        self.n+=1;self.p.stdin.write(json.dumps(dict(jsonrpc='2.0',id=self.n,method=method,params=params))+'\n');self.p.stdin.flush()
        while True:
            line=self.p.stdout.readline()
            if not line:raise RuntimeError('MCP exited unexpectedly')
            r=json.loads(line)
            if r.get('id')==self.n:
                assert 'error' not in r,r
                return r['result']
    def call(self,name,args):
        assert name=='capabilities_get' or any(t['name']=='voxelite_'+name for t in self.catalog),name
        r=self.rpc('tools/call',dict(name='voxelite_'+name,arguments=args))
        assert not r.get('isError'),r
        assert r['structuredContent']['ok'],r
        return r
    def edit(self,tool,**args):
        r=self.call(tool,dict(model_id=self.model,expected_epoch=self.epoch,expected_revision=self.rev,operation_id=str(uuid.uuid4()),**args))['structuredContent']['data']
        self.rev=r.get('revision',self.rev);self.receipts.append(dict(tool='voxelite_'+tool,revision=self.rev))
        return r
    def render(self,**args):
        r=self.call('render',dict(model_id=self.model,revision=self.rev,**args))
        blocks=[b for b in r['content'] if b['type']=='image']
        return [base64.b64decode(b['data']) for b in blocks],r['structuredContent']['data']
    def close(self):
        self.p.stdin.close();self.p.wait(timeout=10)

def write_json(path,data):path.write_text(json.dumps(data,indent=2)+'\n')
def source_info(path):
    p=json.loads(path.read_text());cells=[v for l in p['layers'] if l['visible'] for v in l['voxels']]
    lo=[min(v[i] for v in cells) for i in range(3)];hi=[max(v[i] for v in cells)+1 for i in range(3)]
    used={v[3] for v in cells}
    palette=p['palette']['slots'] if isinstance(p['palette'],dict) else p['palette']
    return p,cells,lo,hi,palette,used

def validate_connected(s):
    # Detached details can look attached from one camera. Require a physical
    # face-to-face path through every cell of each original finished sculpture.
    remaining=set(s.cells);queue=[remaining.pop()]
    while queue:
        cell=queue.pop()
        for axis in range(3):
            for direction in [-1,1]:
                neighbor=list(cell);neighbor[axis]+=direction;neighbor=tuple(neighbor)
                if neighbor in remaining:remaining.remove(neighbor);queue.append(neighbor)
    assert not remaining,f'{s.slug}: disconnected geometry ({len(remaining)} cells)'

def build(s,draft=False):
    if s and not draft:validate_connected(s)
    m=MCP()
    try:
        sname=s.slug if s else 'autumn-workshop';title=s.title if s else 'Autumn workshop'
        print(('Blockout: ' if draft else 'Build: ')+title,flush=True)
        if s:
            # Include both stages when calculating bounds, including intentional erasures.
            all_cells=set(s.cells)|set(s.blockout)
            lo=[min(p[i] for p in all_cells) for i in range(3)];hi=[max(p[i] for p in all_cells)+1 for i in range(3)]
            info=m.call('document_create',dict(name=title,bounds=dict(min=lo,max=hi),expected_epoch=m.epoch,operation_id=str(uuid.uuid4())))['structuredContent']['data']
        else:info=m.call('document_open',dict(path='examples/autumn-workshop.voxl',expected_epoch=m.epoch,operation_id=str(uuid.uuid4())))['structuredContent']['data']
        m.model=info['model_id'];m.rev=info['revision'];layer=info['layers'][0]['id']
        m.call('document_info',dict(model_id=m.model));m.call('layers_list',dict(model_id=m.model));m.call('palette_list',dict(model_id=m.model));m.call('history_read',dict(model_id=m.model))
        if s:
            for i,(name,color) in enumerate(s.colors,1):m.edit('palette_set_slot',slot=i,name=name,rgba=[int(color[k:k+2],16) for k in (0,2,4)]+[255])
            m.edit('voxels_batch',operations=[dict(kind='set',layer=layer,voxels=[dict(at=p,slot=c) for p,c in sorted(s.blockout.items())])],label='Compose primary masses')
            m.edit('versions_save',name='Silhouette blockout')
            if not draft:
                patch=[dict(at=p,slot=s.cells.get(p,0)) for p in sorted(all_cells) if s.cells.get(p,0)!=s.blockout.get(p,0)]
                if patch:m.edit('voxels_batch',operations=[dict(kind='set',layer=layer,voxels=patch)],label='Refine materials and readable details')
        m.edit('versions_save',name='Blockout review' if draft else 'Gallery edition')
        images,meta=m.render(views=['iso','front','right','back'],width=512,height=512,background=[0,0,0,255],image_mode='both')
        folder=EVIDENCE/('blockout' if draft else 'final');folder.mkdir(parents=True,exist_ok=True)
        # Voxelite returns the contact sheet followed by individual images.
        sheet=next(data for data in images if Image.open(io.BytesIO(data)).width==1024)
        (folder/f'{sname}.png').write_bytes(sheet)
        write_json(folder/f'{sname}.json',dict(revision=m.rev,render=meta,receipts=m.receipts))
        if draft:return
        path=OUT/'models'/f'{sname}.voxl'
        # Generated paths only; the original example is never overwritten.
        path.unlink(missing_ok=True)
        m.edit('document_save',path=str(path.relative_to(ROOT)))
        p,cells,lo,hi,palette,used=source_info(path)
        center=[(a+b)/2 for a,b in zip(lo,hi)]
        # Fit projected occupied cell bounds, not just a sparse world-space AABB.
        def camera(yaw):
            yaw=math.radians(yaw);pitch=math.radians(27)
            right=(math.cos(yaw),0,-math.sin(yaw));up=(-math.sin(yaw)*math.sin(pitch),math.cos(pitch),-math.cos(yaw)*math.sin(pitch))
            back=(math.sin(yaw)*math.cos(pitch),math.sin(pitch),math.cos(yaw)*math.cos(pitch))
            ext=[]
            for axis in [right,up]:
                vals=[sum((v[i]+.5-center[i])*axis[i] for i in range(3)) for v in cells];half=sum(abs(a) for a in axis)/2
                ext.append((min(vals)-half,max(vals)+half))
            target=[center[i]+sum(axis[i]*sum(e)/2 for axis,e in zip([right,up],ext)) for i in range(3)]
            span=max(b-a for a,b in ext)*1.35
            return dict(eye=[target[i]+back[i]*120 for i in range(3)],target=target,up=[0,1,0],projection='orthographic',ortho_height=span)
        captures=[]
        for theme,bg in [('dark',[0,0,0,255]),('light',[255,255,255,255])]:
            for angle,yaw in [('main',35),('turn',47)]:
                size=1024 if angle=='main' else 640
                images,metadata=m.render(camera=camera(yaw),width=size,height=size,background=bg,image_mode='individual')
                data=images[0];im=Image.open(io.BytesIO(data)).convert('RGB')
                for label,width in ([('large',1024),('tile',640)] if angle=='main' else [('turn',640)]):
                    resized=im.resize((width,width),Image.Resampling.LANCZOS)
                    for fmt in ['webp']+(['avif'] if features.check('avif') else []):
                        name=f'{sname}-{theme}-{label}.{fmt}'
                        resized.save(OUT/'images'/name,quality=86 if fmt=='webp' else 75,**({'method':6} if fmt=='webp' else {'speed':6}))
                captures.append(dict(theme=theme,angle=angle,revision=m.rev,render=metadata))
        # Stable slots, as written and verified by Voxelite.
        colors=[dict(name=item.get('name',''),hex='#'+''.join(f'{c:02x}' for c in item['rgba'][:3])) for item in palette if item['index'] in used]
        entry=dict(id=sname,title=title,category=s.category if s else 'Buildings',description=s.description if s else 'The original workshop. A warm place to begin.',voxel_count=len(cells),dims=[b-a for a,b in zip(lo,hi)],palette=colors,source=f'/gallery/models/{sname}.voxl',sha256=hashlib.sha256(path.read_bytes()).hexdigest(),revision=m.rev,formats=['webp']+(['avif'] if features.check('avif') else []))
        write_json(folder/f'{sname}.json',dict(source_sha256=entry['sha256'],revision=m.rev,review=meta,captures=captures,receipts=m.receipts))
        return entry
    finally:m.close()

def overview(folder):
    files=sorted(folder.glob('*.png'));w=320;sheet=Image.new('RGB',(w*4,350*math.ceil(len(files)/4)),(16,16,16));d=ImageDraw.Draw(sheet)
    for i,path in enumerate(files):
        im=Image.open(path).crop((0,0,512,512)).resize((w,w));x=i%4*w;y=i//4*350;sheet.paste(im,(x,y));d.text((x+12,y+323),path.stem,fill='white')
    sheet.save(folder/'overview.jpg',quality=92)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--draft',action='store_true');parser.add_argument('--only');args=parser.parse_args()
    for folder in [OUT/'models',OUT/'images',EVIDENCE]:folder.mkdir(parents=True,exist_ok=True)
    entries=[]
    existing=json.loads((OUT/'manifest.json').read_text()) if (OUT/'manifest.json').exists() else []
    for builder in models.BUILDERS+[lambda:None]:
        s=builder();slug=s.slug if s else 'autumn-workshop'
        if args.only and slug not in args.only.split(','):
            entries.extend(e for e in existing if e['id']==slug);continue
        entry=build(s,args.draft)
        if entry:entries.append(entry)
    overview(EVIDENCE/('blockout' if args.draft else 'final'))
    if not args.draft:
        order=['autumn-workshop','instant-summer','small-step','midnight-noodles','under-the-canopy','weekender','forest-friend','after-hours','one-more-credit','desert-hours','slow-sunday','dust-runner','the-last-light','branch-line','still-water','orbit','off-court','lavender-mill','yellow-submarine']
        entries.sort(key=lambda entry:order.index(entry['id']))
        write_json(OUT/'manifest.json',entries)
        subprocess.run([sys.executable,str(ROOT/'scripts/gallery-page.py')],check=True)
