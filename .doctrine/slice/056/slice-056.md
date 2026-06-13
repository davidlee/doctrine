# Orchestrator spawn seam: worktree mechanism into CLI verbs

## Context

Thesis (surfaced by SL-055's holistic skills review): **mechanism in prose is
the design smell.** Today the worktree/dispatch CLI owns three small read-verbs
(`provision`, `check-allowlist`, `branch-point-check`); everything else — the
fork-creation ladder, the import funnel (precond → `S^ == B` assert →
single-non-merge → R-5 `.doctrine/`-touch belt → `apply --3way` non-committing),
worker-mode enforcement, build isolation, GC — lives as **prose asking an LLM to
replay git command sequences**. That prose is token-heavy (worktree 347 lines,
dispatch 312 — the two fattest skills), untestable (no golden pins a prose
ritual), fail-open (a skipped step refuses nothing), and harness-coupled by
accident (written against Claude's `Agent` affordances). A CLI verb is identical
under claude/codex/pi by construction — each mechanism moved into the binary
makes skills shorter **and** more agnostic in the same move.

**Keystone insight — orchestrator-owned fork + disk marker (harness-agnostic).**
The thesis' candidates #1, #2 and #5 turn on *who controls the worker's worktree
and identity*. The trust-bearing, harness-agnostic core is the **orchestrator
owning fork creation and stamping a disk marker** into the worker's worktree
before the worker runs: identity rides *disk* — which every harness has — not a
process env seam, which not every harness has. The `fork`/`import`/`gc` verbs and
the marker are identical under claude/codex/pi by construction.

**The spawn *backend* is a harness concession, not the keystone (Charge XIII).**
The original thesis made *orchestrator-spawns-subprocess* (`claude -p` /
`codex exec`) the keystone, on the premise it bought env-arm + per-wt
`CARGO_TARGET_DIR` + bwrap uniformly. That premise is false for the dominant
harness: **`claude -p` is Anthropic-API-billed, not subscription** — fanning out N
short-lived workers is economically prohibitive — and it is harness-specific, so
it cannot be a *required* skill element (a harness-agnostic framework admits no
harness-specific required command). For claude the only viable backend is the
in-session `Agent` tool, which exposes **no env seam and no exec wrapper**.
Subprocess spawn therefore demotes to a **codex/pi enhancement layer**; claude
runs the agnostic core (fork+marker+import+gc) with the marker as its *primary*
identity signal and `Agent` as a *first-class*, not degraded, backend.

**Per-harness capability layering, not a uniform altitude.** Three enhancements
ride the subprocess seam and are therefore codex/pi-only until a free claude env
backend lands (channels — IDE-004, optimisation tier, never required): (a) env-arm
`DOCTRINE_WORKER` (a *codex/pi optimisation* of the marker, not the identity
itself); (b) per-worktree env provisioning — a **generalisable** framework contract
of which doctrine-the-repo's `CARGO_TARGET_DIR` (ADR-008 D-B1) is one
*project-local* consumer, never a framework primitive; (c) nested bwrap wrap
(ADR-008 D-B3). base-pinned fork (d) is delivered by the `fork` verb for **every**
harness, independent of spawn backend. Multi-harness is real today — the jail
stages `jailed-pi` (depth-2 self-subagent), `jailed-claude`, `jailed-codex`; the
agnostic core has live customers on all three, the subprocess enhancements on
codex/pi.

**Parked governance this slice activates.** ADR-008 (status `proposed`, absent
from the boot accepted list) already designs D-B1 (per-worktree target ≡ thesis
#5) and D-B3 (nested bwrap — the OS big-brother of thesis #2, the intended
discharge of ADR-006 D2b), with IMP-004 as its spike. IMP-004's stated
precondition ("not exercised until /dispatch runs on doctrine — IMP-003") is now
met (IMP-003 resolved). ADR-008 predates the SL-050/051 dispatch experience
(research §5.1 false-green/false-red evidence is exactly what its spike wanted to
learn), so an **accept/revise pass is the natural first gate.**

Enforcement composes across altitudes, but the **reachable** altitude is
**per-harness** (Charge XIII) — the slice does not claim a uniform fail-closed
inversion:

| Layer | Mechanism | Fails | claude | codex/pi |
|---|---|---|---|---|
| CLI — identity | disk marker (orchestrator-stamped) → guard refuses write/Orchestrator verbs | closed | ✓ primary | ✓ |
| CLI — worker-on-main | orchestrator-set `DOCTRINE_WORKER` env (catches a worker dropped on the coordination root) | closed | ✗ no env seam — residual D2b; mitigated by always-isolate + marker-via-SubagentStart-hook | ✓ |
| OS — confinement | nested bwrap, rw only worktree+target (discharges D2b) | closed | ✗ no subprocess to wrap | ✓ (spike) |

The CLI *identity* rung (the disk marker) is the agnostic floor every harness
reaches. The worker-on-main env catch and the OS confinement rung are codex/pi
enhancements; for claude they degrade to the already-deferred ADR-006 D2b residual
(worker-on-main mitigated by always isolating + marker-via-SubagentStart-hook, not closed).

## Scope & Objectives

Consolidated theme: **trust topology + spawn seam, carried by CLI mechanism
verbs.** In scope:

**Governance deliverables (G-group) — design-phase outputs, decisions govern.**
Produced as *drafts within this slice*, not assumed; ordered first because they
govern the code (sequencing below). Scope audit (verdicts grounded in ADR-006,
ADR-008, SPEC-012):

- **G1 — ADR-008 revise → accept** (the gate). Still `proposed`, predates the
  SL-050/051 dispatch experience. Fold the research §5.1 evidence cluster (it
  validates D-B1's premise empirically); record D-B2 as a *standing structural
  fact* (flake already ro-binds `~/.cargo/bin/doctrine` ⇒ in-jail `cargo install`
  impossible — no race because no install); re-scope D-B3's spike around the
  nested-userns feasibility question (OQ-2). **Acceptance gates the IMP-004 work**
  — the natural first phase of SL-056.
- **G2 — ADR-006 amendments** (accepted; targeted edits, not a rewrite — the
  withheld-tier model D1/D4/D9 invariants are preserved). Two amendments:
  (a) **D5/D9 creation-ladder preference** — demote harness-native from "prefer"
  to opportunistic/never-depended (base-pinning + subprocess-spawn kill it for
  workers); align the ADR to exercised reality, citing SL-050/051.
  (b) **D2a mechanism** — replace the `DOCTRINE_WORKER=1` self-arm with a
  **fork-resident worker-marker** written by the orchestrator at fork creation.
  This resolves the D6a tension: D6a's principle is "the *mode*, not the location,
  decides who may write" — a marker the *trusted* orchestrator writes before any
  worker exists is still mode, just recorded durably in the fork instead of
  self-armed env. Fail-closed for dispatch; solo forks get no marker; D6a intact.
- **G3 — ADR-011 (new): the spawn seam.** ADR-006 is silent on *how* workers are
  spawned; subprocess-with-env-seam vs harness-native sub-agent is a
  framework-level (harness-agnostic) decision deserving its own context. One
  coherent decision: orchestrator-owned fork + subprocess spawn + what that buys
  (worker env-arm, per-worktree `CARGO_TARGET_DIR`, bwrap wrap). ADR-006-references;
  cleaner than a D10 bolt-on.
- **G4 — SPEC-012 rewrite** (`draft`; all-pending FRs ⇒ cheap now, expensive
  later). Its container philosophy — "what ships is exactly the machinery the
  funnel *cannot* enforce in prose… the funnel is a discipline, not enforced code"
  — is the exact sentence the thesis attacks. Reframe Overview + Concerns; rewrite
  D3 (currently "fails open at the env boundary") to the fail-closed marker; add a
  D for the verb family; add FRs (import verb, land verb, gc verb,
  orchestrator-fork verb, marker guard).

Untouched governance (every proposal preserves exclusion-by-construction):
ADR-007 (turn-based ledger — orthogonal), ADR-001/003/004 (layering/loop/relations),
the withheld-tier model.

**Mechanism objectives (carried by the G-decisions):**

- **O2 — `doctrine worktree fork` verb** (#1 — *ADR-006 compliance, not
  amendment*; design DC-1/D1). D9 already mandates the orchestrator provision +
  baseline-verify "before handing the worker its task"; worker-self-fork lives only
  in the *skills* — drift from ADR-006, so the strongest move is compliance. The
  verb does create + provision + marker + **emit the per-wt env contract on stdout**
  atomically (codex/pi path; **claude does not call `fork` — it stamps the marker via a
  SubagentStart hook keyed on the dispatch-worker `agent_type`**, the empirically-verified
  mechanism — Charge XIII, [[mem.pattern.dispatch.claude-subagentstart-worker-identity]]).
  The orchestrator's only harness-specific act is the spawn line
  (`env -C "$D" $fork_env codex exec …` / `Agent`+SubagentStart-hook), selected by the
  `/dispatch-*` router. Seam boundary locked at env-emission (mechanism) vs invocation
  (concession).
  Eliminates the fork-from-session-HEAD trap class
  (`[[mem.pattern.dispatch.fork-rung3-base-not-session-head]]`) by construction.
  Spawn backend decided in G3.
- **O3 — Fail-closed worker write-guard** (#2; mechanism of G2(b)). The guard
  refuses doctrine-mediated authored writes **and** the new `Orchestrator`-class
  verbs (fork/import/gc) under the **DC-2 signal** — the orchestrator-stamped
  **disk marker is primary** (harness-agnostic, reusing the `is_linked_worktree`
  precedent); orchestrator-set `DOCTRINE_WORKER` env is a **codex/pi optimisation**
  that additionally catches worker-on-main (Charge XIII: claude has no env seam, so
  this leg is codex/pi-only). Inverts the contract from prompt-self-arm-fail-open to
  orchestrator-stamped; retires the C-I prose contortion. **Validated by a
  guard-spike before G2 amends ADR-006** (Charges III/IV/IX); the spike also gates
  the env-propagation claim for the codex/pi optimisation (Charge III). Marker
  lifecycle owned (written by `fork --worker` (codex/pi) / the SubagentStart hook
  (claude), removed by gc, cleared by a non-Orchestrator verb — Charge II).
- **O4 — `doctrine worktree import`** (#3). Collapse the funnel's deterministic,
  judgment-free steps (clean+`HEAD==B` precond, `S^==B` assert, single-non-merge
  check, R-5 name-only belt, `apply --3way` non-committing) into one fail-closed,
  golden-testable verb. ~60 lines of dispatch prose become "run import; on
  refusal, report+halt."
- **O5 — `doctrine worktree gc`** (#4, answers IMP-041 — and D-B1 hygiene in one
  verb). Remove spent worktrees whose branch is merged/imported, delete the
  branch, **reap the worktree's `wt/<branch>` target dir** (the disk cost of O6),
  warn re stale `env!(CARGO_MANIFEST_DIR)` test binaries. Makes cleanup ownership
  trivial: caller of fork owns gc. IMP-041 and D-B1 disk-reclamation are the same
  verb.
- **O5b — `doctrine worktree land`** (round-4 Charge α). Solo `/execute`'s analog of
  `import`: a fail-closed, `Orchestrator`-classed, **structurally non-squash**
  (`--no-ff`) verb that lands a solo isolated-worktree TDD branch onto the
  coordination branch with ancestry preserved. Closes the unmechanised-prose smell
  (solo's land was "normal git merge" — an unenforced wish); a squash-merge is
  structurally uncertifiable by gc, so the verb cannot express one. gc gains a
  **two-leg** landed check (ancestry for `land`, patch-id for `import`). Both landing
  routes are now verbs — the mechanism-in-the-verb thesis holds for both callers.
- **O6 — Per-worktree env provisioning (generalisable) + build isolation as its
  project-local instance** (#5 ≡ ADR-008 D-B1). The framework primitive is an
  **orchestrator-injected per-worktree env contract**, deliverable only where the
  spawn backend carries env (codex/pi subprocess; claude only via a future channels
  backend — IDE-004). Doctrine-the-repo's `CARGO_TARGET_DIR` keyed
  `…/doctrine-target-jail/wt/<branch>` is one *project-local* consumer of that
  contract (ADR-008 land, **not** the ADR-011 framework primitive). Where the env
  seam exists it obsoletes the three §5.1 mitigation rituals; **where it does not
  (claude), the worker shares the jail-wide target and the §5.1 rituals stand** —
  build isolation is a perf/false-green concern, not a trust signal, so this
  degradation is confessed, not closed. Not baked in the flake (ADR-008 D-B5; cargo
  env-precedence). **No flake change needed for the spike.**
- **O7 — D-B3 bwrap spike** (ADR-008, timeboxed; **codex/pi-only** — needs a
  subprocess to wrap, which claude's `Agent` tool is not — Charge XIII). Spike
  nested bwrap (rw only worktree+target; `bubblewrap` is already pre-staged in
  `jailPkgs`). Feasibility
  gate is unprivileged userns *inside* the jail — the outer bwrap may seccomp-block
  `clone(CLONE_NEWUSER)`; the one-liner empirical probe is
  `bwrap --unshare-user --ro-bind / / true` run in-jail. Land → OS-enforced D2b
  discharge; too costly → back out to D-B1 + D2a CLI guard (O3) and leave D2b
  deferred. Depends on O6.
- **O8 — Skill prose carriers updated + harness-routing split (SL-056 owns it).**
  worktree/dispatch/execute prose rewritten to *call* the new verbs, not restate
  the rituals (the token/agnostic payoff). Plus the **dispatch harness-routing
  split** (Charge XIII): `/dispatch` becomes a thin router that detects the harness
  capability profile and hands off to **`/dispatch-subprocess`** (codex/pi:
  fork-verb + subprocess spawn + env-arm + per-wt env + bwrap) or
  **`/dispatch-agent`** (claude: `Agent` tool spawning a `dispatch-worker` subagent +
  SubagentStart-hook marker, marker identity, **concurrent-safe** — no serial-only
  constraint, the empirical redesign — no env/bwrap). The agnostic cadence stays in the
  CLI verbs so the sub-skills are thin. (Distinct from SL-055's *audience-homing* split
  — Non-Goals.) Ships the Claude surface via a **renamed `claude install`** (ex-`skills
  install`): skills + the `dispatch-worker` agent def + the SubagentStart hook entry.

## Non-Goals

- **Worktree skill *audience-homing* structural split (O4 of SL-055, thesis #6)**
  — re-homing reader audiences is prose-only, SL-055's declared domain. SL-056 owns
  the orthogonal **harness-routing** split (`/dispatch-subprocess` vs
  `/dispatch-agent`, O8) and updates prose to call the verbs; it does not re-home
  audiences.
- **Remote/shared-store workers (research §9, C-VI)** — `format-patch`/`am` path
  through the same cadence. Noted, not specified here.
- **Redesigning dispatch funnel *semantics*** beyond moving deterministic steps
  into verbs and inverting the spawn/guard topology. The cadence order
  (ADR-006 D7) is preserved.
- **`sccache`** (ADR-008 D-B4) — deferred until cold builds actually hurt.
- **`/dispatch` routing slot** (research §9) — deferred until the path is
  exercised.

## Affected surface

- `src/worktree.rs` — new `fork`, `import`, `gc`, `land` verbs + marker helpers + the
  worker-mode observability surface; third `is_linked_worktree` consumer (ADR-001
  leaf; preserve pure/imperative split).
- `src/main.rs` — CLI wiring + read/write classification for the new verbs.
- `src/git.rs` — any new git reads behind the verbs (the impure seam).
- `src/skills.rs` — **rename `skills install` → `claude install`** + an agents leg
  (symlink `install/agents/claude/*.md` into `.claude/agents/`); update the audit label
  + goldens. The `marker --stamp-subagent` gate verb lives in `src/worktree.rs`.
- `src/boot.rs` — a **SubagentStart** `HookSpec` reusing the existing merge core, wired
  by `claude install`.
- `install/agents/claude/dispatch-worker.md` — **new** dispatch-worker subagent def
  (name + description + tool allowlist; user-serviceable markdown, not a Rust type).
- **Governance (the G-deliverables):** ADR-008 (G1, revise→accept), ADR-006
  (G2, amend D5/D9 + D2a), **ADR-011 (G3, new — spawn seam)**, SPEC-012 (G4,
  rewrite Overview/Concerns/D3 + new D + new FRs).
- Skill prose: `plugins/doctrine/skills/{worktree,dispatch,execute}/SKILL.md`
  + the new harness-routing sub-skills `plugins/doctrine/skills/{dispatch-subprocess,dispatch-agent}/SKILL.md`
  (+ re-embed ritual `[[mem.pattern.distribution.skill-refresh-command]]`).
- `flake.nix` — minimal; per-worktree env is set at spawn, not baked (ADR-008
  D-B5). A `dispatch-worker` bwrap profile only if O7 lands.
- Backlog: IMP-004 (the spike — this slice exercises it), IMP-041 (O5 answers it).

## Sequencing

Order matters — **decisions govern, so they land first**; the design phase must
*produce the ADR/spec drafts as deliverables*, not assume them (else the slice
under-delivers to prose+code):

1. **G1 + G3** — ADR-008 revise/accept (the IMP-004 gate) and ADR-011 spawn-seam.
2. **G2** — ADR-006 amendments (D5/D9 ladder, D2a marker), once G3 fixes the spawn
   backend the marker mechanism rides.
3. **G4** — SPEC-012 rewrite (downstream of the ADR decisions).
4. **Code** — O2/O3 (fork + marker guard), O4/O5 (import/gc verbs), O6 (per-wt env
   provisioning + build isolation), O7 (bwrap spike, codex/pi-only), then O8 (skill
   prose calls the verbs + the `/dispatch-*` harness-routing split) — last, because
   the prose carriers describe the now-shipped mechanism.

## Risks / Assumptions / Open Questions

- **OQ-1 — Spawn backend — RESOLVED (Charge XIII `/consult`).** Not a uniform
  keystone: the agnostic keystone is orchestrator-owned fork + disk marker;
  subprocess (`codex exec`, pi self-subagent) is a **codex/pi enhancement layer**
  (env-arm, per-wt env, bwrap). `claude -p` is rejected as a required backend
  (API-billed + harness-specific); claude uses the `Agent` tool with the marker as
  primary identity (first-class, not degraded) and confesses the env/bwrap
  enhancements as unreachable (residual D2b). base-pin is delivered by `fork` for
  all harnesses. A future env-bearing claude backend (channels) is IDE-004. Recorded
  as the per-harness **capability profile + altitude table** in ADR-011 (G3).
- **OQ-2 — bwrap nested-userns feasibility (O7).** Beyond cost: nested bwrap needs
  unprivileged userns creation inside the jail, which the outer bwrap may
  seccomp-block. Empirical, not analysable up front — probe with
  `bwrap --unshare-user --ro-bind / / true` at spike time. ADR-008 D-B3 frames the
  back-out either way.
- **OQ-3 — Build-isolation disk cost (O6).** One full target dir per concurrent
  worktree; cold builds in the jail may be minutes (debug ≈ 10× release timings,
  `[[mem.pattern.testing.debug-vs-release-scale-timing]]`). **Softened:** in-jail
  `~/.cargo` persists across launches, so `wt/<branch>` targets stay warm — cold
  cost is per-branch, not per-session. Disk is the residual cost, reaped by O5 gc;
  a worktree cap or D-B4 (`sccache`) only under disk pressure.
- **R1 — `import` is the load-bearing belt.** The R-5 `.doctrine/`-touch belt is
  called "the real protection" in prose; moving it into a fail-closed verb is the
  point, but a bug here is high-blast-radius — golden + invariant tests that drive
  the write seam (`[[mem.pattern.review.invariant-test-must-drive-the-write-seam]]`),
  not a pure helper.
- **R2 — Behaviour-preservation gate.** The funnel cadence and the existing
  worktree suites are the proof; they must stay green as mechanism migrates.
- **A1 — IMP-004 precondition met.** IMP-003 resolved ⇒ ADR-008 actionable now.
- **A2 — STRUCK (inquisition Charge II).** Originally: "the import verb must encode
  re-anchor-vs-re-dispatch." Adjudicated to **v1 = stationary-head only**: the verb
  refuses `head-moved` and the orchestrator re-dispatches; the moved-shared-main
  3way+disjointness path (research §5.4,
  `[[mem.pattern.dispatch.three-way-import-onto-moved-shared-main]]`,
  `[[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]]`) is a **named
  backlog follow-up** (`--allow-reanchor`), not v1 and not fail-open prose. Design
  D3.

## Summary

Move worktree/dispatch mechanism out of fail-open prose into fail-closed,
golden-testable CLI verbs, with **orchestrator-owned fork + a disk marker as the
harness-agnostic keystone** that inverts the trust topology (fail-open →
fail-closed) on every harness. Subprocess spawn (`codex exec`, pi self-subagent) is
a **codex/pi enhancement layer** adding env-arm, per-worktree env provisioning, and
OS-level confinement; `claude -p` is rejected (API-billed + harness-specific), so
claude runs the agnostic core via the `Agent` tool at marker-only altitude (Charge
XIII). Lands four governance deliverables first (ADR-008 accept, ADR-006 amend, new
ADR-011 spawn-seam contract + per-harness capability profile, SPEC-012 rewrite),
then the verbs/guards/isolation, then the prose carriers (incl. the `/dispatch-*`
harness router).
Activates the parked ADR-008 and discharges (or formally defers) ADR-006 D2b.
Notably #1 is a *return to ADR-006 D9 compliance*, not an amendment — the
worker-self-fork pattern is skill drift. Output of the SL-055 review; sibling,
not successor.

## Follow-Ups

- Skill structural split → SL-055 (thesis #6).
- **Moved-HEAD import re-anchor (`--allow-reanchor`) → IMP-043** (A2 struck; v1 is
  stationary-head only — inquisition Charge II).
- Remote-worker `format-patch`/`am` cadence → backlog if O-scope confirms need.
- `/dispatch` routing slot → deferred (research §9).
- **Channels spawn backend for claude (env-seamed, no `-p`) → IDE-004** (Charge
  XIII; optimisation tier, experimental — `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`).
- `branch-point-check` naming (HEAD-stationarity, not merge-base — research §9
  C-V) → fold into O4/O8 rename or backlog.
