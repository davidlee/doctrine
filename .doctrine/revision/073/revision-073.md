# REV REV-073 — reconcile SL-282

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

SL-282 (DEC-384) retired install-time projection of `[hymns].expose` starters.
Starters are now written on demand by `doctrine prompt customise <slot>`, which
writes the framework body plus a self-`replaces` sidecar; install's one-time
legacy pass removes byte-identical unedited copies in pre-stamp repos.
SPEC-023 and its requirements still described the projector.

## Reconcile narrative

- [RV-422 F-8] SPEC-023: "projected editable starter" / "expose" rewritten to
  "customised starter" / `prompt customise` in § roots, the seal paragraph,
  § precedence, § Suppression, the double-emit risk bullet, the SPEC-009
  sentence, and the `interactions.toml` note. Seal and customise stay the two
  sides of one coin. The existing "hand-authored snippet without a sidecar
  doubles" boundary now names the case SL-282 creates: install no longer
  back-fills a missing sidecar beside an edited legacy starter, so such a
  starter appends to the framework text rather than replacing it.
- [RV-422 F-8] REQ-322: the self-`replaces` obligation moves from install-time
  projection to `prompt customise`.
- [RV-422 F-8] REQ-329: the "expose-side mirror of seal" is the customised
  starter's self-`replaces`.
- DEC-384 named in the audit brief needed no change — it already states the
  delivered model. DEC-382 (a knowledge record) is amended directly, not via
  this REV (RV-422 F-5, F-10).
