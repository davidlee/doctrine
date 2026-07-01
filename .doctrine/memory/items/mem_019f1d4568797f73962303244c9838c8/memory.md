# macOS Seatbelt jail needs the .sb profile written to disk by the command tier — bwrap parity doesn't cover it

The Seatbelt wrap argv references sandbox-exec -f <wt>/.tmp/jail.sb; that file must be materialized by pretooluse::materialize_seatbelt_profile (command tier, fail-closed) — the pure leaf never writes it, and the bwrap arm has no analog (inline flags).

**The trap.** The pure layer is deceptively complete: `seatbelt_profile(resolved)`
builds the SBPL body, `sandbox_exec_argv` emits `-f <profile_path>`, `resolve_inputs`
computes the path and `ensure_dir`s `<wt>/.tmp`. Everything looks wired — but nothing
writes the body to `profile_path`. `seatbelt_profile()` had ZERO prod call sites
(tests only) until SL-183 PHASE-04 T3b. Symptom of the gap: every wrapped Bash call
dies `sandbox-exec: .../jail.sb: No such file or directory` and Seatbelt never
engages — the command just errors, so a naive canary check can read as "contained"
when in fact nothing ran.

**Why bwrap parity misses it.** bwrap confinement is 100% inline argv flags
(`--bind`, `--unshare-net`, …) — no external file. Seatbelt's `-f <profile>` is a
disk file. So the macOS arm carries a materialization obligation the Linux arm never
had; "reuse the same funnel, fork only the argv/profile builder" (the slice's design
seam) silently under-covers it.

**The fix (the seam).** `materialize_seatbelt_profile(&Backend, Decision) -> Decision`
in the impure command tier (`pretooluse.rs`), called in `run_pretooluse` right after
`decide()`. Writes ONLY on `Backend::Seatbelt` + `WrapBash`; io error ⇒ fail-closed
`Deny{seatbelt-profile-write-failed}` (F-B4 — never emit an allow+wrap over a missing
floor). The pure leaf (`seatbelt_profile`) stays untouched — parity preserved. This
is the exact sibling of the T3a consumer-wiring gap (macOS probe_backend was also dead
code from the hook entry). Pattern lesson: when a cross-platform seam forks "only the
builder", audit for per-arm SIDE EFFECTS (disk writes, env) the shared funnel doesn't
carry — a pure builder that returns a path is not the same as one that writes it.
See [[mem.pattern.macos.doctrine-hook-reinstall-resign]] (the other macOS live-run
footgun).
