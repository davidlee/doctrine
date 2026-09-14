<!-- doctrine:section sec-1 -->
## The invariant

`doctrine design apply` has one job beyond changing state: **telling the truth
about whether it changed state.** Every retry, every recovery, every agent
decision to re-send or move on rests on that signal, and today it is wrong in
three directions at once — success reported for input that was discarded,
success reported with no row for changes that landed, and one retired token
making a whole historical snapshot unreadable.

This design makes the signal load-bearing. One invariant, four legs:

1. **Error implies nothing landed.** On a refused or failed submission: no
   revision, no receipt, no change row. The authored tier is unmoved too,
   except across one late-check window `SPEC-029` specifies and whose recovery
   is the journal (`DEC-250`).
2. **Success implies rows that tell the truth.** Every material change emits
   its row, at the seam that performs it (`DEC-248`).
3. **Unknown or inert input implies a typed refusal**, naming what was expected
   (`DEC-244`, `DEC-245`, `DEC-246`, `DEC-247`).
4. **Old snapshots stay readable.** Degrade what is history; refuse what is
   state (`DEC-249`, `DEC-251`).

`SL-256` delivered the first piece of leg 2 — a recorded act emits a change row.
This slice finishes the contract. The design is deliberately *not* a list of
eight bug fixes: seven of the eight originating items are instances of three
classes, and the sections below are organised by class, because fixing them
instance-by-instance is how `ISS-315` recurred three times and how `ISS-450`
was still there to be found by dogfooding this very run.

<!-- doctrine:section sec-2 -->
## Where strictness lives

`QUE-219` asked whether an unknown key should be refused, and how that squares
with stored declarations outliving the binary that wrote them. It framed the
answer as a write/read split: two types, or two deserialisation modes, so that
tightening a submission never tightens a stored declaration.

**That split is not available.** `Declaration` is both the wire type and a
stored type — `Proposal` holds a delegate's declarations verbatim in the
snapshot (`delegation.rs:69`) — and it already carries `deny_unknown_fields`.
There is no type seam to cut along, and inventing one means a second
declaration type, which is the parallel implementation this project refuses
(`EVD-027`).

So `DEC-243` moves the axis. Strictness is not about *which path*; it is about
**what was ever known**:

- **Never known — refused.** A key or token the vocabulary has never held is a
  caller asserting something into the void. Refuse it, and name it.
- **Known, then retired — read.** A retired member stays in a roster declared
  beside the live vocabulary, so it keeps parsing and cannot be newly written.
  This is not a new idea; it is `SL-256`'s `READABLE` / `EMITTABLE` split
  generalised from change-event tokens to wire keys.
- **Beneath both — a floor.** A genuinely unrecognised *stored* token degrades
  its own row visibly (`STD-003`) instead of failing the file.

The floor is the part that matters most, and it is there because the roster
alone is **discipline, not construction**. Someone has to remember to add the
retired entry. `ISS-315` is the record of what happens when they do not — twice
rescued by the roster, and a third token, `integrated_review_recorded`, that
nobody added. A design that relies only on the roster is a design that has
already failed once in this codebase.

### What this does not decide

Splitting `Declaration` into separate wire and stored types remains the clean
layering repair, and it is **deferred as named debt**, not rejected on merit.
The justification is exposure: all 16 live snapshots read `delegation = []`, so
the overlap is latent (`EVD-027`). That is a measurement, and it will stop
being true the first time someone uses delegation in anger.

<!-- doctrine:section sec-3 -->
## The write path: refusal driven by the contract

Leg 3 refuses **input the engine will not act on** — a property of the payload
against the schema and the subject's state. It is deliberately *not* "a
submission that changed nothing" (`DEC-245`): an idempotent re-declare, a
traversal-only submission and a no-op `adopt_authored` all legitimately change
nothing, and a no-change rule could only be evaluated *after* doing the work,
which is the wrong side of the refusal boundary.

### Why not `deny_unknown_fields`

