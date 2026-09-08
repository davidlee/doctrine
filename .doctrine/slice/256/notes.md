# Notes SL-256: Recording an act emits a change row

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage
<!-- `explore.triage`, run rev 11; refreshed at rev 25. An index into where each element lives, not a
     second copy of it — the scope doc is the authority for all five columns. -->

| element | where it lives | state |
|---|---|---|
| open questions | `slice-256.md` § Open Questions — `OQ-1`…`OQ-3` | all settled, each annotated with the DEC that settled it |
| in-run questions | design run `dr-01a0088b` — `inq-1`…`inq-6` | 6/6 resolved; `inq-6` raised mid-drafting |
| risks | `slice-256.md` § Risks & Assumptions — `R1`…`R6` | `R3` retired by research; `R6` added from the `explore.memory` retrieve |
| assumptions | same section — `A1` | discharged by `DEC-240` |
| shaping decisions | `DEC-237` (event shape + terms), `DEC-238` (emit seam), `DEC-239` (roster split), `DEC-240` (durable statement), `DEC-241` (review-disposing arm emits two rows) | all `accepted`, all bound to the node that asked |
| constraining governance | `research.md` § Thread 1 — binding: `STD-001`, `REQ-437`, `REQ-436`, `ADR-001`; checked-not-applicable: `STD-002`, `STD-003`, `POL-001`, `POL-002`, `ADR-019`, and an ADR sweep | confirmed by the canon pass; `STD-003` subsequently ruled on rather than merely noted |

The one live tension left for drafting is not a question but a sequencing fact:
`DEC-239` widened scope to retire a member whose legacy rows appear in `SL-251`'s
own design-run snapshot. Read-path only by design — the roster split exists to
keep exactly those rows parsing — but it wants re-checking at `SL-251`'s
integration rather than assuming.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-09 · PHASE-03 completed — all phases done, slice ready for `/audit` (run `dr-01a0088b`, rev 57, stage `locked`), head `bf22f8487`

### Produced

- **PHASE-01 done** — `ActRecorded`, the `ActRecord` sum, the `admit_and_record`
  seam, `admit_against` deleted, three call sites onto one route
  (`69ce71c1b`..`36b0e1326`). `EX-1`…`EX-6`, `VT-1`…`VT-4`, `VA-1` all
  discharged; `doctrine check gate` green on the landed tree.
- **PHASE-02 done** (`48a04f6a6`, mandate settlement `7c8095664`) — `ALL` split
  into `READABLE` (23) / `EMITTABLE` (22), the subset relation proved by a
  `const fn is_subset` compile-time assert, the `AcceptanceAttested` push gone
  from `apply`'s acceptance arm, both roster enumerations re-pointed and the
  ladder narration corrected. `EX-1`…`EX-6`, `VT-1`…`VT-4`, `VA-1` discharged;
  `doctrine check gate` exit 0 after the last code change; `verify-vt` `✓ PASS`
  on all four rows read after the `completed` flip. `--bin doctrine` 4410/0/4
  unchanged from baseline (`tests.rs` / `fixture.rs` needed no edit, as `sec-4`
  claimed); `--test e2e_design_state` 224/0/1 (221 + the three new checks).
  Conformance: the four touched files all `conformant`; `undeclared` holds only
  PHASE-01's known set, `undelivered` holds PHASE-03's selectors plus
  `render/mod.rs` and `render/change_row.rs` (see Open).
