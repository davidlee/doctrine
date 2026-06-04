# slice-004 audit

Post-build audit of the implementation-plan + phase siblings. Status `done`
(2026-06-04); gates green — `cargo test` (84), `cargo clippy` (deny-level,
default targets — `--all-targets` is *not* the gate, it trips the pre-existing
test-only `unwrap_used`/`indexing_slicing`), `cargo fmt`. The slice-001 + slice-003
suites passed unchanged at every step. End-to-end verified against a scratch
project: install → `slice plan` → author phases → `slice phases` (symlink resolves)
→ `slice phase` (toml_edit mutate) → `slice notes` → idempotent re-run → drift
report → `--prune`; git ignores the whole runtime surface.

§1 is the implementation-vs-design verdict, §2 the findings, §3 the deferred set.
The **appendix** preserves the pre-build design-review trail (rounds 1–2 +
dispositions) verbatim — the *why* of D1–D8 and the six gated revisions. Settled
history; do not relitigate.

## 1. Implementation vs design — verdict: faithful

Every §7 decision and §5.5 invariant landed as specified; the §5.6 seven-step
sequence was built in order, each step green against slice-001/003.

| Design item | Where | Status |
|---|---|---|
| D1 IP = slice facet, 2-Artifact `CreateInExistingEntity` | `slice::PLAN_KIND`/`plan_scaffold` | ✓ no `IP-` entity |
| D2 phase content sorts durability × structure | `plan.toml` (authored) · `phase-NN.toml` (status+log) · `phase-NN.md` (prose) | ✓ |
| D3 tracking in `state.rs`, not the engine; shared IO | `src/state.rs` + `src/fsutil.rs` | ✓ no `MutateInPlace` |
| D4 transactional sub-artefact writer | `entity::write_fileset` (component `create_dir` + `create_new` + symlink arm + reverse rollback) | ✓ **discharges 003 `[M]`** |
| D5 `toml_edit` for the state writer; tracking graduates | `state::set_phase_status`; v1 = status + progress only | ✓ |
| D6 gitignore state + `phases`; state not a managed `[dir]` | `install/manifest.toml [gitignore]` | ✓ recreated on demand |
| D7 verbs `slice plan`/`phases`/`phase`/`notes` | `main::SliceCommand` → `slice::run_*` | ✓ |
| D8 `notes.md` durable+scaffolded; `handover.md` toolless | `NOTES_KIND`/`run_notes`; handover = gitignore only, no verb | ✓ |
| F1 atomic clobber (`create_new`) | `fsutil::create_new_file` | ✓ no TOCTOU |
| F2 component-wise `create_dir` accounting | `entity::ensure_parent_dirs` | ✓ tested rollback |
| F4 shared `fsutil` + `phase_id` validation | `fsutil::{safe_join,is_real_dir,set_symlink}` + `state::phase_stem` | ✓ |
| F5 plan-drift report | `state::init_phases` → `InitReport` | ✓ orphan/prune tested |
| F6 phase-id uniqueness (v1) + immutability rule | `Plan::parse` (dup reject); immutability = authoring contract | ✓ |
| F10 verified symlink refresh | `fsutil::set_symlink` | ✓ wrong-link replaced, squat errors |
| §5.5 tracked-tree purity / symlink-blind / edit-preserving | state writes only under `.doctrine/state`; resolve-by-id; comment/unknown-key survive | ✓ each tested |

**Faithful reconciliations vs design.md:**

- **`phase-NN.toml` is rendered inline (`state::render_tracking`), not an install
  template.** The §5.3 template list names only the four `.md`/`plan.toml` assets;
  the tiny machine-owned tracking skeleton is doctrine-controlled runtime, so it
  lives in code, not `install/templates/`. (The disposable `phase.md` *is* a
  template, as specified.)
- **Validation split, single-source.** §5.6 step 3 attributed phase-id
  well-formedness *and* uniqueness to the `Plan` read model. Uniqueness lands in
  `Plan::parse` (a pure plan invariant); well-formedness lands in `phase_stem` —
  the one `PHASE-<digits>` validator, fired at the filesystem boundary where an id
  becomes a filename (no parallel regex). Matches §9 exactly.
