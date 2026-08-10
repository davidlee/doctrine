# Review RV-352 — reconciliation of SL-248

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance. **Facet:** reconciliation. **Subject:** `SL-248` —
capsule provisioning and Linux backend.

**Surface reviewed.** Branch `sl-248`, tip `cd0d85977`, audited in the linked
worktree `.worktrees/SL-248` against the merge-base `dd8a7f7b6` — *not* against
`edge` HEAD, which has moved 80 commits independently. `SL-248` was driven from a
separate clone rather than by `/dispatch`, so there is no candidate interaction
branch and no `dispatch candidate admit` edge; the evidence refs are the branch
itself plus the tracked artefacts under `.doctrine/slice/248/`.

**Implementation surface**, from `git diff dd8a7f7b6 sl-248`: the new
`crates/doctrine-control/` crate (9 modules, ~23k inserted lines, of which
`conformance.rs` is 14k), `src/interpretation.rs` (+44), `flake.nix` (+6),
`Cargo.toml`. Everything else in the diff is `.doctrine/` authored state.

**Governance claimed:** `ADR-020` (execution capsules as the dispatch authority
boundary), `SPEC-030` `REQ-448`–`REQ-461`, `DEC-134`, `DEC-136`. `REV-046` is the
cutover Revision and is deliberately still `proposed` — explicitly *not* this
slice's to apply. This slice ships machinery beside the incumbent worktree arms;
there is no flag day in it.

**Lines of attack.**

1. **Does the recorded evidence reproduce?** The slice's own verification ledger
   sets two reading conventions — anchor on the `doctrine_control-` binary header
   because that crate's `test result:` line is neither first nor last, and run the
   suite twice because it measures elapsed time. Both were honoured.
2. **Does `slice conformance` agree with what the slice says it says?** The
   mechanical drift signal, checked rather than quoted.
3. **Is the evidence durable?** This slice moved its verification ledger out of
   gitignored `handover.md` mid-flight on exactly this ground. The question
   generalises: what else load-bears on runtime state?
4. **Are the 176 owed items actually dischargeable?** The compact pointer lists
   were checked against the items they point at, not executed on faith.
5. **Vacuity.** The slice's dominant self-criticism is the vacuous pass — a green
   that proves nothing. Whether that lens was turned on its own closure evidence.

## Synthesis

**The implementation is sound and its own evidence reproduces exactly.** Two
independent `doctrine check gate` runs taken by the auditor, outside any worker's
process, both report `297 passed; 0 failed; 9 ignored` at 92.14 s and 92.35 s,
exit 0, 19 warning lines each — matching the slice's recorded close figure
digit-for-digit. `doctrine slice verify-vt 248` reports every `VT` row across all
ten phases PASS. `doctrine-control backend verify` exits 0 with all fourteen
properties and five axes `Proven` and every auxiliary claim `Passed`. Ten phases
report `completed`; the `slice status` divergence warning is the expected
lifecycle artefact this audit exists to resolve.

**That green is conditional on one host fact, and the condition is the most
instructive thing in the slice.** The auditing jail was built from `edge`'s
`flake.nix`, where this slice's `util-linux` addition has not landed, so `setsid`
did not resolve on `PATH`. Seven `conformance::tests` convicted — and convicted
*correctly*, with a stop-and-consult message naming the remedy and forbidding the
three wrong fixes (narrow the row, `#[ignore]` it, substitute another program).
`backend verify` corroborated to the exact row: thirteen of fourteen `Proven`,
one `Unproven`, and it was `Property(ProcessTreeTeardown)` — row 7. This is the
`S4` guard doing precisely what `notes.md` item 157 argued a guard is for, under
an auditor it was not built to anticipate. Restoring the store path the slice's
own notes record (`util-linux-2.42.2-bin`, rooted at `/nix`, a bound readable
root) turned the suite green with no other change. The slice's defence against
the vacuous pass held when it was tested from outside.

