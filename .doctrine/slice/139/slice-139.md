# Uniform entity show and file path surfaces

## Context

Doctrine's CLI contract is meant to be predictable across authored entity kinds: a user should be able to ask a kind to `show` an entity, receive the readable Markdown reconstruction for that entity, and discover the authored files backing the entity without knowing the storage layout.

Today that contract is uneven. Some kinds already have a `show` surface and JSON output, but the shape is kind-specific; file paths are not uniformly surfaced; and concept maps are an authored entity command whose show surface has drifted from the common JSON shorthand.

This slice scopes the cross-kind read-surface normalization. It is governed by SPEC-013's uniform CLI grammar, SPEC-004's entity storage rule, SPEC-014's slice surface, ADR-001's layering boundary, and ADR-005's read-via-`show` discipline.

Related but not governing backlog context: IMP-133 and IMP-135 cover broader CLI UX/help consistency; IMP-125 covers per-kind reference parsing consolidation. This slice is narrower: entity `show` parity and file path exposure.

## Scope & Objectives

- Define the entity-command coverage matrix for `show` parity:
  - in-scope authored entity commands: `adr`, `policy`, `standard`, `rfc`, `spec`, `backlog`, `knowledge`, `slice`, `memory`, `review`, `rec`, `revision`, and `concept-map`;
  - keep non-entity read surfaces (`inspect`, `survey`, `next`, `blockers`, `explain`, `coverage`, `status`) out.
- Ensure every in-scope entity command has a `show` verb that renders the entity's readable Markdown/prose reconstruction plus embedded child/facet content where that kind owns it (for example spec members/requirements, review findings/briefs, or similar kind-specific embedded material).
- Add `-f` / `--filepaths` to in-scope `show` commands as a human paths-only mode. It prints the root-relative path to the primary entity file first, followed by the other regular files in the entity's authored folder in deterministic order.
- Add top-level `filepaths` data to `show --json` output by default, using the same primary-first ordering. Add `-n` / `--no-filepaths` so JSON callers can suppress those paths.
- Prefer a shared show/file-set helper or argument bundle over per-kind bespoke implementations, preserving ADR-001 layering: pure path selection/projection below command-shell clap plumbing; filesystem access only at the shell/seam that already loads the entity.
- Keep existing show JSON payloads faithful to the entity data while adding the file path field in a controlled, documented place; update goldens/conformance accordingly.
- Keep the broader `info` / summary read-surface question out of scope; it is captured as follow-up IMP-145.

## Non-Goals

- Do not change authored storage layout or move entity files.
- Do not add file path output to `list` commands.
- Do not redesign relation inspection; `inspect` remains the relation graph read surface.
- Do not rewrite every kind's show renderer if a shared projection seam can be adapted.
- Do not include runtime `.doctrine/state/` phase files, symlink aliases, or subdirectories in the authored file path list.
- Do not add a slice sibling/current/details read surface here; expose existing authored sibling files only as regular files when they sit in the slice folder.

## Affected Surface

- CLI definitions and dispatch: `src/commands/cli.rs`; per-kind command modules in `src/adr.rs`, `src/policy.rs`, `src/standard.rs`, `src/rfc.rs`, `src/spec.rs`, `src/backlog.rs`, `src/knowledge.rs`, `src/slice.rs`, `src/memory.rs`, `src/review.rs`, `src/rec.rs`, `src/revision.rs`, and `src/concept_map.rs`.
- Shared read/projection seams likely involved: `src/entity.rs`, `src/meta.rs`, `src/governance.rs`, `src/listing.rs`, and any existing show JSON helpers in the per-kind modules.
- Tests/goldens: existing `tests/e2e_*_cli_golden.rs`, `tests/e2e_rec.rs`, `tests/e2e_mcp_server.rs`, and a new or extended show-conformance matrix proving flag availability and JSON file path behaviour across all in-scope kinds.
- Specs/docs if design locks a durable CLI contract change: SPEC-013 and SPEC-014 are the likely authorities to update through the established spec/revision path if required.

## Risks, Assumptions, Open Questions

- JSON compatibility risk: existing golden tests and MCP consumers may assume exact show payloads. The default-on `filepaths` addition must be deliberate and paired with `--no-filepaths` coverage.
- `show --filepaths` is intentionally paths-only for now, while a broader entity summary/info verb is deferred to IMP-145.
- Short `-f` is already the list filter shorthand but is free on show commands today; design should verify no per-kind show conflict.
- Adding booleans to many CLI handlers can trip clippy argument-count/bool-count lints; use an args struct if a handler nears the house lint ceiling.

## Verification / Closure Intent

- Behavioural conformance proves every in-scope entity command accepts `show`, `show --filepaths`, `show --json`, and `show --json --no-filepaths`, with `--filepaths` conflicting with JSON and `--no-filepaths` confined to JSON.
- Golden tests prove human `--filepaths` output is primary-file-first and deterministic, and JSON includes or suppresses file paths as requested.
- Existing per-kind show/list/status suites stay green unchanged except for intentional golden updates.
- `just check` during implementation and `just gate` before close pass with zero warnings.

## Summary

Normalize entity `show` across authored kinds and make backing file paths discoverable: human paths-only output by explicit `--filepaths`, JSON by default with `--no-filepaths` opt-out. Leave broader summary/info surfaces and slice current/details UX to follow-up work.

## Follow-Ups

- Broader CLI UX/help consistency remains with IMP-133 / IMP-135.
- Entity summary/info command exploration is captured as IMP-145.
- Reference parsing consolidation remains with IMP-125 unless the design finds it is directly required for this slice.
