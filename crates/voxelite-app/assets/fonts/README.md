# Bundled Geist faces

`Geist.ttf` and `GeistMono.ttf` are the existing locally bundled variable fonts,
with their accompanying SIL Open Font License notices. The two static Geist
faces are unmodified weight instances of that same `Geist.ttf`, generated with
fontTools 4.65.0. They preserve the original license and copyright metadata.

GPUI 1.25.3's Linux cosmic-text backend matches concrete font faces. Registering
only the variable font resolves medium and semibold requests to its regular
face. Bundle weight 500 and 600 instances so the intended hierarchy renders on
Linux as well as macOS. No fonts are downloaded or generated during app builds.

To reproduce the static files from this directory (optional asset tooling):

```python
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

for weight, name in [(500, "Medium"), (600, "SemiBold")]:
    font = TTFont("Geist.ttf", recalcTimestamp=False)
    instantiateVariableFont(font, {"wght": weight}, inplace=True, updateFontNames=True)
    font.save(f"Geist-{name}.ttf")
```
