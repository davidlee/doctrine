# Sketch: projection and token bounds

PHASE-02 design gate (EN-2). Authored artefact under `.doctrine/slice/233/`,
durable and diffable — not runtime scratch. It answers the seven questions
`plan.toml` PHASE-02 EN-2 enumerates, and it is the surface RV-320 was raised
against.

design.md §9.3 asserts that named limits bound the normal `TurnEnvelope`
against a large-run fixture. The algorithm behind that assertion does not exist
yet. This sketch is that algorithm.

> **Revision 8** (2026-07-29), prompted by no review round. Rev 7 asserted that
> ADR-001's existing layering test could enforce "storage may not depend on
> `ENVELOPE_*`". It cannot: `layering.toml` models one axis — altitude — and
> pure constants classify **leaf**, the tier everything may depend on, so the
> checker would have licensed the violation. The **seventh** instance of the
> class, caught by the provenance rule before a reviewer found it. Replaced by
> **Rust privacy** — constants private to the rendering module make a storage
> reference a compile error, which is stronger and needs no new machinery. The
> limit is named rather than papered over: privacy does not catch a copied
> literal, and nothing here does. See § Revision history.
>
> **Revision 7** (2026-07-29), after RV-320's fourth verify round. Rev 6 set an
> explicit falsifier — a sixth instance the layer rule does not catch means
> widen the rule, never patch the instance — and round 4 found one inside rev
> 6's own fix: `CHANGE_REASON_INPUT_BYTES` = 2048 was correctly layered,
> correctly enforced, and **entirely underivable**. So the mechanism becomes a
> **pair**: the layer rule (where a bound acts) plus **the provenance rule**
> (every bound states what derives it; an underivable bound is removed, not
> guessed). Applying provenance deletes 2048 — the reason is now stored
> unbounded, consistent with section bodies. Enforcement also goes structural:
> the layer boundary becomes a **module dependency rule** on ADR-001's existing
> layering test rather than a grep, and every run-local id is built through
> **one validating constructor** rather than per-path checks. **No byte
> arithmetic moves.** See § Revision history.
>
> **Revision 6** (2026-07-29), after RV-320's third verify round. F-7 contested
> a third time on three counts; all three verified against the text and
> conceded. The first is F-1's defect class a **fifth** time — rev 4 stated
> "a projection bound must never propagate into the record it projects from"
> and then bounded the stored reason "to `ENVELOPE_REASON_BYTES`" in the next
> table. Three rounds of patching the named instance have not held, so rev 6
> states the mechanism instead: § *The layer rule*. A constant's prefix
> declares its layer; a layer binds only its own artefacts; identity and closed
> vocabularies are bounded at **admission** (refused), never at **emission**
> (truncated). It retro-catches all five recurrences and is mechanically
> checkable. Three caps are renamed `DESIGN_*` at unchanged values; the stored
> reason gains a domain bound; ids stop truncating. **No byte arithmetic moves.**
> See § Revision history.
>
> **Revision 5** (2026-07-29), before RV-320's third verify round, and prompted
> by no finding. Rev 4 left `ENVELOPE_NORMAL_BUDGET_BYTES` in front of the
> reviewer as this sketch's open question. It is decided here instead:
> **16 KiB → 24 KiB**. Twice the old ceiling had pressed on design questions it
> should not have been deciding, and a reviewer should not be asked to settle
> the author's constant. Three derived figures move with it (headroom, the token
> upper bound, and a stale expected-case estimate rev 4 left behind); the
> saturated table, every per-field cap, the eviction ladder, and all of
> `plan.toml` are untouched. F-7's answer is unaffected. See § Revision history.
>
> **Revision 4** (2026-07-29), after RV-320's second verify round. F-2 and F-3
> verified; F-7 contested again and rightly. Rev 3 had asserted a payload size
> instead of deriving it — F-1's defect a third time, now in the justification
> rather than the design — and had applied a *projection* bound to the *stored*
> record, truncating reasons and fingerprints on disk to buy space in a rendering
> nobody had asked for. Rev 4 separates the stored row from the rendered row,
> derives every byte, and turns the missing containment check into a test
> obligation. See § Revision history.
>
> **Revision 3** (2026-07-29), after RV-320's verify round. Four findings were
> verified and three contested: F-2, F-3, and F-7. All three contests were
> substantively right, and two of them found rev 2's *fix* reintroducing the
> defect in a new place — an exclusion that made the overflow rule partial, and
> a class table that left overlapping candidates unranked. F-7 found two named
> constants contradicting each other by 112 bytes. What changed is in
> § Revision history.
>
> **Revision 2** (2026-07-29), after RV-320's seven blockers. The first draft
> claimed a constant-size envelope from *cardinality* caps alone, which does not
> follow; asserted a token worst case from an average bytes-per-token ratio;
> gave a drop end for two of six bounded lists; and made an R2 distinguishability
> claim that its own selection stage falsified. What changed is recorded in
> § Revision history at the end, so the disposition of each finding is checkable
> against the document rather than against my summary of it.

## Why this gate exists here

R1 — *protocol ceremony exceeds its value* — is won or lost in this document.
Every other risk in design.md §8 is about correctness; R1 is about whether the
thing is worth using at all.

The circularity is real and acknowledged (plan.md § *The three design gates*):
this sketch bounds a type PHASE-02 has not defined yet, and PHASE-02's type is
shaped by the budget this sketch sets. Something goes first, and rework of the
pure model is the more expensive direction because everything above it inherits
the change.

## The governing claim

**The normal envelope's rendered size is bounded by a named constant, and that
bound is independent of run size.**

Two things make it true, and RV-320 F-1 established that the first alone does
not:

1. **Cardinality bounds** — every collection has a named maximum entry count.
2. **Encoded-size bounds** — every variable-length scalar has a named maximum
   in *encoded bytes of the budgeted rendering*, not characters.

And because a table of worst cases is a prediction rather than a guarantee, a
third mechanism makes the ceiling true by construction:

3. **Measure and evict** — the projection renders, measures, and evicts along a
   defined ladder until it fits `ENVELOPE_NORMAL_BUDGET_BYTES`, or refuses. The
   ceiling is enforced, not merely predicted.

A run with eight inquiry nodes and a run with eight hundred project the same
bound. Only `show --full` scales.

## The budgeted rendering

Bounds are stated against **one** rendering: `design show --format prompt`, the
projection that enters an agent's context and the only one R1 is about. `json`
and `status` derive from the same `TurnEnvelope` (DEC-064) and are *not*
budgeted — `json` is a machine surface whose framing overhead differs, and
`status` is for humans at a terminal.

Within the budgeted rendering:

- **Encoded bytes** means UTF-8 bytes **after** any escaping the rendering
  applies. A cap is enforced on the emitted form, never on the source string.
- Truncating an `ENVELOPE_*`-capped scalar cuts on a UTF-8 character boundary
  and appends an elision marker; the cap includes the marker, and the emission
  path subtracts the marker's encoded length rather than assuming it. The
  marker is U+2026 HORIZONTAL ELLIPSIS — three UTF-8 bytes, not one.
