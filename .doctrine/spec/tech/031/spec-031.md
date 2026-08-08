# SPEC-031: Phase plan surface

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See glossary.md § reference forms. -->

## Overview

The phase plan surface is the **content model of a slice's authored
`plan.toml`** — the artefact the "no code without an approved plan" gate gates
*on*. It realises **PRD-001**'s plan requirements (`REQ-439`–`REQ-447`) and is a
component of the entity engine (**SPEC-004**), sibling to **SPEC-014**.

The split with SPEC-014 is the load-bearing joint. SPEC-014 owns the plan as a
**fileset**: a non-reserved sibling kind materialised into an existing slice
directory, `plan.toml` + `plan.md`, no id or slug of its own, template tokens
substituted from the parent slice. This spec owns everything *inside* that file
— phases, criteria, modes, order, links, evolution, validation, and the read.
Before it, that contract was normative only in shipped reference prose
(`glossary.md`, `using-doctrine.md`, the boot snapshot) and in a template
comment: real rules, at a tier no spec descended from, no code anchored, and
`/reconcile` could not write through.

It also discharges the three questions `PRD-001` left for its descending
technical spec: **`OQ-2`** (act identity) is settled by D9, **`OQ-3`** (ordering
under relocation, split and merge) by D10, and **`OQ-4`** (migration versus
compatibility read) by D11.

**Posture is dual and the distinction is explicit.** Sections marked *Shipped*
describe mechanism that exists in `src/plan.rs` / `src/vtgate.rs` today.
Sections marked *Planned* are forward intent — authored requirements at status
`pending` with no implementation. Nothing here derives status from the spec;
observed coverage is reconciled, never inferred (**PRD-013**).

## Responsibilities

Mirrors the structured `responsibilities` list: the file contract; identity and
order; the VT mandate and its gate; the evolution contract; the governance link
surface; validation and read; and the ownership joints.

### The authored file contract — *Shipped*

`plan.toml` carries a three-field header — `schema = "doctrine.plan.overview"`,
`version`, and `slice` (the parent's canonical id) — followed by two plan-level
link tables and an ordered `[[phase]]` array. Each phase row carries `id`,
`name`, `objective`, `entrance_criteria`, `exit_criteria`, `verification`, and
its own narrowing `specs` / `requirements` arrays.

Two criterion row shapes, and the asymmetry is deliberate:

- **Condition rows** (`entrance_criteria`, `exit_criteria`) are `{ id, text }`.
  Prose is the normative statement; no machine reads it.
- **Verification rows** carry `{ id, expects, test_file, keywords, patterns,
  waived, waived_reason }`. `expects` is free text; the rest is the structured
  mandate a machine can check.

Every field but the phase `id` defaults, so a plan authored before any given
field existed parses unchanged into defaulted empties. That is the
behaviour-preservation gate, not an accident of serde: the plan is hand-edited
and long-lived, and a reader that refused older files would strand the corpus.

The prose sibling `plan.md` carries narrative only. It is not a second
authority: no reader parses its headings.

### Identity and order — *Shipped*

Phase ids are `PHASE-<digits>`, unique within a plan, and **immutable once
authored** — edits append, they never renumber or reuse. Uniqueness is refused
at parse; per-id well-formedness is enforced at the filesystem boundary, where
the id becomes a phase-sheet filename.

**The authored array order is the phase order.** Id arithmetic is never the
authority. This matters precisely because ids are immutable: after a split or an
inserted phase, the id sequence is non-monotonic while the array still reads
top-to-bottom, and any consumer that sorts by id silently reorders the plan.

Criterion ids are **document-local**: bare, unpadded, scoped to their owning
phase, and immutable on the same terms. They are not corpus entity ids —
`STD-002`'s id-is-identity rule governs prefixed entity ids and does not reach
them. Consequently a criterion reference outside its owning phase must be
**phase-qualified** (`PHASE-03/EX-8`); bare `EX-8` means nothing one phase over.

The id prefix carries the class or mode; there is no separate field:

| Prefix | Class | Machine consumer |
|---|---|---|
| `EN-` | entry condition | none |
| `EX-` | exit condition | none |
| `VT-` | verification by automated test | the gate below |
| `VA-` | verification by agent check | none |
| `VH-` | verification by human acceptance | none |

Entry and exit are *conditions*; VT/VA/VH are *verification modes*. A criterion
selects exactly one, and the two axes never collapse — an exit condition is not
evidence.

### The VT mandate and its gate — *Shipped*

A `VT-` row's structured mandate is what makes it checkable: `test_file` (a
project-root-relative path that must exist), `keywords` (raw substrings that
must appear in it), `patterns` (optional line-anchored regexes — the
stronger-shape escalation), and the `waived` / `waived_reason` escape valve.