The obvious fix — decorate the nine wire types that lack the attribute — cannot
reach the places that matter. `ApplyRequest` carries `#[serde(flatten)]`, which
serde cannot reconcile with `deny_unknown_fields`; `Provenance`, `Dispose` and
`DelegationAct` are internally tagged and `WireFacetValue` is untagged, and all
of them buffer their content. So the attribute misses the outermost type — the
only one a caller hand-authors from scratch — which is `ISS-333`, the item that
motivated the leg.

### The mechanism

`DEC-244`: refusal is driven off `payload_contract`'s key inventory, walked
against the submitted JSON before deserialisation.

This is single-sourcing rather than a second schema. `payload_contract` already
describes every key at every nesting level, and `sec-8` pin 1 holds each struct
contract's key set against a **fully populated value's serde output** — so the
contract is already *proved equal* to the wire shape. One walk covers every
nesting level and all three places the attribute cannot go, closing `ISS-333`,
`ISS-328` and `ISS-327`'s key axis together.

It also makes the contract's own output honest. `payload_contract` today prints
`unknown-keys: silently-dropped` on nine types — a disclosure of a defect. After
this, the rendering states a rule.

**Layering (`ADR-001`).** `payload_contract` is a leaf. The walk sits at or
below the command boundary that parses; it must not pull command-tier concerns
down into the leaf. Tests for it live where `design_run` tests live, which may
not name `crate::` (`mem.pattern.design-run.leaf-rule-binds-tests`).

### The state axis

`ISS-327` is the same defect rotated: a key honoured in one subject *state* and
dropped in the other. Four cells — `provenance` (honoured on create), `lifecycle`
(honoured on update), `concerns` and `blocking` (honoured when a finding is
raised). `DEC-246` refuses all four.

The alternative was to define the drop as omission-persist. It does not survive
`blocking`: that is the flag the lock gate reads, so a caller correcting a
finding's `blocking` is told it succeeded and changes nothing — and persist
semantics would make that silence *correct by definition*. `Sparse::Omitted`
already means persist and is reachable by simply not sending the key, so a
**present** value that is discarded has no honest reading as persistence.

This makes admission ask whether the subject is already held. It runs against
the next snapshot, so the fact is available; it is a different check shape from
`SL-249`'s kind-axis table, but it is the same rule on a second axis rather than
a rule plus an exception.

### Payload terms

`ISS-290`: a `PayloadTerm`'s declared `ValueKind` is never compared with the
kind actually constructed, and the kinds carry different admission bounds
(`Token` 32 bytes, `Label` 16). `DEC-247` checks the constructed kind against
`payload_terms()`' declaration at construction, and rules the one live instance:
`Outcome` is a closed two-member vocabulary, so `Label` is the precise kind and
the **declaration** is the error — matching `CheckpointDisposed`, which already
declares its `Disposition` as `Label`.

Fixing the instance without guarding the class is exactly how this instance
survived `SL-233` `PHASE-08`.

<!-- doctrine:section sec-4 -->
## Truthful rows: emit at the seam

Two of leg 2's items are one defect wearing two faces.

`ISS-367`: `live_acts` keys on `(ActKind, DesignId)`, and `act_id` is
`prefix + act.as_str()` — so the tuple is the kind twice. `invalidation_rows`
differences that set, and a same-kind replacement is *identical* on both sides:
empty difference, no `ActInvalidated` row. `live_acts`' own doc promises
otherwise.

`ISS-450`, found by dogfooding this run: a `needs` edge declared at node
creation lands in the graph and emits no `needs_added` row. `run.rs:1294`
returns a lone `NodeCreated` before the diff at `run.rs:1330` that only the
update path reaches. Ten edges landed silently across this run's revisions 2–3;
a control on an existing node at revision 4 emitted its row.

### The rule

`DEC-248`: a material change is emitted by the code that **performs** it, at the
unified admit-store-emit seam `DEC-238` already established. Neither item is
repaired by widening a derivation.

- The `ActInvalidated` row is emitted where `CheckpointActGroup::record`
  retains-then-pushes by kind — the site that *knows* a replacement happened.
- Node creation reaches the same needs diff the update path runs, by diffing
  against an **empty prior** rather than returning early.

