# Drift ledger specification — design note

**Status: deferred. No action now.** This specifies the artefact shape so it can
land *without restructuring* once the things it tracks exist. A drift ledger
records mismatches between a spec and what shipped (or between two
representations of the same fact); doctrine has no spec or relationship
registry yet, so there is **nothing to drift against**. Same boundary as
[relation-index](relation-index.md): record the design, build when the caller
arrives.

## Overview

A **drift ledger** is an append-mostly record of *deferred reconciliations*.
When a sweep, migration, or audit surfaces a mismatch that can't (or shouldn't)
be fixed inline, the ledger captures it: what drifted, the evidence, the
proposed resolution, and whether it's been actioned. It is the durable backlog
of "we noticed this is inconsistent; here's the decision and its state."

It is the doctrine form of spec-driver's drift ledger (e.g. `DL-048`). The
shape is the same in spirit — a document of entries — but the **data/prose
split** is corrected (§ The split).

### Drift ledger vs slice audit

These are distinct and both belong:

- A **slice audit** (`AUD-`, glossary) reconciles *one slice's* shipped code
  against *its own* declared scope — a lifecycle stage keyed to a slice id.
- A **drift ledger** is *cross-cutting and standalone* — it collects mismatches
  surfaced by a sweep across many entities, each entry pointing at a different
  target. It outlives any single slice.

## The split

spec-driver's ledger entries are fenced YAML blocks that co-locate two
different things:

- **queryable facets** — `entry_type`, `status`, `severity`, `assessment`,
  `resolution_path`, `owner`, `affected_artifacts`, and the typed evidence
  substructures (`sources`, `claims`, `evidence`, `discovered_by`);
- **human narrative** — `analysis` (plus the freeform markdown the canonical
  model keeps after the fence), markdown prose wearing a YAML block-scalar
  costume.

Co-locating them is *why* that format needs YAML (block scalars are the one
thing YAML does best) and gives the worst of both: prose that doesn't render or
diff as markdown, data buried in document syntax, and a hard YAML dependency.

doctrine's sister-file discipline ([slices-spec](slices-spec.md) § Metadata)
splits them:

- **`drift-<id>.toml`** — facets only, as an array of tables. Machine-read,
  schema-checked, queryable.
- **`drift-<id>.md`** — the narrative. One `###` subsection per entry; analysis
  and recommendation are *actual markdown* where they belong and diff cleanly.

The two are joined by a stable per-entry `id` (`DL-001.003`). The data is
**wide and prose-heavy, not deep** — once the narrative is lifted out, the
canonical entry's typed substructures (`sources`, `claims`, …) are flat
arrays-of-tables; TOML carries them comfortably, no YAML, no RON.

## On-disk structure

Identical to a slice — a numeric directory under `.doctrine/drift/` with a slug
symlink, reusing the slice machinery wholesale:

```
.doctrine/drift/
  001/
    drift-001.toml
    drift-001.md
  001-de138-fm-block-reconciliation -> 001
```

Ids are allocated by the **same local reservation** as slices
([reservation-spec](reservation-spec.md): the `mkdir` claim), under namespace
`drift/id/<n>`. The reservation primitive is already kind-agnostic; the
directory-entity scan/claim/scaffold code from `doctrine slice` generalises over
the kind — drift is its second caller, not a parallel implementation.

## Metadata (`drift-<id>.toml`)

```toml
id = 1
slug = "de138-fm-block-reconciliation"
title = "DE-138 sweep FM/block reconciliation"
status = "open"                 # open | closed
created = "2026-06-03"
updated = "2026-06-03"
slice_ref = ""                  # optional owning slice/delta (canonical: delta_ref)
anchor = "2afc0833"             # optional: commit/tag the sweep ran against

[[entry]]
id = "DL-001.001"               # entry key (canonical field is `id`, not `ref`)
title = "DE-016: PROD-011 absent from relationships block"
entry_type = "stale_claim"      # singular — one of the 5-value vocab
severity = "significant"        # blocking | significant | cosmetic
status = "open"                 # 7-value entry vocab (see § Lifecycle)
assessment = "confirmed"        # confirmed | disputed | deferred | not_drift
resolution_path = "editorial"   # ADR | DE | RE | backlog | editorial | no_change
resolution_ref = ""
owner = "unassigned"
affected_artifacts = ["DE-016"]
evidence = ["block carries only SPEC-* targets; FM names PROD-011"]

  [[entry.source]]              # typed Source: kind + ref (+ note)
  kind = "spec"
  ref = "PROD-011"
  note = "named in FM, absent from block"

  [[entry.claim]]               # typed Claim: kind + text (+ label)
  kind = "observation"
  text = "derived scope understates the delta's PROD-level reach"

  [entry.discovered_by]         # typed DiscoveredBy: kind (+ ref)
  kind = "sweep"
  ref = "DE-138"
```

