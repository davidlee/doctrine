# Notes SL-243: Spec anchor map

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-02 · stage:design/exploring run dr-019fc13a rev 13 · f6a51758

### Produced

- SL-243 scope (this slice) — design commitments settled pre-slice, see `slice-243.md`
- CHR-052 — SPEC-002's nine `[[source]]` anchors + its `## Source anchors` prose
- IMP-381 — spec-coverage census; map at `.doctrine/state/imp-381-coverage-map-criterion-lineage.md`
- **DEC-111** — the verb reads the corpus in-process, not through doctrine's own
  JSON contract; disposes `inq-4` and carries the test obligation that comes with it
- **DEC-112** — the report's `--json` is a raw struct outside SPEC-013's list
  spine, no `CommonListArgs` flatten; disposes `inq-5`
- **IMP-383** — the deferred half of `inq-5`: a self-identifying JSON head for
  non-list report verbs, retrofitted to `graph`, with the SPEC-013 amendment
  that would state the convention. Spawned by DEC-112
- **DEC-113** — the report states adapter provenance, and the adapter table
  denies unknown fields; disposes `inq-8`. A partial guard, labelled as one
- **IMP-384** — `deny_unknown_fields` for `doctrine.toml` generally. Not an
  attribute: `[priority]` and `[reservation]` are read out of band and are not
  fields of `DoctrineToml`, so the central struct would reject doctrine's own
  config today. Spawned by DEC-113
- **Pre-design research round** — `research/research.md` + `research/raw/` (five
  threads). **Runtime tier, gitignored, disposable.** Its durable residue is
  DEC-111 and the `## Design surface triage` section below; do not treat the
  artefact itself as surviving.
- **`## Design surface triage`** (below, committed) — the round's findings in
  summary, so the disposable artefact is not the only copy

### Learned

- mem.pattern.spec.read-anchors-via-json-not-grep — the read path this slice
  rides, and the two ways a raw TOML grep inflates the count
- mem.pattern.lint.new-workspace-member-cargo-metadata — the new-crate lint
  checklist O3 inherits if `just lint` is widened
- mem.fact.doctrine.agents-skill-mirror-is-published-source — O5 edits the master
  under `plugins/`; `.agents/` is published-sourced derived state
- Baseline figures, via that read path: 48 specs · 81 anchors · 0 non-resolving ·
  29,310 non-test `src/` loc (27%) dark · largest dark `src/review.rs` @ 2,824
- CHR-051 §3 re-verified live: `pi-scout` resolves `deepseek-v4-pro` and
  `pi-research` resolves `deepseek-v4-flash` — inverted from `CLAUDE.md`. Check
  before routing any refresh research thread.

### Open

Held on design run **dr-019fc13a** — read them with `doctrine design resume 243`
rather than from a copy here. What the run cannot represent, and so lives here:

- `inq-1`, `inq-2`, `inq-3` are deferred **to a `/spec-coverage-assessment`
  pass**, not parked indefinitely. A lifecycle move carries no reason field, so
  this note is the only record of where they route. See `## Routing` below.
- `inq-6`, `inq-7` and `inq-9` are the open engineering questions; the triage
  section states each with the evidence that bears on it. `inq-5` is disposed by
  DEC-112 (triage entry Q-b) and `inq-8` by DEC-113 (Q-e).
- Obligations the plan must carry, from those dispositions: the black-box golden
  SPEC-013 `REQ-204` bills the verb (DEC-112), and the provenance block as part
  of the report struct rather than a wrapper (DEC-113).

## Design surface triage
<!-- explore.triage, design run dr-019fc13a rev 5. Evidence: research/research.md -->
recorded: 2026-08-02 · stage:design/exploring

Everything below is sourced from `.doctrine/slice/243/research/research.md`,
which carries the citations and the ✓ verification marks. Ids only here.

### Routing: the governance-placement cluster is deferred

`inq-1`, `inq-2` and `inq-3` are **deferred on the run** (rev 8), not dropped.
They are one question in three parts — where governance for this capability
lives — and `/spec-coverage-assessment` is the skill built for exactly that
judgement (what is already governed, what is dark, where a new spec's boundary
falls by product altitude and C4 level). Running it inside this design session
would cost more context than it is worth, so it happens as its own pass before
the run can lock.

The research round's recommendations below are that pass's **input**, not its
conclusion — offered so the skill starts from evidence rather than cold.

### The three carried questions, and what the evidence recommends

