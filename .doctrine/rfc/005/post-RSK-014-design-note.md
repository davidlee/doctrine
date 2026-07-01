# Post-RSK-014 Design Note: Residual effect of the SL-182 / SL-183 direction on the RFC-005 dispatch hazard survey

<!-- Authored 2026-07-01 after a walkthrough of RFC-005's hazard classes against the
     SL-182 (bwrap confinement) and SL-183 (macOS Seatbelt) designs — both locked,
     SL-182 implementing, SL-183 planned with blocked-on-SL-182 dependency. This note
     records what changes, what stays, and what new disclosures the confinement
     direction forces on the RFC's model. -->

**Status:** design note (companion to RFC-005, not a revision of it). Sections I–III walk
each hazard class; §IV covers cascade effects the RFC did not anticipate; §V presents the
full design-space tradeoff; §VI is the summary.

---

## I. Hazard classes the confinement direction does NOT materially affect

These are structurally unchanged by SL-182/183 and their shared confinement approach.

### H0 — Worktree API doc-trust (DISCHARGED by SL-152)

SL-152 replaced native worktree creation with doctrine-owned `create-fork`. SL-182
operates strictly downstream (at tool-use time, not creation time). No interaction.

### H1 — Wrong-base isolation race (ELIMINATED by SL-152)

The race cannot occur because doctrine creates the worktree. Confinement constrains
what the worker *does* once on the correct base, not whether it lands there.

### H4 — Worker provisioning (gitignored build artifacts)

**Open:** the coord-tree leg survives SL-152 with tactical workarounds (hand-copy of
`web/map/dist/`). The confinement direction does not address this — bwrap/Seatbelt
walls prevent the worker from reaching outside its worktree to fetch missing
artifacts, making provisioning errors fail-closed (artifact missing → test fails →
import blocked).

**Interaction:** the `extra_rw` policy knob can mitigate specific cases (bind a
known artifact path rw into the jail), but this is per-case, not general. IDE-017
(orchestrator-addressable provisioning) is *more* consequential under confinement:
a jailed worker has no fallback to the host filesystem.

### H5 — Configuration and ergonomics (open lower-severity items)

SL-182 expands the ergonomic surface with policy files, per-arming granularity, the
`pretooluse` subcommand, and the `|| exit 2` vanish-guard semantics. None of the
pre-existing H5 items (IMP-126 trunk_preference, IMP-101 deliver_to, etc.) interact
directly — but the new surface carries its own future ergonomic debt (documentation,
runbook entries, operator mental model of the jail policy config).

---

## II. Hazard classes materially changed by confinement

### H2 — Integration integrity (phantom revert, R1 re-opened)

**Pre-confinement:** SL-157 stripped the None-leg speculative resync, banking on
`main`-never-checked-out. R1 re-opened when agents violated that behavioural invariant
(SL-164 drive). The mechanism fix is SL-166 g1 (refuse trunk-mutating verbs on a trunk
checkout) — SL-166 is `done`.

**Confinement effect:** SL-182/183 and SL-166 g1 share a philosophical root (replace
behavioural etiquette with mechanism-level enforcement) but operate at different
protocol layers:

| Layer | SL-166 g1 | SL-182/183 |
|---|---|---|
| Enforces | Verb-entry (integrate-on-trunk-checkout → refuse) | Syscall (Bash write outside worktree → `EROFS`) |
| Closes | The R1 CAS window (orchestrator chose wrong leg) | Worker writes outside worktree (unrelated to integrate) |

**Net residual:** R1 is structurally closed by SL-166 g1 (done). Confinement is
defence-in-depth — it prevents the *worker* from self-inflicting a phantom, but the
integrate CAS window itself is a git-topology issue, not a write-containment issue.
**No untested mid-CAS coverage** remains the open item against R1's closure claim.

### H3 — Arm asymmetry (RESOLVED by SL-152 + SL-154 for creation/recording)

**Pre-confinement:** creation seam converged (SL-152), recording converged (SL-154).
Three asymmetries remained: confinement posture, commit model, and macOS support.

**Confinement effect:** SL-182 + SL-183 close all three to essential symmetry:

