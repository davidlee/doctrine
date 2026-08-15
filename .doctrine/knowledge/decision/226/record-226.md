# DEC-226: Published contract doc ships fixed, not customizable

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Why the question is live

`DEC-224` ships a generated reference doc into the published library. Every
manifest entry must declare a customization status: `publication.rs:86` carries a
two-member vocabulary — `customizable` / `fixed` — required per entry and parsed
fail-closed, so an unknown value is refused outright (`publication.rs:394`).
There is no default to fall into and no way to leave the question open.

This also corrects a triage error worth stating plainly. `ADR-019` was first
recorded as *not engaged* by this slice. That holds for the **embed-root** leg —
`install/` is already a RustEmbed root, so no `flake.nix` graft is needed — but
the **publication** leg is engaged, and that is the leg this record answers.
Asset-policy independence is exactly what lets one apply while the other does not.

## Sealing follows from what the prose is

`DEC-102` and `DEC-122` hold that where the engine enforces a claim, a client
override of that claim is *false* rather than merely different. The usual reason
to make an asset customizable is that it carries editorial opinion a project
might reasonably hold differently.

This document has no editorial axis. Its content is a rendering of what the
binary's deserialiser actually accepts — derived from the types under `DEC-221`'s
pins, not authored judgement. A client project that edits it does not express a
local preference. It states something false about the engine, and the next caller
who believes it constructs a payload that is refused.

That is a worse outcome than having no document, which is the condition
`DEC-122`'s sealing rule exists to prevent.

## What a client project can still do

Read it, and stop reading it. What it cannot do is edit it in place and have the
edit survive as though doctrine vouched for it.

A project wanting local payload guidance — house conventions, which acts it uses,
worked examples from its own runs — writes its own document beside the published
one. Nothing here prevents that, and it keeps the derived and the authored
distinguishable, which is the point.

## Keeping `IMP-372` out

This was the reason the question was asked at all. An override seam would pull
`IMP-372`'s machinery into a slice scoped as a documentation surface, and the
slice's own Non-Goals already commit to *no override seam, no loader* on
`DEC-122`'s deferral of client-project authorship of condition contracts.

Answering `fixed` keeps that commitment without needing a further argument, and
introduces no new engine-tier concept: the vocabulary already exists and is
already enforced.
