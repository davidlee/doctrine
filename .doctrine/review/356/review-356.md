# Review RV-356 — reconciliation of SL-254

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Mode: **conformance** (post-implementation audit of an implemented slice).
Facet: `reconciliation`. Subject: `SL-254`, target-ladder rung 1.

### The surface reviewed

`SL-254` was not driven by `/dispatch`, so there is no candidate interaction
branch. It was built inside a microVM capsule holding a separate checkout and
exported over a capsule sideband:

- code + authored state — `refs/capsule/a/heads/work` @ `3aee9dc7d`, 31 commits
  on base `caf7f2a21` (`plan(SL-254): slice ready`);
- checked out for this audit at `.worktrees/SL-254-audit`, branch `audit/SL-254`,
  HEAD identical to the capsule work tip.

Runtime phase state (`.doctrine/state/slice/254/phases/`, sheets 01–10) rode the
export and was copied into the primary tree so `slice conformance` could see
phase completion. The primary tree never left `edge`.

### Lines of attack

1. **Did the deletion actually complete?** Six of ten phases delete. The claim is
   that four mechanisms — disk-marker identity, the `SubagentStart` stamp, the
   `worktree pretooluse` wall, and the gated `worker_commit` MCP tool — are gone
   from every surface, not merely from the binary. Probe: absence greps across
   `src/`, `tests/`, `scripts/`, `install/`, `plugins/`, `.claude/`, `docs/`,
   `.doctrine/` — with a positive control on every negative result.
2. **Does the surviving arm work?** The slice's premise is that a confined
   `claude -p` subprocess replaces the in-session arm. `PHASE-09` is the only
   evidence that the assembled path runs at all. Probe: the live-fire record,
   the shipped script against what the design says it contains, and the
   confinement proof's skip-not-pass discipline (`DEC-212`).
3. **Is the governance corpus true again?** `PHASE-08`'s `REV-052` re-derived
   its target set and landed eleven entities. Probe: `spec validate`, an
   independent corpus-wide `[[source]]` anchor scan, and the specific regions
   the slice's own notes nominated but the REV may not have reached.
4. **What did the design promise that shipped differently?** The slice records
   twelve successive under-counts of its own surface. Probe: `slice conformance`
   against the `design-target` selector registry, and each `§5.x` claim the
   live fire falsified.
5. **What is agent-facing and now false?** The failure this slice exists to
   remove is an agent instructed to use a mechanism that no longer exists.
   Probe: every surface resolved into an agent's context — the boot snapshot's
   authored source, hymn role bands (shipped, materialised, and project-local
   twins), agent defs, skills, and the memory corpus.

### Invariants held to

- **`INV-2`** (design §7) — a worker cannot skip the *scope belt*.
  `classify_import` remains its enforcing caller; the *check gate* is a separate
  claim and does move altitude.
- **Behaviour preservation** (`PHASE-01`, design §9.1) — the re-home is proved
  by the existing suites staying green *unchanged*.
- **`DEC-212`** — a proof that cannot run must **skip named**, never pass
  vacuously. A vacuous pass is the fail-open shape the slice deletes.
- **Every phase boundary is a green tree** — `just gate` at zero clippy
  warnings.
- **`POL-002`** — nothing host-project-specific leaks into the shipped surface.

## Synthesis

### The closure story

`SL-254` set out to delete four mechanisms that existed for one reason, and it
deleted them. `EVD-023` falsified `ADR-011`'s billing premise; `DEC-202` took the
consequence; ten phases removed the disk marker, the `SubagentStart` stamp, the
`worktree pretooluse` wall and the gated `worker_commit` MCP tool, and replaced
them with a generalised confined-subprocess spawn script that already worked on
the pi arm. Net −4,400 lines across 87 non-`.doctrine` files.

