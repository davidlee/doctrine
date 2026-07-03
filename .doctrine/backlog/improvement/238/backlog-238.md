# IMP-238: Reconcile SPEC-023 inquisition findings

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

RV-237 arraigned SPEC-023 as the durable hardening of RFC-013 and raised three
spec-correction findings:

- RV-237 F-1: dispatch delivery is named in prose but not represented as
  structured interactions to SPEC-012/SPEC-021.
- RV-237 F-2: SPEC-023 owns corpus loading/seal behaviour but its structured
  source anchors omit `src/install.rs`, where the loader and seal accessors live.
- RV-237 F-3: `prompt check` is described as feeding `doctrine check`, but the
  delivered-vs-target boundary does not mark that cadence integration clearly.

## Acceptance

- SPEC-023 structured interactions and prose agree on dispatch/boot/install
  consumers.
- SPEC-023 source anchors or interaction text make corpus-loader ownership
  unambiguous.
- The `prompt check` to `doctrine check` obligation is either wired and covered,
  or explicitly marked as target/follow-up rather than delivered fact.