| Property | Pi subprocess | Claude (Linux) | Claude (macOS) |
|---|---|---|---|
| Write wall | `pi-spawn-confined.sh` bwrap | `pretooluse` bwrap (SL-182) | `pretooluse` Seatbelt (SL-183) |
| Funnel import | Working-tree diff | Captured patch via `SubagentStop` (or `B..S` commit if `.git` loosened) | Same default |
| macOS | Available | N/A | Available (SL-183) |
| Cross-subnat write surface | None | None (default) | `xcrun_db` narrow re-allow (SL-183 OQ-mac4) |

**Persistent asymmetry (documented, not closed):**
- The `xcrun_db` cache-file re-allow on macOS (one narrow cross-subagent write
  surface outside the floor, no Linux analog — SL-183 §5.5 F-E).
- The `launchd` IPC residual on macOS (`launchctl submit` denied by Seatbelt default,
  but pure-IPC mach-service path is a non-goal — sibling of the nix-daemon/postgres
  reachable-peer residual on Linux).

**Implication for RFC-005's H3 class:** the RFC's original frame was
"claude arm is less reliable than subprocess arm." The confinement direction flips
this: the claude arm now matches subprocess arm reliability on Linux, *exceeds* it
on macOS (which the subprocess arm already had via bwrap/nix), and the only
remaining asymmetries are narrow, documented special cases. The H3 class is
effectively closed as a correctness concern — what remains is the config-surface
asymmetry (per-arming policy granularity, §IV-3 below).

### H6 — Silent destructive integration / corpus loss (SL-166, done)

**Confinement effect: narrows g3's threat model.** SL-166's g3 (corpus-shrink refusal
at ref-advance) must guard against the orchestrator-side import making a shrinking
commit. With ro-`.git` (the default confinement posture), the jailed worker *cannot*
perform a corpus-shrinking commit — it has no write access to the shared object store.
So g3's defence-in-depth is now asymmetric:
- **Under ro-`.git`:** g3 protects against orchestrator-side error only (import diff
  that happens to shrink corpus). Worker-side corpus removal is structurally impossible.
- **Under permissive `.git`:** g3 protects against both orchestrator and worker, as
  originally designed.

The confinement direction does not close g3's gap — it just shifts the operator's
risk calculus depending on the `.git` policy chosen.

---

## III. The false-green family (PIR intake: SL-170, IMP-194–201)

### The correction my earlier analysis needed

I claimed confinement "makes every PIR item more urgent on the claude arm." This is
true **only under the ro-`.git` default**, where the worker cannot self-commit and the
orchestrator inherits the full verify burden. Under a permissive `.git` policy (rw
`.git/objects` + worktree-specific refs), the worker self-commits and the existing
`B..S` single-commit delta-check survives unchanged — the urgency of SL-170 becomes
dispatch-configuration-dependent, not structurally imposed.

The actual relationship:

| `.git` posture | Worker self-commit? | Funnel import | SL-170 urgency |
|---|---|---|---|
| ro (default) | No | Captured patch via `SubagentStop` | **Required** — orchestrator must independently verify |
| Permissive (selective rw) | Yes (to shared store) | `B..S` commit delta (existing) | **Optional** — worker's own verify still counts |
| Discrete clone (Path C / IDE-024) | Yes (to private store) | Cherry-pick full commits from worker | **Optional** — isomorphic environment, worker's verify trusted |

In all three postures, the **prompt discipline** items (IMP-197) and **golden
hermeticity** items (IMP-195/196/200) are unchanged — confinement contains the
symptoms of bad prompts but not their cause.

### The honest new fragility (H7 by implication)

Under the ro-`.git` default only, the `SubagentStop` capture hook is the sole seam
through which the worker's diff reaches the orchestrator. If the capture fails
(crash, timeout, harness change that drops `agent_id`, worktree correlator drift),
the worker's entire output is lost — there is no commit to fall back on.

This is a **single-point-of-failure in the execute→import seam** that RFC-005 did not
survey. It is documented in SL-182 as a defined-abort (→ Path C / IDE-024), but as
long as Path L is the production default, the capture hook is a new hazard class:
- **Likelihood:** low (sentinel-guarded, probe-proven on 2.1.x)
- **Impact:** high (full phase output lost, orchestrator must re-dispatch)
- **Recovery:** none defined in-path (orchestrator can re-dispatch; no automated
  rollback to the pre-capture state)

