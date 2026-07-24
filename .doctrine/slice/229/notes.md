# SL-229 — working notes

## Harvest

fresh-as-of: `ready` (pre-PHASE-01) @ `de03c23f`

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
  `[plan; db8e41f5]`.

**Learned** (routed to sinks)
- Pre-existing research-scratch seams (SL-055 gitignore, SL-116
  `Tier::Research`, doctor skip) → design D1 rationale + research.md
  § Cross-thread.
- VT keyword-mandate selection pitfalls; unmarked-row falsification at plan
  re-grep (scout's main.rs parse-test claim) → case-notes `[plan; db8e41f5]`.

**Open**
- A1 (prose runner deferral suffices for both arms), A2 (fixed baseline path
  set incl. plan.toml is enough for v1), R1 (advisory hooks may
  under-deliver; escalation = ADR, not skill tweak) — all in design.md § Open
  questions / risks. No DEC/QUE/ASM entities minted; design.md is the record.
