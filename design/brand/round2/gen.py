import math, cairosvg, os
OUT='/workspace/brand/round2'; os.makedirs(OUT,exist_ok=True)
C=math.cos(math.pi/6)
def squircle(x0=100,y0=100,w=824,n=5,steps=240):
    r=w/2;cx=x0+r;cy=y0+r;pts=[]
    for i in range(steps):
        t=2*math.pi*i/steps;c,s=math.cos(t),math.sin(t)
        pts.append((cx+r*math.copysign(abs(c)**(2/n),c),cy+r*math.copysign(abs(s)**(2/n),s)))
    return 'M'+' L'.join(f'{a:.1f},{b:.1f}' for a,b in pts)+'Z'
SQ=squircle()
def P(x,y,z,s,ox,oy): return (ox+(x-y)*C*s, oy+(x+y)*0.5*s-z*s)
def faces(x,y,z,s,ox,oy):
    p=lambda a,b,c:P(a,b,c,s,ox,oy)
    top=[p(x,y,z+1),p(x+1,y,z+1),p(x+1,y+1,z+1),p(x,y+1,z+1)]
    left=[p(x,y+1,z),p(x+1,y+1,z),p(x+1,y+1,z+1),p(x,y+1,z+1)]
    right=[p(x+1,y,z),p(x+1,y+1,z),p(x+1,y+1,z+1),p(x+1,y,z+1)]
    return top,left,right
def poly(pts,fill,extra=''): return f'<polygon points="{" ".join(f"{a:.1f},{b:.1f}" for a,b in pts)}" fill="{fill}" {extra}/>'
BASE='''<defs><linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#161616"/><stop offset="1" stop-color="#000"/></linearGradient>
<linearGradient id="rim" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity=".22"/><stop offset=".5" stop-color="#fff" stop-opacity=".04"/><stop offset="1" stop-color="#fff" stop-opacity=".1"/></linearGradient>
<clipPath id="sq"><path d="%s"/></clipPath>%s</defs>
<path d="%s" fill="url(#bg)"/><g clip-path="url(#sq)">%s</g><path d="%s" fill="none" stroke="url(#rim)" stroke-width="3"/>'''
def icon(name,defs,body):
    svg=f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">'+BASE%(SQ,defs,SQ,body,SQ)+'</svg>'
    open(f'{OUT}/{name}.svg','w').write(svg); cairosvg.svg2png(bytestring=svg.encode(),write_to=f'{OUT}/{name}-1024.png')
    return svg

# 1 INSIDE: the inside corner of a voxel, lit warm — concave Y you look into
d1='''<linearGradient id="fl" x1="0" y1="1" x2="0" y2="0"><stop offset="0" stop-color="#fff1d0"/><stop offset="1" stop-color="#ffb648"/></linearGradient>
<linearGradient id="wl" x1="1" y1="1" x2="0" y2="0"><stop offset="0" stop-color="#ff9a2e"/><stop offset="1" stop-color="#9a3412"/></linearGradient>
<linearGradient id="wr" x1="0" y1="1" x2="1" y2="0"><stop offset="0" stop-color="#d9590c"/><stop offset="1" stop-color="#431407"/></linearGradient>
<radialGradient id="hal" cx=".5" cy=".6" r=".5"><stop offset="0" stop-color="#ff8a1f" stop-opacity=".35"/><stop offset="1" stop-color="#ff8a1f" stop-opacity="0"/></radialGradient>'''
s=300;ox,oy=512,512
p=lambda a,b,c:P(a,b,c,s,ox,oy+60)
O=p(0,0,0)
floor=[p(0,0,0),p(1,0,0),p(1,1,0),p(0,1,0)]  # not used directly
# back corner at origin; walls x=0 plane and y=0 plane, floor z=0
f=[p(0,0,0),p(1,0,0),p(1,1,0),p(0,1,0)]
wl=[p(0,0,0),p(0,1,0),p(0,1,1),p(0,0,1)]
wr=[p(0,0,0),p(1,0,0),p(1,0,1),p(0,0,1)]
b=[f'<ellipse cx="512" cy="560" rx="420" ry="380" fill="url(#hal)"/>',poly(wl,'url(#wl)'),poly(wr,'url(#wr)'),poly(f,'url(#fl)')]
rimpts=[p(0,1,1),p(0,1,0),p(1,1,0),p(1,0,0),p(1,0,1),p(0,0,1)]
b.append(poly(rimpts,'none','stroke="#ffe2b0" stroke-opacity=".9" stroke-width="5" stroke-linejoin="round"'))
t2,l2,r2=faces(0.18,0.18,0,s*0.32/1.0*1,ox,oy+60) if False else (None,None,None)
ss=s*0.46
q=lambda a,b_,c:P(a,b_,c,ss,*P(0.12,0.12,0,s,ox,oy+60))
vt=[q(0,0,1),q(1,0,1),q(1,1,1),q(0,1,1)];vl=[q(0,1,0),q(1,1,0),q(1,1,1),q(0,1,1)];vr=[q(1,0,0),q(1,1,0),q(1,1,1),q(1,0,1)]
b.append(f'<ellipse cx="{q(.5,.5,0)[0]:.1f}" cy="{q(.5,.5,0)[1]+10:.1f}" rx="190" ry="95" fill="#7c2d12" opacity=".45"/>')
b+=[poly(vl,'#f5f5f4'),poly(vr,'#d6d3d1'),poly(vt,'#ffffff')]
icon('1-inside',d1,''.join(b))


