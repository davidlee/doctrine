# Regression fingerprint pins current_exe — use one binary for capture+diff

The S1 regression baseline cache keys each file `baseline-{base}-{fp}` where the
run-fingerprint `fp` (INV-8, `src/regression_run.rs::fingerprint`) hashes
`{argv, DOCTRINE_WORKER, worker-marker, current_exe}`. `current_exe` is the
resolved path of the *running* doctrine binary.

**Trap.** A jail carries several doctrine binaries: `./target/debug/doctrine`
(in-tree, stable path), `~/.cargo/bin/doctrine` (nix-store path — drifts on
every reinstall), and any earlier nix path. Capturing a baseline with one and
running `check regression diff` with another yields three different `current_exe`
→ three different `fp` → the diff reports **"no baseline under the current
run-fingerprint"**. This is an honest INV-8 cache miss, but the message points at
fingerprint drift, not the real cause (wrong binary), so it eats several probe
turns to diagnose.

**Compounding trap.** The tempting fix — re-run `capture` until it stops
missing — writes a *new* baseline. On the dispatch funnel the worker delta is
**staged** at that point, so the re-capture records a **post-delta** baseline;
diffing S against it cancels a genuine pass→fail regression into a false green.

**Discipline (funnel).**
- Pin **one** binary — `./target/debug/doctrine` (stable in-tree path, survives
  nix reinstall) — for **both** `capture` and `diff` across the whole drive.
- Capture the baseline at **clean B** only. If a delta is already staged,
  `git restore --staged --worktree <paths>` back to B (the delta is preserved in
  the un-reaped worker tree), capture, then re-import.
- Same base **string** for capture and diff (full SHA throughout) — a separate
  keying axis, see [[mem.signpost.doctrine.dispatch-claude-arm-wrong-base]].

Surfaced twice on SL-205 (PHASE-01 base-string, PHASE-02 binary-path). A better
error would name expected-vs-computed `fp` + the `current_exe` in play.
