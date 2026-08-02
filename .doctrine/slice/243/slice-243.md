# Spec anchor map

## Context

A tech spec's `[[source]]` anchors are the only machine-readable edge from
governance to code. They are written in one direction only — spec → path — and
nothing inverts them. So the question *"which governed units of this codebase
are anchored, by which spec, and which are dark?"* is answerable only by an
agent running an ad-hoc census by hand.

That cost is measured, not hypothetical. `/spec-coverage-assessment` instructs
its agent to grep the raw TOML for `[[source]]` blocks and `test -e` each
identifier. IMP-381 paid it (a manual anchor harvest, a manual liveness loop,
and a manual correction for the `NNN-slug` symlinks that double every naive
grep). CHR-052 paid it again, hand-mapping nine modules onto SPEC-002.

A ~30-line spike against the existing JSON contract produced, in seconds, what
that manual work could not: **48 specs, 81 anchors, and 27% of non-test `src/`
loc (29,310 loc across 84 files) governed by nothing.** The largest dark module
is `src/review.rs` at 2,824 non-test loc — the RV adversarial review ledger,
ADR-007's entire subject.

The intent for this already exists. **PRD-012** responsibility 5 admits *"both
hand-authoring and import from code structure, converging on one code-anchored
entity per governed unit rather than parallel surfaces"*; its principles fix
*"the code anchor is the single convergence seam"* and *"a stale anchor is an
integrity finding, not a silent inconsistency"*. **REQ-085** anchors a spec to
code and **REQ-088** requires hand-authored and imported specs to converge on
one anchor. What PRD-012 lacks is a requirement for the **report** — which
governed units are anchored, by which spec, and which are dark. This slice
supplies the report; import-from-code-structure is the later capability it
unblocks, already constrained by REQ-088.

This is IMP-295's second axis ("deterministic aids"). Its first axis — the
`/spec-coverage-assessment` skill — has shipped; this is the half that makes the
skill's census mechanical instead of a charge against its judgement budget.

### Why this is a platform feature and not a project script

The work decomposes such that exactly one step is language-specific:

| step | language-specific? |
|---|---|
| read `[[source]]` anchors from the corpus | no |
| liveness probe (identifier resolves) | no |
| inverse index `unit → [spec]`, shared-anchor detection | no |
| **enumerate governed units, sizes, containment, dependency edges** | **yes** |
| join, rank, render | no, given a unit list |

So the engine owns the join and the report; the project owns the inventory,
supplied through a declared adapter command. This is the pattern SPEC-002 already
ships for `[verification]` and RFC-023 names as the POL-002-clean shape: the
engine knows the contract, never the host's conventions.

## Scope & Objectives

**O1 — `doctrine spec anchors`, a report-only read verb.** Joins corpus anchors
against adapter-supplied units; emits a neutral JSON core plus a markdown
rendering. No gate, no ratchet, no write path. (Originally "markdown and d2
renderings" — narrowed by DEC-115; a diagram rendering is IMP-385, and DOT
rather than d2 when it is taken.)

**O2 — the adapter contract.** A project-declared command per anchor `language`,
in `.doctrine/doctrine.toml`, mirroring `[verification]`. Absent ⇒ an owned
no-op, never a guess at the host's layout.

**O3 — the Rust adapter as a workspace member crate.** Doctrine's own adapter,
dogfooding the contract, outside the shipped binary. Emits the module tree with
non-test sizes and dependency edges.

**O4 — new PRD-012 requirements for the report**, and the spec homes for the
engine and the adapter contract.

**O5 — wire the tool into `/spec-coverage-assessment`** as a documented step, so
the skill stops instructing a manual grep.

### Design commitments carried in from the pre-slice discussion

These are settled inputs to `/design`, not open questions:

- **Granularity is the adapter's decision; legibility is the engine's.** The
  adapter emits a *containment tree* of units, not a flat list. The engine does
  subtree rollup, `--depth` collapse, in-degree, and ranking. This is what
  absorbs the fact that "module" is not portable — Go's package-per-directory,
  Rust's `mod` tree, and JavaScript's file-is-a-module (thousands of leaves)
  are all trees at different depths, and only the adapter needs to know which.
- **Subtree semantics are load-bearing** (IMP-316): an anchor on `src/priority/mod.rs`
  covers `src/priority/**`. Stated generically as containment, never as a
  `mod.rs` special case. Getting this wrong emits hundreds of false gaps.
- **Size is opaque to the engine** — a positive number with an adapter-declared
  unit. That is how `#[cfg(test)]` exclusion stays out of the engine, and it
  matters: `src/slice.rs` is 7,251 lines gross and would top every ranking
  forever if test loc counted.