# 2 BRACKETS: the selection cursor around a voxel that doesn't exist yet
s=270;ox,oy=512,512
b=['<defs><filter id="g2"><feGaussianBlur stdDeviation="14"/></filter></defs>']
pts={k:P(*k,s,ox,oy+20) for k in [(a,b_,c) for a in(-.5,.5) for b_ in(-.5,.5) for c in(-.5,.5)]}
edges=[(a,b_) for a in pts for b_ in pts if a<b_ and sum(abs(x-y) for x,y in zip(a,b_))==1]
hidden=(-.5,-.5,-.5)
front=(.5,.5,.5)
L=0.34
segs=[]
for a,c in edges:
  pa,pc=pts[a],pts[c]
  for (p0,p1) in ((pa,pc),(pc,pa)):
    segs.append((p0,(p0[0]+(p1[0]-p0[0])*L,p0[1]+(p1[1]-p0[1])*L), hidden in (a,c) or front in (a,c)))
for glow in (True,False):
  for p0,p1,hid in segs:
    if hid or front in (p0k for p0k in []): continue
    b.append(f'<line x1="{p0[0]:.1f}" y1="{p0[1]:.1f}" x2="{p1[0]:.1f}" y2="{p1[1]:.1f}" stroke="{'#3b82f6' if glow else '#9cc4ff'}" stroke-width="{64 if glow else 42}" stroke-linecap="round" {'filter="url(#g2)" opacity=".8"' if glow else ''}/>')
icon('2-brackets',' ',''.join(b))

# 3 BEAM: one voxel set down by a shaft of light
d3='''<linearGradient id="beam" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff" stop-opacity="0"/><stop offset=".75" stop-color="#e0ecff" stop-opacity=".55"/><stop offset="1" stop-color="#fff" stop-opacity=".9"/></linearGradient>
<radialGradient id="pool" cx=".5" cy=".5" r=".5"><stop offset="0" stop-color="#9cc4ff" stop-opacity=".5"/><stop offset="1" stop-color="#9cc4ff" stop-opacity="0"/></radialGradient>'''
s=230;ox,oy=512,700
t,l,r=faces(-.5,-.5,0,s,ox,oy)
b=[f'<ellipse cx="512" cy="{oy+30}" rx="330" ry="120" fill="url(#pool)"/>']
bw=[t[3],t[0],t[1]]
b.append(f'<polygon points="{t[3][0]:.1f},{t[3][1]:.1f} {t[3][0]:.1f},0 {t[1][0]:.1f},0 {t[1][0]:.1f},{t[1][1]:.1f} {t[2][0]:.1f},{t[2][1]:.1f}" fill="url(#beam)"/>')
b+=[poly(l,'#3f3f46'),poly(r,'#18181b'),poly(t,'#ffffff')]
icon('3-beam',d3,''.join(b))

# 4 FACETS: three voxel faces pulled apart; the gap is a Y, one face carries the accent
s=300;ox,oy=512,520
t,l,r=faces(-.5,-.5,-.5,s,ox,oy)
cx,cy=P(.5,.5,.5,s,ox,oy)
def push(f,dx,dy,k=.86):
  mx=sum(p[0] for p in f)/4;my=sum(p[1] for p in f)/4
  return [(mx+(x-mx)*k+dx,my+(y-my)*k+dy) for x,y in f]
b=[poly(push(t,0,-22),'#fafafa'),poly(push(l,-20,14),'#3b82f6'),poly(push(r,20,14),'#27272a')]
icon('4-facets',' ',''.join(b))
