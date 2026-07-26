# SL-228 — notes

## Harvest

fresh-as-of: plan authored, slice `plan` status (RV-305 round-3 closed, design converged, User-approved) · head `b627cb02`

### Produced
- `extraction.md` (committed `0603f11f`) — as-built funnel state machine, crux
  verdicts, per-verb invariant table. Design input; stays useful post-close as
  the D7 artifact's ancestor.
- `design.md` — D1–D8 locked with User; internal pass (6 findings) + **external
  RV-303 codex inquisition integrated**: 12 findings (8 blockers), 2 rounds,
  5 second-round contests all upheld, 12/12 verified terminal. Commits
  `ccc8fbae` (round 1), `16d18e8b` (round 2), `1b716673` (synthesis).
- **RV-303** — closed ledger, facet design, raiser inquisitor (codex/GPT-5.5);
  synthesis carries the closure story + standing risks.
- **RV-304** — closed round-2 ledger (fresh codex thread over the post-RV-303
  design; premise: the 12 repairs are unreviewed surface). 10 findings
  (7 blockers), 10/10 confirmed against code, all fix-now, 10/10 verified
  terminal; F-2 took three upheld contests (clobbering write → TOCTOU →
  spawn–gc race), landing the claim→bind→act fork protocol under a per-name
  crash-released claim lock. Commits `be7f3abb`, `0ce0c62e`, `dbcebc22`,
  `527e55bd`.
- **RV-305** — closed round-3 ledger (scoped codex pass over the RV-304 repair
  surface only; governance axes user-excluded; convergence rule declared in
  brief). 2 findings: F-1 blocker (worker_commit crash replay unreachable past
  the AtBase belt) took two upheld contests — belt-gated provenance-indifferent
  adoption, then the one-commit heal rule; F-2 minor (stale-token gloss vs
  modulo-record gate). 2/2 verified terminal; **design declared CONVERGED**
  (12 → 10 → 2 decay). Commits `e3bce6f3`, `5a755f0d`, `3e100b3e`, `09aecf7a`.
- **Design approved by User** (post-RV-305); `/plan` run.
- `plan.toml` + `plan.md` (commit `b627cb02`) — 7 phases refining design §12:
  move E (01 reads, 02 guard/ISS-234), move A split (03 machine+record+
  run_suite, 04 fork-binding protocol, 05 gates+verify+receipt), 06 oracle+
  skills, 07 OQ-5 benchmark. All VT rows carry structured mandates
  (verify-vt: zero UNCHECKABLE). Phase sheets materialised; status → `plan`.
- `design-target` selectors recorded (15).
- Scope `## Follow-Ups` reconciled to design (OQ-2/NEW-OQ-A/NEW-OQ-B → D1/D6/D7).

### Learned (durable sinks already hold these)
- The reverse-diff resync trap (`reset --keep` broken when ref advanced under
  checkout) is memory-pinned; design §5/R3 rides the `restore`-based idiom —
  now per-path proof-gated (RV-303 F-6).
- Two-altitude finding (sub-funnel has no `select_guidance` node) — extraction
  §3; drove D4's "new oracle, not carve-out" framing.
- A record that rides a commit's own tree cannot store that commit's oid —
  Class-1 rows carry input facts only; output identity is post-hoc provenance
  (RV-303 F-2, design §3).
- Sheet-first conclude ordering was the real ConcludeIncomplete window; the
  fix is CAS-first with the sheet as trailing projection (RV-303 F-5, §4/§9).