- `[[entry]]` is an array of tables; the canonical typed substructures
  (`source`, `claim`, `discovered_by`) nest as further `[[entry.*]]` / `[entry.*]`
  tables — append entries without reshaping.
- Fields track the canonical `DriftEntry` model (§ Serde types). The **legacy
  minimal variant** (`target` / `drift_kind` / `disposition` / `detail`) maps in:
  `drift_kind → entry_type`, `disposition → assessment` + `resolution_path`,
  `target → affected_artifacts` / a `source`.
- **Progressive strictness, permissive vocab.** Only `id` + `title` are required;
  every other field defaults empty, unknown keys land in an `extra` catch-all, and
  unknown *vocabulary values* warn rather than reject (canonical DEC-057-08). This
  is what `extra` replaces the old open `observed` map with.
- No `analysis` key — the narrative is prose (§ Prose body).

## Prose body (`drift-<id>.md`)

Pure prose. A short ledger preamble, then one `###` per entry keyed by entry
`id`, carrying the narrative the TOML deliberately omits (the canonical
`analysis`, plus a recommendation the `resolution_path` facet only names):

```markdown
# DL-001 — DE-138 sweep FM/block reconciliation

Tracks FM ↔ relationships-block mismatches surfaced by the DE-138 sweep.
Each entry is a deferred reconciliation pending review.

## Entries

### DL-001.001 — DE-016: PROD-011 absent from relationships block

**Analysis.** DE-016 predates the block schema; its FM named the parent PROD
but the block only carried SPEC-* targets, so derived scope now understates
the delta's PROD-level reach.

**Recommendation.** Add PROD-011 to `specs.collaborators` in DE-016's block
(preserves the block as canonical; restores PROD-level visibility).
```

`doctrine drift new` scaffolds the preamble + `## Entries` heading; `doctrine drift
add` appends a `### <id>` stub and the matching `[[entry]]` row in one step
(so the two never drift apart — the ledger must not itself drift).

## Serde types

Mirrors the canonical `DriftEntry` / `DriftLedger` pydantic models. Vocabularies
are **permissive** — known values lead, unknown ones warn and are kept verbatim,
never a parse error (canonical DEC-057-08). So the enums are *soft*: each pairs a
known set with an `Other(String)` arm rather than failing on an unrecognised
value. The typed substructures replace the old open `observed` map; genuinely
unknown keys fall into `extra`.

```rust
#[derive(Debug, Deserialize, Serialize)]
struct Ledger {
    id: u32,
    slug: String,
    title: String,                 // canonical `name`
    status: LedgerStatus,          // open | closed
    created: String,
    updated: String,
    #[serde(default)]
    slice_ref: Option<String>,     // canonical `delta_ref` — owning change, optional
    #[serde(default)]
    anchor: Option<String>,
    #[serde(default, rename = "entry")]
    entries: Vec<DriftEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DriftEntry {
    id: String,                    // "DL-001.003" — required (with title)
    title: String,
    #[serde(default)]
    entry_type: EntryType,         // singular, soft enum
    #[serde(default)]
    severity: Severity,
    #[serde(default)]
    status: EntryStatus,
    #[serde(default)]
    assessment: Assessment,
    #[serde(default)]
    resolution_path: ResolutionPath,
    #[serde(default)]
    resolution_ref: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    affected_artifacts: Vec<String>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default, rename = "source")]
    sources: Vec<Source>,          // kind + ref (+ note)
    #[serde(default, rename = "claim")]
    claims: Vec<Claim>,            // kind + text (+ label)
    #[serde(default)]
    discovered_by: Option<DiscoveredBy>,   // kind (+ ref)
    /// Unknown keys (incl. the legacy minimal variant's `target`/`drift_kind`/
    /// `disposition`/`detail`) — kept verbatim, never fatal.
    #[serde(flatten)]
    extra: BTreeMap<String, toml::Value>,
}
```

Each vocabulary enum (`EntryType`, `Severity`, `EntryStatus`, `Assessment`,
`ResolutionPath`) carries its known set (§ Lifecycle) plus an `Other(String)`
arm, so an unrecognised value from an external or version-skewed detector
degrades to a single warned row, never a hard parse error that would take down
`doctrine drift list` for the *whole* ledger. `analysis` is **not** a field here —
it is prose (§ Prose body), consistent with the canonical model keeping the long
narrative as freeform markdown after the fence.

## Lifecycle

Values are the canonical drift vocabularies (permissive — unknown warns, § Serde
types):