**Where the slice is weakest is not its code but its account of its own
conformance.** `verification-ledger.md` states, and `handover.md` repeats, that
`slice conformance` reads *24 files, nothing undeclared*. It does not, and cannot
have: the selector registry has not moved since `PHASE-01` and the ten recorded
boundary rows span 195 paths. The real algebra is 24 conformant, 0 undelivered,
**172 undeclared**. The finding is mostly benign in substance — 170 of the 172
are `.doctrine/` authored state swept in by the `PHASE-09`/`PHASE-10` overlap the
ledger already declares at length — but it is not benign in kind. The claim was
restated at three consecutive phase closes by an orchestrator that was otherwise
scrupulous about running the gate itself rather than trusting a worker's tally,
and the plausible mechanical cause is reading the tail of the output where the
high-signal cell is printed at the head. A slice this careful about *a green
that proves nothing* shipped a clean conformance reading that had never been
read. The two paths that are not benign — `flake.nix`, which is what makes row 7
measurable at all, and `LOOP.md` — were invisible inside that same sentence.

**The second-order finding is about durability, and it rhymes.** The registry
`slice conformance` folds lives at `.doctrine/state/slice/248/boundaries.toml`,
which is gitignored. All ten boundary rows exist only there; the tracked record
carries the verdict in prose but not the rows. This audit could run the algebra
at all only because the file was hand-copied out of the clone. An `rm -rf` of
state would have destroyed the slice's entire conformance basis silently, and
conformance fails closed to `unavailable` rather than announcing the loss. The
slice already diagnosed this exact class once, correctly, when it moved the
verification ledger out of `handover.md` because "this is **evidence**". The move
stopped one artefact short.

**The owed ledger is the slice's real deliverable to reconciliation, and it is
honest about its own failures.** 176 items, three self-recorded drifts (items
142, 143, 176), each caught and repaired by a manual orchestrator sweep with no
mechanical backstop. The compact pointer lists in `handover.md` are not safe to
execute: they name `F-31`, whose issue item 70 explicitly withdrew as discharged
in-phase, and they flatten unconditional obligations together with ones the notes
make expressly conditional on the reconciler's judgement. Executed verbatim, that
list would have minted a spurious issue and pre-empted five decisions that are
not the auditor's. The eight entities minted here were taken from the items, not
from the list.

**Standing risks accepted into reconciliation.** The `PHASE-09`/`PHASE-10` delta
overlap is real, unfixable and already declared — a phase blocked on a human
ruling while later phases proceed produces non-contiguous deltas by construction,
and a delta is one contiguous range. Four row-B5 sibling tests remain
load-fragile (`ISS-334`); the slice measured the failure mode, understood it as
the harness being honest under contention, and declined to widen the fixture
window because the cheap fixes all tax every concurrent row. Row 9 ships under a
title it exceeds, deliberately and documented at the test. `REQ-459` criterion 2
is `partial`, not discharged, and says so.

**One decision was genuinely open and was the owner's**, recorded as `F-8` and
raised as a blocker rather than smoothed: `EX-14` in CI. **Ruled 2026-08-10** —
see the closing section. The slice settles the local host
via `DEC-180` and stops there, marking the remainder "(Slice owner /
orchestrator, before close.)". `EX-14` forbids an availability condition, an
`#[ignore]`, a skip or an early return reaching `Admitted`; a runner that cannot
give the backend what the rows need makes the suite red for a reason that is not
a conformance defect, and every available reflex is one `EX-14` forbids. This
audit walked into the concrete instance of that on its first gate run. It is not
a code defect and must not be dispositioned as one.

## Reconciliation Brief

### Per-slice (direct edit)

- **`verification-ledger.md` — the conformance claim (`F-1`).** Replace *"reads
  24 files, nothing undeclared"*, and its restatements at the `PHASE-08`,
  `PHASE-09` and `PHASE-10` close paragraphs, with the measured algebra: 24
  conformant, 0 undelivered, 172 undeclared. Attribute the 170 `.doctrine/` paths
  to the declared `PHASE-09`/`PHASE-10` overlap in the same sentence, so the
  number reads as accounted-for rather than alarming.
