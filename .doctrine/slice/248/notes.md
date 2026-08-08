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
(`ISS-326`). Next: `/phase-plan` `PHASE-03`, the first phase in the clone.
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
