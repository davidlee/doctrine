# SL-077 — Design

## Decisions

### D1 — `read_spec` reader signature

```
fn read_spec(spec_dir: &Path, id: u32) -> anyhow::Result<(Spec, String, String)>
//                                                       parsed  raw-toml  prose-body
```

Mirrors `read_slice` exactly: parsed struct, raw TOML text (for tier1
`[[relation]]` parsing), and prose `.md` body. Consolidates 2 inline parse
sites in `spec.rs`:

| Site | Before | After |
|---|---|---|
| `relation_edges` | `read_to_string` + `from_str::<Spec>` | `read_spec`, uses `parsed` for typed fields, `raw-toml` for `tier1_edges` |
| `run_show` | `read_to_string` + `from_str::<Spec>` + separate prose read | `read_spec` once, destructures all three |

`build_registry` keeps its inline parse — the non-trivial `is_second_parent`
error classification (duplicate/array `parent` key) requires a `match` on the
parse error to carry the finding rather than hard-fail. `read_spec` returns
`Result<(Spec, …)>` — it cannot hold a `BuildFinding` and `continue`.

Path construction internal to the function:

```rust
let name = format!("{id:03}");
let toml_path = spec_dir.join(&name).join(format!("spec-{name}.toml"));
let md_path = spec_dir.join(&name).join(format!("spec-{name}.md"));
```

Behaviour-preservation gate: existing suites stay green unchanged. Pure
mechanical extract — same reads, same parses, same error contexts.

### D2 — Requirement prose: lean `load()` + companion `load_body()`

`requirement::load()` stays as-is — returns `Requirement` from TOML only.
Corpus scans (`spec validate`, `catalog::graph`) don't pay the `.md` read.

```rust
/// Read the requirement's .md prose body. Called only by spec show / req list.
/// Returns None when the .md is missing or all headings are scaffold (empty).
/// Degrade-and-continue — a missing .md does not abort the caller; the
/// requirement renders as scaffold. Aligns with E5 (dangling-member tolerance).
pub(crate) fn load_body(root: &Path, canonical_fk: &str) -> Option<String>
```

Path: `.doctrine/requirement/NNN/requirement-NNN.md`, deterministic from FK
(same `id_from_fk` + `name` pattern as `load`). Returns `None` when the `.md`
file is missing (corrupt entity — visible in output, not a silent error).

### D3 — Empty-heading pruning

```rust
/// Strip headings whose content is only whitespace/HTML comments.
/// Drops the H1 unconditionally, demotes surviving ## → ####.
/// Returns None if nothing substantive remains.
fn prune_empty_headings(body: &str) -> Option<String>
```

Algorithm: split on `\n## `, keep sections containing ≥1 non-comment,
non-whitespace line. Empty scaffold renders as `None` → nothing emitted.

Comment-detection contract (per-line evaluation):
- A line is "comment-only" when `trim()` starts with `<!--` AND ends with `-->`
- Single-line HTML comments only — multi-line `<!-- ... -->` spanning multiple
  lines is NOT detected (intermediate lines are treated as content)
- Non-HTML comment syntax (`//`, `#`) is treated as content — the scaffold uses
  `<!-- -->` only

Edge cases:
- Fenced code blocks within a heading section count as content
- A heading whose entire content is a single `<!-- -->` is empty
- A heading with `<!-- comment -->` on one line and real content on another is
  kept (per-line evaluation)
- A heading with `content <!-- inline comment -->` on the same line is kept
  (the content before the comment saves the line)
- All 10+ current requirements are scaffolds → rendered output unchanged

### D4 — Table render: inline after description

Per-requirement prose renders after `description`, before `acceptance_criteria`,
with one blank line separation. Pruned body already has demoted headings.

**`description` / prose reconciliation:** The `description` field (from
`requirement-NNN.toml`) is the structural summary — a one-line tagline rendered
directly after the `kind · status` line. The prose body (from
`requirement-NNN.md`, pruned) is the full Statement and Rationale — rendered
BELOW the description. Both render; neither is deprecated. If `description` is
absent, the prose still renders. If the prose is scaffold (all headings empty),
it is omitted entirely — the description alone stands.

