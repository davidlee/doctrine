# SL-287 notes

## Further review passes

Written 2026-11-08, after RV-429's Codex pass (F-4..F-6) concluded and all
six findings were verified. No further design pass is needed: the remaining
uncertainty is empirical, not design-level. A further pass would probe only
(a) whether the refresh procedure is too heavy to follow by hand in practice
and (b) whether two plan-review RVs are worth their cost — both are what the
RFC-029 § 9 trial measures on a real plan. Re-review only if the trial
contradicts the design.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-11-08 · close, phases 2/2 · head 34026c5a7

### Produced

- SL-287 scoped; research.md (gitignored, .doctrine/slice/287/research/).
- DEC-415..DEC-420 (accepted; shape SL-287).
- design.md drafted + materialised (9 sections); design-target selectors recorded and re-attested.
- RV-429: F-1..F-3 (user) and F-4..F-6 (Codex) all verified/answered; sec-1..9 attested by the user.
- slice-287.md scope reconciled (items 5, 6); SL-287 fulfils ISS-321 partially.
- IMP-525, IDE-070, IMP-527; RFC-029 § 9 trial note; SL-258 OQ-1 note.
- mem_01a1b85718a47673840ba400f865846c (top-level acceptance records design-accepted).
- plan.toml (PHASE-01 doc+manifest+ship test; PHASE-02 pointers+step-4+glossary), plan.md, commitment-ledger.toml (dogfooded: 40 rows, 37 governing, C-16/18/21 superseded).
- RV-431 (commitments) and RV-432 (sequencing) plan reviews — all findings verified. Plan accepted on user delegation; status ready.
- PHASE-01 ae3013d83, PHASE-02 f58fb2d41 (/super-orchestrator); RV-433 audit (5 findings, all verified; no REV); RV-429 F-6 verified at audit.
- Reconcile 34026c5a7: design.md sec-3 F-3 wording, F-4 + coverage-bullet rewrap; outcome on RV-433. No REV.

### Learned

- quotient pi forms need re-approval after the CHR-184 flake move (observation recorded).
- Adding an inquiry after graph-reviewed voids it (IMP-526 in flight).
- design apply round-trip of a section body grows a trailing blank line (observation recorded).
- A top-level `acceptance` on an attestation-only submission recorded an unintended design-accepted at rev 43 (observation + memory); invalidated by the genuine acceptance at lock.
- The execution seat committed phases without `slice record-delta`; conformance read empty until audit bootstrapped it (RV-433 F-1).
- Dogfood evidence for RFC-029 § 9 (observations recorded): cross-check caught a cited-but-wrong count before commit; reviewers caught lead-in-only quotes (RV-431 F-1) and newline-tainted quotes (F-5); no variance class fits a transcription correction; verify-vt says "keyword present" for absent keywords.

### Open

- ISS-321 stays open: SL-287 fulfils it only partially.
- Live-plan trial deferred (RFC-029 § 9); not a closure condition.
- Harvest at close: no new memory. Each Learned item already sits in an observation, a memory, or a backlog item (IMP-526); the rest are consciously rejected as slice-local.

## Implementation run

Seat: /super-orchestrator, 2026-11-08, PHASE-01..02, main worktree on `edge`.

- quotient forms probed at the seat: `read` (tools present, root read-only,
  network ok) and `write` (fmt-check, clippy, lint-js, build, validate,
  `cargo test -p doctrine -p cordage`, test-bridge all exit 0). No form change.
- Known sandbox-only failures: none (host and write-form suites both fully green).
- Host gate: `doctrine check gate` (builds before validate). Use
  `./target/debug/doctrine` for `install` / `library show` (PATH binary is the
  release and lacks the new manifest entry).
- Operator inputs: none.
- Other writers: leave unrelated dirty paths alone (src/design_run/*, IMP-526
  work, other reviews). Commit path-limited only.
- Background tasks are enabled: spawns run in the background; parent is woken
  by notification.

## Autonomy log
<!-- phase · what · why · tier (decided / deferred / escalated) -->
- PHASE-01 · VA-1 read-through: restored design wording in three places the worker loosened (trigger "reads only the plan's own content"; "host directives (project or harness conventions)" twice, which the worker had narrowed to "project conventions"; a garbled fifth stale condition) · fidelity to C-12/C-14/C-38/C-20 quotes · decided
- PHASE-01 · doc example uses placeholder ids (`C-NN`, `PHASE-NN/EX-n`) not the design's concrete ones · `[A-Z]{2,4}-[0-9]` matches inside `PHASE-0x`, so concrete phase ids fail the shipped-text sweep · decided
- PHASE-02 · `./target/debug/doctrine install -y` also bumped `.doctrine/doctrine.toml` `installed-version` 0.48.1→0.48.3; committed separately as a chore, as install advises · side effect of EX-4's required install · decided
- PHASE-02 · pre-existing "obligation" in `/plan` step 3 (routed-findings paragraph) left as is · this phase's text only; not in EX-2's scope · deferred to audit