Two checks run at **plan time**, before execution:

1. **Shape completeness.** A non-waived VT without `test_file` is
   `BareTestFile` (uncheckable at runtime); with `test_file` but empty
   `keywords` it is `BareKeywords` (a vacuous pass); a waived VT without a
   reason is `MissingWaiverReason` (an opaque waiver — a soft warning, not a
   failure).
2. **Selector fit.** A non-waived VT whose `test_file` is declared by no
   design-target selector is flagged, using the *same* shared predicate the
   dispatch import belt applies at integrate time — so a plan-time flag is
   exactly what the belt would later refuse. Empty selectors yield an empty
   result: an unscoped plan cannot under-declare.

At **execution time** the gate judges each VT row into one of five verdicts:

| Verdict | Meaning | Halts |
|---|---|---|
| `Pass` | mandated file was modified by the slice; every keyword and pattern present | — |
| `Fail` | file missing, or a mandated keyword or pattern absent | **yes** |
| `Uncheckable` | no structured mandate — nothing to check | no |
| `Unattributable` | file exists but the slice did not modify it; the match predates the work | no |
| `Waived` | human-authorised escape, reason surfaced | no |

Order is load-bearing: waiver short-circuits before any filesystem read, then
absent-mandate, then file-missing, then attribution, then the keyword and
pattern match.

**The property worth naming: no non-`Pass` verdict is silently green.** Four
distinct non-halting outcomes stay visible and separately labelled rather than
collapsing into a pass. A gate whose zero-evidence case reads as success is the
defect this taxonomy exists to avoid.

**Threat model is worker omission, not an adversary.** Plain substring matching
over raw bytes is the proportionate floor against a weak worker skipping
mandated work. Semantic correctness of the assertion is a non-goal, and a
keyword satisfied from inside a comment is an accepted, documented weakness —
comment and string syntax is host-*language* convention, and stripping it would
load-bear correctness on the host language, which `POL-002` bars. An author
wanting a code-shape assertion uses `patterns`, which stays language-agnostic
because the author owns the regex.

### The governance link surface — *Planned*

`plan.toml` carries a plan-level `[specs]` (`primary` / `collaborators`) and
`[requirements]` (`targets` / `dependencies`) block, plus per-phase `specs` /
`requirements` narrowing arrays. **Authors populate them and the reader
discards them** — `Plan` deserialises only `phases`, and `PlanPhase` models no
such fields, so every authored link is parsed past. No gate, projection, or read
surface consumes it (`ISS-321`).

This spec governs those tables as part of the plan contract: the reader must
model them, and a read surface must expose them. `REQ-439` AC-2 already requires
every phase to state its applicable canonical links, and the corpus already
carries the data — this is a modelling gap over authored truth, not greenfield.

Per `REQ-447` the plan **cites** canonical requirement and specification
identities; it never restates their normative content as independent plan truth.

### The verification→exit binding — *Planned*