- **`verification-ledger.md` — transcribe the boundary rows (`F-3`).** Copy the
  ten `phase → code_start_oid..code_end_oid` rows out of the gitignored
  `boundaries.toml` into the tracked ledger. This is the same repair the ledger
  itself received when it moved out of `handover.md`, applied to the one input it
  still depends on.
- **`design.md` — record `T9`'s vocabulary widening (`F-7`).** `PHASE-09` `T9`
  added `Observed::Unspoken` to a vocabulary the locked design run closed, on the
  owner's ruling. Answer `notes.md` item 138 in the same edit: whether
  `PHASE-10` `T9`'s `Unrowed`/`Reading` channel is the same widening or is
  covered by the design's existing "reported without a verdict". Answering only
  one of the two is how this drifts a fourth time.
- **`design.md` §6 — mirror the selector additions below.** Human mirror only;
  the load-bearing change is the registry.
- **`design.md` — the sentences the notes say are owed.** `sec-2`'s assembly list
  omits three profile-owned binds (item 37); `sec-2`'s "closed environment" and
  `CapsuleEnv`'s doc are short by the backend-synthesised `PWD` entry (item 128);
  `CapsuleStdio::EmptyInputCapturedOutput`'s justification clause points at the
  harmless direction (item 129); `sec-5`'s `900` justification should say it
  bounds a build/verification contract, not any capsule execution (item 16a).

### Selector registry (`slice-248.toml` — the load-bearing conformance fix)

- **`doctrine slice selector add 248 flake.nix --intent design-target` (`F-2`).**
  `flake.nix` carries `util-linux` and `socat` — the host dependencies row 7 and
  row 5 rest on. It is design-target surface, not incidental. Prose alone leaves
  conformance red.
- **`LOOP.md` (`F-4`).** Reconcile decides: a `scope-relevant` selector declaring
  it touched-but-not-delivered, or a recorded exclusion. Either resolves the
  cell; leaving it unadjudicated does not.

### Governance / spec (REV)

- **`SPEC-030` `REQ-459` — criterion 2 is `partial`.** Criterion 1 is discharged
  over the channels `sec-2`'s ledger names; criterion 2 requires `SL-241`
  evidence **and** production acceptance tests, and this slice has the suite and
  no production acceptance tests; criterion 3 is discharged structurally. Record
  as a contributing `--change`, not a closure (item 139).
- **`SPEC-030` — the conformance suite's host dependencies.** `setsid`
  (`util-linux`) and `socat` are now production host requirements of the shipped
  suite, discovered by this audit the hard way. They are declared in `flake.nix`
  for the jail; the spec should name them where the suite's contract lives.

### Net-new work (already minted on `edge`)

`ADR-021` `unsafe_code` `forbid`→`deny` with a two-site budget (item 33) ·
`IMP-416` agent-execution timeout key (item 16b) · `ISS-334` row-B5 sibling tests
load-fragile (`PHASE-09` `F-35`, item 104) · `ISS-335` spike scripts inherit an
unpinned `PATH` (item 170) · `ISS-336` spike artefacts do not record their
environment (item 171) · `ISS-337` setup decoys stay inheritable across the arm
(item 127) · `ISS-338` `gid_map` has no off-jail reading (item 175a) · `ISS-339`
the conformance suite has never run off-jail (item 175b).

`IMP-417` the third admission outcome — the owner's ruling on `F-8` (below) ·
`IMP-418` a mechanical check reconciling a slice's § *Owed* against its phase
sheets' § *Findings* before the sheets are discarded (`F-5`).

Every backlog item carries a `references --role originates_from` edge to
`SL-248`; `ADR-021` cites the slice in prose, because the engine refuses a
`references` edge from an ADR. `ADR-021` is `proposed` and wants the owner's
acceptance. `IMP-417` is sequenced `after` `ISS-339`.

**Deliberately not minted.** `F-31`'s issue — withdrawn by item 70, which records
the defect as fixed in-phase by `PHASE-09` `T1` and states the merge owes the
*record*, not a live `ISS-`.

### Decisions reconcile must take (conditional in the notes, not the auditor's)

- **Item 118 / item 109 — row 9's title.** Correct the design's wording to the
  mechanism, or keep a title the mechanism exceeds. `ISS-` only if deferred.
