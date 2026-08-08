# Implementation Plan SL-248: Capsule provisioning and Linux backend

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Ten phases over a locked design (`design.md`, design run `dr-019fd432`, revision
85). Each lands as tested machinery beside the incumbent worktree arms, unused.
There is no cutover in this slice, so no phase needs a big-bang landing — with
one qualification stated under *What each phase changes about the tree*.

### Authoring stages

This plan is authored in three passes, because the design does not fit one
comfortable session and the failure mode of splitting it is phase 7 written in a
context that has forgotten what phase 3 promised.

1. **The spine** — the phase list, objectives, file ownership, dependency order,
   and design-section provenance. Complete as of this document. `PHASE-NN` ids
   are immutable and edits append rather than renumber, so the list is the one
   artefact that had to be right first time.
2. **Criteria expansion** — `EN`/`EX`/`VT` per phase, in one or more sessions
   over disjoint phase sets. Each session's inputs are *this spine plus its own
   phases' design sections*, never a previous session's prose. The provenance
   table below is what makes that possible: a stage-2 session reads its own
   sections rather than all ~5100 lines.
3. **Integration** — the seams no stage-2 session can see. Every `EN` discharged
   by an earlier `EX` or by the slice's entry state; no `VT` duplicated across
   phases; the union covering `sec-8`'s evidence table; every design section
   claimed by the provenance table or explicitly out of scope. Complete; the
   four checks are recorded below.

#### What the four checks found

All four were run over the whole plan, not sampled. Two produced a change to the
plan; two produced a ruling and no change.

