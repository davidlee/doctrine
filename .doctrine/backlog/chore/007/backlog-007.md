# CHR-007: SL-057 VT dogfood + historical coverage backfill: real cargo-test checks for REQ-254/255/256/257/114

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred from the SL-057 `/close` (design §9 "Dogfood closure", explicitly optional).

SL-057 shipped the `coverage record`/`verify` machinery; §9 proposes dogfooding it
by recording real VT checks for the slice's own requirements (REQ-254 runnable
check identity, REQ-255 derived status, REQ-256 production write/withdraw, REQ-257
NF-001 confinement, REQ-114) and `coverage verify 57`-ing them green — replacing
hand-authored backfill.

**Why deferred (conscious, user-confirmed at close):**
- The VT machinery is already proven green *deterministically* by the e2e verify
  goldens (`coverage_verify_prints_transition_and_audit_lines` records a VT and
  drives it Planned→Verified). The dogfood adds real-req provenance, not machinery
  proof.
- There is no root `doctrine.toml`, so a *meaningful* (non-vacuous, §4) check needs
  literal `cargo test` `--command`s, run as `verify` subprocesses. Nested `cargo
  test` under the jail's shared `CARGO_TARGET_DIR` is the documented false-RED /
  contention footgun (`mem.pattern.testing.shared-cargo-target-false-red-in-worktree-audit`)
  — poor to introduce at closure.

**Do:** add a root `doctrine.toml` `[verification]` with per-suite aliases (or
literal commands) mapping each REQ to the test(s) that prove it; `coverage record`
a VT cell per req with a matcher asserting the suite's `test result: ok` line;
`coverage verify 57` green. Confirm no shared-target false result (touch + re-run).
Relates: RSK-008 (closure gate-on-Failed), which would make such cells load-bearing.
