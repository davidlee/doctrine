# SL-200 — Implementation notes

2026-07-04 · commit `98e1b09a`

## What was done

Prepended `mem.` prefix to all 138 body wikilinks (32 unique targets) across
32 `.md` files in the shipped memory corpus. All now resolve via
`extract_wikilinks`.

## Method

Single scripted `sed` pass using the dotted `type.domain.subject` form as
discriminator — this automatically excluded `[[status_delta]]` and
`[[evidence_ref]]` (TOML field references, underscore_case, not dotted).
See `preflight.md` for full target list and analysis.

## Verification

| Check | Result |
|---|---|
| Prefix-less memory keys remain | 0 |
| `mem.`-prefixed body wikilinks | 138 (matches preflight count) |
| `[[status_delta]]` / `[[evidence_ref]]` intact | 2 each, unchanged |
| `resolve-links` | 326 → 451 (+125) |
| `backlinks` (lifecycle-start) | 9 wikilink inbound |
| `cargo build` | clean |
| `doctrine memory sync` | 31 unchanged, no drift |
| `doctrine check commit` | all green (5583 tests, clippy zero) |

## Editorial finding

`mem.signpost.doctrine.{rec,rfc,concept-map}` ship as key-named real dirs
(not symlinks to uid dirs). `gather_assets` in `src/corpus.rs:311` skips
non-uid dirs, so these three are never indexed — they appear as danglers in
`resolve-links` despite existing on disk. Pre-existing corpus hygiene issue,
out of scope for SL-200. Should be captured as a backlog item.

## Follow-up

- **Backlog item**: Register `rec`, `rfc`, `concept-map` signposts properly
  (create uid-named dirs or symlink them, or fix `gather_assets` to admit
  key-named dirs).
- **ISS-213** + **ISS-214** (already backlogged): map/catalog ingest of body
  wikilinks, now unblocked by corrected keys.
- **SL-201**: onboarding command/URL, unaffected.