- **`Plan`/`PlanPhase` model only `id`/`name`/`objective`.** Criteria/verification/
  link fields exist in `plan.toml` but are unconsumed in v1 (D5 graduation), so
  modelling them would land dead fields under `deny(unused)`. They round-trip on
  disk untouched.
- **`set_phase_status` takes a `now` parameter** (the clock stays in the shell,
  mirroring `slice::today()`); **`init_phases` takes `prune: bool`** (the `--prune`
  flag the design signature elided).

## 2. Findings — rough edges

None block; all post-v1.

- **[L] `toml_edit` re-renders a *mutated* key's whitespace.** Setting `status`/
  `started`/`last_updated` collapses their cosmetic alignment (`status  =` →
  `status =`). The contract held — comments, unknown keys, and *untouched* keys
  survive byte-for-byte (tested) — only set-key alignment shifts. Cosmetic.
- **[watch] `state.rs` depends on `slice::{Plan,PlanPhase}`.** A deliberate
  consumer edge (state reads the plan), but it couples the runtime module to a
  slice-side type. If a second consumer of `Plan` appears, lift it to a neutral
  home. Fine for v1.
- **No `slice show` reassembler** (Q3) — plan + tracking are queryable separately;
  the read-locality CLI is deferred.

## 3. Deferred (unchanged from design)

- Graduation of per-criterion/verification/task *status* to TOML rows when a
  consumer lands (D5/Q2/Q5) — v1 ships them as a `phase-NN.md` checklist.
- Spec/requirement registry + FK validation (`plan.overview` link fields stay empty).
- Slice close-out audit: harvest phase-sheet risks/decisions/findings into the
  tracked audit; GC `handover.md`.
- Broader `.doctrine/state/` surface (session/lease/review-index).

---

# Appendix — pre-build design-review trail

Pre-build adversarial design review of [design.md](design.md) + [slice-004.md](slice-004.md).
Status `proposed`; no code touched. Reviewer: gpt-5.5 (hostile pass, codex MCP),
adjudicated here. This was the disposition trail the `ready` gate read; it
supersedes the review dialogue. Same rhythm as [003/audit.md](../003/audit.md).

The reviewer's code claims were re-verified against `src/entity.rs` — all accurate
(no claim-drift introduced by the review): `refuse_clobber` (:285) is a bare
`exists()` separated from `fs::write` (:307); `write_fileset` uses `create_dir_all`
(:303); the symlink path (:311) is `AlreadyExists`-tolerant. The design's §5.4
track-and-unlink pseudocode is **not buildable as written** — that is the core find.

## Round-2 confirmation pass (gpt-5.5, `needs-another-round`)

A second hostile pass on the revised design verified the fixes and caught residue + two
contradictions the first pass missed. All adjudicated **accept** and folded in:

- **Writer (1+2) — three real holes:** (i) `created_paths` tracked *before* the content
  write, so a mid-write failure unwinds the partial file; (ii) the `Artifact::Symlink` arm
  is preserved (shared `write_fileset`); (iii) `create_dir` `AlreadyExists` verifies a real
  dir (a file squatting a path component now errors). §5.4 pseudocode rewritten.
- **Regenerable (3) — my own round-1 imprecision:** swept "regenerable" out of runtime
  contexts (§4/§5.3/D5/table). Runtime progress is **disposable / loss-accepted**, not
  regenerable — a timestamped log can't be rebuilt from the plan; only the empty scaffold
  re-seeds.
- **Id↔filename (4):** pinned `PHASE-NN` ↔ `phase-NN.{toml,md}` mapping; `validate_phase_id`
  → `phase_stem` (validates + derives).
- **Phase-id uniqueness (6):** validated in **v1** (not deferred) — phases are consumed
  immediately, so a duplicate id would alias two phases onto one file. Criterion-ref
  validation still graduates with its consumer.
- **Partial-init strand (new):** `init_phases` skip is **per-file**, so a phase half-written
  by a crash completes on re-run.
- **WHAT contradiction (new):** slice-004.md:56 ("state is another engine `tree_root`") was
  stale vs D3 — corrected to the `state.rs` module.
- **Build sequence (11):** restructured so each `fsutil` primitive extracts in the step that
  first calls it — no dead fn trips `deny(unused)`.
