# Implementation Plan SL-019: Backfill Doctrine product-spec corpus

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Author the full product-spec corpus for Doctrine's own capabilities, dogfooding
the SL-015 spec machinery. The work is staged so quality is locked before scale:
derive the taxonomy, prove the shape with one exemplar, correct the authoring
guidance, then fan out, then validate the whole. Requirements are `REQ-NNN`
peer entities throughout (design D-1); prose never re-stores them.

## Sequencing & Rationale

**Why five phases, in this order.** The risk is parallel drift — many agents
authoring specs that diverge in shape, or worse, all inheriting a defect (a
stale template, a prose-requirements anti-pattern). The sequence front-loads
every shared dependency before the fan-out spends tokens:

1. **Taxonomy first** because nothing can be authored until the capability set
   is agreed. It is scaffolding, not canon — kept in the runtime sheet so the
   storage rule isn't violated by committing a derived index.
2. **Exemplar second** because the template is a scaffold, not an enforced
   contract; one fully-authored PRD (Slices, the richest capability) is what
   actually fixes the bar. Its entry ritual also clears the build hazards the
   inquisition surfaced — the template embeds at compile time, and a lone edit
   does not re-embed without forcing the install crate to recompile, so the
   exemplar cannot be authored until a re-embedding rebuild is *verified* by the
   `spec show` grep gate. Authoring against a stale binary would silently brand
   the entire corpus with the old headings.
3. **Reconcile the skill third, not first.** The rework is exemplar-driven: the
   skill should describe the shape the exemplar demonstrates, so it must follow
   the lock. The web-authored draft was blind to the spec-entity code and
   actively prescribes the prose-requirements anti-pattern; it must be corrected
   *before* fan-out so agents read true guidance. Hence: after the exemplar,
   before the backfill. (Only the template — not the skill — is embedded by
   `spec new`, so the skill's in-flight state does not block the exemplar.)
4. **Fan-out fourth**, once both the exemplar and the skill are locked. Parallel
   authoring is safe at the entity level — each PRD is its own tree and
   `spec req add` reserves requirement ids through an atomic claim with bounded
   retries — so the only contention risk is retry exhaustion, mitigated by
   bounding fan-out width against a small corpus.
5. **Validate last** because FK integrity, clean reassembly, and taxonomy
   coverage are corpus-level properties — only assertable once every spec
   exists. The storage-rule check (no committed taxonomy artifact) lands here too.

**Boundaries between phases.** Each phase's exit is a hard gate for the next:
taxonomy accepted → exemplar; exemplar locked + embed verified → skill rework;
skill committed → fan-out; corpus complete → validate. The exemplar's `spec
validate`-clean and the skill's `grep`-clean are mechanical gates, not judgement
calls.

## Notes

- The §4-guidance-line reword (a sanctioned one-line template clarification, not
  a structural re-edit) lands at PHASE-02 entry alongside the template commit.
- The fan-out execution mechanism (Workflow harness vs serial `/execute`) is a
  `/phase-plan` decision for PHASE-04, not fixed here.
- Post-slice: harvest the rust-embed re-embed footgun as a durable memory (the
  inquisition's CHARGE II) — it bites every future template edit, not just this
  slice.
