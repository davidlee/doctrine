<!-- doctrine:section sec-1 -->
## The invariant

The inquiry map is meant to be live: `DEC-061` permits nodes to be added as conditional
discoveries become concrete, `DEC-063` makes redeclaration the edit verb, and no stage
gates `declare` — it is admissible even at `locked`. Two conditions made that expensive.
`initial-concerns-recorded` and `user-accepts-sufficiency` both bound
`Coverage::InquiryMap` at `Reach::Cumulative`, and `coverage_moved` compares the **union**
of keys — so a joiner, a leaver and a changed value were one reading. Adding a node voided
both and re-faced the human gates at `inquiring→drafting`, `drafting→reviewing` and
`reviewing→locked`. `DEC-062` says ordinary map maintenance "does not require human
approval"; the implementation required exactly that.

**The invariant.** *An addition is not a change to what was seen; a move or a re-word is* —
**scoped to what the act covered** (`RV-386` `F-1`). A node added after the act was recorded
was never part of what the user was shown, so moving or re-wording it later changes nothing
that act was given over. Only a move, a re-word, or a flipped blocking judgement on a
**covered** node re-faces the human. Reading `DEC-062`'s "moving" narrowly is `DEC-301`'s
record; scoping it to covered nodes is this design's addition to it.

The second half is what keeps the growth obligation honest: *a question the agent judges
blocking is visible to the human the moment it is added.* That is not a property of the
attestation coverage — it is a property of the blocking set, which this design makes a
property of the node (`sec-3`).

Three facts make the narrowing implementable:

1. `NodeMaterial` is question, provenance, parent, needs, seq — lifecycle and disposition
   excluded, so settling a question was already exempt, pinned by
   `node_material_ignores_progress_and_observes_shape`. **`blocking` joins that material**
   (`sec-3`): a blocking judgement is shape, not progress.
2. The two staleness mechanisms were independent: `CoverageStale` (a coverage diff) and
   `ConfirmationStale` (which `sec-3` retires with the act it linked).
3. Re-declaring an unchanged set is free; declaring a changed one is not.

So the design narrows what the *attested* judgements bind to, scopes the move rule to
covered nodes, and moves the blocking set from a free-standing act onto the node.
(`DEC-300` amended by `DEC-302`, `DEC-301` scoped.)

<!-- doctrine:section sec-2 -->
## Narrowing the attested binding

`coverage_moved`'s `Coverage::InquiryMap` arm diffs the carried `CoveredSet::Nodes`
against `materials()`. `ContentCoverage::diff` reports a joiner, a leaver and a changed
value alike, because it walks the union of keys.

The change is one arm of one function, and it has two projections:

- **Material, over carried keys.** A key the act never covered cannot be a change to what
  it covered. A covered node whose material moved is stale; an uncovered node is not.
- **Blocking membership, over the full set.** The ids whose material says `blocking = true`
  are compared in full, because a *new* blocking node is exactly the event that must reach
  the user (`sec-3`). A key absent from the carried map reads as not blocking, so a new
  non-blocking node is no change.

- **Invisible:** a node added after the act was recorded; a `needs` edge added from a node
  the act did not cover.
- **Still stale:** a change to any node the act *did* cover — a re-worded question, a
  re-parent, an added or removed `needs` edge, a flipped blocking judgement — and any
  leaver among them.

That asymmetry is the point: the predicate gets *sharper*. "This is not the content I was
given" replaces "this is not the whole map".

`blocking-inquiries-dispositioned` is unchanged in *shape* — derived, cumulative, over the
map's dispositions. Only where its set comes from moves (`sec-3`).

**What still re-faces the human.** A move (re-parent) and a re-word, because both change
content the user saw; a flipped blocking judgement on a covered node, for the same reason;
and the arrival of a node judged blocking. The first three are `DEC-301`'s narrowed reading,
now scoped to covered nodes (`RV-386` `F-1`); the fourth is `sec-3`.

<!-- doctrine:section sec-3 -->
## The blocking set is derived from the node

`blocking-set-declared` is a free-standing agent act carrying a list of node ids. It is a
second representation of a fact the node itself can hold, and the user reviews it *beside*
the map rather than *in* it — while `initial-concerns-recorded`'s own guidance already tells
the agent to *"show the map as an indented tree: one line per question, the blocking ones
marked"*. The act is the inconsistency, not the map (`DEC-302`).

**`blocking` becomes a node attribute.**

- `Sparse<bool>` on the node declaration: **required on creation**, optional on update
  (omission persists, a value replaces). A creation that omits it is refused.
