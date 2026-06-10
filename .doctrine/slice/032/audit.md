# Audit SL-032 — disposition of code-review findings

Source: `review.md` (code-review skill, 2026-06-10). Gate at audit: build clean,
`cargo clippy` zero warnings, 738 unit + all e2e green.

Every review finding is dispositioned below. Verdict: **close-ready** once the
now-batch lands; structural debt is routed, not lost.

| # | Finding | Disposition | Where |
|---|---|---|---|
| F-1 | Trunk-ref allocation inert in every production mint path (`&[]`); closure-intent overclaimed it as live | **Fixed (doc)** — closure-intent caveated; this audit records it | `slice-032.md` closure intent; this file |
| F-2 | `integrity::KINDS` is a parallel 12th identity table (raw copies of `entity::Kind` dir/prefix), no compile link; new kind silently escapes `validate` (R-b) | **Deferred → SL-031** — SL-031 becomes the 2nd consumer; it builds the shared registry that both derive from, incl. the set-equality guard test. Doing it now = guess-then-reshape | `slice-031.md` Scope |
| F-3 | `write_class_tests` — ~330 lines of struct-literal theatre, brittle to any field change, bypasses clap | **Backlog (slice-seed)** — argv-driven rewrite; independent of SL-031 | IMP-010 |
| F-4 (exit) | reseat exits non-zero even on a fully-completed reseat (danglers) | **Accepted w/ rationale** — design R-3 deliberately forces the human to act; contract now documented at the verb | `integrity.rs` `run_reseat` doc |
| F-4 (atomicity) | reseat's six post-guard fs ops are non-transactional; mid-sequence failure leaves a half-reseated entity | **Backlog (slice-seed)** — harden to atomic / staged | IMP-010 |
| F-5 | `has_runtime_state: bool` + hardcoded `.doctrine/state/slice` couples the F3 guard to slice | **Deferred → SL-031** — `KindRef` carries the state dir as part of the registry rework (rides F-2) | `slice-031.md` Scope |
| F-6 | `line_cites` trailing-alpha leak (`SL-031x` matched as a citation) | **Fixed** — `after_ok` rejects alphanumeric; test added | `integrity.rs` |
| F-7 | `scan_danglers` globs disposable prose (`handover.md`, runtime `state/**`) → nag noise | **Fixed** — `is_disposable_prose` skip; test added | `integrity.rs` |

## Good (no action)
Exhaustive wildcard-free `write_class` (compile-error-on-new-verb); clean
pure/impure split (env-injected `trunk_ladder`, F4 hard-error asymmetry);
`next_id(local,&[])` byte-identical-to-`candidate_id` (INV-1) unit-proven.

## Residual debt routed off this slice
- **SL-031**: F-2 (shared kind registry + escape-guard test) + F-5 (state dir on
  `KindRef`). The trunk-mint *wiring* itself (F-1's live half) is already SL-031
  scope (D6/§5.4).
- **IMP-010**: F-3 (test rewrite) + F-4-atomicity (reseat transactionality) —
  unrelated to the funnel; one near-term cleanup slice.