- **handover/entity-model (7):** added explicit carve-out justification — `handover.md` is an
  unstructured toolless note, *not* the structured "handoff" cache entity-model:109 homes
  under `.doctrine/state/`; tracked-tree purity is the enforced boundary.

## Resolution (post-review)

All eleven dispositions **folded into design.md** (§10 indexes them). User decisions on the
two judgment calls:
- **Finding 3 (phase-sheet durability):** option (a) — disposable scratch; the **future
  slice close-out audit** harvests durable risks/decisions/findings into the tracked audit
  artefact (new Follow-Up in slice-004.md). Wording corrected: disposable, *not* regenerable.
- **Finding 7 (handover home):** refine the invariant — "the git-*tracked* authored tree is
  pure"; `handover.md` stays in the slice folder (gitignored), and the close-out audit GCs it.

Gate now returns to the user (`ready` is yours). Original verdict + per-finding adjudication
preserved below as the review record.

## Verdict (at review time): not-ready. Six revisions gate the build; one needs a user decision.

Severity after adjudication (reviewer severity → mine where changed):

| # | Finding | Sev | Disposition |
|---|---|---|---|
| 1 | Non-atomic clobber/write (TOCTOU) | BLOCKER | revise — `create_new(true)` |
| 2 | `create_dir_all` can't account created dirs | BLOCKER | revise — component-wise `create_dir` |
| 3 | Phase sheet declared disposable but holds non-regenerable findings | BLOCKER | **revise — needs user call** |
| 4 | state.rs forks fs-safety primitives | MAJOR | revise — extract shared `fsutil` |
| 5 | Idempotent `init_phases` masks plan drift | MAJOR | revise — add drift detection |
| 6 | Unstable criterion refs (`EN-1`/`EX-1`/`VT-1`) | MAJOR | revise — id immutability rule (validation defers with graduation) |
| 7 | `handover.md` pokes the authored/runtime boundary | MAJOR | **user decision** — move or refine invariant |
| 8 | WHAT/HOW disagree on v1 tracking scope | NIT (was MAJOR) | partial-reject — reviewer misread; clarify one line |
| 9 | IP facet→entity escape hatch not additive | MINOR (was MAJOR) | accept-with-note — clarify Q4 |
| 10 | Symlink "refresh" doesn't refresh | MINOR (was MAJOR) | accept — but symlink-blind invariant caps blast radius |
| 11 | No build sequence; templates undrafted; no `toml_edit` dep | MINOR | accept — fill pre-ready |

## Findings & dispositions

### [BLOCKER] 1 — non-atomic clobber/write (D4, §5.4)
`refuse_clobber` does `exists()` (entity.rs:285) then `write_fileset` does `fs::write`
later (:307) — a TOCTOU window. design.md:305 preserves this two-step under the new
transactional writer.
**Disposition: revise.** Collapse refuse-clobber and write into one atomic step:
`OpenOptions::new().write(true).create_new(true)` per file. `create_new` *is* the
clobber refusal (fails `AlreadyExists`) and closes the race in one syscall. The
separate pre-write `refuse_clobber` pass can stay as a fast-fail courtesy but is no
longer the safety boundary. The concurrency threat is low (doctrine is a local
single-user CLI) — the justification is that `create_new` is strictly better and
near-free, not that parallel agents are a live scenario. Add a same-target
double-write test.

### [BLOCKER] 2 — directory accounting is unbuildable (D4, §5.4)
design.md:309 says "record each dir `create_dir_all` newly makes." `create_dir_all`
returns `()` — it cannot report which components it created vs found. The current
writer calls it directly (entity.rs:303). So the "parent exactly as pre-call"
invariant (design.md:344) is unprovable and the rollback can over- or under-clean.
**Disposition: revise.** Replace `create_dir_all` in the transactional path with
component-wise `create_dir`: walk the rel path, `create_dir` each missing component,
push only the ones that returned `Ok` (i.e. *this call* created them) onto
`created_dirs`. Unwind in reverse with `remove_dir`, ignoring `NotFound` and
`DirectoryNotEmpty` (another artefact or process may have populated it — never force).
Tests: (a) pre-existing parent untouched on rollback; (b) a dir concurrently
populated mid-write is not removed.