1. **`EN` discharge — clean.** All 31 entrance criteria discharge. `PHASE-01`'s
   four are entry-state facts about the tree as the slice found it, each
   re-checked at the `/plan` gate (§ *The design's premises were re-checked*);
   the other 27 each name an earlier phase's `EX`. One was tightened rather than
   fixed: `PHASE-09` `EN-2` rested on `PHASE-07`'s `VT` ids alone, and now names
   the `EX` ids that land the algebra alongside the `VT` ids that evidence it.
2. **`VT` duplication — one permitted overlap, two collisions ruled.** The
   overlap is `PHASE-01` `VT-1` and `PHASE-02` `VT-10` keywording two
   export-set titles; permitted, because it is one test re-run at a second
   export-set size and `PHASE-02`'s mandate carries `interpretation` as a third
   keyword, so neither mandate is inert. The two title collisions the design
   carries — one spelling (`both_declared_` versus `both_readable_lists_empty_
   refuses`) and one name (`unknown_key_refuses_naming_the_key` in two files) —
   are ruled at the phases that own them and recorded under § *Corrections
   owed*. Every other same-file cross-phase mandate pair is keyword-disjoint.
3. **`sec-8`'s evidence table — one gap, closed.** Every file the table names as
   a home for the new evidence carries at least one `VT` mandate anchored in it,
   with one exception found here: `transaction.rs`. Its only title —
   `a_transaction_id_carrying_a_path_separator_refuses_at_construction` — sat in
   `PHASE-06` `VT-3`, whose `test_file` is `provision.rs`, so the mandate would
   have looked for it in the wrong file and `transaction.rs` would have carried
   no keyword floor at all. Split out as `PHASE-06` `VT-6`. Beyond the table,
   every title in all seven of the design's `Verification alignment` sections
   was matched to a claiming mandate; none is orphaned.
4. **Design-section claiming — four groups of prose named out of scope.** Every
   section that states implementation is claimed by the provenance table. The
   table's out-of-scope sentence was widened, because it covered `sec-1` and
   `sec-9`'s risks and residuals and left four groups of narrative unmentioned
   rather than out of scope. `sec-7`'s executed `Verification alignment` blocks
   were claimed only by implication and are now named on the rows that own them.

### How the phases were sized

`sec-8` § *Where the new evidence lives* is the sizing authority and had already
done most of this work: it cuts the evidence by surface and module, and states
that the counts are indicative for phase sizing while each section's own list
remains the authority. This plan cuts on that seam rather than across it.

The slice scope's band — *~1000 design lines over 6–9 phases* — was reasoned
before the design was written, and the design came in at ~5100 lines. Taken
literally the band would give an absurd phase count, because most of those lines
are review archaeology (why `F-25` happened, what `in full` cost three times)
rather than implementation instruction. The metric that survives is the one
`sec-8` offers explicitly: roughly 160 pure tests plus 23 executed rows. Ten
phases puts that near 18 tests per phase, against the two failure modes the
scope names — `SL-233`'s 16 phases that left holes, and `SL-244`'s 8 phases that
were uncomfortably coarse.

Where "vertical slice" applies here is the **phase**, not the planning session:
each phase leaves the tree green and self-contained. The planning *sessions* cut
on the design's existing surface seam, because that is what makes them disjoint.

## Sequencing & Rationale

### PHASE-01 must be first, and the checked set must land inside it

`sec-8` records the trap and it is the most expensive thing in this plan to get
wrong: the new crate's tests live in a workspace member that `just check` would
not build by default — `lint`, `build` and `test` resolve to a bare `cargo`
invocation that, with a package at the workspace root and no `default-members`,
selects the root package alone. **A suite that is never built is green by never
running.** If the crate lands before the checked set changes, every subsequent
phase ships vacuously-green tests and no gate notices.

So `default-members = [".", "crates/doctrine-control"]` is not a tidy-up at the
end. It belongs in the first phase that creates the crate, which is this one.

Three further edges bite at the same moment, and all three are invisible to a
reading of the production graph:

- **`cargo test` starts building the library's own test targets.** The export
  set must be transitively closed over `crate::` paths in `#[cfg(test)]` code as
  well as production code — `git.rs`'s test module reaches `crate::kinds`
  (`src/git.rs:2896`, confirmed against the current tree). `cargo build` is green
  while this is wrong; it surfaces only when the library's tests first compile.
- **`pub use` of a `pub(crate)` item is `E0364`**, so `read_path_at`,
  `CaptureError`, `DOCTRINE_TOML` and `read_doctrine_toml_text` become `pub`.
- **`default-members` is a package-selection default, not a build-command one**,
  so it also selects the new crate for `cargo package` and `cargo publish`. Both
  halves of `sec-6` § *Nothing ships* are therefore in this phase — `publish =
  false` in the new manifest **and** `-p doctrine` on the recipe — because each
  alone fails, in opposite directions.

### Why PHASE-02 cannot come before PHASE-01

`src/interpretation.rs` is pure root-package work with no dependency on the new
crate, which makes it look like a free first phase. It is not. `sec-8` is
explicit that `src/main.rs` does **not** declare `mod interpretation;` — nothing
in the agent-facing binary consumes the policy. So until `src/lib.rs` exists to
declare it, the module is compiled by no target at all, and its ≈42 tests never
run. That is the PHASE-01 trap in a second guise: a module declared by no target
is as green-by-never-running as a crate outside the checked set.

### Why the backend splits across PHASE-04 and PHASE-05

`sec-8`'s evidence table groups `sec-2`'s ≈39 tests as one row across two files.
This plan subdivides that row along the file boundary — the contract types and
`CapsulePlacement::try_new` in `backend.rs`, the bubblewrap profile in
`backend/bubblewrap.rs`. That is a finer cut on the same axis, not a cut across
it: the test groupings `sec-8` costed are preserved, and each phase owns exactly
one file.

### Why the conformance suite is four phases, not one

`sec-7` is a third of the design and carries the only executed evidence in the
slice. It divides cleanly by what can be wrong independently:

- **PHASE-07** is the part that can be wrong with no capsule running at all —
  classification and the verdict algebra. `sec-7` names this precisely: it is
  "where a suite silently degrades into one that always passes."
- **PHASE-08** is the executing machinery — fixture, arm shapes, the weakening
  seams, containment — proven on table C, whose four claims contribute to no
  admission and so cannot mask a broken harness with a green verdict.
- **PHASE-09** and **PHASE-10** are the admission rows themselves.

`execute_weakened`'s ten removals and the one grant all land in PHASE-08 rather
than arriving per row, because the enum is complete at PHASE-07 and a
non-exhaustive match does not compile. PHASE-08 makes the backend weakenable in
ten named ways; PHASE-09 and PHASE-10 measure that each weakening falsifies
exactly its own row.

### Why the admission rows split at 8/14

Rows 1–8 are the clause-derived properties. Rows 9–14 are the six the channel
ledger surfaced under review, and they differ from the first eight in a way that
matters for phase sizing: each carries its own named stub mutant, and the
credential pair carries measurements that must be re-run **on the executing
phase's own host, outside this project's jail** — the jail already strips
capabilities and sets `no_new_privs`, so a jailed probe cannot distinguish a
backend that confines credentials from a cage that had already done it.

`Admission::Admitted` is reachable at each of the two phases and means something
true at each: every row in tables A and B is `Proven`. The `Property` enum grows
from eight variants to fourteen across them, which the compiler keeps honest —
`RowId::Property` keys the verdict, so a row the suite can construct that the
enum cannot name is a compile error. That is the one machine-checked projection
of table A (`sec-9` `R9`); the rest are reader discipline.

### Dependency order

```
PHASE-01 ─┬─ PHASE-02 ─┐
          └─ PHASE-03 ─┴─ PHASE-04 ── PHASE-05 ── PHASE-06 ── PHASE-07 ── PHASE-08 ── PHASE-09 ── PHASE-10
```

`PHASE-02` and `PHASE-03` are the only pair with no ordering constraint between
them. They are **not** worth dispatching in parallel: both append to
`.doctrine/adr/001/layering.toml`, which would conflict, and neither is long
enough to pay for the coordination. Serial throughout is the default.

`PHASE-04` depends on `PHASE-03` for `ByteCount`, which `sec-5` defines because
it owns the arithmetic that can overflow, and which `sec-2`'s `file_size_cap`
and `disk_used` are. That is the `backend → config` edge in `sec-6`'s unit table.

### File ownership

Exclusively owned — one phase, no other phase touches the file. The root
package's `src/main.rs` is on this list and is easy to miss, because it takes
exactly one line: `PHASE-01`'s `mod config_file;`. `PHASE-02` `EX-9`
deliberately declines to declare `mod interpretation;` there, so no later phase
touches it.

| phase | files |
|---|---|
| PHASE-01 | `src/config_file.rs`, `src/dtoml.rs`, `src/git.rs`, `src/main.rs` (root package), `Cargo.lock`, `justfile` |
| PHASE-02 | `src/interpretation.rs` |
| PHASE-03 | `crates/doctrine-control/src/host.rs`, `…/config.rs`, `…/capacity.rs`, `.doctrine/doctrine.toml` |
| PHASE-04 | `crates/doctrine-control/src/backend.rs` |
| PHASE-05 | `crates/doctrine-control/src/backend/bubblewrap.rs` |
| PHASE-06 | `crates/doctrine-control/src/transaction.rs`, `…/provision.rs` |

Shared, append-only — several phases add to these and none rewrites another's
entries. They are named here so a stage-2 session does not mistake a shared file
for a boundary violation, and so no two of these phases are ever dispatched
concurrently:

| file | phases | what each adds |
|---|---|---|
| `src/lib.rs` | 01, 02 | 01 creates the lib target and its five-item export set; 02 appends `pub mod interpretation;` and the sixth entry (`EX-8`) |
| `tests/architecture_layering.rs` | 01, 02 | 01 parameterises `check`'s module filter and adds the export-set and two-tree assertions; 02 re-runs two of them with `interpretation` added (`VT-10`) |
| `.doctrine/adr/001/layering.toml` | 01, 02, 03, 04, 05, 06, 07 | one row per unit introduced, in the tree's own section. Self-enforcing: an unclassified unit fails the gate |
| `crates/doctrine-control/Cargo.toml` | 01, 03 | 01 declares the `doctrine` path edge; 03 adds `rustix` (`fs` **and** `std`), `serde` (`derive`) and `toml`. Derive the list from the crate's own `use` statements, never from `sec-6`'s table |
| `crates/doctrine-control/src/main.rs` | 01, 06, 10 | 01 the skeleton, 06 the `provision` verb, 10 the `backend verify` verb. Each verb lands with the code it calls |
| `crates/doctrine-control/src/conformance.rs` | 07, 08, 09, 10 | the design places the whole suite in one unit. Sequential ownership only |
| `Cargo.toml` (workspace) | 01 | `members`, `default-members`, and `include` gaining `!/src/lib.rs` — all three in one phase, per `sec-8` |

### Design-section provenance

What each phase is expanded from. A stage-2 session reads these sections and the
spine, and nothing else. Anything a phase does not claim here is either another
phase's or out of scope.

| phase | design sections |
|---|---|
| PHASE-01 | `sec-6` entire; `sec-8` § *The touch-set*, § *The checked set*, § *What must stay green unchanged*; `sec-9` `R6`, `R7` |
| PHASE-02 | `sec-4` entire except § *The forbidden list has a consumer in this slice*; `sec-6` § *What the root package must export* (the `interpretation` entry) |
| PHASE-03 | `sec-5` entire except its two table C rows; `sec-6` § *The new crate* (the dependency edge table) |
| PHASE-04 | `sec-2` § *Target behaviour: the contract*, § *What a backend is told*, § *What a backend reports*, § *The properties, stated once*, § *The rows are derived from channels*; invariants 1–5, 9, 13, 15; § *Verification alignment* — the placement-validation, descendant-mutant, source-export and vocabulary groups |
| PHASE-05 | `sec-2` § *The bubblewrap backend* entire; invariants 6–8, 10–12, 14; § *Verification alignment* — the argv, closure, inner-`PATH` and descriptor-sweep group |
| PHASE-06 | `sec-3` entire, less the freshness rows its § *Verification alignment* hands to the harness; `sec-4` § *The forbidden list has a consumer in this slice*; `sec-6` § *The two verbs* (`provision`) |
| PHASE-07 | `sec-7` § *Where it lives*, § *The removal is named by the property*, § *A control may grant*, § *Two of the four credential mechanisms cannot be rowed*, § *The arm is the unit of execution*, § *Both arms provision*, § *Liveness first, observation second*, § *The row verdict*, § *The verdict*; invariants 1–6, 9, 10; § *Verification alignment* — the two pure groups |
| PHASE-08 | `sec-7` § *The fixture*, § *Hazard containment, per row*, § *Table C*; invariants 7, 8; § *Verification alignment* — the *executed, table C* block, and the three harness titles in the *claims that need naming* block; `sec-5` § *Verification alignment* (the two table C capacity rows); the table C claims `sec-3` and `sec-4` owe |
| PHASE-09 | `sec-7` § *Table A* rows 1–8 and their subsections — § *Rows 3 and 4 share a mechanism*, § *Row 7's control is `--die-with-parent`*, § *Row 6 observes a value*, § *Row 8 is five payloads*, § *The payload must be the property's strongest negation*; § *Table B* entire; § *Verification alignment* — the first eight *executed, table A* titles, the *executed, table B* block, and the rows 2/6/7/8 and B5 entries in the *claims that need naming* block |
| PHASE-10 | `sec-7` § *Table A* rows 9–14 and their subsections — § *Clause 2 is five rows*, § *Rows 13 and 14 answer no clause*, § *Row 10's observer is its own first false positive*, § *Row 12 observes readability, not delivery*; § *Table A is the inventory*; § *Verification alignment* — the last six *executed, table A* titles and the rows 9–14 entries in the *claims that need naming* block; `sec-6` § *The two verbs* (`backend verify`); `sec-9` `R8`, residual 3 |

**Out of scope, explicitly.** Stage 3 walked every heading in the design against
this table. What follows is the whole of what no phase claims, and each is
narrative that states no implementation — a phase expanding it would be
expanding an argument, not an instruction.

- **`sec-1` entire** — governing context. It cites records and judges
  applicability.
- **`sec-2` § *Current behaviour, and why none of it is reused*** and **§ *The
  arithmetic, and what correspondence survives***. The first is the case for not
  extending `bwrap_core_argv`, discharged by `DEC-155` before this plan starts;
  the second is the rows-to-clauses accounting and its instruction is negative —
  claim no one-to-one correspondence — which `PHASE-04` `EX-15` and `PHASE-10`
  `EX-15` already carry as *add no second unchecked projection*.
- **`sec-8` § *What does not change*, § *Where the new evidence lives*, and
  § *Requirement closure***. The first names what is unedited and retires risk
  `R4`; the second is this plan's own sizing authority (§ *How the phases were
  sized*) rather than a phase's input; the third is the reconciliation brief's
  material, and each phase carries its own closure claim as an `EX` — `PHASE-03`
  `EX-20` and `PHASE-08` `EX-16` for `REQ-461`, `PHASE-06` `EX-19` for
  `REQ-450`, `PHASE-10` `EX-16` for `REQ-459`.
