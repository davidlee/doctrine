# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · **reconcile**, 6 of 6 landed, audit closed (RV-313 done);
`dispatch/230` at 8538a2658, `review/230` at 66d478cc2, **reviewed surface
`candidate/230/review-001` at 18fc99613** (base `main` c2e9191a2 post-SL-228 +
source `review/230`), coord worktree removed · 2f8b3fbdf

### Produced

- **PHASE-01** engine body seam — `BodyMode` + `write_body` in `src/entity.rs`
  (coord 56520c04b→29014846, boundary 74620ec5e).
- **PHASE-02** `record --body` — rides the `memory_scaffold` seam; collapsed
  `seed_by_key`'s duplicate index-1 substitution onto `Draft.body`
  (coord 9a0d7bf→a5a27a403, boundary e7e0d42f).
- **PHASE-03** `edit --body` + the two-tier write ordering — the first caller of
  `write_body`; retires PHASE-01's `expect(dead_code)` staging attributes
  (coord e7e0d42f→232dbd3f, boundary 4d8c1e49).
- **PHASE-04** attestation invalidation — `claim_snapshot` + an infallible
  `clear_verification`; closes § 2.1's observed defect (coord 4d8c1e49→62476d71,
  boundary 162aa662). **The 03/04 inseparable pair is closed** — no intermediate
  HEAD hazard remains, so integration is unblocked (still `/close`'s job, post-audit).
- **PHASE-05** MCP body fields — `body` on `memory_record`, `body`+`body_mode` on
  `memory_edit`, both pure argument mapping onto `run_edit`/`run_record`; the
  `body_mode`-without-`body` totality rule landed as ONE const + ONE bail site
  (coord aae133bb3→43190de93, boundary 3cf7c040).
- **PHASE-06** `validate` own-body staleness — a fourth, independent health check
  counting commits touching `<items>/<uid>/memory.md` since `verified_sha`
  (coord 3cf7c040→eb53c3770, boundary 8538a2658 after `prepare-review`).
- **ISS-248** — selector doctor's redundancy scan is class-blind.
- **ISS-252** — claude-arm funnel verify beat runs against a stale coord
  checkout; fails open as a false green.
- Selectors widened to declare the forced `RecordArgs` ripple (90cc6e67).
- Per-phase detail and PHASE-03's inherited facts: § Phase trail. Orchestrator
  operating facts: § Funnel mechanics. Both prose, both committed.
- `.doctrine/` changes committed promptly and **separately** from code throughout —
  code lands only on `dispatch/230` via the funnel; authored state only on `edge`.
- Gate: **superseded — `doctrine check gate` has now run against the slice code**,
  on `candidate/230/review-001` at audit: **4928 passed, 0 failed, 102 suites,
  zero compiler/clippy diagnostics** (the log's 19 `warning:` lines are doctrine's
  own runtime warning strings from tests exercising warning paths — none carries a
  `-->` source pointer). The earlier note that it had never run is no longer true.
- **I10 and I13 are mutation-verified, not merely asserted** (RV-313 audit). I10 —
  hoisting `write_body` ahead of `apply_edit` (the pre-RV-307-F-3 order) turns
  **T40 red** with `clobbered\n` on disk. I13 — widening D5's pathspec from
  `memory.md` to the item directory turns **T42 red *alone***, the other two
  validate tests staying green. "Red alone" is what separates a real falsifier from
  one that fires on any perturbation; both held. Caveat carried forward: a mutation
  that fails to compile has not been checked.
- **RV-313** — the reconciliation audit ledger. Done: 6 findings, all terminal, no
  blockers. Carries `## Brief` (surface reviewed + lines of attack), `## Synthesis`
  and `## Reconciliation Brief` (the handoff to `/reconcile`).
- **ISS-257** — memory staleness checks render an undeterminable stamp as clean
  (RV-313 F-2). `originates_from` SL-230, `related` RV-313.
- **F-3 fix-now, landed at `18fc99613`** on `candidate/230/review-001` — MCP
  `record` now forwards `body_mode` into `run_record` so its worded refusal fires
  on that surface too; `input_schema` still does not advertise it (EX-1 stands).
  **This commit is NOT on `review/230`.**
- **RV-307 F-25 / F-33 / F-36 / F-37 verified terminal** at audit (they were
  dispositioned `descoped-to-SL-232` but never closed, and the close-gate refused
  `audit → reconcile` while they sat open). Remediation stays SL-232's per DEC-027.

### Learned

- **Unrecorded memory candidates, still owed** (they predate this drive and
  survived the design-phase harvests): the tool-property-is-a-claim-needing-a-
  falsifier lesson; "run the query" (execute a design's stated query against the
  real corpus); mechanical-narrowing-damages-joints + `git show <commit>^:<path>`
  as the cheapest instrument. Bodies in git at 68bf7f21's notes.md; **not**
  restated here.
- § Funnel mechanics' three items are memory candidates once a second slice
  confirms them; ISS-252 carries the load-bearing one as actionable work.
- **Memory candidates from the audit** (RV-313; not yet recorded):
  - **Piping a gate/verifier through `tail` destroys the evidence twice** — the
    window discards the result lines, AND a pipeline's exit status is the last
    command's, so `... | tail` always reports success. Redirect to a file, echo
    `$?` separately. Cost here: one full re-run.
  - **Grepping `^warning|^error` over a doctrine gate log false-positives** on the
    product's own runtime warning strings. Real rustc/clippy diagnostics carry a
    `-->` source pointer; filter on that.
  - **A silent-drop `None` is invisible to every test that asserts on output.**
    `let Some(x) = f() && x > 0` renders "cannot determine" identically to "clean".
    The repo already has the right shape one module over
    (`src/coverage.rs:150-166`, `IsStale::Unknown`, `None => Unknown`).
  - **A superseding-candidate re-admit is legal without a fresh candidate**:
    `admit` requires the recorded merge be an *ancestor of* the admitted tip, not
    equal to it, so an audit repair committed on the candidate re-admits cleanly.
    `candidate status`'s DRIFT advisory ("supersede with a fresh candidate") reads
    stronger than the binary actually enforces.
- **Confirmed, generalisable, still owed as a memory:** a mutation that fails to
  compile has not been checked (PHASE-06's lesson, hit again at audit — a
  `body_mode: None` mutation is rejected by `-D dead-code`; the faithful pre-fix
  state was the only real check).

### Open

- **DEC-027** — the SL-230/SL-232 split boundary; status left deliberately as
  authored, do not settle unasked.
- **OQ-4** — when other kinds adopt `write_body`; for `knowledge` it is a new verb,
  not a new flag.
- **OQ-7** — `write_body` on the MCP surface for other kinds.
- **R4** unmitigated until SL-232 lands — accepted friction per DEC-027.
- **QUE-173**, **QUE-175**, **IMP-317**, **IMP-318**, **REV-034**, **F-14** — moved
  to SL-232; tracked there, not here.
- PHASE-03/04 inseparable, and no `sync --integrate` between them: § Execution
  constraints.
- Moved ids never reused. Next free: **I14, E16, T43** — `E15` was spent at
  reconcile on `clear_verification`'s tolerance (RV-313 F-4, design § 5.5).
- **RV-313 F-6 / SPEC-007 — open, and NOT settled by audit.** Does § "Git-anchored
  staleness"'s explicit-state guarantee bind `memory validate`'s health checks, or
  only the `find`/`retrieve` axis? The sentence does not determine its own scope.
  Reconcile's REV to answer; the brief recommends *clarify the scope* over
  adjudicating conformance, since the code outcome (ISS-257) is identical either way.
- **The MCP adapter rejects no unknown fields at all** — `deny_unknown_fields`
  appears zero times in `src/mcp_server/tools.rs`, so every tool silently swallows
  unknown keys. F-3 closed the one instance that mattered, not the class.
  **Unfiled by decision** — offer it, do not mint it unasked.
- **SL-232's inheritance of RV-307 F-25/F-33/F-36/F-37 is recorded in prose only**
  (DEC-027's table, the finding responses, this file). Those findings are now
  terminal, so nothing structurally gates SL-232 on them. A `needs` edge was
  offered and is **not yet accepted** — do not add it unasked.

### Next

1. **`/reconcile`** — consume RV-313's `## Reconciliation Brief`. Four per-slice
   direct edits (three in `design.md`: the stale D5 figures, the new `E15` edge
   case, the reach limit; plus this file) and one governance REV (SPEC-007 § F-6).
   The brief states "no REQ status changes identified" so it is not re-derived.
2. **`/close`** — and note the sourcing trap: the `close_target` candidate must
   **not** be sourced from `review/230` (`66d478cc2`), which does not carry the
   F-3 fix. Source from `candidate/230/review-001` (`18fc99613`). Expect authored
   divergence (`slice-230.toml` is `reconcile` on `edge`, `started` on the
   evidence refs) → primary-wins.
3. `/design` SL-232 from F-37, F-36, and F-14.

## Execution constraints (dispatch)

Settled 2026-07-26 when the user chose dispatch as the execution mode. These are
constraints on the *orchestrator*, not on any worker — a worker cannot see them
and none is expressible as a phase criterion. Read before `dispatch setup`.

**Serial, not parallel — and know what that buys.** Five of six phases target
`src/memory.rs` (02, 03, 04, 06, and 05's VT-2). The chain is 01→03→04→05 with
04→06. The only file-disjoint pair is PHASE-01 (`src/entity.rs`) ∥ PHASE-02, worth
one phase of wall-clock out of six against a fork, an import, and a conflict
surface on the file PHASE-02 owns. So dispatch is being chosen for **context
isolation and the one-commit-per-phase funnel, not throughput**. Do not let a
later reader re-derive a parallelism benefit that is not there. Note
`dispatch plan-next`'s own `⚠ do not assume file-disjointness` warning is
load-bearing here: `compute_next_phases` (`src/dispatch.rs:3973`) reads status
and plan order ONLY and never consults entrance criteria, so a parallel spawn
over `next` would happily violate PHASE-03's EN-1.

**1. Do not `sync --integrate` between PHASE-03 and PHASE-04.** The inseparable
pair (see § Open) is a *trunk* hazard, not a coordination-branch one: intermediate
coord HEADs never reach trunk, so the whole exposure is integration **timing**.
Integrate no earlier than after PHASE-04 concludes; one integration at the end is
simplest. This is the one place dispatch can actually publish the 03-only state
that § 1 calls strictly worse than today.

**2. PHASE-06 VA-1 runs on the coord/landing tree, NOT in the worker fork.** It
re-measures the live corpus and expects ~11 of 30 anchored memories (30/30 means
the pathspec regressed to the item directory — the registered falsifier). The
measurement is `rev-list verified_sha..HEAD -- memory.md` against real git
history, and a worker fork has a different HEAD and flat topology, so a worker
can false-confirm or false-alarm the single most important check in the slice.
Same reasoning applies more weakly to PHASE-01 VA-1 (ADR-001 layering) and
PHASE-04 VA-1 (read the composed condition): all three are read-and-judge checks,
natural orchestrator work at audit. Workers deliver the VT half.

**3. Watch ISS-028** — worker-marker confinement refuses CLI writes in a stamped
fork, breaking tests that shell the doctrine CLI. Checked, not assumed, at
readiness: `src/memory.rs`'s memory tests are in-process (`GitScratch` + direct
calls), so 02/03/04/06 look clear, and `tests/e2e_mcp_server.rs` spawns the built
binary against a tempdir rather than the fork. Not a known blocker — the thing to
check on first worker return.

**4. Promote edge before setup.** `git fetch . edge:main` then
`dispatch setup --slice 230`, or the worktree forks without the design, plan, and
readiness fixes. Documented footgun; it bites silently.

**Arm.** Not pre-chosen: `/dispatch` routes on a claude↔env-marker agreement, and
the funnel cadence is identical either way (claude arm self-commits via
`worker_commit`; pi arm has the orchestrator import the working-tree diff).

**Arm taken 2026-07-27: claude.** No `[dispatch]` table in `doctrine.toml`, so
`claude-force-subprocess-dispatch` defaults false; `.claude/` present → claude
arm. Coordination worktree `.dispatch/SL-230` on `refs/heads/dispatch/230`.

## Phase trail

Per-phase facts the *next* phase's implementer needs. Distinct from Harvest
(which `/harvest` rewrites wholesale) and from Execution constraints (orchestrator
rules). Append one block per landed phase.

### PHASE-01 — Engine body seam · landed 2026-07-27

Coord `56520c04b → 74620ec5e` (import `290148469`, boundary `74620ec5e`). Worker
model **sonnet**; `src/entity.rs` +190/−0, six tests, no caller wired. Funnel
green: S1 baseline 0 failures at B, `regression diff` no new/changed; R-5 clean.

**`write_body` and `BodyMode` each carry
`#[cfg_attr(not(test), expect(dead_code, reason = "…"))]`, and PHASE-03 must
remove both.** `expect` (not `allow`) fails the build when the lint *stops*
firing, so the attributes self-retire by hard error the moment a caller exists —
deliberate, and the pre-existing project idiom (`src/install.rs:225-228` is the
identical `cfg_attr` form; `src/retrieve.rs:16` and `src/listing.rs:22` document
the same self-retiring phase-scoped pattern, verified live rather than taken from
the worker's word). **PHASE-02 is not the trigger** — it rides the `memory_scaffold`
fileset seam and never calls `write_body`, so the attributes must survive PHASE-02
untouched. PHASE-03 wires the first caller and owns their removal. An implementer
who meets this as a surprise build failure will reach for `allow` or delete the
wrong thing.

**The landed no-op guard is slightly broader than the design's wording.** § 5.2
says "returns false when the write was a no-op (Replace with identical content)";
the implementation compares composed content against existing for *both* modes, so
an `Append` of empty text onto a body already ending in a blank line also returns
false. A harmless superset — and the right shape for D4's `body_changed` signal,
which PHASE-03 reads. No criterion needs restating; noted so it does not read as
drift at audit.

**Reporting nit, no code defect.** The worker returned `Deviations: NONE` while
separately declaring two additions (the `Clone, Copy` derive on `BodyMode`, the
`expect(dead_code)` attributes). Both are sound and additive, and both were
verified here. `Deviations: NONE` is a funnel tripwire precisely because of this
pattern — the mandatory per-phase review it triggered is what confirmed the
`cfg_attr` idiom was real convention and not invention.

### PHASE-02 — record --body · landed 2026-07-27

Coord `9a0d7bf (M) → a5a27a403 → e7e0d42f`. Worker model **sonnet**. Four files,
+210/−7: `src/memory.rs` (+199/−8) plus mechanical field additions in
`src/boot.rs`, `src/retrieve.rs`, `src/mcp_server/tools.rs`. Three mandated tests
green; S1 diff no new/changed; R-5 clean.

**The funnel halted on `undeclared-scope` and the selectors were widened.**
`RecordArgs` is `pub(crate)`, so the plan's own objective ("`Draft` and
`RecordArgs` gain `body: Option<&str>`") forces every struct literal to change —
including 4 in `src/boot.rs` and 3 in `src/retrieve.rs`, both undeclared. All 14
lines are `body: None, body_mode: None` inside `mod tests`; no behaviour. Ruling:
widen (`90cc6e67`), not rework — a `Default` impl for `RecordArgs` would be a
larger change to shared machinery needing its own decision about what
`MemoryType`/`Status` default to. **The ripple surface is now closed**: `RecordArgs`
literals live only in `memory.rs` (10), `boot.rs` (4), `retrieve.rs` (3),
`mcp_server/tools.rs` (1); `src/commands/compare.rs:76` declares an unrelated
same-named struct. **PHASE-03 will not recur this** — `EditFields` is constructed
only in `memory.rs` and `mcp_server/tools.rs`, both long-declared.

**`seed_by_key`'s duplication is gone.** Now that `memory_scaffold` honours
`Draft.body`, `seed_by_key` passes `body: Some(body)` and its post-hoc
`fileset.get_mut(1)` block is deleted — its three existing tests needed no edit.
The design's "exactly `seed_by_key`'s existing move" is therefore now one seam, not
two. A `get_mut(1)` string survives at `:1897` in a *comment* recording the
collapse, and `write_body` appears once at `:1510` in a *doc comment* saying record
deliberately does not call it (EX-4: `materialise_named` stays the sole write).

**For PHASE-03**, three carried facts:
- `--body-mode`'s CLI type is `Option<String>`, not a parsed enum, because
  `record` refuses it unconditionally whatever its value. PHASE-03 gives the flag
  meaning on `edit` and so **owns choosing the real value parser**. `entity::BodyMode`
  is not clap-derivable as it stands.
- `resolve_body(raw, &mut impl Read)` (`src/memory.rs`) is `record`-local and not
  `pub(crate)`. `edit --body -` needs identical stdin semantics — widen or lift it,
  do not reimplement the `raw == "-"` branch.
- PHASE-01's `expect(dead_code)` attributes on `write_body`/`BodyMode` are still
  in place and still PHASE-03's to remove (PHASE-02 wired no caller, as intended).

**Two funnel mechanics learned this phase — see `## Funnel mechanics` below.**

### PHASE-03 — edit --body and the write ordering · landed 2026-07-27

Coord `e7e0d42f (B) → 232dbd3f → 4d8c1e49`. Worker model **opus** — the first
non-sonnet worker of the drive, and the right call: three files, +301/−17, with
six declared judgement calls, none of which re-litigated a stated decision. Funnel
green throughout: base `check prove` clean at B, S1 baseline 0 failures,
`regression diff` no new/changed — **honestly earned this time**, the coord
checkout was `git reset --hard HEAD`-refreshed between import and the verify beat
per ISS-252. R-5 clean; no undeclared scope.

**The three inherited PHASE-02 items are all resolved, and one was cheaper than
the trail expected.**

- **`--body-mode`'s parser** is `Option<String>` at clap, hand-parsed by a new
  `parse_body_mode(raw) -> Result<BodyMode>` sited next to `resolve_body`. **Not**
  `#[derive(clap::ValueEnum)]` on `entity::BodyMode`: `src/entity.rs` is clap-free
  and is one of the very few `src/*.rs` that is, so a derive there would put the
  first command-tier dependency at engine tier (ADR-001). Two corroborating
  reasons: every other enum-ish flag on this verb (`status`/`lifespan`/`trust`/
  `severity`) is `Option<String>` validated downstream with a worded error whose
  text its tests assert; and a shared `ValueEnum` would let clap reject
  `--body-mode bogus` *before* `record`'s landed worded refusal fires, silently
  changing PHASE-02's behaviour.
- **`resolve_body` needed neither widening nor lifting.** The trail said "widen or
  lift it"; in fact it is module-private in `src/memory.rs` and `run_edit` is in
  the *same module*, so it was already in scope. `run_edit` calls
  `resolve_body(raw, &mut io::stdin())` exactly as `run_record` does. Zero
  visibility change — `edit --body -` is stdin-identical to `record --body -` by
  construction rather than by assertion. **PHASE-05 will not inherit this for
  free**: the MCP arm is in another module and takes its body as a JSON string
  with no stdin sentinel, so it needs its own answer.
- **Both `expect(dead_code)` `cfg_attr` blocks are gone** from `src/entity.rs`
  (3 `expect(` at B → 1 now; the survivor is pre-existing and unrelated). The
  self-retiring-by-hard-error mechanism worked exactly as PHASE-01 designed it.

**For PHASE-04**, five carried facts:

- **`run_edit`'s shape is now the design's, minus PHASE-04's steps.** It runs:
  `has_any()` → `root`/`mref`/`toml_path`/`dir` → `parse_body_mode` → `resolve_body`
  → read+parse → `apply_edit` (**last** fallible step, in-memory only) →
  `write_body` (**first** disk write) → explicit `updated` re-stamp on
  `body_changed` → `write_atomic` TOML last. Steps 0/3/4 are absent and
  unobstructed — verified, `claim_snapshot`/`clear_verification` appear zero times
  in the tree. PHASE-04 inserts, not restructures.
- **Use `doc.insert(key, …)`, not `doc[key] = …`, on the owned `DocumentMut` in
  `run_edit`.** Index-assign trips this repo's `-D clippy::indexing_slicing` on an
  *owned* document; it does not fire inside `apply_edit`, whose `&mut DocumentMut`
  is why the same spelling is clean there. `insert` is `apply_edit`'s own
  non-indexing idiom for `memory_key`/`review_by`, so no allowance was introduced.
  PHASE-04's `clear_verification` writes into the same owned document and will
  meet this immediately.
  *Observation, not a defect of this phase:* `insert` on a root key shares
  `apply_edit`'s pre-existing exposure to the F-3 tail-insert hazard
  (`mem.pattern.entity.edit-preserving-status-transition`) **if `updated` were ever
  absent**. It never is — it is scaffold-seeded — and `apply_edit`'s index-assign
  has the identical exposure. Equal to the incumbent, not worse.
- **Three test helpers now exist in `mod tests`; ride them, do not re-roll.**
  `recorded_memory(body) -> (TempDir, uid, toml_path, md_path)`,
  `backdate_updated(toml_path, date)` (necessary because `record` stamps
  `clock::today()`, which masks a same-day re-stamp), and an `edit_body(root, ref,
  body, mode)` wrapper over `run_edit`. PHASE-04's clearing tests want all three.
- **PHASE-05 still owns the `body_mode`-without-`body` totality rule**, both legs.
  It was deliberately not landed and its keyword
  (`body_mode_without_body_is_rejected_on_cli`) is **not** claimed — confirmed by
  grep. Today `edit --body-mode append` with no `--body` falls through to
  `has_any()` false → "requires at least one flag": a correct pre-write rejection,
  just not yet the worded one. `has_any()` counts `body` only.
- **EX-4's test carries an extra leg beyond the mandate.** Alongside the required
  `--body <text> --trust bogus` (a failure raised *inside* `apply_edit` with a
  valid body — the only shape that actually exercises the step order), it also
  asserts `--body <text> --body-mode sideways`. An addition, not a substitution;
  it covers `parse_body_mode`'s `bail!`, which would otherwise be untested.

**`apply_edit` is provably untouched** (EX-5/R3): the whole phase deletes exactly
three lines in `src/memory.rs` — the `use crate::entity::{…}` line and the two
`run_edit` lines it replaced. Its 30+ existing tests are green unedited. Worth
recording as a technique: `git diff B..S -- <file> | grep '^-'` is a cheaper and
stronger proof of "suite untouched" than reading the suite.

### PHASE-04 — Attestation invalidation · landed 2026-07-27

Coord `4d8c1e49 (B) → 62476d71 → 162aa662`. Worker model **opus**. `src/memory.rs`
+332/−0 — a **pure insertion**, which settles "`apply_edit`'s suite is untouched"
(EX-5/R3) by construction rather than by inspection. Funnel green throughout;
`git reset --hard HEAD` applied before the verify beat again (ISS-252).

**VA-1 is DISCHARGED** — the read was done here, on the coord tree, not delegated
to the worker. The composed condition is
`let claim_changed = body_changed || claim_snapshot(&doc) != before;` — a
comparison of snapshots taken either side of `apply_edit`, with **no test of
`fields.*.is_some()` anywhere in it**. That is the thing RV-307 F-8 raised (a
decision recorded in two places and implemented in none), and it is now
implemented where it was recorded.

**EX-4 is genuinely load-bearing, and was proven so.** The test re-sets
`--path-scope` to its identical value and asserts the axis fully intact — and it
takes the verification stamp *after* the scope is in place, so the stamp really
attests that scope. Scope is the only field where the comparison and `apply_edit`
diverge (the scope arms set `changed` unconditionally where the scalars compare),
so a title-based assertion would have passed with `claim_snapshot` deleted
entirely. The worker additionally **mutation-checked both directions**: swapping
the comparison for a flag-presence disjunction fails *only* that test, and
no-op'ing `clear_verification` fails *only* `edit_claim_field_clears_verification`.
Worth generalising — a falsifier that is not mutation-checked is a claim, not
evidence.

**For PHASE-05/06**, five carried facts:

- **`clear_verification` is infallible by force, not by taste.**
  `fn clear_verification(doc: &mut DocumentMut)` — no `Result`. Step 4 sits *after*
  step 2's body write, so a fallible clear would put a failure point after a disk
  write and break PHASE-03's I10 (a rejected edit leaves both tiers byte-identical).
  Anything later that wants to add a check inside it must not.
- **Accepted residual: it diverges from `stamp_verification`'s F-1 posture.**
  `stamp_verification` (`:3417`) *refuses* a malformed file; `clear_verification`
  no-ops per missing table. So a hand-broken `[review]`/`[git]` silently keeps a
  stale attestation rather than erroring. Deliberate — the alternative was an
  unlegislated new failure mode on `edit` mid-slice. Flagged for audit; a
  legitimate finding if the auditor weighs it differently.
- **The guard is table-presence, not key-presence** (a real fork the worker named).
  On a table present but missing a key, `insert` creates it — i.e. it *clears*.
  Clearing is the fail-safe direction; the key-guarded reading would leave a stale
  attestation standing. No tail-insert corruption risk: the writes go into
  `[review]`/`[git]`'s own table bodies, not the root table, so the hazard in
  `mem.pattern.entity.edit-preserving-status-transition` does not arise.
- **EX-6's notice fires on `claim_changed`, not on "was previously verified".**
  So a claim edit to an already-unverified `thread` also emits the nudge — true but
  redundant. Narrowing it would need the pre-state snapshotted; judged not worth
  the machinery. Also `MemoryType::parse(..).ok()` degrades an unparseable type to
  *no notice*: deliberate, because the notice is emitted **after** the atomic write
  and must never be able to fail an edit that has already landed. Same reasoning
  that forces the infallible clear.
- **Test helpers available to PHASE-05/06** (on top of PHASE-03's `recorded_memory`
  / `backdate_updated` / `edit_body`): `stamp_verified`, `axis`, `cleared_axis`,
  `verified_axis`, `edit_fields`, `verified_then_edited`, and `VERIFIED_SHA` /
  `VERIFIED_DAY`. `stamp_verified` drives the fixture through the **real**
  `stamp_verification` with a synthetic `git::Frame` rather than hand-writing
  verified TOML, so the fixture cannot drift from what `memory verify` actually
  writes — reuse it, do not hand-roll a verified fixture.

**PHASE-05's inheritance is not free.** `resolve_body` served `edit` for nothing
because `run_edit` shares its module; the MCP arm does not, and takes its body as a
JSON string with no stdin sentinel. Its EX-5 ("the adapter contains argument
mapping only — no policy, no second body-write path") means delegating to
`run_edit`, which now carries the clearing for free.

### PHASE-05 — MCP body fields · landed 2026-07-27

Coord `aae133bb3 (B) → 43190de93 → 3cf7c040`. Worker model **opus**.
`src/memory.rs` **+73/−0** (pure insertion again — so `has_any()`'s field set and
`apply_edit` are untouched by construction, EX-4/R3, no inspection needed),
`src/mcp_server/tools.rs` +82/−10, `tests/e2e_mcp_server.rs` +148/−0. Funnel
green: base `check prove` clean at B, S1 baseline 0 failures, `regression diff`
no new/changed with the checkout `git reset --hard HEAD`-refreshed at **both**
ISS-252 surfaces (post-import and post-conclude). R-5 clean; no undeclared scope.

**B is a post-`refresh-base` tip, and that ordering is the point.** `dispatch
status` showed trunk **25 commits ahead of fork-point** at pre-spawn. All 25 were
`.doctrine/**` **authored-only** — zero code files (`git diff --name-only
$FP..main | grep -v '^\.doctrine/'` empty), so the merge was clean and left
`src/`/`tests/` byte-identical (`git diff --stat 162aa662 HEAD -- src/ tests/`
empty). Running `refresh-base` **before** capturing B and arming the spawn
dissolves § Funnel mechanics #2 / the handover's caveat 4 entirely: the worker
forks from the post-merge tip, so `B..S` is exactly the worker's delta and no
authored commit can fold into the boundary range. **Generalise: refresh-base at
the pre-spawn beat, never between fork and import.**

Two side-effects worth recording:

- **The authored divergence is resolved, in the right direction.** Coord
  `slice-230.toml` read `status = "ready"` against primary's `"started"`; the
  merge carried primary's value onto `dispatch/230`, which *is* primary-wins. Do
  not expect `dispatch_authored_divergence` to surface it at conclude any more.
- The coord tree's `plan.toml` / `design.md` are now current, so `verify-vt` at
  the conclude cadence reads today's plan, not the fork-point's.

**The one genuinely new problem, and its answer (D-P5-1).** `run_record` and
`run_edit` both resolve the raw body via `resolve_body(raw, &mut io::stdin())`,
where `"-"` means *read stdin*. **Over MCP, stdin IS the JSON-RPC transport** — a
caller sending `body: "-"` would block the server reading its own protocol
stream. Not a wrong answer: a hang plus stream corruption. So the adapter refuses
`body == "-"` outright (`reject_stdin_sentinel`, one const, both handlers), which
is boundary input validation of the same kind the adapter already does for
`memory_type` / `lifespan` / `status` — and therefore not the "policy in the
adapter" EX-5 forbids. The worker sited it in `tools.rs`, **not** `memory.rs`,
on a directional argument worth keeping: the guard exists *only* because of an
MCP transport fact, and the engine tier must not know that a transport exists
(ADR-001). Deliberately covered by a **unit** test, not e2e — an e2e test that got
it wrong would hang the harness rather than fail it.

**Accepted alternative, flagged for audit:** the principled fix is to plumb a
*pre-resolved* body through `RecordArgs`/`EditFields` so the core never touches
stdin at all. Rejected here as a core change inside an adapter phase. A legitimate
audit finding if weighed differently.

**EX-4 is discharged structurally, not by two assertions agreeing.** One const
(`memory::BODY_MODE_REQUIRES_BODY`) and one `bail!` site, in `run_edit`, placed
**ahead of** the `has_any()` gate (D-P5-2) — so a lone `body_mode` gets the worded
message instead of falling through to the generic "requires at least one flag",
while `has_any()` itself stays untouched and `memory_edit_no_flags_returns_32602`
stays green unedited. The MCP surface inherits the identical wording *by
delegation*. The cross-crate literal in `tests/e2e_mcp_server.rs` (an integration
crate cannot see `pub(crate)`) is not duplication to be tidied away — it is the
drift detector the pairing exists to be.

**EX-5's cheap proof:** `git diff B..S -- src/mcp_server/tools.rs | grep -E
'write_body|resolve_body|parse_body_mode'` returns exactly one hit, and it is a
doc-comment. Zero call sites. **EX-3:** neither `len(), 25` assertion appears in
the diff at all. (Their line numbers had drifted from the plan's cited
`:1488`/`:1870` to `:1535`/`:1917` — the plan pins the *count*, not the line;
worth remembering before treating a moved number as a finding.)

**Both falsifiers mutation-checked, in both directions** — continuing PHASE-04's
practice, and it paid: no-op'ing the totality guard reds *exactly* the two
totality tests; dropping the edit adapter's body mapping reds *exactly* the two
e2e body tests and leaves the full 3870-test unit suite green, which is what
proves the e2e pair covers the wiring and the unit test covers the rule. A
sharpening worth carrying: the *literal* `body: None` form of the second mutation
**does not compile** (`-D dead-code` rejects the now-unread `EditParams` fields).
That is a stronger falsifier than a red test, but it yields no runtime signal, so
the worker used `p.body.filter(|_| false)` — fields read, values dropped — to get
an observable one. **A mutation that fails to compile has not been checked.**

**One asymmetry left standing, for audit to weigh.** `body_mode` is deliberately
not exposed on MCP `memory_record` (EX-1 is explicit; not offering the param is a
stronger refusal than a worded one). But `RecordParams` has no
`deny_unknown_fields`, so a caller who sends `body_mode` anyway has it **silently
ignored**, where the CLI *refuses* `--body-mode` on `record` with a worded error.
The rule the design calls "instanced twice" is the `body_mode`-without-`body` one,
which is fully symmetric; this second rule is not. Cheap to close if wanted (accept
the field into `RecordParams` without adding it to `input_schema`, map it to
`RecordArgs.body_mode`, and `run_record`'s existing rejection fires). Left alone
because it sits outside PHASE-05's exit criteria and the orchestrator does not
widen scope mid-funnel.

**Worker-corrected orchestrator error, worth the note:** the distilled prompt
asserted `memory_show` returns the body at `memory.body`. It does not —
`show_json` nests only *metadata* under `memory`, and the body sits at the
**top level** of the envelope (`value["body"]`); the `view`/`include_body` gate
does `obj.remove("body")` on the root object. A worker that had taken the prompt
as ground truth would have written a test asserting on a missing key. The
generalisation is § Funnel mechanics' standing one: the orchestrator's distilled
terrain is a *claim*, and a worker correcting it is the funnel working.

**For PHASE-06:** the test-helper inventory is unchanged (PHASE-03's
`recorded_memory` / `backdate_updated` / `edit_body`; PHASE-04's `stamp_verified`
/ `axis` / `cleared_axis` / `verified_axis` / `edit_fields` / `verified_then_edited`
/ `VERIFIED_SHA` / `VERIFIED_DAY`). Use `stamp_verified` for any verified fixture —
it drives the real `stamp_verification`, so the fixture cannot drift from what
`memory verify` actually writes. **PHASE-06's VA-1 is orchestrator work on the
coord/landing tree** (§ Execution constraints #2), not the worker's.

### PHASE-06 — validate own-body staleness · landed 2026-07-27

Coord `3cf7c040 (B) → eb53c3770 → 8538a2658`. Worker model **opus**.
`src/memory.rs` +143/−3, one file. Funnel green throughout: base `check prove`
clean at B, S1 baseline 0 failures, `regression diff` no new/changed with the
checkout reset per ISS-252. R-5 clean.

Check 4 is a fourth **independent** block guarded on `verified_sha` alone (EX-2),
with the pathspec `format!("{MEMORY_ITEMS_DIR}/{}/memory.md", memory.uid)` —
repo-relative, because `commits_touching` already runs git with `-C root` and the
incumbent Check 2 feeds it repo-relative strings. **An absolute pathspec would
have matched nothing and returned 0 — a false green in exactly the direction that
makes the falsifier pass for the wrong reason.**

**Both mutations red *alone*, as required:** item-directory pathspec →
only `validate_ignores_verification_stamp_commit`; nesting under `scope.paths` →
only `validate_flags_unscoped_memory_body_drift`. That pairing is what separates
"correctly not flagged" from "silently inert", which this phase's falsifier cannot
distinguish on its own.

**A test-fixture defect the sheet predicted and the worker found independently.**
`stamp_verified` attested the literal `VERIFIED_SHA` (`feedface…`), which is not a
real object — so `commits_touching`'s `merge-base --is-ancestor` no-over-trust
guard folds it to `None` and **no own-body test could ever have fired**. Split
into `stamp_verified_at(toml_path, sha)` with `stamp_verified` a thin delegate;
the real `stamp_verification` is still driven, so the anti-drift property holds
and existing callers are behaviour-unchanged. Generalisable: *a fixture constant
that is not a real git object silently disables every check that walks history.*

#### VA-1 — discharged orchestrator-side, with two deviations recorded

Run on the **primary/landing tree**, not the coord tree, and not a fork. The
reason matters: on the coord tree the measurement is *deflated by ancestry* — 67
of 111 anchored memories carry a `verified_sha` unreachable from `dispatch/230`'s
HEAD, and `commits_touching` folds a non-ancestor to `None`. Measuring there
reported 3 of 111 and looked like a catastrophic under-count until the denominator
was corrected. **§ Execution constraints #2 says "coord/landing tree"; read it as
*landing*.**

Both pathspecs run over the live corpus, same denominator:

| pathspec | flagged | of 44 reachable-anchored |
|---|---|---|
| `memory.md` — **shipped** | 3 | 6.8 % |
| item directory — **the regression form** | 23 | 52 % |

**The falsifier is decisively absent**: 7.7× discrimination, nowhere near the
saturation the design predicted for the directory form. Two things not smoothed
over:

- **The design's absolute expectation does not reproduce.** It said ~11 of 30;
  shipped reports 3 of 44. *Lower*, not higher — the safe direction — and the
  corpus grew from 30 anchored to 111 between the design measurement and now, so
  the mix is not comparable. Mutation (a) independently rules out a silently-inert
  check, which is what the absolute figure was standing in for. Recorded because a
  criterion whose number no longer reproduces should be re-stated, not quietly
  re-interpreted.
- **New finding, for audit: 67 of 111 anchored memories are silently exempt from
  BOTH staleness checks.** Their `verified_sha` is not an ancestor of HEAD
  (verified on a branch or linked worktree whose commits never landed), so
  `commits_touching` returns `None` and the row degrades to "no drift". This is a
  **pre-existing property of the incumbent seam**, not a PHASE-06 regression — but
  it caps the corpus-wide reach of both checks at ~40 %, which no criterion in this
  slice states. Candidate ISS.

**ISS-028 / § Execution constraints #3 confirmed, and benign.** `doctrine check
commit` inside a worker fork goes red on three `tests/e2e_adr_cli_golden.rs` cases
with `refusing authored write 'adr status'` — the worker-marker authored-write
guard tripping e2e tests that shell the binary. Orthogonal to this delta; the
`worker_commit` gate runs with the right environment and returned `Committed`. The
constraint said to check this on first worker return — it survived all six phases
without biting, because no phase's own suite shells the CLI.

### Decision records swept from the runtime sheets — RV-313 audit, 2026-07-27

The runtime phase sheets (`.doctrine/state/slice/230/phases/`) are gitignored and
`rm -rf`-able. `D-P5-1` and `D-P5-2` were already durable above; these four existed
**only** in the sheets and would have died with them. Swept verbatim in substance,
compressed in form.

- **D-P5-3 — the refusal wording is a named constant, not a literal.** Per STD-001
  the shared text is one `const` in `src/memory.rs` (`BODY_MODE_REQUIRES_BODY`),
  referenced by the `bail!` and by the VT-2 unit assertion. The e2e test lives in
  an integration crate and cannot see `pub(crate)`, so it asserts a substring
  literal; that duplication is unavoidable and *is* the drift-detector the pairing
  exists to be. The rule the constant encodes: the message must read correctly on
  both surfaces because it is literally one string — never split into two messages.
  *Audit note:* RV-313 F-3 extended the same discipline to `record` — its refusal
  also stays authored once, in `run_record`.
- **D-P6-1 — the pathspec is a repo-relative string, not an absolute path.**
  `commits_touching` shells `git -C <root> … -- <paths>` and the incumbent Check 2
  feeds it repo-relative `scope.paths`, so Check 4 builds
  `format!("{MEMORY_ITEMS_DIR}/{uid}/memory.md")`. The design's
  `root / MEMORY_ITEMS_DIR / uid / "memory.md"` phrasing describes *where the file
  is*, not the argument form. **An absolute pathspec matches nothing and returns
  0** — a false green in exactly the direction that makes the falsifier pass for
  the wrong reason.
- **D-P6-2 — the message must distinguish itself from Check 2's, and keep the
  `"{uid}: "` prefix.** Check 2 says `… behind HEAD on scoped paths`; Check 4 says
  `… behind HEAD on its own body (memory.md)`. The prefix is load-bearing:
  `memory_health_findings_native` extracts the entity by matching it, so the new
  finding rides the doctor surface for free *only* if the shape is preserved. T41
  asserts it sees this finding and not Check 2's — the wording is what makes that
  assertion sharp.
- **D-P6-3 — EX-6 is discharged by an honest comment, not by code.** Global
  masters stay uncovered by construction (unanchored ⇒ empty `verified_sha` ⇒
  guarded out). The deliverable was a doc comment recording *why*, plus the
  standing mitigation (`mem.system.memory.global-master-authoring`), so a later
  reader cannot mistake silence for coverage. No master handling was added.

### Conclude cadence — run 2026-07-27

`slice verify-vt 230` → `dispatch sync --prepare-review` → 8 refs projected
(`review/230`, `phase/230-01..06`, `dispatch/230`). Completeness gate passed: all
six phases carry a registry row.

**S6 embed — VT gate: all 8 rows `✓ PASS`.** Zero `Fail` / `WAIVED` /
`UNCHECKABLE`. The handover's blanket `≈ UNATTRIBUTABLE` block was exactly the
registry-derivation artifact it predicted, and it resolved on `prepare-review`
with no intervening code or plan change — which is now recorded in ISS-252 as
empirical confirmation that the pre-`prepare-review` reading carries no signal.

**S6 embed — S1 regression: green all six phases.** `check regression diff` →
`✓ no new or changed failures` at PHASE-01 (base `56520c04b`), 02 (`74620ec5e`),
03 (`e7e0d42f`), 04 (`4d8c1e49`), 05 (`aae133bb3`) and 06 (`3cf7c040`). Baselines
0 failures at every base. Every green from PHASE-03 on was taken after the ISS-252
reset — genuinely earned.

**ISS-252 gained a THIRD surface here:** `sync --prepare-review` is working-tree-
free too, and left the tree staged with the inverse of its own journal commit
(`D .doctrine/dispatch/230/journal.toml`, −57). Written into the issue as a *class*
rule rather than a third enumerated instance — every working-tree-free write on
this arm has this post-condition, so a fix patching only the known verbs will be
overtaken by the fourth.

## Funnel mechanics (claude arm)

Orchestrator-side operating facts, learned by hitting them. Not slice-specific;
candidates for `/record-memory` once a second slice confirms them.

**1. `dispatch_import` / `dispatch_conclude_phase` do not touch the coord
checkout — refresh it before the verify beat.** Both are working-tree-free by
design (they compose via `merge-tree` into the object DB), so they advance
`dispatch/<slice>` while leaving the coord index and worktree behind. After
PHASE-01's import the checkout still had no `write_body`, and `git status` showed
`M src/entity.rs` / `D boundaries.toml` — the *inverse* of the landed commits.
Consequence: **the first `check regression diff` ran against the pre-delta tree and
was vacuous** — it reported green having never compiled the delta. Caught before
concluding, and re-run honestly. **Ritual: `git reset --hard HEAD` in the coord
tree between `dispatch_import` and the verify beat.** The router's funnel order
("Verify — suite @ S") assumes a tree at S; nothing in it says to put the tree
there.

**2. After `refresh-base`, set the next phase's `code_start` to the post-merge tip,
not the worker's fork base.** `refresh-base` merges trunk into the coordination
branch, so a boundary of `(fork_base, post_import_tip)` spans the merge and folds
every authored commit trunk brought in into the phase's conformance range —
`.doctrine/**` paths that no design-target selector declares, i.e. a pile of
spurious undeclared-edit findings at audit. Using the post-merge tip `M` makes
`M..tip` exactly the worker's delta (verified: 4 files, +210/−7). Safe because
`classify_import` treats `head_at_base`/`tree_clean` as vacuously true on the
funnel path (`src/mcp_server/dispatch.rs:296-299`), so moving coord HEAD before
import is not a precondition breach.

**3. The import belt reads selectors from the COORD tree, not the primary.**
`crate::slice::selectors(&coord.root, ...)` (`src/mcp_server/dispatch.rs:246`). So
widening scope mid-drive is a three-step move: edit + commit in the primary tree →
`git fetch . edge:main` → `dispatch refresh-base --slice <N>`. Editing only the
primary tree leaves the belt reading stale selectors and the refusal stands.
