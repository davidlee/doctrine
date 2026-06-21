# Implementation Plan SL-136: Extend tagging to all entity types — generic cross-kind tag verb

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, strictly **inside-out**: build the shared write core first, expose
it through the verb, wire the read surfaces, then migrate governance storage onto
those surfaces. Each phase leaves the tree green and is independently reviewable;
the order is a dependency chain, not a convenience.

The spine is design D1's uniformity bet — one root `tags` location, one write
leaf, one filter-fold — so the work is *generalise an existing seam*, never a
parallel implementation. PHASE-01 carries the only behaviour-preservation risk
(backlog already tags through the path being hoisted); PHASE-04 carries the only
governance-changing risk (the typed→root storage move that the D6 Revision
ratifies).

## Sequencing & Rationale

- **PHASE-01 (Shared write leaf) first** because everything downstream calls it.
  `apply_tags_set` + `fold_filter_tag` are *hoists* of logic backlog already runs,
  so the existing backlog tag suite is the behaviour-preservation proof — it must
  stay green with backlog reduced to a pure delegate. The D4 root-insert decision
  is already settled (RV-129 F-1): the committed spike CHR-019 proved a root insert
  lands above all trailing subtables (structural to TOML), so VT-2 is satisfied by
  that spike rather than a probe-to-write, and the stale F-1 corruption comments in
  both backlog::apply_tags and dep_seq::apply_status are corrected. No verb, no read
  changes yet: this phase is provably behaviour-neutral or it is wrong.

- **PHASE-02 (Generic verb) second** — once the core is trustworthy, expose it.
  The verb is a thin command-tier shell over the leaf: resolve, guard, gate on
  TAGGABLE, delegate. The membership gate is the load-bearing decision (D2) — an
  excluded kind must refuse with an IMP-144 pointer rather than write metadata no
  read surface renders. Writing precedes reading deliberately: the verb can be
  exercised against backlog/knowledge (already-rendered kinds) before the new
  surfaces exist.

- **PHASE-03 (Read-surface parity) third** — make the newly-taggable kinds
  actually *show* their tags. This is the phase the three Codex passes reshaped:
  full parity means list-filter **and** show **and** json — partial wiring is the
  write-only smell D2 killed, only quieter. Scope here is the **Meta/spec-mediated
  kinds** (slice, spec, REQ); breadth, not depth: one `key()` line, one show row,
  one json field per kind. Governance/RFC are deliberately **not** wired here —
  there is no root `tags` on a gov entity to read until the storage move, so their
  parity is built atomically inside PHASE-04 (RV-129 F-2). REQ's two JSON sites are
  *additive* (they drop tags today), so PHASE-03 asserts tag **presence** (F-5).

- **PHASE-04 (Governance/RFC migration + parity) last** because it is the only
  irreversible, canon-touching step and it depends on every prior phase: the verb
  (to restore RFC-002's tags), the leaf (to write them), plus the struct move and
  the gov read path it builds in-phase. The migration and the gov/RFC three-surface
  parity land together: drop typed `tags` from `Relationships`, add root `tags` to
  `Doc`, repoint the `show` table render, wire `governance::key()` for `list --tag`
  — and `--json` falls out of the serde `to_value(doc)` for free (root-expose, no
  builder change). The storage move contradicts SPEC-005/016/018, so the corpus
  lands non-canonical **by design** until the D6 Revision amends the specs at
  `/reconcile`. VA-1 is the tripwire (kept soft by D7): closure cannot silently
  skip the REV.

## Notes

- **Verification modes.** All exit criteria are test-verifiable (VT) except the
  D6 REV obligation (PHASE-04 VA-1) — a Revision is a `/reconcile`-time governance
  act, not a unit assertion, so it is carried as an agent-checked obligation
  rather than silently dropped.
- **Phase boundaries mirror design §9 phasing** verbatim; the criteria are drawn
  from design §9's quality-validation list and §5.5 invariants. The plan adds no
  scope beyond the design — it sequences it.
- **`just gate` green before every commit**; PHASE-01 and PHASE-03 additionally
  assert the prior suites stay green unchanged (the behaviour-preservation gate).
