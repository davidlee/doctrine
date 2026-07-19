# IMP-298: No edit-preserving CLI seam to revise a requirement statement

## Gap

A REV `modify REQ-NNN` row surfaces at apply as "land by operator hand-edit" with
no CLI seam to actually do it. `doctrine spec req` has only `add` / `status` /
`list`; `doctrine spec edit` touches only descent/parent. So revising a
requirement's statement means raw hand-editing `requirement-NNN.toml`
(`title` / `description`) **and** `requirement-NNN.md` — the one authored edit with
no edit-preserving verb behind it. Everything else in the change loop has a gated
verb; requirement statements are the hole.

## Compounding: two statement-home conventions

The corpus carries **two** conventions for where a requirement statement lives,
by scaffold vintage:

- **Older reqs** (e.g. REQ-164/165/171): statement in the `## Statement` **prose**
  body; template comment says the prose is normative.
- **Newer scaffolds** (e.g. REQ-353–358): the TOML **`description`** field is
  normative and `show` renders it as the statement line; the prose comment warns
  it must *not* duplicate `description`.

Landing a consistent roster under REV-028 meant reconciling both by hand — setting
`description` on the old reqs too so `spec show` renders uniformly. An author with
no CLI seam has to know which vintage they're editing and keep the two tiers
coherent by eye.

## Where it surfaced

REV-028 (RFC-021 C1 projection revision) — landing four `modify REQ` rows
(REQ-164/165/171/043). Captured in `.doctrine/rfc/011/case-notes.md`
(`sess-rev028-land`).

## Candidate fix

A `doctrine spec req edit --statement <text>` verb (edit-preserving, writing the
normative `description`, optionally `--title`), which would:

1. give `modify REQ` rows a real landing seam, and
2. canonicalize the statement home on `description` — collapsing the dual
   convention and letting `show` render one statement line regardless of vintage.

Decide alongside whether the older `## Statement`-prose convention is deprecated in
favour of TOML `description`, or whether both stay legal with `description`
authoritative.

## Related

- [[IMP-297]] — sibling REV/requirement tooling gap (REV `introduce` is SPEC-only;
  no REV path to originate a *product* requirement). Both surfaced landing REV-028.
