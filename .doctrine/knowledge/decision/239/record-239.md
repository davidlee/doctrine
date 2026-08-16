## Decision

**The vocabulary a design run may write and the vocabulary a persisted snapshot
may contain are different sets, and the type models them separately.**

`ChangeEvent` gains two rosters in place of the single `ALL`:

- `READABLE` — every event a persisted snapshot may contain and the renderer
  must handle. Governs the compile-time widest-name assert and the render
  containment checks, because historical rows must stay renderable within the
  same bounds.
- `EMITTABLE` — the events this binary may newly write. Governs writer coverage,
  and is what `every_material_event_kind_persists_a_change_row`
  (`tests/e2e_design_state.rs:1081`) iterates, because that test verifies the
  writer, not historical compatibility.

A retired member is renamed in Rust to carry a `Legacy` prefix while keeping its
serde name and rendered token, and keeps its **actual** stored payload shape:

    #[serde(rename = "acceptance_attested")]
    LegacyAcceptanceAttested,   // payload_terms() => &[]

The Rust name is the point: the type itself warns every future construction site
that the variant is read-only history.

## Why this, and not the two alternatives

**Not a bare serde alias.** Aliasing `acceptance_attested` onto `ActRecorded`
renames the *variant*, not the *row*. Historical `AcceptanceAttested` rows are
run-wide and term-free; canonical `ActRecorded` is subject-bearing with an `act`
term. The aliased row would render as `act_recorded` with an empty payload,
contradicting `ChangeRow`'s self-contained contract (`change_log.rs:475`). The
existing legacy-fragment test (`snapshot.rs:745`) proves whole-snapshot
*parsing*, not row-shape *normalisation*.

**Not whole-row normalisation at deserialise.** Synthesising the subject
(`cpa-design_accepted`) and the `act` term is logically defensible — the old
event deterministically implies them — but it makes old history impersonate the
current write vocabulary: the writer stored a run-wide term-free event, the
reader manufactures a subject and a term, and the next snapshot write persists
the manufactured history as though it had always had that shape. In a log that
describes itself as append-only history, a legibly-legacy row beats a
synthesised one.

## The premise is already written down

This does not invent a concept. `change_log.rs:69-70`, in the doc for the
existing `evidence_invalidated` alias, already states it:

> "`ChangeEvent` deserialises **strictly**, so one unrecognised `event` fails the
> whole snapshot rather than one row, and the change log is append-only
> *history* — **the vocabulary a run writes is not the vocabulary it must
> read**."

The type simply did not model it. One `ALL` forces readable and writable to be
equal, and the e2e enumeration then makes "every readable member must also be
written" a hard constraint — which is why "retain the variant but stop writing
it" was not available, and why `ISS-315` became a live breakage rather than a
routine retirement.

## Scope note

This amends `SL-256`'s original Non-Goal forbidding retirement or rename of an
existing member. The amendment was made deliberately by the human, not admitted
sideways as an incidental alias.

## What triggered it

`AcceptanceAttested` is redundant once acceptance flows through the shared
record seam. Both wire routes — the run-level `acceptance` field and an explicit
`checkpoint_act` — produce the same persisted value, a `CheckpointAct` of kind
`DesignAccepted`, and the gate reads that act rather than which request field
created it (`gate.rs:712`). A third, independent line agrees: `ChangeRow`'s own
doc says `subject: None` is for "a run-wide event — a stage move is about the
run, **which has no run-local id**". A `DesignAccepted` act *has* a run-local id,
so `AcceptanceAttested` being run-wide was already off-contract.

Measured migration surface: 7 `acceptance_attested` rows across 6 live snapshots
(slices 243, 244, 248, 249, 251, 254). For scale, the `evidence_invalidated`
legacy that `ISS-315` broke on is 8 rows — the same order.

## Deliberately not done

