# IMP-205: Spec consistency gate — detect contradictory REQ pairs

**Source:** SL-165 PIR §2.2, S3 (MEDIUM). **Home:** standalone (governance/spec quality — no current RFC).

REQ-316 ("no non-journaled source") ⊥ REQ-317 ("source must be candidate ref")
were both `active` + settled while mutually contradictory — latent since SPEC-022,
surfaced only when SL-165 first needed both. No spec-internal consistency check
exists.

**Fix direction:** `doctrine spec validate` / `check spec-consistency` heuristic —
parse mandatory/prohibitive modality, resolve shared nouns, flag forbids∧mandates
conflicts; refuse `settled` while contradictory. Keyword heuristic, no NLP.

Note: not dispatch-funnel and not selector — does not belong in RFC-004/005/011.
Awaiting a governance-quality RFC or direct slice.
