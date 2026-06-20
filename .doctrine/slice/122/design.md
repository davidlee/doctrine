# SL-122 Design — RFC kind: first-class discussion artifact

Scope: `slice-122.md`. This doc is canon for design intent. Sections lock
incrementally; unresolved questions are named explicitly, not buried.

## §1 Relation model (OQ-2, OQ-3 — LOCKED)

### Grounding

The relation engine is **label-first, not kind-first**. The whole vocabulary is
one flat const table, `RELATION_RULES` (`src/relation.rs:256`); each row is a
5-axis tuple:

```
RelationRule { sources: &[kind_prefix], label, inbound_name, target: TargetSpec, tier, link }
```

A *label* is kind-agnostic; a *rule* binds a label to a set of authoring kinds +
a target constraint. Same label may recur across rows with different source sets
/ target specs (e.g. `related`: governance→`SameKind`, work-items→`AnyNumbered`).
Validation is `lookup(source_kind, label)` → row, then `check_target_kind`
against the row's `TargetSpec` (`relation.rs:897`). Edge logic never lives inside
a Kind — ADR-010's "unify the contract, keep storage bespoke."

"Link to anything" is therefore **additive in the relation layer — no engine
fork** (retires R2): the *relation validation + overlay allocation* take RFC with
no special case (proven below). This is narrower than "a new kind is one const
edit" — kind registration has other hand-maintained surfaces (see "Beyond the
relation layer" below). Three target modes already exist:
- `TargetSpec::AnyNumbered` — link to any *entity* that exists (`related`,
  `reviews`). Full graph citizen: overlay allocated, reciprocity derived.
- `TargetSpec::Unvalidated` — free-text, no kind check, **no overlay, permanently
  dangling**, preserved for visibility only (`drift`, `decision_ref` — the "DEC
  pattern").
- `TargetSpec::Kinds(&[…])` — closed target set (e.g. `governed_by` → ADR/POL/STD).

### Decision 1 — RFC's own outbound edges (OQ-2)

RFC's general "cite whatever is worth citing" need is served by **`AnyNumbered`**,
not free-text. RFC links to any *entity*; those links are real graph edges
(navigable, reciprocal, rendered by `inspect`). Free-text `Unvalidated` is **not**
adopted as RFC's primary edge — dangling refs that never resolve or reciprocate
defeat the point of a citable discussion artifact. (Borrow `Unvalidated` later
only if a concrete non-entity-citation need appears; out of scope until then.)

RFC-as-source slots in with **no engine special case** — confirmed against both
gates:
- `check_target_kind` (`relation.rs:916`): `AnyNumbered` is a bare no-op arm —
  accepts any target prefix, never reads `source_kind`. RFC→RFC same-kind and
  RFC→other both pass.
- overlay allocation (`relation_graph.rs:162`): table-derived, one overlay per
  distinct non-`Unvalidated` label, auto-allocated; `CyclePolicy::Reject,
  Arity::Unbounded`. A new resolvable label joins the graph for free.

Relation mechanics: add `RFC` to the `related` rule's `sources` set (the
`AnyNumbered` row). No new label required for RFC's own context edges.

**RFC↔RFC self-reference** rides the same `AnyNumbered` row — no `SameKind`
collision (the governance `related`/`SameKind` row is a *different* rule; `lookup`
selects by source prefix, so an RFC source resolves to the `AnyNumbered` row).
`CyclePolicy::Reject` is a non-issue: reciprocity is *derived*, so a "see also"
is authored **one direction only** and the reverse view comes free — you never
author both directions, so no cycle to reject.

**Beyond the relation layer (F1 — kind registration is not "one const edit").**
A new kind also touches hand-maintained surfaces that are *not* free:
- `integrity::KINDS` — add the RFC row (`integrity.rs`, hand table).
- `catalog::scan::outbound_for` (`catalog/scan.rs:39`) — explicit prefix dispatch;
  an unrouted kind degrades to empty outbound edges in release (debug-asserts).
- `src/rfc.rs` — **bespoke** verb module (`new`/`show`/`list`/`status`), mirroring
  `adr.rs`/`rec.rs`. `list`/`show` are per-kind, not inherited (cf.
  `rec::list_rows`, `concept_map::run_show`).
So "RFC's own edges are an additive const edit" is true; "the whole kind is" is
not. Scope's Affected Surface reflects this.

### Decision 2 — the RFC↔REV precursor edge (OQ-3)

Two hard constraints collapse the option space:
1. **Temporal order** — RFC is born first; REV is born later (when a proposal is
   enacted).
2. **ADR-004 outbound-only** — the edge author needs the target to exist *at
   write time*.

