# Notes SL-248: Capsule provisioning and Linux backend

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage
<!-- `explore.triage`, design run dr-019fd432. Recorded once at the exploring gate;
     superseded by design.md once the run materialises. -->

### Constraining governance (all read this session)

`ADR-020` (target architecture; items 1–2 of its decision are this slice)
· `SPEC-030` § Transaction authority / Contract and interpretation provenance /
Platform backend contract · `REQ-449`, `REQ-450` (criterion 1 only), `REQ-459`,
`REQ-461`, plus `REQ-448`'s denial half · `DEC-153` (two binaries, split at
canonical mutation) · `DEC-136` (one reader, resolve once from base) · `DEC-134`
(headless worker) · `DEC-133`/`DEC-137` (a harvested capsule is live work —
capacity handling may never delete one) · `ADR-001` (a new top-level module needs a
`layering.toml` classification) · `POL-001` (naming; see Learned) · `POL-002` (bwrap
and any free-space probe are declared host capabilities) · `STD-001` (every
`[interpretation]` key, the schema version, the capacity default — named constants).

### Shaping decisions already taken

- `DEC-153` fixes the topology: new transaction verbs go to a `doctrine-control`
  workspace member over a lib target on the root package. Nothing migrates out of
  the agent-facing binary.
- `DEC-136` fixes provenance: resolve once, from the contracted base, bound into the
  work contract; never re-resolved from a capsule checkout.
- Research fixes three implementation postures: the `[interpretation]` projection
  follows `reserve.rs`'s out-of-band pattern rather than joining `DoctrineToml`; the
  refusal machinery copies `JailPolicy`'s fail-closed shape; the Linux backend reuses
  the existing bwrap vocabulary rather than adding a second builder.

### Open questions carried into the inquiry

1. Does this slice create the `doctrine-control` workspace member and the root lib
   target, or land provisioning inside the existing binary and split later?
2. What exactly is shared with `dtoml` when the source is a git object rather than a
   file on disk — `read_doctrine_toml_text` is `root`-relative, so the shared part
   can only be the pure parse.
3. How the capsule confinement profile relates to `bwrap_core_argv`, which carries a
   byte-parity contract with `scripts/pi-spawn-confined.sh` (VT-7) and therefore
   cannot simply be widened.
4. The form of `REQ-459`'s property suite — what makes it a backend-admission gate
   rather than a Linux test, and how denial is asserted positively.
5. The provisioning mechanism and capsule storage layout, and what proves `REQ-450`
   criterion 1.
6. `REQ-461`'s free-space probe: mechanism, config placement, `POL-002` declaration.
7. How much of the work contract exists here, given `REQ-449` criterion 4 needs the
   phase-contract restriction algebra but the rest of the transaction is out of scope.
8. What `doctrine-control` exposes as a CLI in this slice, and how provisioning is
   verified end-to-end with no launch, harvest, or admission to follow it.

### Risks

Scope `R1` (evidence altitude), `R2` (incumbent suites green unchanged — this slice
touches `jail.rs`), `R3` (a weakly written property suite certifies nothing) stand as
authored. Added here:

- **R4 — the parity contract is a second behaviour-preservation obligation.**
  `bwrap_core_argv` is asserted byte-equivalent to the pi-spawn script
  (`jail.rs:1215`). Any change to it fails a test whose purpose is to notice.
- **R5 — `doctrine-control` inherits every embed root under the cheap path**
  (`DEC-153` § The cheap path). The nix `srcWithDist` graft and the binstall asset
  contract move together or the binary ships hollow with no compile error.

### Assumptions

Scope `A1`–`A3` stand. Added: **A4** — `git::read_path_at` reads a blob at an
explicit OID with no working tree, and is the whole impure surface `REQ-449`'s
resolution needs. To be verified at point of use, not assumed from the code map.

## Execution posture — two phases in-tree, then a clone

Decided 2026-08-08. **Not** a dispatch run and not a worktree: `PHASE-01` and
`PHASE-02` land in the primary tree on `edge`; from `PHASE-03` the slice moves to
a **separate git clone** on an `sl-248` branch, merged back manually at the end.

**Waiting on `SL-250` `PHASE-06` to finish before starting.** Another agent is
live in this repo (`SL-249`/`SL-250`, `.doctrine/rfc/027/`, `SPEC-031` and
`REQ-462`–`REQ-475`).

### Why a clone rather than a worktree

No orchestrator, so the dispatch marker and confinement machinery is pure
overhead. A clone has its own `.git`, keeps the history linear for the close-out
audit, and cannot collide with the primary tree. Every phase from `PHASE-03`
builds into its own in-tree `target/` regardless (`ADR-008` D-B1) — that cost is
paid once either way.

### Why the split falls after `PHASE-02` and not after `PHASE-01`

Measured against `plan.toml`, not assumed:

- `PHASE-01` is the slice's only phase touching shared machinery — `Cargo.toml`,
  `Cargo.lock`, `justfile`, `layering.toml`'s **root** `[tiers]` section,
  `tests/architecture_layering.rs`, `src/dtoml.rs`, `src/git.rs`, `src/clock.rs`,
  `src/main.rs`.
- `PHASE-02` is the last phase touching the root package at all
  (`src/interpretation.rs`, plus `interpretation = "leaf"` in the root `[tiers]`
  section).
- **`src/lib.rs` and `tests/architecture_layering.rs` are shared append-only
  between `PHASE-01` and `PHASE-02`** (plan stage-3 record item 1). Splitting
  those two phases across two trees would have both trees appending to exactly
  the two files the plan already flagged as contended — a manufactured conflict.
- Every root-package path named in `PHASE-03`…`PHASE-10` is a **citation, not an
  edit** — checked. `src/git.rs:2718` appears only to say the export build does
  *not* reach `fetch_refspec`; `src/clock.rs`, `src/reserve.rs`, `src/tty.rs`,
  `src/install.rs` are cited as precedent or rule.

So after `PHASE-02` the clone's whole edit surface is `crates/doctrine-control/**`,
appends to `layering.toml`'s **`doctrine-control`** tier section (created by
`PHASE-01`, a region no other work touches), and one `[capsule]` table in
`.doctrine/doctrine.toml`.

### The hazard to respect in the clone

**Entity id allocation.** The CLI allocates sequential ids by scanning the corpus
it can see, which in a clone is frozen at fork time while `edge` keeps minting.
Two trees can mint the same `DEC-NNN` / `ISS-NNN` / `RV-NNN`, and renumbering to
resolve it breaks the immutability rule everything else references.

**Rule: the clone mints no entities.** Capture decisions and findings in the
phase sheets and here; mint them on `edge` at merge. The exception is
`doctrine observation record` — records are UUID-filenamed, so they are
collision-free and can run freely. Related: `QUE-208` (capsule-side entity id
allocation) is the same problem one level down.

### At merge

`Cargo.lock` and `.doctrine/adr/001/layering.toml` are the two foreseeable
conflicts. Regenerate the lock with a build rather than hand-resolving it.
`.doctrine/state/` is gitignored, so phase status flips never travel — the
tracking TOMLs on `edge` will still read `planned`; that is runtime state and
`/audit` reconciles it.

Courtesy owed on landing `PHASE-01`: it changes `just check`'s scope for everyone
on `edge` — the fast inner loop starts compiling and testing `doctrine-control`,
and `Cargo.lock` gains a member entry.

### Model

`PHASE-01` on **Opus**: it carries the two-tree layering gate (`T5`) and the
`load_layering` change (`T8`), where a vacuously-passing gate is
indistinguishable from a correct one, and it lands directly on `edge` where a
mistake is shared rather than isolated. From `PHASE-03` the work is
well-specified new files in an isolated clone and Sonnet is viable — with the
standing instruction that a criterion which does not compile as written is a
**stop-and-report**, never an adjust-the-criterion. This design has already
produced one such criterion (`PHASE-01` `EX-4`, `DEC-181`) and one count wrong in
five places.

## Owed to the reconciliation brief

<!-- Lifted from the disposable phase sheets so it survives `rm -rf`ing
     `.doctrine/state/`. Each item is a place the locked design or an approved
     criterion is measurably wrong; none is fixed by reopening the design. -->

### From `PHASE-01` execution (`74c398acb`, `6b58d48c8`)

1. **`DEC-181` is superseded by measurement, and needs re-ruling.** The ruled
   `pub use` shim at the bin crate root does not build here: `clippy::pub_use` is
   `deny`, and a per-item `#[expect]` on the `use` hits
   `clippy::useless_attribute` (clippy allows lint attributes on `use` items only
   for a hardcoded allowlist that `pub_use` is not on). `DEC-181`'s four-row
   table was measured on a scratch package under the *rustc* lint config only, so
   the whole clippy restriction list went unexercised — and the repo held no
   `pub use` anywhere, so the lint had never fired. **Shipped instead:**
   `pub mod clock; pub mod config_file; pub mod git;` at the bin crate root — a
   fifth arrangement, absent from the table, that satisfies `unreachable_pub` by
   the same mechanism the shim reached for and costs no suppression at all.
   `EX-3` and `EX-4` cite the shim explicitly and are therefore diverged-from as
   written; `EX-4`'s load-bearing claim (`today` promoted, never dropped) is
   untouched. Recorded durably as `mem.pattern.lint.bin-plus-lib-export-visibility`.

2. **`sec-6`'s `crate::` closure is right as a class and one instance short as an
   enumeration.** `EX-1` names four private modules. Measured: five — `kinds` is
   a *directory* module and `kinds/resolve.rs` reaches `crate::fsutil`, which is
   an `E0432` on the plain lib build, not merely the test build. Declaring
   `fsutil` then makes `unused = "deny"` flag 17 items across `clock`, `fsutil`
   and `kinds` as dead: live in the bin target, dead in the lib one by
   construction. `src/lib.rs` carries `#![expect(dead_code, unused_imports)]`
   and `#![expect(clippy::pub_use)]` accordingly — the latter superseded in
   strength by `VT-1`'s `EXPORTED` assertion, which bounds the surface far more
   tightly than the lint.

