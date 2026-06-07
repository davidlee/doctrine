# Backlog entity v1: work-intake items (one kind + item_kind facet)

## Context

The backlog is the **capture** step of the spec-driver loop
(`capture → scope → design → implement → audit → close`) — the work-intake layer
that feeds slice scoping and prioritisation. Doctrine has it as *intent only*:
the glossary reserves the id schemes and `entity-model.md` fixes the modelling,
but nothing is structural yet. Items are captured ad-hoc in `backlog.local.md`.

This slice makes backlog items **first-class entities** in the Doctrine way —
modelled on Spec-Driver's backlog but simpler and cleaner. It rides the shipped
scaffold engine (`src/entity.rs`, SL-003) the way ADR (SL-006) and spec
(SL-015) did — backlog is the engine's next caller, not a reason to fork it.

**The modelling is already decided by canon, and it diverges from the research
corpus.** `entity-model.md:74` fixes the umbrella choice: *one* `backlog_item`
entity discriminated by an `item_kind` facet — **not** Spec-Driver's four
parallel kinds with four directories and four schemas ("one kind + `item_kind`,
not six schemas; risk gets extra facet fields"). `backlog.local.md` is *input
research*, not a spec; where it conflicts with `entity-model.md` / `glossary.md`,
canon governs. The divergences `/design` must formalise and `/inquisition` must
test are tabulated under *Design Direction*.

## Context Bundles & Sources

Where a `/design` agent should read in, grouped by authority:

**Governing canon (committed — these win over the research corpus):**
- [`doc/entity-model.md`](../../../doc/entity-model.md) — the umbrella taxonomy.
  `:74` fixes `backlog_item` = one kind + `item_kind`, risk gets extra facets.
  `:109` fixes the status vocabulary. `:147` places backlog in the roadmap
  (lands behind its registry gate, one entity at a time, via the supersede
  pattern — exactly how memory and spec arrived).
- [`doc/glossary.md`](../../../doc/glossary.md) — the reserved ids and folder
  flags: `issue ISS`, `improvement IMP`, `chore CHR`, `risk RSK`, `idea IDE`
  (all `folder = y`; `./research/*` + `./context/*` subdirs permitted).

**Reference entities (code — the reuse seams, do NOT fork):**
- `src/entity.rs` — the kind-parameterised scaffold engine (SL-003, done).
- `src/spec.rs` + [`slice/015/design.md`](../015/design.md) — the closest
  precedent: a single entity carrying a **subtype/facet discriminator**
  (`product|tech`) over the shared substrate. `item_kind` mirrors that seam.
- `src/adr.rs` + [`slice/006/design.md`](../006/design.md) — the minimal
  "new entity rides the engine unchanged" precedent.
- `src/slice.rs` — the other substrate caller (status rollup, lifecycle).

**Supporting doc notes:**
- [`doc/relation-index.md`](../../../doc/relation-index.md) — the registry / FK
  layer backlog items participate in (relations to slices/specs/requirements).
  Cache deferred; only the FK-independent surface is in v1 reach.
- [`doc/drift-spec.md`](../../../doc/drift-spec.md) — `:100,:242` make `backlog`
  a first-class drift *resolution path*; a drift row can point at a backlog item.

**External source corpus (read-only):**
- `/workspace/spec-driver/.spec-driver/backlog/{issues,problems,improvements,risks}/`
  — the worked Python backlog: real items, frontmatter, status lifecycles,
  priority registry. Sample for shape; do not import the four-directory layout.

**Local research (GITIGNORED, disposable):**
- [`backlog.local.md`](../../../backlog.local.md) — the full Spec-Driver backlog
  study (concepts, schema, priority system, CLI surface, sync). The richest
  source — but Python-shaped; read for *what it does*, not *how it lays out*.

## Design Direction (canon-fixed + open — formalise in `/design`)

The umbrella decision is **not** open: one `backlog_item` entity, `item_kind`
facet, riding the engine. What `/design` formalises and `/inquisition` tests is
how the research corpus reconciles with canon, and the v1 implement subset.

### Canon vs research corpus — the divergences

| Axis | `backlog.local.md` (Spec-Driver) | Doctrine canon | Resolution |
|---|---|---|---|
| Modelling | 4 kinds · 4 dirs · 4 schemas | one entity + `item_kind` facet (`entity-model:74`) | **canon** — single kind |
| Kind set | issue, problem, improvement, risk | issue, improvement, chore, risk, idea (`glossary`) — `problem` in `entity-model:74` prose only, **no reserved id** | open: §Q1 |
| ID scheme | `ISSUE-010` (5-char, 3-digit) | `ISS-/IMP-/CHR-/RSK-/IDE-NNN` (3-char) | **canon** — 3-char prefixes |
| Status | open→triaged→in-progress→resolved | `open\|triaged\|started\|resolved\|closed` (`glossary:109`) | **canon** — `started` not `in-progress`, adds `closed` |
| Risk states | +accepted, +expired | not in canon vocab | open: §Q2 |
| Layout | `backlog/{kind}s/<ID>/` | one entity → engine's per-kind reservation dir | **canon** — engine layout |

### Open questions for `/design` (D-Q)

- **Q1 — the kind set.** Glossary reserves five (issue/improvement/chore/risk/
  idea); `entity-model:74` prose also names `problem`; Spec-Driver has `problem`
  but not chore/idea. Lock the v1 `item_kind` enum. Candidate: the five
  glossary-reserved kinds; `problem` deferred unless it earns a reserved id.
- **Q2 — risk facet + extra states.** `entity-model:74` says "risk gets extra
  facet fields" (likelihood/impact/controls/origin). Are `accepted`/`expired`
  adopted into the status vocab for risk, or is acceptance a separate field
  (entity-model:110 — "approval is not lifecycle")? Decide the risk facet set.
- **Q3 — facet storage shape.** Which fields are structured TOML facets per kind
  (severity, categories, impact, likelihood) vs the catch-all the research
  corpus uses? Doctrine has no "raw frontmatter hashmap" escape hatch — the
  storage rule wants typed TOML. Fix the per-`item_kind` facet sets.
- **Q4 — reservation namespace.** Per-kind (`backlog/issue/id/<n>`) to match the
  per-prefix sequential ids glossary implies, vs one shared `backlog/id/<n>`.
  Per-kind mirrors `spec/product` vs `spec/tech` (SL-015) and yields the
  `ISS-NNN`/`IMP-NNN` independent counters glossary shows.
- **Q5 — v1 implement subset.** Confirm the create/list/show/edit surface below;
  confirm priority registry + prioritise + sync + the `--from-backlog` bridge are
  deferred. (The consult/capture **skill wiring** is in scope — see Q6.)
- **Q6 — workflow integration points.** Which skills / routing rows *consult* vs
  *capture* the backlog, and at what moment in the loop? Lock the loop-point map
  (route/preflight consult; consult/notes capture; audit/close harvest) and the
  work / knowledge / decision boundary text the skills cite. Sequence as a phase
  after the CLI verbs (behaviour-preservation on the shared skill/boot surface).

## Scope & Objectives

**Design (whole):** the `backlog_item` entity — `item_kind` discriminator, the
per-kind facet sets (incl. the risk facet), the canon status lifecycle, the
reservation namespace, and the relation participation (edges to slices / specs /
drift). Reconcile every divergence row above.

**Implement (coherent subset) — candidate v1, confirmed in `/design`:**

- **`backlog new <kind> "Title"`** — scaffold the item fileset via the engine;
  per-kind reservation (Q4); template frontmatter with kind defaults
  (status `open`, kind facets). `<kind>` ∈ the Q1-locked enum.
- **`backlog list`** — table view; filters `--kind`, `--status`, `--tag`,
  title substring; resolved/closed hidden by default (`--all` to show). Mirrors
  `slice list`'s derived-rollup style where it applies.
- **`backlog show <ID>`** — auto-detect kind from the id prefix; render
  identity + kind-specific facets + timestamps + outbound relations. (Inbound
  reverse-refs deferred to the registry surface, as SL-015 deferred them.)
- **`backlog edit <ID> --status <s>`** — atomic, edit-preserving (`toml_edit`)
  status transition. (Editor-open edit optional; status flag is the v1 floor.)

- **Workflow integration — consult & capture at the right loop points.** Revise the
  routing table (`.doctrine/state/boot.md`) and the affected skills so the backlog is
  consulted and captured at the appropriate moments in the spec-driver loop — closing
  the gap where work intent is stranded in conversation. PRD-009 §5 makes "intake
  stops leaking" a success measure; the CLI verbs are necessary but not sufficient
  without the surfaces that prompt their use:
  - `/route`, `/preflight` — **consult** the backlog (`backlog list`/`show`) at the
    start of substantive work: is this intent already captured? which open items bear
    on it?
  - `/consult`, `/notes` — **capture** emergent issues / risks / ideas (`backlog new`)
    when an obstacle, tradeoff, or follow-up surfaces, instead of losing it.
  - `/audit`, `/close` — **harvest** durable findings into backlog items (risks,
    issues, chores) alongside the existing `audit.md` / memory harvest.
  - **Boundary guidance** so skills route correctly: backlog = latent *work*; memory =
    durable *knowledge*; ADR = *decisions*. The arbiter is the work-intake membership
    test (`mem.concept.backlog.work-intake-membership`).
  Rides the v1 `new`/`list`/`show` surface; sequence as a phase **after** the CLI
  verbs land (behaviour-preservation on the shared skill/boot surface). The deeper
  `slice new --from-backlog` bridge stays a follow-up (Non-Goals).

**Reuse, don't fork.** `src/backlog.rs` mirrors `src/spec.rs`/`src/adr.rs` over
the shared `src/entity.rs` substrate; the fileset descriptor supplies each
`item_kind`'s facet combination. Extract only genuinely-shared substrate.

**Wiring (the authored-entity trap — `mem.pattern.install.authored-entity-wiring`):**
add `.doctrine/backlog` to `install/manifest.toml` `[dirs].create` **and** the
`!.doctrine/backlog/` gitignore negation, or the tree is silently uncommittable.

## Non-Goals

- **Priority registry + interactive prioritise** (`backlog.yaml` ordering,
  `--prioritize` editor flow, head-tail partition merge) — the prioritisation
  layer; designed-noted, built in a follow-up. v1 has no global ordering.
- **`sync`** — registry↔filesystem reconciliation (append/prune/dry-run). Needs
  the registry; later, with the relation-index cache.
- **Delta/slice integration** (`slice new --from-backlog <ID>`) — the capture→
  scope *bridge command*; the prime follow-up, but a separate change. Distinct from
  the consult/capture **skill wiring**, which IS in scope above — the wiring tells
  agents *when* to reach for the backlog; the bridge is the later automation.
- **Relation-index *cache* + reverse-reference scan** — only relation *storage*
  lands; inbound-ref queries deferred (as SL-015 deferred them).
- **TUI artifact browser** integration — no TUI in Doctrine yet.
- **Auto-generated `backlog.md` summary index** — a derived view; deferred.
- **Backlog lifecycle gating / approval** — `status` hand-edited or flag-set,
  ungated, as slices/ADRs/specs ship today (`entity-model:110`).
- **Spec-Driver corpus importer** — migrating the read-only corpus is later.
- **The `problem` kind** — unless §Q1 awards it a reserved id; default deferred.

## Risks, Assumptions & Open Questions

**Assumptions (carried):**
- `src/entity.rs` admits a new caller with a per-`item_kind` fileset descriptor
  with no engine change — supported by SL-003/006/015. The `item_kind`
  discriminator is structurally the `spec` `product|tech` seam (SL-015) one more
  time; exact API verified in `/design`.
- The `mkdir` reservation primitive scales to per-kind backlog namespaces — same
  primitive slices/ADRs/specs use; backlog is one more caller.

**Risks:**
- **Modelling drift back to four schemas.** The research corpus is seductive and
  Python-shaped; the single-kind + facet decision must hold through `/design`.
  The risk facet (extra fields) is the pressure point — keep it a *facet on one
  kind*, not a second entity.
- **Status-vocab divergence.** Adopting `started`/`closed` (canon) over
  `in-progress`/(none) (corpus) means the corpus's status strings can't be
  imported verbatim — flag for the deferred importer; don't silently re-map.
- **Behaviour-preservation gate.** Extending `src/entity.rs` touches shared
  machinery — existing slice/ADR/spec/memory suites must stay green unchanged.
- **Storage rule vs catch-all.** The corpus leans on a raw-frontmatter hashmap
  for extensibility; Doctrine's storage rule forbids un-typed prose-as-data.
  Per-kind facet sets must be enumerated, not deferred to a bag (Q3).

**Open questions** — all resolved in [`design.md`](design.md): §Q1–Q6 above.

## Verification / Closure Intent

"Done" (v1 subset) is judged by:
- `backlog new <kind>` scaffolds each locked `item_kind`'s fileset via the
  engine, with kind-correct template facets and a reserved `XXX-NNN` id.
- `backlog list` filters by kind/status/tag, hides resolved/closed by default,
  shows them under `--all`.
- `backlog show <ID>` auto-detects kind from the prefix and renders identity +
  kind facets + outbound relations.
- `backlog edit <ID> --status` transitions status atomically and
  edit-preservingly (round-trips without dropping comments / unknown keys).
- The routing table (`boot.md`) + affected skills (`/route`, `/preflight`,
  `/consult`, `/notes`, `/audit`, `/close`) are revised to consult / capture /
  harvest the backlog at the right loop points, citing the work / knowledge /
  decision boundary (membership test) as the arbiter; existing skill behaviour is
  otherwise preserved.
- The whole `backlog_item` model (all `item_kind`s incl. the risk facet, the
  deferred prioritisation/sync/integration layer) is designed and locked
  (`/inquisition`) — the deferred layer shown forward-compatible.
- `install/manifest.toml` + `.gitignore` wired (authored-entity trap closed);
  a created item is `git add`-able.
- Existing slice/ADR/spec/memory suites green **unchanged** (behaviour gate).
- `cargo clippy` zero warnings (bins/lib); `just check` clean.
- TDD red/green/refactor throughout.

## Follow-Ups

- **Priority registry + interactive prioritise** — the ordering layer; head-tail
  partition merge. The prime backlog follow-up.
- **`slice new --from-backlog <ID>`** — the capture→scope bridge into the
  spec-driver loop.
- **`sync`** — registry↔filesystem reconciliation; pairs with the relation-index
  cache (the scale-gated half SL-015 also deferred).
- **Reverse-reference scan** — inbound refs in `show`, once the registry surface
  lands.
- **Auto-generated `backlog.md` summary index** (derived view).
- **Spec-Driver backlog corpus importer** — status re-mapping included.
- **The `problem` kind** — if it earns a reserved id (Q1).
- **Backlog lifecycle transitions / approval** — pairs with the absent
  slice-lifecycle transition gap (CLAUDE.md known gaps).
