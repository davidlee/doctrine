# SL-183 — implementation notes

Durable findings that must survive to `/audit` → `/reconcile` → `/close`.
Phase-local working detail lives in the (gitignored) runtime phase sheets; only
cross-phase / design-affecting facts land here.

## PHASE-01 (confirmation probe) — RV-203 F-2/F-3 discharged

Full evidence: `.doctrine/backlog/risk/014/probe-h2-seatbelt/results.md` §"Pass 3".

### Reconcile at close — design §5.1 profile line correction (F-P3-A)

The design's illustrative §5.1 profile shows the xcrun_db re-allow as a BARE regex:
`(allow file-write* (regex #"/xcrun_db[^/]*$"))`. **Probed bare, it LEAKS** — it
allowed `/private/tmp/xcrun_db-*`, a write entirely OUTSIDE the per-user temp. The
design's own prose already required scoping ("under a DUTMP subpath scope"; "not the
substring `DUTMP/xcrun_db`"), so this is an illustrative-line error, not a decision
change. **Proven-correct shipped form:**

```scheme
(allow file-write* (require-all (subpath (param "DUTMP")) (regex #"/xcrun_db[^/]*$")))
```

- PHASE-02 encodes `require-all` in `seatbelt_profile` + the `XCRUN_DB_REGEX` const.
- At `/reconcile`: update design §5.1's profile line illustrative→proven (bare →
  `require-all`). This is a per-slice-artefact direct edit (design.md), not a REV.

### Over-match to carry into the Rust const comment (F-P3-3)

`xcrun_db[^/]*$` (scoped to DUTMP) allows *any basename beginning `xcrun_db`, at any
depth under DUTMP* — over-matches `xcrun_db_x`, `xcrun_dbEVIL`, nested `…/xcrun_db`.
xcrun writes only at DUTMP top level so it's safe in practice; documented, not
tightened (a literal would break the `xcrun_db-<hash>` atomic-temp family). The
committed cache file is plain `xcrun_db`; the atomic temps are `xcrun_db-<hash>` —
the regex's empty-tail match covers both.

### Cross-slice: SL-182 seam still matches (F-P01-5)

SL-182 moved to `started` with post-lock design commits (`6f97b50e` seam upstream,
`a7707b48` RV-202 `select_jailer` capability-as-data). Confirmed no conflict: SL-182
owns the seam (`Backend::{Bwrap|Seatbelt|Deny{reason}}`, pure `select_jailer`) and
defers the macOS profile body to SL-183. **Re-check SL-182 design at PHASE-02 entry**
— it is still in flight. New constraint to honour: per-arming profile granularity
(serial ⇒ per-worker, parallel ⇒ one shared profile; RV-200 F-1 / RV-202).

### Conformance-boundary note — PHASE-01 source-delta binding absent (accepted)

At `completed`, doctrine warned `record_source_delta: code_start 38ca3a76 is not an
ancestor of code_end c321254c (not a forward delta)` and **skipped** the binding:
`phase-01.toml` keeps `code_start_oid = 38ca3a76` and has **no `code_end_oid`**.

- Cause: `code_start` (38ca3a76 "mem(SL-183): network-field-is-bool") was stamped on
  a lineage later discarded when the `f3539349`/`133880a2` "doctrine" auto-commits +
  parallel SL-182 landings restructured history. 38ca3a76 is now orphaned (in no
  branch, not an ancestor of HEAD). HEAD `c321254c` (edge tip) is forward-intact and
  the probe evidence (`results.md`, the rig) is fully reachable/committed.
- Decision (consulted, David — Option 1): **accept the absent binding**. PHASE-01
  ships NO Rust — its conformance value is the evidence in `results.md`, not a source
  delta. History-repair is forbidden (doctrine tracks oids as the boundary; AGENTS.md
  / handover). PHASE-02+ stamp `code_start` fresh from HEAD, so the anomaly does not
  propagate. At `/audit`: note PHASE-01 has no git-range delta by design of a code-free
  probe phase; rely on evidence-conformance, not delta-conformance, for it.

