# IMP-058: Render the requirement .md prose tier (Statement/Rationale) in spec show or a requirement show verb

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

Requirement entities have two authored tiers: structured fields in
`requirement-NNN.toml` (`description`, `acceptance_criteria`) and a prose body in
`requirement-NNN.md` (`## Statement`, `## Rationale`). `spec show <PRD>` renders
**only** the TOML tier under its synthesized Requirements block, and there is no
`doctrine requirement show` verb. So anything authored into a requirement's `.md`
body is **unreachable via the read path** — the `shipped-not-reachable` pattern
(mem.pattern.distribution.shipped-not-reachable).

This contradicts the storage rule, which puts prose in `.md` and reserves TOML for
structured/queried data. Today an author who follows the rule (rich Rationale in
`.md`) gets a worse result than one who crams everything into the one-line TOML
`description` — the model punishes correct authoring. Observed authoring REQ-258
(SL-060): the allowlist rationale, the `descends_from`-not-`needs` decision, and the
IMP-047 gating-surface forward note belong in Rationale prose but won't surface under
`spec show`. Most existing requirements (e.g. REQ-097) sidestep this by leaving the
`.md` body an empty stub — i.e. the prose tier is de facto unused.

## What

Make the requirement `.md` prose tier reachable. Either:

- `spec show` appends each member requirement's `## Statement` / `## Rationale` body
  beneath its rendered TOML fields; and/or
- a `doctrine requirement show <REQ-NNN>` verb that reassembles both tiers (mirrors
  `spec show`'s two-tier synthesis for the requirement kind).

Decide whether the `.md` body becomes the canonical home for requirement rationale
(demoting the TOML `description` to a one-line summary) or stays an optional
deep-dive. Backfill empty stubs only if the body becomes load-bearing.

Related: **IMP-057** (the authoring side — a `/requirement` skill to enrich bare
requirement scaffolds). 057 and 058 interlock: enriching `.md` bodies (057) is
wasted effort while the read path can't surface them (058). 058 is the precondition
for 057's `.md` work being worth doing.
