# CHR-080: Shipped-corpus accuracy sweep: verify shipped claims against the CLI

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

The shipped corpus makes hundreds of claims about what doctrine *does* — how many
knowledge kinds exist, what their default statuses are, which verbs settle a
record, what a template field's allowed values are. Nothing verifies any of them
against the artifact that decides: the CLI.

The failure is invisible from inside this repo for the same reason ISS-309's is.
An author writing shipped prose has `doctrine --help` and the source to hand, so
the claim is true *when written*. The reader is an agent in a client repo, with
no way to notice that the corpus and the binary have diverged — and no reason to
doubt prose that reads as authoritative.

Grounded in POL-002: the product must not ship guidance that only this repo's
state makes true.

## Confirmed instances (verified 2026-09-26)

**`mem.signpost.doctrine.knowledge`** — the shipped orientation for the whole
knowledge surface. ISS-229 logged this as "six kinds, wrong default-status
vocab". Re-verified against `doctrine knowledge new --help`, the live records,
and `install/templates/knowledge-*.toml`: it is worse than logged.

- **Six kinds, not seven.** The CLI mints `assumption decision question
  constraint evidence hypothesis concept`. The signpost omits `concept` entirely
  — no kind, no prefix, no status. (`concept` is `CPT-003`-shaped; default status
  `draft`.)
- **Two wrong default statuses.** The signpost says assumption is
  `pending | proven | disproven | withdrawn`; the real default is `held`. It says
  decision is `pending | active | …`; the real default is `proposed`, and live
  records show `accepted`.
- **The verb list is materially incomplete.** It names "new, list, show, status".
  The surface is `new, list, show, inspect, edit, status, settle, paths` — and the
  missing `settle` is *the* resolving verb, the one an agent most needs named.
- **Its own syntax example is ungrammatical.** `doctrine link EVD-1 supports
  DEC-2` violates the corpus's own 3-digit zero-pad rule (`EVD-001`). Following
  the shipped example produces a refusal.

**`install/glossary.md`** — the doc that *defines* id vocabulary has no row for
`concept`/`CPT-001` in its knowledge-records table. So the reference a client is
told is authoritative cannot spell a kind the CLI mints. (Distinct from the
citation defect in ISS-309: these rows are correct illustrations; there is simply
one missing.)

**`install/claude-activation.md`** — the body narrates behaviour as of `SL-250`
("since SL-250 the plugin/marketplace install path retired…"). The claim may well
be current; nothing checks it, and the id makes it unverifiable to the reader
either way. Recorded as the class, not because this instance is known-wrong.

**Adjacent, already owned.** Three shipped signposts
(`mem.signpost.doctrine.{concept-map,rec,rfc}`) exist on disk and are absent from
the boot snapshot's Memory index. Confirmed a genuine indexing bug, not a stale
snapshot: regenerating with `doctrine boot` does not add them and `doctrine boot
--check` reports the snapshot clean. ISS-215 owns that fix; listed here because
"shipped but not reachable" belongs to the same class as "shipped but not true".

## Method

Per shipped artifact, pair every behavioural claim with the verb that decides it,
then re-derive:

- **Command shapes** — `doctrine <verb> --help`, never prose recollection.
- **Vocabularies and defaults** — the live `list` output plus
  `install/templates/*.toml`, which are the de facto schema contract and already
  carry the correct seed values.
- **Mechanical legs** (cheap, worth scripting before the prose pass): dangling
  `[[mem.*]]` wikilinks (one found: `mem.signpost.doctrine.dispatch` →
  `…dispatch-claude-arm-wrong-base`); and any id whose prefix the CLI does not
  mint.

Scope is both corpora — `install/` and the shipped `memory/` corpus — for the
same reason as ISS-309.

## Boundaries

- **Not the citation sweep** — that is ISS-309.
- **Not the migration of project-local memories into shipped** — that is CHR-036,
  which owns the dispatch corpus and is mid-flight.
- **Not the drift gate.** A check that keeps shipped prose bound to the CLI needs
  its own design; IMP-163 (the SL-143 self-correction gate via the SL-147
  domain-map) is the existing seam and should be assessed before a second one is
  grown.
- Fixing these instances by hand does not close the class — that is precisely the
  argument for a gate, and the argument that should be settled in the remediation
  slice rather than assumed.

## Links

The remediation slice this item and ISS-309 both seed. IMP-484 is the sibling
sweep on the *sufficiency* axis.
