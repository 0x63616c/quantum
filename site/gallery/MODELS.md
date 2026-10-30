# Model render review

All original subjects are authored by `scripts/gallery-models.py` and submitted
through actual document-create, palette, batch, version, render and save MCP
operations by `scripts/build-gallery.py`. No image-generation service or
substitute renderer is used. The existing Autumn Workshop is opened through
MCP; its source in examples/ is untouched.

Blockout review: inspected all 19 subjects at iso/front/right/back, 512 pixels
per view, on the black background. Original contact sheets and immutable
revision metadata are in `screenshots/gallery/models/blockout/`.

| Model | Blockout finding | Refinement for the next render |
| --- | --- | --- |
| Orbit | Clear stance, blank face loses personality | Dark face, mint eyes, coral antenna, chest panel |
| Midnight noodles | Dark eave dominates; counter is featureless | Lanterns, curtain, bowls, stools, back-wall menus |
| Weekender | Recognizable van, blank glazing and roof | Windshield, windows, bumper, lamps, surfboard |
| Slow Sunday | Branches read; crown is one value | Light foliage caps and moss in the pot |
| Instant summer | Lens reads, body is generic | Glass, viewfinder, shutter, flash, emerging print |
| Forest friend | Tail and ears read; blank face is unacceptable | Cream muzzle/chest, nose/eyes, inner ears |
| After hours | Dome reads as a silo | Dome slot, brass telescope, windows, door and steps |
| Small step | Good waving pose, white helmet obscures identity | Gold visor, chest controls, suit bands, flag |
| Under the canopy | Cap/stem separation succeeds | Cream cap flecks and small grass tufts |
| One more credit | Cabinet silhouette is plain | Game screen, marquee, joystick/buttons, coin slot |
| The last light | Readable silhouette, lantern needs structure | Lantern posts, rail, windows, entry |
| Dust runner | Wheels work, top is sparse | Camera, arrays, aerial; physical array support arms at Y=21–22 |
| Desert hours | Strong arms/pot, uniform green | Ribs, sparse spines, pink flower |
| Branch line | Train reads, front and cab are blank | Boiler bands, headlamp, cab glazing, rods, buffers |
| Still water | Too empty to hold attention | Koi, notched lily pads, moss and reeds |
| Off court | Sole works, ankle needs depth | Dark collar, tongue/laces, side stripe, heel, tread |
| Lavender mill | Tower lacks windmill identity | Canvas sails, hub, door/window, lavender beds |
| Yellow submarine | Hull lacks a focal point | Portholes, fins, propeller, conning window, marker |
| Autumn workshop | Existing scene is already coherent | Preserve artwork; frame against both themes |

Final four-view renders use a consistent saved revision. Evidence in
`screenshots/gallery/models/final/*.json` records camera metadata, source SHA-256,
revision and mutation receipts. Tiles use a fitted 35-degree camera and a
47-degree alternate, both at 27-degree elevation. Both themes are rendered by
Voxelite, then encoded locally as 640px tile and 1024px large WebP/AVIF assets.

Final four-view inspection found two additional repairs: the windmill rotor
needed an axle at X=21/Y=27 spanning Z=19–29, visible as disconnected in right
view. Midnight Noodles needed a stepped ridge instead of a broad flat roof.
Both were patched in the authoring recipe and re-rendered through MCP in the
first page review round. The remaining 17 silhouettes, supports and focal
materials were accepted after inspecting their final four-view sheets.

The second page review added a face-connected cell audit. Repairs: Orbit's
shoulder joints at X=8–10 and X=30–32; fox eye inserts extending back to Z=23;
observatory bearing spanning Y=20–22; lighthouse balcony posts moved inside the
rail radius; pond reeds extended down to Y=3. Every original finished model now
has a single connected component. The builder rejects disconnected originals.
This complements the actual four-angle visual review rather than replacing it.
