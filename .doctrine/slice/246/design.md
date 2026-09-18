<!-- doctrine:section sec-1 -->
# Design SL-246: Entity reads carry their knowledge records

## 1. Design Problem

### What a reader cannot do today

Doctrine's managed design workflow split a slice's design in two. The argument
lives in the slice's `design.md`; the rulings live in **knowledge records** —
separate entities, one per settled decision, question, assumption, constraint,
piece of evidence, hypothesis or concept, each with its own files. The split is
deliberate and it is not the problem. The problem is that nothing puts the two
back together at read time.

Take `SL-244` as the specimen. Its `design.md` is 3,456 lines and cites twenty
decision records by id, quoting fragments of them inline. Fifteen knowledge
records point *at* the slice. `doctrine slice show SL-244` names **none** of
them: it renders the entity's own file, which carries only the relations the
slice itself authored — the outbound ones. Every kind's `show` behaves the same
way for the same reason.

So the reader who wants what a design actually ruled has three bad options: read
the argument and trust the quoted fragments; run a second command to get a list
of ids and then a third command per id; or skip the records entirely. In
practice agents do the third.

### The derivation is not missing

The reverse direction already works. Relations are stored outbound-only and
reciprocity is derived (`ADR-004`), and `doctrine inspect <ID>` is the command
that derives it — a kind-agnostic view that groups an entity's inbound edges
under their derived verbs (`shaped_by`, `concerned by`, `originated from`). Run
against `SL-244` it finds all fifteen records correctly.

It emits **ids**. A reader who wanted content now has fifteen more commands to
run, and no indication which of the fifteen are worth running.

Separately, `doctrine knowledge show <ref>` and `doctrine knowledge inspect
<ref>` already render a record's content, the second without its prose body.

Both halves exist. Nothing joins them, and no surface leads an agent from the
question to either one.

### Target behaviour

One command renders an entity together with the **content** of the knowledge
records that point at it, under an explicit verbosity level:

| level | renders |
|---|---|
| `skip` | no knowledge — byte-identical to today's output, and the default |
| `facets` | per record, the fields that say what rules and what would change whether the ruling still stands |
| `full` | the complete record |

The middle level is the one that earns the feature. On the specimen it costs
about 30% of `full` — 31.8 KB against 107 KB across the fifteen records — which
is a real saving but not an order of magnitude, so it has to be *right* about
which fields it keeps rather than merely smaller.

### Where the boundary sits

**In.** One hop. Relation-keyed. Inbound only. Any subject kind that carries
knowledge relationships. The render lands on one command, and each kind's `show`
gains a one-line pointer to it so the reader who asks the question where it is
naturally asked is told where the answer lives.

**Out.** Prose citations are not consulted: `SL-244`'s design cites roughly ten
records it holds no edge to, and closing that gap is a validate-and-warn concern
on the *authoring* path, not a read-path scan. The recursive knowledge closure —
walking record-to-record edges and halting at non-record nodes — is a separate
piece of work; this design leaves a seam for it and stops at one hop. Facet
hygiene is not in scope either: where the corpus has left a record's fields
unfilled, this design says so plainly rather than compensating.

### What success looks like

Reading `SL-244` at the `facets` level surfaces the rulings of the twelve
decisions that shape it, without the eleven backlog rows that merely originated
it, at a cost a working agent would pay on purpose. The existing `show` output
for every kind is unchanged, byte for byte, unless the reader asks for more.

<!-- doctrine:section sec-2 -->
## 2. Current State

Three surfaces bear on this design. Two of them do half the job each; the third
is the corpus scan that feeds the first.

```mermaid
flowchart LR
  subgraph cmd["src/commands/inspect.rs — run_inspect"]
    RI["scan once, then render"]
  end
  subgraph rg["src/relation_graph.rs"]
    IF["inspect_from → InspectView<br/>id · outbound · inbound · danglers"]
    RF["render_from → render_human / render_json"]
  end
  subgraph kn["src/knowledge.rs"]
    RR["read_record → KnowledgeRecord"]
    FM["format_metadata → format_facet"]
    FS["format_show / format_inspect"]
  end
  SC["catalog::scan — ScannedEntity<br/>key · kind · status · title · outbound · risk · tags · body?"]

  RI --> SC
  RI --> IF --> RF
  SC -.feeds.-> IF
  RR --> FM --> FS
  RF -.->|"emits ids only"| GAP(["the join<br/>nobody makes"])
  GAP -.->|"would need content"| RR
```

*The two halves and the missing edge between them. `inspect` derives which
records point at an entity and prints their ids; `knowledge` can render a
record's content but only when a caller already knows which record it wants.*

### 2.1 `doctrine inspect` derives the inbound set correctly

