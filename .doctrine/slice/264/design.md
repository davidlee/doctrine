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
record; scoping it to covered nodes is an explicit amendment **to `DEC-301` itself**, not a
reading the design takes on its own (`RV-386` `F-1`, `F-12`) — the same pattern as `DEC-300`.

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
   `ConfirmationStale` (which `sec-3` keeps only as a frozen legacy read).
3. Redeclaring a node with an unchanged judgement is free; flipping it is not.

So the design narrows what the *attested* judgements bind to, scopes the move rule to
covered nodes, and moves the blocking set from a free-standing act onto the node.
(`DEC-300` amended by `DEC-302`, `DEC-301` scoped.)

<!-- doctrine:section sec-2 -->
## Narrowing the attested binding

`coverage_moved`'s `Coverage::InquiryMap` arm diffs the carried `CoveredSet::Nodes`
against `materials()`. `ContentCoverage::diff` reports a joiner, a leaver and a changed
value alike, because it walks the union of keys.

The two rows stop sharing one comparison, because they judge different things
(`RV-386` `F-6`). Both still carry the same stored shape, `CoveredSet::Nodes` — no stored
act moves — and differ only in the `Coverage` their rule names:

| rule | coverage | compares |
|---|---|---|
| `user-accepts-sufficiency` | `InquiryMap` | material, over carried keys |
| `initial-concerns-recorded` | `ReviewedGraph` *(new)* | material over carried keys, **and** blocking membership over the full set |

- **Material, over carried keys.** A key the act never covered cannot be a change to what
  it covered. A covered node whose material moved is stale; an uncovered node is not.
- **Blocking membership, over the full set** — `ReviewedGraph` only. The ids whose
  *effective* judgement is blocking (`sec-3`) are compared in full, because a *new*
  blocking node is exactly the event that must reach the user. A key absent from the
  carried map reads as not blocking, so a new non-blocking node is no change.

Why sufficiency does not take the second projection: the user who sees a new blocking
question is re-faced by `initial-concerns-recorded`, which is cumulative and so already
guards every edge `user-accepts-sufficiency` guards; and `blocking-inquiries-dispositioned`
holds the edge until the question is answered. Re-asking "has enough been asked" on top
would re-face the same person twice for one event — the cost `OQ-3` ruled out. A flip on a
*covered* node is different in kind and does stale both: it changes content each act was
given over, and sufficiency was accepted in view of those marks.

**One predicate, every reader** (`RV-386` `F-13`). `coverage_moved` and the change log's
`live_acts` both read coverage currency, and `CoveredSet::moved`'s doc comment makes them
agree by construction. That stays true only if the narrowing lives *in* the shared
predicate: `CoveredSet::moved` takes the `Coverage` the act's rule names (looked up from its
`ActKind`), so `live_acts` emits `ActInvalidated` exactly when the gate would call the act
stale — never for a non-blocking addition the gate keeps current. A union-key `diff` stays
for the whole-map users (`is_current`: the integrated review and lock acceptance), which
this design does not touch.

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

- On the wire, **required on creation**, optional on update (omission persists, a value
  replaces). A creation that omits it is refused, and so is `null` in either state: a
  judgement can be changed but not withdrawn, and silently reading `null` as omission is
  `SL-259`'s disease.
- **One key, two homes** (`RV-386` `F-9`). `blocking` is already a finding key —
  create-only, absent reads non-blocking. The wire-key table holds one home per key today,
  so `inert_key` would refuse it at an inquiry. The table gains a key with more than one
  home, each row pairing a kind with its own state rule: `Finding` × `Only(Absent)`,
  unchanged, and `Inquiry` × `EitherState`, required at `Absent`. The kind axis refuses
  only when **no** row for the key names the subject's kind. `design contract` renders
  both homes. The finding's `null`-reads-as-absent is the same disease on the older home;
  it is captured as follow-up work, not fixed here.
- Stored as `Option<bool>` with `serde(default)`. `None` means **unjudged**, and only a
  node that predates this change can hold it (`sec-3` *Compatibility*).
- It is a member of `NodeMaterial`. A flip on a covered node is a material change and
  re-faces the user through `sec-2`'s first projection. A delegation assigned over the
  node goes stale on a flip too — `is_stale` compares the whole stored node, which is right:
  the obligation changed.
