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
fresh-as-of: 2026-11-08 · design reviewing (run dr-01a1b821, rev 44) · head 53dcb30dd

### Produced

- SL-287 scoped; research.md (gitignored, .doctrine/slice/287/research/).
- DEC-415..DEC-420 (accepted; shape SL-287).
- design.md drafted + materialised (9 sections); design-target selectors recorded and re-attested.
- RV-429: F-1..F-3 (user) and F-4..F-6 (Codex) all verified/answered; sec-1..9 attested by the user.
- slice-287.md scope reconciled (items 5, 6); SL-287 fulfils ISS-321 partially.
- IMP-525, IDE-070, IMP-527; RFC-029 § 9 trial note; SL-258 OQ-1 note.
- mem_01a1b85718a47673840ba400f865846c (top-level acceptance records design-accepted).

### Learned

- quotient pi forms need re-approval after the CHR-184 flake move (observation recorded).
- Adding an inquiry after graph-reviewed voids it (IMP-526 in flight).
- design apply round-trip of a section body grows a trailing blank line (observation recorded).
- A top-level `acceptance` on an attestation-only submission recorded an unintended design-accepted at rev 43 (observation + memory). Still current at rev 44; the user has been told.

### Open

- Design LOCKED at run rev 47 (user: "conclude, accept"); RV-429 conducted; the rev-43 act was invalidated by the genuine acceptance. F-1/F-3 now route `review`.
- RV-429 F-6 answered, awaiting Codex's verify (minor; did not hold the lock).
- Next: /plan.