- **Identity and closed vocabularies are never truncated.** Ids, stage names,
  and event names are bounded at admission by their `DESIGN_*` constants, so
  they are within their bound before rendering ever sees them and the emission
  path has no cap to apply. This exemption is not an optimisation — truncating
  an id makes two distinct subjects render identically, which is the failure
  RV-320 F-7's third round found. See § *The layer rule*.
- Numeric fields are bounded by their wire type's maximum rendered width, which
  is a constant per type — this is what closes F-1's observation that counts and
  revisions grow in width.

## (a) The named limits

All constants live in the pure layer under `src/design_run/`, single-sourced per
STD-001, in the shape the corpus already uses (`RETRIEVE_LIMIT_DEFAULT`,
`FETCH_LIMIT`, `CLOBBER_RENDER_CAP`).

**Cardinality caps**

| constant | value | bounds |
|---|---:|---|
| `ENVELOPE_FRONTIER_NODES` | 7 | nearby-frontier entries |
| `ENVELOPE_ACTIVE_PATH_DEPTH` | 6 | active-path entries |
| `ENVELOPE_BLOCKERS` | 5 | blocker entries |
| `ENVELOPE_CHANGE_ROWS` | 10 | material-change delta rows |
| `ENVELOPE_DURABLE_RECORDS` | 8 | linked DEC/QUE/ASM references |
| `ENVELOPE_SECTION_ROWS` | 16 | section / review state rows |

Constants are grouped **by the layer they bind**, and the prefix carries the
layer. Rev 6 makes that grouping load-bearing rather than cosmetic — see
§ *The layer rule*.

**Emission caps — `ENVELOPE_*`** (bytes of the budgeted rendering, and nothing
else). Every one of these may truncate, because every one bounds gracefully
degrading prose.

| constant | value | applies to |
|---|---:|---|
| `ENVELOPE_QUESTION_BYTES` | 160 | an inquiry node's question |
| `ENVELOPE_LABEL_BYTES` | 120 | any title: record, section, fragment name |
| `ENVELOPE_REASON_BYTES` | 240 | next obligation, blocker reason, *live* regression reason |
| `ENVELOPE_CHANGE_REASON_BYTES` | 96 | a regression reason as *rendered* on a change row |
| `ENVELOPE_PAYLOAD_BYTES` | 160 | one *rendered* change row's event payload |
| `ENVELOPE_DECLARATION_EXAMPLE_BYTES` | 1024 | the worked next-mutation example |
| `ENVELOPE_NORMAL_BUDGET_BYTES` | 24576 | the entire budgeted rendering |

**Admission bounds — `DESIGN_*`** (bytes of a value at the moment it is
created or accepted). **None of these ever truncates**; exceeding one is a
refusal at the boundary, because each bounds identity or a closed vocabulary,
and a truncated identity is a *wrong* identity rather than a shorter one.

| constant | value | bounds, at admission |
|---|---:|---|
| `DESIGN_ID_BYTES` | 32 | any run-local id at creation: node, section, gate, inquiry, attestation. Canonical entity refs (`SL-233`) are ≤ 9 B and fit trivially |
| `DESIGN_STAGE_LABEL_BYTES` | 16 | a stage name — closed vocabulary, longest `exploring` at 9 B |
| `DESIGN_EVENT_NAME_BYTES` | 32 | a change-event name — closed vocabulary, longest `section_fingerprint_changed` at 27 B |

**There is deliberately no bound on a stored regression reason.** Rev 6 added
`CHANGE_REASON_INPUT_BYTES` = 2048; rev 7 **removes it**, because nothing
derived 2048 and the provenance rule (below) says an underivable bound is
deleted rather than guessed. The positive argument needs no number: the
snapshot already stores unbounded prose — section bodies are the obvious case —
so a per-field prose bound on one field is special-casing without a rationale.
If storage volume ever needs bounding, that is a storage-model decision applying
uniformly to the snapshot, not a constant smuggled in beside a rendering cap.
The projection is safe regardless: the rendered reason is elided to
`ENVELOPE_CHANGE_REASON_BYTES` = 96 however long the stored one is.

**Storage bounds** (what the snapshot retains; no relation to any rendering)

| constant | value | bounds |
|---|---:|---|
| `CHANGE_LOG_REVISIONS` | 32 | past revisions retained in the snapshot |

**Display abbreviation** — not a cap, and called out separately because it is
the one place a value is deliberately narrowed at emission without being prose:

| constant | value | applies to |
|---|---:|---|
| `ENVELOPE_FINGERPRINT_SHORT_BYTES` | 12 | a fingerprint as rendered on a change row |

A fingerprint is a uniformly distributed digest, so a 12-hex-character prefix
is ~48 bits — collision-resistant enough to identify a section in a rendering
the reader can widen with one command, and the stored row keeps the whole
digest. This is an *abbreviation with a stated collision budget*, not a
truncation that loses recoverable information. It is named here so a later
reader does not have to decide whether it was an oversight. If the collision
budget is ever judged insufficient, the fix is a wider abbreviation, not a
different mechanism.

Reasoning for the numbers, since a number with no argument behind it is a magic
constant with a name:

- **7 frontier / 6 depth.** The frontier is the decision surface for *one* turn,
  not a listing of the map. Seven carries the cursor's children and its siblings
  without either crowding out the other; six levels of decomposition is deeper
  than a readable design tree, and above that the top of the path is context the
  stage and obligation already carry.
- **5 blockers.** More than five simultaneous blockers is a state the agent
  needs to look at whole — see (g).
- **10 change rows / 8 records.** Both sized to what is absorbable in one turn;
  8 matches the existing `FETCH_LIMIT` precedent.
- **16 section rows.** Sections scale with the design *document*, not the run.
  No design.md in this corpus reaches it.
- **160 question bytes.** §5.3 says the question is *concise*. One line, and now
  a line measured in the units the budget is actually spent in.
- **32 revisions.** A storage bound, not a projection bound — deliberately a
  different constant from `ENVELOPE_CHANGE_ROWS`. See (d).

**Names are contract; values are tuning.** A rename or removal changes the
envelope's shape and needs a plan revision. Retuning a value against fixture
evidence does not. Read the table as commitments to *bound a named thing*, with
the numbers as this sketch's best current answer.

## (b) What is dropped first, and how the drop is signalled

Every bounded list names a **retained end** and a **drop end**. RV-320 F-2 was
right that the first draft gave these for two lists and generalised from them.

| list | ordered by | retained | dropped |
|---|---|---|---|
| active path | root → cursor | the **cursor** end | the **root** end |
| frontier | the (c) rank order | highest-ranked | lowest-ranked |
| blockers | `needs` in-degree desc, then `seq` | most consequential | least |
| change delta | revision desc, then event index desc | **newest** | **oldest** |
| durable records | link revision desc, then id | most recently linked | least |
| section rows | outstanding review state first, then section order | earliest | latest |

The active path retains the cursor end and drops the root end: the decision is
local to the cursor, and the top of a decomposition is what stage and obligation
already convey. That is a *loss* with a rationale, which (g) now states.

**No drop is ever silent.** Every bounded field that omitted anything carries its
exact omitted count — `(+N more)` in the budgeted rendering, riding the
convention `CLOBBER_RENDER_CAP` already establishes, and `<field>_omitted: N` in
the JSON projection. One `truncated` flag names every field that bound.

### The no-drop set

