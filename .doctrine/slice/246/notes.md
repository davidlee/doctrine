# Notes SL-246: Entity reads carry their knowledge records

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage (2026-08-05, `exploring`)

Discharges `explore.triage`. The scope doc (`slice-246.md`) already carries the
open questions, risks, assumptions and governing context; this records only what
the exploration pass *added* to them.

### Prior art the design must ride, not re-invent

- **An entity `show` already renders a derived inbound axis.** `backlog show`
  surfaces `fulfilled by` via `derive_fulfils_inbound` (`src/backlog.rs:1368`,
  SL-176 PHASE-03) — full corpus scan → `build_relation_graph_from` → `in_edges`
  on a single label overlay → sorted canonical refs. SL-246's composed read is
  that shape generalised over a label *set*. This is the strongest DRY seam in
  the slice and it was not in the research artefact.
- **Cost note on that seam.** It scans the whole corpus per `show` invocation.
  Acceptable for one label on one item; the composed read must not multiply it.
- **`Detail::{Normal, Full}`** (`src/design_run/render/envelope.rs:86-117`) is
  the verbosity precedent — levels as caps on one code path. Already in the
  scope doc's design inputs.

### Constraints the exploration surfaced

- **`CatalogEntity` deliberately does *not* carry facets.** `estimate`/`value`
  were **deleted** from it at SL-222 PHASE-09, and `src/catalog/scan.rs:270`
  now runs a key-presence tripwire against their residue
  (`src/catalog/hydrate.rs:107`). `OQ-6`'s "grow `CatalogEntity` a `RecordFacet`"
  arm therefore reverses a decision this project made deliberately and left a
  guard behind. That is not fatal, but the arm must argue against SL-222 rather
  than treat the field as merely absent.
- **`facet` is already taken.** `src/facet.rs` `EntityFacets { risk, tags }` is a
  shared projection over *authored* entity facets, consumed by `format_show` and
  the priority graph — a different thing from `knowledge::RecordFacet`. Any new
  type or flag naming must not collide (STD-002; naming matters more than usual
  here).
- **Per-record-kind field lists are a known scatter hazard.** `kinds::RECORD` is
  the canonical const but ~17 sites hardcode the prefix literals
  (`mem.pattern.doctrine.record-kind-touch-sites`, high severity). `OQ-2`'s
  answer is exactly such a list — under STD-001 it must be one named constant
  keyed off `kinds::RECORD`, not match arms at the render site.

### Carried assumptions

- **A2** — the `facets` level's per-kind field lists are a *display* concern,
  not governance. `SPEC-019` declares no tiering, so SL-246 is the authority of
  first impression (research thread 1, ✓). If that is wrong the whole dial needs
  a `REV` first.
- **A3** — the `SPEC-019` four-of-seven gap (`ISS-316`) stays *outside* this
  slice. The design labels any `EVD`/`HYP`/`CPT` field list as invention rather
  than deriving it.

## Parked 2026-08-05 — design run at `inquiring`

Run `dr-019fd1ab-39d9-7370-ab6c-9e39fc8ac2bc`, revision 19. Re-enter with
`doctrine design resume 246`.

**Where it stands.** The `exploring` runbook is discharged and its gate cleared
(`governance-confirmed`, `blocking-set-declared` over `inq-1`..`inq-6`,
`graph-reviewed`). All seven inquiry nodes are resolved. Nothing is outstanding
at `inquiring`; the next move is either further inquiry or the gate to
`drafting`.

**The durable output is the seven decisions, not the run.** The run snapshot
lives at `.doctrine/state/slice/246/design.toml` — runtime tier, gitignored,
disposable. If it is lost the decisions survive; the map does not.

| node | question | record |
|---|---|---|
| `inq-1` | surface | `DEC-145` — rides `doctrine inspect` |
| `inq-2` | composition seam | `DEC-146` — per-id read, not scan-carried |
| `inq-3` | closure seam for `IMP-398` S5 | `DEC-147` — selection split from render; caption is text |
| `inq-4` | record selection | `DEC-148` — filter on source kind, not label |
| `inq-5` | empty facet | `DEC-149` — marked, never papered over |
| `inq-6` | field selection | `DEC-150` — what rules, not the argument |
| `inq-7` | test strategy | `DEC-151` — synthetic goldens + agent attestation |

