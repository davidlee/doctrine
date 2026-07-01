# Review RV-208 — reconciliation of SL-182

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed:** SL-182 authored solo on `edge` (not `/dispatch`-driven) — no
candidate branch; audit runs against the primary tree at `edge`=`9441ca9f`.

**Mode:** conformance (post-implementation). PHASE-01..05 all `completed`; the slice
was flipped to `audit` after VH-1 acceptance.

**Lines of attack:**
1. **VH-1 / §5.5 Fork-path ASM** — the slice's last-standing open assumption. Was
   the Fork-path worktree *persistence* + *branch-case footer* (`worktreePath`)
   proven live, or does canon still read them as an unproven ASM pinned at VH-1?
   Held to: the reap must be `&&`-gated on `import` exit 0 (F-3); no `WorktreeRemove`
   hook ships (INV-6/AF-3).
2. **Mechanical conformance** (`slice conformance 182`) — do the design-target
   selectors match what git touched? Where are undeclared/undelivered cells, and are
   they scope creep, selector staleness, or cross-slice edge-interleave noise?
3. **Gate + suites** — `doctrine check gate` green; `classify_import` pure core
   unmoved (VT-2); `run_import_fork` byte-frozen (EX-4); worktree/dispatch suites green.
4. **Registry integrity** — did the main-tree `completed` flip clobber any phase
   boundary to a degenerate range (`mem.pattern.doctrine.phase-complete-clobbers-boundary`)?

**Evidence gathered this session:**
- Live VH-1 Fork-path E2E (worker `agent-a29cf7c4dba0cac86`, B=`9441ca9f`): footer
  `worktreePath` present on the armed branch case; tree persisted post-return
  (HEAD==B, delta intact); 4/4 write-escapes denied (bash ro-fs bwrap jail ×2,
  Write tool-layer, self-commit ro-`.git`); canaries sha256-intact; `import
  --from-worktree` exit 0 (tracked+untracked both `--index`-staged); reap gated on
  committed delta. Throwaway coord/branch/canaries torn down.
- `doctrine check gate` → exit 0 (clippy 0-warn + fmt + full suite).
- `slice conformance 182` → 13 conformant (all core source targets), undeclared 95
  (cross-slice + selector drift), undelivered 2 (`.claude/` derived selectors).
- `boundaries.toml` → 5 non-degenerate phase rows; PHASE-05 `fa02a50e`→`9441ca9f`
  intact (no clobber from the flip).

## Synthesis

**Closure story.** SL-182 set out to close ADR-008's conceded gap — no real
write-isolation for a claude `Agent` worker — with two PreToolUse walls (Bash→bwrap
opaque rewrite; Edit|Write ancestor-deny) plus the symmetric live-import funnel that
converges the jailed worker's ro-`.git` working-tree delta. All five phases landed
green; the sole open item at audit entry was VH-1, the first *live* Fork-path
exercise of the persistence + branch-case-footer assumptions that PHASE-05's pivot
(SubagentStop-capture retired → live-import) rested on.

VH-1 ran clean this session and closes the last risk. The observed run is exactly
the design's §5.4 four-step funnel: footer `worktreePath` (present on the armed
branch case — the one datum the §5.5 ASM flagged as Passthrough-only) → tree persists
→ `verify-worker` → `import --from-worktree` (exit 0, tracked+untracked `--index`) →
reap gated on the committed delta. The confinement itself held: bash writes above the
worktree and into the host repo hit a read-only filesystem (the bwrap jail), the
Write tool was blocked outside the worktree, and self-commit failed on ro-`.git`.
Canaries were byte-intact. OQ-1/OQ-2 and the §5.5 ASM are now empirically settled.

**Standing risks (consciously accepted).**
- *Edit|Write pretooluse deny not independently isolated.* VH-1's B3 (Write outside
  worktree) was denied by the **harness** isolation guard, which pre-empted doctrine's
  own Edit|Write hook. Doctrine's deny is config-verified installed and INV-4
  unit-tested, and the Bash wall (the load-bearing one) *was* exercised live (B1/B2
  ro-fs). Defense-in-depth means the vector is closed either way; but the live proof
  of doctrine's *specific* Edit|Write hook firing for a subagent remains inferential.
  Tolerable — not raised as a blocker.
- *Conformance registry cross-slice noise (F-3).* Edge-interleave with SL-184, not
  scope creep; the real source targets are all conformant. Boundary hygiene is a known
  systemic item (IMP-222; the clobber-boundary pattern memory), not this slice's debt.

**Tradeoffs.** VH-1 exercised a *synthetic* worker delta (a README line + one
untracked file), not a real phase — SL-182 had no phase left to land. The regression
capture/diff and `record-boundary` beats were deliberately skipped: a synthetic
throwaway delta makes a regression suite meaningless and a boundary row would pollute
the registry. VH-1's contract is confinement + persistence + import-belt-green + gated
reap, all of which the synthetic delta exercises fully. The import belt (`.doctrine/`
/`.claude/` reject, HEAD==B, tree_clean) is the load-bearing funnel check on the claude
arm and it ran real against the live tree.

## Reconciliation Brief

Both design findings are artifact-truth updates (canon lagging reality), not code or
governance changes. No REV needed — SL-182 owns no ADR/spec edits here.

### Per-slice (direct edit — design.md)
- **F-1** — design.md §5.5 ASM (RV-205 F-4), and the §6 OQ-2 residual + §5.4 step-1
  footer note: reconcile from "assumption grounded in hook-presence, *pinned at
  VH-1* / confirmation-not-sole-support" → **"proven live 2026-07-01"** (Fork-path
  worker `agent-a29cf7c4dba0cac86`: armed-branch footer `worktreePath` present, tree
  persisted, import green, reap gated). The ASM becomes a confirmed fact.
- **F-2** — the dispatch-agent funnel design-target **selector** names the derived
  `.claude/skills/dispatch-agent/SKILL.md`; retarget it to the authored source
  `plugins/doctrine/skills/dispatch-agent/SKILL.md`. Drop (or justify) the
  `.claude/skills/dispatch/SKILL.md` design-target — it is undelivered (the planned
  light-touch was unnecessary).

### Governance/spec (REV)
- None.