Never truncated, because a truncated version breaks the protocol rather than
degrading it:

- run identity, revision, schema version;
- stage and the next closed obligation;
- the next-mutation contract and its worked example;
- the pinned slot (see (c));
- **every global total** (see (e), (g)) and every omitted count.

### Global overflow — the eviction ladder

The per-field caps do not by themselves guarantee the whole-envelope ceiling:
individually legal fields can collectively exceed it. So the projection measures
the rendered envelope and, while it exceeds `ENVELOPE_NORMAL_BUDGET_BYTES`,
evicts **one entry at a time** from the drop end, in this fixed field order:

1. section rows
2. durable records
3. change delta
4. blockers
5. frontier
6. active path

**The ladder covers every bounded list.** Rev 2 excluded the active path from it,
reasoning that it is the shortest list and that losing it costs orientation
disproportionately. RV-320 F-3's verify round showed that exclusion made the
overflow rule *partial*: the active path was in neither the ladder nor the
no-drop set, so a state where the ladder is exhausted and no-drop plus active
path still exceeds the ceiling had no defined outcome. The judgement was sound
but the mechanism was wrong — "costs the most to lose" is an argument for
ordering it **last**, not for removing it. As rung 6 it is evicted only after
every other list is empty, which honours the same intent and leaves the function
total. It drops from its own stated drop end (the root end, per the table above).

Each eviction increments that field's omitted count, so ladder evictions are
indistinguishable from cap evictions to the reader, which is the point: the
count is exact either way.

**The terminal rule.** With every bounded list on the ladder, exactly one
irreducible state remains: the **no-drop set alone** exceeds the ceiling. There
the projection refuses with a named error rather than emitting a quietly
malformed envelope. Ladder-exhausted-and-still-over is now that same state by
construction, not a separate unhandled one. That is a testable invariant, and the
fixture carries a case that trips it deliberately.

## (c) How the nearby frontier is selected

**Candidates** are nodes with lifecycle `open`, not derived-`blocked`, that fall
into one of the kinship classes enumerated in the rank table below.

**The rank table is the eligibility rule.** Rev 2 stated eligibility twice — once
as "within primary-parent-tree distance 2 of the cursor, or a direct `needs`
neighbour", and again as the rank table's classes — and RV-320 F-3's verify round
found the two disagree in both directions. Under any consistent edge-count metric
where a sibling is distance 2 (via the parent) and a grandchild is distance 2, an
ancestor-sibling is distance **3** (cursor → parent → grandparent → uncle) and a
nibling is distance **3** (cursor → parent → sibling → nibling). So the
distance-2 rule excluded rank 4, which is also the stated leaf fallback, and
excluded the niblings that rank 6's own parenthetical names.

Two definitions that must be kept in agreement is one more than the design needs.
The rank table is now the single closed enumeration and eligibility derives from
it: a node is a candidate if and only if it matches a class in the table. The
distance metric is gone rather than repaired, because it was only ever a lossy
paraphrase of the enumeration it contradicted.

**The pinned slot is not a frontier entry.** RV-320 F-3 found the first draft's
contradiction — a pinned node "always first, never dropped" against blocked
nodes "never appear" — and it dissolves by separating the two. A pin is its own
always-present envelope field carrying the node and *its current lifecycle and
blocked state*, whatever those are. At most one pin exists (an invariant refused
at `apply`, not resolved at projection). A pin on a node that is resolved,
deferred, pruned, or blocked still renders, and renders *with that state* — the
agent asked to be told about this node, so being told it is now blocked is the
answer, not a reason to hide it. The pinned node is excluded from the candidate
set to avoid rendering twice.

**Rank** is a lexicographic sort key over persisted values only:

1. `kinship` — a closed rank over every admitted candidate class, so no class
   falls through to the tie-breaks for want of a tier:

   | rank | class | note |
   |---:|---|---|
   | 0 | cursor's children | ranks 0/1 **swap** under breadth posture |
   | 1 | cursor's siblings | |
   | 2 | direct `needs` neighbours of the cursor | any distance |
   | 3 | cursor's grandchildren | |
   | 4 | nearest unresolved ancestor-sibling | where traversal goes next |
   | 5 | cursor's parent | |
   | 6 | cursor's grandparent and niblings | |

   **Classes overlap, and the lowest matching rank wins.** A cursor's child may
   also be a direct `needs` neighbour; a nibling may be one too. Rev 2 left that
   unresolved, so a node matching both rank 0 and rank 2 had no defined rank —
   which is the same partiality F-3 named originally, surviving in a new place.
   The rule is now stated: classes are tested in ascending rank order and a
   candidate takes the **first** it matches, so every candidate has exactly one
   kinship rank and `kinship` is a total function on the candidate set.

2. `needs` in-degree, descending — a node many others depend on is more
   consequential than a leaf.
3. `seq` ascending — see below.
4. `inq-*` id ascending — the final discriminator; ids are unique, so the order
   is total.

**`seq` is persisted model state, not container iteration.** The first draft said
"stable map insertion order", which RV-320 F-3 correctly read as either
underspecified or an appeal to iteration order — and iteration order would lose
PHASE-02 EX-5's determinism. So each node carries a monotonic `seq` assigned at
creation from the snapshot's own counter. It is pure: derived from snapshot
state, never from a clock or rng.

**Degenerate states**, each with a defined answer rather than an implied one:

| state | frontier |
|---|---|
| empty map | empty; the totals in (e) say so |
| no cursor set | candidates are the root's children under the same rank |
| cursor at a leaf, no siblings | ancestor-siblings and `needs` neighbours only |
| every node resolved | empty, and `truncated` is false — a true statement, distinguishable from a bound list by the totals |
| cursor on a pruned/resolved node | the cursor is stale; candidates come from its nearest open ancestor, and the envelope says the cursor is stale |

**Posture is load-bearing, not decoration.** It swaps kinship ranks 0 and 1 and
nothing else — the concrete content of design.md §5.3's adaptive traversal.
Together with the separate pinned slot this keeps design R4 (authority
laundering) a type-level property: user-pinned direction cannot be confused with
agent-proposed structure, because they are different fields.

## (d) How the material-change delta is computed, and against what baseline

**Baseline** is the caller's declared `--known-revision`. Absent, the immediately
previous revision. The snapshot's revision is monotonic (DEC-059), so the delta
is the half-open range `(known_revision, current_revision]`.

**Source.** Snapshots are atomically replaced, not retained, so there is no
historical snapshot to diff — the delta must be *recorded*, not computed. The
snapshot carries a change log, and RV-320 F-7 established that the first draft's
`(revision, kind, subject_id)` row cannot encode what the delta promises to
render. The row is therefore **self-contained**:

```
revision      u64      the revision that produced this event
index         u32      position within that revision, assigned in
                       validated-candidate order (see below)
event         enum     the closed vocabulary below
subject       id       the primary subject
payload       full     event-specific, stored at full fidelity; the
                       ENVELOPE_* caps bind its RENDERING, not this row
```

**Event vocabulary** — closed, each carrying what rendering it needs without
consulting history:

