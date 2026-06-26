# RFC-008 notes — bootstrap for the next deliberation pass

Operational scaffolding for an agent picking up RFC-008. The **deliberation** lives
in `rfc-008.md`; this file is the *reading order, the state-of-the-world, the engine
facts, and the pitfalls* so you can engage without rediscovering them. Keep it
current as passes land.

## ✅ RESOLVED → ADR-017 (2026-06-26)

RFC-008 is **resolved**. Gating = **inbound `needs` on an unsettled record**; the sole
engine delta is the **trinary partition** (unsettled record → non-`Terminal` `Gating`
class). No new relation/role/axis. See `rfc-008.md` Outcome + **ADR-017**. SL-158
unparks against the shrunk scope. The notes below are retained for the implementer.

> **Correction (was wrong).** An earlier version of this file claimed the population
> was EMPTY. It was stale: as of 2026-06-26 there are **4 self-dogfooding seed records**
> (QUE-001, CON-001, ASM-001, DEC-001) and **11 `shapes` edges** — including
> `QUE-001(open) ──shapes──▷ SL-158`, which *is* scenario S1 live (SL-158 was parked
> pending QUE-001). The decision was design-led regardless; the seeds suffice to
> validate the gate's bite once the partition lands.

**The ws1↔ws3 sequencing (settled).** ws1 (the trinary partition) lands first as the
target; the `/knowledge` authoring skill (ws3 — **IMP-182**) follows close behind to
validate against a real population. The mechanism is not blocked on the full skill.

## Reading order (annotated — why each matters)

1. **`rfc-008.md`** — the option space (M0/M-P/M-Pk/M-Pr/M-E/M-L), the shorthand, the
   4 scenarios, the decisions D-a..D-e. Start here.
2. **`doctrine rfc show RFC-003`** — the decisive reframing. Read *all* of it, esp:
   **Layer 1** (structure ≠ graph-effect; gating is consumer policy), the **design
   law** (derivable-not-relational), and the **deferred "influences / planes" open
   question** + "does `related` want its own symmetric label" — both bear on whether
   `shapes` should carry a *role* (M-Pr) and on plane/valence modelling.
3. **`doctrine rfc show RFC-007`** — program context; the **ws1↔ws3 coupling** above;
   note RFC-007 repeats IMP-047's "new `RelationLabel`" wording — that is *inherited*,
   not a ruling; RFC-008 exists to reconcile it against RFC-003.
4. **`doctrine backlog show IMP-047`** — the source item; the four gating pairs spelled
   out; the "new RelationLabel + RELATION_RULES rows" sketch RFC-008 revisits.