**The deletion is complete on every surface an agent can reach.** Absence greps
over `src/`, `tests/`, `scripts/`, `install/`, `plugins/`, `.claude/`, `docs/` and
`memory/` for the retired vocabulary return exactly one live hit —
`install/doctrine.toml.example:114`, which deliberately names `worker_commit` as
the *retired* reader. Every other occurrence is a dated historical record in
`.doctrine/`, which is where they belong. One surface was missed and is fixed in
this audit (`F-1`).

**The verification held.** All fourteen unwaived `VT` mandates pass; the two
waived ones (`PHASE-03/VT-1`, `PHASE-06/VT-2`) are waived for the same structural
reason — a *presence* mandate cannot witness an *absence* — and both re-land as
`VA` claims checked at agent altitude rather than being quietly dropped. `just
gate` is green at zero clippy warnings. `spec validate` reports the corpus clean,
`publication validate` and `prompt check` are clean, `boot --check` is clean.
An independent corpus-wide sweep of all **86** live `[[source]]` anchors found
**zero** dangling — confirming `PHASE-08`'s `VH-1` by re-derivation rather than by
trust, which matters because `spec validate` demonstrably reported "corpus clean"
the entire time two anchors dangled.

**`PHASE-09` is the phase that earned the slice.** Six phases of careful
symbol-level work had collapsed the arm onto a script that had never once been
executed, and running it found two independent fatal defects in the shipped exec
path, both invisible to every suite: `claude -p --output-format stream-json`
hard-refuses without `--verbose`, and an inherited `TMPDIR` outside the jail's
writable set made every Bash tool call fail `EROFS` before any command ran. A
third defect — the script swallowing the worker's exit code — meant a worker that
died in 1.4s still exited the script 0. All three are fixed, two of them pinned by
regression tests. Design §9.3's instinct that a live phase is the *only* evidence
here is now empirically vindicated rather than merely argued, and the
generalisable lesson is sharper than the slice's own framing: **a green incumbent
profile carries no signal whatsoever about a new one.**

**The confinement itself was never the problem.** Once `TMPDIR` pointed inside the
jail, the confined worker ran `cargo check` clean in 16.65s against a read-only
`~/.cargo`, edited only its declared file, and handed back uncommitted — with the
belt refusing two earlier imports correctly. Every defect found was in the
plumbing around the jail, not the jail.

### Standing risks

1. **The Darwin arm is non-functional, not merely unverified** (`F-2`). Network
   is denied by construction on the Seatbelt path, so a macOS `claude -p` worker
   cannot reach the API. Two further Darwin residuals ride with it: no
   `sandbox-exec` presence probe (so a missing backend still fails *unnamed*,
   after a fork has been minted), and a `TMPDIR` pointing *inside* the worktree
   whose delta the orchestrator imports — benign here only because
   `.gitignore:14` happens to match.
2. **105 project-local memories describe deleted mechanisms as live** (`F-5`),
   one of them indexed in the boot snapshot. This is the highest-leverage
   non-blocking item in the brief: memories are retrieved *on purpose*, so a
   stale-but-plausible one is worse than falsified ADR prose. The shipped corpus
   is clean at zero — `POL-002` held.
3. **`worker-forbidden-writes`'s configurable tail enforces nothing** (`F-3`).
   No live exposure, honestly marked in code, owner-routed to `SL-255` with the
   fix shape already decided (kernel-level `--ro-bind`, not a post-import belt).
4. **One unconfined dispatch spawn survives in `scripts/`** (`F-4`). Not on
   `/dispatch`'s path, but a live counterexample to the slice's headline claim
   read as a property of the repo.
5. **`REV-046` now describes shipped deletions as unimplemented target work**
   (`F-11`). A `proposed` revision gating the capsule cutover, holding a rationale
   the incumbent has overtaken.

### Tradeoffs consciously accepted

- **Nine of twelve under-counts, and the method that caught them.** The slice
  under-counted its own surface twelve times, always low, and every one was found
  by the mechanism `R8` prescribes — re-derive at the phase, never read off the
  design. The twelfth is the instructive one: `PHASE-08/EX-1` named the sweep
  scope as `.doctrine/{adr,spec,policy,standard}`, and requirement prose lives in
  none of them, so every sweep was *structurally incapable* of seeing 14 falsified
  requirement entities. `F-1` is the same class arriving once more — a directory
  list is itself a claim about where truth lives.