| event | payload |
|---|---|
| `node_created` | parent id, provenance |
| `node_lifecycle` | from → to |
| `node_reparented` | old parent id → new parent id |
| `needs_added` / `needs_removed` | **both** endpoint ids |
| `stage_moved` | from → to, and on a regression the reason in full (elided only when rendered) |
| `evidence_invalidated` | gate id, the subject fingerprint that died |
| `section_created` | section id |
| `section_fingerprint_changed` | section id, old → new fingerprint |
| `review_attested` / `review_invalidated` | section id, attestation id |
| `checkpoint_disposed` | inquiry id, record ref, and `create` vs `adopt` |

### The stored row and the rendered row are different artefacts

Rev 3 got this wrong in a way worth naming, because the mistake has a shape that
will recur. It applied `ENVELOPE_PAYLOAD_BYTES` to the **stored** change row —
truncating a regression reason to 96 B and a fingerprint to 12 characters *on
disk* — so that the record would fit a budget that only ever applies at render
time. The constant's own prefix was the tell: `ENVELOPE_*` bounds the envelope,
and the envelope is a projection.

**A projection bound must never propagate into the record it projects from.**
Truncating at write time destroys information that no later reader can recover,
to buy space in a rendering that reader may not even request. The snapshot is
gitignored runtime state under `.doctrine/state/slice/<NNN>/`, bounded by
`CHANGE_LOG_REVISIONS = 32` — it is neither committed nor multiplied, so there is
nothing to buy.

### The layer rule

Rev 4 stated the paragraph above and then violated it in the very next table,
bounding the *stored* reason "to `ENVELOPE_REASON_BYTES`". That is the fifth
time this defect class has escaped a revision. Item 4 of § *For the reviewer*
has said since rev 4 that a fifth instance means the missing thing is a
mechanism, not an author's care. So rev 6 stops patching instances and states
the mechanism:

> **A constant's prefix declares the layer it belongs to, and a layer may only
> bind its own artefacts.**
>
> - `ENVELOPE_*` bounds the budgeted rendering — nothing else. It may never
>   appear in a storage or admission context.
> - `DESIGN_*` bounds a value at admission: the moment it is created or
>   accepted. Exceeding it is a **refusal**, never a trim.
> - **Identity and closed vocabularies are bounded at admission, never at
>   emission.** Only gracefully degrading prose may be truncated when rendered.

Both halves are needed, and each catches instances the other misses. The
prefix half catches a projection constant reaching into the store; the
admission half catches an identity being trimmed on the way out. Applied
retrospectively the rule catches every recurrence to date — rev 3's stored
reason and stored fingerprints and its unbounded subject id, rev 4's stored
reason at 240 B, and rev 4's truncating id cap — which is the test of a
mechanism: it must explain the failures that already happened, not only the
one just found.

### The second rule: provenance

Rev 6 stated the layer rule alone and set its own falsifier: *a sixth instance
the rule does not retro-catch means widen the rule, never patch the instance.*
RV-320's fourth round found one immediately — and inside rev 6's own fix.
`CHANGE_REASON_INPUT_BYTES` = 2048 sat on the correct layer, was enforced by
refusal exactly as prescribed, and was **asserted with no derivation
whatsoever**. The layer rule is blind to it by construction: it governs *where*
a bound acts, never *whether the bound is justified*.

So the mechanism is a **pair**, and F-1's defect class has always had two
orthogonal halves:

> **The provenance rule.** Every bound states what derives it. A bound that
> cannot be derived is **removed**, not guessed — an underivable number is a
> decision in disguise, and it will later be used to settle questions it has no
> standing to settle.

Between them the two rules account for every instance in this review's history,
which is the standard rev 6 set for itself:

| instance | caught by |
|---|---|
| rev 3's stored reason and stored fingerprints | layer |
| rev 3's unbounded subject id | layer |
| rev 4's stored reason at `ENVELOPE_REASON_BYTES` | layer (both halves) |
| rev 4's truncating id cap | layer (admission half) |
| rev 3's asserted ~320 B payload and 220 B row | **provenance** |
| F-4's 3.5 bytes/token "worst case" | **provenance** |
| rev 6's asserted `CHANGE_REASON_INPUT_BYTES` | **provenance** |

The ceiling's own history is the argument for taking provenance seriously: an
underivable 16 KiB was twice used to reject design alternatives, which is
precisely "a decision in disguise".

### Enforcement — structural, not textual

Rev 6 proposed a grep: "no `ENVELOPE_*` identifier appears in a storage-write or
admission path". RV-320's fourth round was right to reject it, and right about
*why* — it is the containment check's error one level up. A grep proves
sensitivity to one spelling, not the semantic property: an `ENVELOPE_*` value
reaches admission through a re-export, an alias, a wrapper, or a copied literal
while the grep stays green. Detecting one spelling is not proving a property,
which is the same distinction this review established between containment and
identity.

Two structural replacements, neither of which can be evaded by renaming:

1. **The layer boundary is enforced by Rust privacy — a compile error, not a
   test.** `ENVELOPE_*` constants are **private to the rendering module**. A
   storage or admission path cannot reference what it cannot name, so the
   violation stops compiling. This is stronger than any check that runs after
   the fact, and it is closed under aliasing and indirection for the same
   reason: you cannot alias a name that is not in scope, and a re-export would
   itself be a visible, reviewable line *inside* the rendering module.

   **Rev 7 proposed riding ADR-001's `tests/architecture_layering.rs` for this
   and was wrong** — recorded rather than quietly corrected, because it is the
   seventh instance of asserting a bound without checking it. `layering.toml`
   has exactly one axis, altitude (`leaf | engine | command`), and its
   classification rule is "tier = highest altitude of any non-test file". Pure
   constants import nothing, so they classify **leaf** — the bottom tier, which
   every module is permitted to depend on. The existing checker would not merely
   miss the violation, it would *license* it. The layering map cannot express a
   rendering-vs-storage constraint because that is not the axis it models.

   **What privacy does not catch, stated plainly:** a *copied literal* — someone
   typing `96` directly into storage code. No mechanism in this design catches
   that; it is STD-001's territory (no magic numbers, single-source named
   constants) and it is caught by review, not by construction. Rounds 3 and 4
   were both lost to claiming more enforcement than existed, so the limit is
   named here rather than papered over.
2. **Admission is enforced by construction, not per call site.** Every
   run-local id — node, section, gate, inquiry, attestation — is built through
   **one validating constructor**; there is no other way to make one. The
   fourth round's escape was exact: with ids now rendered whole, a single
   admission path that accepts a 33-byte id breaks the 32-byte row premise and
   the envelope arithmetic with it, while every rev 6 test still passes. A
   shared constructor makes the enforcement universal structurally, so the test
   obligation becomes "no path constructs an id any other way" rather than an
   enumeration of paths that must each remember.

**What rev 6 got wrong here is worth naming**, because it is the same mistake in
a new place: making ids non-truncating moved the load-bearing guarantee from the
renderer (which always held, badly) to admission (which held only where someone
remembered). Moving a guarantee without making its new home total is how the
class keeps surviving.

The corollary for this sketch is that three caps rev 4 introduced as rendering
caps were mislabelled: ids, stage labels, and event names are all identity or
closed vocabulary. They become `DESIGN_ID_BYTES`, `DESIGN_STAGE_LABEL_BYTES`,
and `DESIGN_EVENT_NAME_BYTES`. **Their values do not change**, so no byte
arithmetic in this sketch moves; what changes is that exceeding one is now a
refusal at creation instead of a silent trim at render time.

