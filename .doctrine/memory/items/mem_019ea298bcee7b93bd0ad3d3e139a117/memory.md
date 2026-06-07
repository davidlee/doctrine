# Doctrine canonical change loop (ADR-003)

**Pointer, not canon.** Full doctrine in `.doctrine/adr/003/adr-003.md`
(status: **proposed** as of authoring — verify acceptance before treating as
ratified). Doctrine's workflow is a re-implementation of `spec-driver`'s model;
the gaps below are *age*, not divergent intent.

## Lineage
- A doctrine **slice IS `spec-driver`'s delta** — same model, different
  vocabulary (`doc/slices-spec.md` § Overview).
- `design.md` is the slice's in-bundle design record (≈ `spec-driver` DR).
  Authoring it up front is NOT spec-first-aspirational editing — per-slice design
  (the HOW) and evergreen specs (normative truth) are different layers.

## The loop
```
slice → design → plan → phases
  → per phase: phase-plan → implement (TDD red/green/refactor) → complete → [review]
  → audit → reconcile → close
```

## The seams (what each step owns)
- **review** (per-phase, optional/partial): gates truly met + quality + backlog
  current. Sequential OR non-blocking (run while next phase begins). Lightens the
  final audit; does NOT reconcile specs, does NOT replace audit.
- **audit** (whole-slice): close critical read of the impl surface; correctness/
  quality; drift ID + remediation; backlog propagation; gap triage. **Identifies**
  the spec changes + preps reconciliation context — but does NOT write specs.
- **reconcile** (new step): **writes** the spec changes against observed truth.
  The point specs regain authority. Distinct skill from audit.
- **close**: a GATE — no terminal status while owning specs are drifted.
  "Implemented now, fix specs later" = drift, not closure.

## Truth model
Specs own **normative** truth; audit findings (and future contracts) are
**observed** truth. Disagreement = drift, reconciled **explicitly**, never by
timestamp/precedence. Memory staleness is a *surfacing* signal, not an authority
override.

## Deferred machinery (age, not intent)
contracts (efficiency not concept; language-limited) · tech specs (kind exists,
none authored) · the reconcile artefact (≈ revision; name TBD) · `/review` +
`/reconcile` skills (unbuilt; review is manual via `/code-review`) · `/dispatch`
(parallel/worktree/batched phase exec — **placeholder skill exists**, routes to
serial `/execute`; harness/vendor-specific) · per-requirement lifecycle/coverage
states · closure-gate enforcement.

See also [[mem.system.spec.composition-seam]] (the product/requirement spec
entities reconcile targets).
