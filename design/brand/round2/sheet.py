import cairosvg,os,subprocess
R='/workspace/brand/round2'
names=[('1-inside','Inside','Look into a voxel: a lit room with the first block inside.','#ffb648'),
('2-brackets','Brackets','The selection cursor around a voxel that doesn’t exist yet.','#60a5fa'),
('3-beam','Beam','One voxel set down by a shaft of light.','#fff'),
('4-facets','Facets','A voxel pulled apart into its three faces; the gap is a Y.','#3b82f6')]
for n,t,_,acc in names:
  icon=open(f'{R}/{n}.svg').read()
  inner=icon.split('>',1)[1].rsplit('</svg>',1)[0]
  wm=f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 480" width="1600" height="480"><rect width="1600" height="480" fill="#000"/>
<svg x="120" y="96" width="288" height="288" viewBox="0 0 1024 1024">{inner}</svg>
<text x="460" y="292" font-family="Geist" font-weight="600" font-size="150" letter-spacing="-6" fill="#fafafa">Voxelite</text></svg>'''
  open(f'{R}/{n}-wordmark.svg','w').write(wm); cairosvg.svg2png(bytestring=wm.encode(),write_to=f'{R}/{n}-wordmark.png')
  for px in (32,16,64): cairosvg.svg2png(url=f'{R}/{n}.svg',write_to=f'{R}/{n}-{px}.png',output_width=px,output_height=px)
# dock mock per direction: real-size 32/16 + 8x nearest zoom of each
for n,*_ in names:
  subprocess.run(f"magick -size 520x120 xc:'#2b2b2e' \\( {R}/{n}-64.png \\) -geometry +20+28 -composite {R}/{n}-32.png -geometry +110+44 -composite {R}/{n}-16.png -geometry +170+52 -composite \\( {R}/{n}-32.png -filter point -resize 256x256 -crop 96x96+0+0 +repage \\) -composite {R}/{n}-dock.png",shell=True)
