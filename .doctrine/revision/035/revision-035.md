# REV REV-035 — Add the observation ledger container

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-011 and SL-231 establish observations as a distinct raw-signal capability.
PRD-018 defines that product intent, and SPEC-028 defines its observation-ledger
container. Because SPEC-003 is the whole-system context and explicitly enumerates
Doctrine's containers, introducing SPEC-028 leaves that authored inventory
incomplete unless SPEC-003 is revised in the same governance change.

This revision changes only SPEC-003: its structured responsibility and prose
container inventory gain the observation ledger. It does not treat PRD-018 or
SPEC-028 as revision rows because the Revision grammar cannot create specs;
those new draft entities are authored directly under the storage rule.

Before:

> The container inventory names memory and then proceeds to id lifecycle,
> without an observation-ledger container.

After:

> The container inventory names the observation ledger (SPEC-028) as the
> append-only raw-signal capture and inspection boundary.
