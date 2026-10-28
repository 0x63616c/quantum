# Quantum design

Quantum's brand choices here and [the art checklist](design/ART-CHECKLIST.md),
including its never-again list, override impeccable defaults. Pure black and
Geist are intentional; do not flag or replace them. Aim for Vercel/Stripe
restraint: no clutter, one restrained accent, artwork supplies the colour.
Both light and dark themes are first-class. Require WCAG AA: 4.5:1 for normal
text, 3:1 for large text and UI components. Metadata is at least 12px.
Destructive actions never become primary actions.

## Marketing site

[site/assets/site.css](site/assets/site.css) is the token source of truth.
Use pure black `#000` dark and pure white `#fff` light base surfaces, crisp 1px
hairlines and locally bundled Geist / Geist Mono. No cookies or remote fonts.

| Variable | Dark (`:root`) | Light (`html[data-theme=light]`) |
| --- | --- | --- |
| `--bg` | `#000` | `#fff` |
| `--fg` | `#fafafa` | `#171717` |
| `--muted` | `#a3a3a3` | `#525252` |
| `--line` | `#242424` | `#e5e5e5` |
| `--soft` | `#121212` | `#f7f7f7` |
| `--accent` | `#93c5fd` | `#1d4ed8` |
| `--on-accent` | `#000` | `#fff` |

The root also defines `--clear: transparent`, `--current: currentColor`, and
workshop artwork swatches: slate `#375966`, mist `#78979a`, moss `#778d57`,
wood `#9e7750`, amber `#f0a84d`, cream `#f7dfae`. These are artwork colours,
not additional interface accents. `--font-body` uses Geist, Geist Fallback,
system-ui, sans-serif; `--font-code` uses Geist Mono, Geist Mono Fallback,
monospace. Font faces load local WOFF2 assets; metric-matched fallbacks are local.

The CSS is a cascade, not a single scale declaration. Body text is 16px/1.6;
metadata resolves to 12px, navigation and controls use 13–14px, supporting copy
uses 14–18px. The wide hero is 56px/1.06; mobile hero uses
`clamp(34px, 11vw, 44px)`; the base hero rule is `clamp(44px, 6.3vw, 82px)`.
Section headings use 48px (36px mobile), documentation headings 46px/28px/20px
(36px/26px/18px mobile), and download headings `clamp(36px, 5vw, 56px)`.
Other existing display and illustration sizes include 19–23px, 24px, 26px,
28px, 30px, 32px and 38px. Preserve the final responsive rules over earlier
10–11px declarations; do not introduce metadata below 12px.

Controls and code examples use 6px radii; screenshot and panel frames use 8px.
Existing nested swatches/segments use 4px and docs navigation/copy controls 5px.
Keep the main 6–8px character. Spacing uses 4/8px increments prominently:
8, 12, 16, 20, 24, 32, 40, 48, 64 and 80px, with existing 28/36px gaps and
section rules up to 112px. Shell padding is 24px; wide marketing/docs shells
are 1344px, wide hero columns have a 48px gap. Buttons are generally 48px tall;
interactive links and small controls have a 44px minimum target. Focus uses a
2px accent outline with 4px offset, with local inset exceptions.

## Native app and future SwiftUI/macOS surfaces

The viewport is the hero. Follow [ART-CHECKLIST.md](design/ART-CHECKLIST.md),
[quantum-ui tokens](crates/quantum-ui/src/tokens.rs),
[component documentation](crates/quantum-ui/README.md) and
[UI rules](crates/quantum-ui/AGENTS.md). `crates/quantum-ui/DESIGN.md` is
authoritative for app tokens once it exists. Do not copy site values into app
controls or restate app token values here. The same brand, accessibility,
neutral chrome and decluttering rules apply to future SwiftUI/macOS UI.
Use impeccable's `adapt.native.md`, `audit.native.md` and `ios.md` references;
run critique on native screenshots rather than a browser DOM audit.