- **Item 131 / item 158 — the capsule uid.** Keep `CAPSULE_UID` at 1000 and carry
  row 13's uid half as stated-but-non-discriminating, or declare a capsule uid no
  ordinary operator holds (off-jail arm `A3` is the evidence it works). Caveat:
  `gid_map` is unmeasured on every off-jail arm (`ISS-338`).
- **Item 161 — the userns vacuity.** Confirm no shipped assertion reads
  `CredentialsConfined` off userns *identity*; unprivileged `bwrap` creates a
  fresh user namespace whether or not `--unshare-user` is passed. `ISS-` if it
  survives carding.
- **Item 122 (`F-24`) and item 123 (`F-25`).** Whether the phase sheet's wording
  or the harness should change; and whether a genuinely leaking backend mutant is
  wanted, which costs either a production visibility change or a third `unsafe`
  site against a spent budget.
- **Item 113 (`F-15`) and item 117 (`F-19`).** Both discharged; `ISS-` only if
  the reconciler wants the sheet's wording corrected or the residual tracked past
  this slice.
- **Items 76, 85, 96, 87, 132 — production changes wanting a brief line.**
  `Argv::new`; `hold_descriptor_window()` `pub(crate)`; `FIXTURE_TIMEOUT_SECONDS`
  120→30; `socat` as a row 5 host dependency; `CAPSULE_UID`/`CAPSULE_GID`
  `pub(crate)` and whether their durable home is `backend.rs`.
- **`PHASE-06` `F-6` (item 45) and `PHASE-07` `F-3` (item 50).** The capacity
  report's return channel; and `HostDescriptor`'s home — the notes lean toward a
  `descriptor()` method on `HostFacts` over an engine unit reading `/proc`.

### Foreseeable merge conflicts

`Cargo.lock` (regenerate with a build, do not hand-resolve) and
`.doctrine/adr/001/layering.toml`.

### Harvest check — nothing is stranded in the phase sheets

The audit tail swept the ten gitignored phase sheets for durable content not
present in the tracked record. Every finding the sheets themselves mark *owed
upward* — `PHASE-08` `F-28`/`F-31`/`F-35`/`F-36`, `PHASE-09` `F-2`/`F-35`,
`PHASE-10` `F-20`/`F-24`/`F-26`/`F-44`/`F-58` — is cited in `notes.md` or its
shards. Nothing needs sweeping; the slice harvested continuously as it went, and
its § *Owed* ledger is the reason.

That result is not an argument against `IMP-418`. The check took a hand-written
`awk` pass over ten files to reach "clean", and it can only confirm the
*direction* the slice was already careful about. `notes.md` item 176 records the
opposite direction failing — `PHASE-10` `F-64`–`F-67`'s mapping to items 172–175
exists only inside the gitignored sheet.

### `F-8` — the owner's ruling, 2026-08-10

Of the three shapes `notes.md` § *Open* weighed, the owner accepts the second:
**make backend unavailability a distinct non-`Admitted` verdict.** It is the only
one that keeps the answer inside the type `admission()` already returns — the row
vocabulary carries three values while the admission verdict carries two, so the
information exists and is discarded at the fold. It adds no skip, no `#[ignore]`
and no availability condition on any test, so `EX-14` is satisfied by
construction: the new outcome has no path to `Admitted`.

The ruling is criterion-level, and the implementation is net-new work against
`PHASE-10`'s closed surface, not a reconcile edit. Recorded as `IMP-417`,
sequenced `after` `ISS-339`.

**The trap, named in `IMP-417` and load-bearing.** The new outcome must not be
derived from row-level `Indeterminate`. `ISS-334`'s four row-B5 siblings return
`Indeterminate { NoObservation }` under CPU contention — a genuine fixture race,
correctly reported. Folding that into "the host could not establish this"
rebuilds the vacuous pass one level up under a new name. The discrimination
belongs at **precondition** level: a host-capability probe that runs before the
rows and convicts loudly, on the `S4` guard's existing model, feeding the new
outcome from its own result. Row verdicts stay what they are.

