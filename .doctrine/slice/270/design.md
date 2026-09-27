<!-- doctrine:section sec-1 -->
## What changes and why

`RFC-026` `P10` proposes routing each severe design-review finding to the
instrument that can settle it. Its first trial window (`E12`, reported by
`CHR-077`) found 0 of 18 severe findings routed: nothing checked the form, so the
mechanism was never exercised. This slice makes a second window able to measure
the mechanism rather than delivery, and fixes that window's rules before it
opens.

It ships three things:

1. **A six-route set** (`DEC-330`, `DEC-333`): `owner-fix` is replaced by
   `dedupe` and `refresh`, on the pre-registered re-rate `E14`; and one sentence
   separates `probe` from `control`.
2. **A route-presence check at the design-run lock** (`DEC-326`): a design cannot
   lock while a disposed `blocker` or `major` finding on its review carries no
   known route.
3. **A frozen protocol and its chore** (`DEC-334`, `CHR-082`): eligibility, a
   build floor, collection and reading rules, run after this slice closes.

```text
 SL-270 (this slice)                              CHR-082 (after SL-270)
 ┌──────────────────────────────────────────┐     ┌─────────────────────────┐
 │ route set 5→6 ─┐                         │     │ window: next 3 design   │
 │ probe/control ─┼─► reviewing.md,         │     │ reviews, build floor    │
 │   sentence     │   review-ledger.md      │ ──► │ checked, facts          │
 │ lock check ────┴─► design-run gate       │     │ collected, 2nd rater,   │
 │ DEC-334 protocol (frozen)                │     │ report as RFC-026 E15   │
 └──────────────────────────────────────────┘     └─────────────────────────┘
```

Governance follows through `REV-065` (approved, applied at reconciliation):
`ADR-007` D-C5 and `SPEC-032` list the route set, and `SPEC-032` says an unknown
route gates nothing, which the lock check makes untrue for one reader.

Out of scope, with the decision that removed each: refusing `--route` outside
design review (`DEC-331`), and writing counters at `conclude` (`DEC-327`: they
are already journal-derived).
<!-- doctrine:section sec-2 -->
## The six-route set (DEC-330, DEC-333)

| route | the question behind the finding | what settles it |
|---|---|---|
| `review` | should we accept this commitment and its consequences? | unchanged |
| `demonstrate` | can these parts connect as proposed? | unchanged |
| `probe` | does the mechanism withstand the adversary? | unchanged |
| `control` | would the planned check notice failure? | unchanged |
| `dedupe` | are there two live accounts of one fact? | delete the duplicate, verify the surviving owner, sweep the affected class |
| `refresh` | does a record lag the thing it describes? | confirm which side is right, edit the record, sweep the records that cite it |

**Code.** `Route` in `src/review_ledger/vocab.rs` loses `OwnerFix` and gains
`Dedupe` and `Refresh`; `ROUTES` follows, and the existing lockstep test
(`route_known_set_matches_variants`) guards the pair. The MCP `review_dispose`
and `review_amend` schemas already enumerate `ROUTES`, so they follow with no
edit; the literal route lists in `review_show`'s description
(`src/mcp_server/tools.rs:143`) and the CLI help (`src/review/cli.rs:137,182`)
are edited by hand.

**Docs.** `install/design-prompts/reviewing.md`:

- the route table replaces its `owner-fix` row with the two rows above;
- the tie-break reads *take the first of `dedupe`, `refresh`, `control`,
  `probe`, `demonstrate`*;
- a new sentence after the tie-break: *between `probe` and `control`, the
  subject decides: doubt the mechanism's resistance, route `probe`; doubt the
  check's detection, route `control`*;
- a `refresh` whose described thing is itself wrong is a defect and routes
  `review`, stated in the `refresh` response-form bullet;
- the response-form bullets and the prose-settled sentence name `dedupe` and
  `refresh` where they named `owner-fix`.

- "exactly five" (`:118`) and "only the closed five" (`:190`) say six; the
  closing paragraph names the lock check as the one thing that reads the route.

`install/review-ledger.md`'s route-axis entry lists the six.

**Other sites** (RV-400 F-6): the golden tests in `tests/e2e_review_golden.rs`
that write or render `owner-fix`; the memory
`mem_01a0d91d7a0d75f1853c505d08237451`, which teaches `--route owner-fix`; and
`ADR-007` / `SPEC-032` through `REV-065`.

**Legacy.** A client ledger carrying `owner-fix` keeps it: reads are open
(`Route` is parsed only on write), `review show` prints the raw string, and the
lock check (sec-3) reads it as an unknown route. No migration.
<!-- doctrine:section sec-3 -->
## The route-presence check at the lock (DEC-326)

The lock's `review-disposition-attested` condition already re-reads the design
review each time it is evaluated, and refuses while blockers are open
(`src/design_run/gate.rs:1711`, DEC-138). The presence check is a second list
on the same observation.

```text
commands/design.rs ──observe_pass(RV)──► review_ledger::PassFacts
                                          ├ undisposed_blockers   (exists)
                                          └ unrouted_severe       (new)
          ObservedReview { …, unrouted_severe: Vec<String> }
design_run::gate ── non-empty ──► Cause::SevereFindingsUnrouted { findings }
```

**The predicate** (`review_ledger/gate.rs`, beside `undisposed_blockers`, pure):
a finding is listed iff

- it is **severe**: severity is not known `minor` or `nit` (an unknown severity
  counts, as `gates_as_blocker` does for blockers);
- it is **disposed**: status is known `answered`, `contested` or `verified`.
  An unknown status is excluded: `vocabulary_defects` already discloses it and
  no review verb can act on it. A finding already listed as an undisposed
  blocker is excluded, so no finding is reported twice (RV-400 F-7); and
