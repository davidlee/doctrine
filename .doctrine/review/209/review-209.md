# Review RV-209 — reconciliation of SL-183

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject / surface reviewed.** SL-183 (macOS Seatbelt write-confinement arm),
solo-authored (not dispatched). The reviewed surface is the `edge` working tree
at HEAD `cde6311f` — the authored artifacts (`design.md`, `plan.toml`,
`slice-183.toml`, `notes.md`) plus the code diff `6dcf3e75..HEAD` scoped to
`src/worktree/{jail,pretooluse}.rs`. No candidate interaction branch (not a
`/dispatch` slice), so R2 evidence-ref rules do not apply.

**What this audit probes (lines of attack).**

1. **Containment is real, not asserted.** The design's whole claim is a macOS
   write floor at parity with the bwrap arm. Does the shipped consumer actually
   engage Seatbelt end-to-end (resolve → wrap → *materialize the .sb* →
   confine), and is every external write vector denied while the worktree stays
   writable? Hold it to EX-2's live-in-situ bar, not a unit stub.

2. **Fail-closed everywhere (POL-002 / F-B4).** Every deny branch (a–f) and
   every materialization failure must land on `deny worktree-subagent Bash` —
   never an unwrapped pass-through, never an allow-and-wrap over an absent floor.
   The security outcome is invariant even where the *typed reason* is shared.

3. **Parity by reuse (EX-1 / VT-1 / ADR-001).** SL-182's reused leaf functions
   (`validate_policy`, `decide_write`, `pathcheck`, `opaque_wrap`,
   `select_jailer`) must be behaviour-preserved UNCHANGED — the existing suites
   are the proof. The pure/impure split must be honest: no clock/rng/git/disk in
   the leaf.

4. **Conformance algebra.** Path-conformance says where to look. PHASE-01's
   absent source-delta row (registry `incomplete`) is a known anomaly — confirm
   it is the code-free-probe boundary case, not dropped work.