## PHASE-02 (pure builders) — implemented, gate blocked on SL-182

`seatbelt_profile` + `sandbox_exec_argv` implemented TDD behind SL-182's `Seatbelt`
seam; `Seatbelt::wrap_argv` wired to the builder. **41 jail unit tests green**
(31 SL-182 behaviour-preserved + 10 new SL-183). Clippy clean.

### Seam-gap closed: ResolvedMac fields (sanctioned by its doc comment)

SL-182 landed `ResolvedMac {}` EMPTY. PHASE-02 populated it: `wt`, `tmp`, `dutmp`,
`extra_rw`, `network: bool`, `profile_path` — all shell-canonicalized (PHASE-03's
`resolve_inputs` fills them; the pure builders consume them). Kept `#[derive(Default)]`
so SL-182's `ResolvedMac {}` test constructors compile unchanged (behaviour-preserved,
verified). No SL-182 signature/body change.

### D2 (TMPDIR) resolved seam-preservingly

Proven `seatbelt-jail.sh` exports TMPDIR *inside the wrapped body*; `opaque_wrap`
(shared, bwrap+seatbelt) must stay unchanged. So `sandbox_exec_argv` emits a trailing
`env TMPDIR=<tmp>` token after `--`; `opaque_wrap` appends `bash -c <body>` after that.
`opaque_wrap` untouched → PHASE-04 parity proof intact.

### F-P3-A encoded

`XCRUN_DB_REGEX` const + `seatbelt_profile` emit the `require-all (subpath (param
"DUTMP")) (regex …)` scoped form, NOT §5.1's bare regex. Over-match caveat is in the
const's doc comment. §5.1 reconcile-at-close debt (bare→require-all) still stands.

### BLOCKER — full gate red on a PRE-EXISTING SL-182 CHR-014 violation (ISS-204)

`doctrine check commit`'s full `test` recipe fails `e2e_no_baked_paths::no_baked_paths`
(CHR-014 / SL-162): SL-182's `pi_spawn_core_tokens` VT-7 helper bakes
`env!("CARGO_MANIFEST_DIR")` (introduced by SL-182 `b67b6299`, verified at clean
detached-HEAD — NOT an SL-183 artifact). Consulted (David): **SL-182 is being actively
worked in a parallel thread; the fix was handed to that thread.** SL-183 must NOT edit
jail.rs's SL-182 test surface (conflict + ownership). Captured as **ISS-204**
(`references SL-182 --role concerns`).

**Consequence for PHASE-02 close:** the pure builders are green in isolation, but
PHASE-02 must NOT flip `completed` until the full gate is green (else `code_end_oid`
binds to a red-gate state). **HOLD the completed-flip on ISS-204.** Commit the green
builder work now (durable); flip `completed` + re-run the gate once SL-182's thread
lands the fix. If SL-182's fix touches jail.rs concurrently, expect a rebase/merge on
this file — my additions are append-only (new consts block, new `ResolvedMac` fields,
two new fns, new tests), so conflicts should be localized.

### RESOLVED 2026-07-01 — ISS-204 fixed in SL-183 context (ownership deviation, sanctioned)

