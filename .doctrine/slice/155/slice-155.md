# Relation CLI UX fixes + revision list with tag surfacing

## Context

Spun off from IMP-170 (UX review of relation-authoring CLI surfaces). IMP-170's
preflight identified three clusters of gaps. This slice picks off the lowest-risk
cluster (A — code-level fixes) plus one list-view gap (L1 — new `revision list`
verb) with tag surfacing from IMP-144.

SL-153 (CLI verbs for spec-internal edges) closed the last hand-edit-only edges.
IMP-170 surfaced remaining display inconsistencies across `show` and `list`
surfaces. This slice addresses the actionable subset.

## Scope & Objectives

### C1 — RELATION_RULES parent row missing PRD source

`src/relation.rs` ~line 408: the `Parent` rule declares `sources: &[SPEC]`.
Product specs use `parent` too (SL-065, SL-153). Add `PRD` to `sources` and
update `target` to `TargetSpec::Kinds(&[SPEC, PRD])`.

### C2 — Template staleness (3 items)

1. `install/templates/spec-tech.toml` line 19: parent comment says "tech-only" —
   stale since SL-153 (now subtype-aware). Change to "Single decomposition parent
   (SPEC-NNN or PRD-NNN); subtype-aware, single-valued outbound."
2. `install/templates/spec-product.toml`: no `parent` example. Add a commented-out
   `parent` example.
3. `install/templates/interactions.toml` line 3: "hand-authored in v1 (no verb)" —
   stale since SL-153 shipped `spec interactions add/remove`. Change to "Edges are
   authorable via `doctrine spec interactions add`."

### C3 — Stale doc comment in Spec.parent

`src/spec.rs` ~line 746: `/// Single decomposition parent (\`SPEC-NNN\`). Tech-only, …`
Remove "Tech-only" — product specs use it too (SL-065, SL-153).

### G5 — ADR template misdirects supersede authoring

`install/templates/adr.toml` line 7: instructs users to author supersedes via
`doctrine link` — but governance supersedes has `LinkPolicy::LifecycleOnly`, so
`link` refuses it. Fix the template comment to reference `doctrine supersede`.

Also: no governance entity in the corpus has a `[[relation]] label = "supersedes"`
row. Author the canonical edge with `doctrine supersede ADR-012 ADR-004`.

### L1 — New `revision list` verb with tag surfacing

Revision (REV) has no `list` subcommand. Add one:
- **`RevisionCommand::List`** variant with `CommonListArgs`, `--tag` filter,
  `--columns` support
- **`run_list`** function: read all `revision-NNN.toml` files, project to
  `RevRow` (id, status, approval, title, tags), render table
- **`rev_scaffold`** / **`render_revision_toml`**: add `tags = []` to the
  template (IMP-144 — tags must be authorable and visible)
- **`RevDoc`**: add `tags: Vec<String>` field (IMP-144 — tags must be read
  from authored TOML)

Column set mirrors `GOV_COLUMNS`: `id, status, approval, title` default
`id, status, approval, title`. `tags` column available via `--columns tags`
but not in default visible set (following governance precedent — tags aren't
a default column there either, but revisable via `--columns`).

### IMP-144 tag surfacing for revision

The `doctrine tag` verb is generic — IMP-144's taggable set includes REV. This
slice adds the `tags` axis to revision's read surfaces:
- `revision list` supports `--tag` filtering, matching against the authored
  `tags` field
- `revision show` (future work in IMP-170 G2) renders `tags` in the
  relationships block

## Non-Goals

- Not adding `link` writability for `parent`/`descends_from` — they stay
  `TypedVerbOnly`
- Not changing the storage shape of any field
- Not fixing G1-G7 (show surface gaps) — those are IMP-170's remaining clusters
- Not wiring tag surfaces for concept-map/review/REC (those are IMP-144's scope;
  their list views are a separate follow-up)
- Not adding relation columns to list views beyond L1 (those surface in
  IMP-170 L2-L7)

## Affected surface

- `src/relation.rs` — C1 (one row edit)
- `install/templates/spec-tech.toml` — C2a (one comment fix)
- `install/templates/spec-product.toml` — C2b (one example add)
- `install/templates/interactions.toml` — C2c (one comment fix)
- `src/spec.rs` — C3 (one doc comment fix)
- `src/revision.rs` — new `RevisionCommand::List` + `run_list` + `RevDoc` tags
  field + `render_revision_toml` template tags
- `install/templates/adr.toml` — G5 (one comment fix)
- `install/templates/spec-tech.toml` — C2 (one comment fix)
- `install/templates/spec-product.toml` — C2 (one example add)
- `install/templates/interactions.toml` — C2 (one comment fix)
- `install/templates/revision.toml` — add `tags = []` line
- `.doctrine/revision/` — run `doctrine supersede ADR-012 ADR-004` (G5 edge
  authoring)

## Verification

- `just gate` must stay green — no behaviour changes in existing list/render
  paths
- Existing revision tests (in `src/revision.rs`) must stay green unchanged
- `cargo test` must pass (including any new `revision` test)
- `just gate` (clippy zero warnings) must pass

## Summary

Six small, targeted fixes + one new CLI verb. Each follows the established
pattern: edit-preserving, idempotent, forward-validated. No new architectural
decisions or ADRs.

## Follow-Ups

- IMP-170 clusters B (show surface gaps) and C (list view gaps)