- **PHASE-03 done** (`50c41b25c` code, `95e0b52aa` coverage, `bf22f8487`
  harvest) — `#[serde(rename_all)]` and the variant-level `evidence_invalidated`
  alias replaced by `#[serde(try_from = "String", into = "String")]` over
  `as_str`, so the wire token has one source; `Refusal::UnknownChangeEvent` and
  the named `LEGACY_ACT_INVALIDATED` literal added; `AcceptanceAttested` renamed
  `LegacyAcceptanceAttested` with token, rendered name and payload shape all
  unmoved; the `ActInvalidated` doc's attribution corrected to `SL-244` alone.
  `EX-1`…`EX-7`, `VT-1`…`VT-3`, `VA-1`, `VA-2` discharged; `doctrine check gate`
  exit 0; `verify-vt` `✓ PASS` on all eleven rows across the three phases, read
  after the `completed` flip. `--bin doctrine` 4411, `--test e2e_design_state`
  226 — `+1`/`+2` rather than `+1`/`+1`, because the e2e binary `#[path]`-includes
  the leaf's `#[cfg(test)]` modules ([[mem_019fcc9b571e7fe18dfd8e7fbda6e802]]).
  Conformance: six of eight `design-target` selectors conformant.
- **`REQ-478`'s coverage cell is bound and verified** — `sec-4`'s recipe with the
  `=` fix (see Open), `planned→verified` under `coverage verify 256`, which is
  itself the evidence the check ran.
- minted at PHASE-03: `IMP-445` — the new refusal drops the accepted-token list
  serde's derive printed; `originates_from` `SL-256`, `related` `ISS-315`.
- No new backlog items minted at PHASE-02; nothing pending but `flake.lock`,
  which is not this slice's.
- minted: `ISS-448` — a toolchain bump red-lights the gate on files a phase
  never touched, `related` to `IMP-273`; `ISS-449` — `slice verify-vt` prints
  "keyword present" on `UNATTRIBUTABLE` rows it never checked, `governed_by`
  `STD-003`.
- Selectors widened by two, `scope-relevant`: `src/estimate/display.rs`,
  `src/map_server/state.rs` — the `ISS-448` clearance, taken on explicit human
  instruction after `/consult`. Both still read `undeclared` at conformance;
  that is correct and intended (see Learned).
- Two `friction` observations (`.doctrine/observations/records/{d3,18}/…`).
- `.doctrine` changes committed alongside their code, promptly; nothing pending
  but `flake.lock`, which is not this slice's.

- `DEC-237`…`DEC-241` — minted through the run's own checkpoint dispositions
  (`cp-1`…`cp-6` over `inq-1`…`inq-6`), so each is bound to the question it
  answers. `DEC-238` carries an appended correction (`RV-360` `F-1`).
- `REQ-478` (`FR-009` under SPEC-029) — authored `pending`; coverage cell
  deferred until the e2e checks exist.
- `ISS-367`, `IMP-437` — both sequenced `after SL-256`.
- `RV-360` — external adversarial pass over all four sections; 16 findings swept
  the roster split, emit seam, suite fence, coverage binding, and compile-time
  subset proof. Verification and conclusion are uncommitted for responder landing;
  no source code changed and no full gate was run. `RV-359` remains the empty pass.
- `DEC-239` carries an appended correction: the retirement's pinning mechanism
  moves from `#[serde(rename)]` to `#[serde(try_from/into)]` single-sourced
  through `as_str`. Substance unchanged; `STD-001` satisfied rather than excepted.
- `src/design_run/refusal.rs` added as an eighth selector — `ChangeEvent`'s
  `TryFrom` needs one new variant. Fence widening declared, not discovered.
- `design.md` `sec-1`…`sec-4` drafted, revised and materialised (`9fe802b8f`,
  `1e495e88c`). Scope and selectors reconciled in the same commit.
- `research/research.md` + `raw/` — two-thread round, verification pass appended.
- `proto-prompt.md` + fork `proto/SL-256` — the prototype probe round (`93e05f49f`).
  One adopted finding at rev 43/44; see **Prototype probe** below.
- One `friction` observation — `adopt_authored` vs `declare`+`body`, and
  undiscoverable section fingerprints (`.doctrine/observations/records/8a/…`).

### Learned

- `mem_019fbf791c9d71d0815d6e1fc2c9fb9f` — **extended at PHASE-01**, not added:
  `memory record` flagged the near-duplicate on the way in, so the new memory
  was deleted and its one novel section folded in. What it now also carries is
  the fork a widening creates — leave it `undeclared` and explain it, make it a
  real `design-target`, or (never) re-intent to silence the cell — plus the
  `ISS-449` caveat, marked for deletion when that closes.
