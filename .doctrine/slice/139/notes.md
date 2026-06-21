# SL-139 Notes

## 2026-06-22 — Inquisition RV-134: design review

The Inquisition (RV-134) raised 7 findings against the design. All resolved.

### Corrections applied to design.md:

1. **D8 added** (§7): show-parity scope is CLI-grammar parity (every kind accepts `--json` shorthand), not JSON-output-shape uniformity. JSON shape normalization belongs to IMP-145.
2. **paths.rs tier assigned** (§5.3): engine tier. Depends on leaf (stdlib, entity.rs), depended on by command. Rationale: entity.rs already carries filesystem access in the engine tier.
3. **Sub-kind directory paths** (§5.4): explicit paths for backlog (`.doctrine/backlog/{issue|improvement|chore|risk|idea}/{id}/`), spec (`.doctrine/spec/{product|tech}/{id}/`), knowledge (`.doctrine/knowledge/{assumption|decision|question|constraint}/{id}/`).
4. **Exclusion filter** (§5.5): defined exclusion for hidden files (`.`-prefix), editor temporaries (`#`, `~`, `.swp`), and tool artifacts (`.orig`, `.bak`).
5. **Concept-map body tolerance** (§5.5): noted that `concept_map::read_concept_map` tolerates missing `.md` via `unwrap_or_default()` — preserved for `show`, enforced for `paths --md`.

### Corrections applied to slice-139.md (scope body):

6. **MCP exclusion**: added to Non-Goals — "Do not add an MCP `paths` surface."
7. **D8 scope note**: added to Risks section referencing design §7 D8.

### Tolerated (no design change):

- `--entity` flag is syntactic sugar over `-t -m`. Tolerated for discoverability.

### Memory recorded:

- `mem.pattern.design.cli-normalization-ambiguity`: when a design claims to normalize output, distinguish flag-parity from schema-parity.

### Design now ready for phase planning.

### 2026-06-22 — Plan authored (4 phases)

- `plan.toml` + `plan.md` committed (897cf483 on edge)
- 4 phases: PHASE-01 (paths.rs helper), PHASE-02 (concept-map --json), PHASE-03 (numeric stem kinds), PHASE-04 (umbrella + named kinds)
- Slice moved to `ready`

See RV-134 for full findings and dispositions.