**Raised and deliberately not made nodes** — both settled in drafting, neither
turning out to be a node: the flag and level naming landed as `--knowledge
skip|facets|full` (design §5.2), and `DEC-145`'s pointer line left the slice
for `IMP-398` (design §7.2 `D4`).

**Spun out of this stage.** `IMP-403` (knowledge facets systematically unfilled —
carries the TOML-only analysis). `ISS-316` (`SPEC-019` governs four record kinds,
the corpus has seven) predates the stage and stays open.

## Review passes: where the ledger stands (2026-09-18, `reviewing`)

Discharged `review.passes` at run revision 51; **updated in place** after round 5
ran, so it describes the next pass rather than the one that happened.

**Round 4 ran** on an Opus raiser in the primary worktree, against revision 47 —
a different model from rounds 1-3, deliberately, for variety on a late pass. All
five of round 3's dispositions verified with their claims re-derived from the
tree. Five raised: `F-23` (major), `F-24`, `F-25`, `F-26`, `F-27`. Integrated at
revision 48, materialised at 49.

**Round 5 ran** on the same raiser, against revision 49, on an adjudication-only
bar. All five verified — `F-23`'s repair tested kind by kind rather than accepted
on the disposition's account. Two raised: `F-28` (minor), `F-29` (nit).
Integrated at revision 50, materialised at 51. Its judgement: **lock it.**

**`F-23` was the structural one and is worth carrying forward.** Round 3's repair
put a `Silent | Marked` policy on the two leaf facet renderers. That parameter
had no route to `Full`: the entry there is `show_value`, and the text chain is
`format_show` → `format_metadata`, none of which carries policy — so the
by-design marker `X5` requires at *every* level was reachable at `Facets` and
unreachable at `Full`, on both arms. The repair withdraws `EmptyPolicy` outright
rather than threading it up: all three markers compose in the per-record
producers, and `facet_fields`' return shape decides which (`[]` before filtering
is by-design, all-`Absent` is unfilled). `C2` becomes structural — `knowledge
show` never enters a layer that can mark. `DEC-149`'s ruling stands; its siting
clause moved, as `DEC-150`'s encoding clause already had.

**Two claims, and they must not be run together.** The raiser made this
distinction at round 7 and it is right:

- *Is a further adversarial round warranted?* **No** — and the raiser agrees,
  independently. Yields 8, 9, 4, 5, 2, 1; and more tellingly, the rounds at
  revisions 51, 53 and 57 introduced no signature, no decision and no `DEC`.
  There is no remaining surface where either of us expects a structural defect.
  A further pass would be fishing.
- *Is the ledger clean?* **That is a different sentence with a different truth
  value**, and it has been false twice after being written. It is true only when
  `doctrine review status RV-370` says every finding is terminal. Check it; do
  not infer it from the paragraph above.

**The sixth pass was not warranted either, on the reasoning below, and it still
produced `F-30` and `F-31`.** That is not an argument for a seventh — both were
nits-to-minors that changed no decision — but it is the reason the two claims
are separated here.

**Why the repair ground fails, which is the first round it has:** The two grounds that carried rounds 3, 4 and 5 separate here:

1. **The ledger ground still holds** — `F-28` and `F-29` are `answered` and
   `RV-370` is `await=raiser`. The `review-disposition-attested` contract will
   not take a pass while they stand. But that is an **adjudication turn**, which
   is the mechanism closing; it is not a pass.
2. **The repair ground does not.** Every prior round's repair was new design —
   round 3's introduced five signatures and two decisions, round 4's withdrew a
   type and rewrote a layering rule. Round 5's repairs are one clause keying a
   table row on the unfiltered table (`F-29`) and one sentence in the scope
   (`F-28`). No signature, no decision, no `DEC` touched. There is no new
   material for a sixth reader to be the first to read.

