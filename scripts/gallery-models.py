"""Original integer voxel sculptures. +Y is up; +Z is the front.

All geometry is submitted to Voxelite MCP by build-gallery.py. These helpers
only describe cells; they never write .voxl files or synthesize renders.
"""
import math

class Sculpture:
    def __init__(self, slug, title, category, colors, description):
        self.slug, self.title, self.category = slug, title, category
        self.colors, self.description = colors, description
        self.cells = {}; self.blockout = {}
    def box(self, a, b, slot):
        for x in range(a[0], b[0]):
            for y in range(a[1], b[1]):
                for z in range(a[2], b[2]):
                    if slot: self.cells[x,y,z] = slot
                    else: self.cells.pop((x,y,z), None)
    def ellipsoid(self, center, radii, slot):
        for x in range(math.floor(center[0]-radii[0]), math.ceil(center[0]+radii[0])):
            for y in range(math.floor(center[1]-radii[1]), math.ceil(center[1]+radii[1])):
                for z in range(math.floor(center[2]-radii[2]), math.ceil(center[2]+radii[2])):
                    if sum(((p+.5-c)/r)**2 for p,c,r in zip((x,y,z),center,radii))<=1:
                        if slot: self.cells[x,y,z]=slot
                        else: self.cells.pop((x,y,z),None)
    def cylinder(self, center, radius, lo, hi, slot, axis='y'):
        d={'x':0,'y':1,'z':2}[axis]; others=[i for i in range(3) if i!=d]
        for a in range(math.floor(center[0]-radius),math.ceil(center[0]+radius)):
            for b in range(math.floor(center[1]-radius),math.ceil(center[1]+radius)):
                if (a+.5-center[0])**2+(b+.5-center[1])**2<=radius**2:
                    for h in range(lo,hi):
                        p=[0,0,0];p[d]=h;p[others[0]]=a;p[others[1]]=b
                        if slot:self.cells[tuple(p)]=slot
                        else:self.cells.pop(tuple(p),None)
    def line(self, a, b, radius, slot):
        n=max(abs(a[i]-b[i]) for i in range(3))*2+1
        for k in range(n+1):
            self.ellipsoid(tuple(a[i]+(b[i]-a[i])*k/n for i in range(3)),(radius,)*3,slot)
    def stage(self): self.blockout=self.cells.copy()

def model(slug,title,category,palette,description):
    return Sculpture(slug,title,category,[tuple(c.split(':')) for c in palette.split()],description)

def robot():
    s=model('orbit','Orbit','Characters','Ink:27323F Shell:E6DAC2 Teal:64B6AC Glass:214E57 Glow:B7F3D1 Coral:E18269','A small companion with a very big field of view.')
    b=s.box
    for x in [10,23]: b((x,0,12),(x+8,4,23),1);b((x+1,4,14),(x+6,10,20),3)
    b((10,10,12),(31,25,24),2);b((13,25,15),(28,28,22),1);b((8,28,11),(33,42,25),3)
    for x in [4,32]:b((x,14,15),(x+5,25,21),2);b((x,11,15),(x+5,15,22),1)
    s.stage()
    b((8,21,16),(11,24,20),1);b((30,21,16),(33,24,20),1)
    b((10,31,24),(31,39,26),4)
    for x in [14,24]:b((x,33,26),(x+3,36,27),5)
    b((14,14,24),(27,21,25),1);b((16,16,25),(20,19,26),6);b((23,16,25),(25,19,26),5)
    b((19,42,17),(21,47,19),1);s.ellipsoid((20,48,18),(3,2,3),6)
    for y in [14,17,20]:b((31,y,16),(32,y+1,20),1)
    return s

