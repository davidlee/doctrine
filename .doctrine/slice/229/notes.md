# SL-229 — working notes

## Harvest

fresh-as-of: `PHASE-01 completed` @ `73b6c29d`

**Produced**
- `design.md` (D1–D6; internal adversarial pass integrated, 4 findings) —
  commits `1f673ca4`, `1399a647`, amendment in `de03c23f`.
- `plan.toml` + `plan.md` (3 phases, VT mandates) — `535ec090`, `de03c23f`.
- design-target selectors (12) recorded; cli.rs removed post-review (RV-297
  declared-but-undelivered class).
- Dogfood research artefact `research/` (research.md, baseline.toml, raw/) —
  gitignored by design; the design's evidence base.
- IDE-044 storage bullet corrected (D1 supersedes shaping) — `1f673ca4`.
- RFC-011 case-notes entries: `[slice/research dogfood; db8e41f5]`,
  `[plan; db8e41f5]`, `[SL-229-PHASE-01-a1f]`.
- **PHASE-01** (`73b6c29d`, path-limited to 6 source files): `src/research.rs`
  engine leaf (`mint`/`check` over `contentset::diff`; `baseline.toml` serde —
  `slice`/`date`/`[hashes]` over the fixed intent-doc set); `SliceCommand::
  Research { id, restamp }` + thin `run_research` (advisory, always exit 0,
  D6); guard `Write("slice research")`; `contentset::is_stale_against` removed
  (EX-2) with the bool relocated to `SetDrift::is_empty` (now `pub(crate)`,
  consumed by `run_research`); `layering.toml` `research = "leaf"`. VT-1/2/3 ✓,
  VA-1 confirmed; full suite green; `check gate` clean.

**Learned** (routed to sinks)
- Pre-existing research-scratch seams (SL-055 gitignore, SL-116
  `Tier::Research`, doctor skip) → design D1 rationale + research.md
  § Cross-thread.
- VT keyword-mandate selection pitfalls; unmarked-row falsification at plan
  re-grep (scout's main.rs parse-test claim) → case-notes `[plan; db8e41f5]`.
- New `mod` requires an ADR-001 `layering.toml [tiers]` entry or the
  `architecture_layering` gate fails `Unclassified` — not flagged in the
  handover terrain → case-notes `[SL-229-PHASE-01-a1f]`.
- Dogfood `research.md` still carries the *pre-design* storage wording
  (`state/research/` + symlink) that D1 **superseded** (direct
  `.doctrine/slice/NNN/research/`, gitignored in place) — a PHASE-02 authoring
  hazard: mandate the D1 shape, not the dogfood's stale storage claim.

**Open**
- A1 (prose runner deferral suffices for both arms), A2 (fixed baseline path
  set incl. plan.toml is enough for v1), R1 (advisory hooks may
  under-deliver; escalation = ADR, not skill tweak) — all in design.md § Open
  questions / risks. No DEC/QUE/ASM entities minted; design.md is the record.
