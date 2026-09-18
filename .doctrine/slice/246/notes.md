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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-18 · design run at `reviewing` (revision 42) · 55b41b97c

### Produced
- `RV-370` — the design review ledger: round-1 bar + ten lines of attack,
  `F-1`..`F-8` raised by codex and disposed, round-2 bar bound, **round 2 run on
  the Opus reviewer** (codex out of credits) — seven dispositions `verified`,
  `F-4` `contested` and upheld, `F-9`..`F-17` raised, all ten adjudicated
  correct and disposed. Ledger `await=raiser`, rounds 43
- `DEC-261` — the design read reclaims `design show`; supersedes `DEC-260`,
  which is now `superseded`
- `design.md` adopted at run revision 42, `materialise` byte-identical. Round 1
  moved eight of nine sections at r38; round 2 moved five — `sec-3`, `sec-5`,
  `sec-7`, `sec-8`, `sec-9`
- `slice-246.md` scope reconciled to `DEC-261` and extended for round 2
  (discharges `review.scope` for both rounds)
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
  revision and `RV-370` read r38, five sections behind
- `CHR-073` sequencing — must land with or before the code, since a stale memory
  is injected into agent context. Whether it warrants a hard `needs` gate on
  this slice is left to close, deliberately
- `SPEC-013`'s "two-level clap subcommand tree" clause is descriptively false of
  six three-level groups under numbered entity kinds (`RV-370` `F-1`). `SL-246`
  conforms so needs no `REV`; the clause still wants one. Not this slice's.
