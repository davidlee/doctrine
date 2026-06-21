# Uniform entity show and file path surfaces

## Context

Doctrine's CLI contract is meant to be predictable across authored entity kinds: a user should be able to ask a kind to `show` an entity and receive the readable Markdown reconstruction, and separately ask for the authored paths that back an entity when the intent is editor/shell plumbing.

Today that contract is uneven. Some kinds already have a `show` surface and JSON output, but the shape is kind-specific; file paths are not uniformly surfaced; and concept maps are an authored entity command whose show surface has drifted from the common JSON shorthand.

This slice scopes the cross-kind read-surface normalization. It is governed by SPEC-013's uniform CLI grammar, SPEC-004's entity storage rule, SPEC-014's slice surface, ADR-001's layering boundary, and ADR-005's read-via-`show` discipline.

Related but not governing backlog context: IMP-133 and IMP-135 cover broader CLI UX/help consistency; IMP-125 covers per-kind reference parsing consolidation; IMP-145 covers a richer future entity `info` surface. This slice is narrower: entity `show` parity and a dedicated file path verb.

## Scope & Objectives

- Define the entity-command coverage matrix for `show` parity:
  - in-scope authored entity commands: `adr`, `policy`, `standard`, `rfc`, `spec`, `backlog`, `knowledge`, `slice`, `memory`, `review`, `rec`, `revision`, and `concept-map`;
  - keep non-entity read surfaces (`inspect`, `survey`, `next`, `blockers`, `explain`, `coverage`, `status`) out.
- Ensure every in-scope entity command has a `show` verb that renders the entity's readable Markdown/prose reconstruction plus embedded child/facet content where that kind owns it (for example spec members/requirements, review findings/briefs, or similar kind-specific embedded material).
- Add a dedicated `paths` verb to every in-scope entity command: `doctrine <kind> paths <REF>`.
- `paths` prints root-relative paths only, one per line: the primary entity file first, followed by the other direct regular files in the entity's authored folder in deterministic order.
- Add `--single` to `paths` as an explicit shorthand for returning only the primary path (`| head -n 1` without shell dependence).
- Preserve existing `show --json` semantics as body/reconstruction plus kind-owned structured/embedded data; do not add file path data to `show --json`.
- Prefer a shared paths projection helper and reference-resolution wiring over per-kind bespoke implementations, preserving ADR-001 layering: filesystem access stays out of `listing.rs` and isolated from kind-specific renderers.
- Keep the broader `info` / summary read-surface question out of scope; it is captured as follow-up IMP-145.

## Non-Goals

- Do not change authored storage layout or move entity files.
- Do not add file path output to `show`, `show --json`, or `list` commands.
- Do not redesign relation inspection; `inspect` remains the relation graph read surface.
- Do not rewrite every kind's show renderer for file path concerns; `show` remains the body/reconstruction surface.
- Do not include runtime `.doctrine/state/` phase files, symlink aliases, or subdirectories in the authored file path list.
- Do not add a slice sibling/current/details read surface here; expose existing authored sibling files only as regular files when they sit in the slice folder.
- Do not add an MCP `paths` surface. The MCP tools mirror the CLI surface; a future MCP `paths` binding is a follow-up concern, not part of this slice.

## Affected Surface

- CLI definitions and dispatch: `src/commands/cli.rs`; per-kind command modules in `src/adr.rs`, `src/policy.rs`, `src/standard.rs`, `src/rfc.rs`, `src/spec.rs`, `src/backlog.rs`, `src/knowledge.rs`, `src/slice.rs`, `src/memory.rs`, `src/review.rs`, `src/rec.rs`, `src/revision.rs`, and `src/concept_map.rs`.
- Shared read/projection seams likely involved: a new small paths helper module, `src/entity.rs` path helpers, `src/meta.rs`, `src/governance.rs`, and per-kind reference resolution in the modules above. `src/listing.rs` remains pure and untouched for path I/O.
- Tests/goldens: existing `show` tests stay green, concept-map `show --json` parity is covered, and a new paths-conformance matrix proves `paths` / `paths --single` across all in-scope kinds.
- Specs/docs if design locks a durable CLI contract change: SPEC-013 and SPEC-014 are the likely authorities to update through the established spec/revision path if required.

## Risks, Assumptions, Open Questions

- `show --json` is already a relied-on full-inspection surface (body plus structured/embedded data for specs, memory, review, slice, etc.); adding operational file paths there would conflate two intents and create planned rework toward `info`.
- A dedicated `paths` verb is a wider CLI grammar change than a show flag; SPEC-013 may need a revision because the uniform verb set expands beyond `new/list/show/status`.
- Reference parsing remains per-kind today; implement with minimal adapters unless the design proves IMP-125's broader consolidation is required.
- Show-parity scope: the normalization objective is CLI-grammar parity (`--json` shorthand on every kind), not JSON-output-shape uniformity. JSON shape normalization across kinds belongs to IMP-145. See design §7 D8.

## Verification / Closure Intent

- Behavioural conformance proves every in-scope entity command accepts `show`, `show --json` where JSON is part of the show contract, `paths`, and `paths --single`.
- Golden tests prove `paths` output is primary-file-first, root-relative, direct-regular-files-only, and deterministic.
- Existing per-kind show/list/status suites stay green unchanged except for intentional concept-map `--json` parity updates.
- `just check` during implementation and `just gate` before close pass with zero warnings.

## Summary

Normalize entity `show` across authored kinds and make backing file paths discoverable through a dedicated `paths` verb. Leave broader summary/info surfaces and slice current/details UX to follow-up work.

## Follow-Ups

- Broader CLI UX/help consistency remains with IMP-133 / IMP-135.
- Entity summary/info command exploration is captured as IMP-145.
- Reference parsing consolidation remains with IMP-125 unless the design finds it is directly required for this slice.