The rejected alternative was to add a discriminator to `live_acts`' tuple — an
ordinal, or the acceptance digest — so the set difference could see a
replacement. It repairs one derivation and leaves the class, and it contradicts
`DEC-238`, which is the accepted ruling that a recording is an occurrence and is
emitted rather than inferred.

The deeper reason: **a derived difference reports a change only while the
derivation happens to be injective over what changed.** Nobody states that
property and no test guards it. Both items are that property quietly failing.
Emission at the seam makes the row a consequence of the write rather than of a
later comparison, and gives the coverage check something enumerable.

Diffing creation against an empty prior buys a second thing: it removes the
create/update branch split, which is the shape that produced `ISS-450` *and*
`ISS-327` alike. That is why this is a structural fix and not a hunt — and the
hunt does not terminate, which is what finding `ISS-450` during this design
demonstrates.

<!-- doctrine:section sec-5 -->
## The read path: degrade history, refuse state

`STD-003` requires a degraded read be disclosed. `change_log.rs` argues the
opposite on the record: `ChangeEvent` deserialises strictly because *a change
log that quietly accepted an event it could not name would be a change log that
lies.* Both are right, about different things.

`DEC-249` draws the line by **what the token is for**, not by which type it is:

- **A token that records what happened** — a change-log row and its terms —
  degrades visibly. The row is retained with its raw token and disclosed as
  unreadable.
- **A token that constitutes current state** — `Stage`, `IdKind`, `ActKind` held
  live — still refuses. A snapshot whose stage cannot be read is not a degraded
  row; it is an unusable run, and tolerating it converts a loud failure into a
  silent wrong-state one.

This reconciles the two positions rather than overruling either. A retained,
disclosed, unreadable row does not lie — it states precisely what it knows and
what it does not.

### Preserve, and where the opacity lives

`DEC-251`. Two facts decided it.

**The change log is a bounded window, not permanent history.**
`CHANGE_LOG_REVISIONS` is 32 and `retain_window` evicts below an explicitly
recorded floor. `SL-244` sits at revision 92 with floor 61, and its one
offending row is at revision 88 — four revisions from ageing out on its own. So
`ISS-315` self-heals on an active run and bites a dormant one, an opaque row's
cost is bounded, and *dropping* would lose on the very next write what the
window would otherwise have kept.

**`ChangeLog` is tightly encapsulated.** Its whole surface is `record`,
`retain_window`, `covers`, `since`; the only production reader of `.rows`
outside the type is `render/envelope.rs:1071`.

So the opacity sits at the **row**:

```rust
enum StoredRow { Read(ChangeRow), Unreadable(RawRow) }
struct RawRow { revision: u64, index: u32, raw: toml::Value }
```

`ChangeEvent` is not touched. It stays closed, `Copy`, and const-provable, and
every existing pin holds unchanged — the behaviour-preservation gate satisfied
by construction rather than by re-proving.

### Why not an `Opaque(String)` variant

It breaks `Copy`; it breaks `const fn as_str(self) -> &'static str`; and through
that it breaks **both** compile-time proofs — the `DESIGN_EVENT_NAME_BYTES`
bound and `is_subset(&EMITTABLE, &READABLE)`. The second is a trap: its own
comment records that it is the only reader of `EMITTABLE` in `src/`, so removing
it passes `cargo check` and fails `cargo test --bin doctrine`
(`mem.pattern.rust.expect-dead-code-is-per-compilation-unit`). It would also
move the name bound from compile time to runtime, and still cover only the event
token.

The row-level fallback covers strictly more: an unknown `PayloadKey`, an unknown
`ValueKind`, or an over-bound term value all land in the same arm. That is
`DEC-249`'s rule without the per-vocabulary treadmill.

**Strictness is kept where it belongs.** `revision` and `index` stay *required*
on `RawRow`, so a structurally broken row still refuses. The tolerance is for
vocabulary we do not recognise, never for rows we cannot place.

`DEC-239`'s two refused repairs stay refused: a bare serde alias renames the
variant but not the row, and whole-row normalisation at deserialise manufactures
a subject the writer never stored. This design takes neither.

<!-- doctrine:section sec-6 -->
## Atomicity: what "nothing landed" promises

