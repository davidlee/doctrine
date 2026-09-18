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

**A sixth adversarial pass is not warranted, and this is the first round where
that is true.** The two grounds that carried rounds 3, 4 and 5 separate here:

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

**Trend, stated without leaning on it.** Rounds raised 8, 9, 4, 5, 2. The count
never fell cleanly, but the *severity* did — blocker+major, major, major, one
major, then a minor and a nit — and round 5's clean-on-inspection list was long
and specific (every `file:line` in `design.md`, the `STD-001` template question,
the layering rule against the tree, `D7`'s enforceability, `C2` under the policy
parameters). Surface shrinking, tally flat.

**The pattern worth naming.** Rounds 3, 4 and 5 each found their most structural
defect *in the previous round's repair*, not in the original draft: `F-23` was
`F-20` displaced one layer up; `F-29` was `F-23`'s own test sited one step too
late; `F-25` caught diagrams a self-attack pass had rewritten and left a state
short. A repair round is where the next round's findings come from, and reading
one's own repair is not the same check as a reader taking the model from it.

**Not a substitute for the attestations.** Nine remain outstanding and the policy
is `human-only`. An adversarial round cannot discharge them, and each binds a
revision — round 5 read 49, so it could not attest 51 either.

**Reviewer.** codex remains out of credits; rounds 2-5 all ran on Opus.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-18 · design run at `reviewing` (revision 51, materialised) · b00008681

### Produced
- `RV-370` — the design review ledger, **three rounds**, 22 findings. Round 1
  (codex): `F-1`..`F-8`. Round 2 (Opus, codex out of credits): seven `verified`,
  `F-4` contested and upheld, `F-9`..`F-17` raised. Round 3 (Opus, in the primary
  worktree so it recorded its own friction): nine `verified` with their claims
  re-derived from the tree, `F-14` contested and upheld, `F-18`/`F-20`/`F-21`/
  `F-22` raised. All fourteen round-2 and round-3 findings adjudicated correct —
  no confabulation in any round. Ledger `await=raiser`, rounds 64.
  `F-19` is a withdrawn row, not a retracted claim: a shell-mangled `--title` at
  raise, with no edit verb to repair it
- `DEC-261` — the design read reclaims `design show`; supersedes `DEC-260`,
  which is now `superseded`
- `design.md` adopted at run revision 46 and materialised at 47, byte-identical.
  Round 1 moved eight of nine sections at r38; rounds 2 and 3 each moved the same
  five — `sec-3`, `sec-5`, `sec-7`, `sec-8`, `sec-9`. `sec-4` has never moved
- `D6` and `D7`, the round-3 decisions. `D6`: one per-record producer per arm
  (`render_record` / `record_value`), the block a **map** over it, `show_value`
  split out of `show_json`, and each marker composed at the layer that holds what
  it must name. `D7`: the flag partition is stated over **renderings**, not
  spellings — which retired the round-2 `--json` rule at the user's ruling, and
  made `--known-revision` decidable without a fourth ruling
- `slice-246.md` scope reconciled to `DEC-261` — in two passes. The first
  (`876d86a40`) added the scope-addition prose and objective 5 but left
  *Affected surface* untouched, where `src/commands/design.rs` still sat under
  *Dropped by the inquiry*. `review.scope` was recorded here as discharged on
  that pass; it was not. Rewritten at revision 43 as an orientation map over
  § 5.6, and discharged then
- design-target selectors re-pointed off `DEC-260`'s siting at revision 44 —
  two removed, eleven added, including the four emitted-string sites located
  in the doing. `selector doctor`: one unmatched, the § 5.6-new golden
- the `reviewing` runbook is **cleared** — `review.scope`, `review.selectors`,
  `review.passes` all attested, and it stayed cleared across the round-3 adoption
- `IMP-457` closed `duplicate` of `IMP-393`, with the boundary written onto it
- `IDE-054` — audit the CLI for format/content axis coupling (`F-9`'s declined
  principled split); `originates_from SL-246`
- `CHR-073` — re-attest the five memories naming `design show` as the envelope
  read (`F-16`); `originates_from SL-246`. No `needs` gate minted
- `SL-246 fulfils IMP-393 --degree partial`; `DEC-261 shapes SL-246`
- four `friction` observations, all committed — codex credit exhaustion, plus
  three recorded on the round-2 reviewer's behalf (a fork cannot capture its own)

### Learned
- `mem.fact.doctrine.show-is-not-cheap` — `<kind> show` measures slower than
  `inspect`; do not cost a design on `show` being a cheap per-entity read
- `mem.pattern.review.bind-scope-bar-and-never-self-rule` — applied twice, and
  it paid both rounds: `F-8` was found by the design's author and put to the
  reviewer rather than self-ruled
