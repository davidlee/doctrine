# DEC-010: Bounded reachability set for the minimal-projection flip

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

**Decision (SL-227 design, 2026-07-24).** RFC-021's pairing invariant — minimal
projection must not strip access, so the library republishes what the flip stops
projecting — is satisfied by a **bounded** published set, not full corpus parity.

## The set

The library's reachability-parity obligation for SL-227 is **templates (already
published, SL-223) + the reference docs** (`glossary.md`, `using-doctrine.md`,
`review-ledger.md`, `governance.md`). Publishing the reference docs needs one
**additive** `ContentKind::Reference` (publication.rs anticipates additive
widening) plus a per-entry licence call.

## What is NOT pulled in, and why it is not "silently unreachable"

- **Hymns** — C2-entangled (the supported-customization model owns their copy
  semantics); deferred on the same reasoning as [[ASM-003]]. Behaviour prose,
  not user-facing reference.
- **Agents / workflows** — harness-adapter machinery, still installed on demand
  per detected harness (NF-004, `install.rs` `detect_agents`→per-agent loop).
  Not lost by the flip.
- **Memory corpus** — never rode leg-2 projection; already reachable via its
  native `memory find` / `retrieve` / `sync` surface. Answers [[QUE-172]] **no**
  (not published-for-copy this slice).

## Consequence

Keeps SL-227 whole and tractable as one slice (the user's "B unless
intractable"): the licence-classification surface is bounded to ~4 reference
docs, and QUE-172 does not gate the manifest beyond that. The published set may
widen additively in a later slice without reworking this one.
