# Notes SL-194: Actionability interestingness findings

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Close-out harvest (audit RV-240)

**Both phases landed via /dispatch funnel (claude arm).** PHASE-01 core catalogue
(S=3d338655), PHASE-02 β-family (S=d30ed33a). Conformance exact (0 undeclared/
undelivered), gate green, 9/9 VTs pass. VH-1 verdict **useful** on both the core
and the full catalogue.

**Durable outcomes lifted from the disposable phase sheets:**
- **ε seeds held (R3 closed).** PLATEAU_EPS=0.01, INVERSION_MIN_GAP=0.5,
  DISPLACEMENT_MIN_DELTA=3 were seeded guesses to calibrate from the PHASE-01 live
  run; the live volumes (displacements 96, plateaus 30) were sane — no retune. The
  calibration instrument (D5) confirmed the seeds; no follow-up.
- **Purity boundary held.** All disk (one scan, config load, β-endpoint pre-build)
  lives in the surface.rs shell; findings.rs stayed pure/graph-only. `detect` takes
  `Option<&BetaEndpoints>` — no rebuild closure injected (as design mandated).
- **The composite signal is the probe's proof.** IMP-085 fires Fork *and*
  ArmResequencing on the same hub — gates 4 sunk arms whose relative order is
  β-contested. That composition is what a flat list structurally cannot express.
- **Starvation is expected, not broken (R1).** joins / gating / value-inversion /
  provenance silent on the live corpus (thin ADR-017 records + sparse estimates);
  proven by unit fixtures. Activates as RFC-007 workstream-3 data grows.

**Deferred follow-ons (captured, not in-scope):** IMP-247 (order-instability
magnitude/score-gap threshold — the 62-line volume), IMP-248 (rendering
fold-into-survey / arc-strip). Both originates_from SL-194.

**Two worker interpretations ratified at VH-1** (design.md PHASE-02 verdict + F-1):
arm-order-among-arms basis (arms need the hub → absent from actionable frontier);
non-payload `moved:usize` sourcing magnitude (catalogue-mandated semantics, payloads
unchanged).