def kiosk():
    s=model('midnight-noodles','Midnight noodles','Buildings','Charcoal:293645 Timber:AC6A42 Paper:F0DFC0 Vermilion:E75B43 Gold:E9B45B Teal:72B5AE Noodle:FFDC8A','One last bowl, under the paper lanterns.')
    b=s.box;b((2,0,2),(42,2,32),1);b((6,2,5),(38,6,27),2)
    b((7,6,6),(37,26,10),2)
    for x in [7,35]:b((x,6,9),(x+2,27,27),2)
    b((6,12,22),(38,15,28),2);b((5,15,21),(39,17,29),3)
    for y in range(5):b((3-y,27+y,3-y),(41+y,28+y,30+y),1)
    b((9,21,25),(35,27,27),4);s.stage()
    b((-1,27,-1),(46,32,35),0)
    for y in range(7):b((-1+y//3,27+y,1+y*2),(45-y//3,28+y,33-y*2),1)
    for x in [11,16,21,26,31]:b((x,21,27),(x+2,26,28),3)
    for x in [10,20,30]:
        b((x,2,29),(x+1,7,30),1);b((x-2,7,27),(x+3,9,32),4)
    for x in [9,33]:
        b((x,23,29),(x+1,27,30),1);s.ellipsoid((x+.5,21,29.5),(2.5,3.5,2.5),5);b((x,18,28),(x+1,24,29),4)
    for x in [13,23]:
        s.cylinder((x,25),2,17,19,6);s.cylinder((x,25),1.5,19,20,7)
        b((x-2,20,24),(x+3,21,25),2)
    for x in [13,21,29]:b((x,18,10),(x+4,24,11),3)
    b((6,9,6),(7,20,21),3)
    return s

def camper():
    s=model('weekender','Weekender','Vehicles','Rubber:28303A Cream:F1E5C8 Orange:E99458 Glass:4B8288 Chrome:B0C3C0 Wood:AA7050 Board:D5AAA0','A surfboard, a full tank, and no particular plan.')
    b=s.box;b((7,5,4),(25,13,40),3);b((7,13,4),(25,24,36),2);b((8,13,36),(24,20,40),2);b((8,24,5),(24,26,35),2)
    for x in [5,24]:
        for z in [11,32]:s.cylinder((8,z),4,x,x+3,1,'x');s.cylinder((8,z),2,x,x+3,5,'x')
    s.stage();b((9,16,39),(23,21,40),4);b((15,16,39),(16,21,41),2)
    for z in [8,18,28]:b((25,16,z),(26,23,z+6),4);b((6,16,z),(7,23,z+6),4)
    b((6,12,5),(26,14,40),3);b((7,6,40),(25,8,42),5)
    for x in [9,20]:b((x,10,40),(x+3,12,41),2)
    b((12,9,40),(20,10,41),1)
    for z in [10,27]:b((8,26,z),(24,28,z+2),6)
    s.ellipsoid((16,29,21),(4,1.5,17),7);b((15,29,8),(17,31,34),2)
    return s

def bonsai():
    s=model('slow-sunday','Slow Sunday','Nature','Bark:77503A Moss:7D9454 Jade:4E8361 Leaf:A6BC72 Ceramic:D8D7C2 Earth:554F3C','An entire old forest, small enough for a windowsill.')
    b=s.box;b((6,0,7),(29,3,24),5);b((3,3,5),(32,7,26),5);b((5,7,7),(30,8,24),6)
    s.line((16,7,16),(20,19,15),3,1);s.line((20,19,15),(13,29,14),2.5,1)
    s.line((19,18,15),(30,24,15),2,1);s.line((16,23,15),(6,24,19),2,1)
    for c,r in [((12,31,14),(12,5,9)),((29,26,15),(9,4,7)),((6,26,20),(8,4,7))]:s.ellipsoid(c,r,3)
    s.stage()
    for c,r in [((10,34,14),(9,2,7)),((28,29,15),(7,2,5)),((5,29,20),(6,2,5))]:s.ellipsoid(c,r,4)
    for c in [(10,8,16),(23,8,20),(23,8,10)]:s.ellipsoid(c,(4,1.5,3),2)
    for x in [8,24]:b((x,0,9),(x+4,2,22),5)
    return s

def camera():
    s=model('instant-summer','Instant summer','Objects','Ink:303740 Ivory:E8DFCB Mint:78ADA4 Lens:193D49 Glass:87C3D1 Coral:E48373 Gold:DABB6F','A little analog magic, one square at a time.')
    b=s.box;b((4,4,5),(32,21,19),2);b((4,6,18),(32,18,21),3);b((4,21,6),(32,26,18),2)
    s.cylinder((19,14),8,21,25,1,'z');s.cylinder((19,14),6,25,27,4,'z');s.stage()
    s.cylinder((19,14),4,27,28,5,'z');s.cylinder((19,14),2,28,29,4,'z')
    b((6,22,18),(14,25,20),1);b((7,22,20),(13,24,21),7);b((25,22,18),(29,25,20),4)
    b((26,16,21),(30,20,23),6);b((5,2,9),(31,5,21),1)
    b((9,1,18),(28,3,35),2);b((11,2,22),(26,4,32),3)
    b((11,3,27),(26,4,32),6);b((19,3,23),(23,4,27),7)
    for x in [7,9,11]:b((x,8,21),(x+1,13,22),1)
    return s

def fox():
    s=model('forest-friend','Forest friend','Characters','Rust:C66D3D Orange:ED9C54 Cream:F0E0BE Ink:30363D Tail:DC8248','Quiet paws. An excessively good tail.')
    b=s.box;s.ellipsoid((18,12,16),(8,10,6),1);s.ellipsoid((18,23,19),(10,8,7),2)
    for x in [11,23]:b((x,0,16),(x+4,8,22),1)
    for x in [9,23]:
        for y in range(7):b((x+y//3,28+y,15),(x+6-y//3,29+y,20),2)
    s.line((23,6,12),(32,5,7),4,5);s.line((32,5,7),(37,12,9),4,5);s.ellipsoid((37,14,10),(4,5,4),3)
    s.stage();s.ellipsoid((18,19,24),(7,4,5),3);b((16,20,28),(20,22,30),4)
    for x in [12,23]:b((x,25,23),(x+2,27,26),4)
    s.ellipsoid((18,11,21),(5,7,1.5),3)
    for x in [11,24]:b((x,29,19),(x+3,32,20),3)
    return s

def observatory():
    s=model('after-hours','After hours','Buildings','Stone:CDC6B4 Blue:6E99A1 Dome:98BABC Dark:354950 Brass:D8AF64 Glass:92C6D5','For the nights when looking up is the whole plan.')
    b=s.box;s.cylinder((22,22),20,0,2,4);s.cylinder((22,22),15,2,21,1)
    s.cylinder((22,22),16,20,23,4);s.ellipsoid((22,23,22),(15,14,15),2);b((0,2,0),(45,22,45),0);s.cylinder((22,22),15,2,21,1)
    s.stage()
    # Dome bearing joins the restored wall to the hemisphere.
    s.cylinder((22,22),16,20,23,4)
    # Dome slot and a connected upward-pointing telescope.
    b((20,23,22),(24,38,38),4);s.line((22,27,27),(22,36,40),3,5);s.line((22,35,39),(22,38,43),3.5,4)
    for x,z in [(7,20),(36,20),(20,36)]:b((x,9,z),(x+3,16,z+1),4);b((x,10,z+1),(x+3,15,z+2),6)
    b((20,2,36),(25,10,38),4)
    for i in range(4):b((18,0,36+i),(27,4-i,37+i),1)
    s.cylinder((22,22),16,3,5,2)
    return s

def astronaut():
    s=model('small-step','Small step','Characters','Suit:E6E4D6 Shadow:ADB9BA Visor:344854 Gold:E6B767 Orange:DB7855 Blue:80AEBB','A tiny explorer, a very long way from home.')
    b=s.box
    for x in [11,23]:b((x,0,14),(x+7,5,25),2);b((x+1,5,14),(x+6,15,21),1)
    b((11,14,11),(30,29,24),1);b((10,16,6),(31,28,12),2)
    s.ellipsoid((20.5,36,18),(12,11,10),1)
    s.line((9,27,16),(5,18,20),3,1);s.line((32,26,17),(38,32,20),3,1)
    s.stage();s.ellipsoid((20.5,37,26),(9,7,3),3);s.ellipsoid((20,37,28),(7,5,1),4)
    b((15,40,29),(20,42,30),1);b((15,20,24),(26,26,26),2)
    for x,col in [(16,5),(20,6),(24,3)]:b((x,22,26),(x+2,24,27),col)
    for x in [12,24]:b((x,8,14),(x+5,10,22),5)
    b((39,0,20),(40,42,21),3);b((40,33,20),(52,41,21),5);b((42,35,21),(47,39,22),1)
    return s

def mushrooms():
    s=model('under-the-canopy','Under the canopy','Nature','Earth:76604B Grass:9CA873 Stem:E9D9B6 Red:D87562 Light:F6E7CC Shade:B8534E','A miniature gathering on the forest floor.')
    s.cylinder((22,21),18,0,3,1);s.cylinder((22,21),18,3,4,2)
    for x,y,z,r in [(17,23,16,11),(31,15,26,8),(10,11,29,6)]:
        s.cylinder((x,z),max(2,r//3),4,y,3);s.ellipsoid((x,y,z),(r,r*.48,r),4)
        s.box((x-r,y-int(r*.5),z-r),(x+r,y-1,z+r),0);s.cylinder((x,z),max(2,r//3),4,y,3)
    s.stage()
    for x,y,z,r in [(17,23,16,11),(31,15,26,8),(10,11,29,6)]:
        for dx,dz in [(-3,-3),(3,-2),(0,4)]:s.ellipsoid((x+dx,y+r*.35,z+dz),(2,1.5,2),5)
    for x,z in [(9,15),(30,12),(23,33),(35,23)]:
        s.box((x,4,z),(x+1,8,z+1),2);s.box((x-1,6,z),(x+2,7,z+1),2)
    return s

def arcade():
    s=model('one-more-credit','One more credit','Objects','Dark:303345 Purple:8881B5 Pink:E0A2BD Screen:253F48 Cyan:8CD1CE Gold:EBD699 Cream:E6E0DB','The high score can wait. One more game.')
    b=s.box;b((8,0,8),(30,4,26),1);b((8,4,8),(30,21,26),2);b((8,21,8),(30,44,22),2)
    b((7,36,7),(31,44,25),1);b((7,19,21),(31,23,30),3);s.stage()
    b((10,25,22),(28,35,24),1);b((12,26,24),(26,34,25),4)
    b((10,38,25),(28,42,26),3)
    for x in [12,16,20,24]:b((x,39,26),(x+2,41,27),7)
    b((13,23,25),(14,26,26),1);s.ellipsoid((13.5,27,25.5),(2,2,2),3)
    for x,col in [(21,5),(26,6)]:s.cylinder((x,26),1.5,23,25,col)
    b((16,9,26),(22,16,27),1);b((18,13,27),(20,14,28),6)
    for x,y in [(14,29),(17,31),(20,29),(23,32)]:b((x,y,25),(x+2,y+2,26),5)
    b((30,9,11),(31,30,14),3);b((30,12,14),(31,27,17),3)
    return s

def lighthouse():
    s=model('the-last-light','The last light','Buildings','Rock:667D83 Sand:C8BB9B Chalk:E7DFCA Red:C96F59 Roof:425A66 Gold:EFCD84 Sea:739DA1','A familiar landmark at the edge of everything.')
    s.ellipsoid((20,3,20),(17,4,15),1);s.cylinder((20,20),9,4,8,2)
    for y in range(8,42):s.cylinder((20,20),9-(y-8)//9,y,y+1,3 if (y//7)%2 else 4)
    s.cylinder((20,20),9,42,44,5);s.cylinder((20,20),5,44,51,6)
    for y in range(5):s.cylinder((20,20),9-y*1.5,51+y,52+y,5)
    s.stage()
    for x,z in [(15,17),(24,17),(15,23),(24,23)]:s.box((x,44,z),(x+1,51,z+1),5)
    s.box((18,8,28),(22,15,30),5)
    for y in [22,35]:s.box((19,y,26),(22,y+4,28),5)
    for a in range(0,360,45):
        x=20+round(7*math.cos(math.radians(a)));z=20+round(7*math.sin(math.radians(a)))
        s.box((x,44,z),(x+1,47,z+1),5)
    for y in [47]:
        s.cylinder((20,20),9,y,y+1,5);s.cylinder((20,20),7,y,y+1,0);s.cylinder((20,20),5,y,y+1,6)
    return s

def rover():
    s=model('dust-runner','Dust runner','Vehicles','Tire:39434B Hull:D8CEB2 Orange:D19159 Solar:557D96 Line:A6C5D4 Lens:294B54','Built for the scenic route across another planet.')
    b=s.box;b((7,9,7),(29,15,33),1);b((8,15,8),(28,22,32),2)
    for x in [3,29]:
        for z in [10,21,32]:s.cylinder((8,z),5,x,x+5,1,'x');s.cylinder((8,z),2,x,x+5,3,'x')
    b((15,22,17),(18,34,20),2);b((12,33,17),(23,39,23),2)
    s.stage();b((3,21,18),(10,23,21),1);b((26,21,18),(33,23,21),1);b((13,34,23),(22,38,25),6)
    for x in [14,19]:b((x,35,25),(x+2,37,26),5)
    for x in [-3,29]:
        b((x,22,10),(x+10,23,30),4)
        for z in range(11,30,4):b((x,23,z),(x+10,24,z+1),5)
        b((x+4,23,10),(x+5,24,30),5)
    b((10,19,32),(26,21,33),3);b((25,22,10),(26,39,11),1)
    s.ellipsoid((25.5,40,10.5),(2,2,2),3)
    return s

def cactus():
    s=model('desert-hours','Desert hours','Nature','Clay:C78566 Rim:E0AA85 Green:6F9475 Light:A2B587 Sand:E1CB9D Pink:E4A7AD','Patient company for a sunny corner.')
    s.cylinder((20,20),10,0,3,1)
    for y in range(3,13):s.cylinder((20,20),10+(y//5),y,y+1,1)
    s.cylinder((20,20),13,13,16,2);s.cylinder((20,20),11,16,17,5)
    s.cylinder((20,20),5,17,42,3);s.ellipsoid((20,42,20),(5,5,5),3)
    s.line((20,27,20),(10,27,20),3,3);s.cylinder((10,20),3,27,36,3);s.ellipsoid((10,36,20),(3,3,3),3)
    s.line((20,23,20),(30,23,20),3,3);s.cylinder((30,20),3,23,31,3);s.ellipsoid((30,31,20),(3,3,3),3)
    s.stage()
    for x,z in [(19,24),(23,22),(15,20)]:s.box((x,18,z),(x+1,42,z+1),4)
    for y in range(21,42,5):s.box((20,y,25),(21,y+1,26),5)
    for dx,dz in [(0,0),(2,0),(-2,0),(0,2),(0,-2)]:s.ellipsoid((20+dx,47,20+dz),(2,1.5,2),6)
    s.ellipsoid((20,48,20),(1,1,1),5)
    return s

def train():
    s=model('branch-line','Branch line','Vehicles','Coal:303C43 Green:527E70 Cream:D7CDB5 Brass:D2A75C Red:B96455 Steel:9EADB0','A slow train to somewhere worth stopping.')
    b=s.box;b((8,5,3),(26,9,43),1);s.cylinder((17,17),8,15,39,2,'z');b((7,9,4),(27,30,17),2);b((5,30,2),(29,33,19),1)
    for x in [5,26]:
        for z in [10,23,35]:s.cylinder((6,z),5,x,x+3,1,'x');s.cylinder((6,z),3,x,x+3,4,'x')
    s.cylinder((17,32),3,22,32,1);s.cylinder((17,32),5,31,34,1);s.stage()
    b((9,20,17),(15,28,18),3);b((19,20,17),(25,28,18),3)
    b((27,20,6),(28,28,14),3);s.cylinder((17,17),6,39,40,1,'z');s.cylinder((17,17),2,40,42,4,'z')
    for z in [20,28]:s.cylinder((17,17),8.3,z,z+1,4,'z')
    b((4,6,10),(5,8,36),6);b((29,6,10),(30,8,36),6);b((6,5,42),(28,8,45),5)
    return s

def pond():
    s=model('still-water','Still water','Nature','Stone:9EA799 Moss:7F946E Water:6CA7AC Deep:4A7F8A Cream:F2DFB4 Koi:E99964 Leaf:567F63','A koi pond with absolutely nowhere to be.')
    s.cylinder((23,23),21,0,4,1);s.cylinder((23,23),18,3,5,3)
    for c,r in [((8,6,15),(5,4,6)),((14,5,6),(6,3,4)),((35,5,29),(5,3,7))]:s.ellipsoid(c,r,1)
    s.stage()
    for x,z in [(12,29),(28,13),(28,31)]:
        s.cylinder((x,z),4,5,6,7);s.box((x,5,z),(x+5,6,z+1),0)
    for x,z in [(20,22),(28,23)]:
        s.ellipsoid((x,5,z),(2,1,4),5);s.ellipsoid((x,5,z+1),(2,1,2),6);s.box((x-2,5,z-5),(x+2,6,z-3),6)
    for x,z in [(5,21),(13,7),(37,26)]:s.ellipsoid((x,6,z),(3,2,3),2)
    for x in [7,9,11]:s.box((x,3,36),(x+1,12+(x%3),37),7)
    return s

def sneaker():
    s=model('off-court','Off court','Objects','Sole:E8DDC8 Ink:303E4B Teal:729F98 Cream:F4E9D6 Coral:D78C77 Lace:D5B995','Fresh out of the box. Ready to go nowhere fast.')
    b=s.box;b((8,0,4),(25,4,38),1);s.ellipsoid((16.5,4,34),(9,4,9),1)
    b((8,4,5),(25,14,22),3);s.ellipsoid((16.5,7,29),(8.5,6,13),3);b((9,14,5),(24,20,15),3)
    s.stage();b((11,17,8),(22,21,14),2);b((11,12,15),(22,16,26),4)
    for z in range(17,27,3):b((12,16-(z-17)//5,z),(21,17-(z-17)//5,z+1),6)
    b((25,6,8),(26,9,28),4)
    for i in range(7):b((25,8+i//2,27-i),(26,10+i//2,29-i),4)
    b((8,13,5),(25,16,8),5)
    for z in range(7,37,4):b((8,1,z),(9,3,z+1),2)
    return s

def windmill():
    s=model('lavender-mill','Lavender mill','Buildings','Wall:E2CEAA Roof:758F9A Wood:8D6653 Canvas:EDE2C8 Grass:A0AA83 Lavender:9E8FAE','A little wind, a little lavender, a quieter pace.')
    b=s.box;b((0,0,0),(39,2,31),5)
    for y in range(2,29):
        k=(y-2)//9;b((9+k,y,5+k),(30-k,y+1,23-k),1)
    for y in range(9):b((7+y,29+y,3+y//2),(32-y,30+y,25-y//2),2)
    s.stage();b((17,2,22),(23,11,24),3);b((18,20,21),(22,25,23),2)
    # Crossed timber spars with cream canvas at the four tips.
    for dx,dy in [(-1,-1),(-1,1),(1,-1),(1,1)]:
        for i in range(1,17):
            x=20+dx*i;y=26+dy*i;b((x,y,27),(x+2,y+2,29),3)
            if i>5:b((x-2,y,29),(x+3,y+2,30),4)
    s.cylinder((21,27),2,19,29,3,'z');s.cylinder((21,27),3,29,31,3,'z')
    for x in [3,6,32,35]:
        for z in [6,11,16,21]:b((x,2,z),(x+1,6,z+1),5);b((x-1,5,z),(x+2,8,z+1),6)
    return s

def submarine():
    s=model('yellow-submarine','Yellow submarine','Vehicles','Yellow:E5B957 Gold:C08D44 Ink:34515A Glass:88BCC0 Cream:F3DDA7 Red:C97C60','For discoveries just beneath the surface.')
    b=s.box;s.ellipsoid((20,15,23),(10,10,21),1);b((16,23,17),(24,30,29),1)
    b((19,30,22),(22,36,25),3);b((19,34,24),(22,37,30),3)
    s.stage()
    for z in [13,24,33]:
        s.cylinder((15,z),4,29,31,3,'x');s.cylinder((15,z),2.5,31,32,4,'x')
    b((6,11,14),(34,13,22),2);b((17,7,1),(23,23,3),2)
    s.cylinder((20,15),2,-1,3,3,'z');b((12,14,-2),(28,16,0),5);b((19,7,-2),(21,23,0),5)
    b((17,27,28),(23,29,30),4);s.ellipsoid((20,23,37),(2,2,2),6)
    return s

BUILDERS=[robot,kiosk,camper,bonsai,camera,fox,observatory,astronaut,mushrooms,arcade,lighthouse,rover,cactus,train,pond,sneaker,windmill,submarine]
