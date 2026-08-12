<!-- doctrine:section sec-1 -->
## 1. Design Problem

`crates/doctrine-control/src/conformance.rs` is 14,252 lines in one file, and it
is correct. `RV-352` records two independent gate runs at `297 passed; 0 failed`,
with all fourteen properties and five freshness axes `Proven`. Nothing here is a
bug hunt.

The problem is **cost of opinion**. Forming a defensible view about one row today
means loading the production backend, the fixture, the placement validator and
several phases of `SL-248`'s authored plan. `RSK-231` asks whether that surface
can be reduced so correctness is checkable without loading the whole design. It
can, because the file is two things wearing one name: a small body of
backend-neutral judgement — what a row verdict *means*, what the suite is
entitled to conclude from a pair of arms — and a large body of bubblewrap-shaped
construction that produces the inputs that judgement runs on.

Four accepted decisions converge here and none of them lands alone. `DEC-190`
splits the file. `DEC-191` stops the final AND-reduction, because collapsing
nineteen row outcomes to one word destroys the only information a reader needed.
`DEC-194` renames the mechanism axis to *qualification*, and must land with the
split because these types move anyway. `DEC-189` refuses to port row *membership*
across mechanisms, because four of the fourteen rows lose their host-facing delta
under a hypervisor and a microVM earns rows bubblewrap never could.

Stated as one question, the design problem is:

> **Where does the seam cut, such that the neutral half is small enough to hold
> in the head, the mechanism-shaped half is visibly mechanism-shaped, and the
> contract between them binds a mechanism nobody has written yet — while the
> nineteen row verdicts the current suite produces are demonstrably unchanged?**

Three things make that harder than a file split:

1. **The verdict's shape is contested, not given.** `DEC-191` removes the
   reduction but leaves the admission floor unset, and `EVD-021` shows the
   existing reducer already admits an empty row list — `.all()` over an empty
   slice is vacuously true, and the only guard is an external assertion, in one
   test, that one fixed `tables()` is non-empty. `DEC-189` dissolves that guard
   by making membership per-mechanism. So the floor cannot be inherited from
   either decision; it has to be constructed.
2. **Row identity carries two incompatible requirements.** The floor must be
   closed, so it can be reduced by exhaustive match. The profile must be open, so
   a mechanism can mint rows the kernel has never heard of. Today one closed
   fourteen-member `Property` enum is asked to be both.
3. **Behaviour preservation and `DEC-191` contradict each other on their face.**
   `AGENTS.md` requires existing suites green *unchanged* when shared machinery
   moves; `DEC-191` requires the output to change. No inherited record reconciles
   them, and the reconciliation is not cosmetic — it decides what a red gate
   means for the rest of this slice.

The design that answers this is recorded across `DEC-195` through `DEC-201`. This
document states the resulting system, not the route to it.

<!-- doctrine:section sec-2 -->
## 2. Current State

All line numbers are against the working tree at `94d0b5603` and were read
directly, not recalled. Two review passes have now re-read every one of them:
the first found two wrong (`RF-8`) and the second found two more (`RF-14`). All
four are corrected here.

### 2.1 What the unit contains

One file, one `#[cfg(test)] mod tests` at `:5339` running flat to EOF with **no
inner module declaration at all**, and 186 `#[test]` functions inside it. The
crate is bin-only by declared intent, so tests cannot move to a `tests/`
directory — they live inside the unit they test and must follow it.

The neutral material measures at roughly 663 lines types-only, ~1,072 with the
assembly around it. That is a measurement of the current shape and nothing more
— it is not a target, and the kernel's size is decided by what earns its place
(§ 4, `P1`), never by arriving at a number.

### 2.2 The judgement core, as it stands

```rust
// :3177 — the whole of the two-arm algebra
fn row_verdict(probe: ArmResult, control: ArmResult) -> RowVerdict {
    match probe {
        ArmResult::Indeterminate { reason, .. } => RowVerdict::Indeterminate {
            arm: Which::Probe, detail: reason },
        ArmResult::Failed => RowVerdict::Violated,
        ArmResult::Held => match control {
            ArmResult::Indeterminate { reason, .. } => RowVerdict::Indeterminate {
                arm: Which::Control, detail: reason },
            ArmResult::Held => RowVerdict::Unproven,   // control did not fire
            ArmResult::Failed => RowVerdict::Proven,   // probe held, control fell
        },
    }
}
```

This function is the design's centre of gravity and it names nothing
mechanism-shaped. `Unproven` — probe held and control *also* held — is the
`B4` defect class the round-6 split exists to expose: a control that cannot fire
proves nothing.

Note the two `..` patterns. The algebra reads `reason` and discards everything
else `ArmResult::Indeterminate` carries; § 2.4's sixth backward reference is
inside what those dots throw away.

The verdict it feeds, at `:2689`:

```rust
pub(crate) struct AdmissionVerdict {
    pub(crate) backend: BackendId,
    pub(crate) host: HostDescriptor,
    pub(crate) date: String,
    pub(crate) outcome: Admission,                        // the scalar DEC-191 removes
    pub(crate) rows: Vec<(RowId, RowVerdict)>,            // the information already here
    pub(crate) auxiliary: Vec<(Claim, AuxOutcome)>,       // table C: reported, never admitted on
    pub(crate) observations: Vec<(Unrowed, Reading)>,     // read, no verdict at all
}
```

Four evidential tiers, deliberately distinct: rows carry verdicts and are
admitted on; axes are rows by another key; table C claims have outcomes but never
reach admission — structurally, because `admission` is *not given them*; and
`Unrowed`/`Reading` pairs have no outcome slot anywhere, so attaching a verdict to
one is a type change a reader sees in the diff.

`HostDescriptor` is defined here too, at `:2803` — three `String` fields, `os`,
`kernel` and `arch`.

### 2.3 The reduction, and the vacuous path

```rust
// :5166 — one production call site, at :5326
fn admission(rows: &[(RowId, RowVerdict)]) -> Admission {
    if rows.iter().all(|(_, verdict)| matches!(*verdict, RowVerdict::Proven)) {
        Admission::Admitted
    } else {
        Admission::NotAdmitted { reason: NotAdmitted::Rows }
    }
}
```

`admission(&[])` returns `Admitted`. This is known and deliberately unrepaired:
the test at `:11275` is named
`an_empty_row_list_is_admitted_and_the_shipped_tables_are_what_prevent_it`, and
its second assertion is `!tables().is_empty()` with the comment *"the shipped
tables are what stop the vacuous path being reachable"* (`EVD-021`). The guard is
therefore **external to the reducer** and depends on there being one fixed table
set. `DEC-189` makes membership a function of the mechanism, so that dependency
is about to stop holding.

`Admission` has exactly one green path by invariant 1. `NotAdmitted::Unavailable
{ missing, remedy }` is `POL-002` facet 3's descriptive absence, read from
`CapsuleBackend::availability` *before any row runs*.

### 2.4 The seam that already exists, and where it leaks

`verify_over` at `:5270` is already an injected seam — row set, auxiliary
closure, observation closure and row runner are all parameters, which is what
makes the algebra testable without a fixture:

```rust
fn verify_over(
    backend: &dyn ConformanceBackend,
    host: &dyn HostFacts,
    today: String,
    rows: &[Row],
    auxiliary: &dyn Fn() -> Vec<(Claim, AuxOutcome)>,
    observations: &dyn Fn() -> Vec<(Unrowed, Reading)>,
    run_row: &dyn Fn(&dyn ConformanceBackend, &Row) -> RowVerdict,
) -> AdmissionVerdict
```

**Six references point the wrong way for a split, not the three `DEC-196`
enumerated.** Three sit in `verify_over`'s signature, two in its body, and the
sixth in a *field* of a type the other kernel-bound function takes. Those are
three distinct location classes, and each was found only after the previous one
was closed — which is `F7`'s evidence and the reason § 5.1 stops relying on
enumeration as the safeguard:

- `rows: &[Row]` — `Row` owns `Delta`, which owns bubblewrap's removal
  vocabulary. A neutral function taking `&[Row]` drags the mechanism across.
- `backend: &dyn ConformanceBackend` — `ConformanceBackend` at `:638` names
  `PropertyRemoval`, `AuthorityGrant` and `Under` in every method past
  `as_capsule_backend`, all three mechanism-shaped. But `verify_over` itself calls
  only `id()` and `availability()`. (Found while working `inq-4`, after
  `DEC-196` had claimed the enumeration complete.)
- `Delta::Widened(fn(&Fixture) -> Vec<MountedPath>)` at `:2548` — a function
  pointer taking the bubblewrap fixture, sitting inside the type the neutral half
  would otherwise keep.
- **The shell precondition at `:5300`** — `if !host.path_exists(Path::new(SHELL))`,
  returning `Unavailable { missing: SHELL, remedy: SHELL_REMEDY }`. `SHELL` is
  `"/bin/sh"` at `:100` and the code's own comment (`:5298-5299`) says why it is
  tested: *"Every payload runs under `/bin/sh -c`."* That is a statement about the
  payload's probes, made inside the function that would become the kernel.
  (Found by the first review pass, `RF-3`.)
- **The host-descriptor derivation at `:5280`** — `host_descriptor()` at `:2825`
  reads `KERNEL_RELEASE` (`/proc/sys/kernel/osrelease`, `:491`) from disk. Not
  mechanism-shaped, but **I/O**, called from inside the would-be kernel and
  ignoring the `&dyn HostFacts` sitting in the parameter list beside it. (Found
  by the first review pass, `RF-9`.)
- **`ArmResult::Indeterminate`'s `termination` field at `:848-853`** — the arm
  result carries `termination: Termination` beside `stdout` and `stderr`, and
  `Termination` is `backend.rs:751`, *"how a run ended"*, the same field
  `Observation` (`backend.rs:770`) carries as the backend's own report of a run.
  `row_verdict` is the second function bound for the kernel and takes `ArmResult`
  on its parameter list, so this reference arrives through a type's **field** —
  neither a signature nor a body, and never looked at. `row_verdict` does not
  even read it (`:3177` matches `Indeterminate { reason, .. }`). It sits beside
  an internal inconsistency worth naming: `Indeterminacy::BackendError`
  deliberately launders `BackendError` to a `String` (`:881-884`) *"so the
  verdict carries a rendering the backend cannot later change under it"* — the
  sibling field escaped the same discipline. (Found by the second review pass,
  `RF-10`.)

`verify`'s fixture `OnceCell` at `:5217` was named as a further obstacle and is
not one: `run_row` at `:5003` already takes `&Fixture` explicitly, so placing
`verify` and `run_row` on the mechanism side dissolves it with no restructuring.

### 2.5 Row identity

```rust
pub(crate) enum RowId {            // :2580
    Property(Property),            // one per SPEC-030 enforcement channel
    Axis(Axis),                    // REQ-450 criterion 1's five freshness axes
}
```

`Property` is a closed fourteen-member enum, ordered as table A is. Its doc
comment states the load-bearing fact plainly: because `RowId::Property` keys the
verdict, *a row the suite can construct that this enum cannot name is a compile
error* — and that is the one machine-checked projection of table A, and the whole
of it. Everything else about the table is reader discipline.

Row titles mislead, and this cost real time during research:
`TrustedTerminationObservation` (row 8) reads epistemic and is a **file-size
resource bound**, enforced via `RLIMIT_FSIZE` on the child. Read a row's `delta`,
never its name.

### 2.6 Consumers and how the suite is reached

`crates/doctrine-control/src/main.rs` holds the only two production consumers of
the collapsed scalar — `admit` (`:153`) and `render_outcome` (`:217`) — and they
are exactly the two functions `DEC-191` changes. Exit constants at `:60,63`.
`render_verdict` (`:178`) builds the artefact's header line, which carries
`backend=`, `os=`, `kernel=`, `arch=` and **`date=`** — the last of these is why
§ 5.3's rename is source-side only. It then emits one flat
`row {id:?}={row:?}` line per row, with no grouping structure of any kind; § 5.4
and § 9.1 are where that output's successor is constrained.

`BackendId` lives at `backend.rs:811` and `Availability` at `:780`; `DEC-197`'s
re-cut moves both **into** the kernel, which is why § 5.1's edge argument runs
the opposite way from the draft's. `layering.toml` classifies `backend` as
`leaf` (`:261`) and `conformance` as `engine` (`:264`); `:257` carries the literal
`backend verify` and is the only accepted-governance file in `DEC-194`'s rename
radius. Leaf→leaf edges are already normal in this tree — `backend → config`,
`capacity → config, host` (`:259-261`) — so a leaf importing a leaf introduces no
new edge class in either direction.

The crate sits **outside every default gate selection**: `just gate`'s `test-all`
names its packages (`-p doctrine -p cordage`) rather than using `--workspace`, and
`doctrine-control` is Linux-only with live-`bwrap` rows (`RV-353` `F-4`). It is
reached by `just capsule-check` (`justfile:111-113`) and `just capsule-verify`
(`:130-146`), neither of which is wired into `check` or `gate`. **Both need
`bwrap`**: `capsule-check` runs `cargo test -p doctrine-control`, whose
`#[cfg(test)]` suites include tests that assert
`backend.availability() == Availability::Available` and provision real capsules
(`conformance.rs:6915`, `:7244`, `:7453`), and `EX-14` forbids them skipping
instead (`RF-4`). Nested `bwrap` works inside the project jail, so both are
runnable on the development host — `ISS-339`'s never-run-off-jail note is not a
blocker here, and `EVD-022` is the run that proves it.

<!-- doctrine:section sec-3 -->
## 3. Forces & Constraints

### 3.1 Governing authorities

`SL-253` is `governed_by` five records. The last three were found by this design
run's surface triage and linked before the governing context was confirmed, so
the confirmation covers them.

| authority | what it binds here |
|---|---|
| `ADR-020` | The authority floor, **unamended**. Step 5 admission is the control plane's alone, so the verdict may name that axis but must reach no conclusion on it. |
| `ADR-001` | Module layering: `engine` imports `engine + leaf`, no cycles. Decides where the kernel unit may sit and what it may import. |
| `STD-001` | No magic strings. The verb literal and the exit constants stay single-source through `DEC-194`'s rename. |
| `POL-002` | Facet 3: `Unavailable { missing, remedy }` is descriptive absence and survives the rename intact. |
| `RFC-025` (`concerns`) | Flags that *"not ranked"* must not become *"nothing can fail"* — `ISS-341`'s defect family a fourth time. |