Only **REV → RFC** satisfies both without a §5 carve-out: the RFC exists when the
REV is created, REV authors one outbound edge, RFC's inbound derives. This also
matches who REV already is — the active, later, change-axis edge-author (it
already carries `revises`). The mirror (RFC→REV) would need the ADR-004 §5
carve-out *and* an RFC lifecycle flip to justify rewriting the RFC file — more
machinery, no gain.

`supersedes` is the exact structural template (active/later entity holds outbound;
passive/earlier renders derived inbound). New rule row:

| axis | value |
|---|---|
| `sources` | `&[REV]` |
| `label` | `originates_from` — reads "REV-005 originates from RFC-003" |
| `target` | `TargetSpec::Kinds(&[RFC])` — a REV originates from RFCs only |
| `inbound_name` | `"precursor of"` — `inspect RFC-003` shows "precursor of: REV-005" |
| `tier` | `Tier::Typed` |
| `link` | `LinkPolicy::TypedVerbOnly` — **not** generic `doctrine link` |

Arity unbounded both ways (a REV may draw on several RFCs; an RFC may precede
several REVs).

**Revision-owned, not generic-link (F2).** The edge is authored only through a
revision verb — a creation-time flag `doctrine revision new --originates-from
RFC-NNN` (and/or a `revision originates-from <rev> <rfc>` verb), mirroring how
`revises` is `TypedVerbOnly` and authored via `revision change add`
(`relation.rs:429`, `revision.rs:779`). Making it `Tier::One`/`Writable` would let
generic `doctrine link` append a REV outbound edge with only source/label/target
checks (`relation.rs:858`), bypassing REV-local discipline — the exact "don't let
`link` write to a REV" smell. It is **lighter than `revises`**: a provenance ref,
no `[[change]]` payload — so `TypedVerbOnly` here costs a small flag/verb, not the
change-row machinery. (The earlier "tier-1 is cheapest" rested on generic `link`
being free; that "free" *is* the discipline violation, so the cheapest *correct*
option is the revision-owned flag.)

**Verb is outcome-neutral by design.** The edge means "this revision *arose from*
this discussion" — true whether the RFC concluded yes or no. An earlier candidate
`enacts` / "enacted by" was rejected: it smuggles a yes-outcome into the label,
so an RFC that resolved *against* a proposal (and was formalised by a REV
recording the "no") would falsely render "enacted by". That is the same
governance-position leak banned from `status` (see §2), sneaking in via the
relation. `originates_from` / "precursor of" stays honest under rejection and
matches the scope's own word ("the precursor discussion a REV may later
formalise"). Outcome (yes/no) lives in the RFC **body prose** and in what the REV
records — never structurally asserted by the edge. Not `spawns` (taken:
record→backlog).

**Tier-1 over tier-2**: `enacts` is a bare citation with *no payload*. `revises`
is `TypedVerbOnly` only because its `[[change]]` rows carry payload; `enacts`
carries none, so typing it buys a bespoke verb for nothing. Tier-1 writable via
generic `link`.

Consequences:
- RFC needs **no reverse-edge storage**; "enacted by" is pure derived render
  (ADR-004 clean).
- The precursor link is **not** authored by RFC → it puts RFC in no `sources`
  set. Independent of Decision 1's `related`/`AnyNumbered` edge set.

## §2 Lifecycle (OQ-4 — LOCKED)

**Minimal status machine**, governance-neutral terminals:

```
open (default) → resolved | withdrawn
```

- `open` — deliberation live.
- `resolved` — discussion concluded. **Outcome-blind**: means "concluded," not
  "concluded yes". A discussion may resolve *against*; the yes/no outcome lives in
  the body prose, never in the status.
- `withdrawn` — abandoned without conclusion.

Rejected alternatives:
- **Stateless** (REC-path, `meta::read_id`): leanest, but no edge expresses
  *withdrawn* or *resolved-without-a-REV*; those are pure status facts. The
  live/concluded/dropped distinction is real catalog signal no relation captures.
- **`accepted` / `declined` terminals**: forbidden — they make the RFC assert a
  governance position. `resolved` stays outcome-blind precisely to avoid this.
- **`draft` / `superseded`**: speculative; omit. Supersession, if ever needed, is
  an edge not a state.

Reader path: an authored `status` field ⇒ RFC does **not** use REC's status-less
`meta::read_id` scan; it follows the status-bearing read path (cf. ADR/slice).
Transition verb `doctrine rfc status <id> <state>` (status moves via CLI, not
hand-edit — boot rule).

**Status set + list visibility (F4).** `RFC_STATUSES = {open, resolved,
withdrawn}` (known-status set, mirrors the per-kind status sets at
`governance.rs:66`, `concept_map.rs:28`, `revision.rs:50`). `rfc list` default:
**show `open` only**; `resolved`/`withdrawn` are hidden unless `--status` names
them (the live-set-by-default idiom, same as governance kinds hiding
deprecated/retired). `rfc list --status all` (or explicit values) reveals
terminals. `--status` validates against `RFC_STATUSES`.

