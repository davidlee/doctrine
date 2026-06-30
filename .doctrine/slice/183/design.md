# Design SL-183: macOS Seatbelt write-confinement arm

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

<!-- STATUS: design IN PROGRESS. §§ below carry the RSK-014 H2 probe findings
     (pass 1, orchestrator context — `.doctrine/backlog/risk/014/probe-h2-seatbelt/`).
     The architectural decisions (seam shape D-mac2; SL-182 ordering) are NOT yet
     locked with the user. Pass-2 (M1-sub in-situ subagent leg) is OUTSTANDING and
     gates final lock. Do not treat §5/§7 as settled. -->

## 1. Design Problem

Give claude `isolation:worktree` subagents a **real** OS-level write-containment
floor on **macOS**, at parity with SL-182's Linux/bwrap arm, discharging IMP-045
for the claude arm. Today macOS is a fail-closed stub (`deny "bwrap-unavailable"`,
SL-182 §5.5 / POL-002). Reuse the **same** `Decision`/`Target`/policy/funnel — fork
**only** the argv/profile builder behind a single `Jailer` seam.

## 2. Current State

- SL-182 (the bwrap arm) is **`ready`** — design locked, **not yet implemented**.
  `src/worktree/jail.rs` (or equivalent `pretooluse` module) **does not exist
  yet**; the `Jailer` fork-point SL-183 reuses lives only in SL-182's *design*.
  SL-182 chose **Rust subcommand altitude** (`doctrine worktree pretooluse`,
  matcher-dispatched off stdin `tool_name`), riding `HookSpec` + `plan_hook` +
  `hooks.json`. SL-183 forks `seatbelt_profile()` + `sandbox_exec_argv()` only.
- macOS arm today: fail-closed `deny` on non-bwrap platforms (never pass-through).
- **Hard dependency `needs SL-182`** — implementation blocked until SL-182's
  `jail.rs` lands and exposes the fork point. (Open question: does SL-183's design
  *constrain* that seam, or absorb a later refactor? — §6 OQ-mac3.)

## 3. Forces & Constraints

- **ADR-001** layering (leaf ← engine ← command): the profile/argv builders are
  pure (paths/strings in, `String`/`Vec<OsString>` out); impurity (realpath, exec)
  in the shell. No new pipeline — fork one seam.
- **POL-002** platform independence / fail-closed ethos: any ambiguity ⇒ `deny
  worktree-subagent Bash`, never unwrapped pass-through.
- **STD-001** no magic strings: profile tokens, `-D` param names, the device-sink
  allow-set, bind/deny flag strings → single-sourced named constants.
- **Behaviour-preservation gate:** `resolve_target`, `decide_bash`, `decide_write`,
  `pathcheck`, `opaque_wrap`, `validate_policy` reused UNCHANGED — SL-182's suites
  stay green.

## 4. Guiding Principles

- **Inverse of bwrap, not default-deny.** Seatbelt fences *operations* over an
  unchanged fs: `allow-default-deny-write-except`. NOT SBPL default-deny (the
  footgun this design sidesteps).
- **Probe before Rust.** Seatbelt is deprecated + undocumented; every behaviour is
  empirically pinned (RSK-014 H2) before it enters the design as fact.
- **Realpaths, not string-splicing.** `subpath` matches RESOLVED paths; feed
  realpath'd `-D` params, never interpolate paths into the profile body.

## 5. Proposed Design

### 5.1 System Model

The profile (proven shape, RSK-014 H2 pass 1):
```scheme
(version 1)
(allow default)                                 ; reads open (parity: reads OOS)
(deny file-write*)                              ; the floor
(deny file-write* (subpath (param "PTMP")))     ; F-A: coarse deny FIRST (see 5.5)
; device write sinks — F-B (literals/regex, must stay writable):
(allow file-write* (literal "/dev/null")) … (regex #"^/dev/tty") …
(allow file-write* (subpath (param "WT")))      ; worktree rw — SPECIFIC, LAST
(allow file-write* (subpath (param "TMP")))     ; TMPDIR=<wt>/.tmp (D-mac3)
(allow file-write* (subpath (param "RWn")))     ; per validated extra_rw
; (deny network*)  iff policy.network == false  ; coarse (M3 caveat)
```
Invoked: `sandbox-exec -D WT=<realpath> -D TMP=<realpath> -D PTMP=/private/tmp
-D RWn=… -f <profile> -- bash -c "$(base64 -d <<<$B64)"`. Children inherit.

