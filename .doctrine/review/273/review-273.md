# Review RV-273 — design of SL-219

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

- Cross-section consistency of D1/D2/D6 across §1 operative-scalar semantics, §2 feed/ladder, §4 probe postconditions, and §6 verification.
- Soundness of claimed reuse from SL-213/SL-217, especially where estimate-specific exclusions or fallbacks could void the stated probe/calibration contract.