- **`ADR-011` amended, not superseded**, at ~56 of ~70 regions falsified
  including its own `VA-2` verification criterion. Owner ruling; keeps the
  decision history in one document.
- **Two escalations stopped work rather than improvising** — `PHASE-06`'s orphaned
  config reader and `PHASE-08`'s `ADR-020` scoping question. Both were escalated
  with nothing authored, both presented as competing readings rather than a
  preferred answer dressed as a finding, and both were ruled on by the owner. That
  is the behaviour the anti-escape guardrails exist to produce.
- **The clean-room adversarial pass on a prose phase paid for itself.** Denied
  this slice's own `notes.md`/`design.md`, a fresh reviewer returned 14 findings,
  3 blocking — the most valuable being a security-relevant over-claim in which
  eight entities had been given "the fork worktree is the sole write floor" while
  `spawn-confined.sh:179` rw-binds `$CFG_DIR` too. The shipped skill had it right;
  the *amendments* had introduced a contradiction with shipped truth. Prose review
  against prose would never have caught it.

**Verdict: no blockers. `SL-254` is ready to advance to `/reconcile`.** Every
finding is terminal; three were fixed inside the audit; the remaining nine are
documentation, decision, and corpus-hygiene work that belongs to the write surface
`/reconcile` owns.

## Reconciliation Brief

Built from every non-`aligned`, non-`fix-now` finding, grouped by the surface
`/reconcile` will actually touch. `F-1`, `F-7` and `F-12` were fixed inside the
audit and appear below only where they leave a residual.

### Per-slice (direct edit)

- **`design.md` §5.2.1 + §5.1 + `D4` — retire the parity claim, do not repair its
  count** (`F-2`). "Everything harness-specific reduces to two facts: the config
  directory to bind, and the exec line" is false in both directions:
  `DOCTRINE_WORKER` (Darwin lacked it, `RV-355` `F-2`) and `TMPDIR` (Linux lacked
  it, found live at `PHASE-09`). Replace the symmetry assertion with an
  enumeration of the known asymmetries, and re-word `VA-2`'s intent as "enumerate
  every asymmetry" rather than "check Darwin against Linux".
- **`design.md` §5.2.1 — the quoted claude exec line and its flag table**
  (`F-10`). Add `--verbose` to the snippet, and add a `--verbose` row to the
  flag-by-flag justification table citing `DEC-215`. As printed, the design's
  command hard-refuses in 1.4s.
- **`design.md:895` (§5.6) and `design.md:557`** (`F-3`). Both assert
  `classify_import` is `worker-forbidden-writes`'s enforcing reader. It never
  was — it enforces only the two hard-coded floors. Correct to: the key survives
  as a **declaration** with no production reader, the floors are unaffected, and
  the enforcing reader is `SL-255`'s to supply (`IDE-051`).
- **`design.md` §5.2.3 — the `land` edge case** (`F-13`). "Substitute the
  branch-shape role classifier (`shared.rs:77`) returning `"fork"`" would refuse
  every `land`. What shipped is `shared.rs::is_dispatch_fork_branch` (`:112`) —
  `dispatch/` prefix **and** a non-numeric suffix. Intent and the
  "strictly stronger" claim are unchanged; only the mechanism named is wrong.
  Note that `notes.md` routes this to "the REV"; `design.md` is not a REV target,
  so it lands here instead.
- **`notes.md` residual from `F-15`** — record that `SL-254`'s authentic
  `provenance = "solo"` registry rode the capsule sideband into
  `.worktrees/SL-254-audit/.doctrine/state/slice/254/boundaries.toml`, and has been
  copied to the primary tree so `slice conformance` can resolve it. Runtime tier,
  so it is disposable and does not land with the branch — anyone re-running
  conformance on another tree must copy it across again until the resolver changes.