**Status ⊥ `originates_from`.** A REV-authored edge never flips RFC status (that
would reintroduce the target-mutation avoided in §1 Decision 2). An RFC may be
`resolved` then later preceded by a REV, or precede a REV while still `open`. The
two axes are independent.

## §3 Naming (OQ-1 — LOCKED)

`.doctrine/rfc/` — **singular**. The dir is a free per-kind `const X_DIR: &str`
(`adr.rs:24`), but every authored tree is singular (`adr`, `rec`, `slice`, `spec`,
`policy`, `standard`, `requirement`, `revision`, `review`) — zero plurals, even
the acronym peer `adr`. `RFC_DIR = ".doctrine/rfc"`, prefix `RFC-NNN`.

## §4 Visibility (OQ-6 — LOCKED)

Three surfaces, three distinct treatments:

1. **Boot snapshot governance sections — OMIT.** `boot_sequence()` (`boot.rs:90`)
   is a fixed list; its only governance surfaces are `SourceKind::GovRows` over
   `ADR/POLICY/STANDARD_KIND`. RFC is simply not added there. **Soft rationale,
   not a hard never:** RFCs *are* situational awareness (live deliberations a
   fresh agent might want), so the argument to surface them is real — but a line
   in a governance-flavoured snapshot risks being **misconstrued as canon**. We
   omit *on that basis*, and may revisit later behind an explicit disclaimer
   ("deliberation, asserts nothing") if the awareness value proves worth it. The
   design constraint is "not in the GovRows sections," not "invisible forever."

2. **`doctrine status` — SURFACE (in-scope).** This is the work/awareness
   dashboard (slice + backlog counts, next-up, blocked) — no governance flavour,
   so no misconstrual risk. The chosen situational-awareness home, distinct from
   the boot snapshot. Two parts:
   - a count line under `Work` (`rfcs: N open, M total`); and
   - a short **list of unresolved RFC titles** — `status = open` only, ordered
     **most-recent first**, capped at **10** with a `+K more` overflow marker when
     exceeded (idiom matches the existing `Blocked backlog` / `next up` blocks:
     `RFC-007  <title>`). Resolved/withdrawn RFCs drop off the list (count still
     reflects them). Cap 10 over 20 — `status` is a glance surface; 20 open
     deliberations is itself a signal better read via `rfc list`. The cap is a
     single const, trivially bumped if deliberation volume argues for it.
   - **Serialized envelope + empty-state (F4):** `status` has a data model, not
     just a render (`status.rs:22`). RFC count + titles extend the JSON envelope
     too. Open RFCs **do not** flip empty-state: "No active work" (`status.rs:112`)
     stays keyed on slices + open backlog only — deliberation is knowledge, not
     tracked work, so a repo with only open RFCs is still "no active work" (RFCs
     render as an awareness adjunct, not a work driver).

3. **`doctrine rfc list` — catalog.** The on-demand full catalog (bespoke
   `rfc.rs` list, per F1 — not "free", but per-kind like `rec list`).

**Authored-tree wiring (F5 — the silent-uncommittable trap, pinned).** `.doctrine/*`
is gitignored with per-tree re-includes; without the negation the new kind is
silently swallowed even when install creates the dir. Exact edits:
- `install/manifest.toml` — add `.doctrine/rfc` to `[dirs].create`.
- `.gitignore` — add `!.doctrine/rfc/` to the authored-tree negation block.

## §5 Governing ADR (OQ-5 — LOCKED)

One ADR, scoped to the two **contestable** claims (precedent: ADR-007 D-C0,
ADR-013):

- **D1 — RFC is a governance-neutral, first-class authored kind.** Deliberation
  gets a durable, citable home that asserts **no** canon, sources no ADR/POL/STD,
  and is structurally absent from governance surfaces. The novel,
  precedent-setting claim — the kind taxonomy gains a "think in public, decide
  nothing" slot.
- **D2 — RFC's position on the change axis (amends ADR-013).** RFC is the
  *precursor* to a Revision; the REV→RFC `originates_from` edge is
  **outcome-neutral** (no yes-bias, no status coupling) and **revision-owned**
  (TypedVerbOnly, §1 Decision 2). D2 adds a new REV outbound edge to ADR-013's
  change-axis model, so the new ADR **explicitly amends ADR-013** on the REV
  precursor interface (F6) — not a silent fold-in. The ADR carries a `supersedes`
  or amendment relation to ADR-013 and states the REV-edge addition in those
  terms.

Stays design-level (not ADR): lifecycle states (§2), dir naming (§3),
`AnyNumbered` participation for RFC's own edges (§1, rides ADR-004/010).
