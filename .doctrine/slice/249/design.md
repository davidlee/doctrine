<!-- doctrine:section sec-1 -->
# 1. Design Problem

Between a ruling being known and a record holding it, Doctrine loses content —
twice, silently, and at the moment the content is most available.

The **structured tier has no writer.** `doctrine knowledge` offers `new | list |
show | inspect | status | paths`. Filling a `[facet]` means hand-editing TOML,
and the corpus records the result: decision facets 35/142 populated, question
4/38, and `answer`/`answered_by`/`answered_on` on `QUE` at **0 of 38**.
Population is binary and lock-step — a record carries every textual field or
none — which is the signature of a population that either engaged the tier
deliberately or never touched it at all.

The **design-run wire has no slot.** `CreateRecord` accepts `kind`, `title`,
`slug` and `acceptance`. An agent disposing a checkpoint holds the ruling in
hand as it writes, and has nowhere to put it. On SL-248's run six dispositions
reached for the nearest-looking key and sent their prose as `body`;
`Declaration::body` is *section* prose, read only behind a guard on
`derived.section_digests`, so on a `cp-` subject it was accepted and discarded.
`DEC-155`…`DEC-160` minted hollow and ~15k tokens were re-authored. The operator
caught it, not the tool.

The two are one story — a missing slot and a silent sink — and fixing the sink
alone converts a silent loss into a dead end.

What must be true at the end:

- Every facet field of every facet-bearing kind is writable through a verb, and
  the prose tier with it.
- A `form = "create"` disposition mints a record already holding its facet and
  its prose, in one act.
- A wire key that is inert at its subject's kind is refused at submission,
  across the whole field set rather than at the instances someone noticed.
- The kinds the corpus actually has are the kinds governance describes, and that
  correspondence is checked by something other than a promise.

<!-- doctrine:section sec-2 -->
# 2. Current State

## The read side is complete, and stays put

`src/knowledge.rs` carries the whole structured read model: `RecordFacet` as an
enum-of-structs with one variant per kind, four closed value-enums (`Confidence`,
`Basis`, `ConstraintSource`, `Provenance`), a kind-blind `RawFacet` superset for
the tolerant parse, `validate_facet` dispatching on `record_kind` through the
`"" -> None` seam, and a `#[cfg(test)]` `render_facet` pinning the byte-stable
round-trip. `A1` assumes this is sound and unchanged; nothing below revises it.

The facet field inventory it defines is the fact that shapes the CLI surface:
**31 slots, 30 distinct names, exactly one shared** (`confidence`, on assumption
and evidence) — assumption 8, decision 7, constraint 6, question 5, evidence 3,
hypothesis 2, concept 0.

## The write side is a single module with a single consumer

`src/facet_write.rs` holds `set_facet_mixed` / `apply_set_mixed`, a mixed-type
`[facet]` writer over `toml_edit`, serving `doctrine risk set` at
`src/commands/facet.rs:711`. Its `deletes at SL-222 deletion phase` marker covers
only three float-valued symbols and its reason string's premise is false; that is
`CHR-056`, not this slice. The module is **anchored by no spec** — no `sources`
list names it and the string appears nowhere in the authored corpus. Its only
governance is `SPEC-004`'s edit-preserving clause. (`inq-9`, open.)

Two write postures already exist and are distinguished deliberately in
`src/dep_seq.rs`: `apply_status` refuses a missing key (F-1: scaffold-seeded,
edit in place, never create), `apply_scalar` creates one. `DEC-170` rules facet
fields into the first class.

## The status verb is deliberately uncoupled

`set_record_status` documents itself: *"No resolution coupling: `status` and
`updated`, nothing else."* So a question reaches `answered` with its answer
fields empty, which is exactly the 0-of-38 above. `accepted_status` is the
counter-example worth noticing — it derives a per-kind answer from the kind's own
vocabulary rather than enumerating one, and says so.

## The design-run wire

`CreateRecord` (`src/design_run/submission.rs`) carries `kind`, `title`, `slug`,
`acceptance` — no facet, no prose. `Declaration` carries
`#[serde(deny_unknown_fields)]` (`A2`, confirmed), so an unknown key is already
refused; what is *not* refused is a **known** key inert at the subject's kind,
which is `ISS-318`. Its two observed instances are `body` on a `cp-` subject
(accepted and discarded) and `dispose` on an `inq-` subject (skipped by
`plan_checkpoints`, `src/commands/design.rs:882`).

Record creation runs through DEC-086's step sequence: reserve → materialise →
apply effects. `create_record` is the single fresh-creation path for all seven
kinds and its own doc forbids a second reservation+scaffold path;
`apply_record_effects` is step 5, already idempotent, already applying the
acceptance→status move and the `shapes` edge.

## Governance as it stands

`SPEC-019` is emphatically four-kind, repeating "four" at nine sites, and carries
a now-false self-description (*"forward-intent: no code is shipped yet"*).
`PRD-010` § 4 carries the same enumeration plus an extension rule. The corpus has
seven kinds. `SL-159` (`EVD`+`HYP`) and `SL-197` (`CPT`) each ruled their
contracts in a closed slice's design and neither landed the governance axis it
scoped — the debt this slice pays (`inq-7`, open on whether it discharges them
by name).

