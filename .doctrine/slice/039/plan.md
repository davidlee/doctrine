# Implementation Plan SL-039: Backlog dependency ordering — item edges + cordage adapter

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The RE-LOCKED design (`design.md`, §10 RE-LOCKED 2026-06-11, commit `ad6c4f5`)
is the authority; this plan only sequences it. Four phases follow the data's path
through the system model (§5.1): **schema → pure adapter → impure CLI shell →
invariant + harvest**. The cut is deliberately the seam boundaries the design
already draws — model, adapter, shell — so each phase ends green against a
contiguous slice of the §8 VT-1..10 matrix and nothing strands a half-wired
layer.

The spine of the design is that **cordage composes the order; the adapter does
no sort** (I1). PHASE-02 is therefore the load-bearing phase and is kept *pure
over `OrderInput`* — fully unit-testable without disk — so the order-correctness
proof (the §10 A1 longest-path regression, determinism, the hierarchy) is
established before any CLI plumbing can obscure it.

**Corrective re-execution (2026-06-11).** PHASE-01/02 already shipped — against
the *old* `depends_on`/`before` vocabulary, the forward-pointing soft edge, and
`EdgeAttrs(0,0)` on every edge. The round-3 reconcile (D10) brought the authored
vocabulary into line with PRD-009 FR-010/FR-011: `depends_on`→`needs`,
`before`→`after` (now pointing *backward* at predecessors, both edges flipping
uniformly), the `triggers` rider field, and the genuine `(rank,age,src,dst)`
eviction key (retiring the A4 `(src,dst)`-under-zero stand-in). The design now
leads the code; nothing is broken, the shipped artifacts are simply behind. The
corrective work re-executes PHASE-01/02 with their criteria **text corrected
in-place under the same immutable `PHASE-NN`/`EX-`/`VT-` ids** — immutability
binds the id, not the prose, and the objectives (widen the schema + path dep;
build the pure adapter) are unchanged. This is **more than a rename**: it adds
genuinely new surface — the `AfterEdge`/`Trigger` types, the `--rank` flag, the
real array-index `age`, and true layer-precedence eviction — which must not be
mistaken for a mechanical sed. Appending a corrective PHASE-05 was rejected: it
would fragment one coherent change across two phase ids.

## Sequencing & Rationale

**PHASE-01 — data model + the cordage dep, first.** Two reasons it leads.
(1) The adapter cannot be written until the model carries `needs`/`after`
(and the `triggers` field rides along on the same outbound seam) and the crate
can `use cordage` — all pure widening with no behaviour to prove beyond
round-trip + render, so they make a clean, low-risk opening phase.
(2) Adding the path dependency is the ADR-001 milestone (cordage's first
dependent) and is cheapest to land in isolation, before any consuming code can
confuse a dep-resolution failure with an adapter bug. The dep is *unused* at the
end of this phase; that is safe because the repo's `unused_crate_dependencies`
gate is paused (a deliberate, recorded pause), so an unused path dep does not
break the green bar. `list` stays untouched throughout — its sort contract and
black-box goldens are load-bearing elsewhere (D8).

**PHASE-02 — the pure adapter, the heart.** Everything that makes this slice a
*cordage consumer* lives here: the `ItemId`↔`NodeId` bimap, the two named
overlay handles (`needs_overlay`/`after_overlay`), the single 2-layer `OrderSpec`
`[Along(needs), Along(after)]`, the genuine `EdgeAttrs(rank, age)` eviction on
the soft `after` overlay (`age` = the edge's index in the item's `after` array),
and the fixed
`(exposure desc, created, id)` node allocation that realises the tiers-2–3
fallback. It is isolated as one phase because its correctness is subtle and
independent of I/O: the order is produced by cordage's longest-path `ordered()`,
not a sort the adapter performs (I1), and the design's hardest-won finding — that
exposure must be the **fallback**, never an overlay, or it drags
dependency-incomparable items across levels (§10 A1) — can only be proven by
exercising `ordered()` directly. Keeping the phase disk-free lets every one of
VT-2/3/4/8/10 run as a fast unit test over hand-built `OrderInput` fixtures. The
R-C kill (VT-10) is verified here too, at the surface where the tokens live:
`NodeId`/`OverlayId` must not appear in any `pub(crate)` signature, and the two
overlay handles are **named fields** (not a positional pair) so the adapter
cannot transpose its own handles (E4) — token-hiding alone does not buy that.

**PHASE-03 — the impure CLI shell, last in the build-up.** Only once the adapter
is proven do we wire it to real items. This phase is the thin impure shell of the
pure/imperative split: the two set verbs (edit-in-place, the `set_backlog_status`
precedent) and the `backlog order` view (read non-terminal items → project →
build → print). It is deferred to here because its three behaviours (VT-5/6/7)
all *depend on* the adapter: the dep-cycle refusal reuses `dep_cycles`, the soft
eviction surfaces `overrides`, and the membership rules drop terminal/absent
endpoints the adapter never sees. Two design subtleties get their proof here.
First, the cycle policy split is observable at two layers — `needs` refuses
a closing cycle at *author* time, and `order` is the *backstop* hard error — and
both must name the members. Second, the §5.6 honest-record (OQ-D D-min): a
dropped terminal `needs` must surface the endpoint's status **and**
resolution loudly, because an *abandoned* prereq (`wont-do`/`obsolete`/…) floats
the dependent unblocked and the author — not the tool — judges the staleness. The
adapter reports drops by `ItemId` + reason; enriching those with the endpoint's
resolution from the corpus is the render's job, which is why it sits in this
shell phase and not the corpus-blind adapter. (Where exactly the `OrderInput`
projection + `exposure()` live — OQ-A — is settled at `/phase-plan`/execute; the
design's lean is `backlog.rs`, keeping `BacklogItem` private.)

**PHASE-04 — the leaf invariant + the budgeted harvest.** Last because it can
only be judged after real first-consumer use. Two closures: prove cordage stayed
a pure leaf (no `crates/cordage/**` diff, `cargo tree -p cordage` alone — I4),
and record the one R-C interface finding cordage's Lock reserved (objective 5,
OQ-B) — the concrete API bend real use demanded, *or* an explicit null result so
the reserved budget is closed rather than left dangling. The bend is **not**
patched in this slice (leaf invariant); a warranted one becomes a small
non-breaking cordage follow-up slice.

## Notes

- **Boundary discipline.** No phase reopens the locked design; `/plan` sequences,
  it does not re-decide. Settled calls (PRD-009 vocabulary `needs`/`after`/
  `triggers` per D10; exposure = fallback not overlay; `after` Evict / `needs`
  Reject; `after` per-edge `rank` default 0, array-index `age`; OQ-C = single-`to`
  per invocation; OQ-D = D-min; no new ADR) are inputs, not options.
- **Lint as you go.** `just check` green before every commit; `cargo clippy -p
  doctrine` / `-p cordage`, **never** `--all-targets` (it lights up the
  unwrap/expect denials in test code). Cordage-side bans (BTree not Hash,
  `.get(range)` not index, `try_from` not `as`, `#[expect(reason=…)]` not bare
  `allow`) apply to the adapter.
- **Commit scope** `plan(SL-039)` for this stage; `feat(SL-039)` once code
  starts. Leave `AGENTS.md` and `.#*` lockfiles untouched/unstaged.