So the two rows are separated:

| | stored change row | rendered change row |
|---|---|---|
| where | `.doctrine/state/slice/<NNN>/design.toml` | `design show --format prompt` |
| regression reason | **whole, exactly as accepted, with no bound at all** — no rendering constant applies and rev 7 removed the invented domain one. Consistent with section bodies, which the snapshot already stores unbounded | elided to `ENVELOPE_CHANGE_REASON_BYTES` = 96, with an explicit marker |
| fingerprints | full digest | abbreviated to `ENVELOPE_FINGERPRINT_SHORT_BYTES` = 12 |
| ids | whole — bounded at creation by `DESIGN_ID_BYTES`, never trimmed | whole; identity is not truncated |
| bounded by | fidelity, and admission bounds only | `ENVELOPE_PAYLOAD_BYTES` = 160 |

`show --full` reads the stored row, so nothing the log records is unreachable —
only the *budgeted* rendering is narrow. That also disposes of the "recap"
concept rev 3 introduced: there is one reason, stored whole and projected narrow,
not a second lossy copy.

### The rendered payload, derived rather than asserted

RV-320 F-7's second verify round was right that rev 3 asserted a ~320 B payload
with no derivation, so neither its fit nor the 672 B overflow it claimed was
proved — the F-1 defect a third time, this time in the *justification* rather
than the design. The budgeted rendering is `design show --format prompt` and
nothing else (see § The budgeted rendering); `json` framing is explicitly not
budgeted, so the encoding is fixed and the arithmetic is closed.

A rendered payload is space-separated `key=value`. The widest member of the
closed vocabulary is `stage_moved` carrying a regression reason:

| term | bytes |
|---|---:|
| `from=` + stage label | 5 + 16 |
| `to=` + stage label | 3 + 16 |
| `reason=` + elided reason | 7 + 96 |
| two separating spaces | 2 |
| **widest payload** | **145** |

`ENVELOPE_PAYLOAD_BYTES = 160` therefore holds every member with 15 B spare, and
the next widest — `section_fingerprint_changed` at `section=`(8) + 32 +
`old=`(4) + 12 + `new=`(4) + 12 + 2 = 74 B — is not close. The whole row:

| term | bytes | bounded by |
|---|---:|---|
| revision (u64 decimal) | 20 | wire type's max rendered width |
| index (u32 decimal) | 10 | wire type's max rendered width |
| event name | 32 | `DESIGN_EVENT_NAME_BYTES` (admission; actual max 27) |
| subject id | 32 | `DESIGN_ID_BYTES` (admission) |
| payload | 160 | `ENVELOPE_PAYLOAD_BYTES` (emission) |
| row framing and separators | 10 | fixed by the encoding |
| **worst-case rendered row** | **264** | |

Every scalar in that table has a named bound, which is the property F-1 asked
for and rev 3 lost again by leaving the subject id and framing unbounded.

**Rev 6 changes no number here.** The stage-label term is still 16 and the id
terms still 32; what changed is that those are now *admission* bounds, so a
value arriving at the renderer is already within them by construction. The
derivation is consequently stronger than rev 4's: rev 4 computed a worst case
that held *because the renderer would truncate to it*, which is exactly the
reasoning that made two distinct ids render identically. Rev 6's worst case
holds because no value that large can exist. Same arithmetic, sound premise.

**Not material** — cursor moves, posture changes, receipt eviction, prompt
fragment receipts. Reporting them as change rows is precisely the
ceremony-exceeds-value failure R1 names. Current cursor and posture are still
projected as *state*; they are simply not *delta*.

**Within-revision determinism.** One `apply` is an unordered batch (DEC-063), so
a revision can emit many rows and a 10-row cut through one revision would
otherwise be arbitrary. Rows are indexed in the order the *validated candidate*
serialises — a deterministic function of the declaration set, not of submission
order — and the cut is by `(revision, index)` descending. A cut that would split
one revision's rows is permitted and reported: the omitted count plus the
retained rows' shared revision make the partial cut visible rather than
disguised as a complete one.

**Retention floor.** The snapshot records `change_log_floor` — the oldest
revision the log still covers — explicitly, rather than inferring it from the
oldest surviving row. RV-320 F-7 is right that inference breaks when intervening
revisions produced no material rows. With the floor recorded:

- `known_revision >= floor` → the delta is complete for the range;
- `known_revision < floor` → the delta is **unavailable**, and the envelope says
  so in those words, with the floor and a pointer to `show --full`.

An unavailable delta and an empty delta must never render identically. That is
design R2 in its sharpest local form: "nothing changed" and "I cannot tell you
what changed" are opposite facts.

**Plan consequence — now made, not merely named.** This is a new persisted
snapshot group that design.md §5.3 does not enumerate and that PHASE-03's
original criteria did not verify. Rev 2 named the required amendment and left it
as a follow-up; RV-320 F-7's verify round was right that naming it discharges
nothing, because a criterion that does not exist obliges no phase to build the
thing this sketch depends on. The amendment is therefore **applied**: PHASE-03
now carries an appended `EX-13` (persistence, explicit `change_log_floor`,
validated-candidate index order, retention bound, unavailable-vs-empty, and the
payload's own scalar caps), `VT-5` (three named tests, none of which exists at
head), and `VA-6` (their negative control, with the index test required to red
by varying submission order rather than by asserting the order it submitted).
Criteria are immutable, so these are appended and nothing was renumbered.

Settled in favour of carrying the log rather than narrowing `--known-revision`,
because change-only projection is part of R1's answer and the alternative trades
a locked §5.2 interface for implementation convenience.

## (e) Worst-case envelope size

Every row is `cardinality cap × per-entry encoded worst case`, so the total is
independent of the run's node count. Per-entry figures assume every capped
scalar saturated in the budgeted rendering.

| field | worst case |
|---|---:|
| run header (identity, revision, schema, stage) | 256 B |
| global totals (see below) | 288 B |
| next obligation | 304 B |
| pinned slot | 240 B |
| active path (6 × 224 B) | 1344 B |
| frontier (7 × 280 B) | 1960 B |
| blockers (5 × 256 B) | 1280 B |
| change delta (10 × 264 B) | 2640 B |
| durable records (8 × 156 B) | 1248 B |
| section rows (16 × 200 B) | 3200 B |
| derived slice facts | 512 B |
| fragment metadata (6 × 96 B) | 576 B |
| mutation contract + worked example | 1344 B |
| omitted counts and truncation notices | 384 B |
| **saturated total** | **15,576 B = 15.21 KiB** |

`ENVELOPE_NORMAL_BUDGET_BYTES` is 24 KiB — 9000 B, or 57.8%, above the saturated
table. **The headroom is not what makes the ceiling true**; the (b) eviction
ladder is. If a future field or a retune pushes the table past the ceiling, the
envelope degrades by evicting and counting rather than by breaking its bound.

