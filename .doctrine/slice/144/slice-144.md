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

### Scope reconciliation (2026-07-03, design)

Sliced 2026-06-23; the `install/` surface has since drifted materially and this
scope is reconciled to the live tree:

- **`boot-footer.md` is retired**, not a live hook. SL-187 (prompt cascade
  delivery) replaced the boot-footer round-trip: the `## Onboarding` boot
  section is now populated from onboarding-tagged **memory** bodies (seeded via
  `templates/seed-onboarding.md` → `mem.signpost.project.orientation`), and the
  `## Model band` section is a static `install/model-band.md` pull. `boot.rs`
  reads `boot-footer.md` nowhere; the file (and the orphan `.doctrine/boot-footer.md`)
  is stale residue whose header still claims live injection. Objective 2
  **retires** it rather than "hardens" it.
- **`model-band.md` is new** and live (boot `Static` source) — added to the
  hook/IA surface.
- **The install surface is larger than the flat `*.md` set** — it now includes
  `hymns/`, `agents/`, `templates/`, `manifest.toml`, `doctrine.just`,
  `doctrine.toml.example`. Scope shape **C+**: audit the `*.md` set +
  `manifest.toml` reachability as the system; **map** the new subdirs at stable
  altitude (they exist, who consumes them, reachability) without restructuring.
- **Hymns need no ADR-005 revision.** The context-keyed prompt cascade is
  governed by ADR-011 + SPEC-023, a resolver mechanism — not a 4th tier of
  ADR-005's static-doc trichotomy. The IA map cross-references rather than
  amends. `SL-144 references(concerns) SPEC-023`.
- **`reconcile-rules.md` is dropped**, not built. It would be a parallel
  implementation of the hymns `stage`/`project` bands. The general form is filed
  as **IDE-029** (lifecycle-stage hymn seams), sequenced after SL-191.
- **Skills live under `plugins/doctrine/skills/*/SKILL.md`**, not `.agents/skills/`
  (path drift). 30 skills; 8 carry `--flag <ARG>` candidates for restate-line
  triage.

## Scope & Objectives

### Objectives

1. **IA audit of the `install/` ship surface** — audit the `*.md` reference-doc
   set (routing-process.md, using-doctrine.md, glossary.md, governance.md,
   model-band.md, review-ledger.md) + `manifest.toml` as a coherent system:
   overlaps, gaps, contradictions, resolved with a clear content hierarchy. The
   new subdirs (`hymns/`, `agents/`, `templates/`) are **mapped** at stable
   altitude — where they sit in the IA, who consumes them, governance authority
   (ADR-011/SPEC-023 for hymns) — not restructured (scope shape C+).

2. **User-serviceable hooks** — document each customisation surface with a clear
   contract (what it controls, injection/resolution mechanism, precedence, reset
   path):
   - `governance.md` — boot-injected (PUSH), user-owned governance pointer.
   - `model-band.md` — boot `Static` source (PUSH), universal model-band floor.
   - `.doctrine/hymns/` overlay — user overlay of the context-keyed cascade
     (PULL via `prompt resolve`); documented **at altitude** only, internals
     owned by SL-191/SPEC-023.
   - onboarding seed — `templates/seed-onboarding.md` → seeded
     `mem.signpost.project.orientation` (the real onboarding hook).
   - **`boot-footer.md` — RETIRE.** Delete `install/boot-footer.md` and the
     orphan `.doctrine/boot-footer.md`; it is dead post-SL-187.

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

- `install/*.md` reference-doc set — content audit + editing.
- `install/manifest.toml` — reachability oracle (ship/seal/expose sets).
- Skills under `plugins/doctrine/skills/*/SKILL.md` — restate-line scan and fixes.
- `install/routing-process.md` (boot digest) — PUSH-tier completeness.
- `install/using-doctrine.md`, `install/glossary.md` — reference-doc currency.
- `install/governance.md`, `install/model-band.md` — hook contract documentation.
- **Delete** `install/boot-footer.md` + orphan `.doctrine/boot-footer.md` (retired).
- `hymns/`, `agents/`, `templates/` — **map** in the IA (altitude), no restructure.
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
- **ADR-005 compliance is not all-or-nothing.** Some skills may legitimately
  need inline CLI references (e.g. `execute` skill describing a phase
  transition verb). The restate line permits citing a verb and rule by name;
  the test is whether the skill *reproduces* flag tables or mechanics.

## Affected Surface

- `install/*.md` (content audit, edits) + `install/manifest.toml` (reachability).
- `install/boot-footer.md` + `.doctrine/boot-footer.md` — deleted (retired).
- `plugins/doctrine/skills/*/SKILL.md` — restate-line scan target (8 candidates).
- `src/install.rs` — `touch`/embed target for re-embed.
- `.doctrine/state/boot.md` — regenerated after routing-process changes.

## Open Questions

_All three sliced-time OQs resolved at design (2026-07-03):_

1. ~~Reconcile-rules hook shape.~~ **Resolved: dropped.** It would parallel the
   hymns `stage`/`project` bands. Filed general form as **IDE-029**.
2. ~~IA audit methodology.~~ **Resolved: a design doc** (this slice's `design.md`)
   captures the target IA; edits then execute against it. Prevents thrash; no
   new ADR (hymns already governed by ADR-011/SPEC-023).
3. ~~Restate-line automation.~~ **Resolved: grep-assisted + manual triage.** 30
   skills, 8 `--flag <ARG>` candidates — manual triage suffices; a permanent
   lint check is an optional follow-up, not a deliverable.

## Verification / Closure Intent

"Done" means:

- The `install/*.md` set audited and coherent — no overlaps or gaps; the new
  subdirs (hymns/agents/templates) mapped in the IA with governance cross-refs.
- Each live user hook (governance.md, model-band.md, hymns overlay, onboarding
  seed) has a documented contract (what it controls, injection/resolution
  mechanism, reset path).
- `boot-footer.md` deleted from `install/` and `.doctrine/`; nothing references it.
- All skills comply with the restate line (or documented exceptions exist in
  the ADR-005 review ledger).
- `glossary.md` and `using-doctrine.md` cover all current entity kinds and
  verbs.
- Every shipped reference doc is reachable (manifest ship set + a skill/boot
  pointer); orphans fixed. Reachability contract documented.
- PUSH-tier reference-forms block is present and correct in routing-process.md.

## Follow-Ups

- SL-143 (shipped memory corpus overhaul) — the sibling slice this was carved
  from.
- **IDE-029** (lifecycle-stage hymn seams) — the general form of the dropped
  reconcile-rules hook; sequenced after SL-191.
- If the hymns cascade genuinely warrants folding into ADR-005's tier model
  (vs staying under ADR-011/SPEC-023), file an ADR-005 review finding — do not
  amend mid-slice.
- Any skill violations that are architectural rather than content (e.g. a
  skill whose design requires inline flag reference) should be filed as an
  ADR-005 review finding, not papered over.