- **Re-run `doctrine boot` on the landing tree after the branch lands** (`F-1`
  residual). The fix is committed at `1185a631d`, but each tree's snapshot is
  gitignored runtime state and regenerates locally.

### Governance / spec (REV)

- **`ADR-001` — `layering.toml:138`** (`F-6`). `"worktree::jail" = "leaf"  # …
  pure jail core — no disk/git/clock/rng` is false: `PHASE-01`/`DEC-206` moved
  `have_bwrap` (env read + `is_file()`, `jail.rs:152`) and
  `write_seatbelt_profile` (`std::fs::write`, `jail.rs:507`) in, unseamed. REV
  `modify` — restate as *"impurity behind the injected `ResolveEnv` seam"*, the
  module's genuine invariant since SL-183. The twin comment in `jail.rs:5`/`:19`
  is a source edit that must land in the same change or the two will disagree.
- **`ADR-020` Context (`:5-10`, `:8-9`)** (`F-11`). REV `modify` — acknowledge
  that SL-254 discharged two of the four incumbent brittlenesses the capsule case
  argues from (cooperative marker identity; harness-specific spawn paths). The
  decision is unaffected; leaving the Context makes the case read stronger than it
  now is. `:86-88`'s authority clause stays literally true and needs no edit, but
  a reader who takes it as a promise the incumbent is *unchanged* will be wrong.
- **`REV-046` — its rationale, or its gates** (`F-11`). It enumerates precisely
  the mechanisms SL-254 has now deleted and describes them as unimplemented target
  work awaiting a cutover. It is `proposed · approval=none`. Decide which half is
  stale and restate it; do not leave the question in a closed slice's notes.
- **`DEC-210` — knowledge record correction** (`F-9`, `doctrine knowledge edit`).
  Binding `$CFG_DIR` **carries** a subscription credential into the jail; it does
  not **create** one. Record the unstated host precondition: a materialised
  `~/.claude/.credentials.json`, or `CLAUDE_CODE_OAUTH_TOKEN` in the environment.
  Design §5.5 inherits the same gap.
- **`DEC-204` and `DEC-213` — consequence text** (`F-3`). Both name
  `classify_import` as `worker_commit`'s surviving replacement *for the config
  key*. Correct to the shipped truth; `worker_commit` is deleted and nothing
  replaced it, which is what the code comments now say.

### Selector registry (`doctrine slice selector add` — not prose)

- **~25 undeclared code paths** (`F-8`). `slice conformance` reports undeclared
  96 / undelivered 0 / conformant 40. The load-bearing fix is the registry, which
  is what conformance reads; design §5.6 is the human mirror and should be updated
  to match, not instead. Paths: `src/commands/{guard,doctor,cli}.rs`,
  `src/{doctor_checks,finding,install}.rs`, `src/mcp_server/dispatch.rs`,
  `src/worktree/{allowlist,claim_lock,dispatch_record}.rs`, `justfile`,
  `plugins/doctrine/hooks/hooks.json`,
  `plugins/doctrine/skills/{execute,dispatch-spawn}/SKILL.md`,
  `install/hymns/role/{orchestrator,worker}.md`,
  `install/agents/claude/{dispatch-orchestrator,dispatch-probe,dispatch-worker}.md`,
  `install/workflows/drive-slice.js`, `.doctrine/workflows/drive-slice.js`,
  `install/manifest.toml`, `publication/manifest.toml`, `tests/common/mod.rs`,
  `tests/e2e_worker_confinement.rs`, `tests/e2e_dispatch_arm_spawn.rs`, and the
  retargeted `tests/e2e_{worker_guard,worker_guard_explicit_root,worker_gate_skip,
  dispatch_h1_integration,mcp_server,prompt_resolve_golden}.rs`.
  `crates/doctrine-control/src/backend/bubblewrap.rs` is a one-line doc-comment
  rename — declare it or note it as deliberate incidental reach.

