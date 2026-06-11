# IMP-029: RV review verb family: black-box e2e CLI golden + /audit pilot coverage

Surfaced by SL-040 reconciliation audit (RV-001 F-3, major).

The review verb family has no black-box e2e CLI golden — every other numbered
kind ships an `e2e_*_cli_golden.rs`. Today every `run_*` is exercised only by
in-module unit calls; clap dispatch, the `--as` parsing arms, the `--json`
shorthand, and the actual binary surface are untested end-to-end. VA-1 — the
`/audit` pilot producing an RV instead of `audit.md` — is agent-mode only, so
the pilot integration path has no automated regression guard.

Work: add an `e2e_review_cli_golden.rs` covering new/raise/dispose/verify/
contest/withdraw/status/list/show under the turn guard, plus a golden for the
`/audit` → RV path.
