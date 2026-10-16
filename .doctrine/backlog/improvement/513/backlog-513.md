# IMP-513: Hymn starters materialise on customise intent, not install

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

`doctrine install` writes the `[hymns].expose` starter set
(`install/manifest.toml`) into `.doctrine/hymns/` on every install, regardless
of harness or model. They are user-owned (`hymns::Provenance::User`), so they
are tracked, and a fresh client repo gets files nobody asked to customise.

SPEC-009 D8 already says customised hymns "appear on disk only when first
used/customized". The install leg contradicts it.

Symptoms seen on `doctrine install --agent claude --agent codex` (2026-10-16):

- model starters: `anthropic/claude-sonnet-4`, `deepseek/_default`. Stale, and
  nothing for the GPT models a codex user runs. Currency itself is IMP-489.
- harness starters: `harness/claude`, `harness/cursor`; none for codex. The
  list doesn't follow the harnesses installed.

## Direction

Drop the expose leg from install. Materialise a starter when the user signals
intent to customise a slot (e.g. a `prompt` verb that writes the framework body
for a named slot as a user-provenance starter). Install then writes no hymn
files, and the stale expose list stops mattering.

## Related

- ISS-210 — the exposed `role/worker` starter self-replaces; goes away with this.
- IMP-489 — model coverage and currency.
- SL-281 — install artefact ownership; deliberately left the starters alone
  (user-owned, write-if-absent).
