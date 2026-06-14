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
orthogonal to `status`. Two enforcement tiers (resolving the advisory-vs-enforced
tension):
- **Lifecycle transitions are approval-blind** (advisory, surfaced not enforced —
  ADR-009 posture). A Revision can sit `started` at `approval=none`.
- **Apply hard-gates on `approval=approved`** (§4.5) — apply is the irreversible
  truth-write, so it is the one place approval is *enforced*, not merely surfaced.

**Default `gate`** — a Revision writes authored governance/spec truth at
`reconcile`'s altitude, which ADR-009 already defaults to `gate`. **Home (v1):** a
**baked default**, not per-repo conduct config — ADR-009's `[conduct]` table is
*slice-FSM-state-keyed* and does not address Revision; extending it to address
Revision is deferred. A solo dev self-approves (`revision approve REV-N`).

### 4.3 Cardinality — multi-target payload rows + `primary` flag

The touched-entity set lives in a typed payload table (§4.4); **the rows ARE the
edges** (members.toml precedent). The relation-index derives inbound `revises`
reciprocity from these rows, **uniform over all rows** — so `doctrine adr show
ADR-X` lists *every* REV that touches it.

**`primary` is a display/headline hint, not a functional dep anchor** (F1
correction). `needs`/`after` target *entity ids*, never rows — `needs REV-NNN`
blocks on the **whole** Revision reaching terminal, regardless of which row (if
any) is primary. So `primary` is **at most one, optional** (a pure-prose ADR
revision nobody depends on yet carries none); it names the Revision's headline
subject for display/discovery only, never visibility or blocking.

This merges IDE-010 (the Revision *is* the crisp single anchor a slice depends on)
and IDE-003/spec-driver (multi-entity batch payload) at each's strength.

### 4.4 The `revises` payload (TOML, doctrine-native translation)

`revises` is a **Tier-2 typed** `RelationLabel`; source `REV`; targets `{SPEC,
PRD, REQ, ADR, POL, STD}`. **`LinkPolicy = TypedVerbOnly`** (F2) — authored by a
`revision change add` verb that writes the payload row (the members.toml
precedent), **never** by `doctrine link`. The `RELATION_RULES` row exists for
target validation + inbound-reciprocity naming, not a writable Tier-1 edge.

The `[[change]]` table has **two row shapes** (F3 — creation ops can't key on a
durable FK that does not exist yet):

**Existing-target ops** — `modify | retire | move | status` — key on a live FK:
```toml
[[change]]
target    = "REQ-201"    # durable peer id (the FK / anchor)
action    = "status"     # requirement status/lifecycle move
primary   = false
from      = "active"      # AUTO-captured at `change add` (current status then); see §4.5 from-guard
to_status = "retired"    # drives the SL-044 `spec req status` setter at apply

[[change]]
target  = "ADR-006"
action  = "modify"       # ADR/POL/STD: prose-only — no structured detail
primary = true
# prose diff carried as a free-text excerpt in revision-NNN.md
```

**Creation ops** — `introduce | create` — carry no pre-existing target; apply
allocates the id and back-fills:
```toml
[[change]]
action       = "introduce"          # new requirement
member_of    = "SPEC-018"           # destination spec → drives `spec req add`
new_statement = "The writer MUST …" # the requirement's statement line
new_label    = "FR-007"             # optional; else auto-assigned
primary      = false
# apply allocates REQ-NNN, members it, and back-fills `allocated = "REQ-201"`
```

Detail columns are optional, populated only where the target/action warrants.
ADR/POL/STD rows are just `{target, action, primary}` + prose excerpt in MD —
short structured *labels* only in TOML (e.g. a `section_note` pointer); real prose
lives in `revision-NNN.md` (storage rule). **No intra-file structured anchor in
v1** — peer-entity FKs anchor the structured bulk; prose-body regions carry
free-text excerpts. IDE-002 (durable region primitive) is the future
structured-anchor upgrade — `after IDE-002` (soft seq), **not** a hard `needs`.

### 4.5 Apply path — `doctrine revision apply REV-N`

Walks the approved `[[change]]` rows; drives **existing** seams (orchestrates,
never reimplements):

| change action | seam reused |
|---|---|
| `status` (retire / lifecycle) | SL-044 B·P1 `spec req status` setter + compose a `RecDoc` |
| `introduce` | `spec req add` (reserve REQ + member); back-fill `allocated` id |
| `modify` (statement / prose) | applied to `.md` (or flagged for human hand-edit) |
| `create` (spec) | `spec new`; back-fill `allocated` id |
| `move` | **flagged for manual handling** — no existing seam (F4): `spec req` is add-only, the `spec req link`/move verb is the deferred SL-015 follow-on; building it is out of scope for C. `move` rows stage fine; apply surfaces them for the operator |
| ADR/POL/STD prose | flagged for human hand-edit (prose-only) |

**Apply is atomic — one act, one commit (F7).** A `revision apply` runs a
**pre-flight sweep** of all rows, then writes; any pre-flight refusal aborts the
*whole* apply (no partial writes). On clean apply → `status = done`, dependents
unblock; one commit carries the status edits, membership edits, and **N `RecDoc`s**
(one per status row — D-B8 forces one move per REC; the grain differs from SL-044's
one-act-one-commit, intentionally: a Revision apply is N acts in one commit, each
REC self-describing for NF-003 reconstructability).

**No drift re-prompt at apply** — the Revision already captured + approved the
decision; SL-044's interactive drift scan is a reconcile-time aid, redundant here.
Reuse SL-044's `RecDoc` *composition*, not its interactive CLI loop. Apply
**hard-refuses** unless `approval = approved` (§4.2 — the one enforced gate).

