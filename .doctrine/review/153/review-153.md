# Review RV-153 — design of SL-150

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of attack against SL-150 design (facet: design):
- FAMILIES taxonomy + drift-test partition logic — does it truly guard against
  orphan/phantom/duplicate?
- Auto-derive boot-map rule (verbs − SPINE) — is SPINE itself guarded?
- Plain-text grouped help (D8) + CommandMap boot wiring/ordering.
- ADR-005 PUSH/PULL: is the map's boot seat EARNED or merely asserted?
- Storage rule: do the authored tiers (slice scope vs design) tell one truth?
- Behaviour-preservation: are existing boot fixtures accounted for?

## Synthesis

**Judgement: the design is sound — no blocker heresy.** The accused confessed
five taints, all venial (2 major, 3 minor), all reconciled in the same breath as
the design artifact; none gates planning.

The two **major** charges were reasoning gaps, not structural rot:
- **F-1** — the design claimed a PUSH-tier boot seat for the command map but did
  not defend it against ADR-005's compactness creed. Penance: §3 now argues the
  seat is *earned* — the command surface is the Routing table's navigational
  companion, and the factored spine + infra suppression are exactly what keep it
  cheap enough to ride PUSH (the uncompressed ~150-line `--commands` correctly
  stays PULL; the unbounded memory corpus stays PULL signposts).
- **F-2** — the blast radius on existing boot fixtures was unstated. Penance:
  §9 enumerates the `src/boot.rs` tests and shows they survive *by construction*
  (the ADR→Policy→Standard adjacency holds because CommandMap lands after
  "Routing & Process", before Governance); no unit test embeds the full snapshot
  byte-for-byte. The fear was real; the danger was not.

The three **minor** charges hardened the design: F-3 struck an authored-tier
contradiction (slice Context spine vs design SPINE); F-4 folded the
stringly-typed `SUPPRESS_VERBS` into a compile-linked `Family.suppress_verbs`
field; F-5 named the boot-map golden as the explicit SPINE guard.

**Standing risks (tolerated, eyes open):** SPINE validity rests on the boot-map
golden catching drift rather than a dedicated assertion — accepted, with the
≥2-kinds assertion offered if belt-and-braces is later wanted. The "~20 line"
boot-map budget is an estimate until the golden lands (execute phase 1).

**Penance verified.** All five findings reconciled into design.md / slice-150.md
and verified terminal. The design may pass to planning. No durable memory
harvest — the findings live with the slice; no cross-cutting gotcha emerged.

> **HERESIS URITOR; DOCTRINA MANET**