| Vocabulary | Values | Meaning |
|---|---|---|
| `LedgerStatus` | `open` → `closed` | `closed` once every entry is resolved or dismissed. |
| `EntryStatus` | `open`, `triaged`, `adjudicated`, `resolved`, `deferred`, `dismissed`, `superseded` | progress of one reconciliation. |
| `EntryType` | `ambiguous_intent`, `contradiction`, `implementation_drift`, `missing_decision`, `stale_claim` | *what kind* of drift (singular per entry). |
| `Severity` | `blocking`, `significant`, `cosmetic` | how much it matters. |
| `Assessment` | `confirmed`, `disputed`, `deferred`, `not_drift` | the *verdict*: is it real drift? |
| `ResolutionPath` | `ADR`, `DE`, `RE`, `backlog`, `editorial`, `no_change` | *where it goes* to be fixed. |

`status` is the state; `assessment` + `resolution_path` together are the
decision (the old single `disposition` is the legacy minimal-variant form, now
split — § Metadata). All recorded and advanced by hand in v1 — **no gate**, same
as slices. There is nothing to enforce against until the spec system exists.

## Detection

A drift ledger is *written by* a detector — a sweep that compares two sources of
truth and emits entries. That detector is **out of scope here and now**: it
needs the relationship registry to compare against. v1 (whenever it lands) ships
only the *artefact and its CLI* (`new` / `add` / `list`), authored by hand or by
an external sweep; automated detection arrives with the spec system.

## Out of scope

- **The detector / sweep.** No registry to diff yet.
- **Resolution automation** (auto-applying a `resolution_path` back into a
  block). Manual; the ledger records the decision, a human or a later cleanup
  step enacts it.
- **Evidence beyond the canonical substructures.** `sources`/`claims`/`evidence`
  are typed (§ Serde types); anything the spec system later adds lands in `extra`
  until promoted to a field.
- **Cross-ledger queries / dedup.** Same scale argument as
  [relation-index](relation-index.md) — not needed yet.

## Known risks

- **Ledger self-drift.** The `.toml` row and the `.md` `###` for one entry can
  fall out of sync if edited by hand. Mitigation: `doctrine drift add` writes both
  atomically, and `doctrine drift list` warns on an `id` present in one file but
  not the other. A linter, not a hard gate, in v1. The atomic add must append
  **edit-preservingly** (`toml_edit` / structured append), not by serde-
  reserialising the whole `Ledger` — a full reserialize drops hand comments and
  any `extra` keys (spec-entity-spec § Known risks carries the same caveat for
  its mutating verbs).
- **Duplicate entry `id` across concurrent adds / merges.** Entry `id`s are
  sequence-assigned (`max + 1` within the ledger), so two `doctrine drift add`
  invocations on separate branches both mint `DL-001.004` and a clean git merge
  produces silent duplicates. A `mkdir`-style claim cannot arbitrate a *row* (rows
  aren't dirs), so prevention is impossible — detection is the lever: the
  uniqueness check runs at **load over the merged file**, not at write time against
  pre-add state, and a collision is a **hard** lint, not a warning.
- **Unknown vocabulary value from external sweeps.** Any soft enum
  (`entry_type`, `assessment`, …) degrades to `Other(String)` — a warned row, not
  a dead file (§ Serde types). The known sets must still lead the detector so
  skew is visible.
- **Distributed id collision.** Inherited from the shared reservation primitive;
  closed later by the `git-ref` backend (reservation-spec § Known risks).

## Testing

Pure layer, mirroring [slices-spec](slices-spec.md) § Testing:

- `drift-<id>.toml` round-trip — render → parse → same facets; `extra` keys
  preserved; an unknown vocabulary value parses to `Other` (not an error); the
  legacy minimal variant (`target`/`drift_kind`/`disposition`) parses, its keys
  landing in `extra`.
- Entry append — a new `[[entry]]` row and its `### <id>` stub are produced
  together.
- Duplicate-`id` lint — over a *merged* file (two appends that each chose the
  same `id`), a duplicate is reported as a hard error, not a warning.
- Ledger/entry status formatting and `--status` filter (as for `slice list`).
- Self-drift lint — an `id` in the TOML with no matching `###` (and vice versa)
  is reported.

## Follow-ups

- **Glossary.** Add `drift ledger | DL-001 | y` (governance or a new "audit"
  group) once this leaves deferred.
- **Generalise the slice machinery.** `doctrine slice`'s scan/claim/scaffold and
  the reservation namespace want to be kind-parameterised before drift becomes
  the second caller — otherwise it's a parallel implementation. Track as a
  refactor on the slice code, not a copy.