- **`sec-9` § *Assumptions*, § *Open questions*, § *Corrections owed to the
  reconciliation brief*, § *Follow-Up at close***, and every risk and residual
  not distributed above. The risks and residuals that carry a phase obligation
  are on the rows that owe them (`R6`, `R7`, `R8`, `R9`, residual 3); the rest
  are recorded rather than built. The four sections named here are addressed to
  the audit and the close, not to an executing phase — which is why `ISS-323`
  and the two title divergences under § *Corrections owed* below are owed
  **again** at reconcile: `sec-9`'s own corrections list cannot be extended in
  place without a design recovery cycle (`ISS-320`).

### Phase obligations the design defers to execution

These are named in the design as work a phase owes, not as work the design did.
They are collected here because they are easy to lose between sections and each
one is a candidate `EX` or `VT` in stage 2.

| phase | obligation | design |
|---|---|---|
| 01, 03 | derive the new crate's dependency list from its own `use` statements and treat any disagreement with the design's table as the table being stale | `sec-6`, `RV-346` `F-29` |
| 05 | execute whether bubblewrap creates the mountpoint inside the tmpfs for a `readable-roots` entry resolving beneath `/tmp`, rather than refusing the bind | `sec-2` |
| 09 | re-run `EVD-013`'s pid-namespace × `--die-with-parent` 2×2 against the session-escaping payload. If `--die-with-parent` turns out not to be required for it, row 7's control is wrong and the argument, not just the number, has to be redone | `sec-7` |
| 09 | execute **both** legs of row 5 — TCP loopback and the abstract unix socket. If the abstract leg is denied by a different mechanism than the TCP leg, it is a further row rather than a second assertion in this one | `sec-7` |
| 09, 10 | measure every unmeasured delta in the removal table. Three of eleven are measured; the rest are reasoned. A delta that does not produce its row's control failure means **the row is wrong**, not that the measurement is inconvenient | `sec-7`, `R1` |
| 10 | re-run the credential measurements on the executing host, outside this project's jail, rather than inheriting `EVD-014`'s | `sec-7` |
| 10 | measure the executed suite's wall-clock cost on the fast inner loop. The one lawful adjustment is `cargo test -- --skip <module>` on `test:` alone, with `test-all` unfiltered — never `#[ignore]` on the admission test | `sec-8`, `sec-9` residual 4 |

