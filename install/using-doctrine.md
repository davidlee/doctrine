<!-- Shipped reference (ADR-005 PULL tier). Edit the source in
     `install/using-doctrine.md`; the installed copy at `.doctrine/using-doctrine.md`
     is inert. Names verbs and states discipline — it never reproduces
     `doctrine --help`; ask the CLI for exact flags. -->

# Using doctrine

How to *operate* doctrine: which verb for which intent, how to read and edit the
artifacts, and the rules that keep authored state coherent. For **vocabulary and
ids** see `glossary.md`; for the **workflow** (route → slice → design → plan →
execute → close) see the routing digest. For **exact command shapes and flags**,
ask `doctrine <command> --help` — this doc names verbs, never their flag tables.

## Which verb for which intent

Ad-hoc operations (the workflow doc owns the phase *sequence*; this is the
reach-for-it map):

| intent | verb |
|---|---|
| read an entity (all tiers, synthesized) | `doctrine <kind> show <ID>` |
| survey what exists | `doctrine <kind> list` |
| scope a change | `doctrine slice new` |
| capture a unit of work intent | `doctrine backlog new <kind>` |
| survey / inspect the backlog | `doctrine backlog list` · `doctrine backlog show <ID>` |
| transition a backlog item | `doctrine backlog edit <ID>` |
| relate two entities (a slice to its spec/ADR, a backlog item to its slice) | `doctrine link` · `doctrine unlink` |
| transition a phase (e.g. flip `in_progress` → `completed`) | `doctrine slice phase` |
| capture friction as it happens | `doctrine observation record friction` |
| read the friction corpus | `doctrine observation list` · `doctrine observation search` |
| record a durable fact | `doctrine memory record` |
| find / retrieve a memory | `doctrine memory find` · `doctrine memory retrieve` |
| regenerate the boot snapshot | `doctrine boot` |
| check a slice's phase rollup | `doctrine slice list` |

`<kind>` is `slice`, `spec`, `adr`, `memory`, `backlog`, … (see `glossary.md`).
Ask `doctrine <kind> --help` for the subcommands and flags each verb takes.

## Which home for which record

Four homes, told apart by what the record *is* — do not conflate them:

- **Backlog = latent work.** A unit of work intent that can be triaged,
  prioritised, and promoted into a slice — `issue`, `improvement`, `chore`,
  `risk`, `idea`; captured with `doctrine backlog new <kind>`. The gate is the
  **work-intake membership test** (`mem.concept.backlog.work-intake-membership`):
  if a candidate does not fit the work-status lifecycle
  (`open|triaged|started|resolved|closed`), it is **not** a backlog item. A
  `risk` is admitted only as *unresolved work-risk* — uncertain future harm that
  may need mitigation, acceptance, or expiry — never as a general epistemic note.
- **knowledge_record (PRD-010) = epistemic / governance records** — seven kinds:
  assumptions (ASM), decisions (DEC), questions (QUE), constraints (CON),
  evidence (EVD), hypotheses (HYP), and concepts (CPT); each with its own held→validated
  lifecycle. EVD and HYP carry `supports`/`disputes` evidentiary edges for
  tracing provenance. To gate work on a record, the *dependent* work item
  authors `doctrine needs <work> <REC-ID>` — blocked while the record is
  unsettled, unblocked when `knowledge status` settles it; records never
  author dep/seq themselves. Not work; not the backlog.
- **ADR = high-impact architectural decisions** (`doctrine adr new`) — a chosen
  direction with consequences (`proposed → accepted → superseded`).
- **Memory = durable knowledge** (`doctrine memory record`) — a reusable fact,
  pattern, or gotcha a future agent would otherwise rediscover.

When several seem to fit, the membership test arbitrates: the backlog is the home
for unresolved *work intent*, never for every unresolved thing.

## Capturing friction — observations

An **observation** is not a fifth home for the records above. Those four ask
"what should we do / believe / decide / remember"; an observation records *what
happened while working* — the friction, the wrong turn, the thing that cost
twenty minutes — at the moment it happens, before it is understood well enough
to be triaged. Capture is deliberately cheap: no duplicate search, no
classification, no triage. A summary is the only required field.

    doctrine observation record friction "<summary>" [--detail ...]

Read the corpus back with `doctrine observation list` / `search`; correct it
with `supersede` / `retract` (records are never edited in place). Ask the CLI
for flags.

**Which interface — it depends on where you are running.** The corpus is one
shared, authored tree, so capture must go through whatever seam can actually
reach it:

| where you are | how to capture |
|---|---|
| the primary worktree | the CLI, as above |
| a confined worker with the doctrine MCP server | the `observation_record` MCP tool |
| a worker fork with neither | **don't** — report the friction in your hand-back |

That last row is the one that matters. A fork-local capture is written into a
tree that is about to be discarded, so it reads as success and silently loses
the record. Doctrine refuses it rather than accept it, and names the broker to
use if one is available.

