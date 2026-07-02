# Dispatch worker prompt must run the full suite, not only the phase e2e

A distilled dispatch-worker prompt that tells the worker to run only *its phase's
own* test binary (e.g. `cargo test --test e2e_worktree_jail_prefix`) lets the
worker report green while a **different** binary is red — the worker never runs it,
so never sees it.

## What happened (SL-185 PHASE-03, 2026-07-02)

The P03 worker hoisted `pi-spawn-confined.sh`'s inline `bwrap … pi` flags into a
`PREFIX=( … )` bash array. That reformat broke a **unit** test in
`src/worktree/jail.rs` — `bwrap_core_argv_matches_pi_spawn_core_flags` (VT-7), a
`--bin doctrine` test that scrapes the script and asserts flag parity. The worker
ran only `cargo test --test e2e_worktree_jail_prefix` (10 green) and reported the
phase complete. The orchestrator's regression **diff** at S caught the red unit
test (`new: worktree::jail::tests::bwrap_core_argv_matches_pi_spawn_core_flags`)
and halted the funnel — the honest catch, but one round late.

**Why:** a phase's blast radius is not bounded by its declared test file. Any test
that reads a file the phase edits (parity scrapers, golden files, snapshot tests)
can go red from a legitimate change, and it lives in a different binary.

## How to apply

- In the distilled worker prompt, mandate the **regression-relevant** suite, not
  just the phase e2e: at minimum `cargo test --bin doctrine <touched-module>` for
  every module the delta touches (or reads), **plus** the phase e2e. For a script
  edit that a `--bin` test scrapes, that means `cargo test --bin doctrine worktree`.
- The orchestrator's `check regression diff` is the backstop, not the worker's
  self-report — but a worker that runs the right suite red/greens it itself and
  saves a funnel round. Treat the worker's "all green" as scoped to what it ran.

**Relates to RFC-005** (dispatch funnel integrity — hazard survey): a worker
false-green is an availability/throughput hazard, not a correctness one (the S-diff
gate holds the line), but it costs a funnel round and a manual co-fix. Sharpening
the worker prompt's suite mandate closes it at the source.

See also [[mem.pattern.dispatch.pi-spawn-parity-scraper-couples-to-script-format]]
(the specific scraper that broke) and [[mem_019f096865017b8394c9ac82eeed23ff]]
(re-run the suite in audit; distrust dispatch handover failure labels).
