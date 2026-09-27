# Routing trial two

## Context

`RFC-026` `P10` (*route each severe finding to the instrument that can settle
it*) had one trial window, reported by `CHR-077` as **`E12`**: 0 of 18 severe
design-review findings carried a route, so no `demonstrate` / `probe` / `control`
obligation was ever exercised. The mechanism is **untested, not falsified**. The
cause was delivery: the rule sat in `reviewing.md`'s tail and nothing checked the
form. `CHR-077` also ran **`E13`**, a classification of 47 audit and code-review
findings, which settled `QUE-224` (*extend the route axis beyond design review?*)
as *not extended* and filed **`P11`** (*split `owner-fix`, or name the
stale-account class*).

A review of that work (2026-09-27) found the following. Each item is either a
precondition this slice must meet before a second window opens, or a defect in
the evidence it inherits.

### What still lets a second window fail the same way

1. **Route presence is still unchecked.** `SL-268` validates the route *value*
   (`--route` accepts only the closed five), but `--route` is optional on
   `review dispose`, and `reviewing.md` says *"there is no default"* without a
   verb enforcing it. A severe design-review finding can still be disposed bare,
   which is exactly `E12`'s failure. `E12` confound 2 claims `--route` "closes
   exactly" the "nothing checks the form" gap. It closes half of it.
2. **The binary agents actually run may not have `--route`.** On this machine
   `.mcp.json` launches `${DOCTRINE_BIN:-doctrine}`, and the PATH build (0.45.0)
   serves a `review_dispose` tool with no `route` parameter. Meanwhile
   `reviewing.md` tells responders to *prefer the MCP tool*. A responder that
   follows the instruction cannot route.
3. **The at-conclusion capture is a checklist line.** `DEC-276` requires
   `review status` to be copied into an authored record before the gitignored
   baton is lost. `CHR-077` missed it on all three ledgers. A second window run on
   the same honour system will miss it again.
4. **`--route` is not gated by facet**, while `install/review-ledger.md` scopes
   the axis to design-review ledgers. `E13` records this as a residual. It needs
   a gate or a doc concession, and no entity carries it.

### What is weak in the inherited evidence

5. **`E13`'s rater prompt steers the headline.** Its rules say: *"Where a finding
   is about shipped code not matching what the design or a governance record
   says, say so and let that decide the route."* Only `owner-fix` asks *"do two
   accounts disagree?"*. Both raters were told to treat code-vs-record mismatch
   as decisive, then both reported that `owner-fix` absorbs code-vs-record
   mismatch. That agreement is partly the instrument echoing itself.
6. **`E13`'s "both raters at 21 of 47" is a marginal coincidence.** Row by row,
   the raters agree on 34 of 47 findings (72%). Only **18** findings are
   `owner-fix` under both, and 6 of the 13 disagreements involve `owner-fix`,
   3 in each direction. The report presents identical totals as independent
   convergence. The reconciliation facet "fails" the reading rule at 13 of 25
   (52%) against a 50% threshold that was not published before the run. One
   finding decides it.
7. **`P11` is grounded in a population `QUE-224` just excluded.** All of `E13`'s
   `owner-fix` evidence comes from reconciliation and code-review, the facets
   where routing will *not* apply. Within design review, `owner-fix`'s load is
   unmeasured (`E12` recovered one). `QUE-224`'s *not extended* is probably
   right, but for a different reason than the one it records: routing chooses an
   instrument *before* code exists, while reconciliation and code-review findings
   arise *after*. A stale record there is simply edited, which `/reconcile`
   already does.
8. **`E12`'s zero artefact-prose finding is weaker than "consistent with P10".**
   `E11`'s own comparison group (converged ledgers) also had 0% artefact prose,
   so 0 of 18 is what a converging ledger looks like, not a signal. And the axis
   `P10` actually targets, repair text drawing further findings, is *high* in the
   window: 9–11 of 18 (50–61%) repairs drew a later finding, and `RV-377` ran 68
   rounds. `E12` reports those numbers but does not read them against `P10`.
9. `E12`'s commit deleted `RFC-026`'s `## Hypotheses` heading, so H1–H11 read as
   part of `E13`. Restored 2026-09-27, outside this slice.

## Scope & Objectives

Make `P10` testable, then test it once, under rules fixed before the window
opens. The order matters: every mid-window change so far (`SL-268`) has been a
confound.

```text
 1 settle taxonomy ──► 2 enforce form ──► 3 freeze protocol ──► 4 window ──► 5 report
   (P11 split,          (lock route         (DEC-334:              (next 3      (RFC-026
    six routes;          check; REV-065)     eligibility,           design      E15,
    E14)                                     reading, floor)        slices)     CHR-082)
```

1. **Settle the taxonomy before the window.** Done in research: the unsteered
   re-rate (`RFC-026` `E14`) resolved `P11` to a split. The route set becomes
   six: `owner-fix` is replaced by `dedupe` and `refresh` (`DEC-330`), and
   `reviewing.md` gains a `probe`/`control` boundary sentence (`DEC-333`). The
   code and docs change before the window and stay fixed through it.
2. **Enforce route presence in code** (findings 1, 2): the design-run lock
   refuses while a disposed `blocker`/`major` finding on the design review
   carries no known route (`DEC-326`). Two items left the original scope:
   refusing `--route` outside design review (`DEC-331`: the evidence no longer
   supports it, and the lock check already holds the scope), and writing
   counters at `conclude` (`DEC-327`: rounds and contests are already
   journal-derived, finding 3).
3. **Freeze the protocol** in `DEC-334` (supersedes `DEC-276`): eligibility (the
   next 3 code-changing slices with a conducted design review), a build floor
   covering the CLI, the MCP server and installed skills (finding 2), collection,
   a blind second rater, and reading rules headed by `L`.
4. **Run the window** in `CHR-082`, sequenced `after` this slice, as `CHR-077`
   was to `SL-260`.

## Non-Goals

- Extending routing to audit or code-review (`QUE-224` stands). Not refused
  either (`DEC-331`).
- A pass mark. `P10` sets none and three to five ledgers cannot carry one.
- Re-litigating `E11`–`E13` counts. Findings 5–8 qualify their readings, and
  the re-rate in step 1 supersedes `E13`'s headline rather than amending it.

## Summary

The first window measured delivery and found it absent. This slice closes the
delivery gaps in code, settles the taxonomy on unsteered evidence, and fixes
the protocol before a second window, so that window can measure the mechanism.

## Follow-Ups

- Trial window and report: `CHR-082`.
