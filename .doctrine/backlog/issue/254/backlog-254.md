# ISS-254: Completeness gate has no exemption for an evidence-only phase, and its refusal cannot name which input disagreed

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

At SL-228's close, `dispatch sync --prepare-review` refused:

```
prepare-review: conformance registry incomplete: completed phase PHASE-07 has no
recorded source-delta row; record-delta the missing phase(s) before audit
```

PHASE-07 was the OQ-5 benchmark: an **evidence-only** phase, deliberately not
funnel-driven. Its deliverables are authored `.doctrine/` artefacts (`benchmark.md`,
`oq6-retirement.md`, `verdicts.tsv`, `evidence/**`) and its verification is VA-1 +
VH-1 — it has no VT criteria and lands no source. The drive's own handover recorded
"PHASE-07 lands no source delta, so there is nothing to `record-delta` for it",
which was true about the work and false about the gate.

## Two defects

**1. No exemption for a phase whose delta is not source.** `registry_completeness`
requires a registry row for every `completed` phase, with no notion of a phase that
legitimately has none. The only way past it is a synthetic row. SL-228 used
`slice record-delta 228 PHASE-07 --start 0da787d98 --end a54c05566`, which stamps
`Provenance::Manual` — filtered out of guard (3)'s `Funnel | Unknown` check
(`src/dispatch.rs:3409-3424`), so at least it does not falsely claim funnel
ownership. But it is a row asserted to satisfy a gate rather than to record a fact,
and it pollutes conformance's undeclared cell with the phase's `.doctrine/`
artefacts (see IMP-292 defect 1).

Note the projection remains consistent — phase cuts are planned from the
**committed** ledger, where PHASE-07 correctly has no row — so 8 cuts were made for
9 phases. The registry and the ledger disagree by design here, and only the registry
is gated.

**2. The refusal cannot name which of its three inputs disagreed.** The gate reads
primary runtime phase sheets + the primary registry + the committed ledger on the
dispatch ref — three tiers across two trees. Its message names only the symptom
phase. At this close it fired twice for two entirely different causes:

- PHASE-08/09: *"recorded row for PHASE-NN, which is not a completed phase"* — cause
  was a missing primary-tree **sheet** (see IDE-028 refinement), nothing to do with
  record-delta.
- PHASE-07: *"has no recorded source-delta row"* — cause was the missing exemption
  above.

Both refusals prescribe `record-delta`; for the first that advice is simply wrong,
and the operator had to read `src/state.rs` to tell the cases apart. Same family as
ISS-241, and a further instance of the D10 / FR-009 counter-example set that SL-228's
own benchmark documented (RV-312 F-5, verification vehicle IMP-321).

## Acceptance sketch

A phase with no source delta either declares that fact (and the gate honours it) or
is exempt by construction; and every completeness refusal names the tier and tree
whose state it found wanting, so the remedy it prescribes is the remedy that works.