- `mem.fact.coverage.vt-needs-a-check-field` — recorded at PHASE-01,
  **corrected at PHASE-03**: the recipe it carried does not parse. Any
  `--command` value beginning with `-` needs the `=` form
  (`--command=--test`), or clap reads it as an unexpected argument. Corrected in
  the memory rather than only in the slice, because a memory is precisely what
  the next agent copy-pastes.
- `mem_019f89125fb275a2895bf58b5e29ed95` — **extended at PHASE-03**, not added:
  `slice conformance` folds the boundary registry, which only holds *completed*
  phases, so mid-phase it reports the previous phase's picture with no sign the
  range is stale. Same root as the `UNATTRIBUTABLE` rule that memory already
  carries, and it bites earlier because `EX` criteria ask for a clean read
  *before* the flip. In-phase read: `--against <code_start_oid>..HEAD`.
- **The single-sourcing rests on an agreement the design never states as
  checked** — all 23 `as_str` tokens already equalled `snake_case(identifier)`,
  so dropping `rename_all` was wire-neutral. Had one disagreed, the swap would
  have silently moved that member's stored token. Verified arm by arm at
  `/phase-plan` and pinned member-by-member by `VT-1`.
- **Verifying a read-path change against the live corpus beats verifying it
  against fixtures**, and it is one command. `design show` over all seven runs
  carrying a retired token found `SL-244` unreadable — which the fixture suite
  could not have told us, and which turned out to be `ISS-315` rather than a
  regression (see Open).
- `mem.pattern.doctrine.vt-keywords-name-a-literal-the-file-contains` — recorded
  at PHASE-02 out of `F-2`: a `verify-vt` keyword is a raw-byte substring check,
  so a mandate naming a value the suite composes cannot pass; name the composed
  source, and check satisfiability at `/phase-plan`.
- `mem.pattern.lint.indexing-slicing-ban` — **extended**, not added: in a
  `const fn` neither `v[i]` nor `.get(…).unwrap_or_default()` is available, so a
  compile-time walk recurses on slice patterns (`widest`, `is_subset`);
  `PartialEq` is not `const` either, and comparing discriminants instead trips
  `as_conversions`.
- Candidates for `/record-memory` at close, not yet written: `DEC-238`'s
  key-vs-content rule (a derived row can only report changes in the *key* of the
  set it differences); `pi-scout` line anchors drift onto the doc comment or
  attribute above an item while content stays accurate (`research.md`
  § Verification pass errata); adding an inquiry node after the exploring gate
  costs two acts to restore, one of them the user's; an argument from absence
  needs enumeration, not a generalisation (`RV-360` `F-7`).
- **HIGH — grep line numbers from the Bash tool are intermittently wrong.** The
  same `grep -n` returned `1081/1086/1091` early in a session and `1094/1099/1104`
  later for the same three call expressions in `src/design_run/tests.rs`; the Read
  tool agrees with the second. Multi-line `sed` ranges come back with lines elided,
  so content cannot be aligned to a range's numbers either. This corrupted a whole
  verification pass and was caught only because an external reviewer's anchors
  disagreed. Extends `mem.fact.rtk.output-filter-rewrites-identifiers` from
  identifiers to line numbers, and adds that it is **non-deterministic**. Ground
  truth is the Read tool or `awk '{print NR}'` cross-checked against it.
- An acceptance criterion can be a **relieving** clause rather than a positive
  proof obligation. `REQ-478`'s third criterion exempts retired vocabulary from
  the emission obligation; the design read it as demanding proof that emission
  cannot occur, imported a burden it could not discharge, and a reviewer correctly
  found the shortfall. Read the requirement's own rationale before restating a
  criterion in a design's words (`RV-360` `F-2`).