`ISS-361` reported the worst case — a JSON parse error that nonetheless advanced
the revision and receipt-locked the submission. **It does not reproduce.**
`commands/design.rs:1624` parses before `run::admit`, and a truncated payload
exits 1, leaves the revision untouched, and does not consume its `submission_id`
(`EVD-028`).

What the code reading found instead is an ordering. `apply` runs
`plan_checkpoints`, then pass 1 of `run::apply` against provisional ids, then
the `execute_mint` loop — which journals an intent and **materialises a
knowledge record on disk** — and only then `write_journal`,
`recheck_watermark_before_write`, and a second `run::apply` that can itself
refuse (`EVD-029`).

### Most of that is specified, not broken

`SPEC-029` owns the reserve-then-journal protocol and states it: *journal the
checkpoint intent, claim a canonical id, journal the id, materialise the record,
apply status and relations, then complete the snapshot — with every post-journal
recovery resuming the exact reserved id and **no failure path deleting authored
knowledge to repair a runtime error***. It states the late check too: the
watermark is *re-checked immediately before every write it guards, where a
divergence abandons the write without advancing the run while journalled effects
remain and stay recoverable.*

So unwinding materialisation on refusal is not merely unattractive — it is
prohibited by name, and it would be a second writer of the authored tier, which
this codebase has deliberately kept single.

### What is actually defective

The two-pass structure is the intended guard, and its own doc claims pass 1
refuses *while the authored tier is still untouched*. That guarantee holds only
where both passes refuse identically. They differ solely in the `Resolution`
handed in — provisional ids versus claimed — so **a refusal predicated on a
resolved id clears pass 1 and fails pass 2**, orphaning the record. That is the
residual defect once the spec's prescription is subtracted.

`DEC-250`: hoist every check that *can* be hoisted ahead of the mints, so the
claim the code already makes becomes true; and state the residual where a check
cannot be hoisted, because `recheck_watermark_before_write` reads state that may
change concurrently and is inherently late.

Leg 1's promise is therefore stated precisely rather than absolutely:

> On a refused or failed submission the revision, the receipt and the change log
> are unmoved, and the authored tier is unmoved **except across the named
> late-check window**, whose recovery is the journal.

`DEC-100` is the precedent for stating a window rather than designing it away —
it records the analogous `materialise` lost-update residual as consciously
tolerated and disclosed.

`ISS-361` stays **open** against this window. `EVD-028` ruled out its reported
mechanism and `EVD-029` names the suspect, but nothing ties its single witness
to either, and closing it on a plausible story would be the same species of
untruth this slice is about.

<!-- doctrine:section sec-7 -->
## Code impact

The design targets, by the change each carries.

| path | what changes |
|---|---|
| `src/design_run/payload_contract.rs` | The key inventory becomes the source refusal is driven from (`DEC-244`). Nine `UnknownKeys::SilentlyDropped` rows become statements of a rule instead of disclosures of a defect. |
| `src/design_run/submission.rs` | `ApplyRequest`'s `#[serde(flatten)]` envelope stops being load-bearing for strictness; `Declaration::WIRE_KEYS`' kind-axis table gains its state-axis counterpart (`DEC-246`). |
| `src/design_run/change_log.rs` | `ChangeLog.rows` becomes `Vec<StoredRow>`; `record` / `retain_window` / `covers` / `since` handle two arms (`DEC-251`). `PayloadTerm` construction gains the declared-kind check and `StepDischarged`'s `Outcome` declaration moves to `Label` (`DEC-247`). **`ChangeEvent` itself is untouched** — see the risk below. |
| `src/design_run/snapshot.rs` | The ordered-try deserialisation for `StoredRow`, and the legacy-fragment compat pins. |
| `src/design_run/run.rs` | `declare_node`'s create branch diffs against an empty prior instead of returning early; `CheckpointActGroup::record`'s replacement emits its own row; `live_acts` / `invalidation_rows` stop carrying the claim they cannot keep (`DEC-248`). |
| `src/design_run/render/envelope.rs` | The one production reader of the log's rows; gains the degraded-row disclosure (`STD-003`). |
| `src/commands/design.rs` | `apply`'s check ordering — hoisting what can be hoisted ahead of the mint loop (`DEC-250`). |

