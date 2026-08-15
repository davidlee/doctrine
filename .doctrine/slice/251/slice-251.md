# Acts carry their own payload contract

## Context

`doctrine design apply` takes one JSON payload and documents none of it. Its
`--help` lists exactly one option — `--input <INPUT>` — and the run's own
guidance supplies a single worked example rather than a contract. An agent that
needs to know which keys an act accepts has two ways to find out: read
`src/design_run/submission.rs`, or submit and read the refusal.

This slice takes **one** of `IMP-390`'s four candidates — *make the payload
contract fetchable rather than exemplified* — and nothing else. The other three
(`next_obligation`'s missing writer, forward-rendered gate conditions, refusals
naming remedies) stay with `IMP-390`.

**Why this candidate, and why alone.** It carries the most evidence of the four:
RFC-026 **E8.7** measured 33 source reads against 52 `doctrine design` calls over
`CHR-049`'s run, and **15 of those 33 were payload-shape lookups** — the single
largest category. It is also the only one of the four that needs no v1 envelope
wire change; `IMP-390`'s first candidate explicitly does, under `DEC-064`, because
`next_obligation` renders as elided prose and so cannot carry a closed vocabulary.
Bundling the cheap face with the expensive one would price this work at the
expensive one.

**It independently discharges a second item.** `ISS-333` records that
`ApplyRequest` carries `#[serde(flatten)] envelope`, that serde cannot reconcile
`flatten` with `deny_unknown_fields`, and that the outermost submission type
therefore cannot refuse a key it does not know — a nonsense top-level key is
discarded, the revision bumps, a receipt is written, no change row prints, and the
command exits 0. Witnessed three times in succession on `SL-248`'s design run,
2026-08-06, while probing for the section payload shape. That item lists three
candidate fixes; the third is *"accept the limit and close the discoverability half
instead — a schema dump would defeat the probing loop that made this cost three
revisions, without touching serde."* This slice is that option. It does not repair
`ISS-333`'s mechanism (the silent discard remains), and must not be recorded as
having done so.

**The exemplification is not merely thin, it is wrong in a load-bearing place.**
The `declare` hint shows one `traversal` example carrying `pin` / `posture` /
`authority` and omits `cursor` — the one key a resuming agent must set. Fifteen
reads of `submission.rs` is what that omission cost.

**The seam already exists, twice, and this slice rides it rather than parallelling
it.** `SL-244` established the register for this exact problem on the neighbouring
axis: a gate condition now states its own contract, structure in a const table
beside `boundary_conditions` (`DEC-123`), narrative in sealed prose assets
(`DEC-122`), with set-equality across the enumerations as the anti-drift pin. On
the payload axis two tables are already in-tree:

- `ApplyRequest::WRITER_ACTS` — nine top-level acts, each paired with the
  predicate that detects its presence (`submission.rs:989`).
- `Declaration::WIRE_KEYS` — fifteen declaration keys, each paired with the
  subject kind that honours it and the predicate that detects it
  (`submission.rs:563`), landed by `SL-249` for `ISS-318` and pinned two
  independent ways: `I9` (this key set equals a fully populated declaration's
  serde key set) and `I10` (the behavioural matrix).

`SL-249`'s `I9` is the answer-shape this slice inherits: a hand-authored table
whose correspondence with the serde key set is pinned by a test that fails when
they drift, with an exhaustive no-`..` struct literal making a newly added field a
compile error at the pin rather than a silent omission from the table.

**Confound, carried over from `IMP-390` rather than left to the reader.** An agent
with the engine's source in its tree will read it, and doctrine dogfooding itself
makes that unusually cheap. The 33 reads are therefore a *lower* bound on the
confusion and an *upper* bound on the remedy: an installed client project has no
source to read, and the same opacity there produces guessing instead.

## Scope & Objectives

### 1. The payload contract becomes fetchable

An agent about to submit an `apply` payload can ask the binary what that payload
may contain, and get an answer that enumerates the acts and, per act, its fields —
rather than one example it must generalise from. What "fetchable" means concretely
is `/design`'s call, and the two candidates `IMP-390` names differ in kind:

- a **schema surface** — a machine-readable dump of the payload contract; or
- a **`--help` that enumerates each act's fields** — the same content in the place
  a caller already looks.

They are not exclusive and the choice is a design decision, not a scoping one.
Whichever lands must at minimum cover the omission that provoked this: `cursor` is
reachable without reading source.

**Settled by inquiry (2026-08-15), `DEC-224`.** Three renderings from one
generator, not two: a `doctrine design contract [--format json|prompt]` verb (the
schema surface, and the primary one); a one-line *pointer* in `design apply
--help` rather than a full enumeration there; and a generated reference doc
published into the library, sealed `customization = "fixed"` (`DEC-226`). The
`cursor` omission is covered twice over — by the contract itself and by
`DEC-228`, which corrects the worked example in place.

### 2. The contract is pinned against the types

A test must fail when the contract and the types drift. **Corrected by inquiry
(2026-08-15), `DEC-221`:** the original "authored, not derived from the types by a
proc macro" no longer holds as written. The pin splits by what it pins — enum
token vocabularies are **generated**, on `SL-244`'s `condition_vocabulary!`
pattern, because tier-2 equality has no oracle for a variant set; struct field
sets stay **authored and literal-pinned**, on `SL-249`'s `I9` pattern, because
there the exhaustive no-`..` literal already makes a new field a compile error at
the pin. `DEC-221` is held loosely at the user's direction: this is type-design
detail the implementing agent may overturn on evidence.

### 3. Totality across the act vocabulary

The known asymmetry to resolve rather than inherit: `ApplyRequest` has ten
payload-bearing fields but `WRITER_ACTS` has nine rows — `delegation` is absent,
because that table answers "which act writes" for `EX-2`, not "which keys exist".
A contract surface keyed naively off `WRITER_ACTS` would therefore ship a payload
field no caller can discover. Whether the existing table is extended, mirrored, or
left alone beside a new one is a design decision; that the surface is **total over
the payload** is a scope commitment.

**Settled by inquiry (2026-08-15), `DEC-227`.** Totality is the full recursive
wire closure rooted at `ApplyRequest` — roughly twelve wire-participating structs
plus the enums they admit, including `AgentAct` over in `attestation.rs`. Not
top-level only: both failures observed during this slice's own design run were
nested (an unknown key on `Declaration`; `AgentAct`'s external tagging inside
`agent_declaration`), so a top-level contract would have prevented neither. The
closure's boundary is reachability, which is mechanically checkable, rather than
a judgement about what counts as internal — the same class of judgement that made
`WRITER_ACTS` the wrong axis.

## Non-Goals

- **`IMP-390`'s other three candidates.** `next_obligation`'s missing writer (a v1
  wire change under `DEC-064`, and whose ownership between `IMP-390` and `IMP-367`
  is itself unsettled), forward-rendered unmet gate conditions for the next stage,
  and refusals naming remedies. `IMP-390` stays open holding all three.
- **`ISS-333`'s mechanism.** No hand-written `Deserialize`, no
  deserialize-to-`Value`-and-subtract. The unknown-key discard survives this slice;
  only the probing loop it caused is removed. `ISS-333` stays open on its
  serde axis.
- **Read-path tightening of stored declarations.** `ISS-328` and `ISS-333` both
  record the constraint: stored proposal declarations ride the run snapshot and
  outlive the binary that wrote them
  (`mem.fact.design-run.snapshot-outlives-the-binary`), so tightening a read path
  converts previously-readable stored state into a parse failure at exactly the
  moment someone is resuming. A description surface does not touch a read path,
  and must not acquire one.
- **A project-authored contract.** `DEC-122` deferred client-project authorship of
  condition contracts; the same deferral holds here. No override seam, no loader.
- **Generalising beyond `design apply`.** Other commands with JSON payloads are
  out; if the shape generalises, that is a follow-up.

## Affected surface

Coarse, seeded as `scope-relevant` selectors — the exact touch-set is `/design`'s
job:

