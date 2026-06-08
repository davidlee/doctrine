# IMP-001: RV review-ledger kind and review verb family

Implements ADR-007. Adds the `review` kind (`RV-NNN`) and `review` verb family
for turn-based adversarial review across the lifecycle (scope, design, plan,
phase-plan, implementation/code-review, drift, reconciliation).

Scope:
- New authored kind `RV` (install wiring per `mem.pattern.install.authored-entity-wiring`):
  `[review]` (facet, raiser, responder), `[target]` (`ref` + optional `phase`),
  append-only `[[finding]]` rows.
- Outbound relation `RV-NNN ──reviews──▶ ref` (ADR-004).
- Runtime baton in state tier (D-C1); regenerable cache (D-C2).
- CLI verbs: `raise`/`dispose`/`verify`/`contest`/`withdraw`/`status` with
  closed status enum, role guard (`--as`), authored-first-then-baton ordering
  (D-C3/D-C4) and per-review lock/CAS (D-C4a).
- Derived status function (D-C8): empty→active, else derived role, all-terminal→done.
- Prose companion (D-C6): `## Brief` + optional `## Synthesis`.
- Lifecycle teeth (D-C9): review-done = all findings terminal; `/close` refuses
  unresolved `blocker` findings.

Highest product value; load-bearing capability. Can dogfood its own design review
(ADR-007 was itself reviewed by hand). Largest of the four — likely multi-phase.
Governing: ADR-007.