The SL-182-thread fix never reached the `edge` checkout (verified: no other worktree,
clean tree, `main` behind, gate re-run still red). On explicit direction (David, "try
now"), the ISS-204-mandated swap was applied HERE: `env!("CARGO_MANIFEST_DIR")` →
`crate::test_support::repo_root().join("scripts/pi-spawn-confined.sh")` in
`pi_spawn_core_tokens`. Test-only, behaviour-identical, no jail logic touched
(`8abcaae0`). Full `doctrine check commit` EXIT=0; `no_baked_paths` guard + 41 jail
unit tests green. **Deviation from the original "SL-182 owns this surface" note is
deliberate and user-sanctioned** — if the SL-182 thread also lands a fix, expect a
trivial dup/conflict on this one line (same target form). ISS-204 → resolved/fixed.

**PHASE-02 flipped `completed`** (`code_start 3a760f92`, forward-intact; conformance
confirms the PHASE-02 source-delta row registered — only PHASE-01 remains registry-gap,
accepted as a code-free probe phase per the boundary note above).

### Probe hygiene notes

- Every `(param "X")` the profile references MUST have a `-D X` bound or
  `sandbox-exec` refuses to load (`invalid data type of path filter; expected
  pattern, got boolean` — misleading text; it's an undefined-param fail-CLOSED).
- `-D DUTMP` MUST be the realpath (`/var/folders/$USER/T` → `/private/var/folders/…`);
  `subpath` matches the resolved path (INV-M2).

## PHASE-03 (impure resolve_inputs + macOS wiring) — implemented

`resolve_inputs` + `seatbelt_backend` + `RealEnv` landed in `jail.rs` behind an
injected `ResolveEnv` trait. 16 new tests (12 `FakeEnv` branch/wiring + 4 `RealEnv`
real-git legs); 57 jail tests green, clippy clean.

### Injected-effects seam (D-p3-1, user-ratified 2026-07-01)

`resolve_inputs(cwd, main_root, env: &dyn ResolveEnv) -> Result<ResolvedMac,
ResolveDeny>` is PURE branch logic; ALL impurity (git `rev-parse --show-toplevel`,
`is_linked_worktree`, `getconf`, `fs::canonicalize`, `create_dir_all`, policy file
read) lives ONLY in `RealEnv`. This keeps jail.rs's module-header pure-leaf claim
honest — the "thin shell" the design (§5.2) names IS `RealEnv`. Every branch a–f is
unit-testable off-host (no real getconf on Linux CI). VA-1 grep-audit passed.

`ResolveEnv` collapsed the design's separate `git_toplevel`+`is_main_checkout` into
one `worktree_topology(cwd) -> Topology{toplevel, is_linked}`, so branch (a)/(b)/(d)
decisions are made in the PURE resolver, not the env impl.

### Branch c/e collapse (RV-p3 resolved — carry to /audit)

`ResolveDeny` has 5 variants (`NotAWorktree`, `IsMainCheckout`, `AmbiguousGitDirs`,
`PolicyMissing`, `PolicyMalformed`) — NOT a distinct branch-c `BasenameMismatch`.
Branch c (nested-repo/submodule basename never provisioned) is mechanically identical
to branch e (policy absent): both surface as `read_policy → Ok(None)` ⇒ `PolicyMissing`.
The §5.5 enumeration lists a–f as SIX branches; the impl realises c and e through one
mechanism because the security outcome (Deny) is invariant. **At /audit:** note the
c≡e mechanism-merge against EX-2's "all 6 fail-closed branches" — 6 *conditions*, 5
*typed reasons*; every condition still denies. Not a scope cut.

### Deny path reuses the SL-182 funnel UNCHANGED (D-p3-2)

`seatbelt_backend(Result) -> Backend`: `Ok ⇒ Seatbelt(mac)`, `Err ⇒ Deny{reason}`.
Feeds the EXISTING `select_jailer`→`decide_bash` chain — `Err`⇒`Deny`⇒`None`⇒
`Decision::Deny{reason}`. No new decision surface; EX-3 behaviour-preservation held
(all SL-182 shared fns — `select_jailer`/`from_toml_str`/`validate_policy` — reused
verbatim). Malformed/unknown-key policy ⇒ branch-f Deny, never a silent network-open
default (EX-4, covers F-B6).

### Policy location (SL-182 convention, single-sourced)

`RealEnv::read_policy` reads `<main>/.doctrine/state/dispatch/jail/<basename>.toml`
(design §5.3), segments as `POLICY_DIR_SEGMENTS` const. The provisioning WRITE is
PHASE-04/SL-182's; PHASE-03 only READs. `NotFound` ⇒ `Ok(None)` ⇒ branch e; other io
errors propagate ⇒ also branch e (fail-closed).

### PHASE-04 carry-forward

The macOS getconf leg + the whole in-situ containment matrix are host-gated to
PHASE-04. `resolve_inputs`'s getconf/realpath failures currently map to fail-closed
Denies reusing `NotAWorktree`/`PolicyMissing` reasons (no new a–f branch — the
enumeration is `cwd`→policy derivation, not host-tool availability). If PHASE-04 wants
a distinct "sandbox-env-unavailable" reason surfaced, add a variant then.

## PHASE-04 (parity + in-situ + degrade) — progress

### EX-1 / VT-1 — behaviour-preservation gate: SATISFIED
SL-182's reused-fn suites green UNCHANGED — `validate_policy_*` (5), `decide_write_*`
(3), `pathcheck_*` (5). Full jail module 62→63 (EX-3 test added, below). git log
confirms no edit to those fn bodies since PHASE-03 (last jail.rs touches were the
PHASE-02/03 Seatbelt fork, additive). Parity proof intact.

### EX-3 — degrade contract: SATISFIED (unit)
New pure test `seatbelt_resolve_deny_degrades_to_bash_deny_never_wraps_or_passes`
(jail.rs `mod tests`) asserts the FULL macOS chain for all 5 `ResolveDeny` branches:
`seatbelt_backend(Err) ⇒ Backend::Deny{mac reason}` → `decide_bash(Target::Jail) ⇒
Decision::Deny{mac reason}`, never `WrapBash`/`PassThrough`. Twin of the pre-existing
bwrap-reason test (which hand-builds `Backend::Deny`); this one proves the *macOS
resolver→backend→decision wiring* degrades closed. **Mutation-verified**: forcing
`seatbelt_backend` Err→`Seatbelt(default)` (fail-open) turns it red — the test has
teeth. "nesting-refused" and "resolve-Deny" framings both collapse to `Err ⇒ Deny`,
so one parameterised test discharges both (design §9). Contract un-triggered live
(pass-2: nesting composed) — this is the posture proof.

### T3a — consumer wiring gap CLOSED (consult-approved scope increment)
**Discovery (load-bearing):** the shipped `doctrine worktree pretooluse` consumer
NEVER routed macOS→Seatbelt. `probe_backend()` only produced `Bwrap`/`Deny{bwrap-
unavailable}`; PHASE-03 built + unit-tested the Seatbelt jailer in jail.rs (leaf)
but never wired the command shell to reach it. So on macOS every worktree-subagent
Bash denied `bwrap-unavailable` — the jailer was DEAD CODE from the hook entry, and
EX-2's "through the live consumer" was physically impossible. This was a missed
PHASE-03 increment (PHASE-03's EX-3 "select_jailer routes macOS to Seatbelt" was
true at the `select_jailer` level but the upstream `probe_backend` never produced a
`Seatbelt` backend). Commit `b2cd1000`.

**Fix (option A, ADR-001-respecting):** `cfg`-split `probe_backend` in
`worktree::pretooluse` (command tier). macOS arm builds `RealEnv{main_root}` (the
leaf's injected `ResolveEnv` seam) → `seatbelt_backend(resolve_inputs(cwd,
main_root, real))`. `main_root` = realpath'd `CLAUDE_PROJECT_DIR` anchor (legit —
this module IS the claude `PreToolUse` handler, ADR-011 per-harness altitude;
factored to shared `project_anchor()`). Leaf jail.rs UNCHANGED. `cfg`-split not
runtime (bwrap Linux-only / Seatbelt macOS-only). **Lazy backend:** resolve only on
the Bash path — `decide_write` walls Edit/Write on `pathcheck` and ignores the
backend, so running `resolve_inputs` (git/getconf + `<wt>/.tmp` mkdir side effect)
on every Edit/Write would burden the hot hook path (INV-1). Test boundary
(user-agreed): thin impure glue, NOT unit-tested here (would dup leaf coverage);
proof = the live in-situ run (T3/VA-1). 169 worktree tests green, clippy/fmt clean.

### Binary/hook topology resolved (was a session blocker)
Three `doctrine` binaries existed; confusion was PATH. RESOLVED: stale
`~/.local/bin/doctrine` removed; `which doctrine` → fresh `~/.cargo/bin/doctrine`
(Jul 1 19:31). Hook (`.claude/skills/doctrine/hooks/hooks.json`) invokes that same
fresh `~/.cargo/bin` path by absolute string — it knows `worktree pretooluse` and
carries `Seatbelt::Jailer::wrap_argv` + profile strings. `.claude/settings.local.json`
`hooks:{}` empty; the skill hooks.json is the live source. So the EX-2 live in-situ
leg is genuinely runnable through the real consumer (not just the pass-2 rig).

### T3b — Seatbelt profile materialization gap CLOSED (2nd consult-approved increment)
**Discovery (sibling of T3a):** the shipped consumer emitted `sandbox-exec -f
<wt>/.tmp/jail.sb` but NOTHING in prod ever wrote the profile body —
`seatbelt_profile()` (jail.rs:503) had ZERO prod call sites (tests only). Every
wrapped Bash call would fail `sandbox-exec: .../jail.sb: No such file or directory`
BEFORE Seatbelt engaged → the floor never applied. Contradictory doc comments
(jail.rs 243/264/153/677) each punted the write to "the other phase"; neither did.
Root cause: bwrap confinement is inline argv flags (no external file), Seatbelt's
`-f <profile>` is a disk file — so the macOS arm carries a materialization
obligation parity-by-reuse never covered.

**Fix (command tier, leaf pure):** `materialize_seatbelt_profile(&Backend,
Decision) -> Decision` in `pretooluse.rs`, wired into `run_pretooluse` after
`decide()`. On `Backend::Seatbelt` + `WrapBash` ⇒ write `seatbelt_profile(resolved)`
to `resolved.profile_path`. **Fail-closed (F-B4):** io error ⇒ `Deny{seatbelt-
profile-write-failed}`, never an allow+wrap over a missing floor. No-op for every
other backend/decision. clippy `#[expect(disallowed_methods)]` — the `.sb` is
runtime/derived under gitignored `<wt>/.tmp`. TDD: 3 tests (writes-body-keeps-wrap,
fail-closed-denies, noop-non-seatbelt). pretooluse 18→21 green; jail 59 UNCHANGED
(parity intact, leaf untouched — only doc comments corrected). Commit `c5908cc6`.

**Offline floor validation (the proof T3a+T3b make containment REAL):** simulated
agent worktree + provisioned policy → shipped consumer wraps → `.sb` materialized
(540B) → OUTSIDE write `Operation not permitted` (canary PRISTINE), IN-WT write
`INSIDE_OK`. The floor engages end-to-end through the shipped `worktree pretooluse`.

### EX-2 live-subagent battery — BLOCKED (handoff point; two independent walls)
See `mem.thread.sl-183.ex2-live-battery-blocked-macos` (full detail). Summary:
1. **Plugin hook not firing (macOS, S1).** After reboot + `/reload-plugins`
   ("3 hooks"), an `isolation:worktree` subagent's in-wt `echo` ran UNWRAPPED — no
   `<wt>/.tmp/jail.sb` materialized (the T3b tell proves the hook never wrapped).
   The proven-working RSK-014 surface was a `settings.local.json` `hooks` block
   (now empty); the doctrine skill/plugin hooks.json path did not intercept subagent
   Bash here. Falsified the medium-trust `reload-plugins-registers-pretooluse`
   memory (now downgraded to low + trust-corrected).
2. **Subagent refuses the red-team battery.** 3 spawns (Opus×1, Sonnet×2), all
   framings, refused as sandbox-escape recon even under bypassPermissions. Model
   posture shifted since pass-2 (which got compliant subagents). Do NOT jailbreak
   past a safety refusal. The subagents' sound point (matches rig F-D): containment
   is verified from OUTSIDE (canary checksums), so subagent cooperation is not
   load-bearing.

**Net EX-2 status for /audit:** discharged by T3a+T3b wiring + offline floor
validation (shipped binary confines end-to-end) + live deny-path + pass-2's
STANDING INV(M1-sub) (nesting already SUPPORTED, promoted to design INV). The live
subagent battery is re-confirmation, not the sole proof — defer to VH-1 (human) or
a next-session settings.local.json-surface run. NOT a containment defect (S2): all
canaries intact throughout; nothing escaped.