- `#[serde(try_from = "String", into = "String")]` is this repo's idiom for
  single-sourcing a token enum's wire spelling through one `as_str`
  (`ids.rs:131`, `attestation.rs:935`, `change_log.rs:387`), and `Refusal`'s
  `Display` doc names serde's `try_from` as the boundary it exists to cross. A
  serde attribute cannot take a `const`; this is the way round that (`RV-360`
  `F-4`).

### Prototype probe (2026-08-16, fork `proto/SL-256`)

The design's type model was run through a compiler instead of through more prose:
a disposable implementation by a third agent (deepseek-v4-pro, confined via
`scripts/spawn-confined.sh pi`) in `.worktrees/proto-SL-256`, uncommitted, 359
insertions across exactly the four `src/` files `sec-4`'s code-impact table
names. The mandate is `proto-prompt.md`, authored blind — the run needed no
steering. Report: `PROTO-FINDINGS.md` in that fork.

Unlike `SL-238`'s equivalent, the fork carried the **current** design: rev 42 was
committed at `c0099efa0` and the tree was clean, so nobody re-entering it reads a
stale design beside current code.

**It confirmed all three load-bearing claims, which is evidence and not merely an
absence of findings.** `cargo check --bin doctrine` clean and the bin's own unit
suite green at 4363 tests, which corroborates `sec-4`'s claim that `tests.rs` and
`fixture.rs` need no edit. A throwaway test drove all four emission paths and
confirmed the contractual `[ActRecorded, ReviewDisposed]` order — the trap
`RV-360` `F-10` named, where construction order runs opposite to vector order.
The compiler found exactly two `ChangeEvent::ALL` sites in `e2e_design_state.rs`
(`:1090`, `:1220`), matching "the two roster enumerations re-pointed" exactly.

**One finding, verified here by reproduction rather than taken on report.**
`ChangeEvent::ALL`'s only `src/` consumer is the compile-time widest-name assert
(`change_log.rs:46`). `READABLE` inherits it and gains `TryFrom`; `EMITTABLE`
inherits nothing, because both its consumers are in `e2e_design_state.rs`, a
separate compilation unit. The module dead-code gate is `not(test)`-scoped
(`mod.rs:68`) and the crate denies `unused` (`Cargo.toml:224`), so `cargo check`
passes and `cargo test --bin doctrine` fails. Gating the prototype's own
throwaway test out reproduced it: `error: associated constant EMITTABLE is never
used`.

**Disposition — first adopted at rev 43/44, superseded at rev 45/49.** The
observation went in as a fourth note under `sec-4`'s code-impact table, stating
the constraint and leaving the exemption's form to the implementor. The
adversarial pass below found that ruling's stated constraint wrong and its repair
space mis-bounded; the note is deleted, because the repair that landed removes
the finding's subject rather than exempting it.

**Class swept, and it is empty.** The class is *a constant whose only consumers
live in the integration-test compilation unit*. Nothing else this slice
introduces qualifies — `ActRecorded`, `LegacyAcceptanceAttested` and
`UnknownChangeEvent` all have production readers — and the class cannot exist
latently elsewhere, since the current corpus compiles.

**What the round establishes about method.** Yield was one lint-layer consequence
against `SL-238`'s two type-level defects. That is not the method
underperforming: this design had already absorbed 14 `RV-360` findings and is
unusually explicit about its signatures, which is precisely what makes it
compile-clean. Against a design this hardened, confirming three claims at runtime
and returning one consequence nobody had reasoned to is the shape of success. The
finding was also *not* findable by re-reading — it is a property of the crate's
lint configuration meeting a compilation-unit boundary, three files apart.

### Adversarial pass on the rev 43/44 repairs (2026-08-16)

Held to the standard `codex` applies on `RV-360`: verify against source, and
treat a plausible rationale as the thing to attack. Two findings raised and
answered on `RV-360` (`F-15`, `F-16`); the ledger's `await` returned to `raiser`,
so `codex`'s owed verification turn on `F-1`–`F-14` is undisturbed.

