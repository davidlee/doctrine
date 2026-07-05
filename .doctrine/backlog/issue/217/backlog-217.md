# ISS-217: SL-205 surface log: reconcile §5.3 log-on-suppression vs §5.4/F-3 delivery-gate

## Contradiction

The locked SL-205 design contradicts itself on when the tuning log
(`mem-surface.log`) is written:

- **design §5.3** — "Logged **whenever retrieve ran** (any main-thread fire),
  including when the emit is nothing — the suppression cases are exactly what
  retunes the floor." I.e. the `fetched → admitted → suppressed_sev → surfaced`
  funnel line is meant to fire even on an empty/all-suppressed emit, because
  those rows are the retuning signal.
- **design §5.4 + plan EX-4** — "seen-set **and** log append happen **only after
  a successful non-empty emit**" (the RV-254 **F-3** penance / INV-6).

## What shipped (PHASE-03, `9e0afeef`)

The worker followed the explicit, VT-gated F-3/EX-4 contract: `run_surface`
appends **both** the seen-set and the log line only inside the delivered branch
(`if emit_surface(...) { … }`). Consequence: a fire that fetches rows but
surfaces nothing (all sub-floor / all-deduped / empty) writes **no** log line —
the §5.3 retuning signal for suppression is lost.

This is **not** a fail-open or dedup-integrity defect:
- INV-2 (fail-open, exit 0 on every path) — intact.
- INV-6 (a uid recorded seen only after delivery) — intact and REQUIRES the
  seen-set stay delivery-gated.
All 9 PHASE-03 VTs are green; no VT covers log-on-suppression.

## Resolution options

1. **Split the two gates.** INV-6 pins only the **seen-set** to the delivered
   branch. The **log** can fire whenever retrieve ran (§5.3 intent) without
   touching dedup integrity — they are independent artifacts. This restores the
   retuning signal. Small change to `run_surface` ordering + a new VT
   ("suppressed fire still logs, seen-set still untouched").
2. **Amend the design.** If the suppression rows are not actually wanted, delete
   the §5.3 "including when the emit is nothing" clause so the design is
   self-consistent with §5.4/F-3.

Recommendation: option 1 — the seen-set/log coupling in §5.4 was over-broad;
F-3 only ever needed to gate the **seen-set**. Sequence behind the SL-205 audit
(don't reopen a green phase mid-drive).

Accepted as-built for SL-205 per orchestrator/user decision (2026-07-06).
Origin: PHASE-03 dispatch worker flag.
