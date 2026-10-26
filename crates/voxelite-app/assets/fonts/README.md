# Bundled fonts

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

`ShortcutSymbols.ttf` is a 19-glyph subset of
[DejaVu Sans Mono 2.37](https://dejavu-fonts.github.io/), renamed to
`Voxelite Shortcut Symbols`. Its copyright and permission notice is in
`LICENSE-ShortcutSymbols.txt` and retained in the font metadata. It supplies
⌃⌥⇧⌘⎋↩⇥⌫⌦⇞⇟↖↘␣⇪↑↓←→ through an explicit fallback for Geist and Geist Mono.
The browser cannot use system fonts for these non-emoji symbols, and Geist
does not contain all of them. Native and web startup load the same font list.

To reproduce the subset with fontTools from an original `DejaVuSansMono.ttf`:

```python
from fontTools import subset
from fontTools.ttLib import TTFont

font = TTFont("DejaVuSansMono.ttf", recalcTimestamp=False)
options = subset.Options()
options.name_IDs = ["*"]
options.name_legacy = True
options.name_languages = ["*"]
options.recalc_timestamp = False
subsetter = subset.Subsetter(options=options)
subsetter.populate(text="⌃⌥⇧⌘⎋↩⇥⌫⌦⇞⇟↖↘␣⇪↑↓←→")
subsetter.subset(font)
names = {
    1: "Voxelite Shortcut Symbols", 2: "Regular",
    3: "Voxelite Shortcut Symbols 2.37", 4: "Voxelite Shortcut Symbols",
    6: "VoxeliteShortcutSymbols", 16: "Voxelite Shortcut Symbols", 17: "Regular",
}
for record in font["name"].names:
    if record.nameID in names:
        record.string = names[record.nameID].encode(record.getEncoding())
font.save("ShortcutSymbols.ttf")
```
