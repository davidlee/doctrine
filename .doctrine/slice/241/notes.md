# Notes SL-241: Capsule spike rig

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-01 · plan locked, PHASE-01 sheet expanded · 2626072e

### Produced

- design.md written, internally reviewed, externally inquisited — no code yet
  (commits 9d0c852f, 3966256a, eedc26b8, 7fd55d0f)
- RV-323 — 14 findings, all disposed `fix-now`; **all 14 terminal, review done**
- minted: QUE-202 — capsule-model conflict admission path
- amended both tiers: DEC-099 (Amendment 2 + facet), QUE-201, ASM-007
- IDE-009 — third lint canary appended (wholly-empty `[facet]`)
- plan.toml + plan.md — six phases, build/evidence split (4b495982, 2216769f,
  cbce7876, 2dc35d4d); six runtime sheets materialised (2f521a25)
- lifecycle: proposed → design → plan → ready (2626072e); `ready` is ADR-009's
  approved-plan gate, taken on operator approval
- `.gitignore` added as a design-target selector (design § 9.1)
- PHASE-01 runtime sheet expanded — 9 tasks, gitignored, not a durable artifact
- gate: no `src/` change this unit; `doctrine validate` clean at each commit

### Learned

- mem.pattern.doctrine.amend-knowledge-both-tiers
- mem.pattern.doctrine.repair-overshoots-the-named-axis
- mem.pattern.doctrine.path-policy-shell-hardening (raiser-side, 3afa085e)
- CPT-001 — five numbered classes, one with a git-level sub-class
- **environment**: `nix` and `direnv` are ABSENT in the jail (`/nix/store`,
  `bwrap`, `node`, `npm`, `claude` present); `$HOME` writable, `~/capsules/`
  creatable with no new mount. Pinned as PHASE-02 EX-8 / PHASE-04 EX-4.
- **R2 decomposes into two questions**, not one — `worktree import` has two
  scope postures and `conformance --strict` covers only the selector predicate
  (PHASE-01 sheet § T7; rides
  mem.pattern.dispatch.import-scope-belt-omit-slice-for-coupled-paths)
- **the step-0 contamination guard is two-sided** — design § 5.4 protects step 0
  from CPT-001; authoring the light fixture's `interpret:` list is itself a
  trigger enumeration, so PHASE-01 EX-12 + PHASE-04 EX-9 protect the other side.
  Not in the design because the design does not schedule the work.

### Open

- QUE-200 — ingestion mechanism M-A vs M-B; the rig's whole point
- QUE-201 — declaration home; now ergonomics-only, gains a probe-evidence input
- QUE-202 — how the capsule model *admits* the second result; refusal proven,
  admission not designed
- ASM-007 — exhaustiveness carried, confidence low; falsified only by § 5.4
  step 0's independent enumeration, never by fixture rows
- CON-004 — landed state append-only · CON-005 — threat-model fence
- DEC-099/101/102/103/104 — settled, carried
- OQ-1 — evidence-log storage tier; v0 ruling in slice § Risks
- R2 — open, and PHASE-01 T7 is what settles it. A genuine divergence is a
  `/consult`, never a `src/` change (slice non-goal)
- no `research.md` exists — the pre-design research round was never run; the
  drift advisory reports an absent artifact, deliberately not restamped
  (rationale in plan.md § Notes)

## R2 — SETTLED (PHASE-01 T7, EX-6, VA-2, VA-3)

Probed 2026-08-01 by `scripts/spike-capsule/control/probe-r2.sh`, re-runnable,
against a disposable clone of the light fixture. Nine rows, every expectation
declared before the run and asserted; results at
`$SPIKE_CAPSULE_ROOT/probes/r2/results.tsv`.

**R2 was two questions, not one.** `worktree import` has two scope postures —
with `--slice N` it runs the selector predicate, without it only the R-5 belt —
and `slice conformance --against … --strict` covers the selector predicate only.

### R2a — selector agreement: YES

