# IMP-443: Laundered reads outside SL-238's surfaces: 11-site STD-003 census

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found by SL-238 PHASE-07's class sweep (`mem.pattern.review.sweep-defect-class-not-instance`).
PHASE-07 removed the two `read_to_string(..).unwrap_or_default()` laundered reads
from `src/commands/dep_seq.rs` (a failed read became an empty `toml::Table`, so a
corrupt target silently read as "no status"). The sweep asks where else the class
lives. **The census is the deliverable here** — running the grep again is the cost
this item exists to avoid.

Needle: `(read_to_string|from_str|parse)(..) … unwrap_or_default()`. Sites outside
`src/backlog.rs` (whose 5 are SL-238 PHASE-08's leg and already owned), as of
`c3d67f773`:

| site | read |
|---|---|
| `src/lazyspec.rs:430` | `toml::from_str(toml_text).unwrap_or_default()` |
| `src/retrieve.rs:1425`, `:1486` | memory body |
| `src/concept_map.rs:383` | map body |
| `src/commands/config.rs:322` | config text |
| `src/slice.rs:6143` | slice body |
| `src/worktree/create.rs:260` | `serde_json::from_str(&raw)` — a create payload |
| `src/memory.rs:2324`, `:2349`, `:2860` | memory bodies |
| `src/spec.rs:4622` | spec body |

**Not all of these are defects, and the item should not be worked as if they were.**
An *absent optional* prose body legitimately reads as the empty string; that is a
total function, not a laundered read. The STD-003 question is narrower: does the
default silently stand in for a read that FAILED for a reason the caller would have
acted on — a corrupt file, a permissions error, a malformed payload? The parse sites
(`lazyspec.rs:430`, `worktree/create.rs:260`) are the strongest candidates on that
test, since a parse failure is never "absent".

Triage per site against that question; repair only where a failure is being
swallowed, and disclose rather than fail where the corpus must stay readable
(STD-003 tolerates AND discloses).

Related: `RSK-013` is the same class at `coverage_scan.rs` and is separately owned;
`catalog::scan:243/246`, `:289/292` are live instances recorded in SL-238's
`notes.md ### Open`.