- A flip on an uncovered node moves only the set projection — correct, because the set is
  what the user is asked about.
- A flip is a recorded mutation, so it emits one `NodeBlockingChanged` row (`REQ-478`);
  creation carries its `blocking` inside `NodeCreated` and owes no second row.

**Every creation path judges** (`RV-386` `F-10`). `declare_node` is not the only way a node
is born: `design start --from-design` seeds shaping questions and imported open-question
prose through `seed_node`, bypassing it. Import seeds **`blocking: true`**. An open
question the engine found for the agent is conservatively blocking until the agent says
otherwise, so the omission defaults to *visible*, never to *free*. It costs nothing: import
precedes every user act, so no act covers the node yet, and the flip to `false` is a
recorded mutation the user sees in the graph review. `InquiryNode`'s constructors take the
judgement as a parameter, so no creation path can omit it by construction rather than by a
check.

**The set derives.** A node's **effective** judgement is its own when it has one, and
otherwise its membership in the stored legacy set (*Compatibility*, below).
`blocking_inquiries_open` stops reading a declaration and filters the nodes: effective
judgement blocking and lifecycle not `Resolved`. `blocking-inquiries-dispositioned` keeps
its contract (`derived`, `engine(dispositions)`, cumulative) and changes only its source.
`ReviewedGraph`'s second projection compares the same effective set, so the gate and the
coverage cannot disagree about which nodes block.

**The growth obligation closes without a new condition.** `initial-concerns-recorded`
becomes a single-act rule — `Attested([GraphReviewed(User)])` with `Coverage::ReviewedGraph`.
Its second act, the agent's `blocking-set-declared`, and the `confirms` link from the user's
act to it, retire from the rule with the set (a stored link is still read — below). What
replaces the link is `sec-2`'s second projection:

- add a node with `blocking: false` → the set is unchanged → nothing re-faces;
- add a node with `blocking: true` → the set changes → the user's `graph-reviewed` goes
  stale → the user is asked to see it, tree and blocking marks together.

So a question the agent judges blocking cannot pass unseen — `SL-264`'s `R4` closed by
construction rather than by a ninth condition (`RV-386` `F-2`). It closes against the
**user**, not only the agent, which is what `OQ-4`'s "blocking, not a warning" asked for.
And `DEC-126`'s kind axis is untouched: the condition stays `Attested`, and no
`EngineSource` or derivation rule is added.

**`DEC-121`'s two actors survive; its two acts do not** (`RV-386` `F-14`). `DEC-121` makes
initial concerns *two acts*: the user reviews the graph, and the agent declares which
questions block. Both judgements remain, by the same actors — the agent's is now written
per node, at creation and by redeclaration, and the user's `graph-reviewed` binds it
through `ReviewedGraph`. What changes is that the agent's half is node state rather than a
recorded act, so the rule is a single act. That departs from `DEC-121`'s letter — its
rationale for two acts was that a refusal can name the missing half, and a node without a
judgement is now refused at the declaration instead — so `DEC-121` is **amended
explicitly** rather than read around, and `DEC-126`'s table row for the condition with it.

**Retired from writing, kept for reading** (`RV-386` `F-7`). No new
`blocking-set-declared` can be submitted: the key leaves the payload contract and the
parser, and the admission fault that checked its node ids goes with it. But
`AgentAct::BlockingSetDeclared` and `ActKind::BlockingSetDeclared` **stay in their enums**:
most stored snapshots hold the act, `AgentAct` derives `Deserialize`, and the
snapshot is parsed whole — deleting the variant fails the parse before any fallback runs.
Each is documented as *legacy, read-only*, and no rule requires either.

The `confirms` link leaves the **rule** but not the **read**. A stored `graph-reviewed`
carries the digest of the declaration it confirmed (`CheckpointAct::confirms`). If that
digest no longer matched the live declaration at upgrade time, the act was
`ConfirmationStale` — the agent changed the set after the user saw it. Reading only
`ReviewedGraph` would turn that act current, because its carried effective set is computed
from the *current* legacy set. So the check becomes **carried-driven**: an act that carries
a `confirms` digest is still compared against the stored declaration, and
`Cause::ConfirmationStale` stays to report it. Since no new declaration can be written, the
comparison is frozen at upgrade — it can only preserve a verdict, never create one. An act
recorded after the change carries no digest and never reaches it.

