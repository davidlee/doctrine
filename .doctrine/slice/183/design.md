# Design SL-183: macOS Seatbelt write-confinement arm

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

<!-- STATUS: design LOCKED 2026-07-01 (slice → plan). §§ below carry the RSK-014 H2
     probe findings (pass 1 orchestrator + pass 2 in-situ subagent, both DONE —
     `.doctrine/backlog/risk/014/probe-h2-seatbelt/`). D-mac1..4 RATIFIED; OQ-mac4
     RESOLVED (anchored xcrun_db allow); RV-203 inquisition discharged (9 findings
     resolved). Verification obligations carried into /plan: final DUTMP/xcrun_db
     profile probe + SBPL regex semantics (RV-203 F-2/F-3). -->

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
- **STD-001** no magic strings: profile tokens, `-D` param names (`WT`/`TMP`/`PTMP`/
  `DUTMP`/`RWn`), the device-sink allow-set, the `xcrun_db` cache-file regex
  (F-E), bind/deny flag strings → single-sourced named constants.
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

The profile. The **base** shape (allow-default, floor, PTMP deny, device sinks, WT/
TMP/RWn allows) is **probe-proven** (RSK-014 H2 pass 1). The **DUTMP deny + xcrun_db
re-allow** (L68/L75) are **design-decided post-probe (OQ-mac4), NOT yet probed** —
their exact ordering + canary-preservation + xcrun-tool-works verification is
DEFERRED to `/plan` / first-impl (§9, F-B2). Do not read them as pass-1 evidence.
```scheme
(version 1)
(allow default)                                 ; reads open (parity: reads OOS)
(deny file-write*)                              ; the floor
(deny file-write* (subpath (param "PTMP")))     ; F-A: coarse deny FIRST (see 5.5)
(deny file-write* (subpath (param "DUTMP")))    ; /var/folders/$USER/T — coarse FIRST
; device write sinks — F-B (literals/regex, must stay writable):
(allow file-write* (literal "/dev/null")) … (regex #"^/dev/tty") …
(allow file-write* (subpath (param "WT")))      ; worktree rw — SPECIFIC, LAST
(allow file-write* (subpath (param "TMP")))     ; TMPDIR=<wt>/.tmp (D-mac3)
; F-E: re-allow ONLY the xcrun_db cache FILE FAMILY under the per-user temp, NOT
; the whole surface — narrowest hole that fixes the proven breakage (OQ-mac4).
; ANCHORED to one path segment (F-3): observed file is `xcrun_db-<hash>`, so the
; pattern is DUTMP + "/xcrun_db" + non-slash* + end — never a deeper subpath.
; XCRUN_DB_REGEX (named const); SBPL regex-match semantics pinned at /plan-probe:
(allow file-write* (regex #"/xcrun_db[^/]*$"))  ; applied under a DUTMP subpath scope
(allow file-write* (subpath (param "RWn")))     ; per validated extra_rw
; (deny network*)  iff policy.network == deny   ; default OPEN; emitted only on
;                                               ; opt-in, via the same policy→profile
;                                               ; pass as extra_rw (D-mac4 thin seam)
```
Invoked: `sandbox-exec -D WT=<realpath> -D TMP=<realpath> -D PTMP=/private/tmp
-D DUTMP=<realpath getconf DARWIN_USER_TEMP_DIR> -D RWn=… -f <profile>
-- bash -c "$(base64 -d <<<$B64)"`. Children inherit.

### 5.2 Interfaces & Contracts

**Pure/impure split (ADR-001, F-B1).** Realpath, `getconf`, `<wt>/.tmp` creation,
and policy-file reads are IMPURE — they live in the thin shell (`resolve_inputs`),
which fails closed. The two builders are PURE: resolved paths/strings in,
`String`/`Vec` out. No clock/exec/disk in the pure layer.

