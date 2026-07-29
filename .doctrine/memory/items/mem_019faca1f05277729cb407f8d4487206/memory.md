A phase's entrance criteria are authored at `/plan` time, **before** any phase
runs. By the time a later phase reaches its own design gate, an earlier phase may
already have built part of the thing the gate asks to specify — and the criterion
will not say so, because it could not have known.

## The instance

SL-233 PHASE-06's `EN-2` asks for a sketch answering seven questions about the
stable-section-ID marker grammar. It reads as greenfield format design. It is
not: `MARKER_OPEN` / `MARKER_CLOSE` (`src/commands/design.rs:66,68`),
`authored_sections` (`:249`), `authored_section_digests` (`:272`) and
`render_document` (`:1146`) already wrote and read markers at head.

Nothing in `EN-2`, the exit criteria, or the handover packet mentioned it. The
first draft of the sketch was written as a greenfield grammar and had to be
revised to ratify the incumbent delimiters instead.

**The near-miss is worse than the rework.** A grammar specified without reading
the incumbent could easily have chosen different delimiters, silently orphaning
every document the incumbent had already materialised — with no compile error and
no test failure, since the tests use whatever the format currently is.

## The practice

Before authoring any design-gate sketch that specifies a **format, protocol, or
on-disk contract**:

1. grep for the constants and the reader/writer pair by their obvious names;
2. if an incumbent exists, the sketch's job changes from *specify* to *ratify
   what must not move, and enumerate what is unsafe*;
3. state the incumbent explicitly in the sketch, so the reviewer can check the
   rows rather than re-deriving the format.

Reframing that way turned out to be the most valuable content in the artefact —
it produced a twelve-row defect table, three of whose rows were silent
data-loss paths.

Conversely: when authoring a *plan*, a design-gate criterion should name the
incumbent it supersedes, or state that there is none.

Related: [[mem.pattern.doctrine.conventions]], [[mem.pattern.claude.docs-first-then-probe]].
