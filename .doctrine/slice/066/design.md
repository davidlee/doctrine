# Design SL-066: Revision entity — pending revise-intent + staged-delta vehicle

> **STATUS: DRAFT (2026-06-14).** Nine design questions resolved in the `/design`
> clarifying loop (see §8). Scope **C** (full structured delta payload) chosen by
> user (SL-044 is done — the deferral-to-the-writer argument is moot). Awaiting
> adversarial pass + lock. Reference forms: padded ids (`SL-066`, `ADR-013`,
> `REQ-NNN`, `SPEC-002`); doc-local refs bare (`D1`, `OQ-1`).

## 1. Design Problem

Two backlog items name the same gap from two angles — **IDE-003** (a vehicle to
*stage and approve* requirement/spec-prose deltas before they land) and
**IDE-010** (a *work-lifecycle* entity capturing pending intent to revise
governance, so work can `needs` it instead of `needs`-ing an evergreen doc). This
slice introduces **Revision** as **one** entity unifying both, and wires it into
the kind table, the relation contract, the dep/seq overlay, and the SL-044
reconcile-writer apply path.

The unification thesis: a Revision is one entity at two lifecycle lenses — born as
content-light pending intent (IDE-010: dependents anchor on it immediately),
accumulates staged deltas as it is worked (IDE-003), is approved on the ADR-009
conduct axis, then **applied** (deltas land) and settled.

## 2. Current State

- **No Revision kind.** `REV-` is a *deferred* spec subtype
  (`doc/spec-entity-spec.md`); `doc/entity-model.md` §Adjudication leaves its home
  **open**, nudging it change-side.
- **The kind seam** (mapped): `integrity::KINDS` (`src/integrity.rs:47`) — one
  `KindRef` row per kind (`kind`, `stem`, `state_dir`). `entity.rs` `Kind { dir,
  prefix, scaffold }` is **data, not a trait** — the engine is kind-blind. REC
  (`rec.rs`, status-less/immutable) and backlog (`backlog.rs` + `dep_seq.rs`,
  work-lifecycle) are the two templates.
- **Relations** (`src/relation.rs`): `RelationLabel` (15 variants) +
  `RELATION_RULES` (16 rows: `sources`, `label`, `inbound_name`, `target`, `tier`,
  `link`). Typed payload edges (members.toml, interactions.toml) are derived into
  reciprocity by the registry (`registry.rs`), not stored as Tier-1 rows.
- **Dep/seq** (`src/dep_seq.rs`): `needs: Vec<String>` + `after: Vec<AfterEdge>`.
  The **work-like predicate** (`main.rs:3050`) admits only `SL/ISS/IMP/CHR/RSK/IDE`
  as `needs`/`after` source or target.
- **Reconcile writer** (SL-044, **done**): `doctrine reconcile <REQ> --slice <SL>
  --move <accept|revise|redesign> [--to] [--note]`. Its `revise` move is a
  **structural `ReqStatus` write only** — "material spec/ADR *prose* edits go
  through a future Revision vehicle (IDE-003)" (SL-044 §5.1). The B·P1 setter
  `spec req status <REQ> --to <state>` (free any→any, edit-preserving) is the lone
  requirement-status write seam. REC composed atomically, one move per REC
  (`RecDoc`, `rec.rs:121`), `owning_slice: Option<String>`.
- **Spec composition** (SL-015): a requirement is a reserved **peer entity**
  `REQ-NNN`; membership is a spec-side `members.toml` edge (FK + mobile `FR-`/`NF-`
  label + order); `spec req add` is the only producer.

## 3. Forces & Constraints

- **Governance.** ADR-003 (explicit-authorship-not-derivation; names the revision
  artefact missing §11). ADR-009 (conduct axis; **"approval is not lifecycle"**).
  ADR-010 (relation modelling — unify contract, keep storage bespoke). ADR-004
  (relations outbound-only; reciprocity **derived**). ADR-001 (leaf←engine←command
  layering). IMP-047 ("a model change → the canon moves first").
- **Storage rule.** Structured/queried data in TOML; prose in MD; **no embedded
  YAML blocks** (rules out spec-driver's `supekku:revision.change@v1` block — it
  becomes a TOML payload table). Tooling **never parses prose structure / headings**.
- **No parallel implementation.** Apply orchestrates existing write seams (`spec
  req status`/`add`, membership re-link, `RecDoc` composition); never reimplements.
- **Pure/imperative split.** Move classification + `RecDoc` composition pure over
  resolved inputs; git/disk/clock in the shell.

## 4. Target Design

### 4.1 One kind, change axis — `REV-NNN`

A standalone **work-lifecycle** entity kind, peer to slice/REC on the change axis
(NOT a spec subtype, NOT a slice facet). Own folder `.doctrine/revision/`,
prefix `REV`. Wiring per `mem.pattern.install.authored-entity-wiring` (KINDS row,
manifest `[dirs].create`, gitignore negation).