- it is **unrouted**, for one of three reasons, each named in the entry:
  - `no route` — the `route` field is absent;
  - `legacy route: prefix` — absent, and the disposition starts `route:`;
  - `unknown route <raw>` — present, and `Route::parse` refuses it.

Entries are rendered on the ledger side as `F-n (<reason>)`. The gate is `leaf`
and `review_ledger` is `engine` (ADR-001), so the gate treats the entries as
opaque labels, as it already does the blocker ids: it only needs *non-empty
refuses* and *what to show*.

**The refusal** renders *severe findings carry no route: F-3 (no route),
F-7 (unknown route owner-fix)* and is capped like the other list causes
(`carries_a_list`, `cut`). Its remedy names the repair per status:
answered, `review amend <RV> --finding F-n --route <route> --note …`;
contested, dispose again with `--route`; verified, the raiser reopens and the
responder disposes again with `--route`.

**Not covered, on purpose.** Findings on superseded passes and findings
disposed after lock (the lock reads the current pass, once); open findings (open blockers already refuse; open
majors are admissible at lock today); withdrawn findings; a waived review
disposition, which never reads the ledger. The check is not facet-filtered: a
design run's review pass is the only ledger the lock reads.
<!-- doctrine:section sec-4 -->
## Protocol and trial chore (DEC-334, CHR-082)

`DEC-334` fixes the second window before any eligible slice opens, and
supersedes `DEC-276`: its capture rule by `DEC-327`, its procedure by itself.
Amended on RV-400 F-1, F-3, F-4, F-5 and F-9. In short:

| item | rule |
|---|---|
| eligibility | the first 3 code-changing slices whose design run mints a review pass after the window opens, in minting order; waived reviews recorded and skipped; parked or abandoned slices stay in, with status |
| build floor | at window open and each eligible design start: CLI help and the MCP `review_dispose` schema list six routes (they carry the embedded `reviewing.md`); `doctrine install` run at window open and projections recorded current. A failure holds the slice |
| per finding | route; instrument evidence produced (instrument routes); L - the repair or obligation drew a related finding or contest in the ledger, a later design pass, or the audit ledger, up to slice close |
| per ledger | rounds, contests (journal), artefact-prose findings, findings against repair text, design growth, later audit findings, completion |
| second rater | blind, six-route prompt with tie-break and the probe/control sentence; kappa against 0.44 (design review); prompt changes named, not attributed; probe/control reported apart |
| reading | no pass mark, descriptive only; presence checked on the gated population (final pass at lock) and the rest reported apart; headline L against E12 (same-ledger 50-61%, the only baseline), scope difference stated |

`CHR-082` runs it and reports `RFC-026` `E15`. This slice's only obligation to
it is to land sec-2 and sec-3 and to install the result, so the build floor can
pass on the day the window opens.
<!-- doctrine:section sec-5 -->
## Verification

| id | mode | asserts |
|---|---|---|
| VT-1 | test | `Route::parse` accepts exactly the six; `owner-fix` is refused on write with the known list; lockstep with `ROUTES` holds |
| VT-2 | test | `unrouted_severe` lists an answered major with no route (`no route`), an answered blocker whose disposition starts `route:` (`legacy`), a verified major with route `owner-fix` (`unknown route`), and a contested major with no route; omits a routed one, an open one, a withdrawn one, a minor, and an open route-less blocker (already an undisposed blocker) |
| VT-3 | test | an unknown severity counts as severe; an unknown status is excluded |
| VT-4 | test | the lock refuses on a concluded, blocker-free pass with one unrouted major, naming it; the same run locks once the finding is amended with a route |
| VT-5 | test | `Cause::SevereFindingsUnrouted` is a capped list cause (joins `carries_a_list`) |
| VT-6 | test | the MCP `review_dispose` schema's route enum equals the six |
| VA-1 | agent | `reviewing.md` and `review-ledger.md` carry the six routes, the new tie-break order and the probe/control sentence; a repo-wide search finds no `owner-fix` and no "five" route count outside legacy notes, E11-E14 evidence and `REV-065`'s before column |

The gate test needs a fixture pass with a route-less answered major; the
existing fixtures in `src/review/tests.rs` build ledgers with findings and
extend to it.
<!-- doctrine:section sec-6 -->
## Risks and residue

- **A verified unrouted finding is awkward to repair** (reopen, then dispose
  again). Accepted: it arises only when the rule was skipped, and the refusal
  names the path.
- **The gate covers less than the trial counts** (RV-400 F-1): open majors,
  superseded passes and post-lock dispositions escape it. `DEC-334` reports
  them apart rather than widen the gate. Carried to audit as a follow-up: decide
  whether this is the behaviour wanted long-term, or needs closer review or a
  backlog item (owner, 2026-09-27).
- **The check is permanent code on a provisional axis** (`DEC-268`). If
  `CHR-082`'s trial kills `P10`, the check and the route vocabulary go with it.
- **Stale projections.** An agent served by an old binary, MCP server or
  installed skill still sees five routes. The build floor in `DEC-334` holds
  eligible slices until all three are current; this slice does not add a
  freshness check to the binary.
- **Rater noise.** `E14`'s kappa of 0.44-0.56 means any single agent's route is
  a noisy label. The protocol reports disagreement; the lock check guarantees
  presence only, never correctness.
- **Unenforced residue** (`DEC-103`): whether an instrument route's obligation
  is met is still not checked by any gate. The docs' closing paragraph says so,
  updated to name the lock check as the one thing that is.