**Filled example (aspirational — all current requirements are scaffolds, so
output is byte-identical until IMP-057 authors real prose):**

```
### FR-001 (REQ-090) — Graph core is a standalone product-neutral crate

functional · pending

The generic graph evaluation core is built as its own workspace member crate...

#### Statement

The graph core must be a standalone workspace crate with no doctrine vocabulary.

#### Rationale

Keeping the core product-neutral allows external consumers to depend on it
without pulling in doctrine's entity model.

acceptance criteria:
  - The graph core lives in a separate workspace crate...
```

Empty scaffold — no change from current output (prose section omitted entirely).

`render()` signature change: `members: &[(Member, Requirement, Option<String>)]`
— `Option<String>` is the pruned prose, `None` when empty.

### D5 — JSON render: optional `body` field

Member requirement gains `body` when prose is non-empty:

```json
{
  "label": "FR-001",
  "order": 1,
  "requirement": {
    "id": "REQ-090",
    "slug": "...",
    "title": "...",
    "kind": "functional",
    "status": "pending",
    "body": "#### Statement\n\nThe graph core must be..."
  }
}
```

`body` absent when `prune_empty_headings` returns `None`. The pruned body is
stored (headings demoted, H1 dropped) — consumer gets a ready-to-render fragment.

### D6 — `spec req list` prose column

`ReqListRow` gains a `prose: String` field — `"✓"` when prose is filled,
`"—"` when scaffold or the member FK is dangling (no load → no prose check;
conservative assumption). New column added to the registry and default set.

```rust
struct ReqListRow {
    id: String,
    label: String,
    kind: String,
    status: String,
    prose: String,
}
```

Defaults: `["id", "label", "kind", "status", "prose"]`.

`ReqJsonRow` gains `prose: bool` with `#[serde(skip_serializing_if = "std::ops::Not::not")]` — present `true` when filled, absent when scaffold (mirrors `dangling` pattern).

## Verification impact

### Behaviour-preservation gate

Existing `spec show` / `spec list` / `relation_edges` / `build_registry` tests
must stay green unchanged. `read_spec` is a mechanical extract — no logic change.

### New tests

| Test | What |
|---|---|
| `read_spec_round_trips` | Parse + raw text + prose from a fresh scaffolded spec |
| `prune_empty_headings_scaffold` | Template body → `None` |
| `prune_empty_headings_filled` | Statement with content, empty Rationale → returns body with Statement only, Rationale heading dropped |
| `prune_empty_headings_code_block` | Fenced block in Statement → section kept |
| `spec_show_renders_requirement_prose` | Seeded requirement with filled `.md` → prose appears inline in table output |
| `spec_show_omits_empty_requirement_prose` | Scaffold requirement → no prose section rendered |
| `spec_show_json_includes_prose_body` | JSON member has `body` when filled |
| `spec_show_json_omits_body_when_empty` | JSON member has no `body` key when scaffold |
| `req_list_prose_column_scaffold` | All-scaffold roster → `—` in prose column |
| `req_list_prose_column_filled` | Mixed roster → `✓` for filled, `—` for scaffolds |
| `req_list_prose_column_dangling` | Dangling member FK → `—` in prose column |
| `prune_empty_headings_non_html_comment_is_content` | `// comment` / `# comment` syntax in a section → section kept (not pruned) |

## Affected surface

- `src/spec.rs` — `read_spec()`, `run_show()`, `render()`, `show_json()`,
  `relation_edges()`, `build_registry()`, `req_rows()`, `ReqListRow`,
  `ReqJsonRow`, `REQ_COLUMNS`, `REQ_DEFAULT`, `req_list_rows()`
- `src/requirement.rs` — `load_body()`, `prune_empty_headings()`

## Known edge cases

- **`###` within a Statement/Rationale section** causes heading inversion (H3 above
  demoted H4). Rare — authors shouldn't nest H3 inside a requirement section.
  Acceptable; the spec's own body rendering doesn't transform headings at all.
- **Missing `.md` file** — `load_body` returns `None` (the requirement renders as
  scaffold). A corrupt entity is visible in output (no prose section), not a
  silent error. Aligns with the E5 degrade-and-continue pattern for dangling
  member FKs.
