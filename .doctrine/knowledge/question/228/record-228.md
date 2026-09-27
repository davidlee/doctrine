# QUE-228: How does project authority override a pulled library rule

## The tension

ADR-023 (the authority model) ranks accepted project authority (tier 2) above
the Doctrine framework (tier 3), and lists "project preferences override
framework craft defaults without an override mechanism" as a consequence. That
works by precedence *at read time*: the override holds only if the tier-2 text
is in context when the tier-3 text is read.

REV-067 (ADR-005 names an owner per material type, from RFC-033) makes the
reference library the normative owner of rules and concepts, and adds the
memory line: memories point at the library instead of restating it. That is a
good fit for ADR-023 — memories are agent-edited tier-5 evidence and ADR-023
itself refuses to host its elaboration in a memory — but it concentrates
instruction in a tier that:

- is embedded in the binary: a project cannot edit it short of a fork and
  rebuild;
- is *pulled*, via `doctrine library show reference/<name>.md`, so nothing
  co-locates a library rule with the project text that overrides it;
- has, as its only project-side counterweights, boot-resident
  `.doctrine/governance.md` plus accepted ADRs, policies and standards whose
  titles (not bodies) ride boot.

Result: an agent that pulls a library doc mid-task can follow a rule the
project has overridden, without either source being wrong.

## Existing ground (dots)

- DEC-102 — draws the line: craft is overridable, invariants stay sealed. An
  override is legitimate only where it makes the text *different*, not *false*.
  It records the override seam as identified and deferred.
- IMP-372 — the deferred seam: `customization = "customizable"` is declared in
  `publication/manifest.toml` but no code resolves a project override.
- IDE-029 — the hymns cascade already has a `project` band with a replacement
  graph (`doctrine prompt resolve`); the only layered-override mechanism
  doctrine ships today.
- ADR-019 — asset policies (embedding, publication, projection); restoring
  projection for the reference tier was rejected (RFC-033's already-rejected
  list), so "copy it and edit it" is not the answer.

## Candidate directions (unassessed)

- Resolve IMP-372 for library docs: a project-path shadow with framework
  fallback, restricted by DEC-102's line to craft docs.
- Surface overrides at the point of read: `library show` appends project
  governance that cites the doc (a derived backlink), so the override travels
  with the pull.
- Route library craft through the hymns `project` band instead of a second
  override mechanism.

Not in RFC-033's scope; raised during REV-067 review (2026-09-27).


## Concrete case: `governance.md` (SL-273 research C3)

`reference/governance.md` opens "YOURS to edit. Loaded into system prompt by
`doctrine boot`". Boot does load `.doctrine/governance.md` when present
(confirmed by the user 2026-09-27), but install never creates it, so the library
doc is the only copy of the default and its "yours to edit" framing points at a
file the client does not have. What governance.md should be — seeded override
surface, created-on-need hook, or library-only default — is an instance of this
question. SL-273 deliberately leaves it untouched (DEC-343); the user judges
this question wants answering soon.
