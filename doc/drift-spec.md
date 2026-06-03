# Drift ledger specification — design note

**Status: deferred. No action now.** This specifies the artefact shape so it can
land *without restructuring* once the things it tracks exist. A drift ledger
records mismatches between a spec and what shipped (or between two
representations of the same fact); Heresiarch has no spec or relationship
registry yet, so there is **nothing to drift against**. Same boundary as
[relation-index](relation-index.md): record the design, build when the caller
arrives.

## Overview

A **drift ledger** is an append-mostly record of *deferred reconciliations*.
When a sweep, migration, or audit surfaces a mismatch that can't (or shouldn't)
be fixed inline, the ledger captures it: what drifted, the evidence, the
proposed resolution, and whether it's been actioned. It is the durable backlog
of "we noticed this is inconsistent; here's the decision and its state."

It is the Heresiarch form of spec-driver's drift ledger (e.g. `DL-048`). The
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

- **queryable facets** — `target`, `drift_kind`, `disposition`, `owner`,
  `status`, the `observed:` evidence;
- **human narrative** — `analysis: |` and `recommendation: |`, which are
  markdown prose wearing a YAML block-scalar costume.

Co-locating them is *why* that format needs YAML (block scalars are the one
thing YAML does best) and gives the worst of both: prose that doesn't render or
diff as markdown, data buried in document syntax, and a hard YAML dependency.

Heresiarch's sister-file discipline ([slices-spec](slices-spec.md) § Metadata)
splits them:

- **`drift-<id>.toml`** — facets only, as an array of tables. Machine-read,
  schema-checked, queryable.
- **`drift-<id>.md`** — the narrative. One `###` subsection per entry; analysis
  and recommendation are *actual markdown* where they belong and diff cleanly.

The two are joined by a stable per-entry `ref` (`DL-001.003`). The data is
**wide and prose-heavy, not deep** — once the prose is lifted out, TOML
arrays-of-tables and dotted keys carry the rest comfortably; no YAML, no RON.

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
directory-entity scan/claim/scaffold code from `heresy slice` generalises over
the kind — drift is its second caller, not a parallel implementation.

## Metadata (`drift-<id>.toml`)

```toml
id = 1
slug = "de138-fm-block-reconciliation"
title = "DE-138 sweep FM/block reconciliation"
status = "open"                 # open | closed
created = "2026-06-03"
updated = "2026-06-03"
anchor = "2afc0833"             # optional: commit/tag the sweep ran against

[[entry]]
ref = "DL-001.001"
target = "DE-016"               # the entity that drifted
kinds = ["fm_specs_unmatched"]
disposition = "amend"
status = "open"
owner = "unassigned"
# evidence: free-keyed because the facets depend on the (future) spec system.
observed.fm_applies_to_specs = ["PROD-011"]
observed.block_specs_primary = []

[[entry]]
ref = "DL-001.002"
target = "DE-020"
kinds = ["fm_specs_unmatched", "fm_requirements_unmatched"]
disposition = "amend"
status = "open"
observed.fm_applies_to_specs = ["PROD-010", "SPEC-110", "SPEC-113"]
observed.fm_applies_to_requirements = ["ISSUE-025"]
```

- `[[entry]]` is an array of tables — append entries without reshaping.
- `observed.*` is `<facet> → [ids]`; the **keys are open** (a
  `BTreeMap<String, Vec<String>>`) because what counts as evidence is defined by
  the spec/relationship system that doesn't exist yet. Everything else is
  closed and validated.
- No `analysis` / `recommendation` keys — those are prose (§ Prose body).

## Prose body (`drift-<id>.md`)

Pure prose. A short ledger preamble, then one `###` per entry keyed by `ref`,
each with the same two narrative facets the TOML deliberately omits:

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

`heresy drift new` scaffolds the preamble + `## Entries` heading; `heresy drift
add` appends a `### <ref>` stub and the matching `[[entry]]` row in one step
(so the two never drift apart — the ledger must not itself drift).

## Serde types

The facet half maps to closed types; the evidence half stays open.