5. **Canon truth-telling.** Where does the shipped implementation diverge from
   what `design.md` / the plan literally say (design §5.1 illustrative profile
   line; §5.5 a–f branch enumeration vs the impl's 5 typed reasons; T3a/T3b
   consumer-wiring increments vs PHASE-03's EX-3 wording)? Each divergence is a
   finding routed to per-slice direct edit or a REV.

**Invariants pinned.** INV-M1 (nested Seatbelt composes / degrades closed);
INV-M2 (realpath-canonicalized `-D` params, subpath matches resolved path);
POL-002 (no host-convention fallback, fail-closed); ADR-001 (leaf ← engine ←
command, no cycles); STD-001 (named constants, no magic strings); the
behaviour-preservation gate (SL-182 suites green unchanged).

**Where bodies are likely buried.** The consumer-wiring seam
(`pretooluse::probe_backend` cfg-split + `materialize_seatbelt_profile`) — the
two increments T3a/T3b that parity-by-reuse never covered (bwrap is inline argv;
Seatbelt needs a disk `.sb`). And the EX-2 provisioning-trigger caveat: the live
leg proved the *consumer*, but the auto-provision trigger on the non-dispatch
(Agent-tool Passthrough) spawn path is still only unit-tested.

## Synthesis

**Closure story.** SL-183 delivers a *real* macOS write-confinement floor for
claude `isolation:worktree` subagents — the cross-platform completion of the
per-arming policy floor SL-182 laid on Linux/bwrap. The arm forks only the
argv/profile builder behind SL-182's `Backend`/`select_jailer` seam; every
shared leaf function (`validate_policy`, `decide_write`, `pathcheck`,
`opaque_wrap`, `resolve_target`) is reused UNCHANGED, and the existing suites
prove it (F-7: jail 59/0, pretooluse 21/0 this audit; the reused-fn signatures
are untouched in the whole-slice diff). ADR-001 layering held — the leaf stayed
pure, impurity confined to `RealEnv` and the command tier.

The central claim — containment is real, not asserted — is discharged with
**live** evidence (F-5). Both decision legs ran through the shipped `worktree
pretooluse` consumer on a nested macOS subagent: the deny leg fail-closed to
`seatbelt-policy-missing` (F-B4) before the script ran, and the
allow-and-confine leg materialized the 540B `.sb` floor and blocked every
external / `/tmp`-alias / `$HOME` / symlink-deref / child-process write while
keeping the worktree writable. Containment was read from OUTSIDE via canary
checksums (6/6 intact), so subagent cooperation was never load-bearing. The
degrade contract (EX-3) is mutation-verified: forcing the resolver fail-open
turns the test red.

**Two load-bearing discoveries en route (F-6).** Parity-by-reuse did not cover
the whole macOS obligation. (T3a) the shipped consumer's `probe_backend` never
routed macOS→Seatbelt — the jailer was dead code from the hook entry until the
cfg-split wiring landed; (T3b) nothing in prod ever wrote the `.sb` body —
`seatbelt_profile()` had zero prod call sites, so every wrapped Bash would have
aborted before Seatbelt engaged. Both are because bwrap confines via inline argv
flags (no external file) while Seatbelt needs a disk `-f <profile>`. Both fixes
were consult-approved scope increments in PHASE-04, are code-complete and
tested, and are the reason EX-2 could run live at all.

**Standing risks / tradeoffs consciously accepted.**
- **EX-2 provisioning-trigger residual (F-5).** The live leg used the Agent-tool
  Passthrough spawn, so the per-wt policy was provisioned manually; the
  auto-provision trigger on that non-dispatch path stays unit-test-only. This is
  a spawn-path seam, not a containment gap — the consumer is proven live
  end-to-end. Captured as follow-up work.
- **EX-4 cordage perf flake (F-4, tolerated).** The full-workspace gate is
  objectively red on one unrelated perf test (`many_small_cycles_evict_in_
  linear_time`, 113.5s vs budget) — out of SL-183's diff (empty cordage diff),
  timing not logic (correctness sibling passes). SL-183's own suites are green.
  Not gating closure; captured as a backlog risk for the cordage owner.
- **PHASE-01 conformance boundary (F-1, aligned).** The registry is `incomplete`
  because PHASE-01 (a code-free confirmation probe) has no source-delta row — its
  `code_start` orphaned by parallel-thread history restructuring. Evidence-
  conformance (results.md), not delta-conformance, governs it. Consulted and
  accepted; masks nothing.
- **Seatbelt vanish risk (design ASM, unchanged).** Deprecated since ~10.10,
  SBPL undocumented; mitigated by Anthropic's own sandbox-runtime depending on
  it. Low, not zero — a standing assumption, not a finding.

Three findings (F-2, F-3, F-6) are `verified` observations whose remediation is
prose truth-telling routed to /reconcile as per-slice direct edits (not REVs).
Nothing gates closure; no blocker remains.

## Reconciliation Brief

Handoff to `/reconcile`. All remediation here is **per-slice direct edit**
(design.md / plan prose) — SL-183 introduces no governance/spec change, so there
is **no REV**. Each entry cites its finding.

### Per-slice (direct edit)

- **design.md §5.1 (F-2)** — the illustrative xcrun_db profile line is a bare
  regex that leaks (`/private/tmp/xcrun_db-*`). Replace with the shipped, proven
  form: `(allow file-write* (require-all (subpath (param "DUTMP")) (regex
  #"/xcrun_db[^/]*$")))`. Decision unchanged (§5.5 EDGE F-E prose already
  requires DUTMP scoping) — this is an illustrative-line correction only.
- **design.md §5.5 / plan PHASE-03 EX-2 (F-3)** — add a note that the six
  fail-closed *conditions* (a–f) are realised through five typed `ResolveDeny`
  reasons: branch c (nested-repo/submodule basename never provisioned) and
  branch e (policy absent) deduplicate to `PolicyMissing` by shared mechanism
  (`read_policy→Ok(None)`) and invariant outcome (fail-closed Deny). "6
  conditions, 5 typed reasons" — not a scope cut.
- **plan PHASE-03/PHASE-04 or design §5.4 (F-6)** — record that the consumer-tier
  macOS routing (`probe_backend` cfg-split) and `.sb` profile materialization
  completed in PHASE-04 as the consult-approved increments T3a/T3b; PHASE-03's
  EX-3 was true at the `select_jailer` leaf level but the consumer-tier reach
  (the materialization obligation parity-by-reuse didn't cover) landed in
  PHASE-04.
- **notes.md PHASE-01 / design or plan (F-1)** — a one-line note that PHASE-01
  carries no git-range source-delta *by design* (code-free confirmation probe);
  conformance for it rests on evidence (results.md), not a delta row. Makes the
  registry `incomplete` read as intentional.
- **notes.md PHASE-02 (F-8, optional)** — minor oid staleness: the ISS-204 fix
  landed on the current tree via a different oid than the recorded `8abcaae0`
  (which is not an ancestor of HEAD; `66821afe` is). The fix is present and
  correct — a trail-accuracy note only, no action required for closure.

### Governance/spec (REV)

- None. SL-183 makes no governance or spec change.
