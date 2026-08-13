# Notes SL-254: Collapse dispatch onto one subprocess arm

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-13 · PHASE-02 complete (2/9) · see git log

### Produced
- PHASE-02 done — `scripts/pi-spawn-confined.sh` → `scripts/spawn-confined.sh`,
  harness as its first argument, `claude` profile added alongside `pi`
  (DEC-209/210/215/216), Linux `bwrap` probe naming `bwrap-unavailable` ahead of
  the fork (D7/F-6), and `sandbox_exec_argv` now carries `DOCTRINE_WORKER=1` in
  its trailing `env` run (F-2). New `tests/e2e_worker_confinement.rs` proves the
  shipped PREFIX tokens confine live, and skips named where `bwrap` is absent
- PHASE-01 done — the four jail primitives re-homed to `jail.rs`, `jail_prefix.rs`
  re-pointed, `BWRAP_BIN` collapsed onto `jail::BWRAP` (ef59ad0f5). Both relocated
  unit tests byte-identical; `just gate` and `doctrine check gate` exit 0
- capsule execution posture, entity-id collision rule, and the `ADR-001` follow-up
  recorded (f6096d932) — see `## Execution environment and standing directions`
- design locked and nine phases materialised (caf7f2a21)
- minted across scope + design: DEC-202..DEC-218, EVD-023, QUE-214, QUE-215,
  ISS-347, ISS-349, IMP-428, IMP-429, CHR-062, and SL-255 (the clone half)
- ~~**`research/` is GONE**~~ — **RESTORED by the user 2026-08-13**, same session.
  It had been absent when this capsule started (runtime tier, gitignored, so the
  checkout carried no copy). `research.md` reads intact and its baseline pins
  exactly to `5a68bfd9e`. **`raw/` did NOT come across** — the five per-thread
  agent outputs the header cites (`raw/governance.md` …) are still missing, and
  they exist nowhere durable
- the design run `dr-019ff653` snapshot is still gone (runtime loss, not a
  missing run). Authored `design.md` + `plan.toml` stand; `design show` refuses
- 11 friction observations recorded, all committed
- `.doctrine` changes committed promptly throughout, and PHASE-01's code went out
  in its own commit separate from the sheet/notes commit that preceded it

### Learned
- mem.pattern.refactor.move-closure-exceeds-consumer-imports — a re-home set
  derived from the consumer's imports under-counts the definitions' own needs
- mem_019ff650d94a7960a638913a40416165 — collide "what calls this" research
  findings against "what should exist" decisions at synthesis
- mem.fact.dispatch.worker-confinement-is-actor-based — strengthened with the
  process-vs-tree generalisation and re-attested

### Open
- **`bwrap_argv` has the same hole `F-2` fixed on macOS** (PHASE-02 finding).
  The BINARY's Linux prefix builder emits no `--setenv DOCTRINE_WORKER 1`; only
  the spawn script's inline array does. Dormant today because nothing on Linux
  shells `worktree jail-prefix` — but `IMP-429` (the prefix's permanent home) is
  exactly the move that would wake it. Not fixed in PHASE-02: outside every
  criterion, and it would move tokens `EX-2` freezes
- **A Rust unit test consumes the spawn script's TEXT** — `jail.rs`'s core-flag
  parity test opens the script by path and parses its inline array. Neither half
  of the slice's two-direction census (Rust symbols outward, literal strings
  inward) reaches a Rust consumer of a non-Rust file's *text*. Cost one red
  cycle in PHASE-02; re-anchored on `PREFIX=(` and re-keyed to `$CFG_DIR`.
  Observation `019ffbeb-d553` records the generalisable form
- `ADR-001` posture of `jail.rs` — two unseamed impure functions now sit in a
  registered `leaf`; candidate seventh `REV` target for PHASE-08's re-derivation
- macOS/Darwin path unverified — `jail_prefix.rs:40`/`:46` are cfg-stripped in
  this capsule; carried to the AARCH64 sweep, `OQ-5`, PHASE-09 `VA-2`
- entity-id collision risk — ids minted in this capsule can clash with the host's;
  flag every one in the phase sheet and commit message
- `SL-254` `OQ-1` — `pi-spawn-confined.sh:56` passes neither `fork --worker` flag,
  so the collapsed arm inherits unbound forks; a precondition of PHASE-09's `VA-1`
