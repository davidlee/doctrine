# REV REV-067 — ADR-005 names an owner per material type

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-033 (Learning surface: corpus unification and format) measured one
conceptual corpus published across the boot snapshot, the shipped memories, the
reference library and the skills at least four times, with no statement of
which copy is the source. ADR-005 tiers shipped knowledge by access pattern and
claims "single source per fact" as a consequence, but nominates no source — and
it never mentions the shipped memory corpus at all, which is where most of the
duplication now lives.

The user settled RFC-033's frame on 2026-09-26: one information architecture
over the existing serialisations, with ownership by material type; the
reference library is the normative owner of rules and concepts, and memories
point at it rather than restate it. This revision writes that frame into
ADR-005.

The frame lines up with ADR-023 (Authority model for agent instruction), which
ranks reference docs at tier 3 (framework, binding) and memories at tier 5
(evidence, never obeyed). Moving instruction out of memories into the library
puts each rule in the tier that can actually carry it — the same reason
ADR-023 refused to host its own elaboration in a memory. The cost is that more
instruction now sits in a tier a project cannot edit without a fork, while
tier 2's only override pathways are prose precedence and
`.doctrine/governance.md`. That tension is recorded as QUE-228 and is out of
scope here.

It also corrects one stale clause. Tier 2 still describes delivery as
"install copies `install/*` → `.doctrine/*`". Since ADR-019 and SL-227, reference
docs are *published*, read on demand with `doctrine library show
reference/<name>.md`, and never copied to disk.

Deliberately **left unchanged**:

- The Context section and the resolved open questions — the record of the
  original decision.
- The PUSH tier's content rule. Load-bearing rules stay resident; the boot
  snapshot is a *derived* surface (its sources are named in `boot_sequence`),
  so residency is compatible with single ownership.
- The restate line (R-OQ-4). The new memory invariant is its sibling, not an
  edit to it.
- ADR-019. Ownership is orthogonal to its asset policies.
- ADR-024's scope fence. RFC-033 dropped the public target that would have
  reopened it; Change 2 touches only the citation form.
- SPEC-011 and SPEC-026. The frame needs no new mechanism: boot already derives
  its sections, and the publication register already carries the library.

### The `lib:` citation form

Once the library carries most of doctrine's instruction, a citation of it must
be recognisable on sight, the way `[[mem.<key>]]` and `SL-NNN` are. Today boot
teaches the bare form `<name>.md`, while ADR-024 names the logical address
`reference/<name>.md`. The two have drifted, and neither is unambiguous: in a
client, `governance.md` names both a library doc and the project's own
`.doctrine/governance.md`, a tier-2 file (see QUE-228).

Change 2 marks ADR-024's published logical address with a `lib:` prefix —
`lib:reference/glossary.md`. Stripping the prefix yields exactly the argument
`doctrine library show` takes, it covers every published entry (not only
`reference/`), and it is matchable by a pattern, which makes the
citation-resolution criterion mechanical. Boot introduces it once. Migrating
existing bare citations (boot, skills, templates, memories) is slice work
descending from RFC-033, not part of this apply.

No REQ rows: ADR-005 and ADR-024 are the only governed truth that changes. `modify` rows are
hand-landed at apply, so each **After** block below is the exact replacement
text.

## Change 1 — ADR-005 (modify, primary)

### Tier 2 — correct the delivery clause and name the owner

**Before:**

````markdown
2. **PULL-reference — shipped `install/*.md` docs** that skills and boot point to.
   A pull-reference is only *visible* to a client if a skill or the boot digest
   **points at it** (install copies `install/*` → `.doctrine/*`, but an unreferenced
   `.doctrine/*.md` is read by no one — the `AGENTS.md` lesson). The set:
````

**After:**

````markdown
2. **PULL-reference — shipped `install/*.md` docs** that skills and boot point to.
   This tier is the **normative owner** of doctrine's rules and concepts (see
   *Ownership by material type* below). A pull-reference is only *visible* to a
   client if a skill, a memory or the boot digest **points at it**: reference docs
   are published, not projected (ADR-019), cited by their `lib:` address
   (ADR-024) and read with `doctrine library show <address>`; a doc nothing cites
   is read by no one. The set:
````