Full type enforcement — an `EmittableEvent` wrapper or a second enum making
legacy emission unrepresentable — is **not** adopted. `Pending` still accepts any
`ChangeEvent`. The separate roster, the `Legacy` name prefix, the e2e
enumeration over `EMITTABLE`, and a test asserting the legacy member is absent
from `EMITTABLE` are proportionate; a near-duplicate enum would cost more
synchronisation machinery than this vocabulary warrants.

## Coherence with QUE-219 — read this before ruling that question

`QUE-219` (*Submission strictness against snapshot forward-compatibility*) asks
whether the engine should refuse a submission key the schema does not hold, and
how that reconciles with stored state outliving the binary. Its three candidate
answers are **strict on the write path, tolerant on the read path, or migrate**.
It gates `ISS-290`, `ISS-327`, `ISS-328` and `ISS-333` specifically so those four
do not receive four inconsistent rulings.

This decision is "**strict on the write path, tolerant on the read path**",
applied to the change-event vocabulary rather than to submission keys. It is
therefore either a worked precedent for `QUE-219`'s first candidate on an
adjacent surface, or a partial pre-commitment of its answer. The human took that
tradeoff knowingly. Whoever rules `QUE-219` should treat this as evidence about
one surface, not as having settled the question: the surfaces differ in that a
rejected submission key can be re-sent by its author, while a rejected stored row
cannot be re-written by anyone.

## Provenance

Reached in the `SL-256` design run with GPT-5.5 (codex) as peer advisor. Codex
initially preferred whole-row normalisation and moved to the roster split when
shown the `change_log.rs:69-70` quote. The `Legacy` naming and the
proportionality argument against a second enum are codex's.


## Correction, 2026-08-16 — the retirement's pinning mechanism, and STD-001

Raised as `RV-360` `F-4` by an external adversarial reviewer during `SL-256`'s
design review, contested once, and upheld on the second pass. The decision's
substance is unaffected; one mechanism in it is replaced.

**What is replaced.** The snippet above pins the wire name with
`#[serde(rename = "acceptance_attested")]`. That writes `acceptance_attested` a
second time, beside the literal `as_str` already spells, and `STD-001` —
**required** — asks that a recurring meaningful token be named once and
referenced everywhere. A design section cannot grant an exception to a required
standard, and the drift guard first offered in repair detects divergence without
single-sourcing anything.

**What replaces it.** `ChangeEvent` takes
`#[serde(try_from = "String", into = "String")]`, with `Into` returning
`as_str().to_owned()` and `TryFrom` resolving a token by scanning `READABLE`,
plus one arm for the `evidence_invalidated` legacy alias. This is the module's
own idiom, not a new one — `DesignId` (`ids.rs:131`), `IntentSubject`
(`attestation.rs:935`) and `PayloadTerm` (`change_log.rs:387`) already carry it,
and `Refusal`'s `Display` doc names serde's `try_from` as the boundary it exists
to cross (`refusal.rs:457-460`). `as_str` becomes every token's only source, and
`rename_all`, `alias` and `rename` all leave the enum.

**Why this is larger than the finding.** The finding read the duplication as
something the retirement *introduced*. It is not. `rename_all` derives 22 tokens
from variant identifiers while `as_str` hand-writes the same 22; they agree by
convention and nothing enforces it. The retirement is merely the first change to
make one instance visible. So the repair is applied to the class — after it, no
member has two sources.

**What is unchanged.** Every ruling above stands: the readable/emittable split,
the `Legacy` Rust prefix as the warning to future construction sites, the
preserved wire token and stored payload shape, the refusal of a bare alias, and
the refusal of whole-row normalisation. Strict deserialisation is preserved
rather than relaxed — an unmatched token is a `Refusal`, exactly as the derive
refused it, which is what `ISS-315`'s defect class depends on. "Deliberately not
done" is also untouched: it declined an `EmittableEvent` wrapper on
proportionality grounds, and token single-sourcing is a different question.

**Cost carried.** `TryFrom` needs one new `Refusal` variant, so
`src/design_run/refusal.rs` joins `SL-256`'s selectors. The fence widening is
declared in that slice's `sec-4` rather than discovered at execution.
