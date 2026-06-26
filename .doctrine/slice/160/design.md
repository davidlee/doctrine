# Design SL-160: Replace CON (constraint) with INV (invariant)

> **Status: scoped, design not yet run.** This doc carries the technical specifics
> carved out of SL-159's locked design so nothing is lost, plus the **one open
> design question** that motivated the split. A real `/design` pass must resolve
> §2 before this locks. Decisions below marked *(carried)* were settled in SL-159's
> context; re-validate them here.

## 1. The rename (carried, mechanical)

`RecordKind::Constraint → Invariant`; prefix `CON → INV` (CON retired, **not**
recycled — RFC-009 D4); `CONSTRAINT_KIND → INVARIANT_KIND` (dir `…/invariant`).
`kinds::RECORD` swaps `CON → INV`. `integrity::KINDS` row rename + advisory pin
update. Faithful rename; the existing record suites are the behaviour-preservation
proof (adjusted for the rename, never broken).

Facet: `ConstraintFacet → InvariantFacet`; `ConstraintSource → InvariantSource`
(variants kept). `statement, source, applies_to` unchanged.

## 2. OPEN — the `waived → relaxed` semantic question (reason for the split)

CON's vocab is `active, waived, superseded, retired`. The naive rename gives INV
`active, relaxed, superseded, retired`. **Is `relaxed` the right frame?**

- A *constraint* is "a boundary that must not be crossed" — `waived` = "we granted
  an exception to the boundary." Reads clean.
- An *invariant* is "a property that must hold." What is the dual of waiving? An
  invariant that no longer holds is **violated**, not "relaxed." "Relaxed" implies
  the *requirement* was loosened, not that the property failed. These are different
  events: deliberately loosening the rule vs the property being broken.

Candidate framings to weigh in design:
- **(a) `relaxed`** — the rule was deliberately loosened (closest to `waived`).
- **(b) `retired`/`superseded` only** — an invariant either holds or is replaced;
  no "exception" state (drop the waiver concept entirely for INV).
- **(c) a violation/exception model** — distinct from supersession; possibly an
  EVD `disputes INV` edge (SL-159) captures "evidence the property was violated"
  *instead of* a status, making a waiver status redundant.

Note (c) couples to SL-159's `supports`/`disputes`: once EVD can `disputes` an INV,
the "property violated" signal may live on the edge, not the status vocab. Resolve
this interaction before committing the vocab.

## 3. Seed migration (carried, recreate-not-migrate — SL-159 D6)

CON-001 is a disposable one-record seed. Delete the `constraint/` tree; re-mint
INV-001 fresh from the new `knowledge-invariant.toml` template (`active`, same
statement). Re-point **two live citations** `CON-001 → INV-001`: `adr-017.md:21,67`,
`knowledge/question/001/record-001.md:26`. Historical / closed-context prose
(`slice/097`, `rfc/003`, `rfc/008`, `rfc/009`) left as past-state narrative — no
corpus-wide dangler gate fires (`scan_danglers`, `integrity.rs:546`, only on
explicit `reseat`).

## 4. Touch-sites (carried — the ~17 hardcoded prefix sites)

See `mem.pattern.doctrine.record-kind-touch-sites` for the verified list. The
**panic-grade** one: `src/catalog/scan.rs:62` dispatch arm — the `"CON"` literal
becomes `"INV"`; the fallthrough is `debug_assert!(false)` (`:88`). Plus
`catalog/test_helpers.rs`, `dep_seq.rs` (`:29,:83,:267,:273,:285`),
`priority/partition.rs:609`, `search.rs:33,:38`, `tag.rs:17`, `integrity.rs:817`,
`relation.rs` (`:1422,:1427,:1444,:1445` + test pins `:1751,:1783`),
`relation_graph.rs`, `supersede.rs` + `commands/supersede.rs`; template rename;
docs; shipped memory; e2e goldens.

## 5. Verification intent (carried)

CON fully retired (no `Constraint` authorable, no `/constraint` tree); INV in its
place with the resolved §2 vocab; seed migrated; search/tag/templates/docs/goldens
coherent; `just gate` green; existing record suites green post-rename. Grep
`Constraint|CONSTRAINT|"CON"|kinds::CON|/constraint|waived` to zero before close.

## 6. Sequencing

`after SL-159`. SL-159 (EVD/HYP) lands first and appends EVD/HYP to every hardcoded
prefix site; this slice rebases on that and applies the CON→INV rename to the same
lines. Serial — no parallel edits. The Revision (governance axis) coordinates with
SL-159's catalog Revision.