### Corpus hygiene (`/reviewing-memory`)

- **105 project-local memory items reference retired mechanisms** (`F-5`) —
  `SubagentStart`, `stamp-subagent`, `dispatch-agent`, `worker_commit`,
  `arm-spawn`, `pretooluse`. Already carried in `slice-254.md`'s Follow-Ups.
  Retire or re-anchor per item; do not edit in place where a memory's whole
  subject is gone. `mem.signpost.doctrine.dispatch-claude-arm-wrong-base` is in
  the boot snapshot's Memory index and should be handled first. The shipped
  corpus (`memory/`) is clean at zero — confirm it stays that way.

### Backlog (cards, not edits)

- **`scripts/pi-spawn.sh` — decide** (`F-4`). Delete, or bring under the
  confinement prefix. It asserts worker identity with zero `bwrap`.
- **Darwin arm defects** (`F-2`) — fold `H1`–`H4` from `notes.md`'s `VA-2`
  section into `IMP-429` so the next mac-equipped session tests hypotheses rather
  than re-running the arm blind.
- ~~**Phases landing without a source-delta row** (`F-7`)~~ — **withdrawn by
  `F-15`.** There was no such gap: all ten phases recorded automatically. The
  card that replaces it is the resolver item below.
- **`ISS-350` — minted 2026-08-14, so this one is already off the brief** (`F-15`).
  The registry and its sibling resolve by different roots: `boundaries_path()`
  pins to `primary_worktree(cwd)` on read and write (`state.rs:943`),
  `phases_dir()` joins the local `project_root` (`:135`), and
  `registry_completeness()` consumes both (`:1208`) — so an adopted worktree
  reports `10/10` phases and zero source deltas at once. Carries the diagnosis,
  the swap-test evidence, the primary-pinned-write hazard, three fix directions,
  and the residual that runtime state does not travel with a branch.
  `references(originates_from) SL-254`, `references(concerns) RV-356`.
  `/reconcile` need only confirm it exists; it does not need re-deriving.
- **`base64` is an unused dependency**, verified at PHASE-05 against the current
  tree (zero references in `src/` or `tests/`). Gate-neutral — cargo does not warn
  on an unused dep — which is why it was left rather than touched mid-slice.

### Already carried by the slice (confirm, do not re-derive)

`slice-254.md`'s Follow-Ups and `notes.md`'s Open sections already hold three
obligations this audit did not re-raise because they are correctly homed:
contribute the post-capsule finding to `RFC-025`; disposition the seven plausibly
dissolved backlog items (`IMP-269`, `IMP-342`, `IMP-334`, `IMP-337`, `IMP-407`,
`IMP-401`, `IDE-024`) by confirmation rather than assumption; and revisit
`REQ-335`, deliberately **not** retired at `PHASE-08` because design §6 `OQ-2`
keeps its tier `pending` as a contract. Also open, and recorded rather than taken:
no `[[source]]` anchor points at `scripts/spawn-confined.sh`, now the central
shipped mechanism — adding one is a spec-boundary decision.

## Reconciliation Outcome

Reconciled 2026-08-14 on `audit/SL-254`. Every brief item is resolved. All 15
findings remain `verified` — remediation is recorded here, never by mutating a
finding's disposition.

### Direct edits applied

- **`design.md` §5.2.1 opening** (`F-2`) — the parity claim *"everything
  harness-specific reduces to two facts: the config directory to bind, and the
  exec line"* is retired, not repaired. Replaced by an enumeration on **two axes**:
  a harness table (config dir, exec line, completion signal, host credential
  precondition) and a platform table (`DOCTRINE_WORKER`, `TMPDIR`, network,
  capability probe). Both are prefixed by a `> **Reconciled (RV-356 F-2)**`
  blockquote stating that each list is a **floor**, not a closed set — the count
  was never the defect.
