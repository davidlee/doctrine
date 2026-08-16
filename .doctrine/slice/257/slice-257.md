# Authority floor and assurance profile

## Context

Two defects sit at the same seam in the capsule conformance suite, and neither
depends on the mechanism question that is currently unsettled.

**The verdict can be vacuously strong.** `EVD-021` (`confirmed`) establishes that
the existing reducer already admits an empty row list — `.all()` over an empty
slice is vacuously true — and that its only guard is an external assertion that
one fixed `tables()` is non-empty. `DEC-189` makes row membership a function of
the mechanism, which dissolves that guard. The reduction is `admission` at
`crates/doctrine-control/src/conformance.rs:5166`, with one production call site
at `:5326`.

**The spec records a discharge that is false.** `REQ-459` criterion 1 enumerates
seven property families in one undifferentiated list with canonical-authority
denial inside it, conflating an invariant with a set of things that vary.
`REV-051` — `done` and applied — disposes criterion 3 as *"discharged
structurally — one suite parameterised by backend; a second backend passing it
edits nothing."* `DEC-189` falsifies the premise: a hypervisor loses rows 10, 12,
13 and 14 and earns others, so there is no single suite for a second mechanism to
pass.

**Why this is its own slice.** These were objectives 2, 5 and 6 of `SL-253`
(*Conformance verdict kernel*). `SL-253`'s remaining objectives — the
backend-neutral kernel extraction — exist to make a verdict shape portable across
mechanisms, and `QUE-217` (*which casual capsule backends should complement
hardened microVMs*, `open`) has not decided which mechanisms there will be.
`SL-253` is therefore blocked on `QUE-217` and carries a `needs` edge to it
(`ADR-017`). Nothing here waits on that answer: the vacuity is real on the one
mechanism that exists, and `DEC-189` falsifies `REV-051`'s disposition regardless
of whether any code moves.

**What that costs, stated rather than assumed.** `DEC-190`'s *"a few hundred
lines of policy and a bonfire for the remaining 13,500"* does not happen here.
`conformance.rs` stays 14k lines and `RSK-231`'s engagement-cost thesis stands
unaddressed. This slice buys correctness at the seam, not a reduced surface.

## Scope & Objectives

1. **Reduce a closed floor; publish the profile — in place** (`DEC-195`). The
   verdict carries two structures of different semantics rather than one
   AND-reduction:
   - an **authority floor**: a closed enum reduced by *exhaustive match*, not
     `.all()` over a `Vec`, so an empty floor is unrepresentable and adding a
     floor property fails to compile. Its membership is
     `Property::DeniedCanonicalStateAndCredentials` — row 3 — and nothing else;
   - an **assurance profile**: the remaining rows, published per row and **never
     reduced**. Fronts are open grouping metadata and are never a reduction
     target — a front carrying no rows is vacuously strong, which is the same
     defect one level up;
   - **backend unavailability lifts to a top-level verdict variant** rather than
     sitting inside the row-failure enum, preserving `POL-002` facet 3's
     `missing` / `remedy`: not-run and ran-and-failed are different claims.

   The floor's **reading** and its **standing** stay two questions (`SL-253`
   design `D7`): the floor type is total so an empty floor is unrepresentable,
   and a wrapper carries *the floor row was never submitted* — a state of the
   run, distinct from *the answer was no* — with the rows that did run still
   published alongside.

   **The profile keys on today's closed `Property`.** `DEC-198`'s open,
   mechanism-minted assurance key is deferred with `SL-253`: it exists so a
   mechanism nobody has written can mint its own rows, and it is what drags the
   thirteen-of-fourteen `Property` migration `DEC-198` records as a residual.
   Nothing in this slice's floor needs it — closedness is what the floor wants.

2. **Compute the summary word at the command tier** (`SL-253` design `D4`), from
   the floor's standing rather than storing a scalar, because a stored scalar can
   disagree with the rows it was computed from — `DEC-191`'s original complaint in
   miniature.

3. **Label the profile's fronts as *escape* fronts where it renders** (`CPT-002`),
   so a strong profile is not read as a strong safety claim. `DEC-191`'s front
   list stays the sketch it is; closing it is separate work.

4. **Carry the `REV`** — `DEC-201`'s one revision, four payloads, over
   `SPEC-030`/`REQ-459` and `REV-051`, landing with the code so no commit has the
   spec contradicting the binary:
   1. `REQ-459` criterion 1 splits into two criteria of different invariance — an
      authority floor proven on every mechanism, and an assurance profile that
      varies per mechanism and is published rather than reduced;
   2. criterion 3's text narrows — *same floor, own profile*: `edits nothing`
      survives, *the same property suite* does not;
   3. `REV-051`'s criterion-3 disposition is corrected to match;
   4. `IMP-405`'s platform-versus-mechanism rename across `SPEC-030` § *Platform
      backend contract*, and `CPT-002`'s threat priority into § **Concerns** (not
      § Overview).

   **One amendment to `DEC-201`, taken at this cut.** `DEC-201` assumed the
   narrowed criterion 3 landed *discharged*, because `DEC-198`'s open key would
   have shipped alongside it. Deferring that key means a second mechanism would
   still have to edit the enum to publish its own rows, so criterion 3's
   disposition becomes **undischarged** rather than re-discharged, and `SL-253`
   is what discharges it. `DEC-201` itself is unviolated: still one REV, four
   payloads, landing together, with no window in which they disagree.

### Affected surface