Four inherited decisions — `DEC-189`, `DEC-190`, `DEC-191`, `DEC-194` — bind as
premises rather than as governance, and `DEC-156`'s one-property-removed control
discipline binds the shape of every row.

### 3.2 The forces, and where they pull against each other

**F1 — Closed versus open, on one type.** The floor must be closed so it can be
reduced by exhaustive match; adding a member should fail to compile, not pass
silently. The profile must be open so a mechanism can mint rows the kernel has
never heard of, because `REQ-459` criterion 3 says a second backend is admitted
by *passing* the assertions and never by *editing* them. `Property` is asked to be
both and cannot be. **Resolved by `DEC-198`** — two keys, not one.

**F2 — Vacuity survives any reduction over a collection that can be empty.**
`EVD-021` is the concrete instance at the row level. Reducing per *front* instead
does not escape it: `DEC-189` guarantees empty fronts exist, and a front carrying
no rows is vacuously strong. The vacuity is a property of reducing over something
that can be empty, not of the axis reduced on. **Resolved by `DEC-195`** — the
floor is a closed enum reduced by exhaustive match, so an empty floor is
unrepresentable rather than externally guarded.

**F3 — Green-unchanged versus deliberately-changed output.** `AGENTS.md`'s
behaviour-preservation gate and `DEC-191` cannot both be read literally, and three
shape changes this design takes (`DEC-196`, `DEC-197`, `DEC-198`) each edit test
source naming the moved types, so *green unchanged* is not merely awkward — it is
false. **Resolved by `DEC-199`** — the bar drops to the row verdict, and what is
permitted to differ is fixed in advance and closed. Its being *closed* is what
makes it a constraint rather than a licence, and § 5.3's source-only rename is
the first thing it decided (`RF-2`). `RF-11` then showed the first form of that
licence could not express what the split does to the row block at all; § 9.1
carries the repair.

**F4 — The proof does not run under the default gate.** `just gate` names its
packages and `doctrine-control` is not among them, so a phase can be green and
prove nothing about this slice. **Resolved by `DEC-199`** — `capsule-check` per
phase, `capsule-verify` as a phase exit criterion. Neither runs without `bwrap`
(`RF-4`), which bounds where the proof can be run rather than whether it is.

**F5 — Derivation without implementation.** `OQ-1` was resolved narrow by the
owner: derive the Firecracker row membership, build no backend. A derivation that
constrains nothing checkable is decoration. **Resolved by `DEC-198`** — the open
assurance key is exactly what a hypervisor needs and the kernel cannot supply, so
the derivation is discharged by a type rather than by prose.

**F6 — A rename that must not become a second migration.** `DEC-194`'s
qualification rename touches the same types the split moves. Landing it
separately means migrating twice. It rides this slice.

**F7 — Inspection has failed four times on the same question.** *What crosses
the seam backwards?* was enumerated by `DEC-196`, then by the draft, then by each
of two review passes. Every enumeration was careful, every one was believed
complete at the time, and every one was short: the list has gone from three to
five to six (§ 2.4). Three distinct location classes have surfaced so far — a
parameter list, a function body, and a *field* of a parameter's type — and each
was found only after the previous had been closed. A force rather than a risk,
because it constrains the design's shape and not merely its verification: any
answer that depends on someone reading the file correctly has already failed
here, four times. **Resolved in § 5.1, in two parts.** Structurally, the kernel's
entry point admits values and closures only, so that reference class has no way
in rather than needing to be found. Mechanically, `DEC-197`'s compile probe —
because `RF-10` showed the structural half does not reach inside a value's
fields, and the layering gate it was paired with proves tier direction rather
than mechanism neutrality.

### 3.3 Hard constraints

- **No `unsafe` added in the kernel.** `ADR-021` is `proposed`, so its two-site
  budget carries directional weight only, but the kernel is judgement over
  values and has no call for it regardless.
- **Pure/imperative split — a target here, not an inheritance.** No clock, rng,
  git or disk in the kernel. The draft claimed the existing design already
  honours this, citing `today: String` and `&dyn HostFacts`; that is wrong
  (`RF-9`). `verify_over` calls `host_descriptor()`, which reads
  `/proc/sys/kernel/osrelease` from disk, and ignores the `HostFacts` in its own
  parameter list while doing so. The split is where the constraint starts
  holding, and § 5.3 says how.
- **Bin-only crate.** `publish = false` and the header's sealing note. The kernel
  is a sub-module, not a crate; tests stay `#[cfg(test)]` inside their unit.
- **`PHASE-NN` and `EN-`/`EX-`/`VT-` ids are immutable.** Edits append.
- **The REV lands with the code.** `DEC-201`: no commit may have `SPEC-030`
  contradicting the binary.

### 3.4 Explicitly not constraints

- **`ADR-020` is not reopened.** `DEC-191` was constructed to land without it.
- **`DEC-191`'s front list is not closed here.** It is a sketch; closing it is
  separate work, and the design must not depend on its membership. This is what
  keeps fronts out of the kernel entirely (§ 5.3, `D8`).
- **No line-count budget on the kernel.** `DEC-190`'s *"a few hundred lines of
  policy"* states a proportion, not a criterion, and a measured figure (§ 2.1) is
  not one either. What belongs in the kernel is settled by `P1` — proportionality
  to value — and a size is the consequence, never the test.

<!-- doctrine:section sec-4 -->
## 4. Guiding Principles

Six principles, each earning its place by deciding something later in this
document. `P1` governs the rest: it is the one that decides what the others are
applied to.

### P1 — Proportionality to value

**What belongs in the kernel is settled by what it buys, not by how much of it
there is.** A type earns its place there if it is what a reader must load to form
a defensible opinion about a row — that is `RSK-231`'s complaint and the reason
this slice exists. A type that a reader can defer belongs on the other side of
the seam, however small it is; a type that a reader genuinely needs stays,
however large.

This is stated first because the alternative is seductive and wrong. `DEC-190`
says *"a few hundred lines of policy and a bonfire for the remaining 13,500"*,
and § 2.1 measures the current neutral material at ~663 lines types-only. Neither
is a criterion. Read as a target, a number invites the two failures that would
cost this slice its point: trimming the classification and verdict algebra —
exactly what `DEC-191` needs and what a reader most needs — to reach it; or
declaring victory at a line count while the seam still leaks. The right question
at every boundary call in § 5 is *what does keeping this buy a reader, and what
does it cost them*, and a size is the consequence of answering it, never the
test.

`P1` is a review check and not a machine one, which is why § 5.1 states a
*positive* placement criterion beside it. Proportionality tells a reviewer what
to weigh; it does not tell them where the boundary falls, and a principle that
only ever argues against inclusion will eventually be argued past.

The same lens applies to the work itself. `DEC-200`'s test carve costs real
judgement across ~67 tests and is taken because the alternative spreads those 67
calls thin across phases where each is made while doing something else.
`DEC-199`'s characterisation test costs a throwaway scaffold and is taken because
it moves a failure from audit time to the phase that caused it. Neither is
justified by being cheap; both are justified by what they return.

### P2 — Make the wrong state unrepresentable, rather than guarded

`EVD-021` is the worked example: the vacuous-admission path is held shut by an
assertion in a *test*, about a *different* function's return value. That guard is
correct today and dissolves the moment `DEC-189` lands. The design's answer is
not a better guard but a shape in which the failure cannot be written down — a
closed enum reduced by exhaustive match has no empty case to be vacuously true
about, and adding a floor member fails to compile.

Applied again at the payload boundary: no kernel type names `Fixture`, so
§ 2.4's `Fixture`-mediated references — `rows: &[Row]`, since `Row` owns `Delta`,
and `Delta::Widened`'s fixture-typed function pointer — vanish by construction
rather than by re-plumbing. **No count is stated here, deliberately** (`RF-13`):
§ 2.4's has moved three times, and a principle is the wrong place to pin a
number that keeps being wrong.

### P3 — Openness is a contract with a mechanism nobody has written

`BackendId` is the precedent already in the tree: an open, mechanism-minted
`&'static str` newtype, open precisely because the contract must bind backends
that do not exist yet. The assurance key is the same construction for the same
reason. Where the kernel must name something a future mechanism owns, it defines
the *shape* of the name and never the membership.

The converse is equally binding: where the kernel must **reason** about something
— the floor — openness is a defect, because there is nothing to exhaustively
match over.

A corollary the design run had to learn the hard way, and which `DEC-197`'s
re-cut now rests on: openness decides a type's *construction*, never its
*ownership*. The consumer of a cross-mechanism contract owns the contract's
shape, and mechanisms implement it.

### P4 — Publish the evidence; reduce only what must be reduced

`DEC-191`'s complaint is that collapsing nineteen row outcomes to one word
destroys exactly the information a reader needs. The design keeps two structures
with different semantics rather than one with a mode flag: a floor that *is*
reduced because a floor is a claim, and a profile that is *published per row*
because a profile is evidence. Fronts group rows for a reader; they are never a
reduction target (F2).

Where the suite is not entitled to a conclusion, it says so by having nowhere to
put one — `Unrowed`/`Reading` has no outcome slot, and admission carries no
outcome field at all because `ADR-020` gives that axis to the control plane.

### P5 — Preserve the claim, not the bytes

The bar for a refactor of shared machinery is normally *suites green unchanged*.
Here that is unachievable and asserting it would let the slice redefine its own
bar silently (F3). So the invariant is stated at the level the suite actually
measures — the nineteen row verdicts, four auxiliary claims and two unrowed
readings — and everything permitted to differ is fixed **in advance**, so the
licence is closed rather than open and cannot be widened after the fact to fit
whatever the code did. A difference the licence does not cover is a regression,
full stop. § 9.1 carries the licence itself.

This principle is what makes `EVD-022` load-bearing: the pre-split half of the
bracket is uncapturable after the first line of code lands.

### P6 — Ride the seam that exists

`verify_over` is already an injected seam, `run_row` already takes its fixture
explicitly, and `conformance` already declares a layering edge to `backend`. The
design's job is to re-key and narrow what is there — not to invent a new
boundary beside it. Concretely: no generics ceremony for a second backend `OQ-1`
has ruled out building, no second name for `BackendId` anywhere (which is why
`DEC-197` moves the type rather than re-exporting it), and no re-classification
of any existing layering unit.

The same instinct applies to the tests. They are carved into bands *before* the
split, against stable types, so the split moves whole sub-modules instead of
rewriting a test file (`DEC-200`).

<!-- doctrine:section sec-5 -->
## 5. Proposed Design

Every type name below is **provisional**: `DEC-198` records that the names settle
at implementation, and nothing in this design depends on which spelling wins.
What is settled is the *shape* — how many distinct things there are, which are
closed and which are open, and which side of the seam each sits on.

Several of this section's rulings were taken during a review pass rather than the
draft, and they are marked `RF-n` where they appear — `RF-n` is this document's
label for the design run's finding `fnd-n`. `RF-1`–`RF-9` are the first
(agent hostile) pass; `RF-10`–`RF-14` the second. § 7.2 carries their reasoning;
§ 10 carries what the passes found.

### 5.1 System Model

The unit splits in two, and the split is along `ADR-001` tiers rather than merely
along files.

```
                        ┌──────────────────────────────────────┐
   command tier         │ main.rs                              │
                        │   backend qualify                    │
                        │   admit(), render_outcome()          │
                        └──────────────┬───────────────────────┘
                                       │
                        ┌──────────────▼───────────────────────┐
   engine tier          │ conformance.rs        (the PAYLOAD)  │
                        │   Row, Delta, ArmShape, Under, Arm   │
                        │   ArmResult, run_arm, run_row, verify│
                        │   PropertyRemoval, AuthorityGrant    │
                        │   ConformanceBackend                 │
                        │   Table, Front, FrontCatalog         │
                        │   tables(), the fixture, the probes  │
                        │   host_descriptor(), the SHELL       │
                        └──────────────┬───────────────────────┘
                                       │  (one direction only)
   ┌───────────────────────────────────▼──────────────────────┐
   │ qualification.rs           (the KERNEL)      tier: leaf  │
   │   RowId = Floor | Assurance | Axis                       │
   │   RowVerdict, ArmJudgement, Indeterminacy, row_verdict   │
   │   Floor, FloorReading, FloorStanding                     │
   │   BackendId, Availability, HostDescriptor                │
   │   QualificationVerdict, Qualification                    │
   │   Claim/AuxOutcome, Unrowed/Reading                      │
   │   qualify_over(...)          ← values only, no traits    │
   │   imports std — and NO other module in this crate        │
   └───────────────────────────────────▲──────────────────────┘
                                       │  BackendId, Availability
   leaf tier          ┌────────────────┴─────┐   ┌─────────────┐
                      │ backend.rs           │   │ host.rs     │
                      │ CapsuleBackend       │   │ HostFacts   │
                      │ Termination, …       │   │ SystemHost  │
                      └──────────────────────┘   └─────────────┘
```

**Naming.** `qualification` for the kernel and `conformance` for the payload,
because `DEC-194` renames the mechanism axis to *qualification* and that is
exactly what the kernel adjudicates; what remains in `conformance` is the
bubblewrap suite that submits itself to it.

**What the kernel owns, stated as a criterion.** `P1` weighs each candidate but
argues only against inclusion, and a boundary defended solely by *is this worth
it?* drifts — five separate judgement calls have already been wrong in this run.
So the positive rule, stated once:

> The kernel owns the **adjudicative normal form** — the minimal
> mechanism-independent inputs whose distinctions it actually observes, the
> algebra of the claims it emits, and the identity and provenance that bind a
> claim to what it is about. Nothing else.

Read the three clauses as a filter. *Distinctions it observes* is why `Availability`
moves and `ArmResult`'s diagnostics do not. *The claim algebra* is why
`RowVerdict`, `Floor` and `Qualification` are kernel types. *Identity and
provenance* is why `RowId`, `AssuranceKey`, `BackendId` and `HostDescriptor` are,
and why `Front` — a label on a claim, reasoned about by nobody — is not.

The criterion is deliberately **not** *whatever the verdict records*: that
formulation would swallow any rendered diagnostic that happens to be printed
beside a verdict, which is exactly how a kernel becomes a dumping ground. `P1`
remains the review check on over-inclusion, and that residual is not
machine-enforceable — no gate can ask whether a type earns its place.

**The observational-equivalence test**, which makes the first clause mechanical
rather than a feel:

> Two payload values are **observationally equivalent** at the seam if
> substituting one for the other can never change the kernel's verdict. Where
> the quotient is *narrower* than the type, **project** it. Where the quotient
> *is* the type, **move** it.