Under a permissive `.git` posture this fragility does not apply — a committed fallback
exists. Under Path C / IDE-024 the fragility does not exist (durable commits in a
private store).

---

## IV. Cascade effects — new disclosures confinement forces on the RFC-005 model

These are structural consequences the hazard survey did not anticipate.

### 1. The design space is three-dimensional, not one-dimensional

RFC-005 analysed hazards along a single axis (correctness). The confinement direction
replaces that with a three-dimensional tradeoff space:

```
                      strictness
                        ▲
                        │
              Path L     │     Path L
              (ro-.git)  │     (selective .git rw)
                        │
                        ├──────────────────► efficiency
                        │
              Path C / IDE-024
              (discrete clone,
               private .git,
               cherry-pick)
```

An operator (or policy) chooses:
- **Topology** (linked worktree vs discrete clone)
- **`.git` writability** (ro default vs selective rw per dispatch)
- **Funnel import** (captured patch vs `B..S` commit vs cherry-pick commit)

Each combination has different hazard properties. The original RFC assumed a single
funnel topology; the confinement direction makes topology a dispatch-configurable
parameter.

### 2. macOS is no longer a platform gap (SL-183 changes the timeline)

SL-183 is not merely "ready" — it has a locked design, a completed probe (RSK-014 H2
pass 1 + pass 2, both supporting), a plan with four phases, and an inquisition
discharged (RV-203, 9 findings). The only empirical remainder is the DUTMP/xcrun_db
profile shape (OQ-mac4), deferred to PHASE-01.

Crucially, SL-183 constrains SL-182's implementation (D-mac2): SL-182 must expose a
`Jailer` trait / `select_jailer` fork point, capability-as-data `Backend` dispatcher,
and git-topology-based worktree derivation (not a path template, because macOS Agent
worktrees land at `<repo>/.claude/worktrees/agent-<id>`). If SL-182's implementation
diverges, SL-183 absorbs a refactor. This is a **schedule coupling risk** — SL-183's
start is blocked on SL-182's merge, but SL-183 also needs the seam shaped in a
specific way.

The old RFC-005 frame ("macOS is an unconfined platform") becomes: "macOS is
temporarily gated on SL-182's `jail.rs` landing, with a proven replacement mechanism
ready to slot in."

### 3. Per-arming policy granularity is a new dimension of dispatch capability

SL-182's D2 resolution: serial arming ⇒ per-worker policy; parallel fan-out ⇒ one
shared profile per batch. This means the claude arm cannot discriminate between
concurrent siblings' jail profiles on parallel dispatch. On the pi/subprocess arm,
where the orchestrator runs `worktree fork --worker` and knows each name before spawn,
per-worker distinct policies *are* natively achievable.

This asymmetry is durable (not closable by configuration — it's inherent to the
harness's spawn model). RFC-012 (parallel fan-out safety) should account for it: the
confinement posture guarantees of a parallel dispatch on the claude arm are "same
floor for all workers," not "individually tailored floors."

### 4. The orchestrator-jail gap (OQ-5) is accepted, not closed

SL-182's pass-through trusts `agent_id` absence — an unauthenticated tell. A phantom
no-`agent_id` spawn inherits the orchestrator's unconfined access. This is *accepted*
(R7 acquitted): `agent_id` is harness-stamped present-iff-subagent (proven), the
shared `session_id` is the only tell, and a harness-integrity failure that forges
absence would equally break worker/orchestrator discrimination everywhere. OQ-5
("jail the orchestrator too") is deferred — not a hole, but a named residual for
when the platform's threat model tightens.

### 5. Path C / IDE-024 is not a "future fix" — it's the second topology in a settled design space

The discrete-clone topology (standalone clone, private `.git`, worker self-commits,
orchestrator cherry-picks) was present in SL-182's design from the start as the
efficiency recovery path (R8 / IDE-024). It is not an afterthought or a "someone
should fix this later." The design space has two settled topologies (Path L and
Path C), with Path L sequenced first because it's lighter-weight (no clone cost) and
ride the existing linked-worktree seam. Path C is deferred on observed cost, not on
correctness.