3. **`sec-8`'s `cordage` premise is wrong for the `lint` leg.** The section
   argues `default-members` is needed because a new crate inherits `cordage`'s
   exclusion "including `cargo clippy`". Measured: `cordage` is a *path
   dependency* of the root package, so it is in the build graph regardless, and
   bare `cargo clippy` runs `clippy-driver` on it under the full workspace
   deny-set with no `--cap-lints` — identically before and after this phase
   (verified with `cargo clippy -p doctrine`, the pre-change selection). The
   ruling stands: nothing depends on `doctrine-control`, so without the key it
   would be in no build graph at all. Only the analogy was wrong. The exclusion
   that is real is `test`.

4. **`sec-6` / `sec-8`'s "35 call sites across 33 files" is wrong** (carried from
   `/phase-plan` as `F-2`). Measured: 50 occurrences across 21 files. The
   load-bearing claim — no call site is edited — holds.

5. **`sec-9` `R6`'s baseline is wider than recorded and still zero.** 13 textual
   `doctrine::` hits across 4 files under `src/` (`regression.rs`,
   `design_run/document.rs`, `spec.rs`, plus one comment in `main.rs`); every one
   is a string literal or a comment, and no `use` statement names `doctrine::`.

### From `PHASE-02` execution (`10c57a30b`…`a11b158e0`)

6. **`sec-8`'s "it acquires no exemption by being absent from the binary" is
   half true, and the half that fails is the conclusion** (`design.md:4499-4505`).
   `src/interpretation.rs` *is* discovered as a unit by the file-tree walk, as
   the section says. But the gate's completeness assertion demands a tier only
   for units appearing in an **edge**, and this module has neither an in-edge nor
   an out-edge (`out=0` by design; un-imported because `main.rs` deliberately
   does not declare it). **Measured:** deleting `interpretation = "leaf"` from
   `layering.toml` leaves `architecture_layering_gate` green — the only test that
   reddens is `every_exported_item_belongs_to_a_leaf_tier_module`, a different
   mechanism that applies only because this module happens to be exported. `EX-8`
   is satisfied and protected; the design's stated reason is not the one doing
   the work. Filed as **`ISS-326`** (the gate gap, separable and general);
   `VA-1`'s wording in `plan.toml` inherits the same error and is worth amending
   at reconciliation.

7. **`sec-4`'s rule-4 classification has a vacuous fourth case.**
   `design.md:1818-1825` orders the diagnosis *removed → inserted → reordered →
   otherwise*, and "otherwise" is unreachable: if no base row is missing, the
   base either is a subsequence of the refinement (`Inserted`) or is not
   (`Reordered`). A replacement is a removal plus an insertion and the removal is
   diagnosed first. `RestrictionRefusal::VerificationRowReplaced` therefore ships
   named-but-never-constructed, documented as such at the variant.
   **Recommend deleting it at reconciliation** — `PHASE-06` is the first
   consumer, and an impossible variant it must match on forever is a real cost.
   `EX-7` names the four cases and would move with it.

8. **`sec-4` rule 1 (`SchemaMismatch`) cannot be reached through `parse`.**
   `parse` accepts exactly `INTERPRETATION_SCHEMA`, so two parsed policies always
   agree. The rule guards a future v2 rather than any present input; its test is
   the module's only hand-built fixture and says so. Not a correction — a
   property of the algebra worth stating once in the brief, since an auditor
   reading `restrict` will ask.

9. **`sec-4` case 1 is set-shaped where the values are a sequence.** "Some base
   row appears nowhere in the refinement" mis-diagnoses `base = [A, A]`,
   `refinement = [A]` as a reordering; duplicate verification rows are legal
   (only the two set-valued lists reject duplicates). Shipped multiset-aware, so
   that case reports `Removed`. Mechanical, inside the design's intent.

10. **`sec-4`'s validation table enumerates value rules, not shape refusals.**
    Ten rules; thirteen `PolicyRefusal` variants. The four additions — `NotToml`,
    `BlockMalformed`, `ListMalformed`, `EntryMalformed` — are what a total parser
    needs before the table's rules apply. `NotToml` is the only variant carrying
    a dependency's message, and it *carries* rather than *matches* it, which is
    the distinction `sec-4` § *Why the table is walked* is protecting.

### From `PHASE-03` planning (sheet `phase-03.md`, pre-execution)

<!-- Found at plan time, not execution time. No ids minted — this is the clone
     (§ The hazard to respect in the clone). Mint on the parent at merge. -->

11. **`sec-5`'s `HostFacts` sketch contradicts `sec-6`'s unit table, and the gate
    enforces the table.** `sec-5:2247` returns `Result<ByteCount, CapacityUnknown>`;
    `EX-3` puts `ByteCount` in `config.rs` and `EX-6`/`EX-12` force `config → host`,
    so the sketch requires `host → config` and closes a two-node `leaf`-tier cycle.
    The `doctrine-control` tree's tangle baseline is synthesised at **zero**
    (`tests/architecture_layering.rs:517-521`, `PHASE-01` `D1`), so this is a hard
    gate red with no baseline escape and no local fix — the file that could
    baseline it is not this phase's. `sec-6`'s table and `EX-18` are the coherent
    pair; the sketch is the outlier, and `EX-2` already amends the same sketch on
    the clock axis. Resolved in-criteria by the sheet's `D1` (return raw `u64`,
    move one `ByteCount::from` to the call site) — **no `EX`/`VT` altered**. Owed
    as a design correction, not a criterion failure.

12. **`Argv` is used in five places and defined nowhere.** `design.md:298`,
    `:2095`, `:3207`, `:3367`, `:3402`; `sec-6`'s file map assigns it to no unit.
    `config.rs` is the only placement that adds no edge the table lacks
    (`backend → config` is already recorded). Design omission.

13. **`assess_capacity` cannot name the capsule root.** `EX-14` fixes the
    signature at `(probe, &CapacityPolicy)`; `EX-16` and `VT-2` both require the
    root in the emitted warning and refusal, and neither parameter carries one —
    `CapacityPolicy` deliberately holds only the expected size and multiplier
    (`EX-20`'s no-reservation structure depends on that). Resolved by the sheet's
    `D3`: the verdict stays pure and rootless, the *report* carries the root from
    the caller. Both criteria satisfied as written; recorded because the next
    reader meets the same tension.

14. **`plan.md`'s file-ownership table omits `main.rs` from `PHASE-03`.**
    `plan.md:212` gives `PHASE-03` four files and lists `main.rs` under phases 01,
    06, 10 — but the three new modules never compile, so their ~33 tests never
    run and `EN-1` is defeated, unless `main.rs` declares them. Harmless in
    practice (03/06/10 are strictly sequential, the edit is three lines); the
    table should read `main.rs | 01, 03, 06, 10`.

15. **`sec-5` cites `src/install.rs:1818` for the `HOME` read precedent; the site
    is now `src/install.rs:1505`.** Same class as item 4's stale figure. Trivial,
    costs the next reader a grep.

16. **`execution-timeout-seconds`'s justification measures the wrong workload,
    and the fix is a second key rather than a bigger number.** `sec-5` derives
    `900` from the spike's Rust *build* fixture (352s measured, re-bounded to 900
    with headroom). But `ADR-020` makes the capsule the dispatch authority
    boundary, `DEC-134` fixes the headless worker, and the capsule mounts
    `/agent` as "the state a harness accumulates across a run" — so what a
    capsule executes is an **agent**, and `Execution.timeout` bounds it. An agent
    run is not a build: this slice's own `PHASE-03` planner sub-agent took 650s
    writing no code and running no builds, and a worker phase on this slice has
    been taking about a session. At 900 an agent is `SIGTERM`ed mid-phase.

    **Owner's ruling, 2026-08-08 — `900` stands, scoped to build/verification
    execution, and agent execution gets its own bound as separate work.**
    Rationale, in the owner's terms: the two workloads have very different
    characteristics, and a bound slack enough for an agent means waiting two
    hours to discover a `cp` typo. One key serving both is wrong in both
    directions — too tight for the agent, too slack to fail a build fast.

    So this is **two** items at reconciliation, and they are not the same kind:
    (a) a design correction — `sec-5`'s justification for `900` should say it
    bounds a build/verification contract, not any capsule execution; and (b)
    **net-new work**, a second timeout key with its own parse, default posture
    and enforcement, sized against an agent's tail rather than a build's. (b) is
    a backlog item, not a design fix — mint it on the parent at merge
    (this clone mints no ids). Out of scope for `PHASE-03`: `EX-19` fixes the key
    set, so adding the key here would break an `EX` as written (`S4`).

`EN-3`'s overstatement resurfaced at plan time and is **not** a new item — it is
item 6 (`ISS-326`). It holds for `PHASE-03`'s three units only because all three
appear in edges; the sheet's `T12` verifies that by deleting a row rather than by
reading the criterion.

`VH-1` is **discharged** by the ruling above — the five figures are confirmed as
`sec-5`'s sample writes them (`900` / `512` / `8192` / `2` / `5`). Recorded in
the `PHASE-03` sheet as `F-7`, which supersedes `T14`'s escalation clause.

### From `PHASE-03` execution (`2f3a6db78`…`2a22084da`)