A phase's exit criteria are its obligations; its verification criteria are how
they are proved. **No authored field connects them** — the mapping lives only in
the author's prose, so no consumer can derive which obligations are proved,
which are unproved, or what an obligation's evidence is (`IMP-409`; `RFC-029`
§ 1, where two independent studies found this gap from opposite ends and it was
the only candidate fact not already owned).

D6 settles where the link lives: **on the verification row**, naming the exit
criteria it discharges, phase-local by default and phase-qualified across
phases. The inverse — *what proves `EX-3`* — is derived, never authored twice.

### Criterion evolution — *Planned*

When a criterion's **meaning** changes, an authorised plan revision mints a new
immutable record and records the predecessor's disposition; the predecessor is
preserved and stops governing. Five acts are in scope: **replacement,
withdrawal, one-to-many split, many-to-one merge, and cross-phase relocation**.
Relocation is not an id move — the criterion is withdrawn in the source phase
and a successor is minted in the destination, because ids are phase-scoped and
never reused.

Two boundaries hold the cost down:

- **A non-semantic correction is not an evolution.** Fixing a typo or clarifying
  wording edits in place with no ceremony. Records stay immutable; active
  semantics stay correctible.
- **An unchanged criterion is zero-ceremony.** It governs with no evolution
  record and no restated shadow copy. The governing set is *derived* from valid
  lineage, never maintained as a second authored list.

**Evolution carries no act identity of its own** (D9). Predecessor and successor
sets plus a disposition carry the whole audit history; a third identity space
beside phase and criterion ids would need a consumer that does not exist.

**Order under evolution** (D10): the governing set renders in the authored row
order of the *governing* rows within their owning phase. A successor orders by
where it is authored, never by its predecessor's former position and never by
id. A relocated criterion therefore orders by its position in the destination
phase. This is the same rule as D2, applied one level down, and it is what keeps
a plan deterministic after a split, merge, or relocation without renumbering
anything.

The **storage location** of lineage — an opt-in field on the criterion row
versus a separate lineage block — is deliberately not fixed here (`IMP-382`
boundary; OQ-2 below). That is a narrower question than PRD-001's `OQ-2`, which
asked about act identity and is settled by D9. This spec fixes the contract
those rows must satisfy, so a later design has a governed host to descend from,
which is exactly what was missing.

### Validation and the read — *Planned, partially shipped*

*Shipped:* the parser refuses a duplicate phase id; the filesystem boundary
refuses a malformed one; plan-time checks flag bare and undeclared VTs.

*Planned:* before a plan governs execution it is refused for duplicate or
invalid identities, an unsupported criterion mode, an ambiguous cross-phase
reference, a missing predecessor or successor, a lineage cycle, malformed split
or merge cardinality, and an indeterminate governing order. Every refusal names
the offending subject and reason, and **validation never silently repairs
authored truth**.

The ordinary read presents one deterministic ordered governing criterion set.
Superseded and withdrawn criteria are never presented as governing — including
after split, merge, or relocation — and remain reachable as history, with
predecessor and successor identities, dispositions, and phase locations
readable without Git archaeology.

**Migration, and where its boundary falls** (D11). A content-model change that
only *adds* defaulting fields is served by a **compatibility read** — no
authored migration, because an older plan's existing rows keep their meaning.
An **authored migration** is required only when a change would otherwise alter
the meaning of rows already on disk; it is deterministic, reviewable before it
governs, and a failure leaves the prior authored plan intact while reporting the
blocking incompatibility.

Concretely: a plan authored before lineage support remains readable with its
existing criteria governing in declared order, because *absence of lineage means
every authored row is a governing leaf* (D5). That is a compatibility read, not
a migration. No lineage record is ever back-filled onto an unchanged criterion.

### Ownership joints

Stated explicitly, because the failure mode here is one fact authored twice.