Rev 4 raised the saturated total from 15,136 B by deriving the change row
properly (220 B asserted → 264 B derived), which is the honest direction for a
number to move when it stops being a guess — but it left only 808 B of slack
under a 16 KiB ceiling. Rev 5 raises the ceiling to 24 KiB, deliberately and as
a decision rather than a retune, because twice running the old ceiling had
pressed on design questions it had no business deciding (see § Revision
history). The generous headroom is the point: a ceiling this far above the
saturated table cannot be the reason a design alternative is rejected, which is
the failure mode 16 KiB actually produced. The ladder still enforces the bound;
it is simply no longer expected to fire on a well-formed run.

### Tokens, stated honestly

RV-320 F-4 is right that the first draft divided bytes by an average
bytes-per-token ratio and called the result a worst case. It is not one. The
correction has three parts:

- **Defensible upper bound:** for any byte-level BPE tokenizer, a token covers at
  least one byte, so the budgeted rendering is **≤ 24,576 tokens**. That holds
  for adversarial input by construction and needs no measurement. It moves with
  `ENVELOPE_NORMAL_BUDGET_BYTES` because it is derived from it, and rev 5 raised
  that ceiling.
- **Expected case:** at ~3.5 bytes/token for mixed prose and JSON the saturated
  table (15,576 B) is **~4.5k tokens**, and a typical non-saturated envelope is
  well under half of that. This is an estimate and is labelled as one. Note it
  tracks the saturated *table*, not the ceiling, so rev 5's raise does not move
  it — the expected cost of a real envelope is unchanged.
- **What the gate actually verifies:** the PHASE-04 test projects the large-run
  fixture — including a hostile case with maximal multi-byte and escape-heavy
  content — measures with the target tokenizer, and asserts the byte ceiling.
  Measurement, not this table, is the evidence.

**The claim R1 rests on is the constancy, not the constant.** An unbounded
projection of a 500-node map runs past 100 KiB *and grows every turn*, in a
protocol that re-projects on every turn. A bound that is flat in run size is the
property that makes the protocol affordable; the absolute number is a tuning
question the fixture answers.

## (f) What `show --full` may scale with that normal `show` may not

**May scale with the run:** the complete inquiry map, all nodes and edges; all
blockers; all linked durable-record references; the complete section list with
fingerprints; all review attestations and findings; all submission and fragment
receipts; the change log across its whole retained window; and the full
declaration schema rather than one worked example.

**May not scale, even in `--full`:** authored prose is never inlined — `--full`
cites section ids, order, titles, and fingerprints, never `design.md` bodies;
the caller has the file. And the change log stays window-bounded, because
`CHANGE_LOG_REVISIONS` bounds storage rather than projection: `--full` cannot
show what was evicted, and says so via the same floor.

### The relation is subsequence, not prefix

RV-320 F-6 caught a genuine incoherence: the first draft claimed normal is a
*prefix* of full, while (b) retains the cursor end of the active path and the
newest end of the delta — both suffixes under those lists' natural orderings.
Prefix equality and decision-local retention cannot both hold.

Decision-local retention is the one worth keeping, so the invariant weakens
precisely:

> For every bounded list, the normal envelope's entries are an **order-preserving
> subsequence** of `--full`'s entries for that list, with cardinality less than
> or equal, and the retained end is the one named in (b)'s table.

Naming the retained end per list is what makes this testable rather than
vacuous. §9.3's help-text convention tests protect it, and nothing
design-specific may invert the established metadata-only meaning of `inspect`.

## (g) What the agent loses when a limit binds, and why that is acceptable

Field by field, because "bounded is fine" is not an argument:

- **Active path.** Loses the *root* end — the outermost framing of the
  decomposition. Acceptable: stage, next obligation, and the global totals carry
  what that framing conveyed, and the cursor end is where the decision is. This
  loss was unstated in the first draft (RV-320 F-2).
- **Frontier.** Loses the lowest-ranked candidates. Acceptable: the agent acts on
  one node per turn and the next turn re-projects from the new cursor, so a
  dropped candidate is at most one turn from visible and `show --full` reaches it
  now. Nothing becomes unreachable; it becomes one command further away.
- **Blockers.** The least acceptable loss, so it gets the strictest handling: the
  global blocker total is in the no-drop set, and a bound list always sets
  `truncated`. Past five blockers the agent's correct next act is to look at all
  of them, and the envelope says so rather than implying five is all there is.
- **Change delta.** Loses the oldest material changes since the baseline.
  Acceptable: the newest are what invalidated current clearance, and clearance is
  re-derived from evidence rather than from this list. The delta orients; it is
  not a correctness input.
- **Durable records.** Loses least-recently-linked context. Acceptable and
  already scoped: OQ-1 records that the envelope surfaces known linked context
  and does not claim comprehensive discovery.
- **Sections.** Loses rows past 16, outstanding-review rows last. If it binds in
  practice the number is wrong — retune it; that is what (a)'s names-versus-values
  split is for.
- **Declaration example.** Does not truncate. Half a contract is worse than none,
  so it is in the no-drop set and an oversized example is an authoring defect in
  the fragment, refused rather than clipped.

### The R2 claim, repaired

The first draft claimed exact omitted counts make "this run is small" and "this
run was truncated to look small" distinguishable. RV-320 F-5 falsified it, and
the falsification is worth stating plainly because it is the kind of error the
whole gate exists to catch: **the frontier's candidate set is narrowed by
relevance *before* any cap applies.** A 500-node map with a leaf cursor yields at
most a handful of candidates, so `frontier_omitted` is 0 and `truncated` is
false — the envelope reports "nothing omitted" on a huge run. An omitted count
measures what a *cap* discarded, and says nothing about what *selection*
discarded.

The repair is to carry global totals, in the no-drop set, so run complexity is
visible independently of what the neighbourhood happens to contain:

- total nodes, and counts by lifecycle: open, resolved, deferred, pruned;
- derived-blocked count;
- **open nodes outside the frontier candidate set** — the number the first
  draft's claim needed and never had;
- total sections, and sections with outstanding review state;
- total material changes since the baseline, distinct from the rows rendered.

They are integers. The whole block is budgeted at 288 B and it is what makes the
distinction real rather than asserted. The corresponding test is two envelopes
with identical visible rows and radically different distant maps, which must not
render as equally small.

**The honest residual.** Even with the totals, a bounded projection asks the
agent to trust a summary. The mitigation is that the summary's *shape* is
falsifiable — exact counts, named retained ends, an explicit unavailable state —
so a suspicious reader can always reach the whole picture in one command. What
remains is that they have to choose to.

---

## Revision history