- `src/design_run/submission.rs` — `ApplyRequest`, `WRITER_ACTS`,
  `Declaration::WIRE_KEYS`, the act types.
- `src/design_run/prompt.rs`, `src/design_run/render/**` — where the `declare`
  hint and envelope rendering live.
- the `design` command surface — where a `--help` or schema verb would attach.
- `install/design-prompts/**` — the prose assets, if the contract ships partly as
  narrative on `DEC-122`'s pattern.
- `src/design_run/tests.rs` — the drift pin.

Added by inquiry (2026-08-15), from the decisions above:

- `src/design_run/attestation.rs` — `AgentAct` and `ActKind` are inside
  `DEC-227`'s closure, so the contract crosses into this module. Both it and
  `submission.rs` are leaf tier, so no layering edge is created.
- `src/commands/design.rs` — `DEC-224`'s verb, `DEC-225`'s wrap of the payload
  parse refusal at the `serde_json::from_str` site, and the `--help` pointer.
- `publication/manifest.toml` and `src/publication.rs`'s vocabulary —
  `DEC-224`'s published doc and `DEC-226`'s `customization = "fixed"`.
- **Layering constraint, corrected.** `ADR-001` classifies `design_run` as
  **leaf, out-degree 0** (`layering.toml:31`), not engine tier as first triaged.
  The contract table and its renderer must therefore be pure and import nothing
  from the crate; embedding, publication and CLI attachment sit above them in
  command tier.

## Risks, assumptions, open questions

- **R1 — a second description of the payload that can go stale.** The whole point
  of `I9` and `DEC-123`'s set-equality is that this risk is answered by a test
  rather than by discipline. A design that lands the surface without the pin has
  shipped the risk.
- **R2 — the surface is fetchable but not found.** `SL-244` shipped narratives and
  a generated stage reference; whether an agent reaches them is a separate
  question from whether they exist. Discoverability from the refusal and from the
  turn envelope is in scope for `/design` to consider, within the wire constraint.
- **A1 — no v1 envelope wire change is needed.** Assumed on `IMP-390`'s own
  reading of `DEC-064`; if `/design` finds a chosen shape needs one, that is a
  `/consult`, because it would import the cost this slice was carved to avoid.
- **R2 — answered (2026-08-15) by `DEC-225`.** The contract's *address* is pushed
  into the payload parse refusal and into the turn envelope's no-drop region; the
  body stays behind the fetch verb, which is what keeps `DEC-064`'s byte budget
  intact. Injecting the full contract into `resume`'s `contract_section` channel
  was weighed and declined, not foreclosed — it is the recorded escalation if
  cold-re-entry agents prove to fail before fetching.
- **OQ-1 — resolved (2026-08-15) by `DEC-221`.** Neither purely generated nor
  purely authored: the pin splits by what it pins. Note the framing itself was
  re-cut before the decision — research established the real fork was tier 2
  (serde-key-set equality against an exhaustive literal) versus tier 3
  (single-source generation), with tier 1 (hand-written test-copy set-equality)
  excluded because it is blind to the failure it claims to prevent.
- **OQ-2 — resolved (2026-08-15) by `DEC-224`.** Three renderings, one generator.
  See § 1 above.
- **OQ-3 — resolved (2026-08-15) by `DEC-219`.** Shape *and* engine semantics: key,
  type, required/optional, owning act, and the token vocabulary of every enum a
  key admits, including serde tagging style. No per-act narrative asset; that
  option is deferred on `DEC-122`'s pattern, not rejected.

## Verification / closure intent

- The payload contract for every act on `ApplyRequest` is reachable from the
  binary without reading `src/`, and `cursor` in particular is among them.
- A test fails if a payload field is added, removed, or renamed without the
  contract following — pinned against the serde key set, not against review.
- The surface is total over the payload, not over `WRITER_ACTS`.
- `ISS-333` option 3 is discharged and recorded as such; `ISS-333` stays open on
  its serde axis and `IMP-390` stays open on its other three candidates. Closure
  states plainly what was *not* done.

## Summary

## Follow-Ups