- **A unit carries `path` and an optional `qualified_name`.** Path is the join
  key against `identifier`; qualified name corroborates against `module`. Their
  disagreement is a free rot-detector, and it is the exact class IMP-316's leg 2
  found (SPEC-020 wrote module paths into `identifier`, reading as dead while the
  code was present).
- **The adapter table keys on the existing anchor `language` vocabulary**, so
  multi-language falls out of the design rather than being bolted on. "Rust only
  in v1" is one entry in the table; a language with no adapter is cleanly
  *declared but uninventoried*.
- **The declared command is an argv array, never a string**
  (`mem.pattern.shell.declared-command-word-split-loses-quoting`): a word-split
  config string hands its quote characters to the program as literal bytes, and
  fails silently. `[verification]` already has the right shape — copy it.
- **The JSON emits the full PRD → SPEC → REQ → unit chain, with membership as an
  edge and never as nesting.** REQs are members of more than one spec and move as
  they are negotiated, so nesting requirements inside specs bakes in a 1:N that
  is not true. Key on the durable `REQ-NNN`; carry `(spec, label, order)`
  alongside — the same reason the boot rule forbids citing mobile `FR-`/`NF-`
  labels.
- **Mirror SPEC-027's shape; do not import its code.** `CatalogGraph` projects a
  hydrated entity `Catalog` and by construction has no non-entity nodes. Bending
  it to admit modules would drag host-language concepts into doctrine proper.
  Take the pattern — presentation-neutral core, deterministic byte-stable
  renderers — not the type.
- **Consume the published JSON contract, not `.doctrine/` files.**
  `spec list --json` and `spec show <ID> --json` already emit `source[]`,
  `parent`, `descends_from`, `c4_level`, `product_level`, and `members[]`. This
  honours the read-via-`show` guardrail mechanically and makes the symlink
  double-count structurally impossible.
- **Report-only.** No gate or ratchet in this slice. Doctrine has legitimately
  dark modules, and a premature gate would only be waived.

## Non-Goals

- **Any gate, ratchet, or `doctor` leg.** IMP-316 owns the liveness check as a
  `doctor` leg with a different consumer; this slice *composes* with it and does
  not absorb it. The shared surface is a path-existence probe.
- **Generating or importing specs from code structure.** PRD-012 names it and
  REQ-088 constrains it; this slice supplies the inventory it would need and
  stops there.
- **Semantic accuracy of an anchor.** Whether a spec's prose actually describes
  the module it anchors is review work. The tool reports declaration and
  liveness, never fidelity.
- **The "partial / prose-dark" verdict** from IMP-381's census — an anchored file
  whose capability is undescribed. That is a judgement, not a join.
- **Cross-language AST analysis.** The adapter is a declared command; doctrine
  never parses a host language.
- **Non-Rust adapters.** The contract must admit them; only the Rust adapter is
  built here.
- **REQs in the actionability graph.** Estimation, value, and slice sequencing
  driven by requirements is the ambition this unblocks, not this slice's work.
  It is wiring REQ into existing machinery (PRD-011/SPEC-001 priority,
  PRD-014/SPEC-020 estimation, SPEC-024 comparison) and belongs elsewhere.
- **Anchors on ADRs, policies, or standards.** `[[source]]` is tech-spec-only
  (SPEC-017). That POL-002 governs code with no way to anchor to it is a real
  limitation and not this slice's problem.

## Summary

Invert the spec→code anchor edge into a governed-unit map: a language-agnostic
join and report in the engine, fed by a project-declared inventory adapter, with
Rust as the first adapter and doctrine's own corpus as the proving ground.

Closure intent: `doctrine spec anchors` reproduces the spike's headline figures
from the shipped path (48 specs / 81 anchors / 27% dark), renders markdown that
drops into `/spec-coverage-assessment`'s artifact tables without translation, and
the skill cites the verb instead of a manual grep. Baseline to beat is recorded
in Context.

## Follow-Ups

- Focus-scoped **altitude view** — PRD product altitude → SPEC C4 ladder → units,
  for one focused subgraph. Both `product_level` and `c4_level` already ride the
  JSON envelope, and SPEC-027 establishes the ego-view shape
  (`neighbourhood(focus, depth)`, `drop_isolated`). A rendering mode over the
  same core, never the default render — the whole-corpus graph does not fit on a
  page. Carried as a design input, deferred as work.
- **Diagram rendering (IMP-385)** — DOT, naturally the same work as the altitude
  view above, since that view is what makes a diagram page-sized. IDE-046 is the
  reason DOT is the interesting target: rendering to an image and writing it
  inline via the terminal graphics protocol.
- Publishing the Rust adapter as a reference implementation for other Rust
  projects.
- A gate or ratchet over dark loc, once the report has a baseline anyone trusts.
- Anchor-integrity as a first-class finding, per PRD-012's principle — the
  natural join with IMP-316 once both have shipped.