17. **`EX-11`'s fieldless refusal variants cannot name the keys three `VT`
    titles require them to name.** `VT-4`/`VT-7` mandate
    `closure_roots_without_a_resolver_refuses_naming_both_keys`,
    `neither_variable_usable_refuses_naming_the_config_key` and
    `both_readable_lists_empty_refuses`, while `EX-11` fixes
    `ClosureRootsWithoutResolver`, `NoReadableInputs`, `EmptyResolverArgv` and
    `UnresolvableCapsuleRoot` as **fieldless**. The key survives only inside the
    variant's identifier, which no test can assert and no operator can grep.
    Resolved in-criteria by a structured `ConfigRefusal::keys() -> &[&'static
    str]` returning the `KEY_*` constants — variant shapes untouched, and
    structured rather than formatted, matching the posture `EX-16` takes for the
    capacity warning. Same class as item 13: jointly satisfiable, but only once
    someone notices the seam. Not a criterion failure.

18. **`EX-17`'s literal manifest spelling was met by inheritance.** `serde` and
    `toml` are declared `{ workspace = true }` rather than `EX-17`'s
    `serde = { version = "1", features = ["derive"] }` / `toml`.
    `[workspace.dependencies]` carries exactly those, so the *resolved*
    dependency is character-for-character what the criterion names, and
    inheritance is the root package's own convention (`Cargo.toml:61`). `rustix`
    is spelled inline because it is not a workspace dependency. Recorded so an
    auditor diffing the manifest against `EX-17`'s text does not read it as a
    deviation. Form, not substance.

19. **`VA-1`'s suggested grep cannot return empty, and would have produced a
    false stop.** The bare alternative `now` is a substring of
    `CapacityUnknown` — the type `EX-14` requires in `host.rs` — so the grep
    matches fourteen lines of correct code. `VA-1` itself holds; only the method
    was broken. The sound form strips comment lines and uses word boundaries.
    Recorded because an agent running the suggested grep literally could
    reasonably have raised a false criterion failure, which is a worse outcome
    than the grep simply being wrong.

### From `PHASE-04` planning (sheet `phase-04.md`, pre-execution)

20. **`sec-6`'s file map contradicts `sec-6`'s own unit table, and the table is
    right — for the second time.** `design.md:2499-2500` assigns `AcceptedBase`
    to `transaction.rs` and assigns `TransactionRoot` and `SourceExport` to no
    unit at all, while `design.md:2516` records the edge `transaction → backend`
    and `sec-2`'s `CapsulePlacement` carries all three. Defining them in
    `transaction.rs` forces `backend → transaction` and closes a two-node cycle
    the tree's synthesised **zero** tangle baseline rejects — the same mechanism,
    in the same tree, as item 11. Resolved in-criteria by the sheet's `D1` (all
    three in `backend.rs`; `PHASE-06`'s `AcceptedBase` keyword mandate is
    satisfied by the import and the field) — no `EX`/`VT` moves. The brief should
    say the *pattern* out loud: when `sec-6`'s prose and its table disagree, the
    table has been right both times, and the zero baseline is what makes the
    disagreement load-bearing rather than cosmetic.

21. **`EN-3`'s test estimate is stale.** It predicts "≈20 pure tests"; the `VT`
    set mandates **28** titles and the sheet adds two. `EN-3`'s load-bearing
    claim — crate in the checked set, no dependency added — is untouched, and
    both `plan.md:141` and `plan.toml:327` already concede that `sec-8`'s ≈39-test
    row is *shared* across `PHASE-04`/`PHASE-05` rather than apportioned. Sizing
    only; recorded so an auditor counting tests against `EN-3` does not read 28
    as scope creep.

22. **`plan.md`'s file-ownership table is wrong in two further ways — and item 14
    is now the same finding three phases running.** (a) `plan.md:213` gives
    `PHASE-04` only `backend.rs`, but nothing compiles it and none of its 28
    tests run unless `main.rs` declares `mod backend;` — exactly item 14's defect,
    unamended, so the row should read `main.rs | 01, 03, 04, 05, 06, 10`.
    (b) `plan.md:213-214` treats `backend.rs` and `backend/bubblewrap.rs` as
    exclusively owned by `PHASE-04` and `PHASE-05`, but `PHASE-05` must add
    `mod bubblewrap;` to `backend.rs`, so that file is shared 04/05 by one line —
    which `PHASE-05` `EX-19` half-anticipates. Recorded ahead of time so
    `PHASE-05`'s worker does not read its own one-line edit as a boundary
    violation and stop. A table that is wrong the same way in three consecutive
    phases is a defect in the table, not three incidents.

### From `PHASE-04` execution (`5f6eec999`, `4535efde8`)

23. **A mandated test would have passed whether the rule it tests existed or
    not.** `D5`'s arrow puts the filesystem-root check last, and taken as a
    trailing stage it is **unreachable**: `/` is an ancestor of every scope, so
    the general overlap rule answers first and
    `PlacementRefusal::FilesystemRoot` can never be constructed — leaving
    `declared_root_that_resolves_to_the_filesystem_root_refuses` green either
    way. Fixed by testing it per declared entry, immediately *before* the general
    overlap, so the specific diagnosis wins; `D5`'s stage order is unchanged and
    no `EX`/`VT` moved, and mutation `M5` now measures that the test
    discriminates. The brief should carry this as the **vacuous-criterion**
    class: a green mandated test is not evidence its rule is reachable, and only
    a mutation shows the difference. Related in kind to items 7–8 (`PHASE-02`'s
    unreachable algebra variants).

24. **The sheet prescribed a TDD sequence this repository's lint configuration
    forbids.** `T2`–`T10` are written as red/green/refactor cycles over one
    staged module, but the module's `#![cfg_attr(not(test), expect(dead_code,
    …))]` header is stripped under `cfg(test)`, and `unused = "deny"` then makes
    every item lacking a *test* reader a hard error — so the module does not
    compile at all until the last test lands. Measured: 69 errors after the
    type-vocabulary task, **no partial-green state**, then all 30 tests passing
    on the first successful compile. The red phase was reconstructed as a
    seven-mutation battery, which is sound but is not what the sheet asked for.
    **Actionable now, not at reconciliation:** `PHASE-05` is the same shape, so
    its planner must prescribe *write-all-then-mutate* rather than per-task
    red/green. Harvested as
    `mem.pattern.lint.staged-module-unused-deny-collapses-tdd`.

25. **Two in-criteria readings, recorded so an auditor need not re-derive them.**
    (a) The transaction-root carve-out admits an entry *equal to* the transaction
    root, not only a strict descendant — the root is the placement's own writable
    state, refusing it is the over-denial direction `R2` and `RV-346` `F-25` warn
    about, and no criterion or title covers equality. (b) The source must
    *strictly* descend from `<capsule_root>/export/`; the export directory itself
    refuses, since the contracted export is `<capsule_root>/export/<base-oid>`.

**Measurement worth keeping:** the pre-round-4 blanket rule (`M1`) reds **20 of
24** placement tests, not merely the one control the sheet predicted — `RV-346`
`F-25`'s claim that "every conformance row would have failed before running"
reproduced as a measurement rather than an assertion.

Items 26–30 are `PHASE-05` **plan-time** findings (sheet `F-1`…`F-8`), raised
before any source was written. That they are plan-time is itself the point:
four of the five are criteria the design mandates but the workspace cannot
satisfy or a method that would have produced a false failure, and all four were
found by reading and *executing probes* against the criteria rather than by
running into them mid-implementation.

26. **A phase's exit criteria mandate a mechanism the workspace lint posture
    forbids, and the conflict is not resolvable inside the phase.** `EX-15`
    (per-file `RLIMIT_FSIZE` on the child) and `EX-16` (the `/proc/self/fd`
    CLOEXEC sweep) each require an `unsafe` expression — `CommandExt::pre_exec`
    and `BorrowedFd::borrow_raw` respectively — while `Cargo.toml:200` sets
    `unsafe_code = "forbid"` workspace-wide, and `forbid` cannot be excepted by
    `#[expect]` or `#[allow]`. Verified by executing `rustc` probes, not by
    reading. There is **zero** `unsafe` in the repository, so the posture is
    real rather than nominal. `EX-15` additionally needs `rustix`'s `process`
    feature, which `crates/doctrine-control/Cargo.toml:42-44` deliberately
    leaves undeclared citing `RV-346` `F-29`. The design never reconciled its
    own § *Descriptors* / § *Bounds* mechanisms against the workspace posture;
    the brief should record that the gap was structural and reached the phase
    sheet unflagged. **Ruling owed to the slice owner** — see § *Open*.

27. **`sec-6`'s unit table diverged from this slice's needs a third time — and
    in a new direction.** The table gives `backend` the single out-edge
    `config`, but `EN-4` requires `bubblewrap.rs` to reach `HostFacts` for its
    hermetic `PATH` and declared-list probes, so the unit acquires
    `backend → host`. Both are `leaf`, `host` is out=0, nothing cycles, no
    `layering.toml` row moves. Items 11 and 20 were the table being *right*
    against contradicting prose; this is the table being *incomplete*. Three
    consecutive phases is a pattern in the artefact, not three incidents.

28. **A named `Termination` variant had no observable behind it.** Measured:
    bubblewrap exits **1** when `execvp` fails — indistinguishable from a
    capsule that legitimately exits 1, which `PHASE-04` `EX-14` requires be
    `Ok(Exited { code: 1 })` — and writes its diagnostic to the *shared*
    stderr, which is capsule-forgeable and so unusable without violating
    invariant 8. The design's § *Bounds* and § *Descriptors* are silent on the
    channel. Resolved in-criteria via `--json-status-fd` (measured to emit
    `exit-code` only when the child ran, and measured **not** to cross into the
    capsule). The design owes a sentence naming the mechanism.

29. **`EX-4`'s flag list does not say whether it is exhaustive**, and the two
    readings differ in consequence: as an ordering constraint on the
    confinement flags it is satisfied by a leading `--json-status-fd`; as an
    inventory of bwrap options it is violated by it, and item 28's resolution
    becomes a STOP. Ruled in-criteria on the ordering reading. The brief should
    make the criterion say which it is.

