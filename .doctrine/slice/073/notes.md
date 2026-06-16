# SL-073 Implementation Notes

## PHASE-01 (complete — 2026-06-16)

**Commit:** `4e55b57` — feat(SL-073): PHASE-01 static shell

Static app shell delivered via dispatch funnel (worker in worktree fork). Replaced
`web/map/index.html` and `web/map/style.css` entirely. CSS Grid layout (280px sidebar
+ 1fr main), 19 kind colour custom properties with light/dark theme via
`@media (prefers-color-scheme: dark)`. All 12 kind-group filter checkboxes present,
depth selector (0-3), placeholder content in all panel regions. Vendor scripts
loaded in `<head>`. Kind pills use `var(--kind-*)` backgrounds.

**Verification:** `just check` green (1394 tests). Server verified at
`doctrine map serve --port 3001` — HTML and CSS served correctly from the
RustEmbed bundle. Visual acceptance (VA-1, VH-1) pending human review.

**Watch-outs:**
- Binary at `./target/debug/doctrine` is stale — CARGO_TARGET_DIR redirects to
  `/home/david/.cargo/doctrine-target-jail`. Always use the latter or run through
  `just check` which uses the correct path.
- HEADS UP: `doctrine` CLI is the source of truth for command shapes — the binary at
  `./target/debug/doctrine` may be stale if CARGO_TARGET_DIR is set.
- Layout uses CSS Grid, not flexbox. Verify later phases don't accidentally use
  flexbox for the main layout.
- Sidebar pills use `background: var(--kind-PREFIX)` with color: #fff — text colour
  assignment deferred per design (not AA tested).
