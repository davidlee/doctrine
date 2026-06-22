# Review RV-139 — design of SL-137

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

External adversarial pass via codex mcp (GPT-5.5) over `.doctrine/slice/137/design.md`,
following two internal passes (the `--include-memory` collapse F1→D2 and the D6
canonical-normalisation pass). The design is pure-consumption — two read verbs
(`relation list`, `relation census`) over the already-hydrated `Catalog` edge set.

Lines of interrogation:

1. **The load-bearing hydration invariant (A6 / D2).** The `--include-memory`
   collapse and "memory only = `--include-memory --source-kind MEM`" both rest on
   `Validated ⟺ numbered source` ∧ `Raw ⟺ memory source` holding for *every* edge.
   Is this enforced, or merely asserted from memory? A single counter-example
   (a `Validated` label on a memory source, a `Raw` label on a numbered source)
   silently breaks the provenance filter.
2. **`--target` matching against `target_display` (D6).** Free-text targets that
   parse as canonical refs, or a resolved target whose canonical form shadows a
   distinct free-text string. Is the match well-defined and unambiguous?
3. **Error-only stderr (A10 / F5).** Does suppressing Warning/Info hide a real
   partial-corpus signal — a user reading a silently truncated view as complete?
4. **VT sufficiency (A11).** Does VT-1..12 actually pin the four-axis AND
   composition, the `--source-kind MEM` without `--include-memory` ⇒ empty case,
   and the sort determinism, against regression?
5. **Layering / ADR-004 / scope discipline.** Cycle-freedom, outbound-only target
   filtering, and the non-goal boundary (transitive SL-138, export, write).

## Synthesis

**Judgement: heresy found — and burned. The design now tells the truth.** Four
charges sustained, none a blocker; all reconciled into `design.md` (this slice is
pre-implementation, so the artifact *is* the defect — disposition `design-wrong`).

The heaviest taint was in the **diagnostics rationale (X1/F-1)**: the design
boasted that Error-only stderr "diverges from `inspect`, which prints every
diagnostic" — a phantom. `inspect` rides `relation_graph::scan_entities`, an
Error-only scan path; it has no Warning/Info flood to diverge *from*. Worse, the
design claimed suppressed Warning/Info were "surfaced by `--unresolved`/census" —
true for classification diagnostics (the edge survives), false for **edge-dropping**
Warnings (empty memory rows `continue` before the edge exists; `hydrate.rs:289,299`).
A query that silently sheds edges while swearing completeness is a lie told to the
operator. Reconciled: the rationale now states true parity and the shell emits a
bounded dropped-edge summary line.

**X2/F-2** exposed a second, stricter completeness channel the design never named:
illegal hand-edited `[[relation]]` rows are dropped by `tier1_edges` with **no
diagnostic at all** — `validate` re-reads raw TOML precisely because the scan drops
them (`relation_graph.rs:373-374`). Cured by scope, not scope-creep: the verb is now
declared a *validated-live-edge* query; illegal rows remain `doctrine validate`'s
province. **X3/F-3** (memory `--target` matches by UID, not alias) and **X4/F-4**
(three overclaimed VTs + an unspecified sort tie-breaker) were lesser taints,
likewise pinned.

**Counts cleared under cross-examination** (the accused walks on these): the
load-bearing **D2 hydration invariant holds** — `Validated` labels are constructed
only in the numbered-source loop, `Raw` only in the memory loop; there is no mixed
constructor path, so the `--include-memory` collapse is sound. The free-text/
canonical-ref collision cannot arise (`classify_target` never routes a parsable ref
to `UnvalidatedText`). ADR-001 layering and ADR-004 outbound-only stand.

**Standing risks / tolerated taint:** none unresolved. The dropped-edge summary
(X1) and the validated-live-edge scope (X2) are design commitments the implementation
must honour — VT-11 is their verification anchor; a future implementer who omits the
summary line reopens X1. No blocker gates the close.

**Harvest:** nothing durable beyond the artifact — the findings are design-local and
now resident in `design.md` §5.4/§5.5/§9/§10. No new memory, no backlog spawn.

> **HERESIS URITOR; DOCTRINA MANET**