- It is a member of `NodeMaterial`. A flip on a covered node is a material change and
  re-faces the user through `sec-2`'s first projection.
- A flip on an uncovered node moves only the set projection — correct, because the set is
  what the user is asked about.
- A flip is a recorded mutation, so it emits one `NodeBlockingChanged` row (`REQ-478`);
  creation carries its `blocking` inside `NodeCreated` and owes no second row.

**The set derives.** `blocking_inquiries_open` stops reading a declaration and filters the
nodes: `blocking == true` and lifecycle not `Resolved`. `blocking-inquiries-dispositioned`
keeps its contract (`derived`, `engine(dispositions)`, cumulative) and changes only its
source.

**The growth obligation closes without a new condition.** `initial-concerns-recorded`
becomes a single-act rule — `Attested([GraphReviewed(User)])` with `Coverage::InquiryMap`.
Its second act, the agent's `blocking-set-declared`, and the `confirms` link from the user's
act to it, retire with the set. What replaces the link is `sec-2`'s second projection:

- add a node with `blocking: false` → the set is unchanged → nothing re-faces;
- add a node with `blocking: true` → the set changes → the user's `graph-reviewed` goes
  stale → the user is asked to see it, tree and blocking marks together.

So a question the agent judges blocking cannot pass unseen — `SL-264`'s `R4` closed by
construction rather than by a ninth condition (`RV-386` `F-2`). It closes against the
**user**, not only the agent, which is what `OQ-4`'s "blocking, not a warning" asked for.
And `DEC-126`'s kind axis is untouched: the condition stays `Attested`, and no
`EngineSource` or derivation rule is added.

**Retired with the act.** `AgentAct::BlockingSetDeclared`; `ActKind::BlockingSetDeclared`;
and `Cause::ConfirmationStale`, whose only subject was the link between the user's review
and that act. The payload contract, the generated stage table and the condition guidance
all follow.

**Compatibility.** A stored snapshot whose nodes carry no `blocking` reads the retired act:
`blocking_inquiries_open` falls back to the stored `BlockingSetDeclared` declaration when no
node carries a judgement. Read-side only, no stored shape moves (`DEC-059`). A live run
leaves the legacy regime the first time a node is declared with a blocking judgement.

**This supersedes `DEC-300`'s mechanism, not its intent.** `DEC-300` narrowed the attested
rows *and* added a derived condition requiring additions to be re-declared. The narrowing
stands; the condition is not needed, because the set is derived and the user's coverage
compares it. Recorded as an amendment to `DEC-300`, with the decision in `DEC-302`.

<!-- doctrine:section sec-4 -->
## Honest sparse clearing

`design contract` publishes `needs … sparse (omit persists · null clears)`. `declare_node`
matched only `Sparse::Value`, so `null` changed nothing **and reported success** — `SL-259`'s
disease in the one place it did not look. `question` goes through `Sparse::apply`, which
handles `Null`; `parent` has explicit `Value`/`Null` arms; `needs` was the gap.

The semantics already exist — `apply_collection` maps `Null` and `[]` to one clearing — so
the work is wiring, not design:

- `Sparse::Null` joins `Sparse::Value` in `declare_node`'s needs arm, clearing to the empty
  set.
- Rows: one `NeedsRemoved` per removed edge, through the same set difference that already
  serves `[]`. No new event, so no roster change.
- **A no-op emits no rows.** An edge-free `null` records no mutation, and `REQ-478` obliges a
  row for a mutation the run *records*. The two spellings collapse by construction rather
  than by convention.
- **A clearing is a recorded mutation and advances the revision** (`REQ-429`/`REQ-430`).
- **A no-op claims no rows, not a frozen revision.** `apply` increments `run.revision`
  unconditionally before it reads any declaration (`run.rs:347-349`) — that is the
  compare-and-swap invariant, not a mutation the run recorded. So "the two spellings
  collapse" means no change *rows*; no rule is needed to hold the revision still, and the
  design does not claim one (`RV-386` `F-3`).

`SPEC-029` Responsibilities[13] and `DEC-063` already require this, so the contract does not
change. The code was the deviation.

<!-- doctrine:section sec-5 -->
## Code impact