- `SL-254` `OQ-2`..`OQ-5` (`design.md` §6) — none blocking
- deletion boundary widened at drafting (`design.md` §7.2 `D1`-`D3`, §8 `R6`) —
  four surfaces objective 3 does not name; carry into the reconciliation brief
- `SL-254` `OQ-2` backlog dissolution set — IMP-269, IMP-342, IMP-334, IMP-337,
  IMP-407, IMP-401, IDE-024; confirm at reconcile
- memory-corpus sweep at reconcile — 25+ stale claude-arm memories, one in the
  boot snapshot
## Design surface triage
<!-- `explore.triage` runbook step, design run dr-019ff653, 2026-08-13 -->

Ten inquiry nodes declared and resolved: `DEC-203`..`DEC-212`.

### Shaping decisions

| node | record | settled |
|---|---|---|
| inq-1 | `DEC-203` | Bounded Pole B — worker side goes capsule-shaped, `ADR-012`'s coordination topology untouched |
| inq-2 | `DEC-209` | Generalise the incumbent prefix; take no `doctrine-control` dependency, port no hardening |
| inq-3 | `DEC-204` | `worker_commit`'s **transport** retires; two of its six belts re-home to the fetch/admit step |
| inq-4 | `DEC-210` | Subscription credential, `$HOME/.claude` bound wholesale like pi's `$HOME/.pi`; narrowing deferred |
| inq-5 | `DEC-205` | nominate/denominate die with `pretooluse` — closed loop, not a separate choice |
| inq-6 | `DEC-206` | Re-home four jail primitives to `jail.rs` as the FIRST step, before any deletion |
| inq-7 | `DEC-207` | Identity collapses to the env leg; delete the marker and `describe_mode`'s `is_linked` conjunct |
| inq-8 | `DEC-208` | One arm. No degraded in-session rung; fail closed at spawn as the pi arm already does |
| inq-9 | `DEC-211` | One REV over `ADR-011`, `ADR-006`, `SPEC-021`, `SPEC-012` + a hand anchor sweep |
| inq-10 | `DEC-212` | Behaviour-preservation restated as the funnel's observable contract, not "suites unchanged" |

### Governance constraining the surface

Read directly this stage rather than through the research round's quotations —
`DEC-211`'s body carries the site-by-site detail.

- **`ADR-011`** — **eight** amendment regions (superseding this section's earlier
  "five", which mis-cited 263): Context (21-25), `D1` (39-56), `D2` (58-74),
  `D3`'s table (82-92), `D4` (94-111), `D6` (146-207), Consequences (248-259),
  Verification (265-275). `D5` (113-144) and `D7` (209-232) considered and
  deferred. Site-by-site detail in `DEC-211`'s `choice`.
- **`ADR-006` §D2b** — thread 1's claim verified; the `SL-181` note pattern is real
  at line 143. Second site at 308. Its "degenerate case" list already contemplates
  a standalone clone — `SL-254` makes that the *normal* case, so the note re-cuts
  rather than appends.
- **`ADR-012`** — **not touched**, per `DEC-203`. Load-bearing boundary: it is what
  keeps the REV inside the surveyed set.
- **`POL-002`** — satisfied, not strained (`DEC-206` keeps the argv builder in the
  binary and harness specifics at the script tier).
- **`STD-001`** — governs the thoroughness of the `claude-force-subprocess-dispatch`
  deletion and of the belt consts' single-sourcing.

### Corrections this stage made to prior artefacts

- **`DEC-202`'s `choice`** — corrected in place: it read as a claim about worktrees.
- **The scope's behaviour-preservation gate** — "codex/pi suites stay green
  unchanged" is unachievable once the pi arm moves to clones too (`DEC-212`).