This reframes the efficiency critique completely: the ro-`.git` default's throughput
cost is not a permanent liability of confinement — it's an operator choice between
zero-clone overhead (Path L, slower verify) and clone overhead per dispatch (Path C,
self-commit verify preserved). Both are confined.

---

## V. The full tradeoff space

| Property | Path L default (SL-182) | Path L loosened | Path C / IDE-024 |
|---|---|---|---|
| **Topology** | Linked worktree | Linked worktree | Discrete clone |
| **Clone cost** | None (link) | None (link) | Full clone per dispatch |
| **`.git` access** | ro (floor) | Selective rw (policied) | rw (private store, no external effect) |
| **Worker self-commit** | No | Yes (to shared store) | Yes (to private store) |
| **Funnel import** | Captured patch via `SubagentStop` | `B..S` single-commit delta | Orchestrator cherry-picks commits |
| **Efficiency** | Orchestrator re-runs suite | Worker's self-verify counts | Worker's self-verify counts |
| **Capture fragility** | Single-point-of-failure (H7) | Commit fallback exists | Durable commits; no fragility |
| **Imp impersonation risk** | None (ro `.git` prevents it) | Cooperative only (SL-181 guard) | None (private `.git`, no shared store) |
| **Fidelity** | Working tree only | Full git history preserved | Full git history preserved |
| **Status** | Landing (SL-182 PHASE-02+ underway) | Config knob, designed | Deferred; IDE-024 |

---

## VI. Summary

### What changed

- **H2** — confinement is defence-in-depth for the integrate issue, not a fix. SL-166
  g1 is the actual closure.
- **H3** — confinement closes the last three asymmetries (Linux write wall, macOS
  support, commit model convergence). The asymmetry that remains is per-arming policy
  granularity (RFC-012 concern), not correctness.
- **H6** — confinement narrows g3's threat model to orchestrator-side only (under the
  ro-`.git` default).
- **False-green family** — SL-170's urgency is configuration-dependent (required under
  ro-`.git`, optional under permissive `.git`, eliminated under Path C).
- **H7 (new)** — the `SubagentStop` capture seam is a single-point-of-failure under
  the ro-`.git` default that the original survey did not name. Documented, defined-abort
  to Path C, but not closed.

### What stayed the same

- H0, H1 — unaffected.
- H4 — unaffected (confinement does not provision artifacts; IDE-017 remains open and
  more consequential).
- H5 — net expansion of the ergonomic surface.
- PIR prompt/golden items (IMP-195/196/197/198/200) — unaffected by confinement
  posture; they address the quality of what the worker produces inside the wall.

### What the RFC-005 model should account for

1. **Topology is a dispatch-configurable parameter, not a framework constant.**
   Linked worktree vs discrete clone, with different hazard profiles, is an operator
   choice — not a single funnel shape to harden.

2. **`.git` writability is a per-dispatch policy decision, not an architectural invariant.**
   The strictest default is ro-`.git`; the framework allows selective loosening. Each
   choice shifts the hazard surface (capture fragility vs commit-based recovery vs
   impersonation risk).

3. **OS is a per-dispatch parameter.** The confinement floor is available on Linux
   (SL-182 now) and macOS (SL-183, gated on SL-182). The macOS arm's temporary denial
   is bounded; the durable asymmetry is a narrow `xcrun_db` write surface.

4. **Parallel fan-out on the claude arm has a capability ceiling.** Per-sibling
   distinct jail profiles are unavailable — all workers in a parallel batch share one
   profile. RFC-012 should name this.

5. **The capture fragility (H7) wants a defined recovery path.** Under
   ro-`.git`-default operation, a `SubagentStop` capture failure loses the worker's
   entire output. The defined abort is "re-dispatch under Path C" — but that path
   does not exist yet (IDE-024 deferred). Until IDE-024 lands, the fragility is a
   standing operational risk with no automated recovery.

---

*References: RFC-005 (`.doctrine/rfc/005/rfc-005.md`), SL-182 design + plan + notes,
SL-183 design + plan + seatbelt-seam-brief + plan, RSK-014 + probe-h1 + probe-h2-seatbelt
results, ADR-006/008/012.*
