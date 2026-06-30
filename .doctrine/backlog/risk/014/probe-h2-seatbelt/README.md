# Probe H2 apparatus + evidence (RSK-014 — macOS Seatbelt arm, SL-183 / IMP-045)

Sibling of `../probe-h1/` (the Linux/bwrap arm, H1 SUPPORTED). Validates the
foundations the SL-183 Seatbelt design rests on **before any Rust** (the
design-ahead brief's §6 probe-first gate).

Question → `probe-brief-h2-seatbelt.md`. Answer → `results.md`.

**Verdict (pass 1, orchestrator context): H2 SUPPORTED.** A write-floor
`sandbox-exec` profile (`allow default` / `deny file-write*` / re-allow under
realpath'd worktree+TMP) contains every external write vector; all canaries
intact. Two gating unknowns hold (M1-orch nesting, M2 canonicalization incl.
hardlink); the `launchctl submit` IPC residual is empirically denied by default.
**Pass 2 (deferred):** the real `isolation:worktree` subagent in-situ leg (M1-sub)
under both permission modes.

## Apparatus
- `seatbelt-jail.sh` — the profile + argv builder; shell analog of the Rust
  `seatbelt_profile(policy)` / `sandbox_exec_argv(wt, policy)` seam. Opaque base64
  body, **realpath'd `-D` params** (the footgun mitigation), deny-coarse-first /
  allow-specific-last rule ordering (finding F-A).
- `battery.sh` — 13-vector escape battery (11 parity vectors + macOS-only
  `launchctl submit` / `at`). Drives each vector INSIDE the floor.
- `canaries/setup.sh`, `canaries/verify.sh` — plant + independently verify
  checksummed canaries (parent tree, shared-`.git` analog, `/private/tmp`, `$HOME`).
- `probe-brief-h2-seatbelt.md` — the hypothesis (H2 + M1–M5 falsifiable claims).

## Key findings (see results.md)
- **F-A** SBPL last-match-wins → `deny /private/tmp` shadows `allow WT` (worktrees
  live under `/private/tmp`). Order deny-coarse-first. **Design-load-bearing.**
- **F-B** floor denies `/dev/null` et al. → re-allow device sinks.
- **F-E** `/var/folders/$USER/T` is a second tmp surface (xcrun cache) the
  `TMPDIR` redirect misses.
- **F-D** battery self-report lied; the independent canary verifier caught it.
- **F-C** permission-mode is not a write confound in the orchestrator context
  (bare-write control); subagent context is pass 2.

## Re-run
```
export PROBE_BASE=/path/to/gitignored/scratch
bash canaries/setup.sh && bash battery.sh && bash canaries/verify.sh
```
Scripts are committed authored evidence; run artifacts stay in gitignored scratch.
NOTE: `canaries/setup.sh` plants two canaries OUTSIDE the scratch dir
(`/tmp/h2_ptmp_canary`, `$HOME/.h2_home_canary`) to exercise the alias/$HOME
vectors — `rm -f` them after a run (the verifier lists them).