- **SPEC-014** owns the plan *fileset and scaffold*; this spec owns the plan
  *content model*. `src/slice.rs` and `src/state.rs` are anchored by both:
  SPEC-014 governs the kind, scaffold, lifecycle FSM and phase rollup inside
  them; this spec governs the plan read (`read_plan`), the plan verbs
  (`slice plan` / `phases` / `verify-vt`), and the phase-sheet materialisation
  *from* plan content.
- **SPEC-002 / PRD-013** own executable procedure identity, real command
  execution, and the observed-coverage substrate. A plan criterion **references
  or projects into** that seam; the plan must not grow a second command or
  evidence schema. Note the two senses of "VT" are not the same key today —
  SPEC-002's coverage is requirement-keyed, a plan criterion is phase-keyed —
  and the join is not yet owned by either spec (OQ-4).
- **SPEC-018** — criterion lineage edges are document-local rows inside
  `plan.toml`, **not** corpus relations. Criterion ids are not entity ids, so
  the cross-corpus relation vocabulary does not apply.
- **ADR-009** owns the `plan` lifecycle state and its `gate` conduct default.
  Cited, never restated.
- **SPEC-012 / SPEC-021** — dispatch reads the plan (phase set, order, names)
  and projects runtime status over it. Those consumers are **read-only** over
  authored plan truth; observed progress can never rewrite it (`REQ-447` AC-3).
- **Reference docs.** `glossary.md` and `using-doctrine.md` currently state the
  criterion identity and immutability rules normatively. On this spec landing
  they should cite it, or the corpus keeps two independently authoritative
  accounts of one rule.

## Concerns

- **Runtime and authored tiers must not leak.** The authored plan records no
  status. Phase tracking and sheets are gitignored runtime state, derived from
  the plan and discardable without changing its contract. A progress field
  creeping into `plan.toml` would make the authored tier unreviewable.
- **Order authority is fragile by construction.** Immutable ids plus appended
  edits guarantee non-monotonic id sequences. Every consumer must read array
  order; a single sort-by-id reintroduces silent reordering.
- **The link tables are populated and dropped.** Authored data with no reader is
  worse than absent data: it reads as governed and is not. Every plan authored
  meanwhile adds more.
- **Plan-time and runtime checks must agree.** The plan-time selector check and
  the integrate-time import belt share one predicate deliberately. Two seams
  deciding one rule independently is how they drift apart while both pass their
  own tests.
- **Waivers are the soft spot.** `waived = true` short-circuits before any
  filesystem read. The only guard is that an unreasoned waiver is flagged, and
  it is flagged as a warning, not a failure.
- **A stale doc comment is load-bearing for agents, not just humans.** The
  comment on the plan read model once asserted the link tables were empty
  corpus-wide; a research agent repeated the false claim into a written brief
  before corpus verification caught it (`ISS-321` part 1, corrected).

## Hypotheses

- **The plan content model deserves one component spec, not three.** Criterion
  identity, the VT mandate, and lineage are one contract read by one set of
  consumers; splitting the VT gate into its own spec would make it a governance
  island when it is a *consumer* of the plan model and a *projector* into
  SPEC-002's seam.
- **Criterion lineage stays document-local.** Promoting criterion ids to corpus
  entity ids is not required by any proven consumer, and would drag every plan
  edit through id reservation.
- **Existence-and-shape is the right altitude for a plan-side gate.** The plan
  proves that mandated work was *not omitted*; proving it is *correct* belongs
  to the executable coverage seam. Merging the two would grow the parallel
  command schema this spec forbids.
- **Zero-ceremony for unchanged criteria is what makes lineage adoptable.** If
  every criterion needed an evolution record, plans would carry lineage
  bookkeeping proportional to their size rather than to their churn, and authors
  would route around it.

## Decisions

- **D1 — SPEC-014 keeps the fileset; this spec owns the contents.** Shared file
  anchors are disambiguated in prose on both sides rather than split into
  per-capability anchors the corpus cannot express.
- **D2 — the authored array is the sole phase-order authority.** Never id
  arithmetic, because immutable ids plus appended edits make the id sequence
  non-monotonic by design.