| path | what changes |
|---|---|
| `src/design_run/inquiry.rs` | `InquiryNode` / `NodeMaterial` gain `blocking`, a sparse node attribute that is a material member |
| `src/design_run/run.rs` | `declare_node`'s `blocking` arm — refused when absent on creation, persisted on omission, replaced on value — and the `NodeBlockingChanged` row on a flip |
| `src/design_run/change_log.rs` | `ChangeEvent::NodeBlockingChanged`, emittable and driven by the fixture ladder (`REQ-478`) |
| `src/design_run/gate.rs` | `coverage_moved`'s `InquiryMap` arm gains the two projections; `initial-concerns-recorded` becomes a single-act rule; `blocking_inquiries_open` derives from the nodes with the legacy-act fallback; `ActKind::BlockingSetDeclared` and `Cause::ConfirmationStale` retire |
| `src/design_run/submission.rs` | `AgentAct::BlockingSetDeclared` retires |
| `src/design_run/attestation.rs` | `ContentCoverage`: the carried-keys material comparison plus the full-set blocking comparison, beside `diff` |
| `src/design_run/admission.rs` | the blocking-set-names-known-nodes fault retires with the act |
| `src/design_run/tests.rs` | the flipped pin (`stale_conjunct_does_not_satisfy` becomes VT-1), and the new criteria beside it |
| `src/design_run/payload_contract.rs`, `install/design-payload-contract.md` | the retired act and the new node field |
| `install/design-prompts/conditions/initial-concerns-recorded.md` | the single-act shape, with no separate set declaration to confirm |
| `install/design-prompts/conditions/blocking-set-current.md` | **not created** — the condition this design first proposed is dropped |
| `install/design-run-stages.md` | regenerated mirror of the condition table |

The design-target selectors this section commits to: `src/design_run/**`,
`install/design-prompts/conditions/**`, `install/design-payload-contract.md`,
`install/design-run-stages.md`.

<!-- doctrine:section sec-6 -->
## Verification

- **VT-1** — a declaration adding a node after `user-accepts-sufficiency` is attested does
  **not** void it; a node declared `blocking: true` **does** stale `initial-concerns-recorded`
  (the user is re-faced); a node declared `blocking: false` does not; and
  `blocking-inquiries-dispositioned` still reports unsatisfied for an open node with
  `blocking = true`. The derived half demonstrably not exempted.
- **VT-2** — `needs: null` clears and emits one `NeedsRemoved` per removed edge; `null` on an
  edge-free node emits no rows and claims no change rows; `needs: []` behaves identically.
- **VT-3** — the pre-change behaviour is pinned *first*: `stale_conjunct_does_not_satisfy`
  stays green until the change is deliberate, then flips against VT-1 rather than being
  deleted. The same for the contract-block render pins and the regenerated stage table.
- **VT-4** — a move still re-faces for a **covered** node (re-parenting voids
  `initial-concerns-recorded`); a node added after the act and then re-parented does **not**
  re-face, which is `F-1`'s scoping pinned as behaviour rather than left implicit.
- **VT-5** — a creation omitting `blocking` is refused; a flipped judgement emits exactly one
  `NodeBlockingChanged` row; and a snapshot whose nodes carry no judgement yields the stored
  act's set through the legacy fallback.
- **VA** — on a real run, a non-blocking addition does not re-face the human gates;
  re-measured against `RFC-031`'s fitness measure (nodes and edges added after
  `user-accepts-sufficiency`).

<!-- doctrine:section sec-7 -->
## Risks, residuals and deferred debt

- **Loosening invalidation is a truthfulness change** — `RFC-031` T1's class inverted, where
  a condition that should have invalidated and did not becomes a silent lie. VT-1's blocking
  half and VT-4 are the guards, and they are criteria rather than intentions.
- **The blocking set changes representation, not just storage.** `AgentAct::BlockingSetDeclared`
  and `Cause::ConfirmationStale` retire, and the payload contract is published. The
  compatibility path is read-side: a snapshot whose nodes carry no judgement reads the stored
  act, so no live run's meaning moves under it (`DEC-059`).
- **`F-1`'s scoping is a deliberate narrowing.** A node added after the accepting act and
  then moved or re-worded does not re-face the human. That is the coherent reading: the act
  was never shown that node. It is pinned by VT-4 so it cannot drift silently.
- **The condition table loses a row rather than gaining one**, and `blocking-set-current` —
  the ninth condition the earlier draft proposed — is not built.
- **Deferred, by name.** Backward cascade (`IMP-386`, gated on `QUE-218`); surfacing the map
  (`ISS-299`); the derived traversal cursor (`IMP-389`); the requirement-tier statement of the
  map's dynamic mode (`IMP-471`); and whether a traversal-only apply owes a change row
  (`IDE-057`).