- A "record the act durably" repair has three successively deeper failure
  modes — write ordering, overwrite clobber, then check-then-act races
  (incl. against the sweeper) — the stable shape is claim (atomic primitive
  you already own, here the branch ref) → bind → act, with the
  cleanup path taking the same claim (RV-304 F-2's three contests, design §3).
- Replay identity must be built from a retry's *recoverable* facts only —
  any freshly-sampled input (a live tip) in the identity key turns the
  idempotence promise into a refusal (RV-304 F-1, design §2).
- A replay promise is void if the verb's own belts refuse before the replay
  comparison runs — reachability is part of the contract (RV-305 F-1);
  adoption of unprovenanced work is sound only when gated by the same content
  belts the first-path enforces (belts are the authority, not authorship);
  and a heal that lands prefix rows in a separate commit from its own
  transition forges the induction that keeps ungated rows out — heal is
  all-or-none, one CAS commit (design §3/D8).
- Three-round review decay (12 → 10 → 2 findings) with a pre-declared
  convergence rule + scoped round-3 brief worked well as a stopping protocol.
- CLI friction for RFC-011: `explore` is a boot-spine group, not a CLI
  subcommand (use top-level `doctrine inspect`); requirement statement text is
  unreachable via CLI (`spec req list` shows `prose —`) — lives in
  `.doctrine/requirement/NNN/requirement-NNN.toml`; `review status` takes only
  RV refs (no `--slice` filter) (case-notes appended).

### Open
- **Plan approval gate**: plan authored + committed; User signalled dispatch —
  flip `ready` at handover, then `/phase-plan` PHASE-01 under `/dispatch`.
- Plan-phase items deliberately deferred (unchanged): OQ-5 benchmark harness
  shape; hook script asset path (selector added when picked);
  `NextCore.command` binary-name rendering; REQ-287 prose mapping (ship-time
  REV per Non-Goals); stale-baseline derivation exactness (§5 — algorithm
  sketch, R3-load-bearing; PHASE-05 must pin it before landing verify).
- VT keyword floors deliberately minimal where naming is free (PHASE-04 VT-1
  `claim`, VT-4 `lock`) — `/phase-plan` appends stronger mandates once file
  shapes exist.

---

## Harvest — execution (dispatch)

fresh-as-of: **PHASE-08 concluded + reaped** (`ef2d3e81` post-conclude remediation,
`186e1bb1` harvest) · coord tip `186e1bb1` · dispatch/228 · **7/7 landed; only
PHASE-07 (OQ-5 memory-blind benchmark) remains** · VT gate **`23/23 PASS`, 0
UNCHECKABLE, 0 WAIVED — `verify-vt` exits 0** · S1 regression clean at PHASE-08
(`check regression diff --base 1c6949965` → no new or changed failures).
Prior stamp (PHASE-06 + RV-308 closed, 21/21, coord tip `463b512a`) superseded.

### Landed
- **PHASE-01** (Move E reads) — boundary `29e966f9d..8c4e765a`. 4 git.rs primitives
  (`tree_clean_untracked`/`three_dot_diff`/`log_oneline`/`check_ignore`),
  `current_branch` relocated into git.rs (all callers migrated, none remain in
  dispatch.rs), `is_linked_worktree` wrapped, 5 read verbs (`tree-state` CLI+MCP,
  `delta`/`whereami`/`history`/`ignored` CLI). Verify green (0 S1 regressions).
- **PHASE-02** (safe-commit guard) — boundary `f54ab23e..e8c8d9f1`. `dispatch commit`
  + `dispatch hook-check` verbs, embedded coord pre-commit hook, `dispatch setup`
  install + doctor check #11 (`CoordHook`), skill/mechanics prose sweep. ISS-234
  closed. Verify green.

### Decisions (durable)
- **Hook asset → the existing `install/` RustEmbed root** (`install/git-hooks/pre-commit`),
  NOT a new root. **R2 (hollow nix binary) DISSOLVED** — no flake.nix graft; rides
  the already-grafted `srcWithDist`. Resolves the plan-phase "hook asset path" open item.
- **Selector completions** (sole-writer conformance, all forced in-objective, not
  scope creep): PHASE-01 `src/mcp_server/tools.rs` + `tests/e2e_mcp_server.rs`
  (MCP tree-state wiring + registry-count twin, forced by EX-1's MCP requirement);
  PHASE-02 `src/finding.rs` (doctor CoordHook category), `publication/manifest.toml`
  (SL-227 reachability entry for the install/ asset), `plugins/doctrine/skills/dispatch/**`
  (prose-sweep source). design §10 file-map was non-exhaustive on these wiring/twin files.

### Defects to reconcile / audit-notes
- **Skill-source selector defect (affects PHASE-06):** design-target selectors
  `.agents/skills/dispatch*/**` point at the UNTRACKED installed projection. The real
  tracked source is `plugins/doctrine/skills/`. PHASE-06 (rewrites dispatch skills)
  MUST target `plugins/doctrine/skills/{dispatch,dispatch-agent,dispatch-subprocess}/**`
  — `dispatch/**` now declared; declare `dispatch-agent/**` + `dispatch-subprocess/**`
  at that phase. Reconcile: correct/remove the stale `.agents/skills` selectors.
- **Bounded reversion-walk limitation (PHASE-02):** `hook-check`'s `funnel_run_base`
  walks the contiguous `FUNNEL_MARKER` prefix at HEAD; an interleaved non-funnel
  `dispatch commit` between funnel commits would break detection of a later reversion
  of a pre-that-commit funnel-advanced blob. Does NOT affect the ISS-234 flow
  (import-time HEAD is always the funnel tip). Documented bound.
- **CoordHook doctor severity = Warning** (not Error): doctor won't hard-fail on a
  pre-existing hookless coord worktree; re-running `dispatch setup` heals it.

### Operational (dispatch mechanics — for the next orchestrator)
- MCP server runs **PATH doctrine 0.29.0** (DOCTRINE_BIN unset at session launch;
  coord build is 0.31.0). 0.29.0 VERIFIED to carry all funnel mechanics (advanced-tip
  `merge_tree` compose, `SelectorIntent::DesignTarget` gating). Per boot.md DOCTRINE_BIN
  is a *close-time* concern; the drive proceeds correctly. Use `./target/debug/doctrine`
  (0.31.0) for ALL CLI verbs.
- **Working-tree-free MCP writes leave the coord worktree STALE.** `dispatch_import`
  and `dispatch_conclude_phase` mutate the ref object-db-only. After EACH, run
  `git reset --hard HEAD` in the coord tree to sync before the verify beat / next phase
  (the "M"/"D" entries are staleness, not real work — the delta is in HEAD + the fork).

### PHASE-03 — landed, decisions and deviations

**Landed.** `b8e64e10..73f11ccc` (base `b8e64e10` = orchestrator-authored;
import `73f11ccc`; concluded at `7ae2b617`). Verify beat clean; four in-scope
files only (`funnel_machine.rs`, `dispatch.rs`, `verify.rs`, `commands/check.rs`).

**D-P3-1 — `.doctrine/` artefacts must ride the phase BASE, not the worker
delta.** `classify_import` (`src/worktree/import.rs:146-153`) returns
`doctrine-touch` on any `.doctrine/` prefix *before* the selector leg, so no
`design-target` selector can admit one. And `tests/architecture_layering.rs`
`check()` is symmetric — a `[tiers]` row with no on-disk module is a
`StaleEntry` (line 594-601), a module with no row is `Unclassified` (581-588).
So neither "row first" nor "row after" is green. The only green-at-every-beat
ordering is one orchestrator base commit carrying the layering row + an empty
module stub + the mod declaration + the D7 artifact. **This generalises: any
future phase adding a new top-level module needs the same base beat.**

**D-P3-2 — the D7 artifact was authored from design §2, not generated from the
code.** D7's wording says "code-derived golden artifact"; the `.doctrine/` wall
inverts the authoring direction. Both sides derive from design §2's normative
table (which the design already says "the D7 artifact renders exactly"), and the
golden test enforces equality regardless of who typed it first — the worker's
first render matched byte-for-byte, unchanged. **Reconcile: soften D7's wording
to "spec-anchored, golden-pinned" rather than implying generate-then-commit.**

**Worker deviations adjudicated (all accepted; none blocked conclude):**
- *serde derive in the "std only" leaf* — accepted. Precedent is explicit in
  `layering.toml` (`boundary = "leaf" … pure serde`); crate out-degree stays 0
  and the ADR-001 gate is green. The alternative was a mirrored TOML shape in
  `dispatch.rs`, i.e. parallel implementation.
- *`attempt_advance` added alongside `attempt`* — accepted, **reconcile note**.
  The design's `attempt(current, t, facts) -> Result<Position, _>` cannot tell
  the sole writer act-from-replay, so landing a replay would mint a spurious
  commit per retry. `attempt` is kept verbatim as a thin projection of
  `attempt_advance` (one table, one authority) with a test pinning agreement.
  Design §2's signature should acknowledge the act/replay discriminator.
- *`conclude_allowed` treats `paths_since_verify: None` at a moved tip as
  `conclude-verify-stale`* — accepted (fails closed). Design left the `None`
  case unstated; **reconcile: state it.**
- *`preflight` is factless by signature*, so it cannot see replays —
  `preflight(Concluded, Conclude)` refuses `already-concluded` where `attempt`
  would replay. Consistent with the design's own signature; documented in-code,
  `attempt` remains authoritative at landing.
- *`already-<position>` also covers backward attempts*, not only mismatched
  replays; the D7 artifact's one-line gloss is narrower. Semantics-only —
  **reconcile: widen the gloss.**

**Defects surfaced (re-sighted on existing items; nothing new to file):**
- **ISS-028** — `commands/guard.rs:461` resolves the project root from CWD,
  ignoring `-p <temp root>`, so a worktree's worker marker refuses the authored
  writes ~8 e2e targets exercise. Consequence: **a worker's own `check gate` run
  is not a usable signal**, and `just validate` is a no-op in a fork
  (`justfile:36-40`), so the worker has no reliable local green at all — it
  burns a `worker_commit` round trip to find out. *(The CWD-vs-`-p` root cause
  was this phase's contribution; ISS-028 previously read it as inherent
  collateral damage with a workaround only. Briefly mis-filed as a new ISS-240,
  now closed duplicate.)*
- **ISS-219** — `worker_commit`'s `commit-gate-red` refusal payload embeds the
  **entire** ~400 KB test log rather than the failing assertions. Already filed
  from SL-206; this is a second sighting. Compounds with ISS-028: the worker
  cannot self-verify, so it *provokes* the refusal that then floods its context.
- The design's `.doctrine/spec/tech/021/**` design-target selector is **inert
  for import** (the wall pre-empts it) and governs slice conformance only — it
  reads as if a worker may write there. Same class as the `.agents/skills/**`
  mis-targeting already noted for PHASE-06. **Reconcile: correct both.**

**For the next spawn:** base-guard seam greps must be **grepped, not recalled**
from the design file map. This phase's prompt named `read_boundaries_at` in
`src/dispatch.rs` (it lives in `src/mcp_server/dispatch.rs:663`) and
`commit_on_behalf` in `src/git.rs` (it lives in `src/dispatch.rs:5113`). The
worker judged intent-satisfied and proceeded, correctly — but a wrong guard
either aborts a good phase or trains workers to ignore the guard.

### Close-time gate defect found at the PHASE-03 handover (ISS-241)

**`slice verify-vt 228` was exiting 0 while proving nothing.** The MCP
`dispatch_conclude_phase` reuses only `run_record_boundary`'s pure `BoundaryRow`
compute, not its live-file write, so the claude arm lands the committed
boundaries ledger and **skips the arm-neutral source-delta registry** in the
primary tree. `verify-vt` builds its `modified_files` set from that registry
(`src/slice.rs:838`), so with it empty every criterion short-circuits to
`Unattributable` (`src/vtgate.rs:124`) — visible but **non-halting** by design
(INV-4), hence exit 0.

The close ritual is "`verify-vt` — HALT on Fail", so this drive would have
reached close with a green, silent, signal-free gate.

Backfilled by hand from the committed ledger:
`slice record-delta 228 PHASE-NN --start <code_start> --end <code_end>` for all
three landed phases. Gate then reports **8/8 PASS**. Note `--commit <S>` does
NOT generalise here: only PHASE-03's boundary is a single commit — a phase whose
base carries orchestrator-authored `.doctrine/` content (D-P3-1) spans several.

**Every future phase on this arm needs the same backfill after conclude**, until
ISS-241 lands. Pi-arm analogue was ISS-052 (closed); sibling coord→primary
mirror gap was IMP-272/ISS-233 (resolved via the single writer `set_phase_status`).

### PHASE-04 — landed, decisions and deviations

**Landed.** `488300dc..fefa4d8f` (base `488300dc` = orchestrator-authored VT
rows + selector pre-declaration; import `e10248c6`; orchestrator follow-up
`fefa4d8f`; concluded at `9ed7f10c`). Verify beat clean (`✓ no new or changed
failures`). VT gate **16/16 PASS** across the four landed phases after the
ISS-241 record-delta backfill.

**D-P4-1 — the `Spawn` row is landed by the create-fork path, not `arm-spawn`.**
Design §4:377 and §10's `src/commands/` row assign it to `arm-spawn`. That is not
implementable on the claude arm: `SpawnRecord.fork` needs the fork name, and
`arm-spawn` runs *before* the fork exists (the harness assigns the worktree name
at `Agent`-spawn time; `dispatch arm-spawn` has no `--name` and cannot have one).
It also contradicts D8 — `arm-spawn` is strictly pre-act, and Class-2 records land
strictly post-act. **Reconcile: correct §4:377 and the §10 `src/commands/` row.**

Rejected alternatives, both for recorded reasons:
- *Never land Spawn; let import/worker_commit heal it.* Breaks the §6 oracle —
  rung 3 `await-worker` keys on position `Spawned`, so a phase whose worker is
  running would read as no-row and rung 4 would keep prescribing `spawn`.
- *`SpawnRecord.fork` becomes `Option<String>`.* `apply()` writes only the
  transition's own sub-record, so nothing would ever backfill it — a permanently
  empty provenance field.

**D-P4-2 — the landing routes through existing edges; no new module edge.**
A `worktree → dispatch` import would re-introduce the command-tier back-cycle
class SL-204 spent three phases breaking (`layering.toml:66`) — refused.

*Correction to the phase-plan reasoning:* the sheet's D3 asserted such an edge
would "pass the ADR-001 gate green". **It does not.** Probed empirically at
`fefa4d8f` (throwaway `use crate::dispatch::…` in `worktree/shared.rs`, gate run,
reverted): the gate fails `TangleGrew { tier: Command, baseline: 76, actual: 79 }`.
Assertion 2 (upward edges, `tests/architecture_layering.rs:630`) indeed does not
fire — `worktree` and `dispatch` are same-tier, and assertion 2 additionally skips
edges out of sub-classified modules, which `worktree` is. But `worktree` is
already inside the command SCC, so the new edge grows the **tangle ratchet**
(assertion 4, `count_tangle_edges`) and the gate goes red. So avoiding the cycle
was not merely governance hygiene — it was required to keep the build green, and
a worker who took the edge would have been stopped. The residual blind spot
(acyclic same-tier fan-out, sub-module coupling) is already filed as **RSK-227**;
no new item needed.

Instead `run_create_fork()` returns `CreatedFork{dir, spawn}`, and
`src/commands/cli.rs:1675` routes **only** the `CreateFork` arm to
`dispatch::run_create_fork_and_record`. `commands → dispatch` (cli.rs:1676) and
`dispatch → worktree` (dispatch.rs:27) already existed. Verified post-import:
`grep -rn "crate::dispatch" src/worktree/` → empty; `splice_record` still private
with one caller (invariant I1 holds).

**D-P4-3 — no base beat was needed.** `tests/architecture_layering.rs:178-182`
attributes `src/bar/baz.rs` to unit `"bar"`, so the new `src/worktree/claim_lock.rs`
resolves to the already-classified `worktree` unit: no `Unclassified` (:581), and
no row means no `StaleEntry` (:594). In-tree precedent: `worktree::create` and
`worktree::dispatch_record` carry no `[tiers]` row and the gate is green.
**D-P3-1's base beat is for a new *top-level* module only** — generalise it that
way, not to every new file.

**D-P4-4 — the claim lock is `rustix::fs::flock`.** No std API exists, and an
`O_EXCL` lockfile would reintroduce the stale-lease problem §3 deliberately
dissolves. Both `rustix` and `libc` are already in the **normal** (release) dep
graph (`cargo tree -e normal -i`: rustix ← crossterm, libc ← tokio/uuid/console),
so this adds zero new compiled crates; `rustix` won for the safe API over
`libc`'s `unsafe`.

**Worker deviations adjudicated (11 declared; all accepted):**
- *`worker_commit` at `position == None` refuses `not-spawned` rather than
  healing the `Spawn` prefix* — accepted, **and it weakens D-P4-5 below**. Design
  §3:329-331 asks for a row-absent self-record, but D8's one-commit heal rule
  needs a multi-transition splice and `land_funnel_transition` takes ONE
  `&Transition`. Generalising the sole writer is **PHASE-05's declared EX-2**;
  building it here would have duplicated that work and touched the sole writer's
  existing tests. Covered `spawned` (self-record) and `worker-committed` (replay).
- *jail-policy provisioning moved into the bind step* — accepted, an improvement:
  it previously sat after `fork_core` with no rollback, so a failed write left a
  live fork behind. Now under the claim and the compensating rollback.
- *`rollback_fork` also deletes the dispatch record* — accepted, forced by the
  reorder (bind-before-act would otherwise strand a record).
- *`bind_dispatch_record` split from `provision_dispatch_record`* — accepted.
  The fixup-revive path (§5.5) deliberately re-stamps a live record's `base`; a
  blanket no-clobber would have broken
  `worker_commit_wrong_base_revive_refuses_before_any_tip_moves`. Bind = spawn-time
  door (no-clobber); provision = re-stamp door.
- *`ClaimLock::drop` unlocks explicitly (`LOCK_UN`)* — accepted, a real find via a
  ~50%-reproducible flake. A child forked while the lock is held carries a
  duplicate descriptor onto the same open file description until it execs, so a
  close-only release lags. gc holds the claim across ~a dozen `git` invocations.
- *`late-recommit` is a new refusal token* — accepted. Design said only "refuse"
  for a dirty advanced fork; naming it keeps the worker's lost-response ambiguity
  ending in a self-describing terminal answer instead of sharing `not-at-base`.
- *`src/verify.rs` untouched* — accepted, **reconcile note**. PHASE-03's plan
  objective says `run_suite` is "needed by PHASE-04's belt-gated adoption", but the
  self-record leg correctly reuses `run_commit_gate`: `run_suite` *inherits* stdio
  and so cannot produce the captured output `commit-gate-red`'s `detail` carries.
  VT-8's own keywords (`classify_scope`, `run_commit_gate`) confirm the intended
  seam. **Reconcile: drop the run_suite claim from PHASE-03's objective.**
- *`worktree fork --slice/--phase` binds conditionally* (only with `--worker`,
  both flags, and a `<coord>/.worktrees/<name>` layout) — accepted: a record
  outside that layout is unreachable by the resolver, so writing one would be
  unreachable state.
- *fixture upgrade + mechanical signature adaptations at existing call sites* —
  accepted; **every pre-existing assertion unchanged** (S3 respected).

**D-P4-5 — the degraded Spawn-landing path is NOT self-healing until PHASE-05.**
Phase-plan D4 made a failed `Spawn` landing non-fatal on the reasoning that
"import heals it". With the deviation above, that is **not yet true on the claude
arm**: position `None` ⇒ `worker_commit` refuses `not-spawned` ⇒ no fork commit
exists ⇒ import has nothing to heal from. Recovery today is fallback (A)
(live-worktree import), not the funnel. PHASE-05's multi-transition splice closes
it — **re-check this when EX-2 lands**, and until then treat a `Spawn`-landing
warning on stderr as a halt-and-triage signal, not a benign note.

**Orchestrator follow-ups landed inside the boundary (`fefa4d8f`):**
- `src/mcp_server/tools.rs:342` — the `worker_commit` tool description enumerated
  the pre-PHASE-04 refusal set. Added `unprovable-fork`, `late-recommit`, and the
  `already-<position>` family. No test pins the enumeration, so it was silent doc
  drift. `install/dispatch-mechanics.md` carries the same list and is PHASE-06's
  declared work.
- VT-5's keyword retargeted `provision_dispatch_record` → `bind_dispatch_record`
  (the seam the deviation created). Criterion id and `expects` unchanged.

**Selector completions:** `src/worktree/{create,fork,gc,dispatch_record,mod,claim_lock}.rs`,
`Cargo.toml`, `Cargo.lock` promoted to design-target at the phase base
(`src/worktree/**` had been scope-relevant only, so every core file the phase
touches would have returned `undeclared`). The new module's name was **pinned at
phase-plan** (`claim_lock.rs`) precisely so the selector could be exact rather
than a glob.

**For the next spawn — the runtime phase sheet does NOT reach the worker.**
`.doctrine/state/` is gitignored runtime tier, so a fork minted from a commit
carries none of it; the worker reported the sheet absent and worked from the
prompt alone. **A worker prompt must INLINE the decisions/tasks/STOP conditions it
needs — never cite the sheet by path.** This drive survived it only because the
prompt happened to carry D1–D6 and the scope list in full.

### PHASE-05 — landed, decisions and deviations

**Landed.** `a09063e8..84de53ab` (base `a09063e8` = the PHASE-04 harvest tip — **no
base commit was needed**: every file the phase touches was already `design-target`
and it adds no new top-level module; import `84de53ab`; concluded at `2b1810ab`).
Verify beat clean (`✓ no new or changed failures`); `check prove` clean at the
imported tip. VT gate **20/20 PASS** across the five landed phases. Delta: 10 files,
+3199/−262; `dispatch.rs` tests 112 → 129, **all additions — no pre-existing
assertion removed** beyond the sanctioned registry-count bump.

**Decisions pinned at phase-plan (D-P5-1 … D-P5-7)** — the full text is in the
disposable sheet; the ones that changed the build:

**D-P5-1 — the sole writer generalises on two axes in one signature.**
`land_funnel_transitions(… ts: &[Transition], base_tree: Option<&str>, message …)`
with the singular form kept as a one-element projection. All-or-none over the fold.
This is what makes Class-1 conclude (`boundaries.toml ⊕ funnel.toml` in one tree),
import's one-commit heal, and the D-P4-5 closure all the *same* mechanism rather
than three. The worker went one better and lifted the fold into a **pure**
`funnel_machine::fold_transitions`, keeping the impure half in the writer.

**D-P5-2 — the stale baseline is derived EMPIRICALLY, not from marker topology.**
Design §5 step 2 defines it as "the pre-advance tree the checkout last materialized"
and then *sketches* a topological finder ("parent of the oldest not-yet-materialized
funnel-provenance commit"). The sketch is **unsound here**: a commit is materialized
iff the index carries its tree, so the baseline is the newest first-parent-chain
commit accepted by `git diff-index --quiet --cached`. The topological version
overshoots whenever the tree was last synced *at* a funnel commit — which the
orchestrator's own `git reset --hard HEAD` produces every single phase — and every
path materialized at that commit then reads as operator divergence, i.e. a **false
`verify-tree-dirty` on the happy path**. Bonus: the empirical derivation discharges
the index leg of the per-path proof gate wholesale, leaving only the worktree leg.
**Reconcile: replace §5 step 2's parenthetical derivation with the index-identity
one.**

**D-P5-3 — the row-present/row-absent split.** Design §4's last row says
`record-boundary` "routes through `land_funnel_transition(Conclude)` — same
authority, no bypass". Read absolutely it refuses every solo / pre-funnel / legacy
phase `not-spawned`. Pinned: **row present ⇒ the machine; row absent ⇒ today's
path**. No bypass either way — where the funnel governs, the gate applies; where it
does not, there is no position to bypass, and §9's boundary-ahead-of-authority alarm
is the backstop.

**D-P5-4 — only `dispatch_phase_receipt` supplies `Some(position)`.**
`run_status`/`phase_projection` pass `None`, so R1 holds by construction rather than
by care.

**D-P5-5 — "the suite did not run" is a refusal (`verify-suite-unresolved`), not red
evidence.** Red evidence means the suite ran and failed.

**D-P5-6 — import resolves its phase from the durable fork binding**
(`resolve_agent` → `require_binding`), which is exactly what PHASE-04 added
`DispatchRecord.slice/.phase` for. Unbound ⇒ `unprovable-fork`.

**Worker deviations adjudicated (5 declared; all accepted):**
- *D-P5-3's split extended from `record-boundary` to `dispatch_conclude_phase` and
  `dispatch_reap`* — accepted, and **the plan's fault, not the worker's**. The tasks
  read as unconditional gates while STOP-1 forbade editing tests that drive both
  verbs with no funnel row; the two were jointly unsatisfiable. D-P5-3's own stated
  rationale is exactly this case. Verified independently against
  `dispatch_conclude_phase_flips_the_gitignored_sheet_and_lands_the_boundary`.
  **Reconcile: §4's gate table needs the row-present qualifier on conclude and reap,
  not only on `record-boundary`.**
- *reap finds its row by `spawn.fork == name`, not `resolve_agent`* — accepted,
  forced: idempotent completion must work when the fork worktree is already gone,
  which the agent resolver cannot see. No second record read introduced.
- *`import_compose` became `#[cfg(test)]`* — accepted, **reconcile note**. D-P5-6
  makes the gated heal-forward the only production import path, orphaning the
  ungated compose+commit driver that four pre-existing tests drive. The worker made
  it a 12-line test-only wrapper over the **shared** production `import_plan`, so the
  belt/compose/provenance under test are still production code. The plan should have
  said what the old path's callers become. **Reconcile: give `import_compose` a
  stated role or re-point its four tests at `import_plan` + `dispatch_import`.**
- *registry count 26 → 27* — sanctioned maintenance, the twin was named in the task.
- *`expect(dead_code)` removed from `Transition`* — forced: PHASE-05's write verbs
  are now the non-test constructors, so the expectation became an
  `unfulfilled-lint-expectations` hard error. This is the predicted end of the
  staged-predicate pattern, working as designed.

**Conclude got better than pinned:** the sheet flip is not merely trailing, it is
**conditional on a landed conclusion** — a refusal can never leave the sheet
claiming a completion nothing backs. Not asked for; kept.

**T10 landed, not deferred.** Both `worker_commit` legs now heal a missing `Spawn`
rung. **D-P4-5 is closed**: a failed `Spawn` landing is no longer a dead end on the
claude arm, and a `Spawn`-landing warning on stderr is no longer a halt-and-triage
signal.

**Defects filed (checked against the backlog first — neither was covered):**
- **ISS-243 — `dispatch_verify` over MCP corrupts the JSON-RPC stream.**
  `src/mcp_server/mod.rs:44-45` writes JSON-RPC on stdout; `run_suite` spawns with
  `.status()`, which inherits it. Every real suite prints, so this fires on the
  happy path. **Not worker error** — design §5 step 3 pins the inherited-stdio
  runner *and* mandates the MCP mirror; the two conflict, and neither design nor
  phase-plan caught it. The **CLI arm is unaffected** and is the interim workaround.
  Verified independently against both seams. **Reconcile: §5 step 3.**
- **ISS-244 — pre-existing flake in PHASE-04's claim race**
  (`two_concurrent_same_name_spawns_leave_exactly_one_winner`, winner's binding
  intermittently `None`, ~1 in 10 full runs). Nothing in the PHASE-05 delta touches
  `create.rs`; surfaced by repetition, not regression. Consequence is terminal for
  that fork (`unprovable-fork` refuses both `worker_commit` and import), so it is
  worth a real reproduction before a speculative reorder.

**Not covered by tests (declared):** the `dispatch verify` CLI arm's own shell
(`run_verify` — argument plumbing, JSON print, exit code); and nothing exercises
`dispatch_verify` over the real JSON-RPC wire (registry presence only). The engine
beneath both surfaces is fully covered. Given ISS-243, the wire gap is the more
interesting of the two.

**Selector completions: NONE.** First phase of this slice needing no
pre-declaration — every touched file was already `design-target`, and
`undeclared: []` came back from the commit gate. `src/commands/guard.rs` is in scope
via `src/commands/**`; its one-line change is a *forced* exhaustive-match arm for the
new `DispatchCommand::Verify` (correctly classed `Orchestrator("dispatch-verify")`),
not an ISS-028 "fix".

### PHASE-06 — Oracle + skills: `dispatch next` and the verb-driven loop

**Landed** `fbbe4f28..a278741a` (base beat `fbbe4f28` = two prose selectors;
worker commit `96dca2dd`; import `367d76cd`; verify `a278741a`; conclude
`d837b840`; reap `56e6892d`). VT-1 PASS; S1 `✓ no new or changed failures`;
`check prove` green. Slice-228 VT gate now **21/21 PASS** across PHASE-01…06.

**This was the first phase driven through the armed funnel.** Slice 228 carried
no `funnel.toml` until PHASE-06 minted its first row, so PHASE-01…05 all landed
row-absent (D-P5-3's ungated split). Every gate the slice built was exercised in
anger here for the first time — which is where three latent `reap` defects fell
out (below).

**Decisions pinned at phase-plan (D-P6-1 … D-P6-9)** — implemented as pinned. The
load-bearing three:

**D-P6-1 — `kind` is a projection of `expected_next` with exactly ONE
position-derived discriminator.** `next_kind(Expected, Option<Position>)` is a
total `const fn` over `Expected`; the `Verify` arm is the *only* arm that reads
the position (`Verified ⇒ reverify-stale`, else `verify`), because the
`Expected → NextKind` map is not a bijection. A test pins every other arm as
position-invariant across all six positions plus `None`. There is no second
table and no `match position` over the funnel.

**D-P6-2 — ladder totality is a theorem, not a fallback arm.** For a mid-funnel
row `expected_next` can only return `Triage | Import | Verify | Conclude | Reap |
AwaitWorker`: `Expected::Spawn` needs `current == None` (no row ⇒ not mid-funnel)
and `Expected::Terminal` needs `Reaped` (excluded by construction). So rung 1
takes `Triage`, rung 2 the four runnable verbs, and every survivor at rung 3 is
`AwaitWorker`. `select_next` ships with **no `_ =>` arm** and the argument in its
doc comment.

**D-P6-4 — `command` is the runnable literal in the surface that OWNS the verb,
fully parameterised from `(slice, coord_tip, row)`.** CLI line for `verify`;
`tool{args}` for the MCP-only `import`/`conclude`/`reap`. `conclude`'s
`code_start` is read out of the committed `spawn` row — which retires the
handover's bolded "always capture `B` fresh, never reuse a number from this
file" as a class of orchestrator error. An unrenderable field yields
`command: None` plus the reason, never a half-rendered literal.

**Accepted deviations (all disclosed by the worker):**
- *D-P6-5 — `spawn` carries no command literal*, as pinned. Spawn mechanics are
  arm-specific and the oracle is deliberately arm-agnostic. Shipped command-less
  set is **four** kinds (`spawn`, `await-worker`, `triage-verify-failure`,
  `all-reaped`); design §6 names two. **Reconcile: widen §6's list.**
- *`src/commands/guard.rs` touched* — a new `DispatchCommand` variant breaks its
  exhaustive match; classified `Read` (read-only, worker-safe). In the declared
  `src/commands/**` selector, so no scope escape.
- *Three `TOOL_DISPATCH_*` consts relocated* from `src/mcp_server/dispatch.rs`
  (command) to `src/dispatch.rs` (engine), re-exported via `pub(crate) use`.
  `next_command` must render those names and an engine → command import is an
  upward edge the layering gate would red. The alternative was three magic
  strings (STD-001). Layering gate green; every existing reference resolves
  unchanged. **General rule worth promoting: shared vocabulary lives at the
  lowest tier that needs it; the command tier re-exports.**
- `src/funnel_machine.rs` **untouched** — no visibility widening was required.

**VA-1 verified by the orchestrator, not accepted on claim.** (i) Prose sweep:
every `rescue|auto-heal|escape hatch|by.hand|git rev-parse|git diff` survivor
inspected in context — all are either keep-list preconditions (the base capture,
worktree-isolation check, the subprocess arm's `S`-pinning) or *prohibitions*
pointing at `dispatch next`. (ii) Code shape read directly: `expected_next` is
the sole prescription source, `compute_next_phases ∘ plan_next_rows` the sole
readiness source, no second table, no second position match. Zero tests deleted
(the only removals are the 27 → 28 registry count updates).

**The oracle drove its own landing.** From `imported` on, every beat was chosen
and parameterised by `dispatch next` — `verify` → `conclude` → `reap` → `spawn
PHASE-07`. Live evidence that EX-1/EX-2/EX-3 hold outside the test suite.

**Defects filed (backlog checked first — neither covered):**
- **ISS-245 — the patch-id landed-oracle refuses every funnel-managed fork.**
  PHASE-05's atomic one-commit import composes the `funnel.toml` `Import` row
  into the same commit as the worker delta, so the import commit's patch is a
  strict superset of the fork's (`10 files/1274+` vs `9 files/1267+`) and
  `git cherry` reports `+`. Confirmed empirically, not inferred. **A zero-rescue
  regression in the slice whose thesis is zero rescue** — and precisely the
  "`--force` reflex, safety gate collapses" failure `dispatch-mechanics.md` cites
  as the reason patch-id was chosen over delta-emptiness. Third facet: `gc
  --force` lands no `Reap` row, stranding the position at `concluded`; the
  recovery (re-issue `dispatch_reap`, whose gc no-ops on the absent fork) works
  only by luck of idempotence. Preferred fix: **ask the funnel, not git** — a
  phase at `concluded` with `spawn.fork == <branch>` is landed by definition.
- **ISS-246 — `dispatch_reap` over MCP returns opaque `-32603 Internal error`.**
  Its own tool doc pins `not-landed` as a hard `Err`; MCP flattens that to
  `-32603` and drops the CLI's diagnosis *and* its remedy. Siblings
  (`dispatch_import`, `dispatch_conclude_phase`) model refusals as
  `Refused{reason, detail}` data. Direct hazard for PHASE-07: a memory-blind
  orchestrator gets nothing actionable and has no memory telling it to re-run the
  CLI arm.

**ISS-243 deliberately NOT ridden (D-P6-8).** Three reasons: the fix contradicts
a *pinned* design line (§5 step 3) already on the reconcile queue as
irreconcilable; `verify_phase` is shared verbatim by both surfaces so a stdio
mode ripples through a shared seam (R1); and **D-P6-4 routes around it by
construction** — `next` prescribes the CLI literal for verify, so the corrupting
MCP surface is unreachable on the prescribed path.

**Reconcile queue additions:** design §6's command-less list (two → four kinds);
`src/doctor_checks.rs`'s `DISPATCH_READ_TOOLS` allowlist is pinned as "EXACTLY
the three read tokens" and omits `dispatch_next`, so a confined probe/orchestrator
agent-def granting it would be flagged by `doctor` (reported by the worker, not
acted on — outside its declared set and the pin is test-locked).

### Plan amendment — PHASE-08 inserted before PHASE-07 (user-approved)

User decision after PHASE-06: fix ISS-245/ISS-246 **before** the OQ-5 benchmark,
because both fire on the prescribed path of the very loop PHASE-07 measures — a
memory-blind orchestrator would spend its one run measuring a known defect.

Mechanics worth remembering: **ids are immutable, order is plan order.** The new
phase keeps id `PHASE-08` and is positioned *between* PHASE-06 and PHASE-07 in
`plan.toml`, so nothing is renumbered and the readiness authority
(`compute_next_phases`, which scans in plan order) prescribes it next. Confirmed
live — `dispatch next --slice 228` → `spawn — PHASE-08`. PHASE-07 gains `EN-2`
naming PHASE-08 as its entrance.

### Design amendment before PHASE-08 — D9/D10, reviewed as RV-308

**Why design and not the reconcile queue.** PHASE-08's fix contradicts two *locked*
lines — §2's `Concluded → Reap` facts gate ("landed-oracle ok (engine-side)") and
§4's `dispatch_reap` row ("landed-oracle via `run_gc`"). The queue's thirteen items
are descriptive corrections to prose about what was already built; this was forward
intent that changes a gate. It was first pinned only in the gitignored phase sheet —
including the argument for authorising an irreversible `branch -D` on a non-git
proof. Wrong sink for a load-bearing safety argument. Amendment at `3da092c2`,
repairs at `e2ea2541`.

**D9 — reap landing authority.** For a funnel-recorded fork the record replaces git
archaeology, but **position alone is not the proof**. The injected fact is a
conjunction: exactly one row matches the branch (ambiguous refuses), that row is at
`concluded`/`reaped`, **and the live branch OID == that row's `import.fork_tip`**.
Any check failing injects nothing and `git cherry` decides — fail-closed. Computed
by a command-tier **landing-authority resolver** (a named seam, not an inlined
predicate) and handed to engine-tier gc as an already-proven fact; gc never imports
the funnel (ADR-001 — probed at `fefa4d8f`: `TangleGrew{Command, 76→79}`).

**D10 — actionable verdicts are data.** `Refused{reason, detail}` carrying the CLI's
remedy verbatim; `Err` (MCP flattens to `-32603`, message dropped) for internal
faults only. Discharged for `dispatch_reap`; a **rule with one verb proven**, not an
invariant across four — the sibling table is VA-1's output, landing at reconcile.

**RV-308 — the blocker that justified the round (F-1).** The first draft authorised
`branch -D` on fork name + position alone. Two live failures: a fork whose branch
advanced past its imported tip, and a **recycled fork name** binding a new branch to
an older concluded row (`row_for_fork`, `src/mcp_server/dispatch.rs:785`, returns the
*first* `spawn.fork` match). Either deletes unimported commits — the exact harm the
patch-id oracle exists to prevent. The tip-binding conjunction is the repair.

Other findings worth carrying (full text + dispositions on RV-308):
- **F-3** — a Class-2 `Reap` whose row-landing CAS loses returns bare `Reaped`
  today (`dispatch.rs:757`); the shipped driver (`install/workflows/drive-slice.js`)
  proceeds on any non-refusal while `next` still prescribes reap. Remedy amended
  against codex's ("never return `Reaped`") because that contradicts D8 — the fork
  *is* gone; the two facts must be reported distinctly, not collapsed.
- **F-4** — the residue model is a **total typed outcome table** (§4.1): `Busy`
  (claim lock) refuses and must not advance; `CriticalResidue` refuses;
  `AdministrativeResidue{dispatch_record}` advances **and** reports, because blocking
  would strand the funnel behind a fault retry cannot clear.
- **F-5** — `landed_against`'s only external caller is `src/worktree/inventory.rs:249`
  (`worktree list`'s landed column). Until it adopts the resolver, a funnel-landed
  fork renders `NotLanded` there — a published contradiction, not a UX wrinkle. The
  `--no-landed` filter is unaffected (it suppresses the oracle).
- **F-7** — PRD-015's reap requirement and SPEC-021:205 are mechanism-neutral and
  D9 **satisfies** them (durable vs disposable is exactly their distinction).
  `spec-012.md:106` names `git cherry` as *the* mechanism and genuinely contradicts;
  the D7 golden artifact (`.doctrine/spec/tech/021/funnel-machine.md:20`) still
  renders the old gate and regenerates with the code.

**New invariants:** **I4** corrected (reap reaches `reaped` iff gate passes ∧ outcome
is advance-eligible ∧ **the row lands**); **I5** added — the coordination ref is
append-only under CAS, never rewritten; D9's soundness rests on it, so if it is ever
relaxed D9 must be re-derived, not inherited.

**Reconcile queue additions:** SPEC-012's `git cherry` sentence needs the ship-time
sibling REV; D10's three-tool fallibility table lands in design §1/§4.1 at reconcile;
NEW-OQ-C (resolver adoption by CLI gc + inventory) stays open.

### RV-308 closed — the D9/D10 repair surface reviewed to exhaustion

Four rounds, all findings terminal. Round 1 disposed 7/7; rounds 2–4 were the
raiser verifying the repairs, which is where two of them failed. Design commits:
`74884577` (F-4/F-5 round 2), `a3b8e5c8` (F-4 round 3), `463b512a` (cost
correction). Ledger commits on the **edge** line (`1aa60b41`, `f394421f`,
`e507d3f4`) — RV-308 lives in the primary tree's corpus, not on `dispatch/228`;
read it with `review show RV-308 -p /workspace/doctrine`.

**Closed cleanly in round 2:** F-1 (no fourth route to a false landing proof past
the conjunction), F-3, F-7. **F-2** closed with the argument *strengthened by the
reviewer*: I5 is architectural, not merely "CAS prevents rewrites" —
`commit_on_behalf` constructs each commit with `expected_old` as its **parent**
before the CAS advance (`src/dispatch.rs:5478`), and the funnel writer uses that
primitive exclusively (`:5724`). Worth keeping: that is the citation to reach for
if I5 is ever challenged. **F-6** closed-but-weakened, accepted as such.

**F-4 took three rounds and is the substantive one.** Round 1's "total typed
outcome table" was not total, twice over:
1. *Residue is a set, not a variant.* `run_gc_to` accumulates `leftovers` as a
   `Vec` across three independent legs (`gc.rs:404` worktree, `:416` branch,
   `:431` dispatch record) and folds them into ONE `gc-incomplete` bail, so
   critical and administrative residue co-occur — a permissions fault blocking
   `worktree remove` typically also blocks the record delete beneath it. Rule now:
   advance iff the set contains neither `worktree` nor `branch`; critical outranks
   administrative; a mixed set refuses **and reports every member**.
2. *Reporting is not an outcome.* The `Busy` notice (`gc.rs:280`) and — the real
   defect — the post-delete stderr warning (`:443-446`) and success line (`:448`)
   are `?`-propagated, firing **after** both deletes succeed. A failed write turns
   a completed reap into an `Err`. **This is F-3's harm arriving through a
   different door, found under F-4 after F-3 had already been verified closed.**
   The typed entry point therefore returns the verdict as data and drops
   report-write errors; `Busy` is returned, not printed (the MCP caller passes
   `io::sink()` and cannot observe a print).
   *Generalisable:* verifying a finding closes the instance, not the class.

One F-4 item was **contested and won on source evidence**: dry-run's collapse of
would-reap and would-refuse into `Ok(())` is real in the CLI but unreachable from
the funnel (`dispatch_reap` passes `dry_run: false, force: false` at
`src/mcp_server/dispatch.rs:726`, `:754`). Rather than rest on call-site
discipline it is pinned in the type — `reap_fork` takes **neither flag**.

**F-5 was the interesting failure.** Round 1 adopted the reviewer's *structural*
remedy (ship the shared seam) while dropping the finding's *stated minimum* (that
inventory stop publishing a false unlanded). Different claims — a seam nobody
wires un-publishes nothing. `worktree::inventory` now adopts the resolver
in-phase; only CLI `worktree gc` defers, and that deferral has a real basis (it
derives no slice, so adoption needs a cross-slice scan of
`.doctrine/dispatch/*/funnel.toml`). NEW-OQ-C narrowed to that one consumer.
`src/worktree/inventory.rs` added as a design-target selector.

**Cost correction found at phase-plan (D-P8-10).** §4.1 first claimed inventory
adoption was "a substitution at one call site". The semantic change is; the
plumbing is not. **`worktree = command`** in ADR-001's map (`layering.toml:117`)
and `dispatch → worktree` already exists, so `src/worktree/mod.rs` may no more
import `crate::dispatch` than `gc.rs` may — the same 2-cycle S1 forbids, which
had been read as a `gc.rs`-only constraint. Legal injection point is
`src/commands/cli.rs:1683` (already imports `crate::dispatch`); the resolver
reaches `landed_cell` as a bare `&dyn Fn(&Path, u32, &str) -> Option<bool>`
threaded through four signatures. A *callback*, not a pre-resolved map, because
rows are discovered inside `run_list` by `git::list_worktrees`. Deliberately a
closure and not a trait object from `dispatch`, so no `crate::dispatch` name ever
appears in a `worktree` signature. Decision unchanged; cost was understated.

**Also pinned at phase-plan:** S6 — a row at `concluded` with `import == None` is
an integrity alarm, halt and report, never a fall-through. And `dispatch_reap`'s
doc comment at `src/mcp_server/dispatch.rs:700` ("the CLI gc and the MCP verb
agree on the landed verdict") becomes false the moment D9 lands — **in-phase
correction, not a reconcile item**.

**Reconcile queue additions (unchanged from the amendment, restated):** SPEC-012's
`git cherry` sentence rides the ship-time sibling REV; D10's three-tool fallibility
table lands at reconcile (VA-1's output); NEW-OQ-C stays open for CLI `worktree gc`;
the D7 golden artifact regenerates with the code.

### PHASE-08 — landed, decisions and deviations

Reap oracle + refusal shape (ISS-245, ISS-246). Driven on the claude arm.
Worker delta `f6216d1e` (base `1c694996`) imported at `b5bfd33e`, coord-side
`gate` verify green at `e7153ad6`, concluded at `f9d81b5c`, fork reaped,
`record-delta 1c694996..e7153ad6` recorded. Plus the post-conclude remediation
`ef2d3e81` (see *A landed phase has no re-drive path*, below).

**Landed.** All fourteen tasks, none deferred. `funnel::resolve_landing` +
`LandingVerdict{Landed, NotProven, Ambiguous, NoRow}` in `src/dispatch.rs`'s
`funnel` module; the single fork⇄row predicate is `FunnelRecord::rows_for_fork`
(an iterator, so the resolver gets the count without a second copy of the
predicate) and `mcp_server::dispatch::row_for_fork` delegates to it. `GcState`
gains `funnel_landed`; `classify_gc` consults it before the git verdict and
`gather` skips the `git cherry` read entirely when it holds (EX-1's "first" is
literal). `gather`/`classify_gc`/`act` factored into one core under two shells —
typed `reap_fork` (no `force`, no `dry_run`, no writer) and byte-compatible
`run_gc_to`/`run_gc`. `FunnelOutcome::ReapedRowPending{fork, detail}` added;
`reap_verdict` and `land_reap_row` factored out. `inventory::LandingOracle`
threaded `cli.rs` → `worktree::dispatch` → `List` → `run_list` → `resolve_row` →
`landed_cell`. Tool description and `install/dispatch-mechanics.md` narrowed.

**Behaviour preservation held.** `tests/e2e_worktree_gc.rs` green **unmodified**
(14 passed) — the goldens are token-keyed, so extending the `not-landed` message
with the funnel signpost was safe. `landed_cell_tokens_are_stable` unmodified.
`architecture_layering` green with the tangle baseline intact at `command = 76`,
so S1 held: no `use crate::dispatch::…` anywhere under `src/worktree/`.

**Worker deviations (three, all disclosed, none silent).**

1. **S6 realised as an explicit integrity fault, not a halt.** A row at
   `concluded` with `import == None` is only reachable against a corrupted
   record, so halting would have made the phase unfinishable. `resolve_landing`
   `bail!`s naming slice/phase/position rather than degrading to the git oracle.
   Consistent with T11's "`Err` is reserved for internal faults". If a fourth
   verdict token is wanted instead, it is a one-line change plus a
   `Refused{funnel-record-integrity}` arm — **left as an option, not taken.**
2. **Administrative-only residue reports on stderr, not in the structured
   outcome.** `FunnelOutcome::Reaped{fork}` has no detail field and widening it
   would change the wire shape and three existing equality assertions. The fork
   is gone and the row landed, so `Reaped` is the honest outcome; the stale
   dispatch record is named on stderr, never stdout.
3. **`classify_gc_dry_run_does_not_change_the_verdict` retired**, replaced in
   place by a comment. Dropping the `_dry_run` parameter (instructed by T6) makes
   its assertion unrepresentable — the classifier can no longer *see* `dry_run`,
   so the property became structural rather than testable. Coverage of the
   dry-run path is unaffected: two `e2e_worktree_gc` cases plus a new pure
   `dry_run_basis` cell pinning all four basis arms including the funnel one.

**Accepted cost.** `dispatch_reap` now reads the funnel record twice — once
inside `resolve_landing`, once for `row_for_fork`. Two object-db reads per reap,
traded for one named seam rather than threading a pre-read record through the
pinned resolver signature.

**D-P8-7 was live, not cosmetic.** Both old gc call sites wrote the human reap
report to **process stdout**, which under the MCP stdio transport *is* the
JSON-RPC wire. It had never fired only because the two bugs cancelled: the
fork-present path always errored first (ISS-245) and the fork-absent path skipped
gc. **Fixing ISS-245 alone would have manifested ISS-243's failure mode on a
second verb** — the next successful funnel reap would have emitted a bare line
into the stream. `reap_fork` having no `out` parameter removes it by
construction.

#### VA-1 — dispatch MCP `Err`-path sweep (T13, report not repair)

Survivors: an `Err`/`?` path a caller is expected to act on.

| Tool | Site | Trigger | What the caller should have been told |
|---|---|---|---|
| `dispatch_conclude_phase` | `src/mcp_server/dispatch.rs:633` (`set_phase_status(…)?`) | The **durable** conclusion already landed (boundary ⊕ position committed); the *trailing* gitignored sheet projection then fails (permissions, read-only state dir, mirror write into the primary tree). | `Concluded{coord_tip}` **plus** a sheet-projection-pending signal. Today the caller gets `-32603` and cannot tell the boundary DID land — the exact inverse of the documented "only fault outcome is a completed sheet with no committed boundary". This is D10's known instance. |
| `dispatch_verify` | `src/verify.rs:286` (`Command::new(program)…status()`), reached via `src/dispatch.rs:6176` | The suite child **inherits fd 1**, so its stdout is written onto the MCP stdio JSON-RPC stream. | Nothing should reach fd 1; the suite's output belongs in a captured buffer folded into `VerifyFailed{detail}`. **ISS-243 — reported, deliberately NOT ridden (S5).** Needs `.output()` / explicit `Stdio::piped()` plus a decision on where the captured text goes. |
| `dispatch_verify` | `src/dispatch.rs:6213` (`load_doctrine_toml(coord_root)?`) | Unparseable/missing `.doctrine/doctrine.toml` in the coord tree. | A typed refusal in the `verify-suite-unresolved` family naming the config file. An unresolvable *suite* is already typed; the config **read** is not. |
| `dispatch_authored_divergence` | `src/mcp_server/dispatch.rs:1164` (`trunk_commit(…)?.context(…)?`) | No trunk ref resolves (`DOCTRINE_TRUNK_REF` / `origin/HEAD` / `main` / `master` all absent). | A typed `trunk-unresolved` refusal — plainly actionable ("point `DOCTRINE_TRUNK_REF` at the intended local trunk"), sibling of the existing `verify-suite-unresolved`. |
| `dispatch_next_ready` | `src/mcp_server/dispatch.rs:1127` (`plan_next_rows(…)?`) | No plan yet, or unparseable `plan.toml`. | `Refused{plan-unresolved}` — remedy is `slice plan` / repair the plan. A read-only prescription verb erroring out is indistinguishable to the driver from a server fault. |
| `dispatch_next` | `src/mcp_server/dispatch.rs:1221` (`next_core(…)?`) | Same class (the oracle reads the plan and a changed-path set). | Same typed refusal. **Highest impact** — `dispatch_next` is the verb the shipped driver polls to decide what to do next. |
| `dispatch_import` | `src/mcp_server/dispatch.rs:360` (`slice::selectors(…)?`) | Coord tree's slice missing/unparseable, or its design-target selectors unreadable. | `Refused{selectors-unresolved}`. The `?` is *correct* to refuse silently disabling the scope gate; the shape is wrong. Remedy is to author/fix the slice. |
| `dispatch_import` | `src/mcp_server/dispatch.rs:319` (`git_text(rev-parse <name>)?`) | The `name` branch does not resolve (typo, already-reaped fork). | `Refused{unknown-fork}`. **Low severity / near-unreachable** — `resolve_agent(…, ForkExpect::Advanced)` refuses `unknown-agent` first. Listed for totality. |
| `worker_commit` | `src/mcp_server/worker_commit.rs:457`, `:569` (`Belts::gather(…)?`) | Missing/unparseable slice or `doctrine.toml` in the **worker's** tree. | A typed refusal in the belt family. A worker given `-32603` cannot tell a belt problem from a broken server, and its role prompt tells it to report refusals *verbatim*. |
| `worker_commit` | `src/mcp_server/worker_commit.rs:474`/`:574` → `:149` ("resolved commit-gate argv is empty") | `[verification]` names no runnable `commit` cadence. | A typed `gate-unresolved` refusal — exact analogue of `verify-suite-unresolved`, and a config error the operator fixes. |
| `worker_commit` | `src/mcp_server/worker_commit.rs:480` (`stage_and_commit(…)?`) | `git add`/`git commit` fails **after** a green gate (hostile pre-commit hook, locked index, full disk). | The worker needs to know the gate passed and the commit did not. **The one post-gate window with no structured shape.** |

Judged **correct as `Err`** (not survivors; listed so the sweep is total): every
`read_funnel_at` / `read_boundaries_at` `?` (an unparseable committed record is a
record-integrity fault, not a caller decision); `dispatch_tree_state:1197`
(`tree_clean_untracked` — git plumbing); `reap_fork`'s root resolution and gather
reads; and `resolve_landing`'s deliberate integrity `bail!` (deviation 1 above).

**VA-1 verified by reading the code, not by accepting the hand-back** (PIN 3):
the two flipped tests, the `not-landed` signpost text, the three-check
conjunction in `funnel.toml`, and the fresh binary's refusal message were each
checked directly.

#### The two pre-authorised test flips (D-P8-8) — confirmed

Both moved from `unwrap_err()` to a `Refused` match; neither deleted, both
strengthened. `a_fork_with_no_funnel_row_keeps_the_legacy_ungated_reap` now
asserts `Refused{reason: "not-landed"}`, that `detail` names the fork and carries
the single-sourced remedy, and — new — that the unlanded branch **survives** the
refusal. `dispatch_reap_reaps_a_landed_fork_and_refuses_an_unlanded_one` asserts
the same shape; its pre-existing "the unlanded fork branch survives" assertion is
unchanged and its landed half untouched. The verdict is identical in both; only
the shape changed, and each carries a comment saying so, so a future reader does
not mistake it for a loosened gate.

#### A phase that fixes an MCP verb cannot use the fix in its own session

The MCP server is launched once at session start from `${DOCTRINE_BIN}` — the
coord build as it stood *before* the phase. So immediately after PHASE-08 landed
green, the prescribed `dispatch_reap{228, dispatch/agent-a76020385765dae76}`
returned the pre-fix `MCP error -32603: Internal error` — **a live reproduction of
ISS-245 and ISS-246 by the very session that closed them.** Recovery: prove the
three checks by hand from `funnel.toml` (one matching row, `concluded`, live OID
== `import.fork_tip`), reap via CLI `worktree gc --force`, then re-issue
`dispatch_reap` on the now-fork-absent path purely to advance the row. Structural,
no in-session remedy short of a restart. En route it confirmed the design working
as intended: the fresh CLI binary refused without `--force` and emitted the new
signpost with the token unchanged, which is D-P8-2's deliberate CLI deferral (R3)
behaving exactly as specified. **PHASE-08's fix is therefore unexercised through
its own MCP surface in-session; PHASE-07's drive is its first live end-to-end** —
EX-4's proof meanwhile rests on T8(a).

#### A landed phase has no re-drive path (found at the VT gate)

`verify-vt` reported PHASE-08 **VT-1 Fail** *after* conclude + reap:
`keywords = ["concluded", "funnel", "landed"]` on `src/worktree/gc.rs`, where
`funnel` appeared 49 times and `landed` 90 but `concluded` **zero**. Not an
oversight — D-P8-1's tier split moved the word up a tier: gc consumes a bare
injected bool and deliberately never learns funnel positions. The VT's substance
(T4 a/b/c) was delivered and green; the keyword proxy had been falsified by a
legitimate design amendment.

Two facts decided the repair. `verify-vt` reads the mandated `test_file` from the
**working tree** (`fs::read_to_string(root.join(rel))`) and attributes via the
recorded **source-delta** set — which already contained `gc.rs` from PHASE-08's
boundary. So the fix did not need to sit inside the phase boundary. But the
funnel row was at `reaped`, and the machine makes `reaped` terminal bar its own
replay, so **no funnel path can re-drive a landed phase** and a worker spawn
bound to PHASE-08 was structurally impossible. Landed instead as post-conclude
orchestrator remediation (`ef2d3e81`), user-approved: `GcState::funnel_landed`'s
doc comment now spells the conjunction out — exactly one row's `spawn.fork` names
the branch, that row stands at `concluded` (or `reaped`, its replay), and the live
branch oid still equals `import.fork_tip`. This is what R1 asked to be written
into the code doc, and it removes a module-boundary hop for a reader at the
consumption point. The alternative — amending VT-1's keyword list — was rejected
as editing a gate to match its outcome, the failure mode ISS-241 exists to
prevent.

Two cheap preventions for the next slice: run `slice verify-vt` at **phase-plan**
time so a keyword the amended design has made unreachable surfaces as a plan
amendment then, not at close; and when a design amendment moves a concept across
a tier boundary, sweep the phase's VT keywords for words that moved with it.

**Reconcile queue additions from PHASE-08.** VA-1's table above is the D10
fallibility input — the eleven survivors need a disposition each (repair now,
backlog, or accept) and `dispatch_next`'s row is the one that most affects the
shipped driver. Whether `resolve_landing`'s S6 integrity `bail!` should instead
be a fourth verdict + `Refused{funnel-record-integrity}` arm (worker deviation 1)
is an open call. Whether `FunnelOutcome::Reaped` should carry a detail field for
administrative residue (deviation 2) is a wire-shape question. And "no funnel
verb corrects a landed phase" deserves a backlog item — the remediation path used
here (orchestrator commit outside the funnel) works but is undocumented and
unmodelled.

### PHASE-07 — the OQ-5 memory-blind benchmark (evidence: `benchmark.md`)

Run as an `/execute` phase in the coord tree, **not** through the funnel (D-P7-5):
it produces measurement artefacts and a retirement list, not a source delta, and a
worker cannot write `.doctrine/`. No funnel row, no `record-delta`.

**Harness decisions.** Subject = a headless `claude -p` process in a stripped
clone, not a subagent — forced, not preferred: the three landing verbs are
MCP-only and the MCP server binds ONE root at startup (`src/mcp_server/mod.rs:38`),
so a subagent could only ever address the real corpus. Instrument =
`--output-format stream-json`, whose terminal `result` event carries cost,
per-model tokens, turns and terminal reason — both halves of EX-2 without asking
the subject to self-report.

**The nominal top-5 was stale — 4 of 5 already fixed** (IMP-272, ISS-218/SL-225,
IMP-256, IMP-127/236; only #2 coord-staleness still live, and #5 is out of `next`'s
span). Scenario set re-derived from the funnel's own claim surface. A ranked-issue
artefact that outlives its remediation cycle is a benchmark-integrity hazard;
`case-notes-analysis.md` should carry the backlog ids it derives from.

**Result.** A memory-blind orchestrator drove a populated funnel end-to-end twice
(both phases spawn→reap plus `sync --prepare-review`), and — killed at
`worker-committed` — a *second* blind session with no notes reconstructed position
and drove to completion. Zero memory access, zero harness rescue, zero questions
asked. `next` earns D4's positional-guidance claim.

**Three defects found, all filed:**
- **ISS-249** — the PHASE-02 coord pre-commit hook refuses EVERY commit when no
  chained hook exists (`set -eu` + a failing `resolve` assignment aborts before the
  `[ -x $next ]` guard that handles exactly that case). Verified by reproduction;
  this repo satisfies the trigger. It survived because **SL-228's own coord worktree
  predates its hook install — the slice never ran its own hook.** First-install
  breaking for any client project without a global/local pre-commit.
- **ISS-250** — `dispatch_reap`'s `not-landed` refusal prescribes `dispatch_reap`.
  D9's conjunction correctly refused a fork advanced past its imported tip (F-1
  holds, branch survived), but D10's **verbatim** single-sourced CLI remedy is
  caller-relative content: in the MCP arm it names the verb the caller just used,
  leaving only `--force`/`--superseded-head`. The blind operator followed it to CLI
  `worktree gc --superseded-head` and the unimported commit was destroyed. The guard
  fired and its own text walked the operator around it.
- **ISS-251** — `slice selector doctor` reports "healthy" when a path is named only
  in a phase objective / exit criterion (IMP-256 covers VT `test_file` only). Both
  blind subjects closed the gap by reasoning, not by the check that exists for it.

**NEW-OQ-C's revisit trigger is discharged.** Its stated condition was "sooner if an
operator is observed reaching for CLI gc on a funnel-managed fork". Observed on the
first blind run — and caused by the funnel's own refusal text (ISS-250).

**Stated limits** (full list in `benchmark.md`): n=1; a 2-phase markdown fixture;
and the measured arm was the **subprocess arm, not the claude arm** — the clone
lacked untracked `.claude/`, which is what the dispatch skill routes on, so
`arm-spawn` + the Agent-tool spawn went unexercised. Two harness artifacts of the
clone (missing `.claude/`, missing gitignored `web/map/dist/` making the baked
`prove` fallback red) cost the subject ~15 turns.

**Cost:** ~$18.23 total; ~90k output / ~352k cache-write / ~25.7M cache-read across
two full drives, one interrupted drive and two probes.

**Reconcile queue additions from PHASE-07.**
- ISS-249 / ISS-250 / ISS-251 each need a disposition. ISS-249 is a defect in this
  slice's OWN PHASE-02 deliverable (REQ-389) found by its own acceptance gate —
  user-flagged as "worth fixing as part of this slice"; recommended shape is an
  appended **PHASE-09** (PHASE-02 is `reaped` and has no funnel re-drive path), which
  also gives the missing **no-chained-hook VT cell** somewhere to live: §11 REQ-389
  pins the chained-global case but never the degenerate absence that ships by default.
- ISS-250 bears on **D10** — "a refusal's text IS the recovery procedure" (FR-009)
  needs a caller-relative arm; single-sourcing the *token* is right, single-sourcing
  the *remedy* is not. Candidate VT: no refusal's `detail` may name the verb that
  produced it.
- **EX-3 (OQ-6 retirement list) is NOT done** — 147 dispatch/funnel/worktree/worker
  tagged memories staged at `scratchpad/bench/oq6-candidates.txt`, 89 of them
  directly `mem.*.dispatch.*`. Corpus-scale; carried to the next agent.
- VH-1 (User acceptance of the benchmark result) remains open.

### PHASE-09 — landed, decisions and deviations

**Landed.** `28c67a52` (delta) + `74177f1f8` (boundary row), base
`B = f04d78de`. Two files, 84 insertions / 1 deletion:
`install/git-hooks/pre-commit` (one line) and `src/dispatch.rs` (the test cell).
VT gate after landing: **25/25 PASS, exit 0** — 0 UNCHECKABLE, 0 WAIVED.
S1 regression: `check regression diff --base f04d78de` → ✓ no new or changed
failures. Fork `dispatch/agent-a58b785e1b605d972` reaped through the oracle
(`landed ✓`).

**The fix.** `next_real="$(resolve "$next" || true)"`. `resolve` returns non-zero
by design when its argument does not exist, and an assignment from a command
substitution carries that status — under `set -eu` the hook died one line before
the `[ -x "$next" ]` guard written for exactly that case.

**The cell.** `install_coord_hook_permits_commit_when_no_chained_hook_exists`,
three cells under a PINNED absent chain (`GIT_CONFIG_GLOBAL`/`GIT_CONFIG_SYSTEM`
→ `/dev/null`, so the fallback is the temp repo's own `hooks/`, holding only
`*.sample`): (a) hook-check passing ⇒ the commit lands — the regression;
(b) hook-check refusing ⇒ still blocked; (c) binary absent ⇒ still fail-closed,
diagnostic intact. The hook-check leg is driven by a `#!/bin/sh` stub at an
absolute `DOCTRINE_BIN`, so the pass path runs THROUGH the check rather than
around it via `DOCTRINE_ALLOW_DELETE` (which is how the sibling test reaches the
chain leg). A unit test inside `src/` has no `CARGO_BIN_EXE_doctrine`, so a stub
is the only way to control that leg. RED first: the observed failure was an
EMPTY stderr — the defect's exact signature.

**D-P9-5 — line 62 is NOT the same defect.** `self="$(resolve "$0")"` is
syntactically identical to the fixed line, and the worker was asked to argue it
rather than assume. Verdict, accepted: `$0` is the path git just exec'd, so
`[ -e "$0" ]` holds by the invocation contract; `install_coord_hook` sets an
ABSOLUTE `core.hooksPath`, so it is CWD-independent too. The asymmetry is the
point — line 63's argument is *expected absent* in the shipped default, line
62's is *guaranteed present*. Adding `|| true` there would be an unreachable
belt that dilutes the signal about which line is load-bearing. Left alone.
The full VA-1 sweep (`:41 :48 :53 :61 :62 :63`) found no other live fragility:
`:41` is already guarded inside the substitution; `:48`/`:53` are unguarded but
cannot fail where they run (inside a repo, non-bare), and their failure
direction is fail-closed anyway.

**Deviation — landed via the live-worktree import fallback, user-approved.**
`arm-spawn` was run `--base B --slice 228` **without `--phase PHASE-09`** (the
shape this slice's own handover documented, written before PHASE-04 made the
binding `(slice, phase)`). A half-arm binds nothing, so the fork's
`DispatchRecord` carried no binding and `worker_commit` correctly refused
`unprovable-fork`. There is no verb to bind an existing fork. Options were
re-spawn (~$2, clean) or fallback (A) — `worktree import --from-worktree`, same
`classify_import` belt, in-process prove gate, orchestrator commits and attributes
the phase explicitly via `record-boundary`. User chose the fallback. Consequence:
the commit carries orchestrator authorship, not the worker's, and the phase
attribution is explicit rather than fork-time-snapshotted.

**Reconcile additions from PHASE-09.**
- **The `unprovable-fork` refusal named neither the cause nor the remedy** — its
  `detail` is just the branch name. The operator cannot learn from it that the
  arm was half-armed, nor what to do next. This is the same class as ISS-250 and
  a second data point against D10's "a refusal's text IS the recovery procedure"
  as currently written: here the text carried *no* procedure at all. Found the
  same way the benchmark found its own defects — by running the thing.
- **ISS-253 filed** (on `edge`): the `/dispatch` router's env-marker arm test
  answers "absent" from the coordination worktree its own step-2 cwd rule
  mandates, because `.claude/` is gitignored and never materialises in a linked
  worktree. Same root cause as the benchmark's stated limit 3 (a clone carries
  tracked content only), different surface — and here it is the router's own
  instruction that defeats the test. Needs a disposition.
- **The primary-tree phase-sheet mirror cannot work for mid-slice appended
  phases.** `slice phase --status completed` warned: `mirroring completion into
  the primary tree failed: Phase tracking not found at
  .doctrine/state/slice/228/phases/phase-09.toml`. The primary tree is on `edge`,
  whose `plan.toml` has no PHASE-08 or PHASE-09 — both were appended on
  `dispatch/228` and do not exist off it until integrate. So the primary sheets
  stop at PHASE-07 and the mirror (ISS-212) has nothing to write. Benign here
  (`prepare-review`'s completeness gate derives its completed-set from that same
  tree, so an absent sheet is absent from both sides), but it means the mirror is
  structurally unavailable to any phase appended mid-drive. Worth a disposition:
  either the gate should read the coord tree, or the warning should say
  "expected for an appended phase" instead of reading as a failure.

### Follow-through captured for the D10 repair (IMP-321)

The reconcile queue's D10 item ("a refusal's text IS the recovery procedure"
needs a caller-relative arm) now has a **verification vehicle** and a sequencing
edge, so it does not evaporate at close:

- **IMP-321** — "Verify the advice surface: refusals and prescriptions against the
  machine". Filed on `edge` with `after: SL-228` (a structural sequencing edge,
  surfaced and evicted at order time — not prose), `references(originates_from):
  SL-228`, and `related` edges to ISS-249 / ISS-250 / ISS-251 / ISS-253.
- It carries the evidence this slice generated: the benchmark's `stale-record`
  refusal with an EMPTY detail (ledger s4b:69, after which the subject read three
  source files to decode it), ISS-250's destructive remedy, PHASE-09's
  contentless `unprovable-fork`, and the mechanical root — **24 sites construct a
  refusal with a structurally empty detail, 14 of them in
  `src/mcp_server/dispatch.rs`**, the funnel tools every drive runs through.
- Its recommendation splits the work: the contentless-detail and
  self-referential-remedy modes are a **lint** (the ISS-250 VT already drafted in
  this queue), while the incomplete-model prescriptions (`next` conflating
  not-funnel-driven with unstarted; `selector doctor`'s blind spot) need design
  judgement.

**For the audit:** disposition the D10 finding on its merits here, and cite
IMP-321 as where the verification lands — do not re-derive the evidence.

### PHASE-07 EX-3 — the OQ-6 retirement list (delivered)

`.doctrine/slice/228/oq6-retirement.md` + `verdicts.tsv` + the harness under
`evidence/` (`needs.py`, `oq6-sys.md`, `batch.sh`, `run.sh`). 149 candidates, 147
verdicts.

**The finding that matters.** The raw triage proposed **35 retires**; adjudication
cut it to **20**. 28 of the 35 rested on `never-arose`, and **15 of those are
claude-arm facts the benchmark never put on trial** — it measured the subprocess
arm (stated limit 3), so for those memories "never arose" is a tautology of the
harness, not a finding. Tiered: A = 7 retirable on operational proof
(`needed-and-handled`); B1 = 13 arm-neutral, carrier-only; B2 = 15 HELD pending a
claude-arm round; C = 1 rejected (verdict RETIRE with its own `q1` reading "no
carrier found"); 2 amend; 110 keep.

**This session is the counter-example.** PHASE-09 drove the claude arm and hit a
provisioning trap the verbs do NOT carry (the half-arm → `unprovable-fork`), in
the same week a retirement pass would have thinned that arm's corpus on
never-arose evidence.

**Method note for the audit.** Per-memory triage was delegated to
`deepseek/deepseek-v4-pro` (13 batches, each given the situations ledger derived
from the raw transcripts + the verb surface + full memory bodies), then every
RETIRE was adjudicated here. Two delegation defects worth carrying: batch 11
**truncated silently** (3 verdicts for 12 memories, no error) and, worse,
*degraded before it died* — two pre-truncation verdicts contradicted the stated
rule and came back correct on re-run. Verify the **arity** of delegated output,
not just its content.

**Candidate-selection note.** ~28 of the 110 keeps are out-of-scope memories the
body-grep swept in (close/integrate lineage, audit surface, git plumbing, jail
infra, frontend). A future pass should scope by `mem.*.dispatch.*` + explicit
tags, not a grep.

## Harvest — audit (RV-312)

fresh-as-of: **lifecycle `reconcile`** · coord tip `5594078e` (dispatch/228) · edge
`bb0b9428` · **9/9 phases completed** · VT gate **`25/25 PASS`, 0 UNCHECKABLE, 0
WAIVED — `verify-vt 228` exits 0** · `doctrine check gate` **exit 0, 4011 passed /
0 failed**, clippy clean, run twice · `dispatch sync --prepare-review` done (9 refs)
but **stale**: the F-1 fix `707236c0` and this harvest landed after the cut.
Supersedes the "PHASE-08 concluded + reaped" stamp above (7/7, 23/23) entirely.

Closes the drive. VH-1 accepted 2026-07-27; audit ledger **RV-312** done, 7 findings,
all terminal, no unresolved blocker.

### The one defect this audit found

**F-1 (blocker, fixed `707236c0`).** `act_on_create` consumed the one-shot arming
*before* acquiring the per-name claim lock, six lines above a comment asserting the
lock covered the whole claim→bind→act window. Two same-name spawns both consume; one
gets the binding and refuses at the claim, the other wins and forks **unbound**.
An unbound fork is `unprovable-fork` — refused only after a whole worker run is spent,
no re-bind verb. Both consumes moved under the lock; they remain before `fork_core`,
so SL-199 EX-3's stated property is untouched.

Caught only because the gate was run twice: exit 0, then exit 101 on the immediate
re-run. Rate 2/20 before, 40/40 after; suite 4011 passed / 0 failed. **PHASE-04 VT-5's
earlier `verify-vt` PASS was luck** — verify-vt samples each criterion once.
Durable lesson recorded as `mem.pattern.audit.rerun-gate-for-concurrency-invariants`.

Note this is the *second* route to `unprovable-fork`. The drive attributed that dead
end wholly to operator error (arming `--slice` without `--phase`,
`mem.pattern.dispatch.half-arm-unprovable-fork`). It was not only operator error.

### Conformance, fully accounted

`slice conformance 228` → 6 undeclared, 8 undelivered, 30 conformant. Every cell has a
named cause; the genuine residue is **two stale selector declarations**.

- 6 undeclared — all `.doctrine/**` authored metadata. Zero source scope-creep.
  Second independent confirmation of IMP-292 defect 1 (first was SL-219).
- 3 undelivered — `.doctrine/adr/001/layering.toml`, `.doctrine/spec/tech/021/funnel-machine.md`,
  `src/main.rs` — all delivered in `b8e64e109`, PHASE-03's **base commit**, which IS
  PHASE-03's boundary `--start`. A `[start, end]` range excludes its own start, so
  base-beat content is structurally invisible. Recorded as IMP-292 **defect 3**.
- 3 undelivered — `.agents/skills/dispatch*/**`, the untracked install projection;
  real source under `plugins/doctrine/skills/**` is delivered and conformant. (F-3)
- 2 undelivered — `flake.nix`, `src/worktree/shared.rs`: genuinely never touched. (F-4)

### The close ritual was untested and failed twice

`dispatch sync --prepare-review` had never been run for this slice. Both refusals
prescribed `record-delta`; for the first that advice was wrong.

- PHASE-08/09 "not a completed phase" — `registry_completeness` reads the **primary**
  tree's gitignored sheets (`src/state.rs:959/981`), and a mid-drive appended phase can
  have none there (edge's `plan.toml` lacks it, so `slice phases` cannot materialise
  it). The drive recorded this as benign; it hard-blocks close. Repaired by copying
  both sheet pairs into the primary runtime tier. → IDE-028 refinement; IMP-272 is
  marked `resolved` but does **not** cover the appended case.
- PHASE-07 "no recorded source-delta row" — evidence-only phase, no exemption in the
  gate. The handover's "nothing to record-delta for it" was false and untested.
  Repaired with a **Manual**-provenance row `0da787d98..a54c05566` (Manual is filtered
  out of guard (3)'s `Funnel|Unknown` check, so it claims no funnel ownership).
  → **ISS-254** filed. Note the projection still cut 8 phase refs for 9 phases: cuts
  come from the *committed* ledger, where PHASE-07 correctly has no row.

### Delegated, not done here

Reconciliation brief in `review-312.md` carries the write plan. Per-slice: remove 6
selectors (4 unsatisfiable/wrong-tier, 2 undelivered) via `slice selector rm` — the
**registry** is load-bearing, design §6 is its mirror — plus four design.md prose
corrections (D7's inverted authoring direction, `attempt_advance`,
`paths_since_verify: None`, the widened `already-<position>` gloss). Governance:
D10/FR-009 splits by REV — `next` keeps the forcing-function claim for *positional*
guidance; "a refusal's text IS the recovery procedure" demotes to a goal with
**IMP-321** as its verification vehicle. Counter-example set is now **four** (ISS-254
adds the two same-text-different-cause completeness refusals). No `plan.toml` edit:
`EN-/EX-/VT-/PHASE-` ids are immutable-append and off the reconcile surface.

REQ-387 may remain `pending` at close — subprocess-arm gating was deferred by design.

## Harvest — reconcile (RV-312 brief → REV-039)

fresh-as-of: **lifecycle `reconcile`, closing** · coord tip `a5a4483` (dispatch/228,
post `refresh-base`: 181 trunk commits merged clean) · trunk/edge `bbde84cd` ·
**9/9 phases completed** · `verify-vt 228` **25/25 PASS, exit 0**, 0 UNCHECKABLE,
0 WAIVED · `doctrine check gate` **exit 0, 4903 passed / 0 failed**, clippy clean ·
the claim-lock race test hammered **30/30 green** post-merge. Supersedes the audit
harvest's stamp above (coord `5594078e`, 4011 tests) entirely.

Reconciliation brief fully executed. RV-312 carries the
`## Reconciliation Outcome`; REV-039 (`reconcile-sl-228`) is **done, approved**.

### What the registry proved that prose could not

The brief called the selector registry "load-bearing, prose is the mirror" and it
was right in a way worth keeping: `slice selector rm` × 6 moved
`slice conformance 228` from **8 undelivered to 2**, with undeclared and conformant
unchanged (6 / 30). The two survivors are exactly F-2's base-beat pair
(`.doctrine/adr/001/layering.toml`, `src/main.rs`) — delivered in `b8e64e109`,
invisible because that commit *is* PHASE-03's boundary `--start`. So **SL-228's
genuine conformance debt closed at zero**, and the number is trustworthy because
every remaining cell has a named cause. A design.md-only pass would have left the
report reading 8 and the story reading "accounted for" — the exact divergence
conformance exists to prevent.

### Three things the brief got wrong, and what that costs

Recorded because the brief was otherwise excellent, and the failures share a shape:
**every one was a stale or mis-aimed pointer, never missing content.**

1. **`design.md §6` for the selector mirror** (in the brief *and* in F-3's response).
   §6 is `dispatch next`; the mirror is §10, "Code impact summary". Nothing detects
   a wrong prose anchor — you find it by opening the target. Note the irony worth
   keeping: the audit's own handoff artefact exhibited F-5's defect (advice naming
   the wrong target) while documenting it.
2. **The primary tree's `design.md` was 293 lines stale** (763 vs 1056) and is the
   default cwd's copy. Anchors gathered from it were all discarded. The handover
   *did* warn; the warning still lost to the default. See
   `mem.pattern.audit.dispatched-slice-surface-split` — its wrinkle 1 is this trap
   for the selector registry; it now generalises to every slice artefact.
3. **The second REV item was a no-op.** The brief asked to split D10's strong form
   "wherever the tech spec restates it" — SPEC-021 **never restates it** (searched
   `refus`/`prescri`/`remedy`/`recovery`; SPEC-021's own D1–D7 are an unrelated
   decision set). Recorded as inspected-and-inapplicable rather than manufacturing
   an edit to satisfy a brief line.

### The split that REV-039 actually landed — and what it did not

REQ-385 (`FR-009`) lost `with prescription` from its statement; the two-speed claim
now lives in `requirement-385.md`: positional naming **normative and proven** (a
memory-blind orchestrator drove a full funnel and a crash recovery unaided);
prescription completeness a **goal**, vehicle **IMP-321**. `design.md` D10's Refs
column and §2's `IllegalTransition` doc-comment carry the same split.

**Five restatements of the strong form survive, deliberately untouched** — three
source comments (`src/funnel_machine.rs:347`, `src/dispatch.rs:5771`,
`src/mcp_server/worker_commit.rs:1466`) and two pieces of **shipped operator
instruction** (`install/dispatch-mechanics.md:51`,
`plugins/doctrine/skills/dispatch/SKILL.md:104`). None is `/reconcile`'s write
surface; all five are appended to IMP-321 so its fix retires them in one pass.

The sharp one is `worker_commit.rs:1466`: it emits *"the payload IS the recovery
procedure: {detail}"* **to the operator at runtime**, so in any of the 24
empty-`detail` cases it asserts sufficiency in the same breath as supplying
nothing. That is the strongest single argument for IMP-321's lint arm — the claim
is not merely documented, it is *spoken to the operator at the moment it is false*.

### Design prose corrections (F-7), each verified against source before writing

- **§2** — `attempt_advance` + `Advance` documented beside `attempt`, additively.
  `attempt` (`funnel_machine.rs:534`) is a by-value projection carrying an explicit
  `expect(dead_code)` whose stated reason is that §2 publishes it as the seam.
- **§2** — `already-<position>` widened from replay-only to the
  *milestone-already-passed* family: one family, two causes. The replay leg needs
  `t.target(current) == current`, so backward attempts fall to the same tokens.
- **§5** — `paths_since_verify: None` fails **closed** as `conclude-verify-stale`
  (`conclude_allowed:741`, `is_some_and`). Framed as the intended reading of I2 —
  an unanswered identity question is not an attestation.
- **D7** — not softened, *causally explained*: the `.doctrine/` hard wall inverts
  the authoring direction, so the golden is authored first at the orchestrator's
  phase base and the renderer is pinned to it. Same wall as F-3's unsatisfiable
  selector, same wall as F-2's invisible base beat. One cause, three symptoms —
  that is the durable shape, not three separate gotchas.

### Off-surface, held

No `plan.toml` edit; no `EN-`/`EX-`/`VT-`/`PHASE-` id added, changed, or renumbered.
**REQ-387 stays `pending`** — subprocess-arm gating was deferred by design, and
marking it satisfied to tidy the close rollup would be the same over-claim REV-039
exists to remove.
