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

The behaviour is deliberate, not accidental: `UserAcceptsSufficiency`'s own comment says
"a re-seeded or materially changed graph must unmake an acceptance given over the old one".
This design keeps that intent and separates it from a case it was never meant to cover.

**The invariant.** *An addition is not a change to what was seen; a move or a re-word is.*

Three facts make it implementable:

1. `NodeMaterial` is question, provenance, parent, needs, seq — lifecycle and disposition
   excluded, so settling a question was already exempt, pinned by
   `node_material_ignores_progress_and_observes_shape`.
2. The two staleness mechanisms are independent: `CoverageStale` (a coverage diff) and
   `ConfirmationStale` (the digest link between the user's `graph-reviewed` and the agent's
   `blocking-set-declared`).
3. That digest binds the declared **set**, not its coverage — pinned by
   `stale_conjunct_does_not_satisfy`, which re-declares over a changed map with the same
   set and asserts that the confirmation link does not move.

So the design narrows what the *attested* judgements bind to, and gives the map-moved
signal its own home: one condition the **agent**, not the user, must satisfy.
(`DEC-300`, `DEC-301`.)

<!-- doctrine:section sec-2 -->
## Narrowing the attested binding

`coverage_moved`'s `Coverage::InquiryMap` arm diffs the carried `CoveredSet::Nodes`
against `materials()`. `ContentCoverage::diff` reports a joiner, a leaver and a changed
value alike, because it walks the union of keys.

The change is one arm of one function: for the two attested rows the comparison runs
**over the keys the act carried**, not the union. A key an act never covered cannot be a
change to what it covered.

- **Invisible:** a node added after the act was recorded; a `needs` edge added from a node
  the act did not cover.
- **Still stale:** a change to any node the act *did* cover — a re-worded question, a
  re-parent, an added or removed `needs` edge on a covered node — and any leaver among them.

That asymmetry is the point, and it is why "narrow" is not "weaken": the predicate gets
*sharper*. "This is not the content I was given" replaces "this is not the whole map".

`blocking-inquiries-dispositioned` is untouched. It is derived over the recorded
dispositions and must keep refusing advance while a declared blocking node is open.
Nothing here exempts it.

**What still re-faces the human.** A move (re-parent) and a re-word, because both change
content the user saw — so `DEC-062`'s "moving" is read narrowly rather than amended
(`DEC-301`). Pinning is traversal state; deferring and pruning are lifecycle. All three were
already exempt, which is why `DEC-062`'s five operations reduce to one residue.

<!-- doctrine:section sec-3 -->
## The growth obligation

Narrowing alone opens a hole. `blocking-inquiries-dispositioned` quantifies over the
**declared** blocking set, and its doc comment records why that is sound today:

> *It cannot be gated on a set nobody declared: `initial-concerns-recorded` is
> `Reach::Cumulative` and carries the declaration, so this edge and every edge above it
> already require one. A declaration whose map has moved goes stale there rather than being
> silently re-read here.*

The attested row's staleness **was** the mechanism that forced re-declaration. Remove it and
a question added late can be blocking and unnoticed. So the map-moved obligation gets its
own condition:

**`blocking-set-current`** — `Attested([BlockingSetDeclared(Agent)])`,
`binding: Coverage::InquiryMap` (the full union comparison), `reach: Cumulative`.

This is a *split*, not a new kind of thing: the agent's half of `initial-concerns-recorded`
moves out from under the user's judgement. Three properties follow.

- **The agent clears it, the user does not.** Add a node and the next edge refuses until the
  blocking set is declared over the current map. No human act.
- **Re-declaring an unchanged set is free.** The confirmation digest binds the set, so the
  same set leaves `ConfirmationStale` silent. This is the escape hatch that makes the
  obligation affordable, and it is why `DEC-062`'s complaint does not simply move.
- **A changed set re-shows the delta.** Declaring a genuinely new blocking node moves the
  digest, `ConfirmationStale` fires, and the user is asked to review exactly what changed.
  `REQ-425` ("material restructuring is surfaced rather than absorbed silently") is
  discharged by machinery that already exists.

