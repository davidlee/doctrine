# ISS-287: Shipped reviewing.md misdescribes what a section attestation binds

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found during SL-233 PHASE-16's second adversarial design review (2026-07-31),
when a design document cited this fragment as authority and the citation did not
survive checking.

## What

`install/design-prompts/reviewing.md:6-9` reads:

> An attestation binds the payload fingerprint, the disposition, the node and the
> revision. Change any of those and it is stale by construction — that is the
> point of binding it.

It sits immediately after a sentence about **section attestations**, so it reads
as describing them. It does not describe them.

- `Attestation` — the content-bound *section* review attestation (DEC-073) — is
  `{ id, subject, fingerprint, reviewer }` (`src/design_run/attestation.rs:36-41`).
  No disposition, no revision.
- The four-part binding belongs to `AcceptanceAttestation`, whose derived `digest`
  covers the checkpoint payload fingerprint, the inquiry disposition and the run
  revision (`:109-112`). The type's own doc comment is explicit that it is *"a
  different thing from `Attestation`, and deliberately not a `Reviewer` variant"*.

## Why it matters

It is shipped, embedded guidance: it is what an agent is handed at the reviewing
obligation, so the error propagates into whatever that agent then writes. It
demonstrably did — revisions 1 and 2 of `.doctrine/slice/233/sketches/
runbook-runner.md` quoted this sentence as the precedent for a digest-binding
rule, and inherited its error. DEC-101 carried the same wrong claim until it was
corrected at revision 3.

The final sentence — *"change any of those and it is stale by construction"* — is
sound and worth keeping. It is the attribution that is wrong.

## Fix

Either split it into two accurate sentences (section attestations bind subject and
content fingerprint; acceptance attestations bind payload, disposition and
revision), or generalise it to the shared principle without naming a field list.

**Not fixed in SL-233 PHASE-16**, which owns mechanism and holds a scope fence
against fragment bodies. PHASE-08 rewrites the fragment bodies and is the natural
home; if PHASE-08 slips, this stands alone as a two-line edit plus the install
delivery tail (`cargo build` to re-embed, `doctrine boot`, `doctrine install`).
