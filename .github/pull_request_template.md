<!--
Title: Conventional Commit, e.g. `fix(mcp): reject oversized frames`. See CONTRIBUTING.md.
Keep the sections and their order. Write for someone who has not seen the issue.
UI changes: don't paste screenshots by hand. Run `cargo xtask screenshots --publish <PR#>`
(the "PR screenshots" CI job does it for UI paths) and it adds a "Screenshots" section
with a Before | After table above "Testing". Non-UI PRs get no Screenshots section.
Default captures: welcome, editor-dark, editor-light. For migrations, add one plain
column-zero line outside this comment and the generated section, for example:
  Screens: panels-* settings-general-*
Those selectors add to the defaults. Exact names and trailing * prefixes work;
no shell syntax or quotes. Select every affected surface in both themes.
Push or dispatch PR screenshots after changing the line; body edits alone do not
rerun captures. Dispatch's optional screens field overrides the extras for that run.
Only changed/new/removed images are pushed; unchanged screens are listed as text.
-->

Closes #

## Summary

<!-- Plain English, 1–3 sentences: what was wrong or missing, and what is true now. -->

## Changes

<!-- Bullets, most important first. Call out anything deliberately out of scope. -->

-

## Testing

<!-- Commands run and results. Name platforms you could not test. Perf: before/after numbers. -->

-

## Checklist

- [ ] One focus; linked issue's **Done when** items are all met
- [ ] Docs, schemas and fixtures updated where behavior changed
- [ ] No secrets, personal data or machine-specific paths
- [ ] UI only: Screenshots section generated and checked; design-reviewed
- [ ] UI only: colors, spacing, radii and type come from `voxelite-ui` tokens
- [ ] UI only: unchanged screens show no unintended differences
- [ ] UI only: before/after screenshots cover affected screens in both themes
- [ ] UI only: focus, hover and pressed states reviewed
- [ ] UI only: empty and loading states reviewed
- [ ] UI only: text truncation and long labels reviewed
- [ ] UI only: headless golden comparison passes; intentional rebaselines reviewed