- **Impure — `resolve_inputs(cwd, policy) -> Result<ResolvedInputs, Deny>` (shell).**
  Derives the worktree from `cwd` via git (§5.5 F-G / F-B4 fail-closed contract),
  realpaths WT/TMP/DUTMP/extra_rw, runs `getconf DARWIN_USER_TEMP_DIR`, ensures
  `<wt>/.tmp`. **Any failure ⇒ `Deny` (arm denies `worktree-subagent Bash`).**
- **Pure — `seatbelt_profile(resolved) -> String`** — emits the profile body, **rules
  ordered deny-coarse-first / allow-specific-last** (F-A). Device-sink allow-set and
  the `xcrun_db` filename regex (F-E) are named constants (§ constant catalog below).
  The `(deny network*)` line is emitted **only** when `resolved.network == false`
  (SL-182's bool, reused as-is) — default-open on a VALID policy; network rides the
  same policy→profile pass as
  `extra_rw`, never a hardcoded special case (D-mac4). *(Ambiguity handling: an
  unreadable/malformed policy never reaches here — `resolve_inputs` already denied
  the arm, F-B6.)*
- **Pure — `sandbox_exec_argv(resolved) -> Vec<OsString>`** — splices realpath'd
  `-D` params (F-A footgun mitigation), opaque base64 body, sets `TMPDIR=<wt>/.tmp`.

**Named-constant catalog (STD-001, F-minor9).** The design commits `/plan` to
single-source these as Rust `const`s (identifiers illustrative; values are the
contract): `PARAM_WT`/`PARAM_TMP`/`PARAM_PTMP`/`PARAM_DUTMP`/`PARAM_RW_PREFIX`
(`-D` names); `PTMP_LITERAL = "/private/tmp"`; `DEVICE_SINK_ALLOWS` (the F-B set);
`XCRUN_DB_REGEX` (the anchored filename pattern, F-3); `DENY_NETWORK = "(deny
network*)"`. §5.1's profile shows literals for readability ONLY — none ship inline.

**Seam shape (D-mac2, RATIFIED — a CONSTRAINT on SL-182, not landed code).** SL-182
is `ready` but UNBUILT: its `select_jailer` capability-as-data fork point exists in
SL-182's *design*, not on disk (§2, F-B5). SL-183 forks **only** the argv/profile
builder behind that fork point; OQ-mac3 resolved *slot-in-as-is* (no SL-183-driven
SL-182 refactor). D-mac2 is therefore a requirement SL-182's implementation MUST
satisfy — SL-183 planning stays blocked on `needs SL-182` until the API lands.

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
- **EDGE (F-E) — `/var/folders/$USER/T` is a SECOND temp surface. RESOLVED
  (OQ-mac4, 2026-07-01).** macOS per-user temp (`DARWIN_USER_TEMP_DIR`, `$TMPDIR`
  default), distinct from `/tmp`; xcrun hardcodes an `xcrun_db` cache there. The
  `TMPDIR=<wt>/.tmp` redirect does NOT cover it (OBSERVED: the `xcrun_db-<hash>`
  write escaped the `$TMPDIR` redirect — results.md F-E; mechanism *likely*
  `confstr(_CS_DARWIN_USER_TEMP_DIR)`-derived, **unverified**, F-minor8) → denied,
  noisy (cosmetic for python; breaks cache-dependent tools). **Decision:** coarse-deny
  the whole surface, then re-allow **only** the `xcrun_db` cache-file family via an
  ANCHORED filename regex (`/xcrun_db[^/]*$` under a DUTMP subpath scope, F-3), NOT
  the substring `DUTMP/xcrun_db` — the smallest hole that fixes the proven breakage.
  The rest of
  the per-user temp stays denied. **Caveat (load-bearing):** this is a deliberate
  containment tradeoff — `/var/folders/$USER/T` is host-shared and GC-uncontrolled,
  so the allowed cache file is a cross-subagent write surface OUTSIDE the floor;
  scoping to `xcrun_db` keeps it to one OS-owned filename. Other Xcode tools with
  different cache files will still deny → re-surface case-by-case as encountered.
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
- **INV (F-B4) — the `cwd`→worktree derivation is FAIL-CLOSED (POL-002).** The
  derivation algorithm (in the impure `resolve_inputs`): `git -C <cwd> rev-parse
  --show-toplevel` → realpath → `basename` → per-arming policy lookup
  (§5.3). **Every failure branch ⇒ `Deny` (arm emits `deny worktree-subagent Bash`);
  NEVER a fallback path template, NEVER unwrapped pass-through.** Enumerated denies:
  (a) `cwd` not inside a git worktree (rev-parse fails); (b) toplevel == the main
  checkout (not a subagent worktree — no policy provisioned); (c) nested repo /
  submodule where toplevel is the inner repo (basename ≠ a provisioned arming); (d)
  ambiguous / multiple gitdirs; (e) policy file for the resolved basename missing or
  unreadable; (f) policy present but malformed / schema-invalid (covers the network
  ambiguity, F-B6). This is the macOS twin of SL-182's fail-closed posture and the
  POL-002 discharge for the whole arm.
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
  against SL-182's seam AS-IS.** SL-182's *design* specifies the cross-arm `Jailer`
  seam + capability-as-data `select_jailer` fork point (SL-182 commits `6f97b50e`,
  `a7707b48` author the DESIGN; the seam is **not yet built** — jail.rs does not
  exist on disk, §2/F-B5). SL-183 slots the Seatbelt argv/profile builder into that
  fork point *as-is* — a CONSTRAINT SL-182's implementation must satisfy, not
  landed API; no SL-183-driven refactor of SL-182 (See §7 D-mac2). F-G constrains
  the seam: the `Jailer` derives the worktree from `cwd` via git (fail-closed,
  F-B4), not a path template.
- **OQ-mac4 (F-E) — second temp surface. RESOLVED (with user, 2026-07-01):**
  coarse-deny `/var/folders/$USER/T`, re-allow ONLY the `xcrun_db` cache-file family
  via an ANCHORED filename regex (F-3; not the whole surface, not a redirect — the
  write escapes the `$TMPDIR` redirect, mechanism likely `confstr`-derived but
  UNVERIFIED, F-minor8). **Decision made, exact final profile NOT yet probed** —
  ordering + canary + xcrun-works verification deferred to `/plan`/first-impl
  (F-B2). Documented cross-subagent caveat + case-by-case
  re-surfacing for other Xcode caches. See §5.5 EDGE(F-E), §5.1 profile.

## 7. Decisions, Rationale & Alternatives

Seeded from the design-ahead brief (`seatbelt-seam-brief.md`); **D-mac1/2/3/4
RATIFIED with the user 2026-07-01.**

- **D-mac1 — RATIFIED.** Seatbelt = allow-default-deny-write-except, not
  default-deny (the SBPL footgun this design sidesteps). *(Probe-confirmed feasible.)*
- **D-mac2 — RATIFIED.** Single `Jailer` seam; reuse all of SL-182's `jail.rs`
  except the argv/profile builder, slotting into SL-182's `select_jailer` fork point
  as-is — no SL-183-driven SL-182 refactor (OQ-mac3 resolved). **This is a CONSTRAINT
  on SL-182's eventual implementation, not landed API** — SL-182 is `ready` but
  UNBUILT; the fork point exists in SL-182's design, not on disk. SL-183 planning
  stays `needs SL-182`-blocked until it lands (F-B5). **Constraint (F-G/F-B4):** the
  seam derives the worktree from PreToolUse `cwd` via git (toplevel ≠ main checkout,
  realpath'd), **fail-closed on every ambiguity** (§5.5 F-B4), NOT a path template —
  macOS Agent worktrees land at `<repo>/.claude/worktrees/agent-<id>`, a
  harness-version surface; the git relationship is the invariant.
- **D-mac3 — RATIFIED (verification deferred).** `TMPDIR=<wt>/.tmp` + deny
  `/private/tmp`. Folds OQ-mac4: coarse-deny `/var/folders/$USER/T`, re-allow ONLY
  the `xcrun_db` cache-file family (anchored regex, F-3). *(Base profile
  probe-confirmed; the DUTMP/xcrun_db terms are design-decided, NOT yet probed —
  verify at /plan, F-B2. See §5.5.)*
- **D-mac4 — RATIFIED (default-open thin seam).** Network defaults **open** (the
  operating default) **on a VALID policy**. `(deny network*)` is emitted **only** on
  `policy.network == deny`, via the same policy→profile pass as `extra_rw` — not a
  hardcoded special case. **Ambiguity (F-B6, POL-002):** reuse SL-182's `network`
  field AS-IS — a **bool** (`true` default = open, `false` = deny), a closed 2-value
  domain (NOT widened to an enum — that would be the forbidden SL-182 refactor,
  OQ-mac3). A missing/malformed policy does NOT silently default open — `resolve_inputs`
  fails closed and the *whole arm* denies (§5.5 F-B4 branch f). Default-open is a
  property of a validated policy only; on the Seatbelt arm `network == false` emits
  `(deny network*)`. A finer host/port/iface
  egress model later is a policy-schema extension, not a seam refactor (forward-compat
  by design). Coarseness caveat stands: syscall-deny, not iface removal (asymmetric
  with bwrap's netns by design); egress wall remains a non-goal (IPC/egress territory).

## 8. Risks & Mitigations

- **R-mac1 — nesting refused (M1-sub).** **RETIRED to standing posture** — pass-2
  proved nesting composes (SUPPORTED); the degrade contract (`deny`, never unwrapped)
  is now the un-exercised failure posture, not an open gate. *(Was top risk.)*
- **R-mac2 — Seatbelt vanish (deprecated ~10.10, SBPL undocumented).** Mitigated by
  Anthropic's own sandbox-runtime + system `.sb` profiles depending on it. Low,
  not zero.
- **R-mac3 — tooling breakage from the floor (F-B/F-E).** Mitigation: device-sink
  allow-set + temp-surface decision; the probe surfaces the breakage class early.
- **R-mac4 — execution host.** Probe + verification require a macOS host (cannot run
  in the Linux/bwrap jail). Operator ships the slice to macOS to execute.

## 9. Quality Engineering & Validation

- Probe-first gate: RSK-014 H2 (`probe-h2-seatbelt/`) — **pass 1 DONE (orchestrator)
  + pass 2 DONE (M1-sub in-situ, bypassPermissions leg), both SUPPORTED.** The gate
  is discharged. **One item remains UNVERIFIED (not a re-probe of the gate):** the
  exact final DUTMP-deny + anchored-`xcrun_db`-allow profile (OQ-mac4) was
  design-decided *after* the probes — its ordering, canary-preservation, and
  xcrun-tool-still-works must be confirmed at `/plan`/first-impl before the arm
  ships that profile (F-B2/F-3). This is a bounded verification obligation carried
  INTO planning, not a lock blocker for the design decision itself.
- `validate_policy` behaviour-preserved (shared, unchanged) — SL-182 suites green.
- Pass criterion (identical to H1): every external vector `denied`, wt writable,
  wrapper confirmed applied via `updatedInput`. Degrade contract asserted.
- Fail-closed derivation (F-B4) verified: each enumerated `cwd`→worktree failure
  branch ⇒ `deny worktree-subagent Bash` (unit-testable in the pure/shell split).

## 10. Review Notes

<!-- RSK-014 H2 pass-1 + pass-2 findings folded 2026-07-01. D-mac1..4 RATIFIED,
     OQ-mac3 + OQ-mac4 RESOLVED, M1-sub probe DONE (SUPPORTED). Pending: adversarial
     /inquisition pass → resolve RV → user lock → /plan. -->
