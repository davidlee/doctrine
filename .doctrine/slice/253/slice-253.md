# Conformance verdict kernel

## Context

`crates/doctrine-control/src/conformance.rs` is 14,252 lines in one file. It is
the capsule property suite `SL-248` landed, and it works: `RV-352` records two
independent gate runs at `297 passed; 0 failed`, with `backend verify` exiting 0
and all fourteen properties and five axes `Proven`.

The problem is not correctness, it is **cost of opinion**. `RSK-231` asks
whether the conformance surface can be reduced so that correctness is checkable
without loading the whole design; today forming a defensible opinion about one
row requires loading the production backend, the fixture, the placement
validator, and several phases of `SL-248`'s authored plan.

Four accepted decisions converge on this slice, and none of them can land alone:

- **`DEC-190`** splits the file into a small backend-neutral **verdict kernel**
  and a per-mechanism **payload layer**. Owner's direction: *a few hundred lines
  of policy and a bonfire for the remaining 13,500*.
- **`DEC-191`** stops the final AND-reduction. Assurance is a vector over
  escape fronts, not a rank and not a word; the verdict publishes the vector.
  `AdmissionVerdict` already carries `rows: Vec<(RowId, RowVerdict)>` — the
  vector is already in the type, and only the collapse to `Admitted` is wrong.
  Authority stays an invariant floor (`ADR-020`'s territory, unamended).
- **`DEC-194`** renames the mechanism axis to **qualification**, because
  *admission*, *verify* and *conform* each already name a different event on
  `ADR-020`'s work transaction. Its consequences require the rename to land
  *with* this split, since these types move anyway — otherwise it is a second
  migration.
- **`DEC-189`** refuses to port row *membership* across mechanisms. Four of the
  fourteen rows lose their host-facing delta under a hypervisor boundary; a
  microVM earns rows bubblewrap never could. Porting membership would rebuild
  `ISS-341`'s defect family on a new mechanism.

**`DEC-190`'s sequencing gate is clear.** It says *"do not extract before
`QUE-211` settles — a tier-blind kernel is a kernel rewritten."* `QUE-211` is
`answered`: `DEC-191` refused both of its candidates and put a per-front vector
in place of a rank. The central type's shape is therefore known before the
extraction starts, which is the precondition that was missing.

**The four decisions above are the slice's inheritance, not its design.** This
slice's own design run settled seven more — `DEC-195` through `DEC-201` — and
where they refine an inherited shape they govern. The largest such refinement:
`DEC-191`'s *vector over fronts* becomes, in `DEC-195`, a closed authority floor
reduced by exhaustive match plus an assurance profile published **per row**, with
fronts as open grouping metadata that is never a reduction target. Objective 2
states the resulting shape; read it rather than `DEC-191`'s sketch.

**Trimmed and blocked — 2026-08-17.** The claim above that *none of these
decisions can land alone* was wrong in one direction, and the error was
load-bearing. `DEC-195`'s floor-and-profile and `DEC-201`'s `REV` do not need the
extraction: the floor wants a closed set, and `DEC-189` falsifies `REV-051`'s
recorded discharge as a matter of fact rather than as a consequence of any code
moving. Both are **moved to `SL-257`**, which this slice now `needs`. What remains
here is the extraction itself — the kernel seam, `DEC-194`'s rename, the payload
isolation and `DEC-200`'s test bands — and every one of those exists to make a
verdict shape portable across mechanisms.

Which mechanisms those are is not decided. `QUE-217` (*which casual capsule
backends should complement hardened microVMs*) is `open`, and it holds that the
bubblewrap capsule is *"a candidate foundation for the casual tier, not
presumptively disposable and not the presumed hardened production backend"*.
`EVD-025` prices the alternative's host coupling; `OQ-1` below already refuses to
implement against a backend that does not exist. Extracting a neutral kernel
before that answer is building a seam for an unnamed counterpart, on the surface
`RSK-231` says is already over budget. This slice therefore carries a `needs`
edge to `QUE-217` and is gated on it (`ADR-017`), not abandoned: the extraction is
still the right shape once there is a second mechanism to extract *for*.

**The design run is stale against this trim.** It sits at stage `reviewing` with
one blocker and one major outstanding and all ten sections unreviewed, so nothing
downstream is built on it — but its nodes `inq-1`, `inq-2` and `inq-7` settled
decisions that now belong to `SL-257`. Re-enter it when `QUE-217` unblocks this
slice; do not plan from it as it stands.

## Scope & Objectives

1. **Extract the verdict kernel.** The seam cuts at row **identity**, not at the
   row (`DEC-196`). Kernel: the taxonomy (`Axis`, and row identity split per
   `DEC-198` into a closed authority-floor key and an open mechanism-minted
   assurance key), `RowVerdict` and the distinctions between its variants, the
   verdict types, the four evidential tiers (rows, axes, `Claim`/`AuxOutcome`,
   `Unrowed`/`Reading`), `row_verdict`'s two-arm algebra, and `verify_over` keyed
   on row identity. Payload: `Row`, `Delta`, `ArmShape`, `Under`, `Arm`,
   `run_arm`, `run_row`, `verify`, `PropertyRemoval`, `AuthorityGrant` and
   `ConformanceBackend` and `ArmResult` — construction and diagnostics, all of
   it. `DEC-156`'s control discipline divides along that line: its portable half
   is `row_verdict`'s algebra and stays; its construction half is mechanism-keyed
   and goes. `BackendId` **and** `Availability` **move into the kernel**
   (`DEC-197`, re-cut 2026-08-12), which then imports no other module of
   `doctrine-control` at all; `backend.rs` imports them back, a legal leaf→leaf
   edge. The kernel's own unit **classifies as a leaf**, and that is a required
   exit criterion of this slice, not a happy consequence.

   **Two review passes widened what goes to the payload.** The first (design
   `D10`) found two backward references in `verify_over`'s *body*, where the
   three earlier enumerations had not looked: the `/bin/sh` precondition, which
   is a fact about the payload's probes, and `host_descriptor()`, which reads
   disk. Both move out, and with them the last use of `&dyn HostFacts`. The
   kernel's entry point therefore takes **values and three closures and nothing
   else** — `BackendId` and `Availability` rather than any backend trait,
   `HostDescriptor` rather than a host trait plus a disk read.

   The second (design `D12`) found a **sixth** reference, and the first in a
   *field* rather than a signature or a body: `ArmResult::Indeterminate` carries
   `termination: Termination` (`backend.rs:751`), which `row_verdict` never
   reads. The kernel takes a projected `ArmJudgement` instead and the payload
   keeps `ArmResult`. That finding also falsified the *machine check* the design
   had named — the layering gate proves tier direction, not mechanism
   neutrality, and `backend` is itself `leaf`, so a `backend`-resident type walks
   straight through it. `DEC-197`'s placement rider was re-cut in response, and
   the check is now a `harness = false` compile probe that builds the kernel as a
   synthetic crate with **no fake module** — which is why the two types move
   rather than being imported.
2. **Publish the profile instead of collapsing it — MOVED to `SL-257`**
   (*Authority floor and assurance profile*), 2026-08-17. `DEC-195`'s closed
   authority floor and unreduced per-row assurance profile — with `D7`'s
   reading-versus-standing split and `CPT-002`'s escape-front labelling — do not
   need the extraction. The floor wants closedness, and while one mechanism
   exists the profile keys on today's `Property`; `DEC-198`'s open assurance key
   is what needs a second mechanism, and it defers here. `EVD-021`'s vacuous
   admission is live on the backend that ships, so the fix goes ahead of this
   slice rather than behind it. This slice `needs` `SL-257`.
3. **Apply `DEC-194`'s rename** across the extracted surface —
   `QualificationVerdict`, `Qualification`, the verb `backend qualify`, exits
   `EXIT_QUALIFIED` / `EXIT_DISQUALIFIED`. This closes the existing
   `EXIT_REFUSED` / `NotAdmitted` mismatch in passing — a mismatch `SL-257`
   deliberately leaves standing, because `DEC-194`'s own justification is that
   the rename is free *only* while these types are moving anyway. Note that
   `Qualification`'s variants are `Unavailable` and `Ran`, **not** `Qualified` /
   `Disqualified` as this objective first read: `DEC-194` named the axis, and
   design `D4` then ruled that the summary word is *computed at the command tier
   from `floor.standing()`* rather than stored, because a stored scalar can
   disagree with the rows it was computed from — which is `DEC-191`'s original
   complaint in miniature. `SL-257` lands that shape under the pre-rename names,
   so this objective renames a structure that already exists rather than
   introducing one.
4. **Isolate the bubblewrap payload** behind the kernel's seam, unmigrated and
   unported, so what is namespace-shaped is visibly namespace-shaped.
5. **The `REV` against `REQ-459` — MOVED to `SL-257`**, 2026-08-17, as
   `DEC-201`'s single four-payload revision, landing there with the floor and
   profile. `DEC-189` falsifies criterion 3's premise whether or not any code
   moves, so the spec correction does not wait on the extraction.
6. **`REV-051`'s criterion-3 disposition — MOVED to `SL-257`** as payload 3 of
   that same REV. `DEC-201` refused splitting the payloads and they are not
   split: all four move together, which is what keeps the window closed.

   **One amendment rides the move, and it lands back on this slice.** `DEC-201`
   assumed the narrowed criterion 3 landed *discharged*, because `DEC-198`'s open
   assurance key would have shipped beside it. Deferring that key means a second
   mechanism would still have to edit the enum to publish its own rows, so
   `SL-257` records criterion 3 as **undischarged**, and **discharging it is an
   exit criterion of this slice** — see § Verification.
7. **Carve the test bands before the split** (`DEC-200`). The 186 test functions
   sit in one flat `#[cfg(test)] mod tests` with no inner module declaration at
   all, and the crate is bin-only by declared intent, so they cannot move to a
   `tests/` directory and must follow their code. They are sub-moduled into two
   bands — kernel and payload — as a pure reorganisation against stable types,
   *before* any code moves, so the split then moves whole sub-modules instead of
   rewriting a test file. Roughly 67 of the 186 need individual judgement. This
   is real work the phase plan carries explicitly rather than absorbs.

### Affected surface

- `crates/doctrine-control/src/conformance.rs` — the split's subject. The
  AND-reduction is `admission` at `:5166`, with one production call site
  (`:5326`); the verdict types are `:2682-2838`.
- `crates/doctrine-control/src/main.rs` — `run_backend`, `run_backend_verify`,
  `admit` (`:153`), `render_verdict` (`:178`), `render_outcome` (`:217`), and the
  exit constants (`:60,63`). `admit` and `render_outcome` are the only two
  production consumers of the collapsed scalar, and are exactly what `DEC-191`
  changes. `render_verdict` emits `date=` on the verdict's header line, which is
  why the `today` → `observed_at` rename is **source-side only** (design `D9`):
  the header line survives the split byte-identical under `DEC-199`'s
  transformation contract, so re-spelling one of its keys is a difference nothing
  derives. `render_verdict` also emits one flat `row {id:?}={row:?}` line per row
  with no grouping structure — the shape the contract's rules 3, 4 and 5 derive
  the post-split output from.
- `crates/doctrine-control/src/backend.rs` — `BackendId` (`:811`) and
  `Availability` (`:780`) **move out**, into the kernel (`DEC-197`, re-cut
  2026-08-12), and `backend.rs` imports them back. The draft's competing fear —
  that moving would invert an `ADR-001` edge and cascade into `transaction` and
  `provision` — was a misreading of which direction the edge runs: the kernel is
  a `leaf`, so `backend` importing it is a leaf→leaf edge this tree already has
  several of. Import-line churn lands in five files: `backend.rs`,
  `backend/bubblewrap.rs`, `transaction.rs`, `main.rs`, `conformance.rs`.
- `.doctrine/spec/tech/030/` and `REQ-459` — via the `REV`, not by direct edit.
- `.doctrine/adr/001/layering.toml` — `:257` carries the literal `backend verify`
  and is the *only* accepted-governance file in `DEC-194`'s rename radius. The
  file **gains one row** for the kernel unit, classified `leaf`; `conformance`,
  now the payload, keeps `engine`. Nothing existing is re-classified (`DEC-197`).
- `justfile` — `capsule-check` (`:111-113`) and `capsule-verify` (`:130-146`).
  Neither is wired into `check` or `gate`, so this slice's proof does not run
  under the default gate and verification design must say how it is run.
  `capsule-check` **gains a leg** (`RV-354` `F-2`): the compile probe's negative
  control, which inverts the exit status of a second `harness = false` target and
  asserts the diagnostic set is exactly `E0433` naming the forbidden module.
- `crates/doctrine-control/Cargo.toml` — two `[[test]]` targets for the compile
  probe and its negative control, the latter behind `required-features` so
  `cargo test` never builds it.
- `scripts/` — a **new one-shot transform script** (`RV-354` `F-3`), stdlib-only
  Python in the `migrate_value_facets.py` mould. It reads `EVD-022`'s pre-split
  transcript, applies `DEC-199`'s rules 1–7 plus the ordering clause, and emits
  the expected post-split artefact, which is committed **as** the golden. Runs
  before the phase that changes `RowId`; disposable after the split lands.

### Risks and assumptions

- **The behaviour-preservation gate and `DEC-191` pull against each other —
  SETTLED by `DEC-199`.** `AGENTS.md` requires the existing suites to stay green
  *unchanged* when shared machinery moves, and `DEC-190` names `RV-352`'s
  baseline as the bar, but `DEC-191` deliberately changes what the verdict
  renders — and three shape changes taken in design (`DEC-196`, `DEC-197`,
  `DEC-198`) edit test source that names the moved types, so *green unchanged*
  cannot be literally true. The bar is stated in three layers: the **nineteen row
  verdicts**, four auxiliary claims and two unrowed readings reproduce exactly;
  the artefact's text is governed by a **closed transformation contract**; test
  source naming moved types is reviewed as a translation diff in which no
  asserted value moves. `EVD-022` is the pre-split half of the bracket, captured
  before any code lands and uncapturable later.

  **Layer 2 was re-cut on 2026-08-12** (`DEC-199`), and the shape matters for
  planning. It was an enumerated list of three permitted differences; the second
  review pass showed that form cannot express what the split does — a front label
  is new content on a line rather than a re-spelling, and the floor row leaves the
  flat row list entirely, so its line has no successor at all. A deletion is not
  an exception to a list of exceptions. Layer 2 is now a total map from the
  pre-split artefact's lines to the post-split artefact's, held by two totality
  clauses (every pre-split line has exactly one successor; every post-split line
  has exactly one predecessor), seven derivation rules, and stated ordering and
  exit-code clauses. Its instrument is a **whole-output golden test** committed in
  the phase that changes row identity, which is also why the contract must be
  written *before* that phase.
- **"Not ranked" must not become "nothing can fail." — SETTLED by `DEC-195`.**
  `DEC-191` left the admission floor unset and `RFC-025` flagged it directly:
  that would be `ISS-341`'s defect family a fourth time. `EVD-021` showed the
  vacuous path is already reachable and held shut only by a guard `DEC-189`
  dissolves. The floor is now closed, reduced by exhaustive match rather than
  `.all()` over a `Vec`, so the vacuous path is unrepresentable rather than
  externally guarded.
- **Thirteen of fourteen `Property` members become payload constants**
  (`DEC-198`) — recorded as an explicit residual, not a hidden cost. A hypervisor
  publishes none of rows 10/12/13/14 and mints its own, editing nothing in the
  kernel; that is the point, but the migration is real.
- `crates/doctrine-control` is outside every default gate selection — Linux-only,
  live-`bwrap` rows — and is reached by `just capsule-check`, not `just gate`.
  Verification design must account for that or this slice's proof does not run.
  **Both** instruments need `bwrap`, not only `capsule-verify`: `capsule-check`
  runs `cargo test -p doctrine-control`, whose suites assert
  `availability() == Available` and provision real capsules, and `EX-14` forbids
  them skipping. Nested `bwrap` works in the project jail, so the proof runs
  here; a host without `bwrap` has no instrument at all until `IMP-427` lands.
- **Four enumerations of what crosses the seam backwards have been wrong** —
  `DEC-196`'s, the draft's, and each of the two review passes'. The list has gone
  from three to five to six, across three distinct location classes: a parameter
  list, a function body, and a *field* of a parameter's type. Each class was
  found only after the previous had been closed, and every enumeration was
  careful and believed complete at the time. Carried as a force rather than a
  risk (design `F7`), because the answer has to be structural: the kernel's entry
  point removes the parameters such a reference arrives on (`I10`), and the
  compile probe catches what rides inside a value's fields (`I7`) — neither is
  the design promising to look harder. **The right prior for a further pass is
  that there is a fourth location class.** The residual — a kernel that re-derives
  a mechanism fact from `std` alone, or names an external dependency of the
  `doctrine-control` package — is caught by no gate here and is stated as such.
- The pure/imperative split is a **target** for this slice, not an inheritance.
  `verify_over` reads `/proc/sys/kernel/osrelease` from disk today; the split is
  where the constraint starts holding.
- Assumes `ADR-020` is not reopened. `DEC-191` was constructed to land without
  doing so, and step 2's *"same authority properties"* sits entirely on the floor.

### Open questions

- **`OQ-1` — Firecracker row set: implement or derive? — RESOLVED narrow**
  (owner, 2026-08-12). Derive the membership as a design artefact the kernel is
  shaped to admit; implement nothing against a backend that does not exist.
  There is no Firecracker backend in `doctrine-control` — the microVM work is in
  `/workspace/microvm-spike` and has not graduated. The wide reading would have
  pulled a backend implementation in and roughly tripled the slice. Retained
  here rather than deleted because the reasoning binds later phases: the kernel
  must be *shaped* by `DEC-189`'s membership analysis without *implementing* it.
- **`OQ-2` — where does the admission floor sit** once the AND-reduction is
  gone? **CLOSED by `DEC-195`** (2026-08-12). Promoted from open question to
  blocking design decision, carried as `inq-1`/`inq-2` on the design run's
  inquiry map, and settled first rather than last. The floor is a closed set of
  one member reduced by exhaustive match. Retained here rather than deleted
  because `EVD-021`'s reasoning — the vacuous path is real, and its only guard is
  external and per-suite — binds the implementation, not just the decision.

### Verification and closure intent

Done is: the kernel is separable and reviewable without loading a confinement
mechanism, and **its unit classifies `leaf` and the architecture gate passes**
(`DEC-197`) — necessary but not sufficient, so the kernel's entry point taking
**values and closures only** (design `I10`) is a second, reviewed exit criterion
alongside it — and the **compile probe**, *with its negative control*, is a
third, because `I10` governs the parameter list and says nothing about what rides
inside a parameter's type, and because a probe nothing has watched go red is a
claim rather than a check (`RV-354` `F-2`). `I10` itself is narrower than it
reads: design `D13` establishes that the kernel adjudicates rather than receiving
an adjudication, which closes a closure's *return* as a route for authority to
leave the kernel — but a closure's *captured environment* stays invisible to
every instrument here, and a mechanism that misreports its two arms is still
believed. `D13` closes bypass, never fabrication;
`RV-352`'s row-level baseline reproduces unchanged — the nineteen row verdicts
exactly, with the artefact's text derived line-for-line by `DEC-199`'s
transformation contract — while the floor and profile `SL-257` landed survive the
move intact; `backend qualify` replaces `backend verify` with its exits renamed;
the bubblewrap payload is behind the seam and unported; and **`REQ-459` criterion
3, as `SL-257` narrows it, is discharged** — a second mechanism proves the same
floor and publishes its own profile, editing nothing, which `DEC-198`'s open
assurance key is what makes true. That discharge is this slice's, not `SL-257`'s:
`SL-257` records the criterion as undischarged precisely because the open key
defers here.

**How the proof is run** (`DEC-199`), since neither recipe is wired into
`just gate`: `just capsule-check` is the per-phase gate, green at the end of
every phase; `just capsule-verify` is a **phase exit criterion** for every phase
touching the payload, and the default for any phase where it is arguable — only a
phase that plainly cannot reach a row omits it. **Both need `bwrap`** — the
re-cut withdrew this record's earlier claim that `capsule-check` does not, which
had it contradicting the risk stated above.

Four artefacts carry the comparison, all committed: a **key translation table**,
authored in the phase that changes row identity, whose key and front columns are
the transformation contract's rule 4 rather than a reader's aid; a
**characterisation test** recording the row-to-verdict mapping as data, written
before the split and carried through it, so a regression in the *algebra* fails
at the phase that broke it; and a **whole-output golden test** in the same phase
as the table, asserting the post-split artefact verbatim, so a regression in the
*rendering* fails there too. The two tests are not redundant: the first pins
layer 1, the second pins layer 2. And a **one-shot transform script**, which is
what makes the golden's expected value *derived* rather than authored — a golden
proves `actual == expected` and nothing about where `expected` came from, so
without it a migration that drops a line and encodes the same omission into the
golden passes (`RV-354` `F-3`).

Which of the four can be machine-derived is a question with three different
answers, and design § 9.1 carries the taxonomy: the pre-split lines are
observation-backed and must be derived; the characterisation test is
reality-checked, because being written before the split it is asserted against a
reality that already exists; the key translation table is intent-backed, has no
ground truth to derive from, and stays irreducibly reviewer-checked.

## Non-Goals

- **The floor and the profile themselves**, and the `REV` — `SL-257`, per the
  trim. This slice consumes the shape it lands; it does not introduce it.
- **A Firecracker or microVM backend.** Not built here; see `OQ-1`.
- **Migrating the bubblewrap payload to a second mechanism.** `DEC-190` refuses
  it and `DEC-189` explains why porting membership would be actively harmful.
- **A ranking or scoring apparatus.** `DEC-191` refused a rank as firmly as it
  refused equivalence. The fix is to stop collapsing, not to build an ordering.
- **Reopening `ADR-020`.** Authority is the floor and stays where it is.
- **The anti-heresy work `CPT-002` argues is under-weighted.** Named in the
  spec via the `REV` so a reader is not misled; building it is not this slice.
- **`DEC-191`'s front list as a closed enumeration.** It is a sketch. Closing it
  is separate work.

## Summary

Split the 14k-line capsule conformance suite into a small backend-neutral verdict
kernel and an unported bubblewrap payload, cutting at row identity; open the
assurance key so a mechanism nobody has written can mint its own rows without
editing the kernel (`DEC-198`), which is what discharges `REQ-459` criterion 3 as
`SL-257` narrows it; and rename the mechanism axis to qualification while those
types are moving.

Gated on `QUE-217`: every objective here buys portability across mechanisms, and
which mechanisms there will be is undecided. The floor, the profile and the `REV`
were trimmed out to `SL-257` on 2026-08-17 because they do not depend on that
answer.

## Follow-Ups

- The Firecracker row set's implementation, once a backend exists (`OQ-1`).
- `DEC-191`'s front list closed from a sketch into an enumeration.
- `IMP-427` — the third test band, splitting the payload into live-`bwrap` and
  neutral so `just capsule-check` runs meaningfully on a host without `bwrap`.
  Deferred rather than refused (`DEC-200`), and it must never become the skip
  `EX-14` and `DEC-156` forbid.
- `IDE-050` — definition records for bounded-context terms, the general remedy
  for the collision `DEC-194` fixed one instance of.
