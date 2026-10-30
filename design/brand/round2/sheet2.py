import base64,subprocess,cairosvg
R='/workspace/brand/round2'
from sheet import names
def b64(p): return 'data:image/png;base64,'+base64.b64encode(open(p,'rb').read()).decode()
for n,*_ in names:
  subprocess.run(f"magick {R}/{n}-32.png -filter point -resize 96x96 /tmp/{n}-32x3.png; magick {R}/{n}-16.png -filter point -resize 96x96 /tmp/{n}-16x6.png",shell=True)
W=620;H=1080;out=[f'<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{W*4+80}" height="{H+160}"><rect width="100%" height="100%" fill="#0b0b0c"/>',
'<text x="60" y="90" font-family="Geist" font-weight="600" font-size="44" fill="#fafafa">Voxelite icon, round 2</text><text x="60" y="130" font-family="Geist" font-size="24" fill="#8a8a8e">macOS squircle on black · 1024 · Dock at 64/32/16 (actual and zoomed) · wordmark</text>']
for i,(n,t,idea,acc) in enumerate(names):
  x=40+i*W+20;y=180
  out.append(f'<text x="{x}" y="{y+30}" font-family="Geist" font-weight="600" font-size="34" fill="#fafafa">{i+1}  {t}</text>')
  out.append(f'<foreignObject/><text x="{x}" y="{y+68}" font-family="Geist" font-size="17" fill="#9a9aa0">{idea}</text>')
  out.append(f'<image x="{x}" y="{y+100}" width="560" height="560" xlink:href="{b64(R+"/"+n+"-1024.png")}"/>')
  dy=y+700
  out.append(f'<rect x="{x}" y="{dy}" width="560" height="150" rx="28" fill="#2a2a2d" stroke="#3a3a3e"/>')
  out.append(f'<image x="{x+24}" y="{dy+43}" width="64" height="64" xlink:href="{b64(R+"/"+n+"-64.png")}"/>')
  out.append(f'<image x="{x+112}" y="{dy+59}" width="32" height="32" xlink:href="{b64(R+"/"+n+"-32.png")}"/>')
  out.append(f'<image x="{x+168}" y="{dy+67}" width="16" height="16" xlink:href="{b64(R+"/"+n+"-16.png")}"/>')
  out.append(f'<image x="{x+230}" y="{dy+27}" width="96" height="96" style="image-rendering:pixelated" xlink:href="{b64("/tmp/"+n+"-32x3.png")}"/>')
  out.append(f'<image x="{x+350}" y="{dy+27}" width="96" height="96" xlink:href="{b64("/tmp/"+n+"-16x6.png")}"/>')
  out.append(f'<text x="{x+278}" y="{dy+143}" font-family="Geist Mono" font-size="13" fill="#77777c" text-anchor="middle">32 ×3</text><text x="{x+398}" y="{dy+143}" font-family="Geist Mono" font-size="13" fill="#77777c" text-anchor="middle">16 ×6</text>')
  out.append(f'<image x="{x}" y="{dy+190}" width="560" height="168" xlink:href="{b64(R+"/"+n+"-wordmark.png")}"/>')
out.append('</svg>')
s='\n'.join(out).replace('<foreignObject/>','')
cairosvg.svg2png(bytestring=s.encode(),write_to='/workspace/brand/round2-sheet.png')