Projecting a type whose quotient is the type produces an isomorphic copy: a
second name, a second maintenance site, and a fresh judgement of the class that
has now been wrong four times. Moving a type whose quotient is narrower drags the
excess across the seam, which is `RF-10`. The test decides both, and it decides
them the same way every time — which is its point.

**The kernel classifies as a `leaf`** (`DEC-197`). Its only import is `std`: no
module of `doctrine-control` is named in it at all, because `BackendId` and
`Availability` move in rather than being imported. It names no trait and does not
touch `host` (`RF-3`, `RF-7`, `RF-9`). `layering.toml` **gains one row**,
`backend`'s existing row gains an edge to it, and nothing is re-classified:

```toml
qualification = "leaf"      # the verdict algebra — imports nothing in-crate
conformance   = "engine"    # the bubblewrap payload — → qualification, provision, …
```

That classification is a **required exit criterion**, not an expected
consequence: the architecture gate must classify the kernel `leaf` and pass.
`ISS-326` does not void it — the gate exempts *edgeless* modules from
classification, and `backend`, `conformance`, `transaction` and `main` all import
the kernel, so it appears as an edge target and is flagged.

**`BackendId` and `Availability` move into the kernel** (`DEC-197`, re-cut
2026-08-12). The draft left them in `backend.rs` for the kernel to import, on the
argument that an open mechanism-minted newtype is the *contract's* business and
not the verdict's. That argument never faced its counterexample: `inq-8` minted
`AssuranceKey`, which § 5.2.1 describes as exactly `BackendId`'s shape and exactly
its justification, and which `P3` places in the kernel. Two identically
constructed identity atoms on opposite sides of one seam has no merits-based
defence.

The draft's edge argument was simply inverted. Moving `BackendId` does not force
`backend` to import an *engine* unit and cascade into `transaction` and
`provision`; it forces `backend` to import a **leaf**, which is a legal leaf→leaf
edge with no cycle, and this tree already has several (`backend → config`,
`capacity → config, host`). `transaction.rs`, `main.rs` and `conformance.rs`
import from the kernel on the same terms. Nothing is re-exported, so one type
keeps one name (`P6`).

The prize is the boundary probe. A kernel that imports nothing from this crate
compiles into a synthetic crate with **no fake module** — and therefore with no
hand-maintained allowlist a later agent can widen to turn a red target green,
which is the precise failure `R1` and `F7` exist to prevent.

`Availability` **moves** rather than being projected, and the equivalence test is
what settles it: every distinction an `Availability` carries is consumed by the
kernel and reproduced in `Qualification::Unavailable`, so the quotient *is* the
type and a projection would be an isomorphic copy. `ArmResult` is the same test
answered the other way (§ 5.2.5).

**`HostDescriptor` moves into the kernel; `host_descriptor()` does not.** The
type (`conformance.rs:2803`) is three strings and names no mechanism, so it is a
verdict field like any other. Its *derivation* reads
`/proc/sys/kernel/osrelease` from disk (`:2825`) and stays on the payload side —
which is `RF-9`'s repair and § 5.3's purity claim made true rather than asserted.

**What crosses the seam, and in which direction.** After the split, exactly one
direction: payload → kernel. The current unit has **six** references pointing the
other way, not the three `DEC-196` enumerated:

| # | reference | before | after |
|---|---|---|---|
| 1 | `rows: &[Row]` | kernel function takes the mechanism's row type | `qualify_over` takes `&[RowId]`; `Row` is payload-only |
| 2 | `backend: &dyn ConformanceBackend` | kernel names a trait whose methods speak `PropertyRemoval`/`AuthorityGrant`/`Under` | the parameter **drops**: the kernel takes `BackendId` and `Availability`, now its own types, as values (`RF-7`, `D10`) |
| 3 | `Delta::Widened(fn(&Fixture) -> …)` | a fixture-typed fn pointer inside a would-be kernel type | `Delta` is payload-only; no kernel type names `Fixture` |
| 4 | `host.path_exists(SHELL)` at `:5300` | the kernel tests for `/bin/sh`, a precondition of the *payload's* probes — the code's own comment reads *"Every payload runs under `/bin/sh -c`"* | the payload composes its own preconditions into the `Availability` it hands in (`RF-3`) |
| 5 | `host_descriptor()` at `:5280` | the kernel calls a disk read | the payload derives the descriptor and passes it as a value (`RF-9`) |
| 6 | `ArmResult::Indeterminate { termination, … }` at `:848-853` | `Termination` (`backend.rs:751`) rides into the kernel inside a *field* of `row_verdict`'s parameter type, unread | the kernel takes `ArmJudgement`; the payload keeps `ArmResult` and projects into it (`RF-10`, § 5.2.5) |

`verify`'s fixture `OnceCell` needed nothing: `run_row` already takes `&Fixture`
explicitly, so placing both on the payload side dissolves it.

**Three enumerations of this list have now been wrong.** `DEC-196`'s missed
`ConformanceBackend`. The draft's missed the shell precondition and the descriptor
read — both in `verify_over`'s *body*, where no enumeration had looked. The first
review pass closed the body and missed a *field* of a type on the second
kernel-bound function's parameter list. Signature, body, field: three location
classes, each found only after the previous was closed, and every pass careful and
believed complete at the time. The design therefore stops relying on the method.

`R1`'s original answer — *the `leaf` classification is a machine check* — is
necessary and **not sufficient**, and the last three references each walk past it
differently. References 4 and 5 survive it because a kernel that re-declares
`const SHELL: &str = "/bin/sh"` and calls `std::fs::read_to_string` imports
nothing from `conformance` and classifies `leaf` cleanly: a leak by duplicated
constant is invisible to a dependency gate. Reference 6 survives it for a
different reason — **the gate proves tier *direction*, not mechanism
*neutrality***. `backend` is itself `leaf`, so a `backend`-resident mechanism type
walks straight through. The six types `I7` enumerates only *appeared* to be
gate-checked because they all happen to live in `conformance`.

The design's answer is therefore in two parts, one structural and one mechanical.

**Structural — the kernel's entry point takes values, and the only functions it
takes are the three the payload supplies as closures.** No `&dyn` backend, no
`&dyn HostFacts`, no path, no environment. A mechanism-shaped precondition then
has no parameter to ride in on and nothing to be tested against — it cannot be
*expressed* in the kernel, rather than being caught after it is. That is `P2`
applied to the seam itself, and it is what `I10` pins.

**Mechanical — a `harness = false` cargo test target `#[path]`-includes the
kernel into a synthetic crate, and compilation is the assertion** (`DEC-197`).
Because the kernel imports nothing else in this crate, that target needs no fake
module, and so carries no allowlist to widen. `harness = false` is load-bearing:
it stops `cfg(test)` activating and dragging `DEC-200`'s kernel test band in. A
negative control is required at implementation, or the probe can be present and
prove nothing.

What the probe does **not** prove is std-only. The target stays inside the
`doctrine-control` package, so that package's external dependencies remain
nameable and a duplicated constant stays invisible. That residual is `R1`'s and it
is the honest limit of the mechanical half. Rust offers no per-module import
restriction inside one crate — `clippy.toml`'s `disallowed-types` is
workspace-scoped — so the only shape that would make `I7` compiler-proven in
production is a separate crate, which `IMP-404` holds and this slice declines on
scope.

### 5.2 Interfaces & Contracts

#### 5.2.1 Row identity — three things, not two

`DEC-198`'s central move. One closed `Property` enum cannot be both the thing the
floor exhaustively matches over and the thing a hypervisor extends.

