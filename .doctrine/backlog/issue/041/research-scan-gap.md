Report written to `/.doctrine/backlog/issue/041/research-scan-gap.md`.

**TL;DR of findings:**

| # | Finding | Location |
|---|---|---|
| 1 | `outbound_for` returns `Ok(Vec::new())` for CM — comment says "CM authors no outbound relations" | `scan.rs:53` |
| 2 | `concept_map.rs` has **no** `relation_edges` function, no tier-1 reader. `ConceptMapDoc` silently drops `[[relation]]` rows. | `concept_map.rs` |
| 3 | RELATION_RULES declares `CM > contextualizes` as **Tier::One, LinkPolicy::Writable** — contradicts the scan | `relation.rs:370-378` |
| 4 | Every other governed kind follows `read_doc → tier1_edges(kind, text)` pattern; CM is the only one missing it | `governance.rs:268-278` |
| 5 | `doctrine link CM-001 contextualizes SL-047` **writes** `[[relation]]` to the TOML (via `append_edge`), but scan never reads it — **dead write** | `commands/relation.rs:81`, `relation.rs:799` |
| 6 | Install template has **no** `[[relation]]` block — only `dsl = ''''''` | `install/templates/concept-map.toml` |

**Root cause:** RELATION_RULES promises a writable tier-1 edge; the scan ignores all CM edges. Three fix options documented in the report (A: wire CM into scan, B: remove the RELATION_RULES entry, C: hybrid + narrow TargetSpec).