### What each phase changes about the tree

Every phase ends green and adds machinery nothing yet uses, with two landings
worth knowing about in advance:

- **PHASE-01** brings `crates/doctrine-control` into `lint`, `build` and `test`.
  From that point the fast inner loop compiles and tests the new crate. That is
  the intent, not a side effect.
- **PHASE-08 is the flag day, and PHASE-10 completes it.** Table C provisions
  real capsules and executes them, so **from PHASE-08 onward** `cargo test`
  fails on any host that cannot run the backend, and `sec-8`'s `default-members`
  ruling brings that failure forward into `just check`. Every admission row
  after it compounds the same cost. PHASE-10 is where it becomes
  unconditional-by-construction rather than incidental: its admission test
  asserts `Admitted` *unconditionally*, because conditioning it on backend
  availability would reintroduce exactly the green skip `DEC-156` forbids. The
  distinction is worth keeping straight — an earlier reading of this section
  attributed the whole cost to PHASE-10, which would have let a PHASE-08 red on
  a bubblewrap-less host look like a defect rather than the planned landing.
  `DEC-180` § *Where it lands* carries the durable version. `sec-9` residual 3 is
  explicit that the affected set is not "macOS" but *any host, or any nesting,
  that denies what the backend needs* — a seccomp-filtered CI runner as much as
  an agent sandbox. This is the closest thing to a flag day in the slice, and it
  is a considered cost rather than an oversight.

  **Accepted, explicitly, at plan time.** The slice owner's ruling: a state in
  which doctrine does not build green on a machine without bubblewrap is
  acceptable, because in practice nobody else runs this project's tests, and the
  gate that matters is the end of the slice rather than the end of each phase —
  phases are built inline on `edge`. So no mitigation is planned, no phase owes
  one, and `sec-9` residual 3's open question is narrowed rather than closed:
  what stays open is the **CI** ruling the design says is owed by whichever
  slice first runs this suite in CI, not the local-host behaviour, which is now
  decided. Recorded here so a later reader does not read the residual as an
  unmade decision and re-open it.

