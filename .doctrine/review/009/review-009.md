# Review RV-009 — reconciliation of SL-051

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation reconciliation of SL-051 (retire `backlog order`; fold
ordering into `list` as a default-on comparator) against `design.md`, the §7
locked decisions (DD-1..5), the §8 accepted risks, ADR-001 layering, the
pure/imperative split, and PRD-009 / `REQ-097`.

**Prior coverage (not re-litigated, only confirmed landed):**
- `/inquisition` (inquisition.md) burned the *design execution* pre-plan —
  Charges I (third `Order` ref at main.rs:1685), II (~20 undercounted in-crate
  tests), III (A-2 kind-axis prose), IV (transposable `(String,String)` tuple).
  All were integrated into design §4.5/§5/§3/§4.3 before planning.
- A code-review pass (commits 541cfbe, 77fb18c) hardened the impl: `--all`
  terminal-tail VT (usize::MAX branch), `Ordering` → enum (illegal states gone),
  `(String,String)` → named `ListOutput` (Charge IV closed in code), stale
  `backlog_order.rs` module doc.

**Lines of attack for *this* ledger** — does the shipped code reconcile with
canon, and is every §8 accepted risk consciously dispositioned before close:
1. **Conformance** — verb retired (3 refs gone), `compose`/`Ordering`/`OrderBy`
   present, compose-then-filter data flow (DD-1), cycle-degrade exit-0 (DD-3),
   JSON composed-order with envelope unchanged (DD-4), `--by id` opt-out (DD-5).
2. **A-2 membership invariant** — proven by VT3 (set-equality of `list` vs
   `--by id`); off-sequence tail by `usize::MAX` proven by VT9.
3. **Governance** — PRD-009 / `REQ-097` bind the *capability*, not the verb name:
   does retiring `order` leave the requirement satisfied (R-1, reconcile note)?
4. **Accepted-risk disposition** — A-1 (graph cost per default list), the
   JSON+cycle no-in-band-signal cost, RSK-005 (adapter bimap, deferred): each
   must end explicitly aligned / tolerated / follow-up, not silently shipped.
5. **Evidence** — `just check` fully green (SL-050 red resolved by fe1185e);
   8 design VTs ↔ 11 e2e goldens; `cargo clippy` plain zero warnings.

## Synthesis

**Closure story.** SL-051 ships as designed: `backlog order` is retired (all three
live refs gone — clap variant, dispatch arm, access-classifier arm; the third was
the inquisition's Charge I), and ordering folds into `list` as a default-on pure
comparator (`OrderBy::Sequence` default | `Id` opt-out). The locked decomposition
held in code: `compose(&corpus)` builds the cordage graph over the full
non-terminal corpus, `retain` owns membership unchanged, and `sort_by_key` orders
the retained set by composed position with the `usize::MAX` off-sequence tail
(DD-1). Cycle-degrade is total — id-sort + stderr warning + exit 0, never an empty
table (DD-3). JSON carries the composed order with the envelope untouched (DD-4).
The stdout/stderr split is a named `ListOutput`, not a transposable tuple — the
code-review took Charge IV past the design's doc-comment guard into a type.

**Evidence.** `just check` fully green (the concurrent SL-050 red was resolved by
fe1185e, independent of this slice); the 8 design VTs map onto 11 e2e goldens in
`tests/e2e_backlog_list_order_golden.rs` (VT5 and VT8 each split into two cases,
plus VT9 — the `--all` terminal-tail case added by the code-review); `cargo clippy`
(plain) zero warnings. Phases 01/02 both `completed`.

**Dispositions.** Four findings, all reconciling §8's accepted risks, none gating:
- F-1 **aligned** — REQ-097/PRD-009 FR-010 bind the ordering *capability*, not the
  verb name; `list` satisfies it. Reconcile note at close, no spec amendment.
- F-2 **tolerated** — A-1 graph-cost-per-default-list is a conscious tradeoff;
  `--by id` is the zero-cost escape. Do not optimise.
- F-3 **follow-up** — RSK-005 (adapter bimap corruption) stays open, tracked in the
  backlog; the fold consolidates the `project`→`build` call site, marginally
  cheapening a future fix.
- F-4 **tolerated** — JSON+cycle has no in-band degrade signal; accepted cost of
  DD-3 ∧ DD-4.

**Standing risks after close.** RSK-005 (separate card). A-1 graph cost and the
JSON+cycle silent-degrade are accepted-by-design, not defects. No undispositioned
gap, no blocker — audit-ready for `/close`.
