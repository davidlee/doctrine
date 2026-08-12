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
2. **Publish the profile instead of collapsing it.** Replace the all-or-nothing
   AND over a fixed row set with a verdict carrying two structures of different
   semantics (`DEC-195`): a closed **authority floor**, reduced by exhaustive
   match over a closed enum so an empty floor is unrepresentable, whose
   membership is row 3 — `DeniedCanonicalStateAndCredentials` — and nothing else;
   and an **assurance profile**, the remaining rows, published per row and never
   reduced. Fronts are open grouping metadata over rows and are **never** a
   reduction target: reducing per front reproduces the vacuity one level up,
   since `DEC-189` guarantees empty fronts exist. Fronts are **escape** fronts
   and the rendering must say so (`CPT-002`), so a strong profile is not read as
   a strong safety claim. **Fronts live wholly in the payload** (design `D8`):
   its table declares each row's front and the command tier consults it when
   rendering, because the kernel neither reduces over fronts nor validates them
   and `DEC-191`'s front list is still a sketch. Authority remains a floor: a
   composition that weakens it is not a weaker posture, it is not a capsule.

   The floor's *reading* and the floor's *standing* are two questions, not one
   (design `D7`): `Floor` stays total so an empty floor is unrepresentable, and
   a wrapper carries *the floor row was never submitted* — which is a state of
   the run, distinct from *the answer was no*, and which the rows that did run
   are still published alongside.
3. **Apply `DEC-194`'s rename** across the extracted surface —
   `QualificationVerdict`, `Qualification`, the verb `backend qualify`, exits
   `EXIT_QUALIFIED` / `EXIT_DISQUALIFIED`. This closes the existing
   `EXIT_REFUSED` / `NotAdmitted` mismatch in passing. Note that `Qualification`'s
   variants are `Unavailable` and `Ran`, **not** `Qualified` / `Disqualified` as
   this objective first read: `DEC-194` named the axis, and design `D4` then
   ruled that the summary word is *computed at the command tier from
   `floor.standing()`* rather than stored, because a stored scalar can disagree
   with the rows it was computed from — which is `DEC-191`'s original complaint
   in miniature.
4. **Isolate the bubblewrap payload** behind the kernel's seam, unmigrated and
   unported, so what is namespace-shaped is visibly namespace-shaped.
5. **Carry the `REV` against `REQ-459`** as a phase of this slice. `REQ-459`
   enumerates one undifferentiated property list with canonical-authority inside
   it, which is exactly the conflation the kernel un-conflates; shipping the code
   without the revision leaves the spec contradicting the binary. `DEC-201`
   settles the REV's shape: **one REV, four payloads**, landing with the code so
   no commit has the spec contradicting the binary. Criterion 1 splits into two
   criteria of different invariance — a floor proven on every mechanism, a
   profile that varies per mechanism. Criterion 3's **text** narrows to *same
   floor, own profile*: `edits nothing` survives, *the same property suite* does
   not. **That narrowing is an explicit widening of this slice's scope, taken by
   the owner** — the objective as first written revised the criteria around
   criterion 3, not criterion 3 itself. `IMP-405`'s platform-versus-mechanism
   rename applies across § Platform backend contract, and `CPT-002`'s threat
   priority lands in `SPEC-030` § **Concerns** — not § Overview.
6. **The same `REV` revises `REV-051`'s criterion-3 disposition** (owner's
   direction, 2026-08-12). `REV-051` is `done` and applied; it records
   `REQ-459` criterion 3 as *"discharged structurally — one suite parameterised
   by backend; a second backend passing it edits nothing."* `DEC-189` contradicts
   that: if row membership is a function of the mechanism's available deltas,
   there is no single parameterised suite for a second backend to pass, and the
   structural discharge does not hold. Shipping the kernel makes an applied
   revision's recorded reading false, so the correction rides this slice rather
   than being left for a reader to notice. It is payload 3 of objective 5's one
   REV, not a second one — `DEC-201` refused splitting them, because the two must
   be true together and separating them opens a window in which they are not.
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
alongside it — and the **compile probe** is a third, because `I10` governs the
parameter list and says nothing about what rides inside a parameter's type;
`RV-352`'s row-level baseline reproduces unchanged — the nineteen row verdicts
exactly, with the artefact's text derived line-for-line by `DEC-199`'s
transformation contract — while the verdict publishes a floor and a profile
instead of a scalar; `backend qualify` replaces `backend verify` with its exits
renamed; the bubblewrap payload is behind the seam and unported; and `SPEC-030`
no longer contradicts the binary.

**How the proof is run** (`DEC-199`), since neither recipe is wired into
`just gate`: `just capsule-check` is the per-phase gate, green at the end of
every phase; `just capsule-verify` is a **phase exit criterion** for every phase
touching the payload, and the default for any phase where it is arguable — only a
phase that plainly cannot reach a row omits it. **Both need `bwrap`** — the
re-cut withdrew this record's earlier claim that `capsule-check` does not, which
had it contradicting the risk stated above.

Three artefacts carry the comparison, all committed: a **key translation table**,
authored in the phase that changes row identity, whose key and front columns are
the transformation contract's rule 4 rather than a reader's aid; a
**characterisation test** recording the row-to-verdict mapping as data, written
before the split and carried through it, so a regression in the *algebra* fails
at the phase that broke it; and a **whole-output golden test** in the same phase
as the table, asserting the post-split artefact verbatim, so a regression in the
*rendering* fails there too. The two tests are not redundant: the first pins
layer 1, the second pins layer 2.

## Non-Goals

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

Split the 14k-line capsule conformance suite into a small backend-neutral
verdict kernel and an unported bubblewrap payload, cutting at row identity; stop
collapsing the row outcomes the verdict already carries, reducing a closed
authority floor and publishing the rest as an assurance profile; rename the
mechanism axis to qualification while those types are moving; and revise
`SPEC-030` `REQ-459` so the spec stops conflating the authority floor with
confinement strength.

## Follow-Ups

- The Firecracker row set's implementation, once a backend exists (`OQ-1`).
- `DEC-191`'s front list closed from a sketch into an enumeration.
- `IMP-427` — the third test band, splitting the payload into live-`bwrap` and
  neutral so `just capsule-check` runs meaningfully on a host without `bwrap`.
  Deferred rather than refused (`DEC-200`), and it must never become the skip
  `EX-14` and `DEC-156` forbid.
- `IDE-050` — definition records for bounded-context terms, the general remedy
  for the collision `DEC-194` fixed one instance of.