- `mem.pattern.design.counts-state-their-population` — a count asserted as
  measured must name the population it counted over. From `F-16`, where the
  figure was both uncheckable and wrong, from one cause
- `mem.pattern.design-run.correcting-a-locked-run` extended twice —
  `adopt_authored.sections` takes section DIGESTS not bodies and is mandatory
  though the contract prints it optional; and the section body is a **raw byte
  slice**, where a line-splitting implementation drops one newline too few on
  every section but the last (diagnostic signature: last section matches,
  earlier ones do not, whole-file hash matches)
- §3 `F1`'s corpus-scan claim verified independently: every scan reads, parses
  and validates all knowledge records and keeps only their edges — 362 records,
  988,841 bytes
- **reconciling a scope to a decision means walking every section of it.** The
  `DEC-261` reconciliation rewrote the prose the decision was about and left the
  *Affected surface* list asserting the opposite — `src/commands/design.rs`
  *dropped*. A scope's derived-feeling lists (affected surface, selectors) are
  exactly where a decision goes stale silently, because nothing reads them until
  planning. Candidate memory
- **a runbook step is discharged by the machine, not by prose claiming it.**
  The harvest asserted `review.scope` discharged "for both rounds"; the run had
  never taken the discharge, and re-facing it is what surfaced the stale list.
  The two failures are the same failure from opposite ends — the machine was
  right and the note was wrong. Candidate memory
- **three findings can share one cause, and integrating them flat rewrites the
  same block three times.** `F-14` (a guarantee claimed from a return type),
  `F-18` (a JSON arm with no producer) and `F-20` (an empty-state policy sited
  where its inputs are unreachable) were one defect: there was no per-record
  layer. Dependency-ordering the triage before adjudicating is what surfaced it.
  Candidate memory
- **the self-attack pass earns its place on diagrams.** Round 3 found nothing
  wrong with §5.1's flowchart or §5.4's sequence — they were correct when it
  read them. The repair falsified both (`render_block` no longer reads records)
  and only the post-integration self-attack caught it. A diagram is a projection
  of the prose and goes stale silently with it. Candidate memory
- **a repair round is where the next round's findings come from.** Rounds 3, 4
  and 5 each found their most structural defect in the *previous* repair, not in
  the draft: `F-23` was `F-20` displaced one layer up, `F-29` was `F-23`'s own
  test sited one step too late. Round count is not what converges; severity is.
  Candidate memory
- **re-adopting a hand-edited `design.md`**: `adopt_authored.sections` is a map
  of `sec-N` → the **full-length** sha256 hex of the section body, where the body
  is everything after the marker line, `rstrip()` plus one trailing newline.
  `design show` prints those digests truncated to 12 chars, which is the trap —
  sending the truncation, or sending the section text, both refuse with
  `0 missing, 0 unknown, 9 mismatched`. `ISS-320` already carries the gap
  ("a section map nothing emits"); this is the working recipe. Candidate memory
- **the slice scope has no staleness signal** — `design.md` is
  section-fingerprinted so a moved section voids its own attestation, and
  `slice-NNN.md` carries nothing equivalent. Four consecutive rounds repaired the
  design and left the scope asserting something falsified (`F-22`, `F-26`,
  `F-28`). Captured as `IMP-461`
- round 2's own negative result: the reviewer read and found sound §2.6, §3.1's
  six three-level groups and acyclic claim, `F1`, `FacetValue` sufficiency, the
  one-table feasibility, `Full`'s field list, the prose-size hint, `R5`, and
  `F5`'s emitted-string count — at **revision 38**, which is why it cannot
  attest revision 42

### Open
- `QUE-223` — should the design read disclose that `design.md` is behind its run
- `ASM-011` — inbound edges as a sufficient proxy, held with a measured miss
- **nine section attestations outstanding** — `sections_outstanding_review=9`,
  `review_pass STALE`, `review_policy = human-only`. The attestation is a user
  act (`declare` with `attests: sec-N`, `reviewer: human`) and an agent must not
  author it. The adversarial lane cannot substitute: an attestation binds the
  revision, and the last adversarial read was revision 49 against a document now
  at 51. **This is the only thing between the design and its lock** — the ledger
  needs one adjudication turn, which is running; the nine are the user's
- `CHR-073` sequencing — must land with or before the code, since a stale memory
  is injected into agent context. Whether it warrants a hard `needs` gate on
  this slice is left to close, deliberately
- `SPEC-013`'s "two-level clap subcommand tree" clause is descriptively false of
  six three-level groups under numbered entity kinds (`RV-370` `F-1`). `SL-246`
  conforms so needs no `REV`; the clause still wants one. Not this slice's.