| finding | disposition |
|---|---|
| F-1 encoded-size bounds | Encoded-byte caps replace the character cap; the budgeted rendering, escaping, and numeric width semantics are defined; a measure-and-evict ceiling makes the bound enforced rather than predicted. |
| F-2 drop policy for every field | Every bounded list has a retained/drop end in (b)'s table; the global eviction ladder and the no-drop-set refusal are specified; active-path loss is stated in (g). |
| F-3 total pure frontier order | Closed `kinship` rank over every admitted class; `seq` is persisted model state, not iteration order; the pin becomes a separate always-present slot with a cardinality invariant, dissolving the pinned/blocked contradiction; degenerate states tabulated. |
| F-4 worst-case tokens | Split into a defensible ≤16,384-token upper bound, a labelled expected-case estimate, and a measured fixture gate. Byte arithmetic corrected (the first draft misstated 10,374 B as ~10.4 KiB and its headroom as ~15%). *(Numeral superseded by rev 5: the bound is `ceiling × 1 byte/token`, so it became ≤24,576 when the ceiling rose. The construction is unchanged — this row records rev 2's figure.)* |
| F-5 R2 distinguishability | Global totals added to the no-drop set, including open nodes outside the candidate set; the false claim is stated as false and repaired rather than quietly edited. |
| F-6 prefix property | Weakened to an order-preserving subsequence with a named retained end per list, keeping decision-local retention. |
| F-7 material delta contract | Self-contained event vocabulary with payloads; within-revision index determinism from validated-candidate order; explicit `change_log_floor`; the required PHASE-03 criterion amendment is named. `--known-revision` is retained deliberately. |

### Rev 3 — the verify round's three contests

| finding | disposition |
|---|---|
| F-2 contested: global overflow not total | Conceded. Excluding the active path from the ladder left a state with no defined outcome (ladder exhausted, no-drop + active path still over). The active path becomes ladder rung **6** — last, which honours the "costs most to lose" judgement without the partiality. The no-drop-set refusal is now the single terminal rule by construction. |
| F-3 contested: overlap and eligibility | Conceded on both counts. The distance-2 rule contradicted the rank table in both directions (ancestor-siblings and niblings are both distance 3), so it is **deleted** and the rank table becomes the sole eligibility enumeration. Overlapping classes resolve by **lowest matching rank**, making `kinship` total on the candidate set. |
| F-7 contested: delta contract unsettled | Conceded on both counts. The 240 B reason inside a 128 B payload is split into `ENVELOPE_CHANGE_REASON_BYTES = 96` rather than by raising the payload cap, which would overflow the ceiling by 672 B. PHASE-03's missing criteria are **appended, not merely named**. Also fixed unprompted: `section_fingerprint_changed`'s two full digests overflowed the same payload; change rows now carry 12-character short forms. |

### Rev 4 — the second verify round

F-2 and F-3 verified. F-7 contested a second time, and the contest was right on
every count.

| point | disposition |
|---|---|
| The ~320 B payload was asserted, so neither its fit nor the claimed 672 B overflow was proved | Conceded — F-1's defect a third time, in the justification rather than the design. The 672 B claim is **withdrawn**, not re-derived: the storage/projection split makes it moot. |
| Payload fit unproved: section id and framing had no encoded-size bound | Conceded. `ENVELOPE_ID_BYTES`, `ENVELOPE_STAGE_LABEL_BYTES`, `ENVELOPE_EVENT_NAME_BYTES`, `ENVELOPE_FINGERPRINT_SHORT_BYTES` added; every term in a rendered row now has a named cap and the widest payload (145 B) and row (264 B) are derived term by term. `ENVELOPE_PAYLOAD_BYTES` 128 → 160. Saturated table 15,136 → 15,576 B. *(Rev 6: the first three were mislabelled as rendering caps and are renamed `DESIGN_ID_BYTES`, `DESIGN_STAGE_LABEL_BYTES`, `DESIGN_EVENT_NAME_BYTES` — same values, admission bounds. This row records rev 4's names.)* |
| VT-5/VA-6 cover only floor and order; nothing can fail for missing event rows, retention eviction, self-contained payloads, scalar caps, or the container invariant | Conceded — EX-13 named properties no test could fail. PHASE-03 gains appended `EX-14`, `VT-6`, `VA-7`. |
| — | Independently: rev 3 applied a *projection* bound to the *stored* record, truncating reasons and fingerprints on disk. Rev 4 separates the two; the store keeps full fidelity. |

### Rev 5 — the ceiling, decided

No finding prompted this. Rev 4's § *For the reviewer* item 3 put
`ENVELOPE_NORMAL_BUDGET_BYTES` to the reviewer as this sketch's open question;
it is answered here so that round 3 is not asked to adjudicate a decision that
was never the reviewer's to make.

| point | disposition |
|---|---|
| 16 KiB had twice decided what it should not | Rev 3 rejected a design alternative on budget grounds (reasoning since withdrawn); rev 4's honest derivation then ate 440 B of the remaining slack unprompted. A ceiling that changes design outcomes is miscalibrated, not tight. **Raised to 24576 (24 KiB).** |
| What moved, exactly | Three derived figures: headroom 808 B (5.2%) → **9000 B (57.8%)**; token upper bound ≤16,384 → **≤24,576** (it is `ceiling × 1 byte/token`, so it tracks the ceiling by construction). Nothing else — the saturated table stays **15,576 B**, every per-field cap is unchanged, and the eviction ladder's rungs and order are untouched. |
| What did NOT move, and why it matters | The expected-case estimate tracks the saturated table, not the ceiling. Rev 4 left it stale at ~4.3k tokens (15,136/3.5); against 15,576 B it is **~4.5k**. Corrected here. The raise does not change what a real envelope costs — only what the bound permits. |
| Plan impact | None. No criterion pins the number: PHASE-04 EX-10/VT-2 name `normal_envelope_stays_within_named_limits_on_a_large_run`, which reads the named constant, and VA-2 compares a measured figure against answer (e). Checked before editing, so no criterion is silently invalidated. |
| The cost, stated | Raising the ceiling pushes the eviction ladder further from ever firing on realistic input, which makes item 1's under-exercised-mechanism risk worse. Carried explicitly into § *For the reviewer* item 3 rather than absorbed. |

### Rev 6 — the third verify round, and the mechanism

F-7 contested a third time. All three points were verified against the text
before being conceded; all three hold, and the first is worse than stated.

| point | disposition |
|---|---|
| The storage/projection split contradicts itself: EX-14 forbids `ENVELOPE_*` in the persisted row, then bounds the stored reason "to `ENVELOPE_REASON_BYTES`" | **Conceded, and it is the fifth instance of F-1's class.** The stored/rendered table also contradicted the row schema eight lines above it, which had the rule right. Rev 6 stores the reason **exactly as accepted**, with no rendering constant anywhere near it, and adds `CHANGE_REASON_INPUT_BYTES` = 2048 as a *domain* bound enforced by refusal at `apply`. No domain maximum existed before — design §5.4 says only "a direct regression records a reason" — so the contest was right that implementations had to truncate, reject, or invent one. |
| `DESIGN_ID_BYTES` proves containment, not delta identity: ids are not in the no-drop set, so two legal ids sharing a 32-byte prefix render identically | **Conceded — this is F-7's own subject failing.** Ids are now bounded at *admission* and never truncated, so the ambiguity cannot arise. Rev 4's derivation held only "because the renderer would truncate to the cap", which is precisely what destroyed identity. The identifier grammars the contest asked for are bound at their source: PHASE-05 EX-10 obliges the marker grammar to honour `DESIGN_ID_BYTES`, which nothing previously did. |
| EX-13 and EX-14 are simultaneously live and contradictory | **Conceded.** EX-13 now carries the SUPERSEDED form already used by PHASE-05 EX-7 — id retained, text rewritten to record the supersession — leaving one authoritative storage contract. |
| — | The mechanism, not asked for: § *The layer rule*. Prefix declares layer; a layer binds only its own artefacts; identity and closed vocabularies bound at admission, never emission. It retro-catches all five recurrences and is grep-able, which the prose obligations were not. |
| — | **No byte arithmetic moved.** Payload 145, row 264, saturated table 15,576 B, headroom 9000 B all stand; the three renamed constants keep their values. Verified by recomputation, not assertion. |

### Rev 7 — the falsifier fired, so the rule widened

F-7 contested a fourth time. All three points verified before conceding; all
three hold. Rev 6 set an explicit falsifier — a sixth instance the layer rule
does not catch means widen the rule, never patch the instance — and the fourth
round found one **inside rev 6's own fix**. Rev 7 honours the falsifier.

| point | disposition |
|---|---|
| Nothing tests that **every** run-local-id admission path refuses an over-bound value. VA-8 leg 1 uses two *legal* ids near the boundary; PHASE-05 EX-10 covers section ids only. A single path accepting a 33-byte id breaks the 32-byte row premise and the envelope arithmetic while every test passes | **Conceded, and rev 6 created this.** Making ids non-truncating moved the load-bearing guarantee from the renderer to admission without making admission total. Fixed structurally: **one validating constructor** for every run-local id, so universal enforcement is a property of construction rather than of each call site remembering. The obligation becomes "no path constructs an id any other way". |
| VA-8 leg 3's "reachability" grep defines no source roots, transitive boundary, or alias handling; demonstrating failure on one inserted reference proves sensitivity to that example, not completeness. A copied literal or renamed constant evades it | **Conceded — it is the containment check's error one level up**, and this review already established the distinction. Replaced with a **module dependency rule** on ADR-001's existing `tests/architecture_layering.rs`: `ENVELOPE_*` lives in a rendering module that storage and admission may not depend on. A dependency-graph property is closed under aliasing, re-export and wrappers; a text search never is. *(**Rev 8: this replacement was itself wrong** — `layering.toml` models only altitude, and pure constants classify `leaf`, which everything may depend on. Superseded by Rust privacy; see § Rev 8.)* |
| `CHANGE_REASON_INPUT_BYTES` = 2048 sits on the correct layer and is enforced correctly, but nothing derives 2048. The layer rule cannot establish why byte 2049 is refused | **Conceded — the sixth instance, in rev 6's own repair, after the round-4 brief named this as the defect class's favourite hiding place.** The layer rule is blind to it by construction. Answer: **the provenance rule** (§ *The second rule*), and applying it to 2048 deletes the constant. The reason is now stored with no bound, which needs no number to justify — the snapshot already stores unbounded prose, so bounding one field was special-casing. |
| — | The mechanism is now a **pair**: layer (where a bound acts) + provenance (whether it is derived). Together they retro-catch every instance in this review's history, including the three asserted-number defects the layer rule alone could not see. |
| — | **No byte arithmetic moved**, again. 145 / 264 / 15,576 / 9000 all stand; removing `CHANGE_REASON_INPUT_BYTES` touches no rendered term. |

### Rev 8 — the enforcement claim was itself unchecked

No review round prompted this. Rev 7's enforcement leg asserted that ADR-001's
existing layering test could express "storage may not depend on `ENVELOPE_*`".
The project owner pointed at `.doctrine/adr/001/layering.toml`; one read
falsified it.

| point | disposition |
|---|---|
| `layering.toml` cannot express the boundary | It models ONE axis — altitude, `leaf \| engine \| command` — under the rule "tier = highest altitude of any non-test file". Pure constants import nothing, so they classify **leaf**, the tier every module may depend on. The checker would have **licensed** the violation, not caught it. `design_run` is not in the map at all; PHASE-02 creates it. |
| The replacement | **Rust privacy.** `ENVELOPE_*` private to the rendering module makes a storage reference a *compile error* — stronger than a post-hoc check, closed under aliasing and re-export, and requiring no new machinery or ADR-001 surgery. |
| The limit, named | Privacy does not catch a **copied literal**. Nothing here does; that is STD-001's no-magic-numbers rule, enforced by review rather than construction. Named because claiming more enforcement than exists is what lost rounds 3 and 4. |
| The honest classification | This is the **seventh** instance of the class — asserting a bound (here, an enforcement guarantee) without deriving or checking it. The provenance rule caught it, which is the first time a rule in this sketch caught an instance *before* a reviewer did. That is the only evidence so far that the pair is doing real work. |

## For the reviewer

Where I am least confident, in descending order:

1. **The eviction ladder's field order** (b) is asserted from a judgement about
   what an agent can most afford to lose, with no fixture evidence. It is also
   the one mechanism that fires only under conditions the happy-path fixture will
   not produce, so it risks being the least-exercised part of the design. Rev 3
   adds a sixth rung, which extends the untested surface rather than shrinking it.
2. **The within-revision index** depends on "the order the validated candidate
   serialises" being deterministic. That is a property of PHASE-02's
   serialization contract, which does not exist yet — so this sketch is asserting
   a constraint on a sibling deliverable rather than observing one.
3. **The ceiling is settled at 24 KiB — but settling it sharpens item 1.**
   This was rev 4's open question and it is now closed by decision, not by
   retune: 16 KiB had twice pressed on design questions it should not have been
   deciding (rev 3 rejected a fix on budget grounds; rev 4 withdrew that
   reasoning and the slack fell to 808 B anyway). 24 KiB removes budget from the
   argument. What it does *not* remove is the residual, and raising the ceiling
   makes it worse rather than better: at 9000 B of headroom the eviction ladder
   is now further than ever from firing on any well-formed run, so the mechanism
   in item 1 is more under-exercised, not less. Its fixture evidence must
   therefore be **synthetic by construction** — a fixture built to overflow, not
   a large realistic run that happens to. If PHASE-04 only ever exercises the
   ladder via realistic input, the raise will have bought safety on the ceiling
   by trading it against coverage of the thing that enforces the ceiling.
4. **F-1's defect class has recurred six times; rev 7 answers it with a *pair*
   of rules after the fifth answer proved too narrow.** The layer rule (rev 6)
   governs where a bound acts; the provenance rule (rev 7) governs whether it is
   derived. Rev 6's falsifier fired on its own fix — an underivable 2048 — which
   is the strongest available evidence that one rule was not enough. **The
   falsifier now stands for the pair:** a seventh instance neither rule catches
   means widen again. Note the honest reading of the trend — each round has
   found the defect in the *repair* rather than the original, so the thing under
   test is no longer the design but the author's method. The instances:
   `stage_moved`'s reason,
   `section_fingerprint_changed`'s digests, rev 3's unbounded subject id and
   framing, rev 4's stored reason bounded "to `ENVELOPE_REASON_BYTES`", and rev
   4's id cap that truncated identity. Rev 4 said here that a fourth instance
   would mean the check had failed rather than the author. A fifth arrived, so
   the containment check was indeed not the answer: EX-14(c) saturates every
   scalar *at its cap*, which proves a container holds its contents and can
   never detect that a cap is the wrong bound or sits at the wrong layer.
   § *The layer rule* is the replacement, and EX-15 makes it a grep-able test
   obligation. **What would falsify it:** a sixth instance that the layer rule
   does not retro-catch. If one appears, the rule is too narrow — do not patch
   the instance.
5. **The stale-cursor rule** in (c) — projecting from the nearest open ancestor —
   is invented here and appears nowhere in design.md. It may belong in the
   lifecycle model instead.
