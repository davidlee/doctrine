# IMP-376: Acronym qualification rule into the boot snapshot

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Move the doc-local acronym qualification obligation out of the `/design` skill
and into the boot snapshot, where its actual scope is.

## The obligation

`plugins/doctrine/skills/design/SKILL.md:111-113`:

> If the conversation hinges on document-local acronyms (OQ-17, F-4): don't
> assume the user has them memorized. Introduce them qualified by the ID of the
> containing artifact, and provide a brief synopsis if not obvious from context.

## Why it does not belong in `/design`

`SL-233` PHASE-08 held this through every revision of its sketch as the sole
legitimate residue — *"unenforced by construction; it fires on every message and
no edge can hold a per-message obligation"*. That framing had the **scope**
wrong, not the mechanism.

Qualifying a doc-local id by its containing artefact applies to `/audit` reading
`F-` findings, `/plan` citing `OQ-`, `/reconcile` citing review findings, and to
every skill that puts a bare enumerated id in front of a human. It is
doctrine-wide and framework-level. Owner ruling, 2026-08-01.

## Where it goes

`install/routing-process.md:60-63` — the authored source of the boot snapshot's
**Reference forms** paragraph, which already carries the *first half* of exactly
this rule:

> **Reference forms.** Entity ids — prefixed, 3-digit zero-padded (`SL-023`,
> `ADR-005`, `REQ-059`); cite the durable id, never a mobile membership label
> (`FR-`/`NF-`). Doc-local enumerations — bare (`OQ-1`, `D1`, `R1`, `Q1`, `C1`).

That paragraph rules on how a doc-local id is **written** and says nothing about
how it is **introduced**. This is the missing second clause of a rule already in
context every turn, in every skill, in every project.

## Work

1. Extend the Reference forms paragraph in `install/routing-process.md` with the
   introduction clause.
2. `doctrine boot` to regenerate the snapshot; `doctrine boot --check`.
3. `doctrine install` to refresh installed skills.
4. Remove `:111-113` from `plugins/doctrine/skills/design/SKILL.md` — **note
   this is already accounted for in PHASE-08's line arithmetic**, which removes
   the range as part of the rewrite. This item supplies the destination, not the
   deletion.

## Then evaluate whether it fires

The owner's ruling attached an explicit caveat: the details are cheap to change
later, and whether the obligation *consistently fires* from the boot snapshot is
an open question. The snapshot is large and always present, which is close to
the ambient delivery DEC-103 distrusts — so placement is a hypothesis, not a
result.

Live evidence that the *old* placement failed: during the session that produced
PHASE-08's sketch, the agent authoring the acronym triage violated `:111-113`
while presenting that triage to the owner, in bare `D`-numbers. That is not
proof the rule is unenforceable — it is proof that an obligation buried in one
skill's prose does not fire when the work is a *conversation about* that skill.

CHR-049 already carries the live human-in-the-loop exercise post-close (DEC-079)
and is the natural vehicle for checking whether the new placement does better.

## Why it is not in SL-233

Editing `install/routing-process.md` changes the boot snapshot for every agent
in every project — a governance edit. Putting one inside a phase whose design
gate is mid-review is the same manoeuvre PHASE-08 already declined for IMP-374.
The placement is decided; the edit is separable.
