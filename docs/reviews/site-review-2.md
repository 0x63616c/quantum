# Site review 2 — independent screenshot critique

Reviewed the actual `screenshots/site/round-2/desktop-dark.png`, `desktop-light.png`, `mobile-dark.png`, and `mobile-light.png`, including full-resolution crops. Desktop captures are 1440 × 4671; mobile captures are 390 × 5497. I did not read prior review documents. Source was consulted only to confirm screenshot behavior and CTA destinations. This is a visual review, not a claim that interactions or accessibility have been tested.

**Desktop: 7.2/10. Mobile: 6.5/10.**

The typography, pure black/white surfaces, alignment, and real cottage render establish a credible product. They do not yet reach the requested top-tier creative-tool presentation. The page is restrained to the point of being under-art-directed: one excellent model does most of the emotional work, while most remaining sections are monochrome prose separated by rules. The agent-first promise is clear in the headline but weak in the first product evidence.

## Highest-priority fixes

1. **Put agent evidence beside the initial promise.** The hero screenshot explicitly shows “Local session” and “Agent sessions are available in the desktop app.” The actual agent activity proof only appears around y=2300 on desktop and y=2600 on mobile. A visitor must scroll through an API outline and three explanatory steps before seeing the distinctive capability. Use a real native-session hero capture, or move the existing session evidence directly after the hero. Keep the native/browser distinction honest. The current browser workspace alone sells an ordinary voxel editor more persuasively than an agent-first one.

2. **Repair the mobile hero image composition.** At 390px, the crop leaves a narrow, unreadable sidebar on the right and clipped menu text on the left. It looks accidentally clipped rather than intentionally composed. Show a clean canvas crop on mobile, preserving the complete model and removing partial sidebars. Keep the existing full-workspace link for the detailed screenshot. The recognizable artwork is good; the surviving strips of interface are visual debris.

3. **Make the mobile action match the device.** “Try it in your browser” appears as a large button near the top and again as the lower primary CTA, although the adjacent text says desktop keyboard and mouse are required. The caveat is legible, but the action still promises an immediate experience that the current device cannot provide. On narrow screens, use an accurate label such as “Explore the desktop editor” with a useful destination, or promote setup/documentation and retain the browser link as a clearly desktop-specific secondary action. Do not invent a mobile editor or a sharing workflow.

4. **Show the change claimed by “One visible change.”** The session panel proves a recorded +3 operation, but three tiny model thumbnails do not let a visitor see which three voxels changed. The screenshot is useful evidence, yet the headline asks it to do more than it visually can. Add a real before/after detail from the recorded edit if available; otherwise change the headline to describe the verifiable activity record. Also crop out the empty strip to the left of the session panel so the framed evidence feels deliberately prepared.

## Further visual and usability issues

- **The artwork arrives too late in the desktop first viewport.** The screenshot begins around y=600 and the actual model begins around y=827. On a typical 900px-tall view, the visitor sees the headline and browser chrome before seeing much of the work. Reduce the combined hero whitespace and screenshot framing, or use a more immediate product composition. Preserve the strong headline; reduce the distance to the thing it describes.
- **The lower half has little visual escalation.** Keyboard glyph, six swatches, undo glyph, and three outlined extension boxes feel like a specification page. The palette is the strongest of those details because it connects to the model. Give one hand-editing capability an actual small product demonstration or crop, then tighten the surrounding copy. Avoid adding decorative gradients or unsupported showcase assets.
- **Mobile asks for a long reading commitment.** The 5497px page contains an API block, three explanations, a second proof section, a host list, three more feature rows, file formats, a browser pitch, and an open-source pitch. Several are useful independently, but their equal visual weight makes the middle feel flat. Prioritize the agent result, then the few steps needed to understand it; shorten repeated prose and reduce spacing around secondary sections.
- **The bottom CTA gives its fine print unusually high prominence on desktop.** Its two-line requirement text is nearly body-copy size and visually competes with the supporting sentence. Keep the requirements readable, but restore a clear hierarchy between headline, value, action, and prerequisites.
- **Link polish is inconsistent.** “API reference↗” appears with the arrow touching the label, while other links use spaced arrows. Internal documentation links frequently use an external-link arrow. Standardize arrow spacing and use the symbol according to destination behavior.
- **The theme control looks identical in both screenshots.** A sun is shown on the white page as well as the black page. A state-appropriate icon or clearer state cue would make the control easier to interpret. The contrasting page themes themselves are clean and consistent.

## What should stay

The two-line agent-first headline is direct and memorable. Geist works well at both widths. Both themes preserve the artwork's warm colors and a coherent neutral hierarchy. The primary hero CTA has a generous target, the mobile headline does not break awkwardly, and the session crop is substantially more legible than a full application screenshot would be at phone width. The real product captures are valuable evidence; improve their order and framing instead of replacing them with invented interface mockups.

The next revision should improve proof, composition, and the mobile journey before adding any more content. The biggest gain is making the agent capability visible immediately while removing the accidental mobile screenshot crop.

## Implemented response

The hero now uses the authentic native session capture, with revision and native
context in its caption. Mobile crops only the canvas and keeps the full workspace
link. The recorded edit appears before the API outline and is described as a
record, without claiming a visible before/after. Mobile browser links explicitly
say desktop editor. Hero and secondary spacing is tighter; theme controls now
show the destination theme's sun/moon icon. Recapture: `screenshots/site/round-3/`.
