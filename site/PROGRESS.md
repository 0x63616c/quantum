# Site, docs and skills stage

Started 2026-10-25 from `9eb0d95`. Local work only; never deploy or push.
`design/` is read-only and excluded from commits.

## Deliverables

- [ ] Marketing site, explicit dark/light themes, real app screenshots.
- [ ] Rust-generated docs and live MCP tool catalog; format/build guides.
- [ ] Four portable agent skills and host installation notes.
- [ ] Playwright desktop 1440×900 and mobile 390px full-page captures, both themes.
- [ ] Three independent screenshot critique/fix/recapture rounds.
- [ ] Repository and reachable Git history personal-info scan.

## Review scores

Scores refer to the images reviewed before each round's fixes. Final captures
are separate evidence, not retroactively higher review scores.

| Round | Desktop /10 | Mobile /10 | Evidence | Fixes / recapture |
| --- | --- | --- | --- | --- |
| 1 | 6.5 | 5.5 | `docs/reviews/site-review-1.md` | Device disclosure, enlarged crop, native evidence, palette, links; `screenshots/site/round-2/` |

## Verified boundaries

The existing web bundle uses the shared Rust GPUI editor. Browser MCP pairing
is not implemented. Native attached MCP and standalone headless MCP are
available. macOS release packages and signed downloads are not available here.
No unverified download, hosting, benchmark or collaboration claims belong on
the site. Existing Rust toolchain: 1.99.1; default native Linux build works.