5. **`doctrine spec show SPEC-019`** (or `.doctrine/spec/tech/019/spec-019.md`) — D7,
   NF-003, OQ-2, and the **"Priority-engine posture: never actionable, meant to gate"**
   section (the consumer's own account). PRD-010 §3 ("a record shows what it affects")
   + §6 (spawned-work path) define the association the gate would ride.
6. **`.doctrine/slice/158/notes.md`** — the engine-seam findings (don't re-derive them).
7. **The seam**: `src/priority/{partition,channels,graph}.rs`, `src/relation.rs`
   (`RELATION_RULES`, the `Shapes` row), `src/dep_seq.rs`, `src/relation_graph.rs`
   (`dep_seq_for`).

## Engine facts that pin the options to reality (verified this session)

- **`channels.rs` needs ~no change** under any option. `eligible == Workable` (records
  never workable → excluded from `next`); `blocked_by` keeps dep-predecessors with
  `class != Terminal` (a would-gate node blocks; a settled one stops). **Settle→unblock
  falls out free.** The dispute is purely *which edges land on the dep overlay*.
- **`partition.rs`** — `StatusClass{Workable,Terminal,Unrecognised}`; per-kind
  `workable ∪ terminal == <KIND>_STATUSES` drift canary. Trinary generalises it to
  `workable ∪ gating ∪ terminal == vocab`. Per-kind settle boundaries (unsettled →
  would-gate; settled → terminal): ASM `held`/`testing`; DEC `proposed`; QUE `open`;
  CON `active`. Existing test `every_knowledge_status_classifies_terminal_never_workable`
  flips for the unsettled states (expected — consumer revision, not a regression).
- **`needs`/`after` are NOT `RelationLabel`s.** They are typed `[relationships].{needs,
  after}` axes read by `dep_seq::read` / `relation_graph::dep_seq_for` (kind-neutral
  leaf; short-circuits non-authoring kinds with no disk read). **M-E** adds a `gates`
  sibling axis *here* — coordinate with IMP-033, don't fork.
- **`Shapes` rule** (`src/relation.rs`): `sources: RECORD`, `target:
  Kinds(PRD,SPEC,REQ,SL,ISS,IMP,CHR,RSK,IDE,ADR,POL,STD,ASM,DEC,QUE,CON)`, `Tier::One`,
  `Writable`. Currently **graph-inert** — not in `REF_LABELS` (consequence overlays) and
  not in dep/seq. **M-P/M-Pr** project over this row; its wide target set is the source
  of the S2/S3/S4 over-broad worries.
- **Dep overlay is `CyclePolicy::Reject`** → the S4 record↔record cycle hazard is real;
  `dep_cycles` degrades, never panics, but the gate would be lost in the cycle.
- **`Shapes` is in neither `REF_LABELS` nor `CONSEQUENCE_LABELS`** → projecting it to the
  **dep overlay only** leaves scoring/consequence untouched (a clean, contained change).

## Census how-to (for when records exist, or to re-confirm empty)

```
doctrine relation census                       # by-label tally (resolved/unresolved/free_text)
doctrine relation list --label shapes          # the shapes population (today: empty)
doctrine relation list --label shapes --source-kind QUE   # per record-kind
```
Classify each `shapes` edge by **intended** effect: gates-the-dependent vs
informs-only. The ratio is the evidence for M-Pr (need per-edge intent) vs a blanket
rule. Today this returns nothing — re-run once ws3 authoring seeds records.

## Decision targets (detail in `rfc-008.md` Outcome)

D-a coupling/locus (**M-Pr vs M-E** — the core fork; M-P/M-Pk killed by S2, M-L by
RFC-003) · D-b transitivity (direct-target vs propagate-along-references, S3) · D-c
direction (outbound-from-record, ADR-004) · D-d record↔record & cycles (S4) · D-e name.
Convergence on D-a/b/c → **emit an ADR** (gating layer + edge model) → **unpark SL-158**.

## Overlaps to fold in

- **IMP-033** — cross-kind dep/seq capture; shares the dep-overlay machinery (M-E's axis).
- **IMP-053** — record↔record associative class; bears on S4 (intra-family) and on
  whether `shapes` wants a role.
- **RFC-003 deferred** — the `influences`/planes axis and `related`'s symmetry question:
  if `shapes` gains a `{gates,informs}` role (M-Pr), check it against RFC-003's
  plane/valence framing rather than minting in isolation.

## Pitfalls / ops

- **Shared-index commit hazard** — multiple agents commit into one index; a bare
  `git commit` sweeps others' staged files. **Path-limit the commit itself**
  (`git commit -F - -- <paths>`), not just `git add`. (RFC-008's own first commit was
  swept into an unrelated agent's commit `a38b3a07` — content intact, message conflated.)
- **Jail reservation** — id allocation needs `DOCTRINE_RESERVATION_FALLBACK=1`
  (remote reach is disabled; it degrades to local, which is fine).
- **Read entities via `doctrine <kind> show`**, not raw TOML/MD — `show` synthesises
  both tiers.
- **Recency trap** — RFC-007 (newer) repeating IMP-047's "new RelationLabel" wording is
  *not* a ruling that settles the layer; RFC-003's Layer-1 argument outranks the
  inherited phrasing. RFC-008 is the reconciliation.