<!-- doctrine:section sec-3 -->
# 3. Forces & Constraints

## Binding

- **`SPEC-004`** — mutating verbs write entity TOML **edit-preservingly**. Ride
  `toml_edit`; never reserialise. This is the one clause governing
  `src/facet_write.rs` today.
- **`DEC-170`** — F-1 posture on facet fields: refuse an absent key, never
  create it; spell a cleared value as `""`, never by omission. Facet fields are
  scaffold-seeded, so an absent one is damage. `set_facet_mixed` currently
  *creates* missing keys, so riding it needs either a call-site guard or a
  posture parameter.
- **`DEC-088`** — a decision reaches `accepted` only through a content-bound user
  attestation applied at step 5. No verb this slice adds may offer a second
  route.
- **`DEC-168`** — the facet-and-prose write happens at step 5
  (`apply_record_effects`), not by pre-filling the scaffold. Forced by
  crash-resume: `materialise_record_at` re-scaffolds from a journal carrying only
  id, title and slug.
- **`ADR-013`** — the governance amendment routes through a REV landing at
  reconcile, over **two** entities (`SPEC-019`, `PRD-010` — `DEC-175`).
- **`ADR-004` / `SPEC-018`** — `link`/`unlink` own relations; `edit` does not
  touch them.
- **`ADR-001`** — leaf ← engine ← command, no cycles. The pure write seam is a
  leaf; the CLI verbs and the design-run wire are both consumers of it.
- **`STD-001`** — no magic strings. Every field name, status token and kind
  prefix in this design must have exactly one spelling with one owner.
- **`POL-002`** — platform independence: no host-project convention (cargo
  layout, `just` recipes) leaks into engine rules. `DEC-176`'s canary is a
  project-local test, never a `validate` rule.

## Not binding, and worth stating

- **`SPEC-013`** owns the uniform `<kind> <verb>` grammar but states no
  convention for field-mutation verbs. `memory edit` / `backlog edit` are
  precedent, not governance, so the subverb shape and `settle` are unconstrained
  by it.
- **`PRD-019` / `SPEC-029`** govern the design run's identity, idempotency and
  revision-CAS and say nothing about what a created record is *populated* with.
  Objective 2 is ungoverned space, not prohibited space.

## Mechanical

- **Rust has no reflection over struct fields.** Any correspondence between a
  type's fields and something else is authored, and the design question is only
  how it is kept total (`DEC-169`).
- **`Declaration` carries `deny_unknown_fields`; `ApplyRequest` cannot** (it
  carries a `#[serde(flatten)]` envelope). The facet payload therefore lands
  inside `Declaration`, where the guard already holds.
- **Serde's `skip_serializing_if` is total over `Declaration`**, so a
  fully-populated value serialises to exactly the wire key set — the pin
  `DEC-169` uses instead of a proc macro.
- **A `toml_edit` root insert-if-missing is safe; a subtable-nested one is not**
  (`mem_019ee9fd51d87aa38a2dfb31ad6c4eec`, which scopes its own proof and says
  so). `[facet]` fields are subtable-nested, which is why F-1 stands.

## Pressures in tension

The slice exists to close a live data-loss hole, and the user's standing
tie-breaker is to prefer whichever defensible answer lands that fix sooner. Set
against it: this design touches shared machinery (the entity engine, the design
run wire) where the behaviour-preservation gate applies, and it writes governance
that outlives the fix. `DEC-165`'s phase boundary is how both are honoured — the
wire fix ships first and alone, the facet-bearing surfaces follow.

<!-- doctrine:section sec-4 -->
# 4. Guiding Principles

Five, each earned by a ruling this run took rather than asserted as taste.

**Derive the correspondence; do not declare it.** Three separate tables sit in
this design — wire keys against subject kinds, facet keys against record kinds,
status tokens against their evidence fields. Every one of them is authored,
because Rust gives no reflection; none of them may be *checked* by hand. `serde`
key sets pin the first (`DEC-169`), name correspondence derives the third
(`DEC-178`), and the second is pinned against the typed model. A table that can
silently miss a new field is `ISS-318` recurring inside its own fix.

**Refuse where the operator is; report where the reader is.** The same damage
warrants opposite treatment on the two paths. A write-time refusal fires at the
record it names, with someone holding the thing that caused it (`DEC-170`). A
read-time refusal fires at every later reader of an unrelated record and breaks
the tool you would use to diagnose it (`DEC-177`). Symmetry is not the goal;
locality of the report is.

**One table, many consumers — never one table per consumer.** The per-kind facet
field sets are needed by the six subverbs, by `settle`'s coverage, and by the
doctor tripwire. Three hand-maintained copies is the failure this slice is
already paying for elsewhere.

**Make the wrong call unrepresentable where it costs nothing; refuse it clearly
where it does.** Per-kind subverbs make a wrong-field error unspellable, and
relocate rather than remove the kind-mismatch check — accepted, because the check
is one comparison against a value the id already resolves. Where a refusal is
what is left, it teaches: *"`ASM-003` is an assumption; use `knowledge edit
assumption`."*

**Ship the fix ahead of the governance that describes it.** `DEC-165`: the
rulings gated, objective 4 never did. The phase boundary is load-bearing and the
plan must hold it — the wire fix may not quietly absorb facet work to save a
phase.