**If the adjudication contests either, that judgement is void** and a sixth pass
is back on. Two points were put to the raiser explicitly rather than left to
politeness: whether an `EVD` at `Only(Argument)` falling to the *unfilled* marker
is the same "sited where its inputs are not" mistake in new clothes, and whether
the text arm's `Full` marker route should name its mechanism (`format_metadata`
returns `Vec<String>` whose facet block is its own element) rather than leave it
to be rediscovered. Both were judged acceptable; neither was judged obvious.

**Trend, stated without leaning on it.** Rounds raised 8, 9, 4, 5, 2, 1. The count
never fell cleanly, but the *severity* did — blocker+major, major, major, one
major, then a minor and a nit — and round 5's clean-on-inspection list was long
and specific (every `file:line` in `design.md`, the `STD-001` template question,
the layering rule against the tree, `D7`'s enforceability, `C2` under the policy
parameters). Surface shrinking, tally flat.

**The review's most reusable result — a repair inherits the finding's scope.**
Four times, a repair satisfied the arm the finding named and left its twin:
`F-13`/`F-18` (the JSON arm), `F-23` (the `Full` level), `F-30` (the text arm),
`F-31` (the block level). None was carelessness — every one was verified against
the tree before its disposition was written. The finding frames the scope, and a
repair that answers the finding reads as finished. The counter costs one
question per disposition: **which arm did the finding name, and what is its
twin?** Recorded as `mem.pattern.review.repair-inherits-finding-scope`, with
`mem.pattern.review.repair-closes-a-subset-of-the-stated-class` as its sibling —
that one's class is enumerated in the artefact, this one's is implicit in the
claim's quantification, so there is no sentence to re-read.

Second-order, and the reason this slice stopped: once a review has named the
class, fixing the next instance is not enough. `F-31` was found by a reviewer
taking a disposition's own sentence at its word. `X5` and `I6` were then found
by **sweeping** the class at revision 59 rather than waiting for a seventh round
— and `X2`, `X3`, `X4`, `X6`, `X7`, `I1`-`I5`, `I7` checked and found
arm-independent or already qualified. That sweep is the reason to believe the
class is closed. Nothing else is.

**The earlier framing, kept because it is the weaker claim.** Rounds 3, 4 and 5 each found their most structural
defect *in the previous round's repair*, not in the original draft: `F-23` was
`F-20` displaced one layer up; `F-29` was `F-23`'s own test sited one step too
late; `F-25` caught diagrams a self-attack pass had rewritten and left a state
short. A repair round is where the next round's findings come from, and reading
one's own repair is not the same check as a reader taking the model from it.

**Not a substitute for the attestations.** Nine remain outstanding and the policy
is `human-only`. An adversarial round cannot discharge them, and each binds a
revision — round 5 read 49, so it could not attest 51 either.

**Reviewer.** codex remains out of credits; rounds 2-5 all ran on Opus.

## Execution harvest (2026-09-20, `/audit`) — lifted from the phase sheets

The six runtime phase sheets under `.doctrine/state/slice/246/phases/` are
4,457 lines and `rm -rf`-able by contract. What follows is what must outlive
them. Full detail stays in `RV-372`.

### The findings the sheets raised that the plan could not have

- **`PHASE-01` `F-1` → `PHASE-03` `EX-2`.** `design.md` § 5.2 decided the
  *unfilled* marker by "every surviving field is `Absent`". Facet list rows are
  bare `Vec<String>`, never `Option`, and every shipped template seeds them
  empty — so an empty list is `List([])` and can never be `Absent`. Under the
  authored predicate an unfilled `CON` at **both** levels and an unfilled `DEC`
  at `full` would name a record and then silently explain nothing on the text
  arm. Two of seven record kinds, in their default scaffolded state. Caught at
  *planning*, deliberately not built early (an unused helper is a hard
  `dead_code` failure under `warnings = "deny"`), and landed as `EX-2`'s
  blankness amendment with mandatory coverage.
- **`PHASE-03` `F-a` → `EX-4`.** `render_block(root, selected, level)` could not
  produce the empty-set line `D3`/`X1` specify, because that line **names the
  subject** and the authored three-argument signature cannot see it. The
  criterion as written could not satisfy its own required output. Adapted
  (arity only) rather than escalated, because the design specifies the line and
  there is exactly one way to produce it.