## Notes

### If a phase proves oversized

`PHASE-NN` ids are immutable and edits append, so a phase that turns out too
large cannot be split by renumbering. Two split points are pre-identified
because appending at them keeps the list in logical order rather than leaving a
new phase permanently out of sequence. Neither was needed in stage 2, so both
stand available to execution:

- **PHASE-02** divides at `parse` + `canonical_hash` versus `restrict`. They are
  separate pure functions with separate refusal vocabularies, and `sec-3` step 4
  consumes the first while step 5 consumes the second.
- **PHASE-10** divides at the descriptor and stream rows (10, 12) versus the
  credential rows (13, 14), which are disjoint by field and carry the
  out-of-jail measurement between them.

Neither split is taken. They are recorded so that a phase needing one appends
rather than improvising.

### Corrections owed

All transcription-grade, all resolving unambiguously against the design's own
prose, and none reopening a decision. Items 1 and 2 are owed at execution; items
3 to 5 were ruled at stage 3 and are owed only to the reconciliation brief,
because they are corrections to design text and `sec-9`'s own corrections list
cannot be extended in place without a design recovery cycle (`ISS-320`). They
are recorded here so the audit does not have to rediscover them.

1. **`sec-6`'s `EXPORTED` constant omits `today`.** The sketch above it declares
   `pub use clock::today;`, the same section's prose calls `clock::today` "the
   third export" and explains that taking it is what avoids a fourth dependency
   edge, and `sec-8`'s touch-set counts "five items behind three private
   modules" — which is only true if `today` is one of them. The constant lists
   four items plus the `interpretation` module. Implementation follows the
   prose: `EXPORTED` carries six entries, and the export-set assertion is
   written against that. Owned by PHASE-01.