- `crates/doctrine-control/src/conformance.rs` — `admission` (`:5166`) and its
  single production call site (`:5326`); the verdict types (`:2682-2838`). The
  file is not split and does not move.
- `crates/doctrine-control/src/main.rs` — `admit` (`:153`), `render_verdict`
  (`:178`), `render_outcome` (`:217`), and the exit constants (`:60,63`). `admit`
  and `render_outcome` are the only two production consumers of the collapsed
  scalar. `render_verdict` emits one flat `row {id:?}={row:?}` line per row with
  no grouping structure — the shape the rendering delta is measured against.
- `.doctrine/spec/tech/030/`, `REQ-459` and `REV-051` — via the `REV`, not by
  direct edit.
- `justfile` — `capsule-check` (`:111-113`) and `capsule-verify` (`:130-146`),
  neither wired into `check` or `gate`, so verification design must say how the
  proof is run.

### Risks and assumptions

- **Behaviour preservation, at a lighter weight than `SL-253` needed.**
  `AGENTS.md` requires existing suites to stay green when shared machinery moves,
  and `RV-352`'s baseline is the bar: nineteen row verdicts, four auxiliary
  claims, two unrowed readings. Those must reproduce exactly. The artefact's
  *text* still changes — the floor row leaves the flat row list and the outcome
  line changes shape — but this slice does **not** re-key row identity, which is
  what forced `DEC-199`'s seven-rule total line map and derived golden. What
  bracket is proportionate here is `OQ-1` below. `EVD-022` (`captured`) is the
  pre-change transcript and is the right bracket either way.
- **`crates/doctrine-control` is outside every default gate selection** (`RV-353`
  `F-4`) — Linux-only, live-`bwrap` rows — and is reached by `just capsule-check`,
  not `just gate`. **Both** instruments need `bwrap`: `capsule-check` runs
  `cargo test -p doctrine-control`, whose suites assert `availability() ==
  Available` and provision real capsules. Nested `bwrap` works in the project
  jail, so the proof runs here.
- **Assumes `ADR-020` is not reopened.** Authority stays the floor.
- The vacuity fix is a **type-level** change, so its proof is partly a
  non-observation: the test that would have exercised the empty path can no
  longer be written. Verification design must say what stands in for it.

### Open questions

- **`OQ-1` — what weight of behaviour-preservation bracket is proportionate?**
  `DEC-199` cut a total line map with seven derivation rules, a whole-output
  golden and a one-shot transform script deriving the golden's expected value.
  That was sized for a change that re-keys row identity and moves the types.
  This one does neither. Candidates: (a) `DEC-199` as written; (b) layer 1 only —
  a characterisation test pinning the row-to-verdict mapping as data, plus the
  rendering delta reviewed as a diff. Settle in `/design`; the answer decides
  whether the transform script is in scope at all.
- **`OQ-2` — does the `DEC-201` amendment above need its own record?** The
  amendment changes a consequence of an `accepted` decision, and the honest
  options are a new `DEC` superseding that consequence or an appended note.

### Verification and closure intent

Done is: an empty or short row list can no longer be admitted, and the failure is
a **compile error rather than a runtime verdict**; `RV-352`'s nineteen row
verdicts, four auxiliary claims and two unrowed readings reproduce exactly; the
verdict publishes a floor and a per-row profile in place of a scalar, with the
summary word computed at the command tier and fronts labelled as escape fronts;
and `SPEC-030` and `REV-051` no longer assert what `DEC-189` falsified.

`just capsule-check` is the per-phase gate, green at the end of every phase.
`just capsule-verify` is a phase exit criterion for any phase that can reach a
row, which is the default.

## Non-Goals

- **The kernel extraction.** `DEC-190`, `DEC-196` and `DEC-197` stay with
  `SL-253`. `conformance.rs` is not split, `BackendId` and `Availability` do not
  move, and no unit is reclassified in `ADR-001`.
- **`DEC-198`'s open assurance key**, and with it the thirteen-of-fourteen
  `Property` migration.
- **`DEC-194`'s qualification rename.** It was justified as free *because these
  types move anyway*; it is not free if they do not. `backend verify` keeps its
  name and the existing `EXIT_REFUSED` / `NotAdmitted` mismatch stands, unfixed,
  until `SL-253`.
- **`DEC-200`'s test-band carving.** The 186 test functions stay in one flat
  `#[cfg(test)] mod tests`; the carve exists to make a split cheap.
- **A Firecracker or microVM backend**, and any porting of the bubblewrap rows.
- **Closing `DEC-191`'s front list** from a sketch into an enumeration.
- **A ranking or scoring apparatus.** The fix is to stop collapsing.

## Summary

Stop the capsule verdict collapsing to a single all-or-nothing scalar: reduce a
closed authority floor by exhaustive match so the vacuous-admission path
`EVD-021` confirms becomes unrepresentable, publish the remaining rows as an
unreduced per-row assurance profile, and compute the summary word where it is
rendered. Land the four-payload `REV` alongside, so `SPEC-030` and `REV-051` stop
asserting the structural discharge `DEC-189` falsified. The file is not split and
nothing is renamed — that is `SL-253`, which this slice unblocks nothing of and
which waits on `QUE-217`.

## Follow-Ups

- `SL-253` — the verdict kernel extraction, the qualification rename, the payload
  isolation and the test bands, blocked on `QUE-217`.
- `DEC-198`'s open assurance key and `DEC-194`'s rename, with it.
- `DEC-191`'s front list, closed from a sketch into an enumeration.
- `IMP-427` — the third test band, so `just capsule-check` runs meaningfully on a
  host without `bwrap`.