1. **Spec home** → new component spec, parent SPEC-006, descends from PRD-012,
   C4 component, sibling of SPEC-017. The rejected alternative (amend SPEC-017)
   and the strongest argument for it are recorded in research.md so design can
   close it rather than reopen it.
2. **PRD-012 requirements** → the gap is real and narrow. REQ-085 (anchor shape)
   and REQ-088 (hand/import convergence) already exist and must not be
   re-required; nothing across REQ-081..089 requires a *report*.
3. **Identifier-form convention** → SPEC-017 owns it (it defines the field
   semantics at REQ-232, and IMP-316 names it as the site). This slice cites it;
   IMP-316 enforces it.

### New questions the round opened — design must settle these

- **Q-a. In-process read vs the JSON contract.** The scope's phrasing implies
  `spec list --json` carries anchors; ✓ it does not. The verb should ride
  `build_registry`/`scan_ids` in-process. Design must record that, and record
  that the two paths must agree (the 48/81 baseline is the check).
- **Q-b. Envelope or raw struct for `--json`.** SPEC-013 owns `json_envelope` as
  the invariant for numbered-entity `list` verbs; `graph` lawfully opts out.
  This report is closer to `graph`. Must be argued against SPEC-013, not chosen
  by taste — and with it, whether `spec anchors` flattens `CommonListArgs` (it
  should not).
- **Q-c. One argv runner or two.** `coverage_verify::run_argv` has the right
  capture/timeout behaviour but a VT-shaped return type; `verify::run_suite` has
  the wrong stdio posture. Extract a shared helper or justify a second — this
  decides module boundaries, so it is design-level, not plan-level.
- **Q-d. Lint the adapter crate, or exempt it.** ✓ A member absent from the
  root's `[dependencies]` is never clippy-linted by `just lint`. Widening to
  `--workspace` costs the new-crate lint checklist
  (`cargo_common_metadata` + pedantic doc lints).
- **Q-e. How the report says "no adapter declared".** ✓ Config parsing is
  tolerant (no `deny_unknown_fields`), so a mistyped table name is
  indistinguishable from absence, and "absent ⇒ owned no-op" would then report
  the whole corpus as uninventoried. The output must distinguish the two.
- **Q-f. Is d2 wanted at all?** No d2 or mermaid renderer exists for the graph
  projection; DOT does, and is already deterministic. If d2 is kept, it must not
  fork `dot_escape` a third time.

### Constraining governance (beyond what the scope already names)

- **SPEC-013 (CLI surface)** — missed by the research thread, surfaced by the
  canon pass. Owns the verb grammar, the `json_envelope` invariant, the pure
  `src/listing.rs` leaf, and the conformance-matrix + black-box-golden regime.
  A new verb owes a golden.
- **SPEC-010 (Skills distribution)** — O5 edits the master at
  `plugins/doctrine/skills/spec-coverage-assessment/SKILL.md:67`. `.agents/` is
  published-sourced derived state and is not the edit site.
- **ADR-008** — `cargo install` is structurally impossible in-jail, so the
  project-declared argv (`cargo run -p <adapter>`) is the only route that works
  here, not merely the convenient one.
- **SPEC-017 REQ-236** — states anchor liveness is *not* checked. This slice
  checks it. Resolves only while report-only and non-gating; design must say so
  explicitly, and a future gating slice owes a REV.
- **POL-001** — constrains this slice's own new prose (requirement titles, help
  text, code comments): avoid "load-bearing" and the tired physical metaphors.
- **ADR-013** — O4 authors requirements directly via `spec req add`; the REV
  route is for corrective reconciliation, not for this authoring. Design should
  state this rather than leave it implicit.

### Risks

- **R1 — the baseline may not reproduce.** 48/81/0/29,310 came from a spike, not
  the shipped path. Disagreement means one is wrong, and finding out which is
  part of the work, not a surprise.
- **R2 — the adapter is undistributable by both shipped paths** (nix emits only
  the `doctrine` binary; `cargo install` installs only the root package). Fine
  for dogfooding; becomes real if the "publish as reference implementation"
  follow-up is taken, which ADR-019 makes a separate explicit decision.

### Assumptions carried

- The `language` key vocabulary is the existing `[[source]]` `language` values;
  no second vocabulary is minted (STD-001).
- The report stays read-only and non-gating for the whole slice (scope
  commitment; also what keeps REQ-236 from needing a REV).
