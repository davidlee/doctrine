# SL-019 audit — product-spec corpus backfill

Mode: **conformance** (post-implementation, tied to SL-019). Reconciled against
`design.md`, `plan.toml`, `slice-019.md`, and the SL-019 `inquisition.md`. No
spec/validate registry exists for slices themselves — this is hand-authored.

Evidence gathered against the built binary
`/home/david/.cargo/doctrine-target-jail/debug/doctrine` (resolved via `cargo
metadata target_directory`, never `./target`). Corpus committed as `7fc3aad`;
template reword `dd95d3b`; skill rework `77c9795`.

## Evidence

| # | Expected (cite) | Observed | Evidence | Disposition |
|---|---|---|---|---|
| 1 | Corpus-wide FK integrity: no dangling member FKs / dup labels / orphan reqs (PHASE-05 EX-1, VT-1; design §7) | `spec validate` → `validate: corpus clean` | ran post-commit | **aligned** |
| 2 | One PRD per confirmed capability; full taxonomy coverage (PHASE-04 EX-1, PHASE-05 EX-3; design §4) | 8 product specs, 6 members each, one per capability (slices, specifications, skills, memory, reservation-leasing, install, boot-governance, adrs); 48 REQ entities REQ-001..048, contiguous 8×6 | `spec list` | **aligned** |
| 3 | §4 prose carries constraints/invariants only — NO FR-/NF- rows (D-1; PHASE-04 VT-2) | zero FR/NF requirement rows in any of the 8 spec bodies | `grep -rnE '^\s*-?\s*(FR\|NF\|NFR)-[0-9]' …/spec-*.md` → none | **aligned** |
| 4 | §7 no per-requirement coverage table keyed on mobile labels; durable REQ-NNN refs only (CHARGE IX; design §10) | no coverage table; §7 prose describes approach, references REQ-NNN | grep + read of PRD-001/005 §7 | **aligned** |
| 5 | Skill reworked: no `NFR-`, no prose-FR block, exemplar-driven (D-2; PHASE-03 EX-1/2, VT-1) | `grep NFR-` → none; `grep '### Functional Requirements'` → none | grep of canonical SKILL.md | **aligned** |
| 6 | Embed re-run before authoring: spec bodies show `## 1. Intent`, never `## Problem` (design §7a build gate; CHARGE I/II) | 0 specs contain `## Problem`; all carry the 8-section template | grep of corpus bodies | **aligned** |
| 7 | Storage rule: no taxonomy/source-map artifact committed under `doc/` or `slice/019/` (CHARGE VII; PHASE-05 EX-4, VT-3) | committed diff touches only `.doctrine/spec` + `.doctrine/requirement` + template + skill; `slice/019/` dir is untracked scaffolding | `git show --stat` of the 3 commits | **aligned** |
| 8 | Template §4 reworded + committed *before* any spec scaffolded (CHARGE V; PHASE-02 VT-3) | `dd95d3b` reworded §4 to "Constraints and invariants. (…REQ entities…)"; corpus committed later in `7fc3aad` | commit order | **aligned** |
| 9 | `just check` green (PHASE-05 EX-4; design §7) | EXIT=0 — fmt + clippy + full test suite + build all pass | `just check` (recipe: fmt lint test build) | **aligned** |
| 10 | Specs are durable *what/why*, not *how*; altitude holds (design §6, §9 "source skew" risk) | independent read of PRD-001 (exemplar) and PRD-005 (most mechanism-prone): need/value framing, observable behaviour, mechanism deferred to doc/*; no `forgettable`/`mkdir`/git internals leaking up | auditor read of `spec show` | **aligned** |

## Findings requiring disposition

### F-1 — PHASE-02 EX-3 human acceptance gate not exercised (autonomous substitution)

- **Expected:** plan PHASE-02 EX-3 — "User accepts PRD-Slices as the locked
  reference bar" — and the design §5 gate ("`spec show` reviewed and accepted as
  the bar before fan-out"). A human sign-off precedes fan-out.
- **Observed:** the run was autonomous via the backfill workflow. The User
  explicitly waived the exemplar gate for this run ("note: autonomous, no check
  gate for exemplar"). The workflow substituted a mechanical+adversarial gate:
  `spec validate` clean + §4-clean + per-spec adversarial review with one repair
  pass. The exemplar (PRD-001) was authored first and used as the shape the
  fan-out mirrored.
- **Evidence:** handover "Autonomous-mode deviation" section; User instruction
  this session; finding #10 above (auditor independently judged PRD-001 altitude
  sound).
- **Disposition:** **tolerated drift.** The human-acceptance criterion was
  consciously waived by the User in exchange for the mechanical+adversarial gate.
  No rework owed. PRD-001 remains available for post-hoc User review; if the User
  rejects the exemplar shape later, that is a new revision, not an SL-019 defect.

## Handoff to /close

- All exit criteria for PHASE-01..05 met except the consciously-waived F-1.
- **Lifecycle divergence (⚠):** `slice list` shows `019 proposed ⚠ 5/5` — the
  hand-edited `slice-019.toml` status (`proposed`) lags the 5/5 phase rollup.
  Reconciling the slice status is `/close`'s job (no lifecycle-transition verb
  exists yet — hand-edit; see CLAUDE.md known-gaps).
- **Memory harvest (plan §11 post-slice):** record the rust-embed re-embed
  footgun (CHARGE II) — a lone `install/` template edit is invisible until the
  embedding crate is forced to recompile (`touch src/install.rs && cargo build`),
  because there is no `build.rs`/`rerun-if-changed` and rust-embed uses
  `debug-embed`. Companion: resolve the binary via `cargo metadata`, not
  `./target` (CHARGE I). Stray `target-jail/` + a literal-path target dir were
  found in the tree at session start — that footgun is real and recurring.