- **`design.md` §5.1 diagram and `D4` (§7.2)** (`F-2`) — §5.1's
  `PREFIX resolution (harness parameterises ONE token pair)` line now cites the
  enumerated asymmetry sets instead. `D4` now reads *"The decision holds; its
  stated ground does not. Harness does **not** reduce to two tokens…"* — the
  decision is preserved and its false premise withdrawn, which is the honest
  shape for a decision that turned out right for a wrong reason.
- **`design.md` §5.2.1 claude exec snippet + flag table** (`F-10`) — `--verbose`
  added to the quoted command (`claude -p --output-format stream-json --verbose \`)
  and a matching flag-table row citing `DEC-215`,
  `claude_arm_stream_json_carries_verbose` in `jail.rs`, and
  `spawn-confined.sh:236-245`. As printed the design's command hard-refused in
  1.4s.
- **`design.md:557` and `design.md:895` (§5.6)** (`F-3`) — `:557`'s
  `worktree/import.rs` row now reads **untouched**: `classify_import` remains the
  enforcing caller of the two hard-coded scope-belt floors (`.doctrine/**`,
  `.claude/**`), never read `worker-forbidden-writes`, and does not start now.
  `:895` strikes the false claim through and records the key as *a declaration
  with no production reader*, routing the enforcing reader to `SL-255` / `IDE-051`.
- **`design.md` §5.5 — the `land` edge case** (`F-13`) — retargeted from the
  drafted `shared.rs:77` / `classify_worktree_role` (which would have refused
  every `land`) onto the shipped `shared.rs::is_dispatch_fork_branch` (`:112`):
  `dispatch/` prefix **and** a non-numeric suffix. A paragraph explains the
  failure the drafted mechanism would have caused; intent and the
  "strictly stronger" claim are unchanged.
- **`design.md` §5.5 — the `--verbose` edge case** (`F-10`) — settled as
  mandatory, not build-dependent.
- **`design.md` §5.6** (`F-8`) — a
  `**Reconciled: the table was still a floor, and the registry is the
  load-bearing copy**` paragraph enumerating the 51 declared selectors, plus a
  standing-lesson paragraph. The mirror is updated **as well as**, not instead of,
  the registry.
- **`notes.md`** (`F-15` residual) — new
  `## Reconcile — the source-delta registry, and where it physically lives`:
  the authentic `provenance = "solo"` registry rode the capsule sideband
  (`refs/capsule/a/state/implementation`) into
  `.worktrees/SL-254-audit/.doctrine/state/slice/254/boundaries.toml`, was copied
  to the primary tree, is **runtime tier** so it does not land with the branch,
  and `ISS-350` owns the resolver question.

### Selector registry (the load-bearing fix, `F-8`)

51 selectors declared into `slice-254.toml` via `doctrine slice selector add
--intent design-target`, in three batches. `slice conformance` moved:

| | before | after |
|---|---|---|
| undeclared | 96 | **45** |
| conformant | 40 | **91** |
| undelivered | 0 | 0 |

Every remaining undeclared path is the governance-entity class the brief excludes.
`crates/doctrine-control/src/backend/bubblewrap.rs` was declared rather than
noted — a one-line doc-comment rename is still reach.

### Knowledge-record corrections

Each landed on the prose tier via `doctrine knowledge edit <ID> --body-mode
append`, as a dated `## Correction at reconcile (2026-08-14, RV-356 F-N)` section.
The structured `--consequences` flag was **not** used: it is comma-separated and
would have split the existing comma-bearing sentences destructively.

- **`DEC-204`, `DEC-213`** (`F-3`) — both named `classify_import` as
  `worker-forbidden-writes`'s surviving enforcing reader. True of the two
  hard-coded floors, false of the config key. `worker_commit` is deleted and
  nothing replaced it.
- **`DEC-210`** (`F-9`) — binding `$CFG_DIR` **carries** a subscription credential
  into the jail; it does not **create** one. Records the unstated host
  precondition: a materialised `~/.claude/.credentials.json`, or
  `CLAUDE_CODE_OAUTH_TOKEN` in the environment.

### REVs completed

- **`REV-053`** (`reconcile-sl-254`) — **done**. Two `modify` rows, both
  descriptions that stopped being true; no decision in either ADR changes.
  - `ADR-001` `layering.toml:138` (`F-6`) — the `worktree::jail` row restated from
    *"pure jail core — no disk/git/clock/rng"* to **impurity behind the injected
    `ResolveEnv` seam**, the module's genuine invariant since `SL-183`. The twin
    source comments (`src/worktree/jail.rs:5` and `:19`) landed **in the same
    change**, as the row required — they are the copy a reader of the module
    actually meets. Recorded in the REV that nothing mechanical catches this:
    `tests/architecture_layering.rs` parses crate-module `use` edges and has no
    notion of `std::fs`, so the gate is green either way.
  - `ADR-020` Context (`F-11`) — an `**Amended by REV-053**` paragraph recording
    that `SL-254` discharged two of the four incumbent brittlenesses the capsule
    case argues from (cooperative marker identity; harness-specific spawn paths)
    while the load-bearing pair survives. The decision is unaffected. `:86-88`'s
    authority clause was left untouched per the brief — it stays literally true.
  - Rationale and before/after excerpts in `revision-053.md`.

### `REV-046` — restated, by owner ruling

Owner ruling of 2026-08-14: **the rationale is what predates the slice, and it
should be updated** — the gates' debt is unchanged. `REV-046` is
`proposed · approval=none`, a live draft rather than authored governance truth, so
this landed as a **direct prose edit** to `revision-046.md`, not as a row in
`REV-053`. Four restatement notes:

1. **Rationale ¶2** — four of the mechanisms enumerated as *"unnecessary"* are
   already gone, deleted by `SL-254`, a slice explicitly **not** a capsule
   adoption (`DEC-203`): the disk marker, the `SubagentStart` stamp, the gated
   `worker_commit` exception, and per-harness arm routing and altitude. They were
   not retired *by* `REV-046` and their disposal is not evidence for it.
   **`DOCTRINE_WORKER` moves the other way and comes off the list** — it is now
   the incumbent identity mechanism, set by the confining argv; the capsule target
   *substitutes* it rather than removing a redundancy, and must argue it as one.
   The surviving remainder (base-by-placement, coordination-worktree placement,
   patch import, nominated-unjailed choreography) is what the case now rests on.
   `RFC-025`'s 15 DELETE rows stand unrestated as a faithful record of their
   moment.
2. **The `ADR-011` row** — its grounding clause (*"while the Claude arm uses an
   in-session `Agent`, disk marker, hook stamp, and gated worker commit"*) named
   four deleted things. Restated onto the single confined-subprocess arm
   `ADR-011` now describes; the supersession conjunction also lost half its force,
   since `ADR-020` now exists.
3. **The `ADR-008` row** — both deferrals it carried are **already discharged**:
   `SL-254` retired `D-B6` whole (`adr-008.md:26-30`, `DEC-217`) and deleted
   `worker_commit`, whose note `REV-052` swept (`N1`). What remains is the
   confinement-machinery recast, from a nearer baseline than drafted — `SL-254`
   moved bwrap from enhancement to floor ahead of it.
4. **Cutover gate 4** — a note, not a relaxation. Three of its five named
   mechanisms (marker, hook, `worker_commit`) already have no dispatch-run
   dependents; the gate still binds worktree import and coordination-worktree
   placement, which is where its force always lay.

`REV-046`'s cutover debt is unchanged, and it remains `proposed · approval=none`.

### Cards minted

- **`ISS-351`** (`F-4`) — `scripts/pi-spawn.sh` asserts worker identity with zero
  `bwrap`: `:27` forks, `:53` sets `DOCTRINE_WORKER=1`, `grep -c bwrap` is 0.
  Delete or bring under the confinement prefix — a decision, not a fix.
- **`CHR-063`** (`F-5`) — sweep the 104 live project-local memories that still
  assert retired mechanisms. See the corpus-hygiene disposition below.
- **`CHR-064`** — drop the `base64` dependency `SL-254` orphaned. Verified
  2026-08-14: zero references in `src/`, `tests/`, `crates/`; the two
  `tests/mcp-bridge.test.ts` hits are a TypeScript string literal. Its declaring
  comment (`Cargo.toml:166-168`) names `SL-182 PHASE-02` — whose module `SL-254`
  deleted whole. Cargo does not warn on an unused dep, so no gate sees it.
- **`ISS-350`** (`F-15`) — pre-existing, minted during the audit. Confirmed to
  exist, not re-derived: `boundaries_path()` pins to `primary_worktree(cwd)`
  (`state.rs:943`) while `phases_dir()` joins the local `project_root` (`:135`),
  so an adopted worktree reports `10/10` phases and zero source deltas at once.

`IMP-429` (Darwin arm) was **extended, not minted**: a new
`## What a mac-equipped session should actually test` section folds in `H1`–`H4`
verbatim in substance.

### Handed back rather than written

- **`PHASE-09`'s `VA-2` intent** (`F-2`). The brief asked to re-word it from
  *"check Darwin against Linux"* to *"enumerate every asymmetry"*. `VA-2` is a
  `plan.toml` criterion (`:377`) and criteria ids are **immutable-append** — not a
  reconcile surface. The corrected intent was routed to `IMP-429`'s `H4` instead,
  with the reasoning stated in place. No plan file was touched.

### Corpus hygiene — one retraction now, the rest carded

- **Done in this pass**: `mem.signpost.doctrine.dispatch-claude-arm-wrong-base`
  (`mem_019ee28ee9ee7d608a22dba762fdcc26`) **retracted**, not edited — its whole
  subject, the `/dispatch-agent` in-session arm, was deleted by `SL-254`. It was
  the only one of the 105 **indexed in the boot snapshot**, so every agent met it
  on every boot without retrieving anything, which is why it was handled first.
  `doctrine boot` re-run and its absence from the snapshot's Memory index
  confirmed.
- **Deferred to `CHR-063`** by owner ruling: the remaining **104 live** items
  (105 files match, one of which is now the retracted one above). A
  `/reviewing-memory` sweep is per-item retract / re-anchor / leave judgement —
  a different shape of work from reconcile, and folding it in would hold the close
  open on unrelated calls. Not urgent; nothing gates on it.
- **Shipped corpus (`memory/`) confirmed clean at zero.** Nothing `doctrine
  install` seeds into a client project carries a retired mechanism. The exposure
  is local to this repo.

### Deferred to close

- **Re-run `doctrine boot` on the landing tree** (`F-1` residual). The fix is
  committed at `1185a631d`, but each tree's snapshot is gitignored runtime state
  and regenerates locally — so this is a close-time action on the tree the branch
  lands on, not something a reconcile commit can carry.

### Withdrawn / tolerated

- **`F-7`** — withdrawn during the audit itself, superseded by `F-14` and then
  corrected by `F-15`. There was no missing-source-delta gap: all ten phases
  recorded automatically, the capsule sideband delivered the registry, and the
  apparent emptiness was `boundaries_path()` reading the wrong tree. Its
  replacement card is `ISS-350`. Confirmed at reconcile: `slice conformance`
  resolves and reports, because `F-15`'s copy-across had already been applied and
  this audit worktree is a linked worktree of the same repo.
- **`F-1`, `F-12`** — fixed inside the audit; only `F-1` left a residual (above).
  `F-12`'s `----- worker commit -----` transcript banner needed no reconcile
  surface.

### Gate

`doctrine check gate` — exit 0, 117 `test result: ok` blocks, zero
`test result: FAILED`, zero clippy warnings. Run after the `src/worktree/jail.rs`
comment edit, the only source change in this pass.

Reconcile pass complete — handoff to `/close`.