```rust
/// What a row is about. Three kinds, because the floor and the profile have
/// opposite closedness requirements and axes have neither.
pub(crate) enum RowId {
    /// Closed. Exhaustively matched by the floor reduction, so a new member
    /// fails to compile rather than passing silently.
    Floor(FloorProperty),
    /// Open, mechanism-minted. BackendId's construction, for BackendId's reason.
    Assurance(AssuranceKey),
    /// Closed. REQ-450 criterion 1's five freshness axes, which are a property
    /// of the transaction and not of the confinement mechanism.
    Axis(Axis),
}

/// The authority floor. One member today — table A row 3 — and its membership is
/// ADR-020's territory, not a mechanism's.
pub(crate) enum FloorProperty {
    DeniedCanonicalStateAndCredentials,
}

/// An assurance row's key. A `&'static str` newtype so it cannot be built from
/// runtime text; no `Display`, so it cannot drift into a rendering. Exactly
/// `BackendId`'s shape — which `DEC-197` now moves in here beside it — and
/// exactly its justification: the contract must bind mechanisms nobody has
/// written yet.
pub(crate) struct AssuranceKey(&'static str);
```

Placing these two side by side is what forced `DEC-197`'s re-cut. They are the
same construction for the same reason, and the draft had them on opposite sides
of the seam.

**No `Front` type here** (`RF-5`, `D8`). Fronts are a payload concern; § 5.3
says where they live and why the kernel does not name them.

**The residual, stated plainly.** Thirteen of the fourteen current `Property`
members become payload-side `AssuranceKey` constants. That is the point —
a hypervisor publishes none of rows 10, 12, 13 and 14 and mints its own, editing
nothing in the kernel — but it is a real migration and § 9 carries its
translation table.

`Axis` stays closed and stays in the kernel because a freshness axis is a
property of the *transaction*, which every mechanism has, rather than of the
confinement mechanism, which each has differently. It is for the same reason
that an axis belongs to no escape front: freshness is not an escape route, and
grouping it as one would be the `CPT-002` misreading inverted.

#### 5.2.2 The floor — where `EVD-021` dies

The floor is not a filtered `Vec`. It is a value that cannot be built without a
reading for every member:

```rust
/// Every floor row's verdict. One field per `FloorProperty` member: an empty
/// floor is unrepresentable, and adding a member breaks every construction site
/// rather than passing silently (DEC-195's two guarantees, both structural).
pub(crate) struct Floor {
    pub(crate) denied_canonical_state_and_credentials: RowVerdict,
}

/// Whether a floor could be read at all — the distinction `RF-1` found missing.
///
/// `Floor` is total by construction, so *a floor row was never submitted* has
/// nowhere to live inside it. It lives here instead, one level out, which keeps
/// both guarantees: the floor stays unbuildable when incomplete, and the
/// incompleteness is still reportable beside the rows that did run.
pub(crate) enum FloorReading {
    Established(Floor),
    NotEstablished { missing: FloorProperty },
}

/// What the floor established. Not `bool`: "no floor row ran" is a third thing,
/// and it is the one EVD-021 shows a boolean cannot say.
pub(crate) enum FloorStanding {
    Held,
    Breached,
    /// A floor row is missing from the submitted set. The backend does not
    /// qualify, and the reason is that the question was never asked.
    NotEstablished { missing: FloorProperty },
}

impl Floor {
    /// Reduced by exhaustive match — over `RowVerdict`, per member. No `.all()`,
    /// no iterator, nothing with an empty case.
    fn standing(&self) -> FloorStanding {
        match self.denied_canonical_state_and_credentials {
            RowVerdict::Proven => FloorStanding::Held,
            RowVerdict::Violated | RowVerdict::Unproven
            | RowVerdict::Indeterminate { .. } => FloorStanding::Breached,
        }
    }
}

impl FloorReading {
    /// The only route from a submitted row list to a floor reading. **This is
    /// where the vacuous path dies**: `FloorReading::from_rows(&[])` is
    /// `NotEstablished`, never a floor that is vacuously held.
    pub(crate) fn from_rows(rows: &[(RowId, RowVerdict)]) -> Self { /* … */ }

    /// Total, by the same construction: every reading has a standing, and the
    /// third variant is reachable exactly when the third reading is.
    pub(crate) fn standing(&self) -> FloorStanding {
        match self {
            FloorReading::Established(floor) => floor.standing(),
            FloorReading::NotEstablished { missing } =>
                FloorStanding::NotEstablished { missing: *missing },
        }
    }
}
```

Three things are worth stating about this shape, because the first two look like
over-engineering at one member and are not:

- **`Floor`'s totality is the whole repair.** `admission(&[])` returns `Admitted`
  today; there is no value of `Floor` that an empty row list can produce. The
  guard moves from an assertion in a test about a different function's return
  value (`EVD-021`) into the type.
- **`Unproven` breaches the floor — and the floor only.** A control that did not
  fire proves nothing, and the floor is a claim rather than a report, so anything
  short of `Proven` fails it. The assurance profile is governed by no such rule:
  an `Unproven` assurance row is published as `Unproven` and reported verbatim,
  because nothing reduces over the profile and there is therefore nowhere for a
  strictness rule to apply. That asymmetry is `P4` — the floor is a claim, the
  profile is evidence — and § 5.5 `I3` pins it, scoped to the floor.
- **`FloorReading` is a wrapper and not a `Result`** (`RF-1`, `D7`). *The floor
  row was not submitted* is a state of the run, not an error in computing one,
  and a `Result` in a verdict field reads as the second. The named enum also
  keeps the whole verdict constructible in one shape, so the rows that *did* run
  are still published when the floor is missing — which `Result` in the field
  would have made awkward and a third `Qualification` variant would have thrown
  away outright (`P4`).

The floor's departure from the flat row list has a rendering consequence that is
easy to miss and is not cosmetic: after the split, table A row 3 lives inside
`Floor` and has **no** `(RowId, RowVerdict)` entry, so today's
`row Property(DeniedCanonicalStateAndCredentials)=Proven` line has no successor
unless one is derived. § 5.4 and § 9.1 derive it (`RF-11`).

#### 5.2.3 The verdict

```rust
pub(crate) struct QualificationVerdict {
    pub(crate) backend: BackendId,
    pub(crate) host: HostDescriptor,
    pub(crate) observed_at: String,
    pub(crate) result: Qualification,
}

/// Unavailability lifts to the top (DEC-195): *did not run here* and *ran and
/// fell short* are different claims, and burying the first inside the second's
/// reason enum reads as a failure when it is an absence. POL-002 facet 3.
pub(crate) enum Qualification {
    Unavailable { missing: String, remedy: String },
    Ran {
        /// Reduced — after the reading is established. The claim.
        floor: FloorReading,
        /// Published, never reduced. The evidence.
        assurance: Vec<(AssuranceKey, RowVerdict)>,
        axes: Vec<(Axis, RowVerdict)>,
        /// Outcomes, but never admitted on — in either direction. Structural:
        /// `FloorReading::from_rows` is given the row list and nothing else.
        auxiliary: Vec<(Claim, AuxOutcome)>,
        /// No outcome slot anywhere. Attaching a verdict is a type change.
        observations: Vec<(Unrowed, Reading)>,
    },
}
```

There is **no admission field**. `ADR-020` gives step 5 admission to the control
plane alone and no backend property bears on it, so the verdict names the axis in
its documentation and carries nowhere to record a conclusion about it — the
`Unrowed`/`Reading` precedent applied one tier up (`P4`).

`Qualification::Ran` does not carry a summary word either. `Qualified` /
`Disqualified` is a *rendering* computed from `floor.standing()` at the command
tier, not a stored field, so there is no cached scalar to disagree with the
rows it was computed from.

**The verdict carries no front labels**, and § 5.3 explains the placement. The
consequence to note here is that `I4` is untouched: no grouping structure is
introduced into the verdict, so there is no new collection for F2's vacuity to
recur in.

#### 5.2.4 The kernel's one entry point

```rust
pub(crate) fn qualify_over(
    backend: BackendId,                                  // a value — RF-7
    availability: Availability,                          // a value — RF-3, RF-7
    host: HostDescriptor,                                // a value — RF-9
    observed_at: String,
    rows: &[RowId],                                      // re-keyed — DEC-196
    auxiliary: &dyn Fn() -> Vec<(Claim, AuxOutcome)>,
    observations: &dyn Fn() -> Vec<(Unrowed, Reading)>,
    run_row: &dyn Fn(&RowId) -> RowVerdict,              // payload closes over its own table
) -> QualificationVerdict
```

The first three are kernel-owned types after `DEC-197`'s re-cut, so this
signature names nothing from another module of the crate.

Five narrowings, each removing a mechanism name — or a way for one to arrive —
from the kernel's vocabulary:

- `BackendId` and `Availability` **as values**, instead of
  `&dyn ConformanceBackend`. `verify_over` called only `id()` and
  `availability()`; taking their results removes the trait rather than narrowing
  it, and `ConformanceBackend` moves to the payload entire. The weakening and
  granting methods belong to the payload's row runner, which holds its own
  concrete backend.
- `HostDescriptor` **as a value**, instead of `&dyn HostFacts` plus a disk read.
  The kernel now performs no I/O of any kind (`I10`).
- **No shell parameter, and no shell knowledge.** The payload composes its own
  preconditions — the backend's `availability()` *and* the presence of
  `/bin/sh` — into the single `Availability` it hands in. A host with no usable
  shell is still `Unavailable { missing: "/bin/sh", remedy: … }` and still never
  a violated row; what changes is who knows that `/bin/sh` is the thing the
  probes need, and the answer is the layer whose probes need it.
- `&[RowId]` instead of `&[Row]`. The kernel schedules *identities*; the payload
  maps an identity back to its own `Row` through its own table.
- `run_row: &dyn Fn(&RowId) -> RowVerdict` instead of
  `&dyn Fn(&dyn ConformanceBackend, &Row) -> RowVerdict`. The backend is no
  longer threaded through the kernel to reach the runner, because the runner
  already has one.

The payload's side of the contract is one table, one label catalogue, one runner
and one return envelope:

```rust
// conformance.rs — the payload
struct Table { rows: Vec<Row> }

impl Table {
    fn ids(&self) -> Vec<RowId>;                 // what it submits — DEC-189 membership
    fn row_for(&self, id: &RowId) -> &Row;       // total over its own ids — I9
    fn front_catalog(&self) -> FrontCatalog;     // the labels, lifted out — D8, D11
}

/// The front labels and nothing else — what the command tier is handed for
/// rendering. Keyed by `AssuranceKey`, which is what the verdict actually
/// carries; only assurance rows have fronts.
struct FrontCatalog { /* AssuranceKey → Front */ }

impl FrontCatalog {
    /// Total over the keys the same table submitted (`I9`), so no `Option`.
    fn front_of(&self, key: &AssuranceKey) -> &Front;
}

/// What the payload's entry point returns: the verdict, and the labels needed to
/// render it. Transient — consumed by `main.rs` in the process that built it and
/// never stored (`D8`, `D11`; § 10.2 carries the standing caveat).
struct QualificationRun {
    verdict: QualificationVerdict,
    fronts: FrontCatalog,
}

fn run_row(backend: &BubblewrapBackend, row: &Row) -> RowVerdict;
```

`ids()` and `row_for` come from **one** table value, which is what makes
`row_for` total and `run_row` infallible: the kernel can only be handed ids the
payload just produced (`I9`, `RF-6`). A fallible `run_row` was considered and
refused — its error arm is unreachable from the production path, and an
unreachable arm invites a fabricated verdict at the one place a fabricated
verdict would be invisible.

`FrontCatalog` is `RF-12`'s repair and its shape is `D11`. The draft gave `Table`
a `front_of` method and told the command tier to call it, and nothing anywhere
handed the command tier a `Table`: `render_verdict` takes only the verdict, and
the payload builds its table inside its own entry point (`conformance.rs:5254`).
The catalogue is what closes that, and it is keyed by `AssuranceKey` rather than
by `RowId` because a key is what the verdict's `assurance` vector holds — the
draft's `&RowId` signature would have made every call site rewrap
`RowId::Assurance(key)` first.

#### 5.2.5 The two-arm algebra — the judgement moves, the diagnostics do not

`row_verdict` moves into the kernel; what it *takes* narrows on the way
(`RF-10`). The draft moved it **byte-identical**, `ArmResult` and `Indeterminacy`
travelling with it on the claim that they name no mechanism. That claim was
false, and the byte-identical claim goes with it:
`ArmResult::Indeterminate` (`conformance.rs:848-853`) carries
`termination: Termination` beside `stdout` and `stderr`, and `Termination`
(`backend.rs:751`) is the backend's own report of how a run ended — the same
field `Observation` (`backend.rs:770`) carries.

§ 5.1's equivalence test settles the repair mechanically. `row_verdict` matches
`ArmResult::Indeterminate { reason, .. }` (`:3177`) and discards `termination`,
`stdout` and `stderr`; two `ArmResult`s differing only in those three fields can
never change the verdict. The quotient is strictly *narrower* than the type, so
this one **projects**:

```rust
// the kernel — what an arm showed, reduced to what the algebra observes
pub(crate) enum ArmJudgement {
    Held,
    Failed,
    Indeterminate(Indeterminacy),
}

pub(crate) fn row_verdict(probe: ArmJudgement, control: ArmJudgement) -> RowVerdict
```

```rust
// the payload — keeps the diagnostics that make an indeterminate arm triageable
impl ArmResult {
    fn into_judgement(self) -> ArmJudgement { /* … */ }
}
```

`Indeterminacy` travels into the kernel with the judgement. It is closed at four
variants (`EX-11`) and every one names a state of the *arm* rather than of the
mechanism — and `Indeterminacy::BackendError` already launders `BackendError` to
a `String` (`:881-884`) *"so the verdict carries a rendering the backend cannot
later change under it"*. That laundering is the precedent; `termination` is the
sibling field that escaped the same discipline, and the projection applies it.

**What this costs.** The function's *body* is unchanged — the same nine-cell
match, the same arms, the same comments — but its signature is not, so § 9.6's
`row_verdict_is_unchanged` cannot be a byte comparison. It becomes a truth table
over all nine probe/control pairs, plus a projection test asserting that no
diagnostic field can move a judgement. The existing
`an_indeterminate_arm_carries_its_termination_and_output` (`:6176`) survives
unchanged on the payload side, where it now asserts what `ArmResult` carries
rather than what the kernel receives.

**The alternative refused** was to leave `row_verdict` payload-side entirely,
which the finding offered as the cheaper repair. § 7.3 records why it is not.

### 5.3 Data, State & Ownership

**The kernel owns no state and performs no I/O.** No clock, no rng, no disk, no
git, no environment. `observed_at: String` and `HostDescriptor` arrive as values.

This is stated as a target rather than as inherited discipline, because the
current code does not meet it: `verify_over:5280` calls `host_descriptor()`
(`:2825`), which reads `/proc/sys/kernel/osrelease` from disk, and the draft of
this design asserted the opposite (`RF-9`). The split is where it becomes true.
The derivation stays on the payload side, one call earlier, and the kernel
receives its result — the same treatment `today: String` already had, applied to
the fact that was quietly exempt from it.

**`today` → `observed_at`, on the parameter and the field.** The existing
names are `today: String` (`conformance.rs:5220`, `:5273`) feeding
`AdmissionVerdict::date`, sourced from `doctrine::today()` in the shell. Both are
wrong for what the value is. `today` is *relative to now*, and this value exists
precisely to be read back later out of a recorded artefact — a verdict from March
that says `today` is telling the reader nothing about when it was observed. And
`date` does not say what the date is the date *of*. `observed_at` names the event
on both sides, so the field and its input agree. The blast radius is two
signatures, three construction sites, `main.rs:186` and two tests — and these
types are moving under `DEC-194` regardless, so the rename costs a line each and
avoids a second pass over the same call sites.

**The rename is source-side only** (`RF-2`, `D9`). `render_verdict` emits
`date={}` on the verdict's header line, and § 9.1's licence does not cover a
change to it: the header line survives the split unaltered, and re-spelling one
of its keys buys a source reader nothing they do not already get from the Rust
rename. So the rendered key stays `date=` and the identifiers change beneath it. The
rendered spelling is worth revisiting; it is not worth spending this slice's
preservation licence on.

Ownership, stated as a table because the boundary calls are the design:

| thing | owner | why |
|---|---|---|
| What a row verdict *means* | kernel | the judgement; mechanism-independent by construction |
| Backend **identity** and **availability** | kernel | `DEC-197` re-cut: `BackendId` is `AssuranceKey`'s construction, and every `Availability` distinction is consumed by the verdict |
| Floor **membership** | kernel | `ADR-020`'s territory; a mechanism may not vote on the authority floor |
| Assurance **membership** | payload | `DEC-189` — a function of the mechanism's available deltas |
| Assurance **key shape** | kernel | the contract binding mechanisms nobody has written (`P3`) |
| What an arm **showed** | kernel | `ArmJudgement` — the distinctions the algebra observes |
| An arm's **diagnostics** (`termination`, `stdout`, `stderr`) | payload | never read by the algebra; they make an arm triageable, not judgeable (`RF-10`) |
| Fronts, wholly | payload | the kernel neither reduces over them nor validates them (`D8`) |
| Axis membership | kernel | `REQ-450`'s five, a property of the transaction |
| How a row is *run* | payload | arms, deltas, fixture, probes — all bubblewrap-shaped |
| The identity→row map | payload | it owns the table, so it owns the lookup |
| Run **preconditions** | payload | the shell the probes need is the probes' fact (`RF-3`) |
| Host-fact derivation | payload | it reads disk; the kernel receives the result (`RF-9`) |
| Rendering and exit codes | command tier | `main.rs`, unchanged in ownership |

**Two membership sets, deliberately not one.** The floor is closed and kernel-owned
because a mechanism that could add to or remove from the authority floor could
define away the thing it is being tested for. The assurance profile is open and
payload-owned because `REQ-459` criterion 3 — as narrowed by `DEC-201` — says a
second mechanism proves the same floor and publishes *its own* profile.

**Where the fronts live** (`RF-5`, `D8`; transport by `RF-12`, `D11`). The draft
made front-labelled rendering a `CPT-002` obligation and gave it nothing to
compute from. They live wholly in the payload: its table declares each row's
front, `FrontCatalog` is the lookup lifted out of that table, and the payload's
entry point returns the catalogue beside the verdict so the command tier has it
in hand at render time.

Two arguments settle the placement, and they point the same way. The kernel
neither reduces over fronts nor validates them, so a `Front` in the kernel would
be a type a reader must load to understand nothing the kernel does (`P1`), and it
fails the § 5.1 criterion outright — it is a label on a claim, not a distinction
the claim algebra observes. And `A4` keeps the front list a sketch — a shape the
kernel defines but never reasons about is the one case `P3` does *not* cover,
because `P3`'s justification is that the kernel must name what it reasons about.
The verdict is rendered by the same process that built it, so nothing is lost by
the label arriving at render time.

What the command tier is handed is the **catalogue and not the table**. Handing
it a `Table` would put the probes, the deltas and the fixture in the command
tier's hands for the sake of a string, and a free global front lookup could be
asked about a key from a table nobody submitted — which weakens `I9`, the one
thing making `run_row` infallible. `D11` records the call.

**The empty profile is legal.** `DEC-189` guarantees a mechanism whose available
deltas yield no row for some front. An empty `assurance` vector is a truthful
report, and because nothing reduces over it (`P4`, F2), it cannot be vacuously
strong. This is the specific reason fronts are not a reduction target.

### 5.4 Lifecycle, Operations & Dynamics

The run sequence, with the one green path preserved (invariant 1) — no skip, no
early return and no conditional reaches a qualified outcome. The two steps above
the line are the payload's, and that placement is `RF-3`'s and `RF-9`'s repair:

```
  ── payload ──────────────────────────────────────────────────────────
  backend.availability()  ⨝  host.path_exists(SHELL)  →  Availability
  host_descriptor()                                   →  HostDescriptor
  table.ids()                                         →  &[RowId]
        │
  ── kernel: qualify_over(backend, availability, host, observed_at, …) ─
        │
        ├── Unavailable{missing, remedy} ──► Qualification::Unavailable
        │                                    (nothing ran; no rows, no claims,
        │                                     no observations to report)
        ▼
  for id in rows:  run_row(id)          ← payload maps id → Row, runs both arms,
        │                                  projects each ArmResult through
        │                                  into_judgement(), then calls the
        │                                  kernel's row_verdict(probe, control)
        ▼
  FloorReading::from_rows(&verdicts)
        │
        ├── NotEstablished{missing} ──► Ran{ floor: NotEstablished, … }
        ▼                                   (the rows that ran are still published)
  Qualification::Ran { floor, assurance, axes, auxiliary, observations }
        │
  ── payload ──────────────────────────────────────────────────────────
  QualificationRun { verdict, fronts: table.front_catalog() }
        │
        ▼
  ── command tier: main.rs ────────────────────────────────────────────
  floor.standing() ──► Held           → EXIT_QUALIFIED
                    └► Breached       → EXIT_DISQUALIFIED
                    └► NotEstablished → EXIT_DISQUALIFIED
  each assurance row rendered with fronts.front_of(&key)
```

The shell-absence path keeps its current behaviour exactly — a host with no
usable shell is *unavailable*, naming what is missing, and never a violated row.
What moved is only which layer knows that `/bin/sh` is the thing to look for.

**`DEC-194`'s rename lands here**, across the whole surface at once because these
types are moving anyway and a second migration is the thing being avoided:
`AdmissionVerdict` → `QualificationVerdict`, `Admission::{Admitted, NotAdmitted}`
→ `Qualification`, the verb `backend verify` → `backend qualify`, and
`EXIT_REFUSED` → `EXIT_DISQUALIFIED`. The existing `EXIT_REFUSED`/`NotAdmitted`
mismatch closes in passing. Both exit constants and the verb literal stay
single-source (`STD-001`).

The rename's radius is wider than the crate, and each site is named because a
missed one leaves a recipe or a rule referring to a verb that no longer exists:

- `.doctrine/adr/001/layering.toml:257` — carries the verb literal, and is the
  **only accepted-governance file** in the radius;
- `justfile` — `capsule-verify` runs `cargo run … -- backend verify` at `:145`,
  and four comments (`:108`, `:114`, `:129`, `:143`) name the verb in prose;
- `main.rs` — the verb's own definition and the exit constants.

**What the operator sees changes — and the change is *constrained*, not
licensed** (`RF-11`). Today the artefact is a header line, one `outcome=` word,
one flat `row {id:?}={row:?}` line per row, then claims and observations. After
the split it is the same artefact with the floor's standing as a claim, every
assurance row carrying its own front label, and the axes unchanged. The draft
described this as *"grouped under its front … rendered as escape fronts"* and
put the difference on § 9.1's permitted list, which does not reach: group headers
are new lines, not a re-spelling of an existing one. So the rendering is
constrained instead, and § 9.1 derives every line of it:

- **No standalone group headers.** Grouping is carried by *ordering* alone: the
  assurance block is ordered by front, and each line names the front it is under.
  A header line has no pre-split predecessor and nothing would derive it.
- **One self-contained line per assurance row**, carrying front, key and verdict
  together. This is the direct successor of today's `row …=` line, and it is
  where `CPT-002`'s **escape** framing is applied so a strong profile is not read
  as a strong safety claim.
- **The established floor row keeps a `row …=` successor**, derived from
  `FloorReading` rather than from the row list. Without this the line simply
  disappears when the row moves into `Floor` (§ 5.2.2) — a deletion, which no
  key-spelling licence covers.
- **The five axis lines are byte-identical.** The renderer reconstructs
  `RowId::Axis(axis)` from the verdict's `axes` vector, so `{:?}` produces the
  text it produces today, unchanged.

`main.rs` reads the labels from the `FrontCatalog` the payload returns beside the
verdict (`D8`, `D11`, § 5.2.4) — not from the `Table`, which would hand the
command tier the probes and the fixture for the sake of a string.

### 5.5 Invariants, Assumptions & Edge Cases

#### Invariants

- **`I1` — One green path.** Exactly one route reaches a qualified outcome:
  available, every floor member read and `Proven`. Preserved from
  invariant 1 and strengthened, because the floor reading now has a state for
  *not asked* distinct from *answered no*.
- **`I2` — No vacuous qualification.** `FloorReading::from_rows(&[])` is
  `NotEstablished`. There is no input for which an empty or partial row set
  yields `FloorStanding::Held`. This is `EVD-021` closed structurally rather than
  by a guard.
- **`I3` — Only `Proven` holds the floor, and this governs the floor alone.**
  `Violated`, `Unproven` and `Indeterminate` all breach it: a control that did
  not fire licenses no inference (§ 2.2), so it cannot hold an authority floor.
  The assurance profile is explicitly *not* subject to this — its rows are
  published exactly as read, because nothing reduces over them.
- **`I4` — Nothing reduces over a collection that can be empty.** The floor is a
  struct with fixed fields; the profile is published per row and never reduced;
  fronts label and never reduce. F2's failure mode has nowhere to recur.
- **`I5` — Auxiliary claims cannot reach the outcome in either direction.**
  Structural, not promised: `FloorReading::from_rows` is given the row list and
  nothing else.
- **`I6` — Unrowed observations carry no verdict.** There is no slot. Adding one
  is a type change visible in a diff.
- **`I7` — No kernel type names a mechanism type.** No `Fixture`, no `Delta`, no
  `PropertyRemoval`, no `AuthorityGrant`, no `Under`, no `ConformanceBackend` —
  and no `Termination`, which is the one `RF-10` found riding in inside a field.
  The machine check is § 5.1's **compile probe**, a `harness = false` target
  building the kernel as a synthetic crate with no fake module. It is explicitly
  **not** the architecture gate's `leaf` classification: the gate proves tier
  direction, not mechanism neutrality, and `backend` is itself `leaf`, so a
  `backend`-resident mechanism type walks straight through it. The probe's own
  residual — a fact re-derived from `std`, or an external dependency of the
  `doctrine-control` package — belongs to `R1`.
- **`I8` — The nineteen row verdicts are invariant across the split.** Four
  auxiliary claims and two unrowed readings likewise. `DEC-199`; § 9 carries the
  instruments.
- **`I9` — The row list, the row runner and the front catalogue come from one
  payload table.** `Table::ids()` produces what is submitted, `Table::row_for` is
  total over it, and `FrontCatalog` is derived from the same value, so the kernel
  cannot be handed an identity the payload cannot construct and the command tier
  cannot ask for a label on a key nobody submitted. This is why `run_row` is
  infallible (`RF-6`) and why `front_of` returns no `Option` (`RF-12`).
- **`I10` — The kernel takes values and closures, and nothing else.** No `&dyn`
  backend, no `&dyn` host, no path, no environment, no I/O. A mechanism-shaped
  precondition has no parameter to arrive on, which is what makes `RF-3`'s and
  `RF-9`'s class unrepresentable rather than merely caught (§ 5.1). Note its
  limit, which `RF-10` demonstrated: `I10` is about the parameter *list*, and a
  mechanism type can still arrive inside a *field* of a value that satisfies it.
  `I7` and the compile probe are what cover that.

#### Assumptions

- **`A1`** — `ADR-020` is not reopened. `DEC-191` was built to land without it,
  and the floor sits entirely inside the authority the ADR already grants.
- **`A2`** — The Firecracker row set is **derived, not implemented** (`OQ-1`,
  owner). The kernel is shaped to admit a row set it will not contain; the
  `AssuranceKey` newtype is what discharges that shaping, and it is checkable
  rather than decorative because a mechanism minting a key edits no kernel file.
- **`A3`** — `ADR-021` is `proposed`, so its two-site `unsafe` budget carries
  directional weight only. The kernel adds no `unsafe` regardless — it is
  judgement over values.
- **`A4`** — `DEC-191`'s front list stays a sketch. Nothing in this design
  depends on its membership, which is why fronts stay payload-side labels rather
  than a kernel type or a reduction target.

#### Edge cases

| case | behaviour | why it is not a bug |
|---|---|---|
| No rows submitted | `FloorReading::from_rows` → `NotEstablished` → disqualified | `I2`; the repair of `EVD-021` |
| Floor row missing, other rows ran | `Ran { floor: NotEstablished, … }` — disqualified, profile still published | the floor is a claim and failed; the rows that ran are still evidence (`P4`) |
| Assurance profile empty, floor held | qualified, with an empty profile published | `DEC-189` guarantees empty fronts; an honest empty report is not a weak one |
| A floor row `Indeterminate` | `Breached` → disqualified | `I3`; indeterminacy is not a pass |
| Backend unavailable | `Qualification::Unavailable`, no rows, no claims, no observations | nothing ran, so there is nothing read to report; `POL-002` facet 3 |
| Host without a shell | `Unavailable`, not a violated row | preserved exactly; the payload now composes it (`RF-3`) |
| An indeterminate arm with a `Termination` the kernel might key on | not representable — the kernel receives `ArmJudgement`, which has no such field | `RF-10`; the projection is where the field is dropped |
| A mechanism mints a key colliding with another's | both publish under one key | keys are mechanism-scoped in meaning; the verdict records `backend: BackendId`, so a reader always knows whose profile they are reading |
| Two rows share a front | both render under it, adjacent by the front ordering | fronts label rather than key; nothing indexes or reduces by them |
| An axis is asked for a front | not representable — `FrontCatalog` is keyed by `AssuranceKey` | axes belong to no escape front (§ 5.2.1) |
| A payload emits a `RowId::Floor` the kernel does not know | not representable — `FloorProperty` is closed | `P2` |
| The kernel is handed an id the payload cannot construct | not reachable — `ids()` and `row_for` come from one table | `I9` |

<!-- doctrine:section sec-10 -->
## 10. Review Notes

Two review passes have been conducted. Both were agent hostile passes run against
the working tree rather than against the document alone, and **both were run by
the design's author**, which is the one thing neither could do anything about:
they cannot disagree with the design's own framing. The external adversarial pass
is still unspent.

`RF-n` is this document's label for the design run's finding `fnd-n` — the two
numbering schemes are the same sequence.

| pass | run revision | against | findings | outcome |
|---|---|---|---|---|
| first | 43 | the draft at `499c2ebbc` | `RF-1` … `RF-9` | dispositioned at rev 45, integrated at rev 46 |
| second | 50 | the integrated design | `RF-10` … `RF-14` | three blocking, two nits; integrated here |

### 10.1 What the passes changed

**The first pass — four shape changes.**

- **`RF-1` → `D7`.** `FloorStanding::NotEstablished` was unreachable:
  `Qualification::Ran` held a total `Floor`, and `standing()` matched only `Held`
  and `Breached`, while § 5.4 and § 9.6 both assumed the third variant existed.
  `FloorReading` now carries *did we get a floor at all* one level out.
- **`RF-3` + `RF-7` + `RF-9` → `D10`.** Two further backward references were
  found in `verify_over`'s **body**, where no previous enumeration had looked:
  the `/bin/sh` precondition, which is a fact about the payload's probes, and
  `host_descriptor()`, which reads disk inside the would-be kernel. Both are
  fixed by the same move — the kernel takes values and closures and nothing else
  — which also removes `CapsuleBackend` from its vocabulary entirely.
- **`RF-5` → `D8`.** Fronts were an obligation in § 5.4 with no type, field or
  contract function to compute them from. They are now wholly payload-side.
- **`RF-2` → `D9`.** `D5`'s rename would have changed the rendered header line,
  which this slice's preservation licence does not cover. The rename is
  source-side only.

Two changed a claim without changing the design: `RF-4` (both instruments need
`bwrap`, not one) and `RF-8` (two line cites). `RF-6` was dispositioned by
stating `I9` rather than by changing a type.

**The second pass — three more shape changes, and a governance re-cut.**

- **`RF-10` → `D12`, and `DEC-197` re-cut.** A **sixth** backward reference, and
  the first found in a *field* rather than in a signature or a body:
  `ArmResult::Indeterminate` carries `termination: Termination`
  (`backend.rs:751`), which `row_verdict` never reads. Two repairs followed. The
  kernel now takes `ArmJudgement` and the payload projects into it (`D12`),
  which costs § 5.2.5's byte-identical claim. And the finding's third
  consequence — that `I7`'s named machine check *cannot fire*, because the
  layering gate proves tier direction and `backend` is itself `leaf` — forced
  `DEC-197`'s placement rider to be re-cut: `BackendId` and `Availability` move
  **into** the kernel, which then imports nothing else in the crate, and `I7`'s
  check becomes the compile probe.
- **`RF-11` → the rendering is constrained, not licensed.** `D8`'s front-grouped
  output was a fourth rendered difference on a licence closed at three, and the
  floor row's departure from the flat row list was a fifth. Widening the licence
  was refused for `D9`'s reason. § 5.4 now constrains the rendering so every
  post-split line has a pre-split predecessor, and § 9.1 carries the relation.
- **`RF-12` → `D11`.** `D8` gave the command tier a method it had no route to
  call. The payload now returns a `FrontCatalog` beside the verdict.

`RF-13` and `RF-14` were nits and changed prose: a stale count in `P2` and two
off-by-one cites.

**The finding that should worry a third reviewer most is `RF-10`, and not
because of what it was.** Four enumerations of *what crosses the seam backwards*
have now been made — `DEC-196`'s, the draft's, and each review pass's — every one
careful, every one believed complete, and every one wrong. Three location classes
have been found so far: parameter list, function body, and a field of a
parameter's type. Each was found only after the previous had been closed, and the
`leaf` classification the design originally leaned on would have caught none of
them. § 3.2 `F7` and § 5.1 carry the structural and mechanical answers taken
instead. **The right prior for a third pass is that there is a fourth location
class**, not that the list is finally complete.

### 10.2 Attack these first

Refreshed after the second pass; items it answered are marked and kept, because
what a pass *closed* is as useful to a later reviewer as what is open.

- **`D1` / `I3` — only `Proven` holds the floor.** Still open, and still the one
  substantive ruling in § 5 with no banked decision behind it. `DEC-195` settled
  that the floor is reduced; it said nothing about what `Unproven` does to it.
  The ruling is owner-confirmed and scoped to the floor, but the argument is
  mine: *a control that did not fire licenses no inference, so it cannot hold an
  authority floor*. If that is wrong, `FloorStanding` is wrong. Neither pass
  touched it.
- **`D7` — is `FloorReading` a wrapper too many?** Still open. Two types and a
  three-variant standing for a set of size one. `D2` and `D3` were each defended
  on their own; `D7` is the third layer and was added under review pressure,
  which is exactly when ceremony gets added without noticing. If
  `floor: Result<Floor, FloorProperty>` reads acceptably in a verdict, `D7` is
  ceremony and `R6` is real rather than mitigated.
- **The `Qualification::Ran` shape.** Still open. Four collections in one variant
  (`assurance`, `axes`, `auxiliary`, `observations`) plus a `floor`. Ask whether
  `axes` genuinely belongs beside `assurance` rather than inside it — the design
  says a freshness axis is a property of the transaction rather than of the
  mechanism, which is a real distinction, but it is the boundary call in § 5.2 I
  am least sure of. Both passes endorsed keeping them separate on `CPT-002`
  grounds; both endorsements came from the same author.
- **`D10` — does the value-only entry point close `F7`'s class?** **Answered:
  no.** The second pass was invited to find a leak cheaper than `R1`'s
  std-re-derivation residual and found one — `RF-10`, arriving inside a field of
  a value that satisfies `D10` perfectly. The invitation stands in its new form:
  `I10` covers the parameter list and the compile probe covers in-crate imports;
  find the third thing neither sees.
- **§ 5.1's placement criterion is new and untested.** The adjudicative normal
  form and the observational-equivalence test were written *in response to* five
  wrong judgement calls, which is the worst moment to trust a new rule. Run them
  backwards over the boundary calls already made and check they reproduce them —
  and if one does not, the criterion is wrong or the call was.
- **`D8` / `D11` — fronts in the payload.** Partly answered. `RF-12` closed the
  transport gap; the deeper argument still rests on the verdict being rendered by
  the process that built it. If a verdict is ever serialised, stored, or compared
  across runs, that argument fails and the front labels are lost. Check whether
  anything downstream of this slice wants that.

### 10.3 Where the design is deliberately incomplete

Not oversights; flagged so a reviewer does not spend effort finding them.

- **Type names are provisional** (`DEC-198`). `qualification`, `FloorProperty`,
  `AssuranceKey`, `FloorReading`, `FloorStanding`, `ArmJudgement`,
  `FrontCatalog`, `QualificationRun` — all settle at implementation. Argue the
  shapes, not the spellings.
- **`OQ-3` (a possible fifth `DEC-189` row) is not resolved here** and does not
  need to be: an open `AssuranceKey` admits whatever membership the analysis
  lands on. It belongs to `DEC-189`.
- **`OQ-4` (what a host failing a row should report) stays open.** `RV-352` `F-8`
  left it; `DEC-199` routes around it rather than closing it.
- **The kernel's final size is not predicted** (`P1`). A review finding of the
  form *this is more than a few hundred lines* is answered by `P1` and § 3.4, and
  the owner has ruled that the numbers are not a criterion.
- **The payload's `Table` type is sketched, not designed.** § 5.2.4 gives it
  three methods, and `RF-12` corrected the reason: two of them (`ids`, `row_for`)
  are what the kernel's contract needs, and the third (`front_catalog`) is what
  the *command tier* needs, which is a different consumer and was the source of
  the gap. How the table is built, and whether it subsumes `tables()`, is
  implementation.
- **The compile probe's negative control is not specified.** `DEC-197` requires
  one — a probe that cannot fail is a probe that proves nothing — but what it
  looks like is an implementation call.

### 10.4 Known weak points in the evidence

- **The ~67-test figure is a symbol-count estimate**, not a classification
  (§ 6, `R3`). It is used only for budgeting.
- **`EVD-022` is a single run on one host.** It reproduces `RV-352`'s two-run
  baseline claim, which is the corroboration; but the pre half of the bracket is
  one execution, and it is now unrepeatable at that commit in this working tree's
  future.
- **§ 2's line numbers are against `94d0b5603`, and "verified twice" was not
  enough.** They were re-read during drafting (two cited in the research artefact
  were wrong: `ConformanceBackend` is `:638` not `:630`; `Delta::Widened` is
  `:2548` not `:2547`), re-read by the first review pass, which found two more
  (`RF-8`: `BackendId` is `:811` not `:805`; `AdmissionVerdict` is `:2689` not
  `:2688`) — and this document then claimed every cite had been verified twice.
  The second pass falsified that immediately, finding two more (`RF-14`: the
  shell precondition is `:5300` not `:5299`, the comment being `:5298-5299`;
  `render_verdict` is `:178` not `:177`). Both were off-by-one against a
  preceding comment line, which is now a known failure mode of this document's
  cites rather than a coincidence. **No count of passes is claimed here.** What
  can be said is narrower and checkable: `RF-14` re-read the remaining cites
  against the same tree and reported them clean, and a cite introduced after this
  point carries no provenance at all.

### 10.5 Conformance to governance, stated for checking

| authority | how this design answers to it |
|---|---|
| `ADR-020` | The floor sits inside the authority the ADR already grants; step 5 admission is named and carries no outcome field. Not reopened (`A1`). |
| `ADR-001` | Kernel classified `leaf`, importing `std` and **no other module of this crate** (`DEC-197` re-cut); `layering.toml` gains one row and `backend`'s row gains an edge to it, with nothing re-classified. Leaf→leaf edges already exist in this tree. The gate is the check on the *classification*; `I7`'s neutrality claim is checked by § 5.1's compile probe instead, because the gate proves tier direction only. |
| `STD-001` | Exit constants and the verb literal stay single-source through the rename; `layering.toml:257` carries the verb and is in the rename radius. |
| `POL-002` | Facet 3's `Unavailable { missing, remedy }` survives, and is *promoted* — it lifts to the top of the verdict rather than sitting inside a row-failure enum. The shell-absence case keeps the same shape from the payload's side (`D10`). |
| `RFC-025` | *"Not ranked"* does not become *"nothing can fail"*: the floor is closed, reduced, and cannot be vacuously held (`I2`, `I4`). |
| `ADR-021` | No `unsafe` added in the kernel (`A3`). |
| `AGENTS.md` pure/imperative split | Honoured by the kernel after the split, and **not** honoured by the code today (`RF-9`, § 3.3). The kernel performs no I/O (`I10`). |
| `AGENTS.md` behaviour-preservation gate | Not literally satisfiable across three type-shape changes, which is why `DEC-199` restates the bar at the level the suite measures. § 9.1 carries the restatement and `EVD-022` the pre-split half of the bracket. |

<!-- doctrine:section sec-6 -->
## 6. Open Questions & Unknowns

The design run opened eight inquiry nodes and closed all eight; what follows is
what remains genuinely open, plus the two slice-level questions retained because
their reasoning still binds.

### Still open

- **`OQ-3` — does `DEC-189` have a fifth row?** Row 8's control removes a
  file-size resource bound enforced via `RLIMIT_FSIZE` on the child. Under a
  hypervisor that limit is enforced by the *guest* kernel, so it is guest-internal
  and host-blind — the same reasoning that struck rows 13 and 14 from the ported
  set. `inq-8` carried this and `DEC-198` did not resolve it, because the kernel's
  shape does not depend on the answer: an open `AssuranceKey` admits whatever
  membership the analysis lands on. It is a question for whoever writes a
  hypervisor backend, and it belongs to `DEC-189` rather than to this slice.
  **Impact if wrong: none on this design.** Recorded so the derivation is not
  silently taken as complete.
- **`OQ-4` — what should a host that fails a row report?** `RV-352` `F-8` left
  this open, and it is why `capsule-verify` is deliberately not wired into
  `just gate`. This slice does not close it — `DEC-199` routes around it by making
  `capsule-verify` a phase exit criterion rather than a gate leg. Closing it would
  decide whether a developer laptop failing row 11 should redden CI.

### Retained from the slice scope, because the reasoning binds

- **`OQ-1` — Firecracker: implement or derive?** Resolved narrow by the owner:
  derive the membership, build no backend. Retained because it constrains later
  phases — the kernel must be *shaped* by `DEC-189`'s analysis without
  *implementing* it, and `A2` records how that shaping is discharged checkably
  (a mechanism minting an `AssuranceKey` edits no kernel file) rather than in
  prose.
- **`OQ-2` — where does the admission floor sit?** Closed by `DEC-195`, carried
  as `inq-1`/`inq-2`. Retained because `EVD-021`'s reasoning binds the
  implementation and not merely the decision: the vacuous path is real, and its
  only current guard is external and per-suite.

### Known unknowns, named rather than resolved

- **The ~67-test triage is an estimate.** `DEC-200`'s figure comes from a crude
  symbol-based classification: 46 tests name only kernel symbols, 73 only payload
  symbols, 28 name both, 39 name neither. The 67 needing judgement is
  `28 + 39`, and it sizes the work for budgeting — it is not a classification and
  the phase doing the carve will find the real split.
- **The kernel's final size is not predicted.** Deliberately (`P1`). § 2.1's
  measurement describes the current shape and is not a target.
- **Whether `AssuranceKey` collisions ever matter in practice.** Two mechanisms
  could mint the same string. The verdict records `backend: BackendId`, so a
  reader always knows whose profile they hold, and no cross-mechanism comparison
  is defined. If one is ever defined, this becomes a real question.

<!-- doctrine:section sec-7 -->
## 7. Decisions, Rationale & Alternatives

Seven decisions are banked as durable records and are **not restated here** —
each `DEC-` id below links the full context, choice, rationale and refused
alternatives. This section carries what the records do not: how they compose, and
the design-local rulings taken while writing § 5 that no record covers.

### 7.1 The banked decisions, and how they compose

| id | the move | what it settles |
|---|---|---|
| `DEC-195` | floor + profile + a named admission axis carrying no outcome | the verdict's shape; floor membership is row 3 alone |
| `DEC-196` | the seam cuts at row **identity**, not at the row | what is kernel and what is payload |
| `DEC-197` | the kernel classifies `leaf`; `BackendId` and `Availability` move into it | where the kernel sits in `ADR-001`'s map, and what it may import |
| `DEC-198` | row identity splits: closed floor key, open assurance key, closed axis | how one type stopped having to be both closed and open |
| `DEC-199` | the preservation bar is the nineteen row verdicts | what *green* means for the rest of this slice |
| `DEC-200` | test bands carved before the split, two bands | when the ~67-test triage is paid for |
| `DEC-201` | one REV, four payloads | what `SPEC-030` says once the code lands |

They compose in one direction and it is worth naming, because a reader arriving
at any single record will not see it. `DEC-195` decides *what the verdict is*.
`DEC-196` decides *where the cut falls* to produce it — and cuts at identity
precisely because `DEC-195` made identity the thing the floor reduces over.
`DEC-198` then discovers that identity itself must split, because `DEC-195`
wants it closed and `REQ-459` criterion 3 wants it open; `DEC-196`'s claim to
have largely answered `inq-8` in advance was withdrawn on that finding.
`DEC-197` places the result. `DEC-199` is downstream of all four, because it is
the three *shape* changes (`DEC-196`, `DEC-197`, `DEC-198`) that make *suites
green unchanged* literally false and force the bar to be restated. `DEC-200` and
`DEC-201` are the two costs the others incur.

The composition also runs backwards once, and that is what re-cut `DEC-197`.
`DEC-198` minted `AssuranceKey` *after* `DEC-197` had ruled on `BackendId`'s
placement, and the two are the same construction for the same reason — so a
decision taken at `inq-4` was falsified by one taken at `inq-8`. A reader working
forward through the records will not see that; it is why `DEC-197` carries a
re-cut rider dated after its own acceptance.

**A row in this table is a summary and never the record.** Two clauses of
`DEC-197` in particular do not fit in one: that the kernel's imports are `std`
alone, and that `D10` narrows the record's own `&dyn CapsuleBackend` clause
(below). Read the record.

### 7.2 Design-local rulings, taken here

These are `D`-numbered because they are doc-local: they refine the records
without contradicting them, and none was large enough to warrant its own.
`D7`–`D10` were taken during the first review pass and `D11`–`D12` during the
second; each names the finding that forced it.

**`D1` — Only `Proven` holds the floor, and only the floor.** `DEC-195` says the
floor is reduced by exhaustive match; it does not say what `Unproven` does to it.
Ruled: `Violated`, `Unproven` and `Indeterminate` all breach. *Rationale:* a
control that did not fire licenses no inference, and a floor is a claim rather
than a report. *Scope:* the floor alone — the assurance profile publishes
`Unproven` verbatim, because nothing reduces over it and a report is not a claim.
*Alternative refused:* treat `Unproven` as non-fatal on the floor, which would
mean an authority property could be recorded as holding on the strength of a
control that could not fire — the `B4` defect class the round-6 split exists to
remove. Owner-confirmed. Pinned by `I3`.

**`D2` — `Floor` is a struct with one field per member, not a map or a `Vec`.**
`DEC-195` requires two guarantees: an empty floor unrepresentable, and adding a
member failing to compile. *Rationale:* a struct gives both by construction —
construction is total, so there is no empty case; and a new field breaks every
construction site. A `BTreeMap<FloorProperty, RowVerdict>` gives neither, and a
`const ALL: [FloorProperty; N]` array gives only the second, and only if someone
remembers to extend `ALL` — the array length does not force it. *Cost:* at one
member the struct looks like ceremony. Accepted: it is exactly right as the floor
grows, and the floor growing is `ADR-020`'s business, not a mechanism's.

**`D3` — the floor's only constructor may refuse, and the refusal is a state.**
*Rationale:* this is where `EVD-021` actually dies. A floor built by filtering a
row list can be built from an empty list; a floor that can only be built by a
function which refuses a missing member cannot. The repair moves from an
assertion in a test about a different function's return value into the type's
only entry point. *Consequence:* *the question was never asked* is a distinct
claim from *the answer was no*, so it needs somewhere to live — and `D7` is where
the draft failed to put it.

**`D4` — no stored summary word on the verdict.** `Qualified`/`Disqualified` is
computed at the command tier from `floor.standing()`. *Rationale:* a stored
scalar can disagree with the rows it was computed from, and that disagreement is
`DEC-191`'s original complaint in miniature. *Alternative refused:* a cached
`outcome` field, which is what exists today and is exactly what is being removed.

**`D5` — `today` → `observed_at`, on the parameter and the field.** The existing
names are `today: String` feeding `AdmissionVerdict::date`. *Rationale:* `today`
is relative to now, and the value exists to be read back later from a recorded
artefact — a verdict from March saying `today` tells a reader nothing; and `date`
does not say what the date is *of*. *Cost:* two signatures, three construction
sites, `main.rs:186`, two tests — all of which `DEC-194`'s rename touches anyway,
so the alternative is a second pass over the same call sites. *Scope, narrowed by
`D9`:* source identifiers only. Owner-raised.

**`D6` — the kernel unit is named `qualification`, the payload keeps
`conformance`.** *Rationale:* `DEC-194` renames the mechanism axis to
*qualification*, and adjudicating that axis is precisely what the kernel does;
what remains is the bubblewrap suite submitting itself to it. *Alternative
considered:* `verdict`, which names the output rather than the judgement, and
would leave `conformance` ambiguous between the two halves. All type names remain
provisional per `DEC-198`.

**`D7` — `FloorReading` wraps `Floor`; the missing-row state lives one level
out.** Forced by `RF-1`: `D3` said a missing floor row must be reportable and
`D2` said `Floor` is total, and the draft's types could not both be true —
`Qualification::Ran` held a `Floor`, so `FloorStanding::NotEstablished` was
unreachable and § 9.6 named a test for a state nothing could produce.
*Rationale:* the two guarantees are about different questions — *what did the
floor say* and *did we get a floor at all* — so they belong at different levels.
`Floor` keeps its totality; `FloorReading::{Established, NotEstablished}` carries
the second question; `standing()` moves to the wrapper and is total over it.
*Alternatives refused:* `floor: Result<Floor, FloorProperty>`, because a `Result`
in a verdict field reads as a failure to compute rather than a state of the run;
and a third `Qualification` variant, which would discard the assurance rows that
did run and so contradict `P4`.

**`D8` — fronts stay wholly in the payload.** Forced by `RF-5`: the draft made
front-labelled rendering a `CPT-002` obligation and gave no type, field or
contract function anything to compute it from. *Ruling:* the payload's table
declares each row's front and answers for one; the command tier consults it when
rendering; the kernel never names a front. *Rationale:* the kernel neither
reduces over fronts nor validates them, so a kernel `Front` would be a type a
reader must load to understand nothing the kernel does (`P1`), and it fails
§ 5.1's placement criterion — a label on a claim is not a distinction the claim
algebra observes; and `A4` keeps the list a sketch, which is the one case `P3`
does not cover — `P3` justifies naming shapes the kernel must *reason* about.
*Alternative refused:* a `Front` newtype in the kernel with an
`AssuranceRow { key, front, verdict }` triple, on the argument that a recorded
verdict must carry its own labels. It does not: the verdict is rendered by the
process that built it, so the label is available at render time without being
stored. *Amended by `D11`*, which supplies the transport `D8` left missing.

**`D9` — `D5`'s rename is source-side only; the rendered key stays `date=`.**
Forced by `RF-2`: `render_verdict` emits `date={}` on the header line, and the
header line is unchanged by the split, so re-spelling one of its keys is a
difference this slice's preservation licence does not cover. *Ruling:* rename the
Rust identifiers, leave the rendered spelling. *Rationale:* `D5`'s argument is
about a reader of the *source* being misled, and that is fully bought by the
source rename. Widening the preservation licence costs a decision record and
dilutes the one instrument standing between this slice and an unnoticed behaviour
change; the rendered spelling is not worth that. *Follow-up:* the rendered key is
worth revisiting once the licence is spent.

**`D10` — the kernel's entry point takes values and closures, and nothing
else.** Forced by `RF-3`, `RF-7` and `RF-9` together, and by `F7` behind them.
`qualify_over` takes `BackendId` and `Availability` as values rather than
`&dyn ConformanceBackend` or `&dyn CapsuleBackend`; `HostDescriptor` as a value
rather than `&dyn HostFacts` plus a disk read; and knows nothing of `/bin/sh`,
because composing the backend's availability with the shell the *probes* need is
the payload's job. *Rationale:* `verify_over` called exactly `id()` and
`availability()` on its backend, so taking their results removes the trait
instead of narrowing it — and with the shell check and the descriptor read moved
out, `&dyn HostFacts` has no remaining use either. The stronger reason is `F7`:
three enumerations of *what crosses backwards* have each been wrong, and this
shape removes the parameters such a reference could arrive on. *Alternative
refused:* narrowing to `&dyn CapsuleBackend`, which the draft took. It is
strictly weaker — the trait still names a mechanism contract, and it leaves a
`&dyn HostFacts` beside it that the shell check was riding in on.

*Relation to `DEC-197`.* `D10` **narrows a clause of the record it sits under**,
which is the one thing a doc-local ruling is normally not allowed to do, so it is
stated rather than left to be noticed: `DEC-197`'s original rider had
`verify_over`'s backend parameter *narrow* to `&dyn CapsuleBackend`; `D10` drops
the parameter outright, so no backend trait survives in the kernel at all. The
re-cut record now carries this. § 7.1's summary row cannot, which is why the row
says to read the record.

*Known limit, found by `RF-10`.* `D10` is a rule about the entry point's
**parameter list**, and `row_verdict` is a second kernel-bound function the
payload calls directly (§ 5.4). A mechanism type can therefore still arrive
inside a *field* of a value that satisfies `D10` — which is exactly what
`ArmResult::Indeterminate`'s `termination` did. `D12` is that repair; `I7` and
the compile probe are what generalise it.

**`D11` — the payload returns a `FrontCatalog` beside the verdict.** Forced by
`RF-12`: `D8` told the command tier to ask the payload's table for a front, and
nothing hands the command tier a table — `render_verdict` takes only the verdict,
and the payload builds its table inside its own entry point
(`conformance.rs:5254`). *Ruling:* the payload's entry point returns a transient
`QualificationRun { verdict, fronts }`; `FrontCatalog::front_of` is keyed by
`AssuranceKey` and is total. *Rationale:* keying by `AssuranceKey` rather than
`RowId` matches what the verdict's `assurance` vector actually holds, so no call
site has to rewrap `RowId::Assurance(key)` first; and totality follows from `I9`,
because the catalogue is derived from the same table value that produced the ids.
*Alternatives refused:* return the whole `Table` to the command tier — refused
because it hands the probes, the deltas and the fixture to the tier that needs a
string; and a free global front lookup over the payload's table constant —
refused because it can be asked about a key from a table nobody submitted, which
is precisely the guarantee `I9` exists to give and the reason `run_row` is
infallible.

**`D12` — the kernel takes `ArmJudgement`; an arm's diagnostics stay
payload-side.** Forced by `RF-10`: `ArmResult::Indeterminate` carries
`termination: Termination` (`backend.rs:751`), so moving `ArmResult` into the
kernel would carry a mechanism type in with it — a sixth backward reference, in a
location class (a type's field) no enumeration had looked at. *Ruling:*
`row_verdict` takes `ArmJudgement { Held, Failed, Indeterminate(Indeterminacy) }`;
the payload keeps `ArmResult` and projects with `into_judgement()`. *Rationale:*
§ 5.1's equivalence test decides it mechanically — `row_verdict` matches
`Indeterminate { reason, .. }` and discards `termination`, `stdout` and `stderr`,
so the quotient is strictly narrower than the type and the type must be projected
rather than moved. `Indeterminacy::BackendError` already launders `BackendError`
to a `String` for exactly this reason (`:881-884`); `termination` is the sibling
field that escaped the discipline. *Cost, and it is a real one:* § 5.2.5's
**byte-identical** claim is surrendered, and § 9.6's `row_verdict_is_unchanged`
becomes a truth table over all nine probe/control pairs plus a projection test.
*Alternative refused:* leave `row_verdict` payload-side entirely — see § 7.3.

### 7.3 What was considered and refused at design level

Recorded because a later reader will re-propose them:

- **Reduce per front rather than per row.** Refused: `DEC-189` guarantees empty
  fronts, and a front with no rows is vacuously strong — the vacuity is a
  property of reducing over something that can be empty, not of the axis reduced
  on (F2). Fronts group; they never reduce.
- **Keep one closed `Property` enum and grow it per mechanism.** Refused as
  precisely what `REQ-459` criterion 3 forbids, and because it puts a microVM's
  vocabulary in a leaf the microVM does not own.
- **Make `Property` wholly open and drop the enum.** Refused: with no closed set
  there is no exhaustive match, and `EVD-021`'s vacuous path returns as a runtime
  concern rather than a compile-time impossibility.
- **Re-export `BackendId` from the kernel through `backend`.** Refused: a second
  name for one type across one dependency edge (`P6`). `DEC-197`'s re-cut *moves*
  the type instead, so there is one name and the edge points the other way. The
  draft's competing fear — that moving it would invert an `ADR-001` edge and
  cascade into `transaction` and `provision` — was a misreading: the kernel is a
  `leaf`, so `backend` importing it is a leaf→leaf edge this tree already has
  several of (§ 5.1).
- **Project `Availability` into a kernel-owned value instead of moving the
  type.** Refused as an isomorphic copy: every distinction it carries is consumed
  by the kernel and reproduced in `Qualification::Unavailable`, so the projection
  preserves everything and buys nothing — a second name, a second maintenance
  site, and a fresh judgement of the class that has already been wrong four times
  (§ 5.1's equivalence test).
- **Leave `row_verdict` on the payload side**, since its inputs are arms and
  § 5.1 already places `Arm`, `ArmShape` and `run_arm` there. This was `RF-10`'s
  own first-offered repair and it is the cheaper one. Refused: the two-arm algebra
  *is* the judgement. `DEC-190`'s central claim is that a reader can form a
  defensible opinion about a row without loading a confinement mechanism, and the
  nine-cell match is the thing they must read to do it — a kernel that adjudicates
  rows but not arms would leave the actual reasoning on the other side of the
  seam. Projecting the input is the smaller move and keeps the kernel's reason for
  existing inside it (`D12`).
- **Extract the kernel as its own crate.** The only shape that makes `I7`
  compiler-proven in production, since Rust has no per-module import restriction
  within a crate. Refused for this slice on scope, not principle: it turns every
  kernel item `pub(crate)` → `pub`, the layering gate is blind across crate
  boundaries so `DEC-197`'s exit criterion would be falsified rather than met, and
  `just gate`'s `test-all` names its packages so a new member would not be
  auto-gated. `IMP-404` holds it, with structural promotion triggers rather than a
  date.
- **Parameterise the kernel over the mechanism's fixture type** (`Delta<F>`,
  `Row<F>`). Refused as generics ceremony for a second backend `OQ-1` has ruled
  out building.
- **A fallible row runner** — `run_row: &dyn Fn(&RowId) -> Option<RowVerdict>`,
  so a payload handed an identity it cannot construct could say so (`RF-6`).
  Refused: `I9` makes that unreachable, because `Table::ids()` and
  `Table::row_for` come from one table value. An error arm nothing can produce is
  worse than none — it is an invitation to fabricate a verdict at the one place a
  fabricated verdict would be invisible.

<!-- doctrine:section sec-8 -->
## 8. Risks & Mitigations

Ordered by what would cost most to discover late.

### `R1` — The split lands and the seam still leaks

**What goes wrong:** a kernel type keeps a mechanism-shaped reference that nobody
notices, because the code compiles and the tests pass. The kernel is then not
independently reviewable, which was the entire point (`RSK-231`).

**Why it is plausible:** three enumerations of the backward references have now
been made and all three were wrong. `DEC-196` named two of five. `inq-4` found
`ConformanceBackend` and the draft named three. The review pass found the shell
precondition (`RF-3`) and the host-descriptor read (`RF-9`), both in the body of
`verify_over` rather than its signature, where every previous pass had been
looking. This is `F7`, and its lesson is not *look harder*.

**Mitigation, in two layers of different strength.**

1. *Structural, and the one that carries the weight.* The kernel's entry point
   takes values and three closures — no `&dyn` backend, no `&dyn` host, no path,
   no environment (`D10`, `I10`). References 2, 4 and 5 all arrived on parameters
   this shape does not have. A leak now requires the kernel to *originate* a
   mechanism fact rather than merely accept one.
2. *Machine-checked, and narrower than the draft claimed.* The `leaf`
   classification (`I7`) refuses a kernel that **imports** `conformance`. It
   would not have caught references 4 or 5: a kernel re-declaring
   `const SHELL: &str = "/bin/sh"` and calling `std::fs::read_to_string` imports
   nothing and classifies cleanly. `DEC-197` makes the classification a required
   exit criterion and that stands — but it is a check on the import graph, not on
   the vocabulary, and the draft's claim that it was *the* answer to this risk
   was wrong.

**Residual:** a kernel that re-derives a mechanism fact from `std` alone is still
writable. Nothing here catches it; the review lens in § 9.1 layer 3 is where it
would surface, and a reviewer's question for any kernel constant is *whose fact
is this?*

### `R2` — Behaviour preservation is asserted rather than demonstrated

**What goes wrong:** the row verdicts drift and the slice closes anyway, because
*green* was checked against a gate that never ran the suite. `just gate` names
its packages and `doctrine-control` is not among them (F4).

**Mitigation:** `capsule-check` every phase, `capsule-verify` as a phase exit
criterion (§ 9.4), plus two independent artefacts — the characterisation test,
which fails at the phase that broke it, and the key translation table, which
makes the post-split comparison mechanical. `EVD-022` is the pre half of the
bracket and was captured before any code landed, which is the part that could
not have been recovered later.

**Residual:** **both** instruments need `bwrap`, not just `capsule-verify`
(`RF-4`). `capsule-check` runs `cargo test -p doctrine-control`, whose suites
include tests asserting `availability() == Available` and provisioning real
capsules, and `EX-14` forbids them skipping. Nested `bwrap` works inside the
project jail, so the development host is covered and this slice's proof runs; a
host without `bwrap` has **no** instrument at all, and `IMP-427`'s third band is
what would eventually give it one.

### `R3` — The test carve is under-budgeted and bleeds into later phases

**What goes wrong:** the ~67 judgement calls turn out to be ~120, the carving
phase overruns, and triage decisions start being made inside phases that are
supposed to be moving types.

**Why it is plausible:** the figure is a crude symbol-based estimate (§ 6), and
28 tests naming *both* kernel and payload symbols are the ones most likely to
need splitting rather than sorting.

**Mitigation:** `DEC-200` puts the carve **before** any code moves, against
stable types, with a mechanical exit criterion — same 186 names, all green,
nothing semantic changed. An overrun is then visible as an overrun of one phase
rather than absorbed invisibly across five. A test that genuinely cannot be
sorted without changing its body is evidence for `IMP-427`, not work for this
phase.

### `R4` — The REV and the code fall out of step

**What goes wrong:** a commit exists in which `SPEC-030` says one thing and the
binary does another. Worse: the code lands and the REV does not, leaving
`REQ-459` criterion 3 and `REV-051`'s applied disposition both asserting
something the shipped kernel contradicts.

**Mitigation:** `DEC-201` — one REV carrying all four payloads, landing **with**
the code. Splitting it into two REVs was refused for exactly this reason: the
payloads must be true together, and separating them opens a window in which they
are not.

**Residual:** the REV widens `REQ-459` criterion 3's *text*, which is a scope
widening the owner took explicitly. If that widening is later judged too broad,
the correction is a further revision rather than a reversal of this one.

### `R5` — `DEC-194`'s rename becomes a second migration

**What goes wrong:** the rename is deferred as "not essential to the split", and
the same call sites are edited twice.

**Mitigation:** it rides this slice by `DEC-194`'s own consequence clause, and
§ 5.4 lands it across the whole surface at once. `D5`'s `today` → `observed_at`
folds into the same pass for the same reason — as a source rename only, `D9`.

### `R6` — The single-member floor reads as over-engineering and gets simplified

**What goes wrong:** a later reader sees a struct with one field, a wrapper enum
around it, and a three-variant standing, judges the lot ceremony, and replaces it
with a lookup or a boolean. `EVD-021`'s defect returns.

**Why it is plausible:** it genuinely does look like ceremony at one member, and
the reasoning for it lives in a decision record rather than in the code. `D7`
adds a second type to the same one-member set, which makes the appearance worse
before the floor grows.

**Mitigation:** `D2`, `D3` and `D7` state the reasoning in this document; § 9.6
pins all three guarantees with named tests
(`an_empty_row_list_cannot_build_a_floor`,
`a_missing_floor_row_is_not_established_not_breached`,
`a_missing_floor_row_still_publishes_the_rows_that_ran`); and the implementation
should carry the reasoning in a doc comment on `Floor` and `FloorReading`, citing
`EVD-021` by id, so the argument is where the temptation is.

### `R7` — `A2`'s derivation is decoration after all

**What goes wrong:** the Firecracker membership analysis constrains nothing, and
`OQ-1`'s narrow resolution turns out to have bought only prose.

**Mitigation:** the open `AssuranceKey` is the check. A second mechanism mints
its own keys and edits **no kernel file** — that is a falsifiable claim about the
kernel's shape, not an assertion about a backend nobody has written. If a
hypervisor backend ever requires a kernel edit to publish its rows, this design
was wrong and the failure is visible as a diff. `D10` strengthens it: a second
mechanism also composes its own availability and derives its own host facts, so
none of the three things a backend differs about reaches the kernel as a type.

<!-- doctrine:section sec-9 -->
## 9. Quality Engineering & Validation

`DEC-199` settles the bar and the instruments. This section makes both
executable.

### 9.1 The bar, in three layers

**Layer 1 — invariant.** The nineteen row-to-verdict mappings, the four auxiliary
claims and the two unrowed readings reproduce **exactly**. `EVD-022` is the
pre-split half of the bracket: `just capsule-verify` at `4662e64eb`, in-jail,
before any code lands — `outcome=admitted`, all nineteen rows `Proven` (fourteen
properties, five axes), four claims `Passed`, two observations `Read`. It
reproduces `RV-352`'s baseline claim and is **uncapturable after the first line
of this slice lands**.

**Layer 2 — a closed transformation contract.** Not a list of exceptions. A
**total map** from the pre-split artefact's lines to the post-split artefact's,
and the two totality clauses are where the strength is:

> Every pre-split line has **exactly one** successor — so nothing may silently
> disappear. Every post-split line has **exactly one** predecessor — so nothing
> may silently appear.

| # | pre-split line | successor | how it is derived |
|---|---|---|---|
| 1 | `backend=… os=… kernel=… arch=… date=…` | byte-identical | the header survives the split untouched; `D9` keeps the rendered `date=` key |
| 2 | `outcome=<word>` | the floor's standing, then the published profile | `DEC-191`'s one licensed *semantic* change |
| 3 | `row Property(DeniedCanonicalStateAndCredentials)=V` | `row Floor(DeniedCanonicalStateAndCredentials)=V` | **from `FloorReading`**, not from the row list — the floor row has no `(RowId, RowVerdict)` entry after the split (§ 5.2.2) and would otherwise vanish |
| 4 | `row Property(X)=V` — the other thirteen | `row Assurance(<key>)=V front=<front>` | key spelling per `DEC-198`; the front label rides the **existing** line |
| 5 | `row Axis(A)=V` — five lines | byte-identical | the renderer reconstructs `RowId::Axis(axis)` from the verdict's `axes` vector |
| 6 | `claim <section>/<name>=<outcome>` — four lines | byte-identical | untouched by the split |
| 7 | `observation <section>/<name>=<reading>` — two lines | byte-identical | untouched by the split |

Two clauses sit outside the line map because they are not about a line's text:

- **Ordering.** The assurance block is ordered by front. That is a stated
  permutation of rule 4's outputs, not an unlicensed difference — and it is how
  grouping is expressed, since a group *header* would be a line with no
  predecessor and the second totality clause rejects it.
- **The exit code.** `EXIT_REFUSED` → `EXIT_DISQUALIFIED` (`DEC-194`). A process
  exit code, not a line of the artefact.

**A line the contract does not derive is a regression.** So is a pre-split line
with no successor. The contract may be amended — deliberately, in a decision
record — but it cannot be quietly widened, which is the whole reason it replaced
an exception list (`RF-11`). Under an enumerated licence an unlisted difference
is a regression *if a reviewer notices at audit*; under a total map, the noticing
is done by a test.

**The instrument is a whole-output golden test**, committed in the same phase as
§ 9.2's key translation table and asserting the post-split artefact verbatim.
The translation table supplies rule 4's key and front columns; the golden test is
what makes the two totality clauses machine-checked rather than reviewer
discipline. Written before the phase that changes `RowId`, because that phase is
the first that can break the contract.

The contract has already decided two things. `D9`: rule 1 is byte-identical, so
renaming the rendered `date=` key has no derivation and the rename stays
source-side (`RF-2`). And `RF-11`: § 5.4's rendering constraints are not
stylistic preferences — no standalone group headers, one self-contained line per
assurance row, a derived successor for the floor row, byte-identical axis lines
— they are what rules 3, 4 and 5 and the two totality clauses require of the
renderer.

**Layer 3 — mechanical.** Test source naming moved types is translated and
reviewed as a *translation diff in which no asserted value moves*. A reviewer's
question for every hunk in that class is: did any expected value change? If yes,
it is not a translation. `R1`'s residual adds a second question for kernel source
specifically — *whose fact is this?* — because a constant the kernel re-derives
from `std` alone is the one leak no gate in this design catches.

### 9.2 The key translation table

Authored as a **committed artefact in the phase that changes `RowId`**, so the
post-split comparison is mechanical rather than a judgement made at audit. Its
shape, with the classification this design implies:

| row | current `Property` member | after |
|---|---|---|
| 1 | `FreshMutableState` | assurance |
| 2 | `BoundedInputSet` | assurance |
| **3** | **`DeniedCanonicalStateAndCredentials`** | **floor** — the whole floor |
| 4 | `BoundedFilesystemVisibility` | assurance |
| 5 | `ExplicitNetworkPosture` | assurance |
| 6 | `DeterministicWorkingDirectory` | assurance |
| 7 | `ProcessTreeTeardown` | assurance |
| 8 | `TrustedTerminationObservation` | assurance |
| 9 | `ImmutableInputSet` | assurance |
| 10 | `ClosedDescriptorSet` | assurance |
| 11 | `ClosedEnvironment` | assurance |
| 12 | `OwnedStandardStreams` | assurance |
| 13 | `MappedCapsuleIdentity` | assurance |
| 14 | `ConfinedCapabilities` | assurance |
| B1–B5 | `Axis::{Checkout, Repository, Runtime, TemporaryState, Process}` | unchanged — closed, kernel |

Thirteen of fourteen become payload-minted `AssuranceKey` constants. The table
carries a **third column for each assurance row's front**, because `D8` puts the
front list in the payload's table and this is where that table's contents are
first written down.

Those two columns are exactly § 9.1's rule 4 — the new key and the front label —
so the table is not merely a reader's aid: it is the contract's input, and the
golden test consumes what it says.

Row 8's name is a standing trap and the table is where it gets corrected in
writing: `TrustedTerminationObservation` reads epistemic and is a **file-size
resource bound** (`RLIMIT_FSIZE` on the child). Read a row's `delta`, never its
name.

The table's other job is `OQ-3`: rows 10, 12, 13 and 14 are the four `DEC-189`
strikes from a hypervisor's set, and row 8 is the open candidate for a fifth.
Recording them beside the translation keeps the derivation legible to whoever
writes the second backend.

### 9.3 The characterisation test

Written **before the split**, against stable types, and carried through it
mechanically. It records the row-to-verdict mapping **as data** rather than as
prose, so `EVD-022`'s invariance becomes an executable assertion.

- **Why before:** after the split there is nothing to characterise — the shape it
  would pin has already moved.
- **Why data:** a table of `(row key, expected verdict)` translates through
  `DEC-198`'s key change by editing the key column and nothing else, which is a
  layer-3 translation diff a reviewer can check by eye.
- **What it buys:** the invariance fails **at the phase that broke it**, not at
  audit. `DEC-199` took both this and the artefact comparison rather than
  choosing: an executable assertion localises the failure, a compared artefact
  catches what no assertion was written for.
- **Its lifespan is two phases and that is accepted.** It is scaffolding written
  against types that are about to move; `P1` justifies the cost by what it
  returns, not by being cheap.

It and the golden test are **not redundant**, and the distinction is worth
holding: the characterisation test pins layer 1 (the row-to-verdict mapping,
which must not change), the golden test pins layer 2 (the artefact's text, which
changes in exactly the ways the contract derives). A regression in the algebra
fails the first; a regression in the rendering fails the second.

### 9.4 Instruments, and how the proof is run

Neither recipe is wired into `just gate`, and `just gate`'s `test-all` **names**
its packages (`-p doctrine -p cordage`) rather than using `--workspace`, so
`doctrine-control` is reached by neither (`RV-353` `F-4`). A phase can therefore
be green and prove nothing about this slice — F4. The routing:

| instrument | when | needs `bwrap`? |
|---|---|---|
| `just capsule-check` | **every phase**, green at its end | **yes**, for part of the suite |
| `just capsule-verify` | **phase exit criterion** for every phase touching the payload, and the default wherever it is arguable — only a phase that plainly cannot reach a row omits it | yes |

The characterisation test and the golden test are not a third instrument: both
are `#[cfg(test)]` tests in `doctrine-control` and ride `capsule-check`. What
`capsule-verify` adds over them is the *live* artefact — the thing `EVD-022`
captured and the thing § 9.1's contract is a contract about.

**Both need `bwrap`** (`RF-4`), which the draft got wrong for `capsule-check` —
as did `DEC-199` until its re-cut withdrew the claim. `cargo test -p
doctrine-control` runs `#[cfg(test)]` tests that assert
`backend.availability() == Availability::Available` and provision real capsules
(`conformance.rs:6915`, `:7244`, `:7453`), and `EX-14` forbids them skipping
instead, so on a host without `bwrap` they fail rather than pass thinly. The
consequence for this slice is small and the consequence for the statement is not:
`capsule-check` is a **narrower** instrument than the draft advertised, and § 9.5
and `IMP-427` were already written as though this were true.

Nested `bwrap` works inside the project jail, so both run on the development
host; `ISS-339`'s never-run-off-jail note is not a blocker, and `EVD-022` is the
run that demonstrates it. Capsule time-to-interactive is ~2 min from
`capsule-baseline` (owner, 2026-08-12), which is cheap enough that `DEC-199`'s
default leans to running it more often rather than less.

### 9.5 Test bands

`DEC-200`: the flat `#[cfg(test)] mod tests` (186 functions, no inner module) is
sub-moduled into **two** bands — kernel and payload — as a pure reorganisation
against stable types, in the same pre-split phase as the characterisation test.

- **Exit criterion is mechanical:** the same 186 test names, all green, nothing
  semantic changed. A test whose *body* changes in this phase is out of scope for
  it.
- **~67 need individual judgement** (28 name both kernel and payload symbols, 39
  name neither); 119 sort themselves by symbol. The figure sizes the phase; it is
  not a classification.
- **Two bands, not three.** The live-`bwrap`-versus-neutral split inside the
  payload is deferred to `IMP-427` — and it must never become the skip `EX-14`
  and `DEC-156` forbid. § 9.4 is why that deferral has a cost: until it lands,
  there is no instrument at all on a host without `bwrap`.

One band assignment is settled by `RF-10` rather than by symbol:
`an_indeterminate_arm_carries_its_termination_and_output` (`:6176`) stays
**payload**-side. It asserts what `ArmResult` carries, and `ArmResult` does not
cross the seam (§ 5.2.5).

### 9.6 New tests this design requires

Beyond the carve and the characterisation test:

| test | what it pins |
|---|---|
| `an_empty_row_list_cannot_build_a_floor` | `I2` — the direct successor to `EVD-021`'s test, asserting `NotEstablished` where the old one asserted `Admitted` |
| `a_missing_floor_row_is_not_established_not_breached` | `D3`/`D7` — the third `FloorStanding` variant is reachable and distinct |
| `a_missing_floor_row_still_publishes_the_rows_that_ran` | `D7`'s refused alternative — a third `Qualification` variant would have discarded them (`P4`) |
| `only_proven_holds_the_floor` | `I3`, one case per `RowVerdict` variant |
| `an_unproven_assurance_row_is_published_verbatim` | `I3`'s scope — the profile is not subject to the floor's strictness |
| `an_empty_assurance_profile_qualifies_when_the_floor_holds` | the `DEC-189` empty-front case, and that nothing reduces over the profile |
| `unavailability_carries_no_rows_claims_or_observations` | `DEC-195`'s lift — not-run and ran-and-failed are different claims |
| `a_host_without_a_shell_is_unavailable_not_violated` | `RF-3` — the behaviour is preserved exactly across the move to the payload |
| `row_verdict_is_a_truth_table_over_nine_arm_pairs` | § 5.2.5 — the algebra itself, all nine `ArmJudgement` probe/control pairs enumerated as data. **Replaces `row_verdict_is_unchanged`**, which `RF-10` made impossible: the signature changes, so there is no byte comparison to make |
| `arm_diagnostics_do_not_move_a_judgement` | `D12` — `ArmResult`s differing only in `termination`, `stdout` or `stderr` project to the same `ArmJudgement`, which is § 5.1's equivalence test made executable |
| `every_submitted_assurance_key_has_a_front` | `I9`/`D11` — `FrontCatalog` is total over the keys its own table submitted, which is why `front_of` returns no `Option` |
| `the_qualification_artefact_matches_the_transformation_contract` | § 9.1 layer 2 — the whole-output golden test. Committed in the phase that changes `RowId`, beside § 9.2's table |

The compile probe (`DEC-197`, § 5.1) is not in this table because it is not a
test: it is a `harness = false` cargo target whose *compilation* is the
assertion, and `I7` is what it pins. It needs a negative control at
implementation — a probe that cannot fail proves nothing — and § 10.3 records
that the control's shape is deliberately left to implementation.

`I10` — the kernel takes values and closures only — is pinned by its own
signature and by review, not by a test: there is nothing to execute, because the
claim is about what the parameter list does not contain. `RF-10` is the standing
reminder of that pin's limit: it says nothing about what rides *inside* a
parameter's type, and `I7` and the compile probe are what cover that.

`an_empty_row_list_is_admitted_and_the_shipped_tables_are_what_prevent_it`
(`:11275`) is **retired with its reasoning recorded**, not deleted silently: it
asserted a defect held shut by an external guard, and this design removes the
defect rather than the guard.