- **D3 — mode stays encoded in the criterion id prefix.** There is no separate
  mode field. The prefix is already the immutable handle, and a second
  representation of one fact could disagree with it.
- **D4 — a meaning change mints a new record; a non-semantic correction edits in
  place.** Records are immutable, active semantics are correctible, and the line
  between them is *meaning*, not *bytes*.
- **D5 — absence of lineage means every authored row is a governing leaf.** The
  migration contract for every plan authored before lineage support: readable,
  governing, no back-filled records.
- **D6 — the verification→exit link is stored on the verification row; the
  inverse is derived.** Three reasons, in order of weight: it mirrors the
  corpus-wide outbound-only-plus-derived-reciprocity rule (`ADR-004`); it keeps
  EN/EX rows at their zero-ceremony `{id, text}` shape; and verification rows
  are the later-authored, more frequently revised side, so the pointer lives on
  the row already being edited rather than forcing edits into settled exit
  criteria. The gate that already iterates verification rows can attribute
  verdicts to obligations in one pass.
- **D7 — only `Fail` halts, and every other outcome stays visible and
  distinct.** `Uncheckable`, `Unattributable`, and `Waived` are neither failures
  nor passes. Collapsing them in either direction is how a gate manufactures
  false green or false red.
- **D8 — no plan-side check may depend on host-language syntax.** No comment or
  string-literal stripping, no build-tool assumption (`POL-002`). The accepted
  cost is that a keyword matched inside a comment satisfies its mandate.
- **D9 — criterion evolution gets no act identity.** Predecessor and successor
  sets plus a disposition carry the full audit history. A per-act identity would
  open a third identity space beside phase and criterion ids with no consumer
  needing it — the same ownership test that rejected seven of eight candidate
  obligation fields (`RFC-029` § 1). *Settles `PRD-001` `OQ-2`.*
- **D10 — the governing set orders by authored position of the governing rows.**
  Never by id, never inherited from a predecessor. A successor orders where it
  is authored; a relocated criterion orders in its destination phase. D2's rule,
  one level down, and it is what keeps the active plan deterministic through
  split, merge, and relocation without weakening immutability.
  *Settles `PRD-001` `OQ-3`.*
- **D11 — additive changes get a compatibility read; only meaning-changing ones
  get an authored migration.** A new defaulting field never triggers a
  migration. A change that would alter the meaning of rows already on disk does,
  and it is deterministic, reviewable before it governs, and failure-safe — a
  failed migration leaves the prior plan intact and names the blocker.
  *Settles `PRD-001` `OQ-4`.*

## Open questions

1. **Where does the criterion↔evidence join live?** SPEC-002 keys observed
   coverage by requirement; a plan criterion is phase-keyed. Neither spec owns
   the join today. (`RFC-029` § 2.)
2. **Lineage storage location** — an opt-in field on the criterion row, or a
   separate lineage block? Directly determines whether the zero-ceremony
   boundary (D4) holds in practice. Deliberately open per `IMP-382`'s boundary.
   Narrower than `PRD-001`'s `OQ-2`, which asked about act identity and is
   settled by D9.
3. **Historical rendering surface** — which read verb dims predecessors, and
   does it overlap the per-phase criteria read surface (`IMP-260`)?
4. **Does evidence survive supersession?** Evidence attaches to the criterion
   revision it evaluated; whether a successor inherits a predecessor's evidence
   is unsettled and defaults to *no*.
5. **VT pattern-set conservation** — checking that a successor's mandate is no
   weaker than its predecessor's. Named here so it has an attachment point (the
   shape check, extended); explicitly not built.
6. **Does the selector-conformance read surface owe the plan anything?**
   `ISS-251` — a selector doctor reporting healthy while an *exit* criterion
   names an undeclared file — is a joint between this spec and the
   selector-conformance surface (`IMP-310`), not folded in here.