- **`PHASE-03` `F-b` → `CHR-074`, and `RV-372` `F-3`.** `C7` (a facet-only read
  at `Facets`) is a design mandate with **no** plan criterion —
  `grep -n C7 plan.toml` returns nothing. Filed rather than built. The audit's
  addition: the type's doc comment claimed the undelivered property, which is
  repaired; the mandate itself goes to reconcile.
- **`PHASE-06`'s traced deletion chain.** `scaffold_design_doc` was not a leaf.
  `run_deprecated_slice_design` → `scaffold_design_doc` →
  `entity::materialise(DESIGN_KIND, design_scaffold)` → `render_design`, plus
  `DESIGN_DEPRECATION_NOTICE`. **None of those four** appears in `EX-1`/`EX-2`
  or in `VA-1`'s grep list, and three dedicated unit tests exercise only that
  dead chain. They are not scope creep — under this repo's `unused = deny` they
  are what `EX-5`'s green gate *requires* once `EX-1` executes, and `cargo test`
  alone will not reveal them (a `cfg(test)`-referenced item reads as live).

### The reusable lessons

- **A deletion phase needs a call graph, not a grep.** `PHASE-06`'s own planner
  note says it: the phase was planned by a smaller model on the argument that
  the compiler enumerates the work once the variant is deleted. That held for
  execution and failed for *planning* — re-deriving the inventory by hand found
  a dependency chain the plan and the prior brief both missed. Related:
  `mem_01a00d11d24d70a1bf531fe6561c426b` (a design's call-site census ages).
- **A placement mandate justified by a layering argument must be checked against
  the layering gate before implementation** (`DEC-274`'s own third consequence).
  Neither design, plan nor phase sheet consulted `layering.toml`, so `EX-2`'s
  premise — that `relation_graph → knowledge` was a tier-crossing edge — went
  unfalsified until the end-of-phase gate reported `TangleGrew { 76 → 77 }`.
  Both modules are tier `command`, and a reverse path already existed.
- **A `VA` grep row should say what it means, not what is easy to write.**
  `PHASE-01` `VA-1` claims "exactly ONE per-kind facet field enumeration
  survives in `src/`". Two do; the second is `#[cfg(test)]` and predates the
  slice. The row's *intent* — one enumeration on the render path — is met.
  (`RV-372` `F-9`.)
- **Filing is the cheapest way to make a gap look handled.** Three of this
  slice's execution-era artefacts are backlog items standing in for work
  (`CHR-074` for `C7`, `CHR-075` duplicating `CHR-073`, and the four `ISS` rows).
  Two of the five turned out to need audit intervention.

### Ids minted during execution

`DEC-274` (accepted) · `CHR-074` (open, carries `C7`) · `CHR-075`
(**closed/duplicate** of `CHR-073` at audit) · `ISS-463`, `ISS-464`, `ISS-465` ·
`ISS-466` (**renumbered from `ISS-462`** at audit — collided with `edge`'s;
`ISS-279` widened with the instance) · three memories
(`mem.fact.doctrine.entity-key-inbound-order`,
`mem.pattern.testing.mutation-beat-asserts-application`,
`mem.pattern.dispatch.verify-worker-gate-claims`).

Minted at audit: `RV-372` · `ISS-467`, `ISS-468`, `ISS-469` · `IMP-465`.

### The human verdict

`VH-1` discharged 2026-09-20: **"disappointing as a feature, but not obviously
incorrect."** Correctness is not disputed and the gate is green; the product is.
Carried by `IMP-465`, with the withheld-tier complaint split to `ISS-467` as a
real `STD-003` gap. `RV-372` `F-5` holds the verbatim attestation.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-20 · `/reconcile` written — `RV-372`'s brief discharged into `design.md` and `slice-246.md` (eleven edits, zero REVs); the outcome is `RV-372` § *Reconciliation Outcome*, not restated here. Remaining before `/close`: `CHR-073` (gating, `/reviewing-memory`), then land `dispatch/246` by **merge, not squash** — a squash orphans `mem.pattern.platform.never-export-git-dir-to-a-test-run`, recorded on this worktree. · dispatch/246 @ 4a25746c5