```rust
#[derive(Debug, Deserialize, Serialize)]
struct Ledger {
    id: u32,
    slug: String,
    title: String,
    status: LedgerStatus,
    created: String,
    updated: String,
    #[serde(default)]
    anchor: Option<String>,
    #[serde(default, rename = "entry")]
    entries: Vec<DriftEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DriftEntry {
    #[serde(rename = "ref")]
    reference: String,
    target: String,
    kinds: Vec<DriftKind>,
    disposition: Disposition,
    status: EntryStatus,
    #[serde(default)]
    owner: Option<String>,
    /// Evidence facets: open-keyed, values are id lists. The keys are defined
    /// by the spec system, not by this type.
    #[serde(default)]
    observed: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum DriftKind {
    FmSpecsUnmatched,
    FmRequirementsUnmatched,
    RequirementUnparseable,
    DescriptionPlaceholder,
    AcceptancePlaceholder,
    BodyRiskNarrative,
}
```

`DriftKind` is a **closed enum on purpose**: kinds are *emitted by a drift
detector*, not hand-authored, so the detector and the enum evolve together and
an unknown kind is a parse error (a typo or a detector/schema version skew),
not silently accepted. The seed set above is lifted from the spec-driver sweep
that motivated this — it grows with Heresiarch's own spec system. `Disposition`
(`amend | accept | defer | dismiss`) and the status enums are closed for the
same reason: they are a fixed decision vocabulary, not data.

## Lifecycle

| Vocabulary | Values | Meaning |
|---|---|---|
| `LedgerStatus` | `open` → `closed` | `closed` once every entry is resolved or dismissed. |
| `EntryStatus` | `open`, `resolved`, `dismissed`, `deferred` | progress of one reconciliation. |
| `Disposition` | `amend`, `accept`, `defer`, `dismiss` | the *decision*: fix it / accept the drift as legitimate / punt / it's a non-issue. |

`status` is the state; `disposition` is the chosen path. Both are recorded and
advanced by hand in v1 — **no gate**, same as slices. There is nothing to
enforce against until the spec system exists.

## Detection

A drift ledger is *written by* a detector — a sweep that compares two sources of
truth and emits entries. That detector is **out of scope here and now**: it
needs the relationship registry to compare against. v1 (whenever it lands) ships
only the *artefact and its CLI* (`new` / `add` / `list`), authored by hand or by
an external sweep; automated detection arrives with the spec system.

## Out of scope

- **The detector / sweep.** No registry to diff yet.
- **Resolution automation** (auto-applying an `amend` back into a block). Manual;
  the ledger records the decision, a human or a later cleanup step enacts it.
- **`observed` key schema.** Deliberately open until the spec system defines what
  evidence exists.
- **Cross-ledger queries / dedup.** Same scale argument as
  [relation-index](relation-index.md) — not needed yet.

## Known risks

- **Ledger self-drift.** The `.toml` row and the `.md` `###` for one entry can
  fall out of sync if edited by hand. Mitigation: `heresy drift add` writes both
  atomically, and `heresy drift list` warns on a `ref` present in one file but
  not the other. A linter, not a hard gate, in v1.
- **Closed `DriftKind` vs external sweeps.** An external detector emitting a kind
  the enum doesn't know fails to parse. Intended (it surfaces version skew) but
  means the enum must lead the detector. Accepted.
- **Distributed id collision.** Inherited from the shared reservation primitive;
  closed later by the `git-ref` backend (reservation-spec § Known risks).

## Testing

Pure layer, mirroring [slices-spec](slices-spec.md) § Testing:

- `drift-<id>.toml` round-trip — render → parse → same facets; unknown `observed`
  keys preserved; an unknown `DriftKind` is rejected.
- Entry append — a new `[[entry]]` row and its `### <ref>` stub are produced
  together; `ref`s are unique within a ledger.
- Ledger/entry status formatting and `--status` filter (as for `slice list`).
- Self-drift lint — a `ref` in the TOML with no matching `###` (and vice versa)
  is reported.

## Follow-ups

- **Glossary.** Add `drift ledger | DL-001 | y` (governance or a new "audit"
  group) once this leaves deferred.
- **Generalise the slice machinery.** `heresy slice`'s scan/claim/scaffold and
  the reservation namespace want to be kind-parameterised before drift becomes
  the second caller — otherwise it's a parallel implementation. Track as a
  refactor on the slice code, not a copy.
