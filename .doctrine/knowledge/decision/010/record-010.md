# DEC-010: Published set = full projection complement

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

**Decision (SL-227 design, 2026-07-24; revised after external review RV-299).**
RFC-021's pairing invariant — minimal projection must not strip access, so the
library republishes what the flip stops projecting — is satisfied by publishing
the **full projection complement** (Option A), *not* a bounded subset.

## The set

The library publishes **every embedded `install/` asset the flip stops
projecting**: `{embedded_filenames()} − {base backings}` (~70 entries — all
`templates/*`, the operator docs, `hymns/*`, `agents/*`, `workflows/*`,
`mod.just`, `LICENSE`, `boot-footer.md`, `model-band.md`, …). All are **MIT**
(everything under `install/` is MIT — `install/LICENSE`), so the licence surface
is trivial. `ContentKind` widens additively to a small closed set
(`{Template, Reference, Guidance, Integration}`).

## Why full, not bounded (the reversal)

The original decision bounded the set to *templates + 4 reference docs* on the
theory that hymns were C2-entangled, agents/workflows survived via the
harness-gated install legs, and the memory corpus rode its own surface — so only
the reference docs were "at risk." **External review RV-299 (X-F1) refuted the
completeness of that accounting:** the flip stops projecting the *entire*
`install/` embed (`install.rs:1394-1409`), and ~43 templates plus `mod.just`,
`LICENSE`, `boot-footer.md`, `model-band.md` fell into neither bucket — a real
reachability hole against the slice's own no-loss objective.

Publishing the full complement dissolves the hole *and* the judgement call: the
no-silent-unreachable gate becomes a **derived** set-containment
(`delta ⊆ published`, design D7/§9) that a future added asset cannot silently
escape. The rejected alternative (classify each dropped asset into
reachable-elsewhere buckets + an allowlist) mints a second governed surface and a
curated check the gate cannot mechanically enforce.

## QUE-172

Answered **no** — the memory corpus is not published-for-copy this slice. It is
not part of leg-2 projection, so the flip does not strip it; the eager install
seed (`seed_authoring_memories`) is **gated** (design D8), not published.
