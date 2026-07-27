# Design SL-234: Review prime ignores non-file selector entries

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`review prime` treats every path printed by `git ls-files` as readable file
content. Doctrine commits slug symlinks whose targets are entity directories;
glob selectors therefore admit paths that `std::fs::read` cannot hash. The
repair must exclude non-file Git entries without weakening the shared
content-set contract or changing literal-selector absence semantics.

## 2. Current State

`run_prime` obtains the slice selector union, calls
`resolve_selectors_to_fileset`, and hashes the returned paths through
`contentset::compute`. Literal selectors pass through unresolved. Glob
selectors expand over path-only `git ls-files` output, which discards the Git
index mode needed to distinguish regular blobs from symlinks and gitlinks.
`contentset::compute` hashes every supplied path, omits `NotFound`, and
propagates all other I/O failures.

The verified code map is recorded in `research/research.md`.

## 3. Forces & Constraints

- Preserve the literal-selector absence-implies-stale behaviour.
- Keep `contentset` a hash-what-you-are-given leaf under ADR-001.
- Do not follow directory symlinks or make the hash depend on their descendant
  trees.
- Preserve ordinary glob expansion ordering and deduplication.
- Shared review and content-set suites remain the behaviour-preservation gate
  required by SPEC-004.

## 4. Guiding Principles

- Filter at the seam that owns selector-to-fileset conversion.
- Use Git index identity, not fallible working-tree metadata, to decide which
  tracked entries are content files.
- Make the regression resemble Doctrine's own committed slug-symlink shape.

## 5. Proposed Design

### 5.1 System Model

For the first glob selector, load the tracked Git index once as typed entries.
Each entry carries its mode and root-relative path. Retain only regular-blob
modes, then apply the existing glob matcher and `BTreeSet` ordering. Literal
selectors bypass this index expansion exactly as they do today.

### 5.2 Interfaces & Contracts

Change only the private `resolve_selectors_to_fileset` implementation. Invoke
`git ls-files --stage -z` and parse each NUL-delimited record as:

```text
<mode> <object-id> <stage><TAB><path>
```

Accept entries whose mode denotes a regular blob (`100644` or `100755`);
exclude symlinks (`120000`) and gitlinks (`160000`). A malformed record is a
named error rather than silently disappearing. Paths remain arbitrary UTF-8
strings and are not split on spaces.

### 5.3 Data, State & Ownership

No persisted schema changes. The typed tracked-entry representation is private
and ephemeral inside `review`; the review cache continues to store only the
resolved path list and hashes.

### 5.4 Lifecycle, Operations & Dynamics

Prime still reads selectors, resolves the union, acquires the review lock,
hashes the resolved fileset, and atomically writes the runtime cache. The
filter changes only which tracked entries a glob can resolve.

### 5.5 Invariants, Assumptions & Edge Cases

- A literal symlink selector remains literal and therefore may fail during
  hashing. This slice fixes Git-glob expansion over entity roots; it does not
  redefine explicit literal-path semantics.
- An absent literal remains in the fileset so its later appearance is drift.
- A symlink whose target is a regular file is still excluded because the
  review tracks Git blob content paths, not target-following semantics.
- Empty glob matches remain legal and produce no tracked path from that
  selector.

## 6. Open Questions & Unknowns

None. The choice is constrained by the existing ownership seams and Git mode
contract.

## 7. Decisions, Rationale & Alternatives

- **D1 — Filter non-regular entries during glob resolution.** This preserves
  `contentset::compute` as a strict hasher and fixes the defect where entry kind
  is first available.
- **D2 — Parse staged, NUL-delimited Git output.** Path-only output is
  insufficient; newline-delimited staged output is unsafe for legal filenames.
- **Rejected: forgive `IsADirectory` in `contentset::compute`.** It would make a
  shared leaf silently reinterpret an invalid input and still leave other
  non-file entry kinds ambiguous.
- **Rejected: inspect working-tree metadata after path-only listing.** It adds
  one filesystem probe per tracked entry and makes selection depend on target
  resolution rather than committed Git identity.

## 8. Risks & Mitigations

- **R1 — staged-output parser drift.** Pin regular, symlink, and space-bearing
  path records in focused unit coverage; malformed records must error.
- **R2 — accidental literal semantic change.** Retain the literal tests
  unchanged and add the regression through a glob selector.
- **R3 — shared-mechanism regression.** Run the existing review/content-set
  suites unchanged and the project quick check.

## 9. Quality Engineering & Validation

Use TDD:

1. Add a Git-backed review test that commits a directory plus a slug-style
   symlink to it, selects the parent with a glob, and demonstrates the current
   `IsADirectory` failure.
2. Implement typed index parsing and regular-blob filtering; assert prime
   succeeds, hashes the real files, and omits the symlink path.
3. Cover malformed staged output at the parser boundary if the parser is
   extracted as a pure helper.
4. Run focused review tests, existing content-set tests, `doctrine check
   quick`, and finally `doctrine review prime RV-315`.

## 10. Review Notes

The design deliberately changes neither `src/contentset.rs` nor any public CLI
surface. Its exact implementation target is `src/review.rs`; the live RV-315
prime is acceptance evidence, not a unit-test substitute.