- **The scope's governance survey** — missed `SPEC-012`'s responsibility *prose*
  (`spec-012.toml:18`, `fork`'s "stamp", `import` as "the belted dispatch funnel")
  and `ADR-011`'s fifth site.
- **Research thread 3's auth recommendation** — `ANTHROPIC_API_KEY` forfeits the
  subscription billing `EVD-023` establishes (`DEC-210`).
- **Research thread 2's `worker_commit` reading** — "every check is meaningless in a
  clone" is wrong on two of six belts (`DEC-204`).

### Newly surfaced — NOT in the scope's reconcile survey

- **The memory corpus.** At least 25 memories describe claude-arm mechanisms this
  slice deletes — `SubagentStart` stamping, `PreToolUse` jail behaviour,
  `WorktreeCreate` provisioning, `worker_commit` resolution, marker identity —
  several at `high` trust / `high` severity. After the collapse they are
  **stale-but-plausible**, the most dangerous class: an agent retrieving
  `mem_019ebfb61ba870219aafc14f8dc7da3b` ("worker identity via `SubagentStart`
  hook") would act on a deleted mechanism. `mem.signpost.doctrine.dispatch-claude-arm-wrong-base`
  is indexed in the **boot snapshot**, so the staleness reaches every session's
  context. Needs a deliberate `/reviewing-memory` sweep at reconcile — carried in
  the scope's Follow-Ups.

### Open / deferred

- `IMP-428` — harden the worker confinement prefix (deferred out by `DEC-209`).
- Clone disk/time cost for this repo: still unmeasured. `POL-002` forbids baking a
  local measurement into the platform regardless.
- `../microvm-spike`'s narrowed `~/.claude` mount set: deferred by `DEC-210`; its
  identity-section partial is not definitively proven.

## Review passes — RV-355 and what a further pass would probe

*Written 2026-08-13, after the first pass, per the reviewing runbook's
`review.passes` obligation.*

**Pass 1 (done).** External adversarial, codex/GPT-5.5, against the design at run
`dr-019ff653` rev 41. Seven findings, all verified against the code before
disposition, all dispositioned `design-wrong` — the defects are in the design
artefact, not the implementation. Three blockers: `F-1` (Mode B loses its funnel
entry point), `F-4` (`SPEC-012` carries four falsified requirements, not one
responsibility), `F-5` (`ADR-012`'s harness-synthesis rule names the deleted arm
normatively). The pass also reported eight claims it checked that held — the
`require_binding` census, the nomination closed loop, `verify-worker`/`arm-spawn`
having no surviving consumer, `classify_import`'s single-sourcing, the
pi-specificity of the fifo/reap machinery, and no reachable unconfined fallback.
That negative half is worth as much as the findings: it is what makes silence in
those areas informative.

**What a second pass should probe, once the `F-1`–`F-7` fixes land.**

1. **The widened governance landing.** `F-4` and `F-5` grow the `REV` from four
   target entities to five and `SPEC-012` from one responsibility to four
   requirements plus that responsibility. The survey has now been wrong twice in
   the same direction — under-counting — so a second pass should re-derive the
   target set from the entities rather than from `DEC-211`, and specifically ask
   what else in `SPEC-012` and `ADR-012` the first two sweeps missed.
2. **Mode B's retirement, once it is written down.** `F-1`'s remedy records the
   confined-orchestrator funnel arm as retiring with the in-session arm. That is
   a consequence nobody chose directly — it falls out of the unbound-fork
   settlement — so it deserves adversarial attention in its own right: does
   anything else ride Mode B, and is `REQ-335`'s "stays pending" still honest
   when the substrate it was pending on is deleted?
3. **The macOS arm as a whole.** `F-2` found the seatbelt argv missing
   `DOCTRINE_WORKER` — the one place `OQ-5`'s "parity holds by construction" was
   false. One instance of a class is not a census. A pass over the Darwin path
   specifically, holding it to every claim §5.1 and §5.2.1 make about the two
   profiles differing in exactly two tokens, is the obvious next probe.
4. **What a design pass structurally cannot reach.** Nothing here exercises a
   real `claude -p` under bwrap. `A2`'s residue is provisioning, and provisioning
   fails at runtime or not at all — that is the `VA` leg's job, not a reviewer's.
   A second design pass should not spend effort simulating it.

**What a second pass should not re-litigate.** `DEC-213`'s split into `SL-255`,
and the unbound-fork settlement it forced. Both are owner decisions taken on
stated evidence. A finding that this slice should have taken the clone half, or
should bind its forks, is a finding about those decisions — admissible as one,
but not as a defect in the draft.

---

**Pass 1 closed 2026-08-13, rev 42 → 49.** All seven findings verified as raiser
and `RV-355` concluded. Every section body moved, so the pass is `STALE` against
current content by construction — a second pass reviews a different artefact, not
a patched one.

*Probe 1 above was discharged in the course of the fixes, and it found more than
it expected.* Re-deriving the target set from the entities rather than from
`DEC-211` grew the `REV` from four entities to **six**, not the five `F-4`/`F-5`
implied: `ADR-008` was absent from every prior survey and carries seven regions
(`D-B3`'s "codex/pi-only … not a subprocess to wrap" clause, `D-B6`'s whole
nominate/gate/`SubagentStop` mechanism, `N1`'s `worker_commit` exception);
`ADR-006` is nine regions rather than two, reaching `D2a`'s decision body and both
`D9` amendments; `ADR-011` is eleven rather than eight. Recorded as `DEC-218`,
which supersedes `DEC-211`'s enumeration and carries the derivation *method* so
the next reader re-derives rather than re-cites. `R8` and the second `VH` claim
(§9.4) are the standing guards.

*Probe 2 is now written down as `DEC-217`* — Mode B retires with the in-session
arm, as a consequence of the unbound-fork settlement rather than a choice. Its
two sub-questions are answered in the record: what else rides Mode B is
`SPEC-021` `REQ-384`/`REQ-387`, both narrowed in the `REV`; and `REQ-335`'s
"stays pending" is honest **as a contract** while its one partial implementation
retires, which the scope's Non-Goals now say explicitly. Still worth adversarial
attention, but against a stated position rather than a silence.

*Probe 3 (the macOS arm as a census, not one instance) is untouched and remains
the sharpest open probe.* `F-2`'s fix is one token in `sandbox_exec_argv`; nobody
has swept the Darwin path against every claim §5.1 and §5.2.1 make.

**New, found while fixing, not by the pass.** Two `doctor` checks are live
consumers of deleted symbols and were in no earlier list: #10 `SpawnSeamSymmetry`
reads `PRIVILEGED_AGENT_TYPES` and the `SubagentStart`/`PreToolUse` registries, so
**the deletion does not compile** without removing it; #9 `AgentConformance`
allowlists `mcp__doctrine__worker_commit` as the worker's one MCP token, which
`DEC-216` leaves empty. `src/finding.rs` loses a `Category` variant with #10. A
second pass should ask what *else* consumes a deleted symbol — the design's §5.6
is a hand-built list, and this was the second sweep to extend it.

---

## The symbol census — sixth sweep of §5.6, and the method change

*Run 2026-08-13 at rev 49, before section attestation, so §5.6's edits do not
stale an attestation twice.*

The `doctor_checks.rs` find above was a compile-breaker sitting behind a symbol
the design had already marked for deletion — which said the *method* was wrong,
not that one entry was missing. Sweeps one to five read the code and asked "what
does this change touch", which is answered from a mental model. The sixth
inverted it: enumerate every `pub` item in the three dying modules and in
`jail.rs`, grep each across `src/` and `tests/`, subtract consumers that are
themselves dying. Mechanical, and it reproduced `doctor_checks.rs` as a positive
control before finding anything new.

**What it found.**

1. **`jail.rs` loses its entire pure decision layer** — ~300 of 981 production
   lines and about half its ~60 unit tests. `pretooluse.rs` was the sole caller of
   `resolve_target`, `decide_agent`, `decide_workflow`, `decide_bash`,
   `decide_write`, `pathcheck`, `opaque_wrap`, `shell_single_quote`,
   `acquire_policy`, `resolve_inputs`, `seatbelt_backend`, `is_privileged_agent_type`,
   `enum Decision`, `enum Target` and five `REASON_*` constants. `pub(crate)` with
   no caller is `dead_code`, and `just gate` is clippy-at-zero-warnings, so this is
   a gate-breaker in the same class as `doctor_checks.rs`. §5.6's old jail.rs row
   read as a four-item edit; it is a third of the module.
2. **`src/commands/guard.rs` is where the write-class registry lives**, not
   `main.rs` as §5.6 said. Its `Marker { stamp_subagent: true }`, `Nominate` and
   `Denominate` arms don't compile once those `Command` variants go; the bespoke
   `WriteClass::MarkerClear` class retires with `run_marker_clear`; and
   `worker_guard`'s two-branch dual-cause message collapses when the marker leg does.
3. **`justfile:37`** — `validate`'s worker-context skip has a marker-file leg
   alongside the two env legs. Non-Rust, so no symbol grep would have found it; it
   surfaced only by grepping the literal path string.
4. **Six unlisted test files**, one of them (`e2e_dispatch_arm_spawn.rs`) an
   outright delete, plus `tests/common/mod.rs`, whose fixture helper *stamps* the
   marker.

**Two positives worth as much as the findings.** `F-2`'s fix is on live code —
`sandbox_exec_argv` is reached from `jail_prefix.rs:168` via `Seatbelt::wrap_argv`,
not from the dying wall, so the census does not undercut it. And `DEC-206`'s
re-homing set is provably complete: `jail_prefix.rs` imports exactly those four
primitives from `pretooluse.rs` and nothing else. ~~No fifth primitive is
hiding.~~ — **qualified 2026-08-13 at `/phase-plan` PHASE-01.** True of
`jail_prefix.rs`'s *import list*, which is what it was derived from. The
**move's** closure is wider: `have_bwrap` reads two private `pretooluse` consts,
`BWRAP_BIN` (`:71`) and `ENV_PATH` (`:77`), and `BWRAP_BIN` is an `STD-001`
duplicate of a constant `jail.rs` already owns (`:73`, `const BWRAP`). Not a
design defect — a scope-of-claim correction, and the ninth time a survey here
has read low.

**Method limits, recorded so the plan phase inherits them.** A negative grep is
worthless without a positive control — `e2e_priority_golden.rs`'s three
`denominate` hits are the arithmetic denominator, and `boot.rs`/`relation_graph.rs`
both define unrelated `resolve_target` functions. And the census sees Rust symbols
only; every non-Rust consumer (`justfile`, `.claude/settings.json`, the scripts,
the `spec-*.toml` source anchors) needs a separate literal-string sweep.

**Probe 3 (the macOS/Darwin census) is still untouched** and remains the sharpest
open probe for a second adversarial pass. This sweep touched the Darwin path only
where it intersected the orphan question.

## Execution environment and standing directions (implementation sessions)

Established by the user at `/phase-plan` PHASE-01, 2026-08-13. These are facts
about *where this slice is being built*, not about what it builds — but several
of them invert assumptions the design and the skills would otherwise apply.

**This is a microVM capsule holding a separate checkout.** Consequences, each of
which switches off machinery the repo's standing guidance would otherwise
demand:

- **No dispatch, no worktree, no fork.** No main-worktree contention to isolate
  from, so `/worktree` and `/dispatch` are not in play — work happens directly
  in this checkout. The `edge`/`main` promotion ritual and the
  "never checkout the primary tree" rule are about the host repo, not here.
- **No confinement hooks.** The `PreToolUse` wall, the worker marker and the
  `DOCTRINE_WORKER` env leg are all *subjects* of this slice here, never active
  constraints on the session doing the work. A confinement behaviour observed —
  or not observed — in this capsule is evidence about nothing.
- **Transient state is not visible from the host.** Anything the host session
  would have seen in `.doctrine/state/` is absent or divergent here. Do not
  reason from its absence.
- **Claude is available two ways** — `claude -p` and in-session subagents. `pi`
  is installed but has **no API keys and no egress route configured**, so
  `./scripts/pi-scout` / `pi-research` will not work until the user sets that
  up. Research that needs them has to wait or route through claude.

**Entity-id collision risk — flag every minted id.** Ids minted in this capsule
(`DEC-`, `RV-`, `IMP-`, `ISS-`, `EVD-`, …) can collide with ids minted
concurrently on the host, because the reservation surfaces are not shared.
**Memories and observations are exempt** — those are uid-keyed, not counter-keyed.
Standing rule for the rest: **any id minted during implementation gets called out
explicitly in the phase sheet and in the commit message**, so reconciliation can
find and re-key them. Prefer not minting at all where a phase-sheet note or a
`notes.md` entry carries the same weight.

**Adapt the plan; keep receipts.** User direction, verbatim in substance: given
the choice between following the plan slavishly and doing the thing properly,
choose properly — and record the divergence. Two immediate applications:

1. **`PHASE-01`/`VA-1`'s byte-identity leg is relaxed.** The authored criterion
   asks that the two relocated `write_seatbelt_profile` unit tests be diffed
   line-by-line and be "identical but for the module they sit in". The user is
   not uptight about existing tests holding at the byte level. **What survives
   is the substance**: a relocated test whose *assertions* changed means the move
   was not behaviour-preserving and the phase has failed. Import-form and
   symbol-reference churn (`fs::` qualification, `BWRAP_BIN` → `jail::BWRAP`) is
   not a failure. `EX-4`'s `git diff --stat tests/` leg is **untouched** and
   still binds — that is about the e2e suites, which are the real
   behaviour-preservation proof.
2. **`PHASE-01`/`D1` is settled on the collapse** — see below.

**macOS is acknowledged and deferred.** `jail_prefix.rs:45`/`:47` are
`cfg(target_os = "macos")` and are not compiled in this capsule, so the
re-pointed imports there pass a green gate unverified. The user will sweep the
Darwin path in a later phase on AARCH64. This is `OQ-5`'s residual and design
§9.5's "deliberately not verified here", arriving one phase earlier than the
design anticipated; `PHASE-09`/`VA-2` (probe 3, the Darwin census) is where it
lands.

### Follow-up: `jail.rs`'s ADR-001 posture

**The finding.** `.doctrine/adr/001/layering.toml:138` classifies
`"worktree::jail" = "leaf"` with the comment *"pure jail core — no
disk/git/clock/rng"*, and `jail.rs`'s own module doc opens *"PURE leaf: no clock
/ git / disk / rng"*. Two of `DEC-206`'s four re-homed primitives are impure:
`have_bwrap` stats the filesystem (`dir.join(BWRAP).is_file()`) and reads the
environment; `write_seatbelt_profile` calls `fs::write`. The design does not
mention purity, ADR-001, or the leaf classification anywhere — the re-homing set
was derived from `jail_prefix.rs`'s imports, and the question of whether the
destination could hold them was never asked.

**Why PHASE-01 proceeds anyway.** The stated contract is already not the real
one. `jail.rs` houses `impl ResolveEnv for RealEnv` (`:865-916`), which shells
`getconf` and makes three `std::fs` calls. The module's genuine invariant is
*"impurity behind the injected `ResolveEnv` seam, so the pure surface stays
`FakeEnv`-testable"* — not "no I/O". The two arrivals are **unseamed free
functions**, which is new for the module, but routing them through `ResolveEnv`
would change `jail_prefix.rs`'s call sites, and PHASE-01 is defined by not doing
that. So: move verbatim per `DEC-206`, record here.

**Nothing mechanical catches this.** `tests/architecture_layering.rs` parses
crate-module `use` edges and checks tier direction; it has no notion of
`std::fs`. The gate is green either way — which is the reason to write this down
rather than the reason to let it go.

**What to do about it, in order of preference.**

1. **`PHASE-08` candidate target.** That phase re-derives the governance `REV`
   target set from `.doctrine/adr` against the corpus as it then stands (`EX-1`,
   `VH-2`), and design §5.6's six-entity table does not name `ADR-001`. This is
   exactly the "any entity or region beyond the six is **recorded, not
   absorbed**" case. At minimum `layering.toml:138`'s comment wants correcting to
   describe the seam rather than assert an absence that has not held since
   SL-183.
2. **If `PHASE-08` declines it**, a backlog card: seam `have_bwrap` and
   `write_seatbelt_profile` behind `ResolveEnv`, or split them out to a
   `worktree::jail_io` command-tier sibling. Either restores the stated contract;
   neither belongs inside a behaviour-preserving re-home.

**Standing caution for the rest of the slice.** `PHASE-04` deletes ~300 lines of
`jail.rs`'s pure decision layer and about half its unit tests. What is left is
disproportionately the impure remainder plus the argv builders. The leaf
classification deserves a deliberate look at that point rather than an inherited
one — the module that emerges is not the module ADR-001 classified.

### PHASE-01 `D1` — settled: collapse `BWRAP_BIN` onto `jail::BWRAP`

Once `have_bwrap` moves out, `BWRAP_BIN` (`pretooluse.rs:71`) has exactly one
remaining reference — a unit test at `:516` — so it is dead under
`cfg(not(test))`, and `Cargo.toml:206`'s `warnings = "deny"` makes that a hard
compile error rather than a lint.

**Chosen:** delete `BWRAP_BIN`, promote `jail.rs:73`'s existing
`const BWRAP: &str = "bwrap"` to `pub(crate)`, point the moved `have_bwrap` at
it, and retarget the one test reference. **Rejected:** keep `BWRAP_BIN` alive
under a broadened `#[cfg_attr(not(test), expect(dead_code, …))]`.

The only argument for the rejected route was preserving a staying test's text
byte-for-byte, and the user has released that constraint. What remains is
one-sided: the collapse **removes** a live `STD-001` duplicate of the literal
`"bwrap"` across two modules of one subsystem, where the annotation route spends
an `expect(dead_code)` to **keep** it — and keeps it only until `PHASE-04`
deletes `pretooluse.rs` whole, at which point the duplicate would have gone
anyway. Paying a lint suppression for three phases of duplication is the worse
trade in every dimension. The test's assertion is untouched; only the symbol it
names changes.
