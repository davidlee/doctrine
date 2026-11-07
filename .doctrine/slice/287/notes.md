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
fresh-as-of: 2026-11-08 · ready, phases 0/2 (design locked rev 47) · head 939bf20a0

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

### Learned

- quotient pi forms need re-approval after the CHR-184 flake move (observation recorded).
- Adding an inquiry after graph-reviewed voids it (IMP-526 in flight).
- design apply round-trip of a section body grows a trailing blank line (observation recorded).
- A top-level `acceptance` on an attestation-only submission recorded an unintended design-accepted at rev 43 (observation + memory); invalidated by the genuine acceptance at lock.
- Dogfood evidence for RFC-029 § 9 (observations recorded): cross-check caught a cited-but-wrong count before commit; reviewers caught lead-in-only quotes (RV-431 F-1) and newline-tainted quotes (F-5); no variance class fits a transcription correction; verify-vt says "keyword present" for absent keywords.

### Open

- RV-429 F-6 answered, awaiting Codex's verify (minor; did not hold the lock).
- design.md sec-3 line 139 overlong (edit artefact) — tidy at reconcile.
- Next: execution of PHASE-01..02 via /super-orchestrator (handover.md); halts before audit.
