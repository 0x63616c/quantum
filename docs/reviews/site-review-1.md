# Site design review — round 1

Reviewed independently from these four supplied screenshots: `screenshots/site/round-1/desktop-dark.png`, `desktop-light.png`, `mobile-dark.png`, and `mobile-light.png`. The desktop dark and mobile light captures were also inspected at original resolution. `site/index.html` and its stylesheet were consulted to confirm visible copy, type sizes, and destinations. No interaction or screen-reader testing was performed. These are findings and proposed fixes, not completed changes.

| View | Score | Judgment |
| --- | --- | --- |
| Desktop, both themes | **6.5/10** | Controlled type, alignment, and a credible editor capture. The composition is competent but generic; too much of its central promise is asserted in prose rather than demonstrated. |
| Mobile, both themes | **5.5/10** | A tidy stack with sound main-button sizing, but a shrunken desktop image, an excessively long text sequence, and a browser CTA that conceals its device requirement until much later. |

The real cottage scene is doing almost all the emotional and product-specific work. After it, the page becomes a monochrome sequence of headings, rules, and small text. That is orderly, but it is not yet a distinguished creative-tool launch page. Dark and light modes are consistent; theme switching alone does not solve the hierarchy and evidence problems.

## 1. P1 — The first CTAs do not explain what the visitor can actually use

**Evidence:** Both mobile screenshots prominently offer “Try it in your browser” directly below the hero, with no adjacent device qualification. Only the much later “A little world, one tab away” section says “Desktop browser · Keyboard & mouse.” The native-app requirement for live agent sessions is also delayed until this lower section, despite “Connect your agent” being the main action at the top. The hero editor capture itself contains a tiny native-app restriction that is unreadable at mobile size.

**Why it matters:** A phone visitor gets an immediate invitation to an experience whose requirements the page discloses several screens later. An agent-focused visitor also cannot tell whether connecting means installing a native app, configuring MCP, or opening a browser. The hierarchy makes a high-friction setup path look instant.

**Proposed fix:** Put one concise requirement sentence directly beneath the first action group: browser editing uses desktop keyboard and mouse; live agent sessions use the native app. Give the primary action a more explicit setup label, such as “Set up your agent,” if that is what the linked documentation does. On mobile, use an honestly labeled desktop-editor preview or an explanatory browser entry; do not promise immediate phone editing. Keep the requirement attached to the relevant CTA when buttons stack.

## 2. P1 — Mobile scales the proof down until most of it stops communicating

**Evidence:** In both 390px mobile captures, the entire desktop editor is squeezed into a roughly 358px-wide image. Menu labels, layer controls, palette labels, and the session panel are effectively miniature texture. The cottage survives, but occupies only part of the narrow image. The screenshot also starts around 590px down the page, after a three-line title, subheading, two full-width buttons, and a slogan.

**Why it matters:** This is the only substantial product image on the page. Mobile visitors spend most of the first screen reading claims, then receive proof that requires magnification to inspect. Merely fitting the screenshot within the viewport is not a satisfactory mobile composition.

**Proposed fix:** Use a mobile-specific crop that prioritizes the actual cottage and a meaningful portion of the canvas. Retain the full workspace on desktop and make the uncropped capture available through a clearly labeled detail view. Label the crop as the real desktop editor. Tighten the hero's vertical sequence and deliberately set the mobile headline breaks so that the artwork arrives sooner. Verify at 360px and 390px that the model can be understood without zooming.

## 3. P1 — “Built for AI agents” lacks a concrete, visible demonstration

**Evidence:** The hero shows a finished cottage. The “From intent to something tangible” section then shows four tool names and a four-angle render configuration beside three explanatory paragraphs. There is no actual user brief, visible intermediate state, rendered contact sheet, or change that connects an agent action to the cottage. “A workflow outline” correctly describes the code panel, but also exposes how little evidence it provides.

**Why it matters:** The site's most prominent differentiator remains a claim. A reader can understand that MCP tools exist without seeing why the workflow is useful or believing the advertised creative loop. The code panel looks like documentation transplanted into a marketing section.

**Proposed fix:** Pair a short, specific example instruction with an authentic visible result: for example, a real instruction to change the roof or add a tree, the corresponding existing tool operation, and the actual resulting capture. Alternatively, show the real four-view contact sheet promised by the copy. Use verified captures and label illustrative material honestly. Keep the full reference accessible, but make the first visual argument understandable without reading method names.

## 4. P2 — The lower page has too little visual variation for its length

**Evidence:** From the workflow section through the file-format strip, both desktop themes use essentially the same monochrome heading/body/rule treatment. On mobile, the three agent explanations are followed by three more stacked text features, then three boxed file extensions. The editor-feature symbols are generic text glyphs (`⌘`, `◧`, `↶`), while “A palette with personality” contains no visible colors. The colorful screenshot is never echoed in another product detail.

**Why it matters:** Equal visual treatment makes distinct benefits feel interchangeable. The long mobile scroll accumulates words without giving the eye a new reason to stop. The outlined file boxes suggest importance, but contain little information and look more like controls than evidence of usable exports.

**Proposed fix:** Replace at least one prose-only editor feature with a real, legible detail: palette swatches, a selected tool with its shortcut, or a genuine version/history view. Keep the material drawn from the product rather than adding decorative gradients. Reduce repetitive explanatory copy and consolidate the three file cards into a compact export row or show an actual export outcome. Aim to make every major section contribute a new piece of evidence, not merely a new headline.

## 5. P2 — Supporting links are styled like incidental metadata

**Evidence:** On mobile, the code-panel heading and “API reference” link are 10px, the code and panel footer are 11px, and footer links are 12px. The API link is packed into the panel's narrow top bar. In the original-resolution mobile capture these remain decipherable, but they require disproportionate attention compared with the generous hero typography. “Bring your own agent” also occupies a three-row block with each row carrying little content.

**Why it matters:** Setup and documentation are essential to this product, yet their entry points receive footnote treatment. This is an accessibility and discoverability concern, not a claim that the ordinary body-text colors fail contrast: their contrast is visibly strong in both themes.

**Proposed fix:** Raise actionable supporting text to a comfortable 13–14px, give standalone links generous touch areas, and move the API reference below or outside the small panel label if necessary. Keep noninteractive labels small only where they are truly secondary. Compact the agent-host block while keeping each host link easy to tap. Confirm focus visibility and touch-target spacing in a browser; screenshots alone cannot establish those behaviors.

## Retain

Keep the real editor capture, restrained theme palette, consistent content alignment, and clear primary/secondary button distinction. The main mobile buttons already have useful height. The next revision should strengthen product evidence and device clarity before adding more styling. A redesign that merely brightens borders or adds decorative color would leave the central weaknesses intact.

## Implemented response

The next capture adds immediate device/agent requirements, an enlarged mobile
canvas crop with a full-workspace image link, an authentic cropped native session
showing a three-voxel edit and three captured angles, real workshop palette
swatches, and larger supporting links with touch spacing. These are fixes, not a
revised score. Recapture: `screenshots/site/round-2/`.