2. **`sec-8` enumerates `just check` as five legs.** It runs six —
   `fmt lint lint-js build validate test`. `lint-js` is a bun/eslint leg over
   `web/map` with no cargo package selection in it, so the checked-set ruling is
   unaffected and the omission changes nothing. Recorded so a stage-2 session
   reading `sec-8` against the recipe does not treat the difference as drift.
3. **One refusal is spelled two ways.** `sec-2` § *Verification alignment* lists
   `both_declared_lists_empty_refuses`; `sec-5` § lists
   `both_readable_lists_empty_refuses`. One refusal (`NoReadableInputs`), one
   test, one file (`config.rs`). **`sec-5`'s spelling stands** — it is the
   section that owns the reader, the refusal type and the file, and it is what
   `PHASE-03` `VT-4` already carries, so no mandate moves. `sec-2`'s entry is a
   cross-reference to `sec-5`'s test rather than a second obligation, as is its
   identically-spelled
   `closure_roots_without_a_resolver_refuses_naming_both_keys`. Neither is a
   duplicated `VT`.
4. **One title names two tests.** `unknown_key_refuses_naming_the_key` appears
   in `sec-4` § and `sec-5` § *Verification alignment*, for the
   `[interpretation]` walk and the `[capsule]` `deny_unknown_fields` refusal
   respectively. **Deliberately not qualified.** The two `#[cfg(test)]` modules
   are in different files, so this is Rust-legal, and each `VT` mandate is
   file-scoped — `PHASE-02` `VT-1` reads `src/interpretation.rs`, `PHASE-03`
   `VT-4` reads `config.rs`, and neither can be satisfied by the other's file.
   Renaming one plan-side would put the plan out of step with the locked
   design's title authority to buy a grep convenience, which is the wrong trade;
   the cost accepted instead is that a reader greping the bare title finds two
   tests and disambiguates by module.
5. **`sec-9` residual 2's count is stale.** It states `PropertyRemoval` at nine
   variants; the vocabulary this plan lands is ten (`PHASE-07` `EX-4`), with
   `AuthorityGrant::AllCapabilities` a separate enum on top. The residual's
   claim is about the *trend* — the vocabulary grows every round — and that is
   unaffected and still true. Nothing rests on the numeral: `PropertyRemoval` is
   a closed enum the compiler checks, and `PHASE-07` `EX-4` is the authority.
   Recorded so a reader does not take nine as current.

### The design's premises were re-checked against the tree

`/plan` requires the design's concrete references to be resolved against the
current tree rather than trusted from author-time. Every grep-able claim
checked out at the cited location: the absent `src/lib.rs`, the 94 `mod`
declarations in `src/main.rs`, the workspace members, `include`'s allow-list,
the `rustix` edge and its `Cargo.toml:77` zero-new-crates note,
`git::read_path_at` and `CaptureError`'s `pub(crate)` visibility,
`fetch_refspec`, `git.rs`'s test-module reach into `crate::kinds`, `dtoml`'s two
items, `clock::today`, `bwrap_core_argv` and its byte-parity test,
`have_bwrap`, the layering gate's `discover_units` / `extract_edges` /
`CratePathCollector` / hardcoded `Path::new("src")`, `reserve.rs`'s tolerant-doc
shape, and the `publish` and `pkg-check` recipes. The tolerant top-level parse
that lets `[capsule]` be added without disturbing the 35 incumbent call sites
was confirmed at `DoctrineToml`. No premise was found stale, so no design
back-edge was required.