**Compatibility, per node, not per run** (`RV-386` `F-8`). A node's effective judgement is
its own `Some(bool)`, else its membership in the stored `BlockingSetDeclared` set. The
fallback is resolved per node, so a run where some nodes are judged and some are not keeps
every legacy blocker: `SL-252`'s open `inq-5` and `inq-11` stay blocking after any other
node is judged. A legacy node leaves the fallback only by being judged itself. Read-side
only, no stored shape moves (`DEC-059`). Nothing flips on read: an unjudged node's stored
material carries `None` on both sides of every carried comparison, and its effective
judgement is the one the stored set already gave it. Judging a covered legacy node *is* a
material change, and re-faces once — disclosed in `sec-7`.

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
| `src/design_run/inquiry.rs` | `InquiryNode` / `NodeMaterial` gain `blocking: Option<bool>` (`serde(default)`, `None` = unjudged legacy), a material member; constructors take the judgement; the effective-judgement read with the legacy set |
| `src/design_run/run.rs` | `declare_node`'s `blocking` arm — refused when absent on creation or `null`, persisted on omission, replaced on value — and the `NodeBlockingChanged` row on a flip; import's `seed_node` callers pass `blocking: true`; `live_acts` passes each act's rule `Coverage` to `CoveredSet::moved`; the covered-set constructor (`run.rs:841`) builds `Nodes` for `ReviewedGraph` too |
| `src/design_run/change_log.rs` | `ChangeEvent::NodeBlockingChanged`, emittable and driven by the fixture ladder (`REQ-478`) |
| `src/design_run/gate.rs` | `Coverage::ReviewedGraph`; `coverage_moved` reads through `CoveredSet::moved`; `initial-concerns-recorded` becomes a single-act rule over `ReviewedGraph`; `blocking_inquiries_open` derives from effective judgements; the `confirms` requirement leaves the rule while a stored `confirms` digest is still read, frozen, through `Cause::ConfirmationStale`; `ActKind::BlockingSetDeclared` stays, legacy read-only |
| `src/design_run/submission.rs` | the `blocking-set-declared` key leaves the parser; the wire-key table admits a key with two homes (`blocking`: `Finding` create-only, `Inquiry` either state) and `inert_key` / the state axis honour every home; `null` refused at the inquiry home |
| `src/design_run/attestation.rs` | `CoveredSet::moved` takes a `Coverage` — carried-keys material for `InquiryMap`, plus the full-set effective-blocking comparison for `ReviewedGraph`; `diff` stays for the whole-map `is_current` users; `AgentAct::BlockingSetDeclared` stays, legacy read-only |
| `src/design_run/admission.rs` | the blocking-set-names-known-nodes fault retires; `coverage_fault` accepts `Nodes` for both `InquiryMap` and `ReviewedGraph` |
| `src/design_run/tests.rs` | the flipped pin (`stale_conjunct_does_not_satisfy` becomes VT-1), and the new criteria beside it |
| `src/design_run/payload_contract.rs`, `install/design-payload-contract.md` | the retired act, the new node field and its two-home key |
| `install/design-prompts/conditions/initial-concerns-recorded.md` | the single-act shape, with no separate set declaration to confirm |
| `install/design-prompts/conditions/blocking-set-current.md` | **not created** — the condition this design first proposed is dropped |
| `install/design-run-stages.md` | regenerated mirror of the condition table |

The design-target selectors this section commits to: `src/design_run/**`,
`install/design-prompts/conditions/**`, `install/design-payload-contract.md`,
`install/design-run-stages.md`.

<!-- doctrine:section sec-6 -->
## Verification

