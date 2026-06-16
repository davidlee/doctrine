# SL-077 design review — domain_map

## Areas
- src/spec.rs — `run_show`, `relation_edges`, `build_registry`, `render`, `show_json`, `req_rows`, `ReqListRow`, `ReqJsonRow`, `REQ_COLUMNS`, `REQ_DEFAULT`
- src/requirement.rs — `load`, the toml reader path; `requirement_scaffold` (the .md path)
- .doctrine/slice/077/design.md — D1 through D6
- .doctrine/slice/077/slice-077.md — scope §1–§5

## Invariants
- Behaviour-preservation: existing tests stay green unchanged
- Storage rule: prose in .md, structured data in .toml — never queried/derived data in prose
- read_slice precedent: `(parsed, raw-toml, prose-body)` signature
- second_parent: build_registry must carry the finding, not fail hard
- degrade-and-continue: E5 pattern for dangling member FKs in req list
- Pure/imperative split: no disk in the pure layer (render / prune_empty_headings)

## Risks
- description field vs ## Statement prose overlap — two sources for same concept
- D1 table including build_registry despite scope exclusion
- prune_empty_headings comment detection underspecified
- load_body error surface undefined for per-req degradation