**The adopted claim itself held, and was re-verified by reproduction rather than
by re-reading.** Every link checked against source (`change_log.rs:46`;
`mod.rs:68-74`; `Cargo.toml:97`/`:224`; `e2e_design_state.rs:6-8`/`:40-46`), then
the failure reproduced in a scratch worktree at `c546a9ae0` by adding a
consumer-less `pub(crate)` associated const: `cargo check --bin doctrine`
finished, `cargo test --bin doctrine --no-run` gave
`error: associated constant … is never used`, `-D dead-code implied by -D unused`.
`bounds.rs:49` is an intra-doc link, and a doc link is **not** a dead-code read,
so it could never have stood in for one.

**`F-15` — the note named a constraint that does not bind.** It warned that an
`expect` exemption would forbid a *future* bin-side test. The binding fact is
that `cfg(test)` is true in **both** compilation units — the bin's test build and
the integration-test crate that `#[path]`-includes the module — so
`cfg_attr(test, expect(dead_code, …))` is unfulfilled in the e2e unit and stops
*that* build today. Reproduced:
`error: this lint expectation is unfulfilled`, `-D unfulfilled-lint-expectations`.
`cfg_attr(test, allow(dead_code, …))` does compile, but only because `just gate`
runs `cargo clippy` without `--all-targets` (`justfile:67`), so
`clippy::allow_attributes = "deny"` (`Cargo.toml:252`) never sees it.

**`F-16` — the repair space was mis-bounded, and that is what hid the fix.**
`notes.md` had recorded any production consumer as reopening `RV-360` `F-2`.
`F-2` and `DEC-239` decline type enforcement at the *construction seam* — a
constraint on writers, in the converse direction — so a consumer that constrains
no writer cannot reopen them. `sec-2` already named the candidate without
claiming it. Promoting `EMITTABLE ⊆ READABLE` from a runtime assertion to a
compile-time `is_subset` proof gives `EMITTABLE` a `src/` consumer, so
`dead_code` never fires and `sec-4`'s note has no subject; it also upgrades a
planned test to a proof. Probed green on `cargo check`,
`cargo test --bin doctrine`, `cargo test --test e2e_design_state` and
`cargo clippy`. Two shorter spellings are closed off by the workspace gate and
are now named in `sec-4` so nobody re-derives them: a discriminant compare trips
`clippy::as_conversions`; deriving `READABLE` from `EMITTABLE` by a const-block
copy trips `clippy::indexing_slicing`.

**Siting, the fourth target, dissolved.** With the note deleted there is nothing
to re-site; the premise it rested on now lives in `sec-2`'s caption, which is the
section that owns the roster split.

**On method — the cost of reasoning where compiling was available.** The rev
43/44 ruling was reached by recalling
`mem.pattern.lint.dead-code-derives-count-as-reads`, whose own conclusion is
*"Do not reason about it; compile."* One build during adoption would have covered
what five scratch-worktree builds established here. Captured as a friction
observation.

**`/record-memory` candidates at close.** (a) `cfg(test)` holds in *both* the bin
test build and any integration-test crate that `#[path]`-includes a module, so no
`cfg(test)`-keyed `expect` can be correct for an item live in only one of them.
(b) `just gate` runs clippy without `--all-targets`, so clippy restriction lints
— `allow_attributes` among them — do not see `tests/` or the bin's test cfg.

### Further review passes (written after `RV-360` concluded)

**A further *design* pass is not needed; the residual risk has moved downstream to
`/plan`.** The design has now taken four independent attacks — `RV-359` (empty),
`RV-360` (16 findings, external, concluded), a prototype that ran the type model
through a compiler, and an internal adversarial pass over the rev 43/44 repairs
whose two findings `codex` re-derived rather than accepted. The last two rounds
each returned exactly one class of defect and the last returned none at all on
re-derivation, which is the shape of a surface that has stopped yielding to
reading.