### 5.2 Interfaces & Contracts

Two new pure functions behind the `Jailer` seam (shell analogs proven in
`probe-h2-seatbelt/seatbelt-jail.sh`):
- `seatbelt_profile(policy) -> String` — emits the profile body, **rules ordered
  deny-coarse-first / allow-specific-last** (F-A). Device-sink allow-set is a
  constant.
- `sandbox_exec_argv(wt, policy) -> Vec<OsString>` — realpaths WT/TMP/extra_rw into
  `-D` params (F-A footgun mitigation), opaque base64 body, sets
  `TMPDIR=<wt>/.tmp`.

Seam shape (trait vs runtime-os branch) — **D-mac2, NOT yet locked** (§6/§7).

### 5.3 Data, State & Ownership

Reuses SL-182's per-arming policy file (`<main>/.doctrine/state/dispatch/jail/
<worktree-name>.toml`, schema `extra_rw` + `network`), provisioned by the
create-fork hook, looked up by `cwd → basename`. **No new state.** `validate_policy`
(reject `/`, root-ancestors, `.git`) is platform-agnostic, shared unchanged.

### 5.4 Lifecycle, Operations & Dynamics

The funnel import/delta-check is identical to SL-182 (the ro-`.git` self-commit
consequence is the same — the worktree's real gitdir is outside wt → write-denied
by the floor). No fork in the funnel; only the argv builder.

### 5.5 Invariants, Assumptions & Edge Cases

Pinned empirically (RSK-014 H2 pass 1, orchestrator context):

- **INV (F-A) — SBPL is LAST-MATCH-WINS.** macOS temp worktrees live UNDER
  `/private/tmp`. The coarse `deny PTMP` MUST be emitted *before* the specific
  WT/TMP/extra_rw allows, or it shadows the worktree itself → floor denies in-wt
  writes. **Load-bearing ordering invariant** for `seatbelt_profile`.
- **INV (F-B) — device sinks stay writable.** `(deny file-write*)` denies
  `/dev/null`, `/dev/std{out,err}`, `/dev/tty*`, `/dev/fd`, `/dev/dtracehelper` →
  breaks tooling (proven: python3). Re-allow them (constant set).
- **EDGE (F-E) — `/var/folders/$USER/T` is a SECOND temp surface.** macOS per-user
  temp (`DARWIN_USER_TEMP_DIR`, `$TMPDIR` default), distinct from `/tmp`; xcrun
  hardcodes an `xcrun_db` cache there. The `TMPDIR=<wt>/.tmp` redirect does NOT
  cover it → denied, noisy (cosmetic for python; breaks cache-dependent tools).
  **DECISION NEEDED (D-mac3 refinement):** also redirect/allow
  `/var/folders/$USER/T`, or accept the breakage class.
- **INV (M2) — canonicalization containment holds.** Realpath'd `-D` params are
  sufficient: absolute, `../`, symlink-deref, **hardlink** (`ln` to outside target
  denied — Seatbelt resolves the link target), `/tmp` alias, shared-`.git`, `$HOME`
  — all denied. No extra canonicalization needed.
- **INV (M1-sub) — PROVEN (pass 2, 2026-07-01).** The floor composes inside a real
  `isolation:worktree` subagent where Claude's own native Seatbelt is active. Under
  the clean **yolo (bypassPermissions)** leg: every external vector denied by the
  nested floor, all canaries intact; floor-bypassed control (same mode) ⇒ all four
  canaries breached. Sole difference = the floor ⇒ Seatbelt is the cause. Degrade
  contract NOT triggered. Evidence: `probe-h2-seatbelt/results.md` (Pass 2). The
  abort/degrade contract (nesting-refused ⇒ `deny worktree-subagent Bash`, never
  unwrapped) remains the standing failure posture, now un-exercised.
- **INV — `updatedInput` honoured on macOS (H1b analog) — PROVEN (pass 2).** All
  15 armed-yolo subagent Bash calls logged `WRAP`; the harness ran the rewritten
  `sandbox-exec` command, the original never ran. Previously proven on Linux only.
- **INV (F-G) — derive the worktree from PreToolUse `cwd` via git, NOT a path
  template.** macOS Agent-tool worktrees land at `<repo>/.claude/worktrees/agent-<id>`
  (≠ Linux `.worktrees/`, ≠ the `/private/tmp` location pass-1 assumed). The git
  relationship (toplevel ≠ main checkout, realpath'd) is the invariant; the path is
  harness-version surface. The `Jailer` MUST bind via git, load-bearing for the
  cross-arm seam.
- **ASSUMPTION (M1-sub permission-mode) — RESOLVED (F-F).** In the *subagent*
  context the permission gate is NOT transparent to writes (unlike pass-1
  orchestrator F-C): under `auto`, gate/operator-popup denials mask most vectors
  before Seatbelt. In-situ containment claims MUST be measured under
  `bypassPermissions` — which the verdict does.

## 6. Open Questions & Unknowns

- **OQ-mac1 — nesting vs harness Seatbelt (THE GATE). CLOSED (pass 2, 2026-07-01):
  SUPPORTED.** Subagent-context M1-sub proven under bypassPermissions; nesting
  composes, `updatedInput` honoured. See §5.5 INV(M1-sub) + `results.md` Pass 2.
- **OQ-mac2 — launchd IPC residual.** MEASURED LOWER than the brief feared:
  `launchctl submit` is **denied by Seatbelt default** (rc=1, no launchd job;
  control proves it works rc=0 unsandboxed); `at` denied too. Record as
  *measured-low residual* (OS-version variance unmeasured), owned by the
  IPC/egress wall (non-goal), not *open*.
- **OQ-mac3 — SL-182 seam ordering. RESOLVED (with user, 2026-07-01): design
  against SL-182's seam AS-IS.** SL-182 already upstreamed the cross-arm `Jailer`
  seam + capability-as-data `select_jailer` fork point (commits `6f97b50e`,
  `a7707b48`). SL-183 slots the Seatbelt argv/profile builder into that existing
  seam; no SL-183-driven refactor of SL-182. (See §7 D-mac2.) Note F-G constrains
  the seam: the `Jailer` derives the worktree from `cwd` via git, not a path
  template.
- **OQ-mac4 (F-E) — second temp surface** redirect-or-accept (see 5.5).

## 7. Decisions, Rationale & Alternatives

Seeded from the design-ahead brief (`seatbelt-seam-brief.md`); **D-mac1/2/3/4 are
PROPOSALS, not yet ratified with the user.**

- **D-mac1** — Seatbelt = allow-default-deny-write-except, not default-deny.
  *(Probe-confirmed feasible.)*
- **D-mac2** — single `Jailer` seam; reuse all of `jail.rs` except the argv/profile
  builder. *(Seam shape trait-vs-branch open — §6 OQ-mac3.)*
- **D-mac3** — `TMPDIR=<wt>/.tmp` + deny `/private/tmp`. *(Probe-confirmed working;
  needs F-E refinement for `/var/folders`.)*
- **D-mac4** — `network` knob → `(deny network*)`, coarseness caveat; egress non-goal.

## 8. Risks & Mitigations

- **R-mac1 — nesting refused (M1-sub).** Mitigation: degrade contract (`deny`,
  never unwrapped); pass-2 probe gates lock. *(Top risk.)*
- **R-mac2 — Seatbelt vanish (deprecated ~10.10, SBPL undocumented).** Mitigated by
  Anthropic's own sandbox-runtime + system `.sb` profiles depending on it. Low,
  not zero.
- **R-mac3 — tooling breakage from the floor (F-B/F-E).** Mitigation: device-sink
  allow-set + temp-surface decision; the probe surfaces the breakage class early.
- **R-mac4 — execution host.** Probe + verification require a macOS host (cannot run
  in the Linux/bwrap jail). Operator ships the slice to macOS to execute.

## 9. Quality Engineering & Validation

- Probe-first gate: RSK-014 H2 (`probe-h2-seatbelt/`) — pass 1 DONE (orchestrator),
  pass 2 (M1-sub in-situ, both permission modes) OUTSTANDING.
- `validate_policy` behaviour-preserved (shared, unchanged) — SL-182 suites green.
- Pass criterion (identical to H1): every external vector `denied`, wt writable,
  wrapper confirmed applied via `updatedInput`. Degrade contract asserted.

## 10. Review Notes

<!-- RSK-014 H2 pass-1 findings folded in 2026-07-01. Pending: user ratification of
     D-mac1..4 + OQ-mac3 (SL-182 seam ordering); pass-2 M1-sub probe; then
     adversarial review → lock. -->
