# Review RV-200 — design of SL-182

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Inquisition of `SL-182/design.md` at design-lock, feeding `/plan`. Posture:
`--raiser inquisitor`; aspect: design. Doctrine held to: ADR-008 (the claude-arm
confinement gap this discharges), ADR-006 (D2b raw-tree confinement, D-sole-writer),
POL-002 (platform independence / fail-closed), STD-001 (no magic strings), and the
RSK-014 probe-h1 EMPIRICAL ground truth (`.doctrine/backlog/risk/014/probe-h1/`).
Hook claims tried against the local `docs/claude` cache (authoritative over web).

Lines of interrogation (the four soft targets the author flagged + an independent
sweep): (1) A7 keying — is the pre-spawn-declare → create-fork-provision handshake
race-free for the ARMED worker path? (2) V-plugin — is locking the unproven plugin
registration home, while forbidding the proven fallback, acceptable at lock?
(3) OQ-2 — does the harness surface the worktree diff once ro-`.git` blocks the
worker's self-commit? (4) R7 — orchestrator pass-through god-mode: accepted residual
or must-land? Independent adversarial pass run via codex (GPT-5.5, read-only);
its findings corroborated and extended the charge sheet.

## Synthesis

**Judgement: HERESY FOUND. The design is NOT clean — three blockers gate the
close.** The probe's apparatus is sound and the slice's intent is righteous, but
the graduation from throwaway scripts to installed machinery smuggled in three
load-bearing assumptions the authoritative docs actively contradict, plus a
self-contradicting decision register. The strict default floor still ships safe;
it is the *embellishments* — custom policy, plugin registration, funnel
convergence — that confessed under cross-examination.

**The three that gate `/plan` (blockers — reconcile before the slice advances):**

1. **F-1 — per-worker custom policy is unbuildable** through the single-slot
   arming rendezvous (`arm-spawn` writes one shared `base`; `dispatch-agent`
   sanctions N parallel spawns off one arming). The promised per-worker tuning is
   a fiction the topology cannot honour, and a parallel batch would *leak* one
   worker's loosening onto another — a confinement weakening masquerading as the
   "absence can only tighten" floor. Penance: cut custom policy to the strict
   default floor (recommended), or serial-scope it with a named critical section.

2. **F-2 — the installer fails OPEN.** Bare-PATH plugin exec + the harness rule
   that only `exit 2` blocks (hooks.md:629-643) means a stale/missing `doctrine`
   binary lets the tool call proceed UNCONFINED — RSK-014 reopened by the very
   hook meant to close it, and a direct §5.1/D1-vs-§5.4 contradiction. Penance:
   fail closed — embed a resolved absolute exec (resolve_exec at install) or a
   shim that `exit 2`s on command-not-found.

3. **F-3 — the funnel rests on a teardown the design cannot see.**
   `WorktreeRemove` auto-`git worktree remove`s the worktree when the subagent
   finishes, with NO decision control (hooks.md:2442/680/814) — the uncommitted
   diff is destroyed, and "identical on both arms" is false (pi's orchestrator
   owns lifecycle; claude's harness does not). Penance: name a contingency —
   snapshot `git diff` in a WorktreeRemove/SubagentStop hook before removal
   (recommended), or Path C / IDE-024, or defer ro-`.git`.

**The corroborating majors (reconcile in the same pass):** F-4 stale `agent_id`
keying in D2 + the authored scope contradicting the corrected §5.3; F-5 the
V-plugin fallback forbidden rather than planned (make D-reg conditional, fallback
same-phase); F-6 the Edit/Write wall matching an undocumented `NotebookEdit`/
`notebook_path` surface (drop it or pin its schema first).

**The minors/nits (sweep-grade):** F-7 `network=true` default contradicts the §4
"strictest floor" wording; F-8 the policy file's false "ancestor" rationale (ro-ness
is from `--ro-bind / /`); F-10 §10 understates the doc coverage of the wire fields.

**Tolerated / acquitted:** F-9 — R7's orchestrator-pass-through residual is
ACQUITTED. agent_id is harness-stamped present-iff-subagent (probe Exp 1/3); a
confined worker cannot forge its absence; OQ-5 deferral is sound. Soft-target 4
answered: accepted, not must-land.

**Standing risks after reconcile:** the slice's correctness now hinges on TWO
unverified harness behaviours (V-plugin registration firing; WorktreeRemove
snapshot timing) — both must become first-execute verification gates WITH named
fallbacks, not bare "verify later." The strict-default-floor reframing (F-1) is the
safe spine; everything else is opt-in tuning that should land only on proven ground.

**Sentence:** the design returns to its author for reconciliation. F-1 and F-3
carry remediation OPTIONS, not a single forced fix — those are a User/`/design`
decision (scope cut vs serial-scope; which OQ-2 contingency). F-2, F-4, F-6 have a
clear corrective direction. The nine answered charges are withheld from `verify`
deliberately: the three blockers hold the close-gate until canon tells the truth.

> Let the false guarantees be put to the fire; the floor that holds shall remain.

### Reconcile (2026-07-01) — all 10 charges terminal, design re-locked

The penance was done in the same sitting. All ten findings integrated into
`design.md` (+ `slice-182.md` scope) and verified terminal; RV-200 `done ·
await=none`. Two User decisions settled the option-bearing blockers:

- **F-1 → serial-scope with shared parallel profile.** Profile granularity is
  per-arming: serial ⇒ per-worker; parallel fan-out ⇒ one profile shared by the
  batch. The User chose "parallel workers share one profile" over the stricter
  "parallel workers get only the baseline floor" — the single arming slot carries a
  single intent, so there is no differing-sibling to leak; rationale recorded durably
  in design §5.3 for later challenge.
- **F-3 → snapshot-before-remove.** A `WorktreeRemove`/`SubagentStop` capture hook
  lifts the worker diff to an outside-the-worktree patch before the harness's
  auto-removal; abort to Path C / IDE-024 if the capture cannot observe the tree
  intact. Reframed OQ-2 as a lock-time risk with a defined abort.

F-2 (fail-closed exec) and F-6 (drop NotebookEdit) took their clear corrective
direction; the minors/nits swept. Design status flipped **draft → LOCKED**. The
slice is clear to advance to `/plan`. *Doctrina manet.*
