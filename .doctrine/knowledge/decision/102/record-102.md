# DEC-102: Craft is overridable, invariants stay sealed

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The fork this settles was posed as *"should all the fragments become hymns, and
ultimately end-user overridable?"* — which would have reversed the only five
`customization = "fixed"` entries in `publication/manifest.toml` and unsealed
`stage/design`.

The question was mis-shaped. It reads as one axis (framework versus user) when
the assets already split along a different one.

## The line

**An asset is sealed when a project override would make its content false rather
than merely different.**

`install/hymns/stage/design.md` says "every mutation compare-and-swaps against
the run's revision", "`adopt_authored` is the only lawful crossing", "a payload
cannot declare itself accepted". A project that edits those sentences has not
changed its workflow — the code still does exactly what the original text said.
It has only made its own documentation lie.

`plugins/doctrine/skills/design/SKILL.md:98` says "prefer multiple choice
questions when possible". This project's own `CLAUDE.md` says the opposite for
design loops, and is right to, because the owner reframes questions rather than
picking from a list. That is a legitimate disagreement about craft, and today it
has to be expressed in a *different file* because the guidance carrying it cannot
be overridden.

So craft is overridable and invariants are sealed, and the existing five fixed
entries stand exactly as PHASE-07 shipped them.

## What actually changes — and what was claimed too early

Nothing in the manifest is reversed.

The first revision of this record went further and said the new thing — obligation
runbooks ([[DEC-101]]) — would ship `customizable`, "giving v1 exactly one
override seam". **That was false, and an external review caught it before
implementation.** `customization` is parsed and displayed, but its only production
consumer outside `publication.rs` is the library view (`src/commands/library.rs:111`),
and design assets resolve straight from the embed via `install::asset_text`
(`src/commands/design.rs:1633-1638`). Declaring a runbook customizable would have
made it no more overridable than anything else — a label with no machinery behind
it.

Owner's ruling, 2026-07-31: **the seam is identified and deferred.** The v1
runbook ships embedded like every other design asset, and the resolution work —
project-path lookup, framework fallback, precedence, fingerprint scope — is a
backlog item rather than a claim.

**What survives is the line itself**, which is what this record is actually for:
which assets are sealed, which are craft, and why. And the concrete win is real —
the craft has moved out of skill prose into data, which is the hard part. What is
honestly downgraded is only the delivery: the `SKILL.md:98` contradiction is fixed
for *this* repository and not yet for anyone else, because overriding a runbook
still requires a rebuild.

Recording the downgrade rather than quietly restating the goal matters here. A
deferral nobody wrote down is indistinguishable from an oversight, and this slice
has already been bitten twice by surfaces claiming a capability they did not have.

## Why this narrows DEC-077 rather than superseding it

[[DEC-077]] says v1 "does not create role/model variants **or design user-override
semantics**", and therefore that "framework prompt content **may** remain embedded
and authoritative". That is a deferral with a permissive modal, not a prohibition.
Shipping one customizable asset class exercises the deferral for that class and
leaves every other clause of DEC-077 standing — including the embedded-and-
authoritative status of the five sealed assets, which this record affirms rather
than reverses.

[[ADR-019]] does not move. It mandates only that every projected asset *declare*
a customization; it names no hymn and carries no per-asset value. If it named
specific overridable hymns it would be operating at the wrong level, and it
doesn't.

## Related

- [[DEC-101]] — the runbook runner, and the authoring rule this is the
  asset-policy face of.
- [[DEC-077]] — narrowed for one asset class, otherwise intact.
- [[ADR-019]] — unmoved; embedding, publication and projection stay independent
  asset policies.
