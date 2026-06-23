# ADR-005 full compliance: reference-doc IA, user hooks, restate-line audit

## Context

ADR-005 ("Shipped knowledge is tiered by access pattern; skills route,
reference docs explain") was accepted on 2026-06-08, with open questions
resolved by inquisition. Several deliverables were scoped as evidence-bound
(R-C1, R-C3), deferring full rollout:

- **PULL-tier CLI/editing reference doc** (R-C5) — a new shipped doc covering
  hand-editing mechanics, storage-tier read/write, edit-preserving rules, and
  which verb for what; distinct from `--help`.
- **Restate-line enforcement** (R-OQ-4) — skills MUST NOT reproduce flag
  syntax, option/enum tables, or storage-tier mechanics as prose.
- **PUSH-tier reference-forms block** (R-OQ-5) — appended to routing-process.md.

The initial evidence-bound scope fixed named file:line offenders but did not
do a systematic sweep. Since then, the codebase has grown new entity kinds
(REC, REV, POL, STD, knowledge records), new CLI surfaces (revision, policy,
standard, review verbs), and new user-facing hooks (governance.md,
boot-footer.md). The reference docs and templates have not kept pace.

The `install/*.md` set — routing-process.md, using-doctrine.md, glossary.md,
governance.md, boot-footer.md, review-ledger.md — is the entire user-facing
documentation surface that ships to every client. Its information architecture
(what goes where, how they cross-reference, how agents discover them) has
never been audited as a system.

Carved out from SL-143 (CHR-021) to keep the shipped-memory corpus overhaul
focused on memory bodies, not the broader documentation IA.

## Scope & Objectives

### Objectives

1. **IA audit of `install/*.md`** — audit the set as a coherent system:
   routing-process.md, using-doctrine.md, glossary.md, governance.md,
   boot-footer.md, review-ledger.md. Identify overlaps, gaps, contradictions.
   Resolve them with a clear content hierarchy.

2. **User-serviceable `.md` hooks** — document and harden the customisation
   surface:
   - `governance.md` — boot-injected, user-owned governance pointer.
   - `boot-footer.md` — boot-injected, user-owned footer.
   - **New: reconcile-rules.md** — user-owned reconciliation rules (what
     the reconcile/close loop consults for project-custom drift handling).
   Ensure each hook has a clear contract (what it controls, how it's injected,
   precedence rules, how to reset to default).

3. **Restate-line audit** — scan every skill for violations of ADR-005 R-OQ-4
   (skills MUST NOT reproduce flag syntax, option/enum tables, or storage-tier
   mechanics as prose). Fix named file:line offenders per the evidence-bound
   principle (R-C1). Skills MAY name a verb and cite a rule by name.

4. **Reference-doc currency** — update `glossary.md` and `using-doctrine.md`
   for all current entity kinds (REC, REV, POL, STD, knowledge records) and
   CLI verbs (revision, policy, standard, review, knowledge). Ensure they are
   pointed-at by the boot digest and relevant skills.

5. **PUSH-tier completeness** — verify the reference-forms block in
   `install/routing-process.md` is present, correct, and comprehensive per
   ADR-005 R-OQ-5. Fix if not.

6. **Reachability** — verify every shipped reference doc is reachable from
   at least one skill or the boot digest. Fix orphans. Document the
   reachability contract.

### In scope

- All files under `install/` — content audit, restructuring, editing.
- All skills under `.agents/skills/` — restate-line scan and fixes.
- `install/routing-process.md` (boot digest) — PUSH-tier completeness.
- `install/using-doctrine.md` — reference-doc currency.
- `install/glossary.md` — reference-doc currency.
- `install/governance.md` — hook contract documentation.
- `install/boot-footer.md` — hook contract documentation.
- **New** `install/reconcile-rules.md` — create and document as new user hook.
- `src/boot.rs` — if boot-injection needs changes for reconcile-rules.md.
- Re-embed and re-sync cycle per batch of edits.

### Out of scope

- **Shipped memory bodies.** The 30 shipped memories (signposts, concepts,
  patterns, facts) are handled by SL-143.
- **New entity kinds or CLI verbs.** This slice documents what exists; it does
  not create new kinds or commands.
- **Architecture changes to the memory engine, entity engine, or core CLI.**
- **Client-project memories or documentation beyond `.doctrine/`.**
- **Substantive changes to the boot snapshot format or delivery mechanism.**
- **Skills not yet authored.** The restate-line audit covers only existing
  skills.

## Risks & Assumptions

- **Restate-line scope creep.** If many skills copy flag tables, the fix
  count may be large. Hold the evidence-bound line: fix named file:line
  offenders. A skill that correctly cites a verb without reproducing its
  flags is already compliant.
- **Re-embed footgun.** Edits to `install/*` require `touch src/install.rs`
  (or whichever embedding crate) + `cargo build`. Batch edits accordingly.
- **New reconcile-rules hook must not break existing flow.** Design it as an
  optional include — if the file is absent, reconcile/close behaves as today.
- **ADR-005 compliance is not all-or-nothing.** Some skills may legitimately
  need inline CLI references (e.g. `execute` skill describing a phase
  transition verb). The restate line permits citing a verb and rule by name;
  the test is whether the skill *reproduces* flag tables or mechanics.

## Affected Surface

- `install/` — all `.md` files (content audit, edits, new reconcile-rules.md).
- `.agents/skills/*/SKILL.md` — restate-line scan target.
- `src/boot.rs` — if reconcile-rules.md needs boot-injection.
- `src/install.rs` — `touch` target for re-embed.
- `.doctrine/state/boot.md` — regenerated after routing-process changes.

## Open Questions

1. **Reconcile-rules hook shape.** A new `.md` file under `install/` that
   ships to `.doctrine/reconcile-rules.md`. Should it be boot-injected (like
   governance.md) or skill-read (like using-doctrine.md)? The latter is simpler
   and avoids boot bloat. Defer to design.
2. **IA audit methodology.** Should the audit produce a formal document (e.g.
   an ADR or a design doc) describing the target IA, or should it be
   resolved inline by editing until the docs are coherent? A design doc
   prevents thrash but adds ceremony. Defer to design.
3. **Restate-line automation.** Could a lint-style check (grep for `--flag`
   patterns in skill files) be useful, or is manual audit sufficient given
   20-odd skills? Defer to plan.

## Verification / Closure Intent

"Done" means:

- All `install/*.md` documents audited and coherent — no overlaps or gaps.
- Each user hook (governance.md, boot-footer.md, reconcile-rules.md) has a
  documented contract (what it controls, injection mechanism, reset path).
- Reconcile-rules.md ships as an optional hook (existing flows unchanged when
  absent).
- All skills comply with the restate line (or documented exceptions exist in
  the ADR-005 review ledger).
- `glossary.md` and `using-doctrine.md` cover all current entity kinds and
  verbs.
- Every shipped reference doc is reachable from at least one skill or the
  boot digest.
- PUSH-tier reference-forms block is present and correct in routing-process.md.

## Follow-Ups

- SL-143 (shipped memory corpus overhaul) — the sibling slice this was carved
  from.
- Any skill violations that are architectural rather than content (e.g. a
  skill whose design requires inline flag reference) should be filed as an
  ADR-005 review finding, not papered over.