### [BLOCKER] 3 — the phase sheet is not disposable (D2/§5.1/§5.3) — needs a user call
The design calls `phase-NN.md` "disposable / regenerable from the plan" (design.md:107,
:255) and gitignores the whole `.doctrine/state/` tree (D6). But §5.3 (design.md:272-275)
routes **assumptions/STOP, risks, decisions, findings, task-details** *only* into that
sheet — none of which exist in `plan.toml`, so none are regenerable. `rm -rf
.doctrine/state/` then destroys real execution-time decisions and findings, and a later
TOML "graduation" (D5) silently drops hand-edited prose because the tool never parses it.
This is the deeper form of the durability issue already tightened in §4/D2 this round —
the *status framing* was fixed, but the *prose sheet* still claims a disposability its
contents don't have.
**Disposition: revise — user decides the cut.** Two coherent resolutions:
- **(a) Accept the loss.** Execution-time risks/decisions/findings *are* throwaway working
  notes; anything worth keeping is the author's job to lift into a durable surface
  (`notes.md`, or a future audit). Then the sheet is honestly disposable — but say so
  explicitly and drop the "regenerable" claim (it is disposable, not regenerable).
- **(b) Route durable content out.** Keep only genuinely regenerable scaffolding
  (objective echo, reading-list, task checklist) in the disposable sheet; send durable
  *decisions/findings* to a tracked artefact. Heavier; closer to the entity model's
  "findings are rows" stance (entity-model.md:73).
Recommend **(a)** for v1 (cheapest, matches "disposable runtime"), with the wording fixed
so no one trusts the sheet to survive `rm -rf`. **User picks.**

### [MAJOR] 4 — state.rs forks filesystem safety (D3)
`safe_join`, atomic-create, and the symlink policy are private to `entity.rs`
(entity.rs:331). A new `state.rs` IO owner (design.md:409) re-implements all of them,
and `phase_id` is an unvalidated input (design.md:208) — a `phase_id` containing `/` or
`..` would escape the state tree.
**Disposition: revise — and it strengthens D3.** Keep mutation out of the scaffold
engine (reject `MutateInPlace`, the author is right there), but the missing piece is
**shared IO primitives**, not a forked module. Extract a small `fsutil` (safe-join,
atomic `create_new` write, real-directory check, symlink-set-or-replace) consumed by
both `entity.rs` and `state.rs`. Add explicit `phase_id` filename validation (reject
empty, `/`, `..`, leading dot). This is a net design improvement — fold it into D3.

### [MAJOR] 5 — idempotent init masks plan drift (Q1, §5.5)
`init_phases` skips phases whose files exist (design.md:200); the idempotence invariant
only promises "re-run adds new phases" (design.md:346). A phase renamed, reordered, or
removed in `plan.toml` after tracking accrued leaves an orphan `phase-NN.{toml,md}` that
a rollup still reads as live.
**Disposition: revise.** `init_phases` must diff existing tracking `phase` fields against
the current plan and **report** three classes: new (materialise), orphan (tracking exists,
plan phase gone), stale (id reused / renamed). Orphan repair is explicit/destructive, never
silent. Add an invariant + a rename-then-reinit test.

### [MAJOR] 6 — unstable criterion refs (Q2/Q5, §5.2/§5.3)
Plan criteria carry local ids `EN-1`/`EX-1`/`VT-1` (design.md:178); future tracking rows
join by `ref` (design.md:233) with no FK validation (no registry). Renumber/delete/reuse
aliases accrued status onto the wrong criterion or dangles it.
**Disposition: revise — the rule now; validation with the consumer.** State an invariant:
phase and criterion/verification ids are **immutable and never reused** once authored;
edits append, never renumber. The live bite is *deferred* — v1 ships status as a markdown
checklist (graduation, D5), so no `ref` join exists yet — but the immutability rule is
free to write down now and is the precondition the graduated consumer (`slice validate`,
M5) will validate (dup/dangling check). Use phase-qualified refs for any cross-phase
reference.