30. **A `VA` criterion is false as literally written — the third of its class.**
    `VA-2` ("no bubblewrap flag token appears anywhere outside this module")
    holds within `crates/doctrine-control/` and fails immediately across the
    workspace: 67 hits in six root-package files, which is the worktree arm
    `DEC-155` decided *not* to reuse. Scoped to the crate. Same class as item 19
    — the criterion holds, the *method* needed naming, and an agent running the
    literal grep would reasonably have raised a false failure. Items 19 and 30
    are the mirror of the vacuous-criterion class (items 7, 8, 23): there a
    passing check proved nothing; here a failing check disproves nothing.

**Measurements worth keeping (`PHASE-05` plan time, bubblewrap 0.11.2).**
(a) bwrap **creates** the mountpoint inside a `--tmpfs` for a bind resolving
beneath it — the design declined to claim this either way, and `EX-3`'s lawful
case does hold. (b) The *same* probe with `--ro-bind` before `--tmpfs /tmp`
leaves `/tmp` empty inside the capsule: the tmpfs silently shadows the earlier
bind, exit 0, no error anywhere. `EX-4`'s assembly order is load-bearing, and
this executed control is stronger evidence than the argv-shape assertion the
sheet mandates. (c) `--uid`/`--gid` take effect under nesting inside this
project's own jail, but the overflow gid `65534` survives as a supplementary
group — recorded for `PHASE-10`, whose rows 13/14 assert invariant 15's "no
supplementary groups", so `--uid`/`--gid` are not later read as discharging it.

Items 31–39 are `PHASE-05` **execution** findings (sheet `F-9`…`F-19`). Two of
them (31, 32) were produced by the mutation battery and by nothing else, which
is the strongest evidence this slice has that the battery earns its cost.

31. **A real defect, found by the battery rather than by review.** The CLOEXEC
    sweep `mark_inherited_descriptors_close_on_exec` skipped a descriptor whose
    `fcntl_getfd` failed but **propagated** a failing `fcntl_setfd` — so a
    descriptor another thread closed between the listing and the mark failed the
    whole sweep, roughly **1 full-suite run in 20**. Its own doc comment already
    claimed the skip; the code implemented half of it. A flake that rare would
    have shipped and been blamed on the test suite. It surfaced only because the
    battery made an *unexplained extra red* visible against a known expectation
    — the value was in the expectation, not the mutation. Fixed to skip on
    `Errno::BADF` and only that errno (a sweep that swallows every failure is a
    guard that cannot fail); 25 consecutive clean runs. Harvested as
    `mem.fact.rust.cloexec-sweep-races-on-ebadf`.

32. **A mutation redded nothing — and the criterion was fine.** `M10` (sort the
    derived `PATH` instead of preserving host order) redded **zero** tests: the
    fixture's host `PATH` was `/opt/toolchain/bin:/usr/bin`, whose host order and
    lexical order coincide, so the mutation was a no-op *on that input* and the
    assertion held under either rule. The rule and its mandated title were
    untouched; the **fixture** could not discriminate. Fixed by a fixture whose
    two orders disagree, plus an `assert_ne!` against a sorted copy so the
    fixture's discriminating property is itself asserted. This is the
    vacuous-criterion class (items 7, 8, 23) caught *by the method the sheet
    mandated* — the criterion was sound, the evidence for it was not, and only
    the battery could tell the difference. Harvested as
    `mem.pattern.tests.mutation-needs-a-discriminating-fixture`.

33. **Does the `forbid` → `deny` flip warrant an ADR? The worker recommends yes,
    and so do I.** `F-1/R` asked the question explicitly and did not settle it.
    The flip is repo-wide, is not reversible without finding every site, and
    replaces a guarantee (`forbid` cannot be overridden by anyone, at all) with a
    categorically weaker one. What makes `deny` acceptable is
    `the_unsafe_budget_is_exactly_two_sites` — and **a test can be deleted by
    anyone with no governance trace**, whereas an ADR records why the ceiling
    exists. **Mint on the parent at merge.**

34. **Three `S5`-blocked debts against `backend.rs`, all one-line, all owed to
    whoever owns that file next.** (a) `backend.rs:50-53`'s module doc says the
    `backend` unit is "`leaf`, out-edges `{config}`" and adds "no row **and no
    edge**"; with `bubblewrap.rs` importing `crate::host` the out-edges are
    `{config, host}` — the "no row" half holds, the "no edge" half is now false
    (see item 27). (b) `CapsuleEnvVar` has `ALL` and `fixed_value()` but no
    `name()`, so the seven environment-variable *names* are single-sourced in
    `bubblewrap.rs` — names and values now live in different files, which is the
    `STD-001` shape. (c) `backend.rs:1649`/`:1660` carry the bare string
    `"bwrap"` in a PHASE-04 test fixture, now duplicating `bubblewrap.rs`'s
    `BWRAP_EXECUTABLE`. None were fixable in `PHASE-05`: `S5` scoped it to one
    line of `backend.rs` and `F-1/R` widened that by two manifests and nothing
    else. The constraint worked exactly as intended — it converted three
    temptations into three recorded debts.

35. **`Execution` carries no kill grace, so `timeout -k` had no configured
    figure.** `config::ResourceBounds::kill_grace()` parses
    `execution-kill-grace-seconds`, but PHASE-04's `Execution` carries `argv`,
    `env`, `timeout`, `file_size_cap`, `stdio` — and no grace. Worked around
    without touching `backend.rs`: `BubblewrapBackend` holds a `kill_grace`
    defaulting to 5s with a `with_kill_grace` builder for `PHASE-06` to thread
    the configured figure through. **The clean fix is a field on `Execution`**,
    and this is the note that stops a temporary default becoming permanent.

36. **`D4`'s pure signature cannot perform `D5` step 4.** `D4` fixes
    `closure_members(&QueryOutput, resolver) -> Result<Vec<PathBuf>, ProfileRefusal>`,
    but `D5` step 4 is existence-and-resolution of each returned path, which
    needs `HostFacts` — impure, unavailable behind that signature. Resolved by
    keeping `closure_members` exactly as `D4` specifies and doing step 4 in
    `expand_closure_root`, which has the host. `D5`'s step list should say which
    half owns step 4.

37. **`sec-2`'s assembly list omits the three profile-owned binds.** `/source`,
    `/capsule` and `/agent` cannot arrive through either declared vector —
    `RESERVED_INNER_DESTINATIONS` refuses an entry naming them (PHASE-04 `EX-6`)
    — so the backend derives all three from the placement's typed fields. Ruled
    in-criteria on the same order-not-inventory reading as item 29, but a reader
    checking the design's list against the real argv finds three flags the list
    does not mention. The design owes a sentence.

38. **`D6` enumerates seven `ProfileRefusal` variants; eight were needed.**
    `EmptyReadableSet` is the fail-closed floor for *every source declared, none
    produced anything* — reachable when a closure resolver returns no members,
    which `ConfigRefusal::NoReadableInputs` cannot catch because it runs at parse
    time, before expansion. Without it the backend would assemble a capsule with
    no readable input and report **confinement success** for what is really a
    configuration failure. It is also what makes the sheet's own mandated
    `the_readable_set_is_never_empty` reachable at all.

39. **The MSRV forbids a pipe, so `D11`'s status channel is a file.**
    `std::io::pipe` stabilised in 1.87; the workspace pins `rust-version =
    "1.85"` and `clippy::incompatible_msrv` is in the denied `all` group.
    `rustix`'s `pipe` feature is undeclared and `F-1/R` widened `S5` by exactly
    two manifests, so declaring it was not this phase's call. Resolved with a
    file under the transaction root, outside every bound path so the capsule can
    neither read nor forge it, cleared of `CLOEXEC` *after* the sweep and removed
    *before* `disk_used` is measured so the trusted side's bookkeeping is not
    billed to the capsule. If the MSRV moves, a pipe is the tidier form.

### From `PHASE-06` planning (sheet `phase-06.md`, pre-execution)

40. **`EX-9` / `VT-5` cannot be built against `src/interpretation.rs` as it
    stands — the second phase in a row stopped by a criterion that needs an edit
    outside its own file ownership.** The module has **zero `impl` blocks**;
    `forbidden_executables` and `ExecutableName`'s tuple field are both private,
    with no accessor, no `Deref` and no exposed round-trip. So there is no way
    for `doctrine-control` to ask a restricted policy whether a resolver
    basename is forbidden. Re-deriving `sec-4`'s normalization locally would be
    a second spelling of a one-module rule (`STD-001`); reading the base TOML
    directly bypasses `restrict` and checks the *base* list, the precise
    ordering error `EX-9` exists to prevent. Ruled below.

41. **`plan.md`'s file-ownership row is wrong for the fourth consecutive
    phase.** Items 14 and 22 record `PHASE-03`, `04`, `05`; `PHASE-06`'s real
    reach is six files wider than the table says. This has now been wrong every
    time it has been checked, which makes it a defect in the table rather than
    four incidents. Recommendation for the brief: regenerate the table from the
    phases' criteria, or drop it in favour of the per-phase sheets, which have
    been right each time.

