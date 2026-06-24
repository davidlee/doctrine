# Shipped memory corpus overhaul: holistic onboarding & self-correction

## Context

Doctrine ships a corpus of 29 global-orientation (shipped) memories under
`.doctrine/memory/shipped/` — signposts, concepts, patterns, and facts that
orient agents via `/retrieve-memory` in any client repo. They were authored
in two batches (June 5–6 core, June 15–16 entity-kind signposts) and have not
been systematically reviewed since.

Several symptoms indicate drift:

- **Stale since June 15–17.** The codebase has evolved significantly across
  ~hundreds of commits — new entity kinds (REC, REV, POL, STD, knowledge),
  new CLI surfaces (revision, policy, standard, review), new dispatch
  mechanics. The shipped memories don't reflect these.
- **`show` fallthrough to shipped/ was broken** and was only recently fixed
  (IMP-148 Gap 8). The corpus has been effectively invisible during retrieval
  for a period, compounding the staleness.
- **No drift-detection mechanism.** There is no check at reconcile/close time
  that shipped memories remain consistent with the current codebase state.
  The corpus decays silently.
- **No systematic onboarding path.** An agent new to the corpus has no
  curated progression from overview → core concepts → workflow → reference.
  The web of wikilinks is there but not structured as a learning path.

## Scope & Objectives

### Objectives

1. **Audit & update** all 29 shipped memories for currency, correctness, and
   completeness against the current codebase (commit edge as of slice start).
2. **Restructure as a holistic onboarding path** — curated progression from
   overview → core concepts → workflow patterns → entity reference → CLI
   surface, with coherent cross-linking that lets an agent orient in minutes,
   not sessions.
3. **Ensure memory reachability** — verify every shipped memory is reachable
   via the wikilink web and from at least one skill or boot digest; fix orphans.

### In scope

- Review and revision of all 29 shipped memory bodies (`memory.md` + `memory.toml`
  metadata) in the `memory/` source directory.
- Re-embed and re-sync cycle per batch of edits.
- `doctrine memory sync` and `doctrine claude install` re-run to materialize
  changes.
- Minimal reachability fixes: if a shipped memory has no inbound wikilinks,
  add them from the nearest appropriate parent memory.

### Out of scope

- **Full ADR-005 compliance audit.** Reference-doc IA (glossary, using-doctrine,
  templates), user-serviceable `.md` hooks (governance.md, boot-footer.md,
  reconcile rules), skill restate-line audit, and the overall information
  architecture of `install/*.md` are handled by a separate slice (CHR-023 /
  SL-nnn).
- **New entity kinds or CLI verbs.** This slice does not add new memory types,
  new entity kinds, or new CLI commands — it updates what exists.
- **Architecture changes to the memory engine.** The write path, sync
  mechanism, and retrieval semantics stay unchanged.
- **Client-project memories.** This slice touches only doctrine's own shipped
  corpus — not `.doctrine/memory/items/` in any client.
- **Full skill audit.** Skills are touched only where they bear on shipped
  memory reachability or the new self-correction gate.

## Risks & Assumptions

- **Re-embed footgun.** Every memory edit requires `touch src/corpus.rs && cargo build`
  to force RustEmbed recompilation. The loop is slow; batch edits and verify in
  one build cycle.
- **Corpus must stay evergreen.** Shipped memories are `repo=""` + `anchor_kind=none`
  (ADR-002) — they are reference-grade, not capture-store. Edits that introduce
  repo-specific detail or stale anchors break the class contract.
- **Self-correction is approximate.** A manifest-driven staleness check can flag
  *potential* drift (e.g. "CLI surface has new verbs not mentioned in memory X")
  but cannot verify semantic accuracy. It is a prompt to re-audit, not a
  correctness proof.
- **Dependency on IMP-148.** The `show` fallthrough fix may still have edge
  cases; verify before relying on shipped memory visibility.

## Affected Surface

- `memory/` — the RustEmbed source for all 30 shipped memories (TOML + MD).
- `src/corpus.rs` — `touch` target for re-embed.
- `.agents/skills/reconcile/SKILL.md` — drift detection addition.
- `.agents/skills/close/SKILL.md` — may invoke drift check.
- `Cargo.toml` / build — only for the `touch` re-trigger.

## Open Questions

All resolved by design lock (2026-06-24):

1. **Self-correction mechanism shape** → Deferred to IMP-163, which depends on
   SL-147's domain-map mechanism. No new CLI verb or skill mechanism in this slice.
2. **Manifest format** → Deferred with self-correction. The domain-map from SL-147
   will drive staleness detection.
3. **Ordering of phases** → Resolved: PHASE-01 (audit), PHASE-02 (content update),
   PHASE-03 (overview rewrite), PHASE-04 (reachability), PHASE-05 (re-embed & gate).
   Two phases keep evidence (audit ledger) separate from action (content update).

## Verification / Closure Intent

"Done" means:

- All 29 shipped memories reviewed and updated for currency; 3 new signposts
  created, 1 local memory promoted, 1 stale signpost deleted (net 32).
- References to CLI verbs tested against `doctrine --help`.
- Wikilinks between memories form a coherent onboarding path (no dead ends,
  orphans, or loops) — an agent new to doctrine can follow a curated
  progression from overview → core concepts → workflow → entity reference.
- Self-correction check runs as part of `/reconcile` (or equivalent gate) and
  surfaces drift warnings before close.
- Every shipped memory is reachable from at least one other memory via
  wikilinks, and from at least one skill or the boot digest.

## Follow-Ups

- **Self-correction gate** — tracked as IMP-163 (:after SL-147). Once SL-147's
  domain-map mechanism lands, the reconcile/close loop will use it to flag
  potentially stale shipped memories.
- ADR-005 full compliance (reference-doc IA, user hooks, restate-line audit) —
  tracked as SL-144.
- IMP-148 Gap 8 (show fallthrough) — monitor for edge cases surfaced during audit.
- Entity kind signposts for knowledge-record kinds (ASM, DEC, QUE, CON) and
  backlog-item kinds (CHR, IMP, ISS, RSK, IDE) — deferred to future slices;
  this slice fills the 3 most critical gaps (REC, RFC, CM).