Storage (authored tier):
- `revision-NNN.toml` — identity + spine + structured delta payload (§4.4).
- `revision-NNN.md` — prose rationale + free-text before/after excerpts for
  prose-body section edits.
- `NNN-slug` symlink alias (tracked, like siblings).

### 4.2 Lifecycle + approval

Work lifecycle (borrow backlog's, **not** slice's 9-state):
`proposed → started → done` (+ `abandoned` from any non-terminal). `done` = deltas
**applied** and landed. Dependents gate from `proposed` (the IDE-010 anchor works
from birth).

**Approval is a separate field** (hard canon: entity-model "approval is not
lifecycle"; ADR-009): `approval = none | requested | approved | rejected`,
orthogonal to `status`, gated by the ADR-009 conduct axis. **Default `gate`** — a
Revision writes authored governance/spec truth at `reconcile`'s altitude, which
ADR-009 already defaults to `gate`; consistency follows. Advisory in v1 (surfaced,
not enforced); a solo dev sets `actor=self` and self-approves.

### 4.3 Cardinality — multi-target payload rows + `primary` flag

The touched-entity set lives in a typed payload table (§4.4); **the rows ARE the
edges** (members.toml precedent). Exactly one row carries `primary = true` — the
**dep anchor** a `needs REV-NNN` resolves against and the headline intent. The
relation-index derives inbound `revises` reciprocity from these rows, **uniform
over primary + non-primary** — so `doctrine adr show ADR-X` lists *every* REV that
touches it. `primary` governs the dep anchor only, never visibility.

This merges IDE-010 (crisp single anchor) and IDE-003/spec-driver (multi-entity
batch) at each's strength.

### 4.4 The `revises` payload (TOML, doctrine-native translation)

`revises` is a **Tier-2 typed** `RelationLabel`; source `REV`; targets `{SPEC,
PRD, REQ, ADR, POL, STD}`. The payload table in `revision-NNN.toml`:

```toml
[[change]]
target  = "SPEC-018"     # durable peer id (the FK / anchor)
action  = "updated"      # spec: created|updated|retired
primary = true           # exactly one row corpus-per-revision
# spec-family targets carry optional rich detail:
requirement_flow = { introduce = ["REQ-201"], retire = ["REQ-090"], move_in = [], move_out = [] }
section_note = "tighten §3 invariants"   # free-text, prose-body region (excerpt in .md)

[[change]]
target  = "REQ-201"
action  = "introduce"    # requirement: introduce|modify|move|retire
primary = false
to_status = "pending"    # for retire/lifecycle moves → drives spec req status
member_of = "SPEC-018"   # for introduce/move → drives spec req add / re-member

[[change]]
target  = "ADR-006"
action  = "updated"      # ADR/POL/STD: prose-only — no rich detail
primary = false
# prose diff carried as a free-text excerpt in revision-NNN.md
```

The `[[change]]` table is **uniform over all targets**; detail columns
(`requirement_flow`, `to_status`, `member_of`, `section_note`) are optional,
populated only where the target kind warrants. ADR/POL/STD rows are just
`{target, action, primary}` + prose excerpt in MD. **No intra-file structured
anchor in v1** — peer-entity FKs (`REQ`/`SPEC`/`PRD` ids) anchor the structured
bulk; prose-body regions carry free-text excerpts. IDE-002 (durable region
primitive) is the future structured-anchor upgrade — `after IDE-002` (soft seq),
**not** a hard `needs`.

### 4.5 Apply path — `doctrine revision apply REV-N`

Walks the approved `[[change]]` rows; drives **existing** seams (orchestrates,
never reimplements):

| change kind | seam reused |
|---|---|
| requirement status (retire / lifecycle) | SL-044 B·P1 `spec req status` setter + compose a `RecDoc` |
| requirement introduce | `spec req add` (reserve REQ + member) |
| requirement move | membership re-link (`members.toml` rows) |
| spec/req statement/prose edit | applied to `.md` (or flagged for human hand-edit) |
| spec create/retire | `spec new` / spec status |
| ADR/POL/STD prose | flagged for human hand-edit (prose-only) |

**No drift re-prompt at apply** — the Revision already captured + approved the
decision; SL-044's interactive drift scan is a reconcile-time aid, redundant here.
Reuse SL-044's `RecDoc` *composition* (atomic write), not its interactive CLI loop.
Apply refuses unless `approval = approved` when conduct = `gate`. On full apply →
`status = done`, dependents unblock.

### 4.6 REC composition

REC stays **untouched** (its `status_delta`/`evidence_ref` model is
requirement-status-specific, immutable — SL-042/SL-044). A Revision `produces` a
REC **only** for the requirement-status-delta subset of its changes; everything
else (prose, membership, introduce) never had RECs and doesn't now — the
Revision's `[[change]]` rows + git are the record. A standalone (non-slice-close)
status change → REC `owning_slice = None` (the field is already `Option`) + a
`produces`/`recorded_by` edge Revision↔REC. **No REC schema change.**

### 4.7 Dep/seq wiring

Add `REV` to the work-like predicate (`main.rs:3050`) as **both source and
target**: `SL/ISS/IMP/…` can `needs REV-NNN` (the IDE-010 payoff); a Revision can
itself `needs` a spike. Governance docs remain **excluded** as dep/seq targets
(the SL-060 invariant) — depending on governance routes through the Revision.

## 5. Code Impact

| Path | Change |
|---|---|
| `.doctrine/adr/013/` | **ADR-013** (PHASE-01): Revision as first-class change-axis kind; gov→work dep via Revision |
| `doc/entity-model.md` | close §Adjudication open question (home = change axis); cite ADR-013 |
| `doc/spec-entity-spec.md` | relocate `REV-` off the deferred spec-subtype; cite ADR-013 |
| `src/revision.rs` (new) | `REV_KIND`, `RevDoc` (id/slug/title/status/approval/`[[change]]`), scaffold, `run_new`/`run_show`/`run_status`/`run_apply` |
| `src/integrity.rs` | `KINDS` row (REV, stem `revision`, `state_dir: None`) |
| `src/relation.rs` | `RelationLabel::Revises` + `RELATION_RULES` row (sources `[REV]`, targets `{SPEC,PRD,REQ,ADR,POL,STD}`, Tier Typed) |
| `src/registry.rs` (or relation-index) | derive inbound `revises` reciprocity from `[[change]]` rows (members.toml precedent) |
| `src/main.rs` | `Revision { command }` dispatch; work-like predicate += REV |
| `install/manifest.toml` + `.gitignore` | `.doctrine/revision` dir + negation |

## 6. Verification Alignment

- **Lifecycle/approval** — `revision status` FSM tests (advance/abandon; approval
  orthogonal to status; default `gate`). Behaviour, not trivial impl.
- **Cardinality/reciprocity** — golden: 3-target Revision (1 primary), assert
  `adr show`/`spec show` inbound lists **all** touching REVs; `primary` selects the
  dep anchor only. Pins the user's "find all REVs that changed ADR-X" requirement.
- **Dep/seq** — `slice needs REV-NNN` accepted; governance-doc target still
  refused (SL-060 invariant intact).
- **Apply** — each `[[change]]` kind routes to the right existing seam; status
  moves emit one `RecDoc` each; REC schema unchanged; idempotent/edit-preserving.
  `apply` refused when `approval≠approved` under `gate`.
- **Behaviour-preservation gate** — existing entity-engine / relation / dep_seq /
  reconcile suites stay green unchanged (shared-machinery gate).
- **Wiring** — fresh-install scaffolds `.doctrine/revision`; a REV is committable
  (gitignore negation — the `adr` trap, authored-entity-wiring memory).

## 7. Phase Sketch (refined by /plan)

1. **PHASE-01** — ADR-013 + doc edits (governance moves first; IMP-047 principle).
2. **PHASE-02** — kind + spine: `revision.rs`, KINDS row, manifest/gitignore,
   `revision new`/`show`, lifecycle status + approval field, `revision status`.
3. **PHASE-03** — relations: `Revises` label + rules + `[[change]]` payload table;
   relation-index reciprocity derivation; `link`/`inspect` rendering.
4. **PHASE-04** — dep/seq: work-like predicate += REV; `needs`/`after` REV
   source+target; SL-060 governance-exclusion regression.
5. **PHASE-05** — apply: `revision apply` orchestrating existing seams; REC
   `produces` edge; approval gate enforcement at apply.
6. **PHASE-06** (or follow-up) — `/revise` skill + workflow integration.

## 8. Resolved Design Questions

1. **One kind or two?** ONE — two lifecycle lenses (§1, §4.1).
2. **Home?** Standalone change-axis kind, `REV-NNN` (§4.1); ADR-013 anchors.
3. **Scope?** **C** — full structured delta payload (user; SL-044 done).
4. **Cardinality?** Multi-target payload rows + `primary` flag; reciprocity
   derived (§4.3) — selected by the "find all REVs touching ADR-X" constraint.
5. **IDE-002?** Relate-not-block; FK anchors + free-text excerpts for prose body;
   peer-entity model shrinks IDE-002 to a prose-body-only future upgrade (§4.4).
6. **Lifecycle?** Backlog-style; `done` = applied (§4.2).
7. **Approval?** Separate field, default `gate` (§4.2).
8. **Apply / REC?** Orchestrate existing seams, no drift re-prompt; REC untouched,
   loose `produces` edge, `owning_slice = None` for standalone (§4.5–4.6).
9. **ADR altitude?** ADR-013 (A), authored as PHASE-01 (§5, §7).

## 9. Open Questions

None blocking. Carried for plan/execute:
- OQ-1: `[[change]]` detail column names + soft-enum action vocab — finalize against
  the apply-path seams in PHASE-03/05 (the table shape in §4.4 is provisional).
- OQ-2: whether `revision apply` is all-or-nothing or per-row resumable (lean:
  atomic per invocation, mirroring SL-044's one-act-one-commit).
