# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · started, PHASE-03 (3 of 6 landed; coord `dispatch/230` at
4d8c1e49) · 08274695

### Produced

- **PHASE-01** engine body seam — `BodyMode` + `write_body` in `src/entity.rs`
  (coord 56520c04b→29014846, boundary 74620ec5e).
- **PHASE-02** `record --body` — rides the `memory_scaffold` seam; collapsed
  `seed_by_key`'s duplicate index-1 substitution onto `Draft.body`
  (coord 9a0d7bf→a5a27a403, boundary e7e0d42f).
- **PHASE-03** `edit --body` + the two-tier write ordering — the first caller of
  `write_body`; retires PHASE-01's `expect(dead_code)` staging attributes
  (coord e7e0d42f→232dbd3f, boundary 4d8c1e49).
- **ISS-248** — selector doctor's redundancy scan is class-blind.
- **ISS-252** — claude-arm funnel verify beat runs against a stale coord
  checkout; fails open as a false green.
- Selectors widened to declare the forced `RecordArgs` ripple (90cc6e67).
- Per-phase detail and PHASE-03's inherited facts: § Phase trail. Orchestrator
  operating facts: § Funnel mechanics. Both prose, both committed.
- `.doctrine/` changes committed promptly and **separately** from code throughout —
  code lands only on `dispatch/230` via the funnel; authored state only on `edge`.
- Gate: `doctrine check gate` last green at 56520c04b (pre-drive). Since then the
  standing gates are per-phase — `check prove` at each base + `check regression
  diff` at each tip, both green for 01 and 02. `check gate` has NOT run against
  the coord tip; it is the close ritual, not a per-phase beat.

### Learned

- **Unrecorded memory candidates, still owed** (they predate this drive and
  survived the design-phase harvests): the tool-property-is-a-claim-needing-a-
  falsifier lesson; "run the query" (execute a design's stated query against the
  real corpus); mechanical-narrowing-damages-joints + `git show <commit>^:<path>`
  as the cheapest instrument. Bodies in git at 68bf7f21's notes.md; **not**
  restated here.
- § Funnel mechanics' three items are memory candidates once a second slice
  confirms them; ISS-252 carries the load-bearing one as actionable work.

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
- Moved ids never reused. Next free: I14, E15, T43.

### Next

1. `/phase-plan` PHASE-04, then dispatch it. Half the inseparable pair — 03's HEAD
   alone is strictly worse than today, so **do not `sync --integrate` until 04
   concludes**. Steps 0/3/4 of design § 5.4 land into the shape 03 created.
2. Then PHASE-05, PHASE-06.
3. Conclude cadence once 6/6: `slice verify-vt 230` → `dispatch sync
   --prepare-review` → remove coord worktree (keep refs) → `slice status 230
   audit` → `/audit`.
4. `/design` SL-232 from F-37, F-36, and F-14.

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