### [MAJOR→user] 7 — handover.md pokes the authored/runtime boundary (D8)
`handover.md` is disposable + gitignored but lives in `.doctrine/slice/<id>/`
(design.md:130), while the invariant says all churn lives under `.doctrine/state/`
(design.md:338; entity-model.md:107). So a gitignored runtime file sits inside the
authored entity dir.
**Disposition: user decision.** Two clean options:
- **Move** to `.doctrine/state/slice/<id>/handover.md` — keeps the boundary literally pure;
  costs the zero-ceremony "right next to the slice" ergonomics that are the whole point.
- **Refine the invariant** to "the git-*tracked* authored tree is pure" — `handover.md` is
  gitignored, so it never enters the tracked surface; physical adjacency is allowed for a
  toolless convenience file.
Lean **refine-the-invariant** (preserves the design's stated rationale), but it is a
genuine boundary judgment — **user picks**. Either way `notes.md` (durable, tracked) is
unaffected and correct.

### [NIT] 8 — WHAT/HOW v1 tracking scope (partial reject)
The reviewer read slice-004.md:22 ("task counts … per-task done/blocked flags") as a v1
commitment the design (status+progress only, design.md:212) violates. **Reject the
contradiction:** line 22 describes the *spec-driver source schema* being adapted, and the
Scope section (slice-004.md:68-79) already commits to graduation. No conflict.
**Disposition: clarify one line.** Add a half-clause to slice-004.md:22 marking that list
as the source schema, trimmed per the Scope graduation note — so a future reader can't
misread it as the v1 surface.

### [MINOR] 9 — IP escape hatch additivity (Q4)
Q4 calls facet→reserved-`IP-`-entity "additive" (design.md:372), but plan schema and state
paths key off the slice id (design.md:163, :195); multi-plan would migrate accrued state.
**Disposition: accept-with-note.** Clarify Q4: `plan.toml` remains a permanent 1:1 slice
facet. If multi-plan ever emerges, it is a *new additive surface* (a reserved entity for the
*new* plans) — the legacy facet persists unmigrated, not rewritten. State it; no schema
change now.

### [MINOR] 10 — symlink "refresh" doesn't refresh (§5.4)
Tolerant-create (entity.rs:311) accepts any existing path, so a stale/wrong
`.doctrine/slice/<id>/phases` symlink — or a real dir squatting it — persists.
**Disposition: accept, bounded.** Fix the writer to verify the target and replace a wrong
symlink / fail on a real file or dir. But severity is MINOR: the **symlink-blind invariant**
(id is identity, the tool never follows the link) means a stale symlink is a human-browsing
nuisance, not a correctness bug. Reviewer's MAJOR over-rates it.

### [MINOR] 11 — build sequence, templates, dependency (author-admitted gaps)
No ordered build sequence (slice-003 had one in its §5.4); `plan.{toml,md}`/`phase.md`/
`notes.md` templates undrafted; `toml_edit` absent from `Cargo.toml`.
**Disposition: accept — fill before `ready`.** Add an ordered, each-step-green build
sequence (the 7-step skeleton in the handover, re-confirmed against findings 1–7);
draft concrete template contracts (the storage rule constrains `phase.md` — prose only,
no queried data); add `toml_edit`.

## Build-gating set (must land in design.md before `ready`)

1. **Transactional writer rewrite** (1, 2) — `create_new` atomic write + component-wise
   `create_dir` track/unwind. Rewrites §5.4 and the D4 pseudocode. *This is the slice-003
   `[M]` debt — get it right or the discharge is fiction.*
2. **Phase-sheet durability** (3) — pick (a) or (b); fix the "disposable/regenerable"
   wording to match. **User input needed.**
3. **Shared `fsutil` + `phase_id` validation** (4) — fold into D3.
4. **Plan-drift detection in `init_phases`** (5) — new invariant + test.
5. **Id-immutability invariant** (6) — rule now, validation graduates with the consumer.
6. **handover boundary** (7) — move or refine-invariant. **User input needed.**

Clarifications (cheap, non-gating): 8 (slice-004.md:22), 9 (Q4), 10 (symlink fix), 11
(build sequence + templates + dep).

Two user decisions before the design revision is final: **finding 3** (phase-sheet
durability cut) and **finding 7** (handover home). Everything else is mechanical revision.