**Landed with the ruling.** `just capsule-verify` on `sl-248` (`79a6f9093`) — the
shipped verb rather than the crate's `#[cfg(test)]` modules, which `just test`
already runs via `default-members`. A `### host` header prints first so an absent
binary is a fact in the transcript rather than an inference from a strange
verdict (`ISS-335`, `ISS-336`). Standalone and deliberately not wired into
`gate`, since wiring it in is what `F-8` was about. `justfile` was already a
`design-target` selector, so it adds no undeclared path. In-jail smoke run:
`outcome=admitted`, 19/19 `Proven`, exit 0.

## Reconciliation Outcome

Written by `/reconcile`, 2026-08-10, on `sl-248`. All eight findings were
terminal (`verified`) at entry; remediation is recorded here rather than by
mutating any disposition.

### Direct edits applied

- **`verification-ledger.md` — the conformance claim (`F-1`).** *"reads 24 files,
  nothing undeclared"* replaced with the measured algebra: **24 conformant, 0
  undelivered, 172 undeclared**, with the 170 `.doctrine/` paths attributed to the
  declared `PHASE-09`/`PHASE-10` boundary overlap in the same sentence. The
  mechanical cause is named (reading the tail of the output, where `conformant`
  prints, rather than the head, where `undeclared` does) so the next reader does
  not repeat it. The only other restatements were in `handover.md`, which is
  gitignored and already superseded.
- **`verification-ledger.md` — the boundary rows (`F-3`).** All ten
  `phase → code_start_oid..code_end_oid` rows transcribed out of the gitignored
  `.doctrine/state/slice/248/boundaries.toml` into the tracked ledger, so the
  conformance basis survives an `rm -rf` of state.
- **`design.md` — `T9`'s vocabulary widening (`F-7`), both halves in one edit.**
  `PHASE-09` `T9`'s `Observed::Unspoken` recorded against the design's
  three-variant block. And `notes.md` item 138 **answered rather than deferred
  again**: the design's `AdmissionVerdict` is written as a complete struct with
  two reporting channels while the shipped type carries three, so the vocabulary
  *is* closed over channels, `PHASE-10` `T9`'s `observations` channel is the same
  class of widening, and it owes the same record. Direct edit, not a REV —
  `design.md` is a per-slice artefact, which is also why the `RV-` named in
  `handover.md` was a category error.
- **`design.md` — the sentences the notes say are owed.** The assembly list is an
  order, not an inventory, and three profile-owned binds it does not name are
  emitted anyway (item 37). `CapsuleEnv` is short by the backend-synthesised
  `PWD`, corrected in `design.md`, the property table's row 11, and the shipped
  doc comment (item 128). `EmptyInputCapturedOutput`'s justification pointed at
  the harmless direction — bytes reaching the trusted side is the *specified*
  behaviour of a capture endpoint; the hazard is the capsule reading descriptor 1
  — corrected in both design and code (item 129). `sec-5`'s `900` now says it
  bounds a build/verification contract, not any capsule execution (item 16a).

Doc comments only in `backend.rs`; no behaviour changed. `cargo fmt --check` and
`cargo clippy -p doctrine-control --bins` clean, `doctrine validate` clean.

### Selector registry (the load-bearing conformance fix)

- **`flake.nix` → `design-target` (`F-2`).** It carries `util-linux`/`setsid` for
  row 7 and `socat` for row 5. Conformance moved **24/0/172 → 25/0/171**.
- **`LOOP.md` → `scope-relevant`, as adjudication (`F-4`).** *The brief's premise
  was wrong and the ruling reflects the corrected one.* The brief offered a
  scope-relevant selector or a recorded exclusion, "either resolves the cell".
  Neither does: `conformance_outcome` reads **only** `design-target` selectors
  (`src/slice.rs:2899`) and the registry has no exclusion mechanism. Nor can the
  cell ever be empty — 170 `.doctrine/` paths remain by the declared overlap. So
  the achievable goal is adjudication, and the selector's note records that
  `LOOP.md` stays in the undeclared cell **by ruling, not oversight**.
  `design-target` was rejected: it would clear the row by calling orchestrator
  scaffolding design surface.

