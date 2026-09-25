<!-- Shipped reference. Published, not projected: there is no copy on disk in an
     installed project — read it with
     `doctrine library show reference/shipped-corpus-authoring.md`. It states
     the grounding rule once and the procedure for applying it; the governance
     record that owns the rule is not itself shipped, so this document is the
     rule's delivery copy. -->

# Shipped-corpus authoring

How to write text that doctrine **ships into a client repo** — the published
reference docs, the shipped memory corpus, and the installed skills. Every one of
those is read by an agent working in a repo that has *none* of doctrine's own
corpus, so anything they assert about doctrine's own repository is, for that
reader, either unreachable or — worse — silently true of something else.

## The rule

**Ground every claim on an address a client can resolve.** Six forms qualify:

1. **Prose that stands without a reference.** Inline the fact. Often the whole
   fix: a one-clause rationale rarely needs an id to carry it.
2. **A published logical address** — `reference/<name>.md`, resolvable in every
   client through `doctrine library show reference/<name>.md`. A published doc
   has no file on disk in a client, so this is an address, not a path.
3. **A shipped memory key** — `[[mem.<key>]]`, where the key is present in the
   shipped corpus.
4. **A skill name** — the shipped skills are invoked by name.
5. **A CLI verb** — `doctrine <verb> …`. The CLI is the source of truth for
   shapes; ask it with `--help`.
6. **An in-corpus relative path** — only where the target is installed *next
   to* the citing file, such as a skill linking to its own sibling reference. A
   published document has no directory to be relative to, so a relative link
   from one reaches nothing.

**Never ground a claim on a repository-private entity id or a repository-private
source or spec path.** Not at any tier, not "just this once", not with a
disclaimer. Entity ids are minted per repository and count from zero, so a client
has minted its own record at the same number: the citation does not dangle, it
resolves to *that* record, silently, with no error and no signal. A broken link
announces itself; this substitutes one claim for another in guidance an agent is
being asked to act on.

## What is *not* a violation

Most id-shaped text in shipped prose is correct, and sweeping it corrupts the
very documents that define the vocabulary. Three classes are **left alone**:

- **Reference-form illustrations.** Tables and headers that *define* what an id
  looks like — the glossary's kind/abbreviation table, the reference-form headers
  in the entity templates, the commented payload examples. The correct referent
  is the client's own record, which is exactly the point.
- **Client-structure references.** A published doc that tells a client where
  *the client's* specs, ADRs, slices, plans or handovers live is citing the
  client's own tree, not doctrine's.
- **Fill-in-the-blank scaffolding.** Placeholder lines in a template that the
  client replaces with its own ids.

The test throughout: **citing the client's structure** versus **citing this
repository's contents**.

## The disposition procedure

For each candidate site, in order:

1. **Is it one of the three classes above?** If yes — leave it. Record that you
   considered it and why.
2. **Does the fact stand alone cheaply?** If yes — inline it and drop the id.
3. **Is the referent durable and about the corpus itself?** If yes — repoint to a
   published `reference/<name>.md` address, existing or newly published.
4. **Could the reasoning live in a published doc?** If yes — publish it, then
   repoint. The rationale moves; the citation follows it.
5. **Otherwise** — drop the clause.

| class | action |
|---|---|
| illustration, client-structure reference, fill-in-the-blank | **leave** |
| one-clause rationale | **inline**; drop the id |
| durable, corpus-internal referent | **repoint** to a published address |
| reasoning with no shipped home | **publish**, then repoint |
| none of the above | **drop** |

Two disciplines when you apply it:

- **The replacement resolving is the evidence — not the id being gone.** A
  zero-hit search proves the deletion ran and nothing more. What has to be true
  is that the reader is now served: the published address resolves, or the prose
  stands without an id. Check *that*.
- **A grep locates; it does not conclude.** When you use one to find remnants,
  pair the empty result with a known-positive control, so the search itself is
  trustworthy.

## The maintainer-note header

Several published docs open with an editorial comment aimed at doctrine's own
maintainers — where the source lives, which governance record it descends from.
That class is **governed by the same rule**, not exempt from it: a header is
shipped text. Either express the source as the doc's own published address
(naming the document, never the repository's directory layout), or keep the note
with references admitted only to the published corpus itself. What a header may
never contain is a repository-private path or entity id — an editorial comment is
still read by the client agent.

## Where this is recorded

The rule is durable governance — an architecture decision record descending from
the shipped-knowledge tiering and from the embedding/publication/projection
policies. That record is not itself shipped and a client cannot reach it, so a
link to it is not admissible here either; the governance record cites *this*
document's published address, and this document is the rule's delivery copy.
Same rule, two tiers: the record owns it, this doc states it for an author at the
point of writing and adds the procedure.