`--strict` reaches the belt's scope-leg verdict on every edge probed. This is
not merely predicted from shared code: the belt's scope leg
(`src/worktree/import.rs:159`) and conformance both call
`crate::conformance::undeclared_paths`, so what could genuinely diverge is the
**path extraction** in front of it, and the two gathers differ — the belt uses
`diff --name-only` (`src/mcp_server/dispatch.rs:487`), conformance uses
`diff --name-status` folded by `actual_from_range` (`src/slice.rs:2894`). The
edges that would expose a difference do not:

| edge | delta | `--strict` | reading |
|---|---|---|---|
| 1 | `docs/notes.md` | refuse | ordinary undeclared path |
| 2 | `src/tax.ts` | clean | matches a `design-target` selector |
| 3 | `src/naïve.ts` | clean | **non-ASCII extracted verbatim** — the `core.quotePath=false` hardening holds through the `--name-status` fold |
| 4 | `src/money.ts` → `docs/money.ts` | refuse | **both legs visible** — `--no-renames` holds; the destination is the undeclared path and the source did not vanish |
| 7 | `A..A` | clean | empty range is clean, not an error |
| 8b | `src/audit.ts` | clean | edge 8's positive control |

### R2b — separation: `--strict` does NOT cover the prefix legs

**This is the load-bearing result.** `--strict` has no `.doctrine/`/`.claude/`
predicate at all; the belt runs those legs *before* the scope leg and
independently of it (`classify_import`, `import.rs:146-152`, and its own test
`classify_import_doctrine_path_is_doctrine_touch_even_when_undeclared`).

| edge | delta | `--strict` | belt prefix legs |
|---|---|---|---|
| 5 | `.doctrine/probe/payload.md` → `src/payload.md`, **both declared** | **clean** | `doctrine-touch` |
| 6 | `.doctrine/probe/kept.md` modified, **declared** | **clean** (reported *conformant*) | `doctrine-touch` |

**⇒ design § 5.2's conform LEG 3 (forbidden paths) is LOAD-BEARING. PHASE-03
must not skip it, and must not fold it into leg 2.** A pipeline that ran only
`--strict` would pass a `.doctrine/` touch whenever a selector happened to
declare that path — and this slice's own selectors (`.doctrine/rfc/025/**`,
`.doctrine/knowledge/**`) are exactly that shape.

**Edges 5 and 6 had to be run against a slice that DECLARES the `.doctrine/`
path.** Against the fixture's base slice (`src/**` only) a `.doctrine/` path is
undeclared, so `--strict` refuses it — for the wrong reason — and R2b scores
backwards as "`--strict` covers the prefix legs". The probe builds SL-002 with
`.doctrine/probe/**` declared for this reason alone. This correction was found
in the pre-execution re-check, not during the probe; the sheet's original edge
list would have walked into it.

### Edge 8 — the empty-selector asymmetry

`--strict` against a slice with **no** selectors refuses (everything is
undeclared); `classify_import` with empty selectors is a documented no-op
(`import.rs:668`). Divergent but **benign**: empty selectors is precisely the
belt's no-`--slice` posture, where the scope check is meant to be absent.

Note the help text's "refuses a clean diff when the registry is unavailable or
incomplete (fail-closed)" describes the *other* arm — `run_conformance`'s
`--against` path documents that it bypasses both the registry read and the
completeness ladder.

### No `/consult`, no `src/` change

STOP-1 covers a genuine `--strict`-vs-belt divergence. R2a **agrees**, and
R2b's separation is the *predicted* answer that makes leg 3 load-bearing — a
result, not a defect. Nothing in `src/` was touched.

## Forward compatibility

- **RFC-023 (executable plan gates / adversarial TDD)** — substantial revisions
  to plan gates are expected. Operator ruling 2026-08-01: adopt current plan
  machinery as-is for this slice; expect heavy revision to follow. Nothing in
  the four-stage capsule pipeline (CON-004, DEC-104) depends on plan-gate
  mechanics, so the revisions should land orthogonally to this rig.