**A refinement of `DEC-300`'s wording, stated here rather than hidden.** `DEC-300` says a
*derived* condition; this is an *attested* one. The substance is unchanged — the engine
notices mechanically, the agent must re-declare, and an unchanged set costs no human act —
and `DEC-126`'s ledger prefers attested where an act exists. It also avoids a new
`EngineSource` and a new derivation rule, reusing the coverage machinery already in place.
Flagged for review rather than assumed.

Naming: `blocking-set-current` parallels `blocking-inquiries-dispositioned`. One says the
declared set is current; the other says its members are answered.

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
- A clearing is a real mutation: it advances the revision (`REQ-429`/`REQ-430`). A no-op must
  not invent one.

`SPEC-029` Responsibilities[13] and `DEC-063` already require this, so the contract does not
change. The code was the deviation.

<!-- doctrine:section sec-5 -->
## Code impact

| path | what changes |
|---|---|
| `src/design_run/gate.rs` | the two attested rows' binding; the new `blocking-set-current` row in the `condition_vocabulary!` table; `coverage_moved` gains the carried-keys arm |
| `src/design_run/attestation.rs` | `ContentCoverage`: the comparison that ignores joiners, beside `diff` |
| `src/design_run/submission.rs` | the new row's key-home / state-axis entry, where one is owed |
| `src/design_run/run.rs` | `declare_node`'s `needs` block: `Sparse::Null` |
| `src/design_run/tests.rs` | the flipped pin: `stale_conjunct_does_not_satisfy` becomes the VT-1 assertion, and the new criteria land beside it |
| `install/design-run-stages.md` | regenerated mirror of the condition table, golden-pinned by the render tests |

The design-target selectors this section commits to: `src/design_run/gate.rs`,
`src/design_run/attestation.rs`, `src/design_run/run.rs`, `src/design_run/tests.rs`,
`install/design-run-stages.md`.

<!-- doctrine:section sec-6 -->
## Verification

- **VT-1** — a declaration adding a node after `user-accepts-sufficiency` is attested does
  **not** void it; `blocking-set-current` reports unmet instead; re-declaring the same set
  clears it with no user act; and `blocking-inquiries-dispositioned` still reports unsatisfied
  for an open declared blocking node. The derived half demonstrably not exempted.
- **VT-2** — `needs: null` clears and emits one `NeedsRemoved` per removed edge; `null` on an
  edge-free node emits no rows and claims no change; `needs: []` behaves identically, which
  `apply_collection`'s existing equality already guarantees.
- **VT-3** — the pre-change behaviour is pinned *first*: `stale_conjunct_does_not_satisfy`
  stays green until the change is deliberate, then flips against VT-1 rather than being
  deleted. The same for the contract-block render pins and the regenerated stage table.
- **VT-4** — a move still re-faces: re-parenting a covered node voids
  `initial-concerns-recorded`, which is `DEC-301`'s narrowed reading pinned as behaviour.
- **VA** — on a real run, a node added after sufficiency is accepted does not re-face the
  human gates; re-measured against `RFC-031`'s fitness measure (nodes and edges added after
  `user-accepts-sufficiency`).

<!-- doctrine:section sec-7 -->
## Risks, residuals and deferred debt

- **Loosening invalidation is a truthfulness change** — `RFC-031` T1's class inverted, where
  a condition that should have invalidated and did not becomes a silent lie. VT-1's derived
  half and VT-4 are the guards, and they are criteria rather than intentions.
- **Seventeen live snapshots change semantics underneath.** The change is read-side only; no
  stored shape moves, and `DEC-059` prefers that. Nothing becomes unreadable
  (`mem.fact.design-run.snapshot-outlives-the-binary`).
- **`blocking-set-current` is a ninth condition** on a table of eight. `DEC-126`'s ledger is
  2 derived / 7 attested / 0 claimed today; an attested row keeps that shape, where a derived
  one would have changed it.
- **Deferred, by name.** Backward cascade (`IMP-386`, gated on `QUE-218`); surfacing the map
  (`ISS-299`); the derived traversal cursor (`IMP-389` — the cursor went `STALE` twice during
  this very run, once when its own node resolved, and no verb derives the next); the
  requirement-tier statement of the map's dynamic mode (`IMP-471`); and whether a
  traversal-only apply owes a change row (`IDE-057`, observed during this run).