- **VT-1** — a declaration adding a node after `user-accepts-sufficiency` is attested does
  **not** void it, **whether the node is declared `blocking: true` or `false`**
  (`RV-386` `F-6`); a node declared `blocking: true` **does** stale
  `initial-concerns-recorded` (the user is re-faced); a node declared `blocking: false` does
  not; and `blocking-inquiries-dispositioned` still reports unsatisfied for an open node with
  `blocking = true`. The derived half demonstrably not exempted. The change log agrees with
  the gate in each case: an `ActInvalidated` row exactly for the act the gate calls stale,
  and none for a non-blocking addition (`F-13`).
- **VT-2** — `needs: null` clears and emits one `NeedsRemoved` per removed edge; `null` on an
  edge-free node emits no rows and claims no change rows; `needs: []` behaves identically.
- **VT-3** — the pre-change behaviour is pinned *first*: `stale_conjunct_does_not_satisfy`
  stays green until the change is deliberate, then flips against VT-1 rather than being
  deleted. The same for the contract-block render pins and the regenerated stage table.
- **VT-4** — a move still re-faces for a **covered** node (re-parenting voids
  `initial-concerns-recorded`); a node added after the act and then re-parented does **not**
  re-face, which is `F-1`'s scoping pinned as behaviour rather than left implicit.
- **VT-5** — a creation omitting `blocking` is refused, and `blocking: null` is refused in
  either state; a flipped judgement emits exactly one `NodeBlockingChanged` row; `blocking` is
  admitted at an inquiry in either state and still create-only at a finding (`F-9`).
- **VT-6** — legacy compatibility, over a fixture holding a stored `blocking-set-declared`
  act (`F-7`, `F-8`): the snapshot **parses**; with no node judged, the effective set equals
  the stored act's; after judging one *other* node `blocking: false`, every unjudged legacy
  blocker is still open in `blocking-inquiries-dispositioned` (the mixed regime); and no
  condition's verdict differs from the pre-change binary's on read **except** an act stale only
  by an addition, which now reads current — the slice's intended change; a stored
  `graph-reviewed` already `ConfirmationStale` stays stale. A negative control
  deletes the variant and asserts the parse fails, so the criterion can see the defect.
- **VT-7** — every creation path judges (`F-10`): `design start --from-design` seeds shaping
  and imported-prose questions with `blocking: true`, and they appear in the effective set.
- **VA** — on a real run, a non-blocking addition does not re-face the human gates;
  re-measured against `RFC-031`'s fitness measure (nodes and edges added after
  `user-accepts-sufficiency`).

<!-- doctrine:section sec-7 -->
## Risks, residuals and deferred debt

- **Loosening invalidation is a truthfulness change** — `RFC-031` T1's class inverted, where
  a condition that should have invalidated and did not becomes a silent lie. VT-1's blocking
  half and VT-4 are the guards, and they are criteria rather than intentions.
- **The blocking set changes representation, not just storage.** `blocking-set-declared`
  leaves the published payload contract; its enum variants stay as legacy read-only, and
  `Cause::ConfirmationStale` survives only as a frozen legacy read. The compatibility path is
  read-side and per node, so
  no live run's meaning moves under it (`DEC-059`).
- **Judging a covered legacy node re-faces once.** Its stored material goes from `None` to
  `Some`, which is a change to covered content even when the judgement matches the legacy
  set. Honest rather than free, confined to runs that predate this change, and cheaper than
  a special case that treats a matching judgement as no change.
- **Import's conservative default.** Seeding every imported question `blocking: true`
  front-loads flips onto the agent during `exploring`. Deliberate: an omission must default to
  visible.
- **Sibling left in place.** A finding's `blocking: null` still reads as absent — the older
  home of the same key, outside this slice's surface; captured as follow-up work.
- **`F-1`'s scoping is a deliberate narrowing.** A node added after the accepting act and
  then moved or re-worded does not re-face the human. That is the coherent reading: the act
  was never shown that node. It is pinned by VT-4 so it cannot drift silently.
- **The condition table loses a row rather than gaining one**, and `blocking-set-current` —
  the ninth condition the earlier draft proposed — is not built.
- **Deferred, by name.** Backward cascade (`IMP-386`, gated on `QUE-218`); surfacing the map
  (`ISS-299`); the derived traversal cursor (`IMP-389`); the requirement-tier statement of the
  map's dynamic mode (`IMP-471`); and whether a traversal-only apply owes a change row
  (`IDE-057`).