### Add tier 4 and the ownership table — insert after tier 3

**Before:**

````markdown
3. **Skills — thin task routers.** Each states *when* and *what to do for this
   stage*, and references tiers 1–2 for the shared *how*.

**Invariants:**
````

**After:**

````markdown
3. **Skills — thin task routers.** Each states *when* and *what to do for this
   stage*, and references tiers 1–2 for the shared *how*.

4. **PULL-orientation — the shipped memory corpus.** Dense, retrieved on demand,
   agent-voiced. A memory carries orientation (where to look next), sharp edges,
   and facts the reference library does not own; for anything the library owns
   it **points** (cites its `lib:` address) rather than restating.

**Ownership by material type (RFC-033).** Every rule or concept has exactly one
normative home. Every other surface either **derives** from it mechanically or
**cites** it by name.

| material | home | other surfaces |
|---|---|---|
| structured / generated (vocabularies, command shapes, boot sections) | code, `--help`, or the publication register | derived; checked by regeneration |
| rules and concepts | the reference library (tier 2) | cite by `lib:` address |
| orientation, sharp edges, facts the library does not own | memories (tier 4) | point at the library for anything it owns |
| routing | skills (tier 3) | — |
| human orientation (tour, quickstart, mental model) | its own documentation mode | may paraphrase for a human reader; must cite the owner |

**Invariants:**
````

### Invariants — append two bullets

**Before:**

````markdown
- A pull-reference doc must be **pointed-at** (by a skill or boot) — shipping ≠
  reachable.
````

**After:**

````markdown
- A pull-reference doc must be **pointed-at** (by a skill, a memory or boot) —
  shipping ≠ reachable.
- **The memory line:** a memory **MAY** cite a library-owned rule or concept by
  name; it **MUST NOT** restate it. The same applies to any memory the boot
  snapshot inlines.
- A library citation carries the **`lib:` marker** (ADR-024) and must
  **resolve**: every `lib:` address a shipped surface cites is in the
  publication register. A bare `<name>.md` is not a library citation.
````

### Consequences / Positive — qualify the single-source claim

**Before:**

````markdown
- Single source per fact (DRY); skills shrink to routing.
````

**After:**

````markdown
- One normative home per rule or concept (DRY); skills shrink to routing and
  memories to orientation.
````

### Verification — append two criteria

**Before:**

````markdown
- **VA** — no shipped doc reproduces `doctrine --help` content; each pull-reference
  is pointed-at by a skill or boot (reachability check).
````

**After:**

````markdown
- **VA** — no shipped doc reproduces `doctrine --help` content; each pull-reference
  is pointed-at by a skill, a memory or boot (reachability check).
- **VT** — every `lib:` citation in a shipped skill, memory or boot source
  resolves to a publication-register entry (citation-resolution check).
- **VA** — no shipped memory restates a rule or concept the reference library
  owns (the memory line). Restatement is not mechanically detectable; this
  criterion is held by review.
````

## Change 2 — ADR-024 (modify)

### Form 2 — mark the published logical address

**Before:**

````markdown
2. **A published logical address** — `reference/<name>.md`, resolvable in every
   client through `doctrine library show` (`ADR-019`). A published doc has no
   file on disk in a client, so this is a logical address, not a path.
````

**After:**

````markdown
2. **A published logical address** — cited as `lib:<address>`, e.g.
   `lib:reference/glossary.md`, where `<address>` is exactly the argument
   `doctrine library show` takes (`ADR-019`) and may name any published entry.
   A published doc has no file on disk in a client, so this is a logical
   address, not a path; the `lib:` marker says so on sight and tells it apart
   from a client file of the same name.
````

### The no-new-syntax sentence

**Before:**

````markdown
admissible, at any tier.** These six are not a new citation syntax; they are the
resolution seams a client already has.
````

**After:**

````markdown
admissible, at any tier.** Apart from form 2's `lib:` marker, these six are not
a new citation syntax; they are the resolution seams a client already has.
````

### Disposition table — the repoint row

**Before:**

````markdown
| a durable, corpus-internal referent | **repoint** to a published `reference/<name>.md` |
````

**After:**

````markdown
| a durable, corpus-internal referent | **repoint** to a published `lib:` address |
````