### Records are authored by default

Authoritative records are **authored collection data**: committed, diffable
TOML under `.doctrine/observations/records/`, visible to git like any other
authored entity. Capture itself never stages or commits — a new record sits
untracked until you or a coordinator commits it.

Only the reserved publication temporaries are gitignored. The writer publishes
a record by writing a complete sibling temp file and hard-linking it into
place, so an interrupted capture can leave a `.tmp.`-prefixed name behind; that
one narrow pattern is installed for you, and nothing else in the tree is
ignored.

**This has a cost, and it is a real one.** Committed records show up in
diffs and pull requests. A team capturing friction liberally will see review
noise from records that have nothing to do with the change under review.

There are two ways to opt out, and they are not equivalent:

- **Repository-wide** — ignore `.doctrine/observations/records/` in the
  project's `.gitignore`. A shared decision: it applies to everyone.
- **Locally** — exclude the same path in `.git/info/exclude`, or via a global
  ignore file. Your own choice; it does not affect collaborators, and it will
  surprise them when your records never arrive.

**Either way you are choosing local-only storage, and giving up three things:**
the corpus stops being shared, so nobody can see friction but its author;
records can no longer be correlated across people, machines, or worktrees; and
there is no audit history — a local corpus has no durable record of what was
observed when, and vanishes with the checkout. Git is the transport here. If
you turn it off and still want those properties, something else has to provide
them.

The default is deliberate. Keep records authored unless the review noise is
actually hurting, and prefer the repository-wide choice over the local one when
you do opt out — a decision the whole team can see beats one that silently
differs per developer.

## Reading entities — always via `show`

Read an entity through `doctrine <kind> show <ID>`, never by opening one raw
file. An entity is stored across tiers: structured data in `*.toml`, prose in
`*.md`. `show` synthesizes both. A `*.md` body may be **empty by design** — its
substance living in the sibling `*.toml` — so judging an entity "hollow" from its
prose tier alone is a false reading. When in doubt, `show` it.

## Storage tiers — what goes where

Three tiers; know which one you are writing:

- **Authored** (`*.toml` + `*.md`, committed): structured/queried data in TOML,
  prose in MD. **Never put queried or derived data in prose** — it goes stale and
  lies. Lifecycle fields (e.g. a `status`) live in the TOML and are hand-edited
  there.
- **Runtime state** (under `.doctrine/state/`): disposable, gitignored progress —
  never commit it, never record progress in an authored file.
- **Derived**: regenerable indexes / caches — gitignored.

**Example — inside a slice directory:** `slice-NNN.toml`, `slice-NNN.md`,
`design.md`, `plan.toml`, `plan.md`, and `notes.md` are **authored** (committed,
diffable). `handover.md` and the `phases/` symlink are **runtime** (gitignored) —
they carry disposable context and phase tracking, never committed progress. See
`glossary.md` for the full directory layout.

**Hand-edit vs verb.** Reach for a verb to create or transition an entity; hand-
edit the TOML for fields no verb yet owns (cite the CLI gap if so). Prose is always
hand-edited. Keep each datum on its correct side of the tier split.

## Relating entities

Connect entities with the **`link` verb**, not a hand-written row. `doctrine link
<source-id> <label> <target-id>` writes the outbound relation; `doctrine unlink`
removes it. Storage is **outbound-only** — you link from the source side and
reciprocity is derived (ADR-004); `inspect` / `show` render both directions.

The legal `(source, label) → target` vocabulary lives in **`RELATION_RULES`**
(`src/relation.rs`, ADR-010) — the single source of truth. Don't transcribe it;
`link` rejects an illegal pair. Not every axis is `link`-writable: most relations
(e.g. a slice's `governed_by` / `specs` / `supersedes`) are, but the spec spine
(`descends_from` / `parent` / `members` / …) stays a typed key written by its own
flow — `RELATION_RULES` says which is which, so ask it rather than guess.

Either way, do **not** hand-author `[[relation]]` rows into a `.toml` — hand-rows
drift malformed and skip the legality check (`doctrine link` is the validated seam).

## Edit-preserving rules

- **Ids are identity, and immutable.** Phase ids (`PHASE-01`) and criteria ids
  (`EN-1`/`EX-1`/`VT-1`/`VA-1`/`VH-1`) are never renumbered or reused — **edits
  append**. The slug is never authoritative; cite the prefixed id.
- **Cite the durable id**, never a mobile membership label (`FR-`/`NF-` move per
  spec — cite the `REQ-NNN` they label). Reference forms: `glossary.md`.
- Preserve surrounding structure when hand-editing — match the file's existing
  shape rather than reformatting it.

## Pointers

- `glossary.md` — kinds, ids, reference forms, verification taxonomy.
- `doctrine <command> --help` — the authoritative, self-documenting command shapes.
