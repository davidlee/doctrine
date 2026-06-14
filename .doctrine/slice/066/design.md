# Design SL-066: Revision entity — pending revise-intent + staged-delta vehicle

> **STATUS: DRAFT (2026-06-14).** Nine design questions resolved in the `/design`
> clarifying loop (see §8). Scope **C** (full structured delta payload) chosen by
> user (SL-044 is done — the deferral-to-the-writer argument is moot). Three
> adversarial passes integrated: internal (§11), external codex/GPT-5.5 (§12),
> external Opus (§13, fresh-mind — G1 blocker on the KINDS-consumer wiring).
> Awaiting lock.
> Reference forms: padded ids (`SL-066`, `ADR-013`,
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
  (`RecDoc`, `rec.rs:121`; the single `move` + `owning_slice: Option<String>` live
  on the nested `RecMeta`, `rec.rs:105/111`, accessed `RecDoc.rec.owning_slice` —
  G5).
- **Spec composition** (SL-015): a requirement is a reserved **peer entity**
  `REQ-NNN`; membership is a spec-side `members.toml` edge (FK + mobile `FR-`/`NF-`
  label + order); `spec req add` is the only producer.

## 3. Forces & Constraints

- **Governance.** ADR-003 (explicit-authorship-not-derivation; names the revision
  artefact missing §11). ADR-009 (conduct axis; **"approval is not lifecycle"**).
  ADR-010 (relation modelling — unify contract, keep storage bespoke). ADR-004
  (relations outbound-only; reciprocity **derived**). ADR-001 (leaf←engine←command
  layering). **ADR-003 / §1** — the canon-moves-first authoring ethos PHASE-01 rides
  (G7: *not* IMP-047, which is the unbuilt *trinary-actionability* improvement —
  "kinds gate work without being actionable"; v1 REV pending statuses classify
  `Workable`, the IMP-047 `Gating` reclassification is a follow-up, §4.7).
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
`proposed → started → done` (+ `abandoned` from any non-terminal). Dependents gate
from `proposed` (the IDE-010 anchor works from birth).

**`done` = every change row landed** (E1). Apply auto-lands only the rows with a
safe engine seam (`status`, §4.5); `introduce`/`create`/`move`/prose rows are
**surfaced-for-manual** and land by operator hand-edit. So apply moves a
status-only Revision straight to `done`; a Revision carrying manual rows stays
`started` after apply (status rows landed, manual list surfaced) until the operator
completes them and marks `done`. `done` therefore never lies to a dependent — it
means the authored truth is in (E1 closes external M1: prose-only Revisions cannot
silently reach terminal).

**Approval is a separate field** (hard canon: entity-model "approval is not
lifecycle"; ADR-009): `approval = none | requested | approved | rejected`,
orthogonal to `status`.
- **Lifecycle transitions are approval-blind** (advisory, surfaced not enforced —
  ADR-009 §108 posture). A Revision can sit `started` at `approval=none`.
- **Apply requires `approval=approved`** (§4.5) — an apply-time **forcing-function
  checkpoint**: apply refuses unless an explicit approval act has been recorded.
  This is **not** actor-attributed authz — ADR-009 §113 is invoker-blind, so the
  checkpoint cannot tell who approved (a solo dev self-approves). It forces a
  deliberate, separate approval step before the irreversible truth-write; real
  identity-gated enforcement is the ADR-009 OQ-1 identity follow-up, not v1 (E3
  reframes external M3 — a *checkpoint*, not a "governance gate").

**Default `gate` posture** — a Revision writes authored governance/spec truth at
`reconcile`'s altitude, which ADR-009 defaults to `gate`. **Home (v1):** a **baked
default**, not per-repo conduct config — ADR-009's `[conduct]` table is
*slice-FSM-state-keyed* and does not address Revision; extending it is deferred. A
solo dev self-approves (`revision approve REV-N`). This default is advisory for
lifecycle and only materialises as the apply checkpoint above.

### 4.3 Cardinality — multi-target payload rows + `primary` flag

The touched-entity set lives in a typed payload table (§4.4); **the rows ARE the
edges** (members.toml precedent). Inbound `revises` reciprocity is derived by
**`relation_graph`** (E2 — the cross-kind inbound seam: a new
`revision::relation_edges` accessor reads the `[[change]]` rows, `outbound_for`
dispatches to it, and inbound is an **O(1) indexed reverse-adjacency lookup**
(`in_edges`) over the prebuilt graph — *not* a per-query scan; the one corpus walk
is upstream in `scan_entities`, run once per `inspect` (G6). Exactly the
members/interactions path; **not** `registry.rs`, which is only the spec-validate
seed. Adding REV to `KINDS` means every `inspect` now also walks the `revision/`
dir — additive cost, RSK-006-class defer; the slug-symlink alias is already skipped
(`entity::scan_ids` filters `is_dir()` + `parse::<u32>()`, no symlink-follow). The
inbound list
surfaces on **`doctrine inspect ADR-X`**, **not** `adr show`/`spec show`: ADR-004 §3
reserves inbound completeness to the registry-/scan-backed surface (`inspect`),
never the one-way `show` reader ("no cross-corpus scan", `spec.rs:906`). So
`inspect ADR-X` lists *every* REV that touches it, uniform over all rows.

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
member_of    = "SPEC-018"           # destination spec (must be a live SPEC-NNN)
new_statement = "The writer MUST …" # the requirement's statement line
new_label    = "FR-007"             # REQUIRED + frozen at `change add` (E4)
primary      = false
# apply allocates REQ-NNN, members it, and back-fills `allocated = "REQ-201"`
```

**Creation rows freeze their label (E4).** `new_label` is **required**, not
auto-assigned at apply — `spec req add` computes label+order from the *live* member
set (`spec.rs:850-854`), so an omitted label would let membership churn between
draft and apply silently change what lands (external M2). Capturing `new_label` at
`change add` keeps approved == landed. **No cross-row creation deps in v1** —
`member_of` must name a live `SPEC-NNN`, so "create spec then introduce into it" in
one Revision is out of scope for C (the new spec has no id until applied).

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

**v1 auto-applies `status` rows only (E1/E5).** The full payload *stages* (scope C),
but apply auto-lands only rows with a genuine engine seam. Everything else is
**surfaced-for-manual** — staged, listed at apply, landed by the operator. This is
the honest consequence of external B1+B2: `spec req add`/`spec new` are
non-transactional CLI handlers (`spec.rs:826` "NOT transactional by design"), so
auto-applying creation rows would risk orphaned half-writes the "one commit" cannot
undo. Narrowing v1 to status both removes that hazard and removes the handler→engine
refactor (status already rides the engine-callable `requirement::set_status`,
**defined `requirement.rs:339`** — `spec.rs:897` is only the existing CLI call site
(G4) — no ADR-001 violation).

| change action | v1 disposition |
|---|---|
| `status` (retire / lifecycle) | **AUTO** — `requirement::set_status` (engine seam) + compose a `RecDoc` |
| `introduce` | surfaced-for-manual — `spec req add` non-transactional (orphan risk); operator runs it, fills `allocated` |
| `create` (spec) | surfaced-for-manual — `spec new` materialises immediately; operator runs it |
| `modify` (statement / prose) | surfaced-for-manual — human hand-edit `.md` |
| `move` | surfaced-for-manual (F4) — no membership-move seam (`spec req link` is the deferred SL-015 follow-on) |
| ADR/POL/STD prose | surfaced-for-manual — prose-only human hand-edit |

Auto-applying `introduce`/`create` returns once `spec::add_requirement` /
`spec::create_spec` exist as transactional engine helpers (external B2 follow-up,
§5) — additive, no model change.

**Apply over status rows is all-or-nothing (F7, refined by E1).** With v1 narrowed
to status rows — each a single edit-preserving `requirement::set_status` write —
apply runs a **pre-flight sweep** of all status rows (existence + `from`-guard), then
writes; any pre-flight refusal aborts the *whole* apply before the first write. The
earlier "atomic over N heterogeneous seams" claim was unachievable (external B1: the
creation seams are non-transactional); status-only apply makes all-or-nothing real,
because the only auto-writes are independent per-file edit-preserving sets with
nothing to half-create. One commit carries the status edits and **N `RecDoc`s** (one
per status row — D-B8 forces one move per REC; the grain differs from SL-044's
one-act-one-commit intentionally: a Revision apply is N status acts in one commit,
each REC self-describing for NF-003 reconstructability).

**Terminal disposition (E1).** Status-only Revision → `status = done`, dependents
unblock. A Revision also carrying surfaced-for-manual rows stays `started` after
apply (status landed, manual list printed); the operator completes the manual rows
and marks `done`. `done` always means every row landed (§4.2).

**No drift re-prompt at apply** — the Revision already captured + approved the
decision; SL-044's interactive drift scan is a reconcile-time aid, redundant here.
Reuse SL-044's `RecDoc` *composition*, not its interactive CLI loop. Apply
**refuses** unless `approval = approved` (§4.2 — the apply-time checkpoint, not
actor-attributed authz).

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
requirement-status-specific, immutable — SL-042/SL-044). Apply composes a REC
**only** for the requirement-status-delta subset (the v1 auto set); everything else
(prose, membership, introduce) never had RECs and doesn't now — the Revision's
`[[change]]` rows + git are the record. A standalone (non-slice-close) status change
→ REC `owning_slice = None` (the field is already `Option`). **No REC schema change.**

**No `produces`/`recorded_by` relation edge in v1 (E6).** Those labels do not exist
in `RelationLabel`/`RELATION_RULES` (external B4) and inventing them is a governed
ADR-010 relation addition, not a "loose edge." v1 links REC↔REV implicitly: the
status row names the `REQ`, the REC records the same delta, and the apply commit
groups them. A first-class REV↔REC label is a follow-up if a query demand appears.

### 4.7 Dep/seq wiring

Add `REV` to the work-like predicate (`main.rs:3050`) as **both source and
target**: `SL/ISS/IMP/…` can `needs REV-NNN` (the IDE-010 payoff); a Revision can
itself `needs` a spike. Governance docs remain **excluded** as dep/seq targets
(the SL-060 invariant) — depending on governance routes through the Revision.

**A new `KINDS` row is read by THREE corpus-walk tables, not one (G1–G3, §13).** The
work-like predicate alone is insufficient; each must carry a REV arm or the wiring
half-works:
- **`priority::partition` (G1, blocker).** Blocking is computed from
  `status_class` (`partition.rs:186`), **not** the FSM. A kind with no `PARTITION`
  row classifies `Unrecognised` for *every* status (`partition.rs:191-193`), and
  `blocked_by` excuses a predecessor only when `class == Terminal`
  (`channels.rs:67`) — so a `done`/`abandoned` REV stays `Unrecognised != Terminal`
  and **blocks its dependent forever**, the inverse of the IDE-010 payoff. Add a
  dedicated `KindPartition { prefix: "REV", workable: ["proposed","started"],
  terminal: ["done","abandoned"] }` + a `REV_STATUSES` const so the VT-1 drift
  canary binds. REV's vocab differs from backlog's (`open/triaged/resolved/closed`),
  so it gets its **own** row — it cannot ride the backlog arm.
- **`relation_graph::dep_seq_for` (G2).** For REV-as-*source*: the verb authors a
  REV-sourced `needs`/`after`, but `dep_seq_for` routes only `SL` + the five backlog
  prefixes to a reader; every other kind short-circuits to an empty `DepSeq` with no
  disk read (`relation_graph.rs:134-137`), so REV's own edges never reach the
  blocker/`next` view. Add a `"REV"` arm (mirror the `SL` arm — leaf `dep_seq::read`
  over `revision-NNN.toml`). REV-as-target needs only G1; REV-as-source needs G2.
- **`relation_graph::outbound_for` (G3, already in §5).** Its fallthrough is
  `debug_assert!(false, …)` (`relation_graph.rs:78`); the REV arm (the
  `revision::relation_edges` accessor, §4.3) must land **with** the `KINDS` row, not
  a phase later, or debug-build scans panic mid-PHASE — see §7 sequencing.

## 5. Code Impact

| Path | Change |
|---|---|
| `.doctrine/adr/013/` | **ADR-013** (PHASE-01): Revision as first-class change-axis kind; gov→work dep via Revision |
| `doc/entity-model.md` | close §Adjudication open question (home = change axis); cite ADR-013 |
| `doc/spec-entity-spec.md` | relocate `REV-` off the deferred spec-subtype; cite ADR-013 |
| `src/revision.rs` (new) | `REV_KIND`, `RevDoc` (id/slug/title/status/approval/`[[change]]`), scaffold, `run_new`/`run_show`/`run_status`/`run_change_add`/`run_approve`/`run_apply` |
| `src/integrity.rs` | `KINDS` row (REV, stem `revision`, `state_dir: None`) |
| `src/relation.rs` | `RelationLabel::Revises` + `RELATION_RULES` row (sources `[REV]`, targets `{SPEC,PRD,REQ,ADR,POL,STD}`, Tier Typed, **`LinkPolicy::TypedVerbOnly`**) |
| `src/relation_graph.rs` | (a) `revision::relation_edges` accessor + `outbound_for` arm (REV → reads `[[change]]` rows); inbound `revises` derived via indexed `in_edges`, surfaced on `inspect` (E2 — **not** `registry.rs`, **not** `show`; ADR-004 §3). (b) **`dep_seq_for` REV arm** (G2 — leaf `dep_seq::read` over `revision-NNN.toml`; mirrors the `SL` arm) so REV-as-source `needs`/`after` reach the blocker view |
| `src/priority/partition.rs` (G1) | **dedicated REV `KindPartition`** (`workable ["proposed","started"]`, `terminal ["done","abandoned"]`) + `REV_STATUSES` const for the VT-1 canary — WITHOUT it a `done` REV classifies `Unrecognised`, never `Terminal`, so `needs REV-N` never unblocks (`partition.rs:191-193`, `channels.rs:67`) |
| `src/requirement.rs` (E5) | v1 apply rides the existing engine-callable `set_status` (`spec.rs:897`) — **no refactor**. `spec::add_requirement`/`spec::create_spec` engine helpers are the introduce/create-apply follow-up (external B2), not v1 |
| `src/main.rs` | `Revision { command }` dispatch (`new`/`show`/`status`/`change add`/`approve`/`apply`); work-like predicate += REV |
| `install/manifest.toml` + `.gitignore` | `.doctrine/revision` dir + negation |

## 6. Verification Alignment

- **Lifecycle/approval** — `revision status` FSM tests (advance/abandon; approval
  orthogonal to status; default `gate`). Behaviour, not trivial impl.
- **Cardinality/reciprocity** — golden: 3-target Revision, assert **`inspect ADR-X`
  / `inspect REQ-N`** inbound lists **all** touching REVs (uniform, not just
  primary) — `inspect`, never `show` (E2; ADR-004 §3). Pins the user's "find all REVs
  that changed ADR-X" requirement. Separately: `primary` is display-only — `needs
  REV-N` blocks on the whole Revision regardless of `primary` (and with zero
  `primary` rows).
- **Dep/seq** — `slice needs REV-NNN` accepted; governance-doc target still
  refused (SL-060 invariant intact); `needs REV-N` blocks until REV-N terminal.
- **Apply (status-only auto, E1)** — clean apply lands every `status` row via
  `requirement::set_status`, emits N `RecDoc`s in one commit, REC schema unchanged; a
  single pre-flight refusal aborts the whole apply (no partial writes); `apply`
  refused when `approval≠approved` (checkpoint). `introduce`/`create`/`modify`/`move`/
  prose rows surfaced-for-manual, NOT auto-applied. **`done` only when every row
  landed** — a Revision with manual rows stays `started` post-apply (M1 regression); a
  status-only Revision reaches `done`.
- **`from`-guard (pre-flight)** — any `status` row whose `from` ≠ the target's
  current `ReqStatus` aborts the whole apply + surfaces the stale set; never
  clobbers an intervening `reconcile` move. Non-status rows carry no guard.
- **Behaviour-preservation gate** — existing entity-engine / relation / dep_seq /
  reconcile suites stay green unchanged (shared-machinery gate).
- **Wiring** — fresh-install scaffolds `.doctrine/revision`; a REV is committable
  (gitignore negation — the `adr` trap, authored-entity-wiring memory).

## 7. Phase Sketch (refined by /plan)

1. **PHASE-01** — ADR-013 + doc edits (governance moves first; ADR-003 / §1 ethos —
   *not* IMP-047, G7).
2. **PHASE-02** — kind + spine: `revision.rs`, manifest/gitignore, `revision
   new`/`show`, lifecycle status + approval field, `revision status`. **The `KINDS`
   row lands with its three corpus-walk arms in the SAME phase (G3):** the
   `outbound_for` REV arm (empty-then-filled in PHASE-03 is fine), the `dep_seq_for`
   REV arm (G2), and the `priority::partition` REV row + `REV_STATUSES` const (G1) —
   otherwise the debug-build corpus scan hits `outbound_for`'s `debug_assert!(false)`
   / a missing partition the moment a REV is minted, going RED at PHASE-02's end
   before later phases supply the arms.
3. **PHASE-03** — relations payload: `Revises` label + rules + `[[change]]` payload
   table; fill the `revision::relation_edges` accessor (reciprocity derivation, E2);
   `inspect` rendering.
4. **PHASE-04** — dep/seq surface: work-like predicate += REV (`main.rs:3050`);
   `needs`/`after` REV source+target end-to-end (the partition + `dep_seq_for` arms
   already exist from PHASE-02); SL-060 governance-exclusion regression.
5. **PHASE-05** — apply: `revision apply` auto-landing `status` rows via
   `requirement::set_status` + RecDoc; surface-for-manual for creation/move/prose;
   approval checkpoint; from-guard pre-flight. (No `produces` edge — E6.)
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
- **No machine-verification that surfaced-for-manual rows actually landed** (G2/hunt
  item 2). `done` requires every row landed (§4.2), but nothing checks the manual
  subset before `revision status REV-N done` is accepted — a dependent `needs REV-N`
  unblocks on the operator's word. This is the **same trust model** as the
  invoker-blind approval checkpoint (E3) and SL-044's authored-truth stance;
  blast radius is the manual subset only (status rows auto-land). Honour-system by
  design, not an oversight.
- **No reversal of a partially-applied Revision on abandon** (hunt item 4). Apply
  lands the `status` rows (RecDocs, one commit) before a manual-row Revision settles;
  if the operator then `abandoned`s it, those status deltas **stay in force** — they
  are real reconciliations and must not be un-reconciled. `abandoned` disclaims only
  the unlanded manual remainder. The resulting "abandoned REV with landed status
  deltas" is coherent, not a leak.

## 10. Open Questions

None blocking. Carried for plan/execute:
- OQ-1: `[[change]]` detail column names + soft-enum action vocab — finalize against
  the apply-path seams in PHASE-03/05 (the table shape in §4.4 is provisional). Also
  absorbs a `change add` authoring-validation case (G-hunt item 1): **two `status`
  rows on the same target REQ with different `to_status`** would be last-writer-wins
  at apply — a `change add` validation/dedup concern, not a soundness blocker (the
  pre-flight `from`-guard reads pre-apply state, so row-2 never reads row-1's write).
- OQ-2 (**resolved**, F7 → refined E1): `revision apply` is all-or-nothing **over
  `status` rows** — pre-flight sweep, then write, one commit, N RecDocs. Heterogeneous
  atomicity dropped (external B1); creation/move/prose are surfaced-for-manual.
- OQ-3 (carried): introduce/create **auto**-apply — needs transactional
  `spec::add_requirement`/`spec::create_spec` engine helpers (external B2). v1
  surfaces these rows for manual handling (§4.5).

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

> **Note (E-tags):** F7/F8 were *superseded* by the external pass — see §12. The
> "atomic over heterogeneous seams" thesis (F7) and the "refactor spec handlers"
> plan (F8) were both unachievable / unnecessary for v1; E1/E5 narrow v1 apply to
> `status` rows, which dissolves both.

## 12. Adversarial Pass (external — codex/GPT-5.5, 2026-06-14)

Seven findings (4 blocker, 3 major), each **verified against source + governance**
(not taken on the reviewer's word) and integrated (E-tags above). Verdict on entry:
*needs-rework*; all dispositioned.

- **B1 → E1/E5** — "atomic apply over N heterogeneous seams" was fictional: the
  creation seams are non-transactional (`spec.rs:826` self-documents it). v1
  auto-applies `status` rows only (real all-or-nothing over edit-preserving writes);
  creation/move/prose surfaced-for-manual (§4.5). Cascade: dissolves B2 and M1.
- **B2 → E5** — F8 understated. `spec new`/`spec req add` are CLI handlers, not engine
  fns (ADR-001 bar); the real status seam is `requirement::set_status` (`spec.rs:897`),
  not `spec.rs`'s handlers. v1 (status-only) needs **no refactor** — it rides
  `set_status`. `spec::add_requirement`/`create_spec` extraction is the
  introduce/create-apply follow-up (OQ-3, §5).
- **B3 → E2** — reciprocity is `relation_graph` (`outbound_for`/`in_edges`), not
  `registry.rs`; inbound surfaces on `inspect`, not `show` (ADR-004 §3) (§4.3/§5/§6).
- **B4 → E6** — `produces`/`recorded_by` labels don't exist (`relation.rs:45`).
  Dropped from v1; REC↔REV linkage is implicit (§4.6).
- **M1 → E1** — prose-only Revision could reach `done` with truth unlanded. `done` now
  requires every row landed; manual-row Revisions hold at `started` (§4.2).
- **M2 → E4** — creation rows drifted (label/order computed at apply, `spec.rs:850-854`).
  `new_label` required + frozen at `change add`; no cross-row creation deps in v1 (§4.4).
- **M3 → E3** — approval reframed: an apply-time forcing-function **checkpoint**, not
  actor-attributed authz (ADR-009 §113 invoker-blind). Real enforcement is the ADR-009
  identity follow-up (§4.2).

## 13. Adversarial Pass (external #2 — Opus, 2026-06-14)

Fresh-mind second external pass on a fresh hunt list (from-guard self-interaction,
done honour-system, `relation_graph` cost, abandon-mid-apply, `primary` residual,
PHASE-01 sequencing). Seven findings, each **verified against source by the
triaging agent** (not taken on the reviewer's word). **Root cause** of the three
substantive ones: §5/§7 under-counted the corpus-walk tables that consume an
`integrity::KINDS` row. A new kind row is read by **three** walk surfaces; only one
(`outbound_for`) was listed. Verdict on entry: *minor-fixes* (G1 blocks lock).

- **G1 (blocker) → §4.7/§5/§7.** `needs REV-N` never unblocks. Blocking is computed
  from `priority::status_class` (`partition.rs:186`), not the FSM: a kind with no
  `PARTITION` row classifies `Unrecognised` for *every* status
  (`partition.rs:191-193`), and `blocked_by` excuses a predecessor only at
  `class == Terminal` (`channels.rs:67`). A `done`/`abandoned` REV stays
  `Unrecognised != Terminal` → blocks its dependent forever, the inverse of the
  IDE-010 payoff. **Fix:** dedicated REV `KindPartition` (`workable
  ["proposed","started"]`, `terminal ["done","abandoned"]`) + `REV_STATUSES` const
  for the VT-1 canary; its own row (REV's vocab ≠ backlog's). §5 +=
  `priority/partition.rs`; lands with the KINDS row.
- **G2 (major) → §4.7/§5/§7.** REV-as-source dep/seq silently dropped.
  `dep_seq_for` (`relation_graph.rs:101`) routes only `SL` + the five backlog
  prefixes; every other kind short-circuits to an empty `DepSeq`, no disk read
  (`relation_graph.rs:134-137`). Editing the work-like predicate lets the verb
  *author* a REV-sourced edge that is never read back. **Fix:** a `"REV"` arm
  mirroring `SL`. Target-only needs only G1; source needs G2.
- **G3 (minor, sequencing) → §7.** `outbound_for`'s fallthrough is
  `debug_assert!(false, …)` (`relation_graph.rs:78`); a KINDS row added a phase ahead
  of its arm panics every debug-build corpus scan the moment a REV is minted (RED at
  PHASE-02 end). **Fix:** land the row + all three arms in one phase (§7).
- **G4 (minor, citation) → §4.5.** `requirement::set_status` is **defined**
  `requirement.rs:339`; `spec.rs:897` is only the CLI call site. Substance
  (engine-callable, no refactor) holds.
- **G5 (nit, citation) → §2.** `owning_slice` + the single `move` live on `RecMeta`
  (`rec.rs:105/111`), accessed `RecDoc.rec.owning_slice`; `rec.rs:121` is `RecDoc`.
- **G6 (minor, wording) → §4.3.** `in_edges` is an O(1) indexed reverse-adjacency
  lookup, **not** a scan; the one corpus walk is upstream in `scan_entities`, once
  per `inspect`. REV joins the per-`inspect` scan set (additive, RSK-006-class). The
  slug-symlink footgun is **already defended** (`entity::scan_ids` `is_dir()` +
  `parse::<u32>()`) — hunt-item-3 clean.
- **G7 (minor, attribution) → §3/§7.** IMP-047 is the *trinary-actionability*
  improvement ("kinds gate work without being actionable"), **not** a
  "canon-moves-first" principle. PHASE-01's doc-first ordering rides ADR-003 / §1.
  v1 REV pending statuses classify `Workable` (G1); the IMP-047 `Gating` reclass is a
  follow-up.

**Hunt-list dispositions (no new hole):**
- *from-guard self-interaction (item 1)* — sound. v1 auto-applies only `status`
  rows; the pre-flight sweep reads current status before any write, so two rows on
  one REQ both check the same pre-apply state (row-2 never reads row-1's write). The
  different-`to_status` case is `change add` authoring-validation, folded into OQ-1.
- *done honour-system (item 2)* — real but acceptable/named (§9): same trust model as
  the invoker-blind approval checkpoint; blast radius = the manual subset.
- *abandon mid-apply (item 4)* — real but acceptable/named (§9): landed status
  RecDocs are real reconciliations; abandon disclaims only the unlanded remainder.
- *primary residual lean (item 5)* — clean. Nothing in apply/from-guard/RecDoc/
  blocking keys on `primary`; "at most one" is a `change add` rule, breaks only
  display if violated.
- *PHASE-01/ADR-013 (item 6)* — doc-level ordering fine (ADR-013 is new; max is
  ADR-012). The real ordering hole is intra-code → G3.

**OQ-1 / OQ-3** confirmed genuinely plan/execute-deferrable. G1–G3 are one coherent
§5/§7 wiring fix; G4–G7 citation/wording sweeps; the two honour-system gaps got §9
lines.