`inspect_from` (`src/relation_graph.rs:636`) builds the relation graph from a
pre-scanned entity slice, gates on the id actually existing, and returns an
`InspectView` (`:572`) of four fields: the queried `id`, `outbound` groups,
`inbound` groups, and `danglers`. Inbound is recomputed from `in_edges` on every
query — no reverse field is stored, per `ADR-004`.

Two properties matter downstream.

**Groups are keyed by `(label, role)`, not by label alone.** A `references` edge
groups under its role, so inbound `implements` and `concerns` stay in distinct
buckets with distinct derived verbs ("implemented by", "concerned by") instead
of collapsing into one. The group key is what the composed read will caption a
record with.

**Each group's sources are sorted by `EntityKey`** — prefix lexicographic, id
numeric — so the render is deterministic and permutation-invariant past id 999.
A composed read inherits that ordering for free.

`render_from` (`:760`) turns the view into bytes: `render_human` for the table
format, `render_json` for `--json`. The human render is built as a `Vec<String>`
of parts each carrying its own newline, joined by `concat`, and each of the three
sections is omitted when empty. `render_inbound` (`:873`) is nineteen lines: for
each group it looks up the derived verb via `relation::inbound_name(label, role)`
and prints `  <verb>: <comma-joined refs>`.

The command shell (`src/commands/inspect.rs:52`) performs one corpus scan and
shares it between the relation render and the priority actionability block it
appends below. The JSON arm builds `inspect_value(view)` and injects
`actionability` as an additive key.

Ordinary `inspect` output is roughly one line per group. That is what makes
`skip` the honest default: the surface is currently cheap enough to run without
thinking about it, and it must stay that way.

### 2.2 `knowledge` already renders a record three ways

`read_record` (`src/knowledge.rs:1637`) reads one record's `record-NNN.toml`,
parses and validates it, attaches its tier-1 relation edges, and then reads the
sibling `.md` **unconditionally** — erroring if the file is absent.
`relation_edges` (`:1654`) is the existing `pub(crate)` accessor over it, added
so callers outside the module can reach a record's edges without reaching into
its internals. It is the shape the composed read's accessor should copy.

Rendering is already tiered, and this is the useful surprise:

- `format_metadata` (`:1795`) — a pure function of the record's own state:
  identity line, slug/kind/status, dates, tags, then `format_facet`,
  `format_evidence`, and the five relation axes.
- `format_show` (`:1835`) — `format_metadata` plus the prose body.
- `format_inspect` (`:1842`) — `format_metadata` alone, no body. This is
  `doctrine knowledge inspect`, and it is very nearly the `full` level already.

`format_facet` (`:1867`) dispatches on the record kind and emits each populated
field in template order through `show_opt_line` (`:1848`) or `show_list_line`
(`:1856`). Two behaviours are load-bearing for this design:

1. **Absent fields are silent.** `show_opt_line` returns an empty string for
   `None`.
2. **An entirely unpopulated facet renders as nothing at all** — not a blank
   block, not a header. `format_facet` emits the `\n[facet]\n` header only when
   the body is non-empty.

`RecordFacet::Concept(_)` returns an empty string by construction: a concept
record has no facet fields, because every concept rides its attributed prose
body. So a concept is permanently in state (2) **by design**, and is not the
same fact as a decision whose author never filled anything in.

The JSON path disagrees with both: `facet_json` (`:1994`) emits every field,
`None` as `null`.

### 2.3 What the corpus scan carries, and what it does not

`ScannedEntity` (`src/catalog/scan.rs:105`) is the reusable half of the old
relation-graph build — the all-kind walk with no reference graph on top,
consumed by both `inspect` and the priority graph. It carries the entity key,
kind descriptor, authored status, title, outbound edges, an optional risk facet,
tags, and an optional prose body gated by `ScanMode`.

It carries **no** `RecordFacet`. Neither does `CatalogEntity`, but that struct is
not on `inspect`'s path at all, so it is not the comparison that matters.

The scan's history is a warning: `[estimate]` and `[value]` facets were parsed
here and were deleted at `SL-222`, leaving a tripwire behind to catch any
residue. Adding a facet parse back to the scan would walk that decision
backwards, and would charge every consumer — `validate`, `survey`, the priority
graph, `backlog show` — for a parse only one of them wants.

### 2.4 What pins the current behaviour

`SPEC-013` pins rendered output byte-exact per verb through black-box goldens.
The relevant suites are `tests/e2e_inspect_golden.rs`,
`tests/e2e_inspect_transitive_golden.rs` and `tests/e2e_knowledge_cli_golden.rs`.

`tests/e2e_inspect_golden.rs` hand-seeds a synthetic corpus in a temporary
directory rather than reading the repository's own `.doctrine/` tree, precisely
because `inspect` reads only authored TOML: fixed bytes in, byte-exact bytes out.
That existing choice is the one this design extends.

