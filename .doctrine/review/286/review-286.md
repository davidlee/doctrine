# Review RV-286 — design of SL-223

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

External hostile review of SL-223's publication-mechanism design against
ADR-019, PRD-017, SPEC-026 (especially D1/D2/D3/D6/D9), and the slice's claimed
REQ-373/374/376/379 coverage. Lines of interrogation: prove that the proposed
manifest cannot enter install projection; distinguish a real consumer seam from
an unexercised abstraction; test storage independence and read-only structure;
verify fail-closed licence admission; audit the neutral asset-source extraction
and every `Assets::get`/`Assets::iter` migration for behavioural loss; and reject
verification claims whose tests do not establish their mapped requirements.

## Synthesis

Judgement: the design is heretical and must not advance. D-C's factual premise is
false: install sweeps every embedded name except the root `manifest.toml`, so the
proposed publication manifest is projected into the client. Break this claim on
the wheel first. Next replace the stale-byte `#[test]` masquerading as a
build/validate gate with an automatic, release-relevant admission check. Then
repair the resolver contract: either one honestly injected adapter or a
source-keyed registry, with typed confined backing references. Keep REQ-373 and
REQ-381 pending unless their runtime-manifest and structural-read-only criteria
are actually built. Finally, prove byte emission through a real command-free
consumer and reconcile the provenance field spelling.

Standing risks after sentencing: changing projection to special-case the new
manifest expands Contract A and requires its own regression proof; choosing a new
embed root incurs the flake graft and artifact smoke gate the design sought to
avoid. Neither cost licences the current falsehood. No taint is tolerated.

> **HERESIS URITOR; DOCTRINA MANET**