### REVs completed

- **`REV-051` (`reconcile-sl-248`) — `done`.** Two rows.
  - **`REQ-459` `pending` → `in-progress`** (auto-landed, `REC-113`). Criterion 1
    discharged *over the channels `sec-2`'s ledger names* — the qualifier is
    load-bearing, since *in full* was claimed four times in this slice and was
    false the first three. Criterion 3 discharged structurally. Criterion 2
    partial, so `active` would assert a closure it does not support while
    `pending` no longer describes it (item 139's "contributing `--change`").
  - **`SPEC-030` modify** (surfaced, landed by hand). Names `setsid`
    (`util-linux`) and `socat` as production host requirements of the shipped
    suite, and states criterion 2's two shortfalls.

  **Criterion 2 carries a SECOND shortfall, which this brief did not.** It is
  recorded because a backlog item does not discharge a reconcile ruling. The
  conformance fixture binds whole host top-level roots including `/nix`;
  `--ro-bind` is read-only, **not `noexec`**, and measured off-jail a capsule
  sees 691 store paths and runs `git`, `curl` and `gcc`. That is the host-shaped
  default `PHASE-10` `EX-8` exists to refuse, in the very fixture that hosts the
  capsules where `BoundedInputSet` is demonstrated. The nineteen rows are not
  falsified — they are evidence about the properties, not about the environment
  production constrains. Carried as `ISS-341`; slice `SL-252` minted.

### Decisions taken (the brief's conditional set)

- **Row 9's title** (items 118/109) — design wording corrected to the mechanism;
  the test keeps its name, because `PHASE-10` `VT-2` makes it a verification
  floor. Rename deferred to `IMP-422`, which must carry the `VT-2` mandate with
  it. Owner: *"longer term we should rename it not to lie."*
- **The capsule uid** (items 131/158) — `IMP-419`, sequenced `after` `ISS-338`
  (`gid_map` unmeasured off-jail, so the baseline is half-verified). Not taken
  here: net-new work against `PHASE-10`'s closed surface, the disposition `F-8`
  took to `IMP-417`.
- **The capacity report's channel** (item 45) — stderr named explicitly in
  `design.md`, including that a non-terminal caller gets no programmatic access
  to a `Low` verdict. Better shape deferred to `IMP-420`.
- **`HostDescriptor`'s home** (item 50) — `IMP-421`, leaning to a `descriptor()`
  method on `HostFacts`. Coupling argument, not correctness.
- **Items 76, 85, 96, 87, 132** — recorded; 132's relocation is `CHR-061`.

### Settled by measurement rather than ruled

- **Item 161's userns vacuity does not survive carding.** `CredentialsConfined`
  is **retired** — it survives only in two doc comments describing its
  replacement by `MappedIdentity`, which is *measured to fire* — and nothing in
  `conformance.rs` reads `/proc/self/ns/user`. No `ISS-` owed.
- **Item 117's residual** — shipped row 10 opens a fresh decoy set *per arm* (the
  `F-26` repair) and `hold_descriptor_window()` is `pub(crate)`. The in-process
  window is carried knowingly and documented; same neighbourhood as `ISS-334`.
- **Item 113** — discharged; the sheet's wording is moot, the sheet being
  gitignored and discarded.
- **Item 122 (`F-24`)** — no stub can run through `run_row` at all: `provision`
  reads a transaction identity back out of stdout and fails closed first. The
  harness is right.
- **Item 123 (`F-25`)** — no leaking-backend mutant. It costs a production
  visibility change or a third `unsafe` site against a spent budget; the
  uncovered surface is the parent-side sweep's own mechanism, recorded not fixed.

### Withdrawn / tolerated

None. All eight findings are `verified`; `F-5` and `F-6` needed no per-slice
write (their remediation is `IMP-418` and the mint set the audit already
executed), and `F-8`'s ruling is `IMP-417`, criterion-level and net-new.

### Handoff

`/close`. Outstanding before the terminal transition: the cross-branch id
collision sweep, the post-merge `SL-252 references ISS-341` edge, and `LOOP.md`
as an unlisted merge conflict — see `handover.md`.