If one were run anyway, the two things it should probe are:

- **`sec-3`, the emit seam.** It drew findings in `RV-360` and has been
  byte-identical since rev 44, so it is the section with the most attention paid
  and the least recently. Its non-bypassability is a convention rather than a
  guarantee, which the design states plainly — but `IMP-437` carries that and is
  deliberately out of scope, so a pass would be re-confirming a known boundary.
- **Whether `REQ-478`'s three criteria survive phase decomposition.** Every
  criterion is discharged by tests nobody has written yet, and the coverage cell
  is deliberately deferred until they exist. A design pass cannot test that; the
  phase plan can, and `/phase-plan` is where an under-specified section shows.

### Open

- **`design.md` carries a token erratum for `/reconcile`** — subject and term
  spelled `cpa-design_accepted` / `act=design_accepted` at `:593`, `:626`
  (`sec-3`'s sequence diagram) and `:708` (`sec-4` check 4), where act tokens
  are kebab (`ActKind::DesignAccepted => "design-accepted"`,
  `attestation.rs:116`). `plan.toml` `VT-1` was corrected in place at PHASE-02
  (`7c8095664`, and see `mem.pattern.doctrine.vt-keywords-name-a-literal-the-file-contains`);
  the design half is not an implementor's edit to make, the run being `locked`.
- **Two declared selectors are `undelivered` and may be right to prune at
  close** — `src/design_run/render/mod.rs` and `render/change_row.rs`. Research
  Thread 2 found the render path data-driven off `as_str` plus generic term
  rendering, so neither PHASE-02 nor PHASE-03 has a reason to touch them. If
  PHASE-03 leaves them untouched, prune the selectors at reconcile rather than
  manufacture an edit to satisfy the fence.
- `IMP-437` — the emit seam is a convention, not a guarantee; needs its own
  scope because the fix reaches `fixture.rs` and `SL-251`'s `tests.rs`.
- `ISS-367` — `live_acts` blind to same-kind replacement. `sec-1`/`sec-3` state
  the boundary so the design promises no symmetry it does not deliver.
- **`SL-251` coordination: three sites, not two** — its `design.md` ¶ 422–428,
  its ledger row at 2289, and `payload_contract.rs:501`. Discharged at that
  slice's reconcile, not here.
- `QUE-219` — not this slice's to settle; `DEC-239` bears on it and the relation
  carries the descriptor.
- ~~Coverage cell for `REQ-478`~~ — **done at PHASE-03**, `planned→verified`.
- **A second `design.md` erratum for `/reconcile`** — `sec-4`'s coverage recipe
  (`:958-964`) does not parse: `--command --test` is refused by clap
  (`unexpected argument '--test' found`) and needs `--command=--test`. The cell
  was recorded with the `=` form and stores the argv the design intended, so
  this is spelling, not substance. Joins the `cpa-design_accepted` erratum above;
  the run is `locked`, so neither is an implementor's edit.
- **Prune the two render selectors at reconcile.** PHASE-03 left
  `src/design_run/render/mod.rs` and `render/change_row.rs` untouched, as
  Research Thread 2 predicted — the render path is data-driven off `as_str`. The
  conditional above is now discharged: prune, do not manufacture an edit.
- **`ISS-315` is still live and still reproducible**, found while verifying
  PHASE-03's read path against the real corpus. `doctrine design show SL-244`
  fails on `integrated_review_recorded` — a retired token with no `READABLE`
  member and no alias. **Not this slice's regression**: the pre-slice binary
  fails identically (positive control — that same binary parses `SL-251`). Out
  of scope here; the issue's line reference has moved `:4991` → `:4927`.
- **`memory validate` exits 1** with six findings (five stale anchors, one
  dangling link). None is against either memory this slice touched, and none is
  scoped to `src/design_run/**` — the scopes are `flake.nix`, `src/dispatch.rs`,
  `src/worktree/**`. Pre-existing corpus drift, flagged for `/audit` to rule on
  rather than absorbed here.
