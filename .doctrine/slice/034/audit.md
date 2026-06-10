# Audit SL-034 — doctrine-partner subset + route comprehension/posture provision

**Mode:** conformance (post-implementation, tied to the slice).
**Commits:** `2317e5e` (PHASE-01), `c846d69` (PHASE-02), on `design a882f6f` / `plan 1bd68e8`.
**Date:** 2026-06-10.

## Evidence

| # | Check | Result |
|---|---|---|
| 1 | `discover_excludes_marketplace_only_domains` (extended) | ok (1 passed) |
| 2 | full bin suite | ok (754 passed, 0 failed) |
| 3 | `cargo clippy` (plain) | 0 warnings |
| 4 | `just check` | green (build + lint + all suites) |
| 5 | `plugins/partner/` net diff | absent |
| 6 | `doctrine-partner/skills/{pair,walkthrough}` symlinks | resolve to canonical SKILL.md |
| 7 | `doctrine skills list` | `pair` + `walkthrough` present; no `doctrine-partner` domain |
| 8 | `boot.md` after `doctrine boot` | comprehension row (:16) + posture line (:30-32) project through |

## Findings (against design.md / criteria)

### F-1 — Skill relocation to canonical domain (EX-1) — **aligned**
Expected (§5.1): `pair`/`walkthrough` SKILL source under `plugins/doctrine/skills/`.
Observed: `git mv` recorded both as renames (R); skills resolve under domain
`doctrine`. ✔

### F-2 — doctrine-partner subset structure (EX-2, INV) — **aligned**
Expected (§5.1/§5.2d/e): `plugins/doctrine-partner/` = plugin.json + README +
`skills/<id>` symlinks `../../doctrine/skills/<id>`. Observed: symlinks staged as
mode `120000`, byte-identical form to doctrine-memory; both resolve. ✔

### F-3 — Catalog integrity / discovery exclusion (EX-4, VT-1, INV-1/2) — **aligned**
Expected (§5.2a): `PARTNER_SUBSET_DOMAIN ∈ MARKETPLACE_ONLY_DOMAINS`; exactly one
entry per skill id. Observed: RED first surfaced the *real* failure mode — `discover()`
`Err`'d with `Duplicate skill id 'pair' across domains` (symlink double-emit), not
a mere tidy-up; GREEN after the const. Test asserts no `doctrine-partner` domain +
`pair`/`walkthrough` under `doctrine`. The "(sole)" doc comment corrected. ✔

### F-4 — Interim reset (EX-3, R3) — **aligned**
Expected (§2): net diff shows no `plugins/partner/`; marketplace `partner` →
`doctrine-partner`. Observed: `partner/` removed (renames + delete + empty-dir
cleanup); marketplace entry swapped. ✔

### F-5 — Route provision, both surfaces (EX-1/2, VA-1, D4) — **aligned**
Expected (§5.2b/c): comprehension-exit row after `/preflight` + posture line, on
both `install/routing-process.md` and `route/SKILL.md`. Observed: both carry the
row + posture; boot snapshot projects them (boot.md:16,:30-32). Surfaces agree
(phrasing differs — table cell vs numbered item — content matches). ✔

### F-6 — README accuracy, OQ-1 (b)+ (EX-3, VH-1) — **aligned**
Expected (§5.2e/f, §6): both subset READMEs state accurate source-vs-distribution
wording; no "duplicated / byte-identical copies" / "update both copies". Observed:
doctrine-partner authored accurate (PHASE-01); doctrine-memory corrected to match
(PHASE-02). Both read truthfully vs the on-disk symlinks. Retires the former
reconcile-the-READMEs follow-up. ✔

### F-7 — Boot golden safety (VT-1) — **aligned**
Expected (§2/§10): routing section asserted by presence only, so row additions are
golden-safe. Observed: boot suite green unchanged; behaviour-preservation gate
holds (only `discover_excludes…` deliberately extended). ✔

## Disposition summary

All findings **aligned**. No fix-now, no design-was-wrong, no tolerated drift.

## Follow-ups (harvested, not blocking)

- **Pre-existing boot drift** — `Active Policies` / `Active Standards` sections
  unpopulated in the snapshot (R4). Predates this slice; deliberately not folded
  in to keep the boot regen reviewable. Track separately.
- **`--only-partner` flag** — YAGNI per D3; revisit only if standalone-partner
  installs become common.

## Notes for /close

Rollup shows `2/2 ⚠` — the `⚠` is the SL-009 divergence between the complete
rollup and the hand-edited `status = proposed`. Closure reconciles status
`proposed → done`. `src/memory.rs` (SL-035) is already committed (`fea2119`); the
tree is clean of cross-slice residue.