Three of these are the same class of change reaching three files, which is the
point: the sections above are organised by class because the fixes are.

<!-- doctrine:section sec-8 -->
## Verification

Per leg, and pinned at the tier that breaks rather than at the nearest
convenient unit.

**Leg 1.** A refused submission leaves the snapshot bytes, the revision and the
receipts unchanged. Separately — and this is the honest half — the named
late-check window is exercised for *recoverability through the journal*, not for
non-occurrence. A test asserting the window cannot happen would be asserting
something `SPEC-029` does not promise.

**Leg 2.** A same-kind act replacement emits `ActInvalidated`. A `needs` edge
declared at node creation emits `needs_added` — the control being the same edge
on an existing node, which is how `ISS-450` was distinguished from a dropped key
in the first place.

**Leg 3.** Each of unknown-key (at every nesting level, including inside the
flatten envelope and the internally-tagged enums), state-inert key, and
wrong-`ValueKind` term yields a typed refusal whose **reason** is pinned, not
merely whose exit code is. A refusal that fires for the wrong reason is a test
passing for the wrong reason.

**Leg 4.** `snapshot::parse` over a **literal legacy fragment** carrying a
retired `ChangeEvent` token parses, retains the row, and discloses it. Precedents
to copy, named by `mem.fact.design-run.snapshot-outlives-the-binary`:
`a_snapshot_written_before_the_policy_reads_as_human_only` and
`a_snapshot_written_before_the_intent_subject_key_still_parses`. A unit
round-trip over the inner type does **not** discharge this — it proves the type
reads what the type writes, which is not the question.

Also pinned: a structurally broken row (missing `revision` or `index`) still
refuses, so the tolerance cannot be mistaken for leniency about row shape.

**VA.** `doctrine design show 244` reads.

**Behaviour preservation.** `ChangeEvent`'s existing suites stay green
*unchanged* — that is the evidence that `DEC-251` sited the opacity correctly,
and it is the gate `AGENTS.md` requires when touching shared machinery.

<!-- doctrine:section sec-9 -->
## Risks, residuals and deferred debt

**R1 — the snapshot measurements date fast.** Every figure in this design (16
live snapshots, 15 parsing, `delegation = []` throughout, `SL-244` at revision
92 / floor 61) was taken on 2026-09-14 against a *runtime* tier that moves under
its own runs. Re-probe before implementing rather than trusting the numbers as
written. The two probes are in
`mem.fact.design-run.snapshot-outlives-the-binary`.

**R2 — resolved.** `flatten` versus strictness stops being a constraint once
`DEC-244` moves the check off serde.

**R3 — do not tidy the const proofs.** `change_log.rs`'s two `const _: ()`
blocks look redundant and are not. `is_subset(&EMITTABLE, &READABLE)` is the
only reader of `EMITTABLE` in `src/`, so deleting it passes `cargo check` and
fails `cargo test --bin doctrine` — a per-compilation-unit trap with no
attribute that substitutes.

**A1 — refuted.** `ISS-361` does not collapse into the silent-absorb class
(`EVD-028`). It stays open against `DEC-250`'s residual window.

### Deferred, by name

- **Splitting `Declaration` into wire and stored types.** The clean layering
  repair. Deferred because no live snapshot exercises the overlap — a
  justification that expires the first time delegation is used in anger.
- **Whether stage should gate acts it does not gate today.** The residual of the
  struck `ISS-362`, whose premise was disproved by repro: a `cp-` dispose and a
  `sec-` body are both *honoured* at `stage = exploring`. The witnessed
  inertness was `ISS-355`'s missing row, fixed by `SL-256`. This is a
  state-machine scoping question, not a defect in apply's exit signal.
- **Contract legibility and the section edit loop** — out of scope per the
  slice, and the next items in `RFC-031`.

### One thing this design owes its own subject

`ISS-450` was found by using the design run to design the design run, at
revision 3 of this slice's own inquiry. That is evidence for `DEC-248`'s
reasoning — the hunt for missed emission sites does not terminate — and it is
also a caution: the classes here are the ones visible from inside a single
run's worth of use. Others may not be.

