# Plan: CSS modularisation (SL-093)

## Rationale

Three phases, sequenced for safety: CSS-only changes first, then
behaviour-touching TS/HTML changes, then verification and closure.

**PHASE-01 is pure CSS** — no behaviour change, no TS touched. The original
`style.css` is deleted only after all 10 replacement files exist and pass a
Vite build. This phase can be reverted trivially (`git checkout -- style.css`)
until the build gate passes.

**PHASE-02 touches TS and HTML** — class renames and `style.display` removal.
This is the highest-risk phase because `classList` string mismatches won't
fail at build time (TypeScript can't type-check class name strings). The
visual verification in PHASE-03 is the safety net.

**PHASE-03 is verification and closure** — manual visual comparison across all
views and colour schemes, plus automated audit gates. This phase is human-gated
(VH-1) because CSS visual regression testing has no automated harness in this
project.

## Phase sequencing

```
PHASE-01 (CSS only)
  → PHASE-02 (TS/HTML, depends on new CSS files existing)
    → PHASE-03 (visual verification, depends on everything being wired up)
```

No parallelism possible — each phase depends on the prior.
