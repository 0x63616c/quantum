# Quantize artwork

Calum approved the original Quantize concept in #84: a smooth left hemisphere
and a right hemisphere quantized into rounded voxels, separated by a hard vertical
seam. The two SVG masters are readable vector artwork with no embedded bitmaps.

- [icon.svg](icon.svg) preserves the original 1024 canvas, 824 px tile, 185.4 px
  continuous corners (60% smoothing), 12 px shadow offset/blur and subtle rim.
  The sphere has radius 276, lighting at (539.6, 340.88), a 447.12 px radial
  gradient, and a clipped left edge at x=509. The right grid has 46 px pitch,
  41 px cells, 6 px radii and the approved six grey levels. There is no seam blur.
- [mark-small.svg](mark-small.svg) is the optical master for 16/32 pt icons and
  favicons. It keeps the lighting and hard seam, with four rows of 106 px cells,
  32 px gaps and 12 px radii. Retina images select this by logical size, so a
  32 pt @2x icon differs intentionally from a 64 pt @1x icon.

Run `cargo xtask icons` after editing a master. It uses the locked resvg/tiny-skia
renderer to produce the iconset, `icon-1024.png`, `Voxelite.icns`, the app's
`app-icon-256.png`, the site/README mark, SVG/PNG favicons and
[size/theme preview](../../docs/images/quantize-preview.png). The preview uses
bundled Geist and includes actual-pixel-size samples on both backgrounds.
`--preview PATH` also copies it to a review location.

The iconset contains the ten standard macOS filenames plus 64 pt (1x/2x) and
1024 px copies. ICNS stores the ten standard PNG representations with explicit
size/scale tags. The app targets macOS 13+. Packaging copies the generated ICNS
unchanged into `Contents/Resources/Voxelite.icns`, matching the existing
`CFBundleIconFile`; artwork replacement does not rename bundle identifiers.

Commit all derivatives. `cargo xtask icons --check` and the registered `icons`
validation gate compare exact bytes. Tests check transparent margins, the hard
seam, PNG dimensions, ICNS framing, Retina mapping and packaging's unchanged copy.
macOS Dock/Finder presentation and native ICNS decoding still need Mac verification.