42. **`EX-14` and `design.md:1398` disagree on the execution count, and the
    criterion is what a worker reads.** `EX-14` says "three separate
    `backend.execute` calls" and enumerates three `git` commands; step 12 at
    `design.md:1398-1401` adds a fourth that reads the identity back. A worker
    reading only the exit criteria builds three and then satisfies
    `capsule_identity_persists_into_the_clone_config` by a *trusted-side* read of
    `<root>/capsule/repo/config` — a different assertion, and a trusted-side
    touch of a capsule-authored repository (`SPEC-030`). No criterion needs to
    move (`EX-14`'s subject is the clone; step 12 is a separate step), but the
    total is four and three is reachable honestly. Pre-empted in the sheet.
43. **The thirteen steps are a rule list, not a call list: steps 2 and 7 are one
    call.** The declared-entry existence probes (step 2) and the closure
    expansion (step 7) are both already inside PHASE-05's `readable_set`, and
    separating them would re-implement it (`EN-2` forbids). `provision` calls it
    once, at step 7's position. `EX-9`'s ordering still holds by construction —
    step 6 runs strictly before the one call that can invoke the resolver — but
    a reader checking thirteen steps against thirteen statements finds no
    statement for step 2. Renumber, or say the list is rules. `PHASE-06` `F-4`.
44. **`ForbiddenScopes`' credentials list has no configuration source, and is
    passed empty.** Neither `[capsule]` (`EX-1`'s field set) nor
    `ProvisionRequest` (`EX-6`) carries one, and inventing a source would be a
    new configuration surface with no criterion behind it. The consequence is
    exact: a declared readable entry naming `~/.ssh` or a token file is not
    refused by the placement's scope check today. It is still read-only and
    still authored trusted-side, so this is a defence-in-depth gap, not an open
    door — most likely routed alongside admission (`REQ-455`). `PHASE-06` `F-5`.
45. **A non-refusing capacity report has no return channel, so it goes to
    stderr.** `EX-6` fixes `provision`'s three parameters and `EX-1` fixes
    `CapsuleTransaction`'s nine fields, so a warn/report outcome cannot be
    returned. `provision` refuses when the report refuses and otherwise writes
    one structured `key=value` line to a locked stderr — a side effect in a
    function the design describes as returning a value, invisible to a
    non-terminal caller. Decide: an observation field on the transaction, a sink
    parameter, or stderr made explicit. `PHASE-06` `F-6`.
46. **`sec-3`'s `SliceId` / `PhaseNumber` do not exist, and `EN-3` forbids
    adding them.** `PhaseIdentity` carries `String` / `u32` instead. `VT-6`'s
    keyword floor is met; what is lost is parse-don't-validate on the slice id,
    which is now an unvalidated `String` — a caller can pass `"248"` or
    `"sl-248"` and nothing objects. `PHASE-06` `F-7`.
47. **A mandated `VT` keyword is a floor on test *names*, and a name is not
    evidence — three-for-three now.** PHASE-06's battery found three mandated
    tests that redded under no mutation: the identity test covered neither step
    12's wiring nor the clone's two `-c` pins, and the adoption test could not
    discriminate adoption from build-lose-adopt, whose observables are identical.
    All three were fixed in the tests, never in the rules. With `notes.md` items
    31 and 32 this is the third consecutive phase in which the battery found what
    review did not. Consider whether phase sheets should mandate the
    discriminating *fixture* alongside the keyword. `PHASE-06` `F-8`.

### From `PHASE-07` planning (sheet `phase-07.md`, pre-execution)

48. **`Axis` lands complete at PHASE-07, so a PHASE-09 criterion is discharged
    one phase early.** `EX-14` defers `Property`'s membership, and PHASE-09
    `EX-1` says `Property` gains its first eight variants "and `Axis` its five"
    there — which leaves `RowId { Property(Property), Axis(Axis) }` uninhabited
    at PHASE-07 and four mandated `VT` keywords unwritable as anything but names.
    No PHASE-07 criterion mentions `Axis`'s membership, so landing it here
    contradicts nothing in this phase; it does mean a later reader must not read
    PHASE-09 `EX-1`'s `Axis` clause as skipped. Same shape as `PHASE-06` `F-4`, a
    criterion satisfied by construction rather than by an edit. `PHASE-07` `F-1`.
49. **`plan.md`'s ownership tables miss two unavoidable edits, for the fifth
    consecutive phase, and `EX-1`'s out-edge list is aspirational.** `main.rs` is
    assigned to phases 01, 06 and 10, but a module file must be declared in the
    crate root or it is not compiled, linted, layering-checked or tested — so
    `mod conformance;` had to be appended. Separately, `EX-1` requires
    `layering.toml` to classify `conformance` as engine with out-edges
    `provision`, `transaction`, `backend`, `config`, `host`; at this phase the
    unit imports only the last three, and `provision`/`transaction` arrive with
    PHASE-08's fixture and freshness delta. The row was written with `EX-1`'s
    five and a trailing annotation naming which two are not yet real, which
    satisfies the criterion's text without asserting an edge that does not exist.
    `PHASE-07` `F-2`.
50. **`HostDescriptor` has no source at this altitude; the in-ownership route was
    taken and the better one is still open.** `EX-15` requires the verdict to
    carry the host, `verify`'s three parameters are fixed by `EX-2`, and
    `HostFacts` carries no OS, kernel or architecture. Shipped (`D7`):
    `std::env::consts::OS`/`ARCH` plus a `/proc/sys/kernel/osrelease` read with a
    documented `"unknown"` fallback — an engine unit reaching disk directly,
    which `provision` already does. Per the orchestrator's `F-3/N`, the whole
    derivation sits behind **one** named function, `host_descriptor()`, so the
    swap is a one-function replacement. Two better homes remain live at merge and
    both are `S1` edits: `rustix::system::uname()` (needs the `system` feature on
    `crates/doctrine-control/Cargo.toml`), or a `descriptor()` method on
    `HostFacts` (edits `host.rs`). **Lean toward the third**: `host.rs` exists to
    be the host-facts seam, and an engine unit reading `/proc` is a second door
    onto facts that module was built to own. A coupling argument, not a
    correctness one. `PHASE-07` `F-3` / `F-3/N`.
51. **`Indeterminacy`'s four variants cannot cleanly name "the subject exited
    before the observer ran".** `EX-11` closes the vocabulary; the design requires
    two concurrent failure modes to classify `Indeterminate`. The first (the
    backend returned without ever calling the observer back) maps cleanly to
    `NoLiveness`. The second does not map cleanly to anything — the observer
    *did* run and *did* emit its marker; what failed is the subject's liveness
    during the window, a property of the arm rather than of either payload.
    `NoObservation` is adopted for it (`D5`) and is a stretch, taken because the
    two modes must carry *different* reasons or `M5`/`M6` cannot be separated,
    and written into the doc comment where `Indeterminacy` is declared. A fifth
    variant was not taken. Worth revisiting at PHASE-08, which builds the
    choreography and will discover whether the two modes are actually
    distinguishable trusted-side; if they are not, `D5`'s witness is wrong.
    `PHASE-07` `F-4`.

### From `PHASE-07` execution (`570fe4853`)

52. **`D2`'s `verify_over` signature is one parameter short of testable.** `D2`
    specifies `verify_over(backend, host, today, &tables(), &run_row)`. `M13` and
    `M14` — the two mutations proving a `Failed` and a `Skipped` auxiliary
    outcome cannot reach `Admission` — are unfalsifiable unless a test can
    *inject* an auxiliary outcome, so the shipped signature also takes
    `auxiliary: Vec<(Claim, AuxOutcome)>`. `verify`'s own three-parameter
    signature (`EX-2`) is untouched and `admission(rows)` still takes the row list
    alone, so "table C cannot reach admission" remains structural rather than
    promised. Not a design defect so much as a reminder that `D2`'s split existed
    *for* the battery and was specified without it in view. `PHASE-07` `F-5`.
53. **A `fn`-pointer field costs a type its `PartialEq`, and `PartialEq` is what
    keeps sibling fields alive.** `PidProbe.argv` is `fn(HostPid) -> Argv` —
    `D3`'s pid-rendered payload — so `PidProbe`, and transitively `ArmShape`,
    `Delta` and `Row`, cannot derive `PartialEq`
    (`unpredictable_function_pointer_comparisons`, denied). `Clone` and `Debug`
    do **not** substitute: rustc ignores both for dead-code analysis. The four
    types therefore carry item-level `#[expect(dead_code)]`, all self-clearing at
    PHASE-08 or PHASE-09. No assertion was weakened — nothing compares two
    payloads — but the accounting is a standing cost for any later vocabulary
    type that holds a function. `PHASE-07` `F-6`.
54. **`doctrine slice verify-vt 248` cannot attribute this phase's `VT`s.** Run
    before and after committing, it reports PHASE-07 (and 08, 09, 10) as
    `≈ UNATTRIBUTABLE VT-N — keyword present but crates/doctrine-control/src/
    conformance.rs not modified by this slice`, while PHASE-01…06 `PASS`.
    `git diff --name-only edge...HEAD` does list the file, so it *is* in the
    branch delta and whatever base `verify-vt` resolves is not that. Two smaller
    oddities in the same output: phases whose tests do not exist yet also report
    "keyword present", and `UNATTRIBUTABLE` is indistinguishable at a glance from
    a genuine keyword miss. Not chased — the driver owns the attribution base.
    Friction observation recorded. `PHASE-07` `F-7`.
55. **Item 47's suggestion works: mandate the discriminating fixture, not just
    the keyword.** This sheet did exactly that — § *Verification map* spelled out,
    per keyword, what the fixture must carry (the *failed* token with no marker; a
    wrong-value sibling; a killed run that still printed; a non-empty row set with
    a counting runner; one row of each non-proven kind *plus* a `Proven` one) —
    and named the four sole-evidence mutations in advance. Result: **15 of 15
    mutations red their mandated test and none redded nothing**, breaking a
    three-phase run in which the battery found inert mandated tests every time.
    One data point, but it is the first phase where the fixture requirements were
    written at plan time rather than discovered at battery time.
56. **`mem.pattern.lint.dead-code-derives-count-as-reads` was wrong where it was
    load-bearing, and cost a compile cycle.** It claimed "the derived `Clone` and
    `PartialEq` impls read every field". rustc's own note: *"has derived impls for
    the traits `Clone` and `Debug`, but these are intentionally ignored during
    dead code analysis."* Only `PartialEq` counts. Its PHASE-02 measurement was on
    a type deriving `PartialEq` too, which alone explained the observation — the
    `Clone` half was inferred, not measured, and shipped at `trust = high`.
    Corrected in place; friction observation recorded. Worth a line in the brief
    because it is a corpus-quality failure mode rather than a slice one: a memory
    that generalises past its measurement is worse than no memory, since it is
    retrieved precisely when it is being relied on.
57. **Item 54 answered, and the answer is structural: a worker can never see its
    own phase's `VT`s attribute.** `verify-vt` resolves attribution against the
    phase's *recorded delta boundary*, and that boundary is recorded by the
    orchestrator at close — after the worker has exited. Run at close with
    `PHASE-07`'s boundary in place, the same command reports `PHASE-07` 3/3 PASS,
    0 `UNATTRIBUTABLE`; the four phases beyond it read as keyword misses, which is
    correct, because their tests do not exist. So `F-7` was not a defect in
    attribution but a diagnostic read from a position where it cannot yet be true.
    Two things for the brief, both cheap: the `UNATTRIBUTABLE` message should say
    *which* boundary it resolved against (naming the absent one would have closed
    this in a line), and phase sheets should tell workers that `verify-vt` is an
    orchestrator-close instrument, not a worker self-check — this cost a finding,
    an observation and a chase, and it will cost them again every phase until
    someone says so. Related: the CLI **refused** to narrow `PHASE-07`'s delta to
    the source commit alone, and the refusal was right — dropping the harvest
    commit would have manufactured the exact `UNATTRIBUTABLE` symptom `F-7`
    reported. Good error; it explained the consequence rather than the rule.

### From `PHASE-08` execution (sheet `phase-08.md`, shard `notes_07-09.md`)

58. **A product defect, and the one item on this list that blocks a later
    phase: `BubblewrapBackend::run`'s parent-side descriptor window is not
    thread-safe.** `run` mutates *process-wide* descriptor flags either side of
    its spawn — mark every inherited descriptor `CLOEXEC`, then un-mark the
    status file — and what reads those flags is `fork`. Two runs in flight at
    once corrupt each other's handover. Measured, not theorised: a capsule
    spawned with `--json-status-fd 4` holding **no** descriptor 4, and a
    *different* transaction's `bwrap-status.json` at descriptor 6; it then
    blocked at bubblewrap's user-namespace handshake for ever while holding the
    harness's capture pipe. Contained in-phase by a `#[cfg(test)]` mutex around
    the window — evidence about a phase should be about that phase — but the
    hazard belongs to the mechanism: **`ArmShape::Concurrent` (row B5) is
    exactly a caller that runs two capsules at once**, so `PHASE-09` must not
    build row B5 on the mechanism as it stands. Wants an issue and a production
    fix that narrows the window to the fork itself. Sheet `F-31`.
59. **`EX-12`'s "records the capsule's session id *before* the arm runs" is not
    realisable, and the phase's answer changes what row 7 proves.** No session
    exists before the capsule does; the seam that works is a callback *during*
    the run (`execute_noticing`). Compounding it: the escape the design
    attributes to `Teardown` is real but **unobservable by the harness**. The
    descendant outlives its parents *inside the pid namespace*, whose init
    holds the harness's capture descriptors until the namespace empties, so the
    arm cannot return while the escapee is alive — a `sleep` payload takes the
    arm to its wall bound and the escapee dies with the tree. The escape a
    harness can *see* is `ProcessVisibility`'s, where there is no pid namespace:
    end-of-file in 25ms, escapee alive in a session of its own. `F-9` stands
    (`--die-with-parent` is what reaps); `F-30` adds what `F-9` never measured,
    which is whether anything could watch. Row 7's criterion should name the
    property and not the removal.
60. **The design's `ConformanceBackend` sketch is two methods short of the
    rows it must serve.** `Delta::Granted` has no execution path at all
    (`execute_granted`, sheet `F-13`), and containment cannot be tied to a row
    shape, so a fourth defaulted method carries the pid seam for *every* shape
    (`execute_noticing`, `F-16`/`F-19`). Both are additive and both are in the
    design's own tables — the sketch, not the tables, is what is wrong.
61. **Two measured corrections to `D5`'s bubblewrap vocabulary.** `--share-net`
    is not what bwrap 0.11.2 enforces, and there is **no `--share-pid` at all**
    — `ProcessVisibility` is expressed by *omitting* `--unshare-all` and
    enumerating the other five unshares (`F-7`, `F-8`). The design's flag names
    should be replaced with the enumerated set, which is now a named constant.
62. **`Delta::SharedRoot` is a rebase of the placement, not a swap of its root
    field.** Swapping the root alone builds a placement `try_new` *refuses*:
    the writable carve-out is licensed by that placement's own transaction
    root, so the entries beneath must move with it. The design describes the
    swap. Sheet `F-29`; the mutation that expresses the defect (`M11b`) reds
    with `ForbiddenScopeOverlap`, which is the same trap from the other side.
63. **Invariant 4 was tested at the seam that *obeys* it, not the seam that
    *wires* it — and only the battery found that.** The probe-arm test built
    its own `Arm`, so it established that `run_arm` does not rewrite what it is
    handed and said nothing about `run_row`, the only place a delta could reach
    the probe. `M12` redded nothing. Closed in-phase with a new test over a
    `Recording` backend that provisions with the real mechanism and intercepts
    only the arms. For the brief: when a design says *nothing else would catch
    this*, the test has to sit at the seam the defect would enter by, and the
    battery is what tells you it does not. Sheet `F-33`.
64. **A deny list can make a plausible defect uncompilable, and a battery row
    that cannot compile is not a green row.** Three of `PHASE-08`'s 21
    mutations had to be re-expressed to run at all — `dead_code` and clippy's
    `drop_ref` refuse two of them outright, and a third reds 21 tests on a
    fixture refusal rather than on the property it targets. Re-expressed as
    `M11b`/`M21b`/`M20a`+`M20b` and recorded as such. Cheap fix for future
    sheets: when authoring a battery row, say which *lint* would catch it if no
    test does. Sheet `F-32`.
65. **`EX-2`'s "real disk, never tmpfs" had no mechanism, and the default route
    violates it on this host** (`F-4`); the second-filesystem probe is
    conditional by construction and cannot be made unconditional (`F-10`,
    `F-11`, `F-28`). The auxiliary claims that depend on a second filesystem
    report `Skipped` naming the reason, which is the honest answer and is now
    tested — but the criterion should say so rather than promise the reading.
66. **A killed run leaks its fixture root, and can leave a capsule alive.**
    `Drop` does not run for a process that takes `SIGKILL`, so 28 fixture roots
    and one orphaned `bwrap` — parented to init, blocked for two days — were
    waiting when `VA-1` was walked. The leak is bounded and self-identifying
    (dedicated base, pid-prefixed roots) and the remedy is to sweep the base,
    **not** to add a delete primitive: invariant 8 and `VA-3` forbid one, and
    `PHASE-08` introduced exactly one removal call in the whole phase
    (`TempRoot::drop`, over a root the fixture created). Sheet `F-35`.
67. **`plan.md`'s file-ownership table was wrong for this phase too — the
    sixth consecutive phase.** Recorded once more rather than argued: at this
    point the table's *shape* is the defect, not any individual row. Sheet
    `F-1`.
68. **A destructive test instrument must be floored before it is aimed, not
    after.** `EX-12`'s session sweep refused only the suite's own session — but
    the harness is not in it. In this jail `bwrap` is pid 1 in session 0 and the
    agent process is pid 2, also session 0, while the suite runs in a session of
    its own; session 0 was foreign to the guard and killable, and killing it
    takes the sandbox and the agent down with no error and no shutdown path.
    The operator lost their session twice. The mechanism, topology and symptom
    match exactly; the recording path was never caught in the act, and the claim
    is bounded to that. Floored at `3860a948c`. **The process defect is the
    sequencing:** the floor was scheduled as `T12` so as not to change code
    under test mid-battery — defensible in itself, and it put the safety guard
    *behind* the twenty-one rows that exercise the thing it makes safe. The
    battery passed; the teardown happened during `T12`, before the guard inside
    it was committed. Sheet `F-36`.

## Open

**RULED 2026-08-09 — option 1, free-function spelling. Nothing open here.** The
ruling is written into the `PHASE-06` sheet as `F-1/R`, which governs: one
`pub fn forbids(policy: &InterpretationPolicy, candidate: &str) -> bool` in
`src/interpretation.rs`, no `impl` block, basename derived inside the function so
a caller cannot fail open by passing an unsplit path, exact byte comparison (no
case folding — entries sort case-sensitively), two unit tests in that file, and
nothing else in the module moves. `S5` widened by exactly that one file. `T6` and
`VT-5` unblocked. Kept below for the auditor: the question as put, and the
options not taken.

**Was owed to the slice owner — `PHASE-06` `F-1`.** See item 40. `EX-9`, step 6
and `VT-5` were blocked until this was ruled; everything else in the phase
proceeded regardless, so the ruling gated one task and one test, not the phase.

Options, cheapest first — the first two both edit `src/interpretation.rs`, which
`plan.md` assigns exclusively to `PHASE-02`:

1. **One accessor carrying the comparison**, keeping normalization inside the
   owning module. The planner recommended the method form
   `pub fn forbids(&self, basename: &str) -> bool`; the orchestrator notes the
   module's own idiom is free functions over `&InterpretationPolicy`
   (`parse`, `restrict`, `canonical_hash`), so
   `pub fn forbids(policy: &InterpretationPolicy, basename: &str) -> bool`
   delivers the same thing without introducing the file's first `impl` block.
   Either spelling: no export-set movement — `interpretation` is already
   `pub mod` at `src/lib.rs:60` and `tests/architecture_layering.rs`'s `EXPORTED`
   list names the module, not its members — so no assertion, no
   `RootLibraryExports` binding and no layering row changes. Verified.
2. **Expose the data**: `forbidden_executables() -> &[ExecutableName]` plus an
   accessor on `ExecutableName`. Wider surface, and it moves the comparison rule
   out of the module that owns the rule, which is how option 3's drift starts.
3. **Defer `EX-9` and `VT-5`**, shipping step 6 unenforced. Leaves `REQ-449`'s
   forbidden list a validated field nothing reads until launch exists — the
   outcome `EX-9`'s own text says it exists to prevent. Not recommended.

Third stop of the slice, and the same shape as the last one: a criterion that
cannot be met without an edit outside the phase's ownership, caught at plan time
before a worker spent a session on it.

### Settled

**RULED 2026-08-08 — option 1. Nothing open here.** The slice owner took the
planner's recommendation; the ruling is written into the `PHASE-05` sheet as
`F-1/R`, which governs, and `S1` is discharged. `unsafe_code` goes `forbid` →
`deny` with exactly two `#[expect]` sites plus a `VA` check holding the count at
two, and `rustix` gains its `process` feature. Kept below for the auditor: the
question as put, and the two options not taken.

The workspace forbids `unsafe`; `EX-15` and `EX-16` require it (item 26). The
loop stopped here per `LOOP.md` § *Stop conditions* rather than spawning a
worker — the second time this slice has stopped on a real question and the
second time the stop was cheaper than the improvisation (`DEC-181`, then the
`[capsule]` figures, now this).

Options as the planner put them, cheapest first:

1. **`forbid` → `deny` at `Cargo.toml:200`**, plus exactly two
   `#[expect(unsafe_code, reason = …)]` sites in `bubblewrap.rs` and a `VA`
   inspection that the count stays at two; plus `"process"` on the existing
   `rustix` feature list (a feature, not a new crate — `linux-raw-sys` is
   already in `Cargo.lock`, and feature selections are not recorded there, so
   the lockfile does not move). Keeps `unsafe` denied by default everywhere and
   makes the two sites explicit and reviewable. **Planner's recommendation.**
2. **`EX-15` only, no `unsafe`:** enable `process` and set `RLIMIT_FSIZE` on
   the *parent* before the spawn — rlimits are inherited across `fork`/`exec` —
   restoring it after. Correct, but it caps the trusted side for the window and
   diverges from `EX-15`'s "on the child". Leaves `EX-16` unsatisfied.
3. **Defer both**, leaving invariant 12 and `sec-7` row 10 with no mechanism
   behind them. The option the design explicitly argues against.

Either way this edits two files `PHASE-05` does not own, one of them a
workspace-level safety posture — which is why it is not a planner's call.

## Traps — what already bit this slice

Tracked here rather than in `handover.md`, which is gitignored and does not
survive an `rm -rf` of state (`LOOP.md` § *Notes, sharded*: never put anything
load-bearing only there). The handover points at this section.

- **This clone mints no entities.** `DEC-`/`ISS-`/`RV-`/`REQ-` ids collide with
  the parent's. Findings go in the sheet and here as prose; mint at merge.
  Memories and observations are exempt (UUID/key-named) — record freely.
- **Run the gate yourself before flipping a phase.** `check`/`gate` build before
  validating, which is what gives the corpus check a fresh binary. A worker's
  report is a claim; the gate is the evidence.
- **Do not reap a slow worker.** Sheet mtime alone is not liveness — it goes
  quiet exactly when the work is slowest and dearest to lose. `LOOP.md`'s guard
  has four legs and a 90-minute floor; any one fresh means alive. Waiting is
  cheap, reaping is not.
- **Expect to need `record-delta`.** The auto-boundary spans every commit
  between phases, so any firing that commits driver or notes changes around a
  worker's commit inherits them. `PHASE-05` picked up seven that way and was
  tightened to `ab06b2fa7^..ab06b2fa7`. Read the boundary warning after every
  status flip.
- **`verify-vt` takes the slice id only** — `verify-vt 248`, not
  `verify-vt 248 PHASE-05`. It prints every phase; later ones FAIL on
  not-yet-existing files, which is expected.
- **A literal grep for `#[expect(…)]` under-counts** — rustfmt splits the
  attribute across lines. Normalise whitespace before counting.
- **A green mandated test is not evidence its rule is reachable** (item 23), and
  **a mutation that reds nothing may mean the fixture cannot discriminate**, not
  that the rule is untested (item 32). Only the battery tells them apart.
- **Lint, all `deny`:** `as_conversions` + `cast_possible_truncation`,
  `integer_division`, `indexing_slicing`, `unwrap_used` / `expect_used` /
  `panic`, `allow_attributes` (`#[expect(…, reason)]` only), `unreachable_pub`
  and `module_name_repetitions` (hence `pub(crate)`), `disallowed_types` bans
  `HashMap`/`HashSet`. `unsafe_code = "deny"` with a **hard budget of two**
  `#[expect]` sites, both spent in `bubblewrap.rs`; a third is a finding and a
  stop.
- **`unfulfilled_lint_expectations` is denied** via `-D warnings`. A staged
  `dead_code` header whose consumer has landed is a hard **error**, not a
  warning — `PHASE-06`'s named debt, measured at five files.
- **`cargo clippy --tests` / `--all-targets` is a trap** — enables `unwrap_used`
  in test code, reds ~100 pre-existing errors. Use `doctrine check gate`.
- **Path-limit every commit.** `git status --porcelain` first,
  `git commit <paths> -F -`, never pathless, never `git add -A`. A live worker's
  in-flight edits sit in the shared index — this already nearly swept a
  half-finished phase into a doc commit.
- **MSRV 1.85**, and `clippy::incompatible_msrv` is in the denied `all` group —
  post-1.85 std APIs are unavailable (`std::io::pipe`, 1.87, item 39).
- Dev binary `./target/debug/doctrine`, never `~/.cargo/bin`.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-08 · **`PHASE-01` and `PHASE-02` executed and green
in-tree on `edge`.** `PHASE-01` `74c398acb`+`6b58d48c8`, flipped `completed`
on the user's acceptance of the adaptations (`DEC-181` superseded — owed item
1); its delta range tightened to `74c398acb^..6b58d48c8` because the automatic
boundary had swept in four `SL-250` doc commits and one `IMP-412` backlog
commit. `PHASE-02` `10c57a30b`…`a11b158e0`, four commits, `doctrine check gate`
exit 0, lib target 210 tests (53 in `interpretation`, up from a 157 baseline).
Five further items owed to reconciliation (6–10), one backlog item minted
(`ISS-326`). **`PHASE-03` and `PHASE-04` executed in the clone** on branch
`sl-248`, driven by the `LOOP.md` loop: orchestrator routes, an Opus planner
writes the sheet, an Opus worker executes it. `PHASE-03` `2f3a6db78`,
`a79408d37`, `2a22084da` — gate exit 0, 33 tests, `verify-vt` 8/8 PASS.
`PHASE-04` `5f6eec999`, `4535efde8` — gate exit 0, **63** `doctrine-control`
tests, `verify-vt` 6/6 PASS. Neither phase's auto-recorded boundary needed
tightening: each spanned exactly its own commits, because the clone has no
concurrent agents. Fifteen further items owed to reconciliation (11-25).
Next: `/phase-plan` `PHASE-05` — **its planner must prescribe
write-all-then-mutate, not per-task red/green** (item 24).
Design run locked at revision 93 · 6b5036c38

### Produced

- Stage 2 complete — `PHASE-01` and `PHASE-03`…`PHASE-10` criteria expanded
  (`41f9abdc2`…`721f1fa77`, one commit per phase). Ten phases carry 31 `EN`,
  160 `EX`, 56 `VT`, 35 `VA`, 1 `VH`.
- `PHASE-07` `EX-14` amended mid-stage — `Property`'s variants arrive with their
  rows, not all at `PHASE-07`; `PropertyRemoval` is the enum that must be
  complete early, for a different reason.
- `DEC-180` § *Where it lands* amended — the cost arrives at `PHASE-08`, not
  `PHASE-10`; the decision itself does not move.
- **Stage 3 complete — the integration pass.** All four checks run over the
  whole plan (plan.md § *What the four checks found*), and every
  `# NOTE for the integration pass` discharged into either a criterion, a
  ruling, or plan.md prose; what survives in `plan.toml` under `# Stage-3
  record` is the cross-phase fact an executing agent needs.
  - Check 1 (`EN` discharge) clean; `PHASE-09` `EN-2` tightened to cite
    `PHASE-07`'s `EX` ids alongside its `VT` ids.
  - Check 2 (`VT` duplication) — the `PHASE-01` `VT-1` / `PHASE-02` `VT-10`
    export-set overlap ruled **permitted** (one test at two export-set sizes,
    and `PHASE-02`'s third keyword makes the mandates non-interchangeable).
  - Check 3 (`sec-8`'s evidence table) — one gap, closed. `transaction.rs`
    carried no mandate: its only title sat in `PHASE-06` `VT-3` under
    `test_file = provision.rs`. Split out as **`PHASE-06` `VT-6`** (the one new
    criterion id this stage minted). Every title in all seven of the design's
    `Verification alignment` sections was matched to a claiming mandate.
  - Check 4 (section claiming) — the provenance table's out-of-scope sentence
    widened from `sec-1` + `sec-9`'s risks/residuals to name all four groups of
    unclaimed narrative; `sec-7`'s executed alignment blocks named on the
    `PHASE-08`/`09`/`10` rows that own them.
- plan.md corrections landed: `src/lib.rs` and `tests/architecture_layering.rs`
  moved to the shared append-only table (`01, 02`); root `src/main.rs` added to
  `PHASE-01`'s exclusive row; the flag day re-attributed to `PHASE-08`.
- § *Two corrections owed at execution* → § *Corrections owed*, extended to five
  — items 3–5 are stage-3 rulings on design text, owed to the reconciliation
  brief only.
- No code touched, so no `doctrine check gate` applies; `doctrine validate`
  clean and `verify-vt` reports **0 `UNCHECKABLE`**. All `.doctrine` changes
  committed path-limited — another agent's `SL-249`/`SL-250` and
  `.doctrine/rfc/027/` left untouched.
- **The `/plan` critical pass (steps 7–9) is run** (`18121962d`; plan.md
  § *What the critical pass found*). Five findings, four landed as plan edits:
  `PHASE-01` `EX-4` amended (`today` must go `pub`; `ISS-323`'s root cause in a
  second instance), `PHASE-07` `EX-18` appended (sixteen `VT` mandates pin one
  `conformance.rs`; a split would strand them behind `ISS-271`-shaped noise),
  `PHASE-10` `VA-1` amended (falsifying measurement, no negative path),
  `PHASE-06` `EX-12` corrected to `renameat_with`. Selectors tuned:
  `+src/clock.rs`, `-tests/**`.
- Checked and found sound, no change: `sec-9` obligation coverage (every
  uncovered item a deliberate deferral), measured-versus-reasoned delta ordering
  (no delta depended on before the phase that measures it), phase sizing
  (`PHASE-02` largest, split point already pre-identified).
- **`/phase-plan` `PHASE-01` run** (`66bfa0bef`, `6b5036c38`). Runtime sheet
  filled: eight tasks, six `VT` mapped, five `VA` flagged as executed-not-reasoned,
  all four `EN` re-checked against the tree. Two findings and one sub-decision
  carried in the sheet (`F-1`, `F-2`, `D1`).
- **`DEC-181`** — `EX-4`'s visibility promotions do **not** compile alone. Ruled
  by the user; `plan.toml` `EX-3`/`EX-4` amended in place (text only, ids
  untouched). See Open for what this leaves owed.
- § *Execution posture* added above — `PHASE-01`+`PHASE-02` in-tree on `edge`,
  clone from `PHASE-03`, and the clone-mints-no-entities rule.
- **`/phase-plan` `PHASE-02` run.** Nine tasks, ten `VT` mapped, one `VA`.
  Three decisions taken in the sheet: `D1` (no `proptest` — an exhaustive
  generator over a small alphabet, emitting documents so the property covers
  `parse` too), `D2` (no speculative accessors; `sec-3` step 6's reader arrives
  with its consumer in `PHASE-06`), `D3` (pre-authorised `#[expect]` for
  `module_name_repetitions`, which duly fired).
- **`PHASE-02` executed** — `src/interpretation.rs`, 53 tests, four commits.
  `ISS-326` minted from `VA-1`'s measurement. Five owed items added above
  (6–10); `plan.toml` was not edited.

### Learned

- `mem.pattern.doctrine.assign-vt-by-code-owner-not-provenance-block` — a design's
  `Verification alignment` groups titles by section, not by phase; five titles
  moved phase this session.
- `mem.fact.bubblewrap.unshare-user-is-a-no-op-unprivileged`.
- `EVD-014` — the measured arms carry table A rows 13 and 14.
- `ISS-320` — no verb emits the `adopt_authored` section map; validate the
  recomputation against the unedited document before relying on it.
- `ISS-271` / `ISS-226` — at plan time `verify-vt` FAILs a `test_file` that is
  the phase's own output and mis-words `UNATTRIBUTABLE`; expected tooling
  behaviour. The signal that *is* trustworthy at plan time is the
  `UNCHECKABLE` count.

### Open

- **The design reopen is DONE.** `C-1` is settled and `PHASE-06` is unblocked.
  The critical pass had found that `fetch_refspec` is `pub(crate)`
  (`src/git.rs:2718`) and absent from the export set, while `sec-3` said the
  per-base export build rides it and `sec-6` said nothing else becomes `pub` —
  so `PHASE-06` `EN-3` read as met while `EX-11` could not compile. **Ruled by
  the user: wrap `git` locally in `doctrine-control`**, with a comment at the
  seam naming the alternative. The export set is unchanged, so `PHASE-01`,
  `PHASE-02` and the two `VT` mandates are untouched — the exposure noted here
  before the ruling did not materialise. `sec-3` § *Why the wrapper is local*
  and `PHASE-06` `EN-3`/`EX-11` carry it.
- **Seven further corrections landed in the same reopen**, so they are owed to
  nobody: `sec-6`'s `today` omission in both prose and the `EXPORTED` constant
  (`ISS-323`), the `renameat2` → `renameat_with` symbol, the
  `both_declared_`/`both_readable_` spelling, `just check`'s six legs, a stale
  `CredentialsConfined` in `sec-9` residual 2, and — found during the reopen
  rather than handed to it — `sec-4`'s claim that `closure-resolver` is the only
  trusted-side external command, which was false in four places once the export
  build drives `git`. plan.md § *Corrections owed* records what happened to each.
- The **informal `codex` sanity pass** over the reopen diff was run — the user's
  call, explicitly *not* an `RV` round, and no ledger was opened. It found four
  real classes of residue (stale present-tense narrative, a stale count ruling,
  an off-by-one `SPEC-030` citation, and a wrong line count for
  `fetch_refspec`), all now fixed.
- `/phase-plan` for `PHASE-01` is the step after that, not instead of it.
- `ISS-323` — **no longer owed at reconcile.** It and the three stage-3
  corrections that joined it were all applied in the design at the reopen, so
  plan.md § *Corrections owed* items 1, 2, 3 and 5 are closed rather than
  carried. Item 4 — one test name shared by two files — stands as a deliberate
  ruling. One of the four turned out to be an error in the *plan* rather than
  the design: `sec-9` residual 2's `PropertyRemoval` count of nine was correct
  all along (`SharedRoot` is a `Delta` variant), and it is `PHASE-07` `EX-4`'s
  "ten variants" over a list of nine that was wrong. **Nine variants, ten
  removals** — `ResourceBound(Bound)` carries two.
- `DEC-180` — settles the local-host case only; the CI ruling stays owed by
  whichever slice first runs this suite in CI.
- `QUE-208` — capsule-side entity id allocation; parked, does not block.
- `ISS-319` — separable defect, fixable independently of `QUE-208`.
- `SL-248` `OQ-1` (five-slice decomposition, provisional), `OQ-3` (three
  cross-cutting requirements), `OQ-4` (what replaces `review/*` and `phase/*`).
- `IMP-397` / `QUE-204` — egress allowlist and non-Git build inputs, out of
  scope, adjacent to `REQ-459`'s network row.
- `REQ-448`, `REQ-450`, `REQ-460` close in no single slice; this slice's share is
  `REQ-448`'s denial half and `REQ-450` criterion 1.
- Corrections owed at reconcile and follow-ups at close are carried in
  `design.md` `sec-9`, plus `ISS-323`.
- **Two NEW corrections owed to the reconciliation brief** (`DEC-181`,
  `/phase-plan` `PHASE-01`). The locked design was **not** reopened for either.
  1. `sec-6`'s "The exported items change visibility… **Nothing else does**" and
     invariant 2's "four visibility promotions, one module relocation behind
     re-exports, and one test parameter" are **incomplete**: the bin target also
     needs a `pub use` shim, because `main.rs` keeps its own module tree and
     `unreachable_pub = "deny"` rejects a `pub` in a private module there.
     `#[expect]` fails the lib target; `#[allow]` fails `clippy::allow_attributes`.
     All four combinations measured.
  2. The **call-site count is wrong in five places** — `sec-6`, `sec-8`, `EN-3`,
     `EX-3`, `VA-4` all carry "35 call sites across 33 files". Measured: **50
     occurrences across 21 files**. Load-bearing claim unaffected (the relocation
     is behind re-exports, so no call site is edited); the untouched set is the
     20 files listed in the `PHASE-01` sheet under `VA-4`.
- **`RV-350` RESOLVED — deleted, and the design run repointed at `RV-346`.**
  It was a zero-finding stub with an unfilled template brief, untracked in git
  while 973 other review files were committed, and the design run cited it as its
  review pass. Deleted at the slice owner's instruction 2026-08-08.
  - **The substance was never in the entity.** The run's `[review.pass]` carries
    its own `covered` fingerprint map; the RV entity was an empty shell
    throughout. So the run asserted a pass that the ledger showed no evidence of
    having run.
  - Deleting it broke `doctrine design show 248` outright — the run held a hard
    reference in two structural fields. **Repointed to `RV-346`** (`design`,
    `done`, `reviews SL-248`, 38 findings, raiser codex / responder claude — the
    pass that actually ran): `[review.pass] review` and
    `[acts.act.disposition] pass`. The run reads again; the acceptance act's
    `digest` is not verified over that field.
  - The reader now reports **`review_pass STALE — it no longer covers current
    content`**, which is the correct verdict and was invisible while the
    reference dangled: the coverage map pins pre-reopen fingerprints and the
    reopen changed six sections after `RV-346`'s rounds.
  - **Three prose fields still name `RV-350`** — deliberately left. One is
    `[acts.act.acceptance] basis` under `authority = "user"`, the slice owner's
    recorded waiver rationale; rewriting it would launder the record. An auditor
    reading the run will see prose citing `RV-350` and a pointer to `RV-346`;
    this note is the explanation.
  - Run state is gitignored runtime state, so none of this is committed. Backups
    of the deleted entity and the pre-edit run state are session-local only and
    will not survive.
