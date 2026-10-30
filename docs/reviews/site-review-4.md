# Site review 4 — show the product, make the proof readable

Reviewed 2026-10-25. Target: a credible 9/10 creative-tool launch site, judged
against the restraint and clarity of the selected Concept A, Vercel and Stripe.
This is a fresh assessment of the actual screenshots, not a carry-forward of
previous scores or a claim of an external reviewer. Both themes matter equally.

## Evidence and method

Before editing, visually inspected all eight images under
`screenshots/site/final/` at baseline commit `520373c`: full-page desktop and
mobile homepages plus desktop and mobile Getting started views, dark and light.
Inspected the hero again at native size, since a 5,000px page squeezed into one
image hides poor readability. The baseline images remain recoverable from Git.
Read `PLAN.md` and `design/ART-CHECKLIST.md`; `design/` stayed read-only.

After fixing, reran `scripts/capture-site.mjs screenshots/site/final` and inspected
the new captures. The final set includes full pages, first viewports, documentation,
and both states of the mobile proof. The desktop viewport is 1440×900; mobile is
390×900, all at device scale 1. Full-page images are not evidence that something
is visible without scrolling; the separate viewport captures establish that.

## Pre-fix verdict

**Desktop 7.8/10. Mobile 7.0/10.** The real artwork and honest product boundaries
are good. The page is clean. Clean alone is not exceptional art direction.

1. **The opening wastes its best asset.** At 1440×900, the old centered headline,
   buttons and boilerplate consumed most of the first screen. The editor began
   around y=542; the workshop was cut off by the fold. On a phone, the screenshot
   began around y=509, while the actual agent activity sat below it. An editor
   should earn attention with its workspace, not require a scroll to reveal it.
2. **The central proof asks visitors to play spot-the-difference on postage
   stamps.** Two mobile images were approximately 165px wide. A three-voxel change
   was barely legible. Desktop could compare the images, but the small change is
   still modest evidence for such a big positioning claim. Provenance is useful;
   it cannot rescue an unreadable demonstration.
3. **The mobile session crop looks unfinished.** The bottom cuts into the next
   “Captured views” row. It reads like accidental overflow rather than a carefully
   chosen excerpt. Separate image captions also take more vertical room than
   their information deserves.
4. **Docs look like a nicely typeset README.** The mobile view spends its first
   screen on orientation and build prerequisites. A reader arriving to connect
   an agent has no local shortcut to that task. Detached copy buttons sit below
   code blocks; language and action lack a clear shared container.
5. **The page loses energy after the proof.** Repeated feature rows, format cards,
   a large browser invitation, and another open-source invitation feel formulaic.
   The mobile “Create at your desk” headline also sounds like dismissal at the
   point where the site should offer a useful next step.
6. **Tiny type undermines the otherwise sound system.** Several eyebrow and
   proof labels fall to 10–11px on phones. Light mode exposes that fragility even
   more than dark. A premium presentation should not depend on zooming to read
   its evidence labels.

## Changes made

- Recomposed the wide-screen hero as a left-hand introduction beside the full
  native workspace. The whole window, artwork, session panel and caption now fit
  in the 900px viewport. The headline remains the agents-first product promise;
  Connect your agent remains the primary action.
- Reduced mobile button/caption overhead and tightened the session excerpt to
  the complete committed edit row. The artwork now fits in the first viewport.
  The screenshot is still a real recorded native session, clearly labeled r8.
- Kept simultaneous before/after images on desktop; added native Before / After
  radio controls on mobile. The selected render is now 342px wide, roughly twice
  its former width, with no camera movement between states. It works with pointer,
  arrow keys and JavaScript disabled. Actual PNGs and provenance are unchanged;
  the headless r1 recording is still distinguished from the native r8 screenshot.
- Raised marketing metadata to 12px and made the mobile closing invitation
  “Your next little world,” leading to the setup guide with the desktop editing
  requirement still explicit.
- Added three direct setup routes: Build the app, Connect MCP, Your first model.
  Code blocks now have a language/action bar, 44px copy target and visible focus.
  Without JavaScript, code and all task links remain readable and operable.
- Extended capture coverage to first viewports and both mobile comparison states.
  Hidden lazy images are explicitly loaded during capture; otherwise decoding a
  hidden Before panel can wait indefinitely. Added functional coverage for the
  comparison and the no-JavaScript setup shortcut.

## Post-fix verdict

**Desktop 8.6/10. Mobile 8.2/10.** These are scores of the recaptured result,
not points awarded for implementing the checklist. Neither is a 9 yet.

| Surface | Before /10 | After /10 | Assessment of the new captures |
| --- | --- | --- | --- |
| Desktop homepage, dark + light | 7.8 | 8.6 | A much stronger first screen: complete product beside a clear proposition; restrained surfaces and convincing artwork. Native chrome is still small at this scale, though the full-size source is linked. |
| Mobile homepage, dark + light | 7.0 | 8.2 | The artwork fits, session clipping is gone, and the change can actually be inspected. The long single-column lower half still feels repetitive. |
| Desktop Getting started | 7.4 | 8.0 | Clear task entry points and code ownership; navigation and reading measure are coherent. Still lacks a full section TOC and search. |
| Mobile Getting started | 6.8 | 7.7 | Readers can jump to the task immediately. The shortcuts consume space; this is a useful navigation tradeoff, not a shorter article. |

Dark and light receive the same scores: neither has an unaddressed contrast or
layout defect in the inspected captures. This is not a screen-reader audit or
an automated accessibility certification.

What still keeps this below 9:

- The recorded edit is only three voxels. A substantial, reproducible agent build
  with visible intermediate decisions would make the primary promise more
  convincing. A screenshot cannot stand in for that richer demonstration.
- The middle and lower mobile sections remain a long sequence of similarly
  weighted blocks. They need more editorial selection and less repeated closing
  copy, not another ornamental card or gradient.
- The art is one strong workshop fixture, not a diverse finished showcase. A
  curated gallery and real process recordings belong to their own production
  milestone; this review does not pretend they exist.
- Docs need search and a richer section-level navigation system to reach the
  reference products' standard. The new setup shortcuts solve one concrete
  navigation problem, not the entire documentation specification.

## Verification

- `scripts/build-site.sh`: remapped Wasm build and site generation passed;
  `scripts/build-site.sh --skip-web` regenerated the edited documentation.
- `python3 scripts/check-site.py`: 7 pages, 224 local links/assets, all 38 tool
  schemas, skill download parity and editor bundle parity passed.
- `node scripts/capture-site.mjs screenshots/site/final`: 16 captures; homepage
  and all five docs routes fit 320, 390, 768, 1024 and 1440px in both themes.
  Theme persistence, decoded assets, clipboard contents, skip link, reduced
  motion, comparison controls, no-JavaScript navigation and no external homepage
  requests passed. Browser version and individual captures are in
  `screenshots/site/final/verification.json`.
- `cargo build --locked --workspace`: passed, including the native Linux GPUI app.
- `cargo test --locked --workspace`: 112 passed, including the published tutorial.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- Rebuilt `/app/?backend=webgl` opened Autumn Workshop under headless Chromium
  with SwiftShader, produced a 339,615-byte rendered screenshot, and emitted no
  page errors. The screenshot was visually checked as well.

No design files were changed. No deployment, push or remote creation occurred.