**`from`-guard on `status` rows (the only staleness check), run pre-flight.**
Dropping the drift re-prompt opens a silent-clobber: a status row applies via
SL-044's **free any→any** setter, so if the target moved between draft and apply
(e.g. `reconcile` already changed `REQ-201`), blind apply silently reverts it — no
git conflict (an edit-preserving TOML field set that "succeeds"). Closed cheaply:
a `status` row records `from` (auto-captured at `change add`, §4.4); the pre-flight
sweep **reads current `ReqStatus` for every status row and refuses the whole apply
if any `current != row.from`**, surfacing the stale set ("REQ-201 moved to `active`
since draft — re-draft"). Compare-and-set scoped to the one place it bites, keyed
off data already stored, **surfaced not silently-overridden** (drift-surface
posture, ADR-009; `mem.pattern.safety.resolve-every-ref-before-pure-compare`).
Prose / `move` / `introduce` / `create` rows carry **no** guard — human-in-loop +
git. This is *not* optimistic locking (no version stamps, no approval-retraction
FSM — see §9 Non-Goals).

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
| `src/revision.rs` (new) | `REV_KIND`, `RevDoc` (id/slug/title/status/approval/`[[change]]`), scaffold, `run_new`/`run_show`/`run_status`/`run_change_add`/`run_approve`/`run_apply` |
| `src/integrity.rs` | `KINDS` row (REV, stem `revision`, `state_dir: None`) |
| `src/relation.rs` | `RelationLabel::Revises` + `RELATION_RULES` row (sources `[REV]`, targets `{SPEC,PRD,REQ,ADR,POL,STD}`, Tier Typed, **`LinkPolicy::TypedVerbOnly`**) |
| `src/registry.rs` (or relation-index) | derive inbound `revises` reciprocity from `[[change]]` rows (members.toml precedent) |
| `src/spec.rs` (F8) | expose `spec req status` / `spec req add` as **engine-callable fns** (apply calls them programmatically; refactor if currently CLI-handler-bound — ADR-001 command→engine) |
| `src/main.rs` | `Revision { command }` dispatch (`new`/`show`/`status`/`change add`/`approve`/`apply`); work-like predicate += REV |
| `install/manifest.toml` + `.gitignore` | `.doctrine/revision` dir + negation |

## 6. Verification Alignment

- **Lifecycle/approval** — `revision status` FSM tests (advance/abandon; approval
  orthogonal to status; default `gate`). Behaviour, not trivial impl.
- **Cardinality/reciprocity** — golden: 3-target Revision, assert `adr show`/`spec
  show` inbound lists **all** touching REVs (uniform, not just primary). Pins the
  user's "find all REVs that changed ADR-X" requirement. Separately: `primary` is
  display-only — `needs REV-N` blocks on the whole Revision regardless of `primary`
  (and with zero `primary` rows).
- **Dep/seq** — `slice needs REV-NNN` accepted; governance-doc target still
  refused (SL-060 invariant intact); `needs REV-N` blocks until REV-N terminal.
- **Apply (atomic)** — clean apply routes every row to the right seam, emits N
  `RecDoc`s in one commit, REC schema unchanged; **a single pre-flight refusal
  aborts the whole apply (no partial writes)**; `apply` hard-refused when
  `approval≠approved`. `move` rows surfaced-for-manual, not auto-applied (F4).
- **`from`-guard (pre-flight)** — any `status` row whose `from` ≠ the target's
  current `ReqStatus` aborts the whole apply + surfaces the stale set; never
  clobbers an intervening `reconcile` move. Non-status rows carry no guard.
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

## 9. Non-Goals

- **Optimistic locking** — declaring an expected version per change target and
  refusing apply / retracting approval on drift. Gold-plating, and anti-grain:
  doctrine *surfaces* drift, never hard-rejects on it (ADR-009). The narrow
  `from`-guard (§4.5) covers the one real silent-clobber hazard; prose drift is
  caught by the human-in-loop + git. Full version-stamping + an approval-retraction
  FSM is disproportionate to an advisory (`gate`) approval. Recorded as rejected.

## 10. Open Questions

None blocking. Carried for plan/execute:
- OQ-1: `[[change]]` detail column names + soft-enum action vocab — finalize against
  the apply-path seams in PHASE-03/05 (the table shape in §4.4 is provisional).
- OQ-2 (**resolved**, F7): `revision apply` is **atomic** — pre-flight sweep, then
  all-or-nothing write, one commit, N RecDocs. Mirrors SL-044's one-act grain at
  the per-row level.

## 11. Adversarial Pass (internal, 2026-06-14)

Eight findings, all integrated:
- **F1** — `primary` demoted to a display/headline hint (at-most-one, optional);
  `needs` blocks on the whole Revision, not a row (§4.3).
- **F2** — `revises` is `TypedVerbOnly` (authored by `revision change add`, not
  `doctrine link`); RELATION_RULES row is for validation + reciprocity naming (§4.4).
- **F3** — `[[change]]` has two row shapes: existing-target ops vs creation ops
  (which carry no FK; apply allocates + back-fills) (§4.4).
- **F4** — `move`-apply deferred to manual-flag — no existing membership-mutation
  seam (`spec req link`/move is the unbuilt SL-015 follow-on) (§4.5).
- **F5** — approval is advisory for lifecycle, hard-enforced at apply (§4.2/§4.5).
- **F6** — conduct config for Revision is a baked default in v1; ADR-009's
  slice-state-keyed `[conduct]` table doesn't address Revision (deferred) (§4.2).
- **F7** — apply is atomic with a pre-flight guard sweep (resolves OQ-2) (§4.5).
- **F8** — apply needs `spec req status`/`add` exposed as engine-callable fns;
  refactor if handler-bound (§5).
- **F-minor** — `from` auto-captured at `change add`, not hand-typed (§4.4).
