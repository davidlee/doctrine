# SL-229 — working notes

## Harvest

fresh-as-of: `PHASE-02 completed` @ `14a9f9f8`

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
- **PHASE-02** (`14a9f9f8`, path-limited to 3 files): `plugins/doctrine/
  skills/research/SKILL.md` new master — trigger-form description; five
  contract items (Shape / Citation forms / Verification discipline / Runner
  deferral + raw capture / Prompt duties); inverse pointers single-sourced;
  ADR-005 restate line held (verb named + when, no flag syntax / baseline
  format); D1 storage mandated (direct in-slice, gitignored) — dogfood's
  stale state-dir prose NOT propagated. `install/governance.md` § Research
  agents socket stub; `install/glossary.md` `research/` contents line
  (research.md, raw/, baseline.toml). Authoring-only per plan — embed ritual
  deferred to PHASE-03. VT-1/2/3 ✓ (post `record-delta --commit 14a9f9f8`),
  VA-1 self-review confirmed; `check gate` clean.

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
  (PHASE-02: hazard avoided; skill mandates D1.)
- verify-vt attribution reads the source-delta registry — a docs-only phase
  still needs `slice record-delta <id> <PHASE> --commit <S>` or its VTs stay
  `Unattributable`. And that verdict's reason string says "keyword present"
  without evaluating keywords (`vtgate.rs:124`) — misleading at red-phase →
  case-notes `[execute; SL-229-PHASE-02-b7c]`.
- `mem.pattern.skills.yaml-frontmatter-colons` sweep recipe false-positived
  (its own `FILENAME": "` separator matched the `: ` grep); memory body
  corrected in place → case-notes ditto.

**Open**
- A1 (prose runner deferral suffices for both arms), A2 (fixed baseline path
  set incl. plan.toml is enough for v1), R1 (advisory hooks may
  under-deliver; escalation = ADR, not skill tweak) — all in design.md § Open
  questions / risks. No DEC/QUE/ASM entities minted; design.md is the record.
