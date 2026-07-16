# Review RV-275 — design of RFC-020

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

RFC-020 is arraigned as design intent for the proposed replacement of authored
value/estimate facets with ledgered absolute claims, the authority-ladder
resolver, and hierarchy-value admission. The Inquisition tests it against
RFC-019's axioms and shipped comparison model, ADR-015/ADR-017/ADR-018,
PRD-014/SPEC-020, the SL-219 `AnchorMap` seam, and Doctrine's storage,
relations, and pure/imperative rules.

Lines of interrogation:

1. Does one domain-parameterised judgement schema actually type every promised
   form and payload without ambiguity, especially tombstones and estimate
   ranges?
2. Is the authority ladder deterministic under conflicting, concurrent,
   lens-scoped, superseded, and abstaining claims, or are policy gates deferred
   behind implementation phases that claim to ship independently?
3. Does migration preserve provenance honestly and idempotently, including
   human-vs-agent uncertainty, sequence/date assignment, rollback, and mixed
   old/new readers?
4. Can diagnostic-only hierarchy coherence be computed without smuggling in an
   aggregation rule the RFC explicitly leaves open?
5. Are capture, resolution, rendering, retention, governance revisions, and
   spec-routing obligations complete enough to prevent parallel ontologies and
   silent behavioural drift?
6. Do the stated phase boundaries remain independently shippable under the
   declared critical-path and open-question dependencies?

`doctrine review prime RV-275` was attempted and correctly refused because
RFC-020 is not a slice target and therefore has no selector cache. The court
proceeds from the RFC's explicit relation set and CLI-rendered authorities.

## Synthesis

### Judgement

RFC-020 contains a worthy ontological correction—absolute magnitudes are
judgements, not timeless entity attributes—but the present design is tainted
and must not yet sire implementation. Two blocker heresies strike its claimed
shipping path: the authority ladder cannot deterministically reduce concurrent
same-tier or lens-scoped claims to one `AnchorMap` entry (F-1), and the hierarchy
diagnostic sums member values immediately after declaring that sum unsound
without aggregation semantics (F-2). Three major defects compound the taint:
migration invents agent authorship and judgement dates (F-3), confidence is
simultaneously declared derived-only and retained as authored estimate payload
(F-4), and the free `authority` field cannot substantiate an operator-only pin
(F-5).

### Sentencing

1. Before Phase 1, define the complete active-claim algebra: row identity,
   authority vocabulary, provenance invariants, supersession, same-tier
   concurrency, lens participation, pin contest/demotion, and deterministic
   conflict handling. Prove it with permutation, duplicate-merge,
   cross-session, conflicting-pin, and lens-isolation tests.
2. Remove additive cross-level diagnostics from v1. If cross-level arithmetic
   is desired, first settle aggregation modes and the ADR-018 revision, then
   test package-valued, portfolio-valued, partial-fulfilment, overlap, and
   multi-membership counterexamples.
3. Give migrated facets an explicit `migrated`/`unknown` provenance class and
   separate conversion time from judgement time. Verify import idempotence,
   full corpus census, and lossless rollback/inspection.
4. Distinguish value-fit certainty (derived feasible bounds) from estimate
   confidence (legacy payload), and state whether the latter constrains
   inference or merely annotates it.
5. Bind `pin` to an explicit operator admission path, derive ordinary authority
   from mandatory provenance, reject contradictory rows, and express demotion
   through append-only supersession.

### Standing risk and harvest

Until those sentences are served, Phase 1 and Phase 3 are not independently
shippable despite the RFC's claim. No taint is tolerated. No separate backlog
item or memory is harvested: every finding is proximate design work on RFC-020
and is durably owned by this ledger. The prime-cache refusal is expected for a
non-slice target and does not diminish the verdict.

Let the undefined winner, the counterfeit provenance, and the illicit sum be
burned at the stake before they breed implementation. *Fiat doctrina, pereat
ambiguitas.*

> **HERESIS URITOR; DOCTRINA MANET**
