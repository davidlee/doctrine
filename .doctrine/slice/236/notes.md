# Notes SL-236: Worker-guard honours explicit project root

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-28 · design (pre-plan) · 90028c846

### Produced

- SL-236 — scope, `design.md`, `research/`, 7 `scope-relevant` + 32
  `design-target` selectors
- DEC-093 — fix axis locked (global `-p` over scattered write-path checks)
- `mem.fact.clap.global-and-local-arg-share-id`
  (`mem_019fa8f1ade17f31822539fa80d778f4`)
- Corrections landed on ISS-028 and ISS-267 (their recorded fix directions were
  wrong / mostly-subsumed respectively)

### Learned

- `mem.fact.clap.global-and-local-arg-share-id` — no collision; migration is
  stageable, but completeness rests on a source scan, not the compiler
- ISS-267 is ~2/3 subsumed by ISS-028: 19 of 29 files pass `-p` and dissolve
  with the guard fix; 10 are genuine residual
- `resolve_mode` requires `is_linked && marker_present` — a marker file in a bare
  tempdir is never refused (drives the VT fixture requirement)
- Research-agent reliability is uneven; see `research/raw/VERIFICATION.md`

### Open

- **DEC-093** — settled, but its scale rationale was corrected post-spike
- **F-4** (`design.md` §10) — `worktree fork`'s `-p` is the *source* root, so the
  guard will evaluate that tree. Accepted as a conscious assumption; no VT covers
  it
- **F-5** — VT-s must count clap attribute declarations, not raw string hits
- **OQ-1** — exact golden count still unmeasured (bounded tidy-up, not unknown)
- **Decision pending** — external adversarial review vs straight to `/plan`
