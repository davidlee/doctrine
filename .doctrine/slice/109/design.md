# Design: MCP server for review commands

## 1. Architecture overview

```
                    ┌─────────────────────────┐
                    │   MCP client (agent)     │
                    │   stdio JSON-RPC 2.0     │
                    └──────────┬──────────────┘
                               │
                    ┌──────────▼──────────────┐
                    │  src/mcp_server/         │
                    │  ┌─────────────────────┐ │
                    │  │ tools.rs            │ │  tool defs + dispatch
                    │  │ protocol.rs         │ │  JSON-RPC types
                    │  │ transport.rs        │ │  stdio codec
                    │  │ mod.rs              │ │  serve loop
                    │  └──────────┬──────────┘ │
                    └─────────────┼────────────┘
                                  │ calls review::run_*
                    ┌─────────────▼────────────┐
                    │  src/review.rs           │
                    │  ┌─────────────────────┐ │
                    │  │ ReviewOutput enum   │ │  structured return (NEW)
                    │  │ run_new/raise/...   │ │  → anyhow::Result<ReviewOutput>
                    │  │ with_turn (generic) │ │  → anyhow::Result<T>
                    │  │ vocab, lock, baton  │ │  unchanged
                    │  └─────────────────────┘ │
                    └─────────────┬────────────┘
                                  │ CLI path
                    ┌─────────────▼────────────┐
                    │  src/main.rs             │
                    │  ┌─────────────────────┐ │
                    │  │ print_review()      │ │  single formatting pass (NEW)
                    │  │ Command::Serve      │ │  --mcp flag (NEW)
                    │  └─────────────────────┘ │
                    └──────────────────────────┘
```

**Principle:** The review engine returns structured data. The CLI formats it for humans. The MCP server returns it as JSON. No logic is duplicated.

## 2. Design decisions

### D1 — `ReviewOutput` is a variant enum

Each `run_*` function returns exactly one variant carrying exactly the data its
caller needs. No optional fields, no ambiguity. Exhaustiveness checked by the
compiler at every match site.

```rust
#[derive(Debug, Serialize)]
pub(crate) enum ReviewOutput {
    Created  { id: u32, canonical: String, dir: PathBuf },
    Raised   { finding_id: String, review_id: u32 },
    Disposed { finding_id: String, review_id: u32 },
    Verified { finding_id: String, review_id: u32 },
    Contested{ finding_id: String, review_id: u32 },
    Withdrawn{ finding_id: String, review_id: u32 },
    Showed   { json: serde_json::Value, formatted: String },
    Listed   { rows: Vec<ListRow>, formatted: String },
    Status   {
        canonical: String,
        status: String,
        awaiting: String,
        findings_count: usize,
        rounds: usize,
        cache_verdict: Option<String>,
        formatted: String,
    },
    Primed   { cache_paths: Vec<String>, stale: Vec<String> },
    Unlocked { canonical: String },
}
```

**Why `Showed` and `Listed` carry both structured and formatted:** `run_show` and
`run_list` already have `Format::Table`/`Format::Json` split. Rather than refactor
that stack, each variant carries the human-formatted string *and* a structured
payload (`json` for show, `rows: Vec<ListRow>` for list). The CLI prints
`formatted`; MCP returns the structured field.

**`ListRow` is the existing `#[derive(Serialize)]` struct** from `src/review.rs`
(fields: `id`, `status`, `awaiting`, `facet`, `target`, `title`). It carries the
table row data that `list_rows` already computes. The CLI path calls
`listing::render_columns` to produce `formatted`; the MCP path serialises `rows`
directly.

**Why `Status` carries concrete fields + formatted:** `run_status` produces a
multiline output: canonical header, derived status/await/findings/rounds, plus
optional cache staleness. Rather than leak internal enums, `Status` carries
pre-rendered strings for the format-sensitive fields *and* a `formatted: String`
for the full CLI output. MCP returns the structured fields; CLI prints `formatted`.

### D2 — `with_turn` is generic over closure return type

Current signature: `F: FnOnce(...) -> anyhow::Result<()>`.

New signature: `F: FnOnce(...) -> anyhow::Result<T>`.

Returns `anyhow::Result<T>`. This is a one-line change: replace `()` with `T`
and add the generic parameter.

**Which verbs need `T != ()`:** Only `run_raise` and `run_dispose` — their
closures return the new finding id (`String`) so the outer function can construct
the `ReviewOutput` variant. `run_verify`, `run_contest`, and `run_withdraw`
delegate to `run_raiser_transition` which passes a `|| Ok(())` closure to
`with_turn` — these stay `T = ()` because the finding_id is already in scope
from the args and the outer function constructs the variant directly.

### D3 — MCP server calls `run_*` directly, not engine internals

Each MCP tool handler is a ~3-line wrapper: deserialise args, call `run_*`,
return `ReviewOutput`. The lock/baton/turn protocol is honoured by `run_*`
as-is. No duplication of orchestration logic.

```rust
fn handle_review_raise(args: serde_json::Value, root: &Path) -> anyhow::Result<ReviewOutput> {
    let args: RaiseArgs = serde_json::from_value(args)?;
    review::run_raise(Some(root.to_path_buf()), &args, Role::Raiser)
}
```

### D4 — MCP server is hand-rolled, zero new crates

The MCP tools-only protocol surface (`initialize`, `tools/list`, `tools/call`,
`notifications/initialized`) is ~80 lines of serde types + ~60 lines of stdio
transport. The project already has `serde`, `serde_json`, and `tokio` with
`io-util`. No new dependency.

### D5 — Project root resolved once at startup

The MCP server resolves the project root from cwd or `--path` at startup. Every
tool call uses that root. This matches the CLI's `crate::root::find()` semantics.
If cwd is wrong, the server fails at init — clean and early.

```rust
// src/mcp_server/mod.rs
pub(crate) struct McpConfig {
    pub(crate) path: Option<PathBuf>,
}

pub(crate) async fn serve(config: McpConfig) -> anyhow::Result<()> {
    let root = crate::root::find(config.path, &crate::root::default_markers())?;
    // ... transport loop ...
}
```

### D6 — `--mcp` flag gates the serve mode

```rust
// src/commands/serve.rs
#[derive(Args)]
pub(crate) struct ServeArgs {
    #[arg(long)]
    pub(crate) mcp: bool,

    #[arg(long)]
    pub(crate) path: Option<PathBuf>,
}

pub(crate) fn run_serve(args: ServeArgs) -> anyhow::Result<()> {
    if args.mcp {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(crate::mcp_server::serve(crate::mcp_server::McpConfig {
            path: args.path,
        }))
    } else {
        anyhow::bail!("`serve` requires --mcp (other serve modes not yet implemented)");
    }
}
```

Future serve modes (HTTP, etc.) add flags without breaking the interface.

### D7 — Decomposition deferred to follow-up

`src/review.rs` (~1940 non-test lines) will be decomposed into
`src/review/{types, engine, verbs, render, cache}.rs` as a separate follow-up
slice. The decomposition is a mechanical move of code blocks with zero
behavioural impact. Keeping it separate from the signature refactor means each
commit has a single, reviewable purpose.

## 3. Code impact

### `src/review.rs`

| Function | Current return | New return | Notes |
|---|---|---|---|
| `run_new` | `()` | `ReviewOutput::Created` | Returns id, canonical, dir |
| `run_raise` | `()` | `ReviewOutput::Raised` | Returns finding_id, review_id |
| `run_dispose` | `()` | `ReviewOutput::Disposed` | Returns finding_id, review_id |
| `run_verify` | `()` | `ReviewOutput::Verified` | Returns finding_id, review_id |
| `run_contest` | `()` | `ReviewOutput::Contested` | Returns finding_id, review_id |
| `run_withdraw` | `()` | `ReviewOutput::Withdrawn` | Returns finding_id, review_id |
| `run_show` | `()` | `ReviewOutput::Showed` | Carries both JSON and formatted |
| `run_list` | `()` | `ReviewOutput::Listed` | Carries formatted string |
| `run_status` | `()` | `ReviewOutput::Status` | Carries pre-rendered fields |
| `run_prime` | `()` | `ReviewOutput::Primed` | Carries cache paths + stale |
| `run_unlock` | `()` | `ReviewOutput::Unlocked` | Returns canonical |

**`with_turn`:** signature change from `FnOnce(...) -> anyhow::Result<()>` to
`FnOnce(...) -> anyhow::Result<T>`. One-line change, zero behavioural impact.

**`ReviewOutput`:** new enum added at module scope, `#[derive(Debug, Serialize)]`.

**`RaiseArgs`, `DisposeArgs`, `NewArgs`, `PrimeArgs`:** gain
`#[derive(Deserialize)]` for MCP argument deserialisation.

**`Severity`, `Facet`:** gain `Deserialize` via custom deserializer forwarding to
the existing `parse` methods (e.g. `#[serde(deserialize_with = "Facet::deserialize_from_str")]`).
No change to the enums' internal representation.

**No changes to:** lock/baton infrastructure, mutation helpers, cache/prime
engine, templates, tests.

### `src/main.rs`

- New `Command::Serve(ServeArgs)` variant in the CLI enum
- New `print_review(&ReviewOutput) -> anyhow::Result<()>` function — single
  formatting pass, one match arm per variant. For dual-output variants
  (`Showed`, `Listed`, `Status`), prints `formatted` and ignores the
  structured field.
- 11 call sites change from `review::run_*(...)?;` to
  `let out = review::run_*(...)?; print_review(&out)?;`

### New files

| File | Purpose | Est. lines |
|---|---|---|
| `src/mcp_server/mod.rs` | `serve()` entry point, tokio loop | ~60 |
| `src/mcp_server/protocol.rs` | JSON-RPC 2.0 types | ~80 |
| `src/mcp_server/transport.rs` | Stdio codec (read/write framed messages) | ~60 |
| `src/mcp_server/tools.rs` | Tool definitions + handler dispatch | ~100 |
| `src/commands/serve.rs` | `ServeArgs` + `run_serve()` CLI wiring | ~25 |

### Test impact

**Zero.** Tests call `run_*` functions and check disk state, not stdout. After
the refactor, `.unwrap()` ignores the `ReviewOutput`; `.unwrap_err()` still
works for error cases. No test assertions change.

## 4. MCP protocol mapping

### Server capabilities (initialize response)

```json
{
  "capabilities": {
    "tools": {}
  }
}
```

No resources, prompts, logging, or sampling — tools only (v1 scope).

### `tools/list` response

Returns all 10 review tools with JSON Schema parameter definitions:

- `review_new` — facet, target, phase?, title?, raiser?, responder?
- `review_list` — facet?, target?, status?
- `review_show` — reference, format? (table | json)
- `review_raise` — reference, severity, title, detail, as?
- `review_dispose` — reference, finding, disposition, response, as?
- `review_verify` — reference, finding, note?, as?
- `review_contest` — reference, finding, note?, as?
- `review_withdraw` — reference, finding, as?
- `review_status` — reference
- `review_prime` — seed?, domain_map?, reference

### `tools/call` flow

1. Client sends `{ method: "tools/call", params: { name: "...", arguments: {...} } }`
2. Server deserialises `arguments` into the matching arg struct
3. Server calls `review::run_*()` — acquires lock, validates, mutates, derives, unlocks
4. Server serialises `ReviewOutput` to JSON
5. Server returns `{ result: { content: [{ type: "text", text: "<json>" }] } }`

### Error mapping

| Condition | MCP error code | `data` payload |
|---|---|---|
| Unknown tool | -32601 | `{ method: "..." }` |
| Invalid arguments | -32602 | `{ parse_error: "..." }` |
| Review not found | -32000 | `{ code: "NOT_FOUND", reference: "..." }` |
| Wrong role for verb | -32000 | `{ code: "ROLE_MISMATCH", expected: "raiser", got: "responder" }` |
| Finding not in required state | -32000 | `{ code: "STATE_MISMATCH", finding: "F-3", current: "verified", expected: "answered" }` |
| Dangling target ref | -32000 | `{ code: "DANGLING_REF", target: "SL-099" }` |

All review engine errors pass through `anyhow::Error` → MCP error response with
`-32000` (server error) and the error message in `data: { message: "..." }`.
Structured error codes (`NOT_FOUND`, `ROLE_MISMATCH`, `STATE_MISMATCH`) are
parsed from the error string where the engine's `anyhow::bail!` messages follow
a predictable pattern (e.g. `"verify expects ..."` → `STATE_MISMATCH`).
Unrecognised errors fall through with `message` only.

## 5. Verification strategy

### VH-1 — CLI output behaviour-preserving

The `print_review()` function produces output identical to the current
`writeln!` calls. Golden tests: capture current stdout for each verb before
refactor, then assert `print_review` produces identical strings after refactor.
Not byte-exact (path formatting may differ between `PathBuf::display()` and the
original `dir.display()`), but semantically identical for every verb.

### VH-2 — Tests stay green unchanged

Existing review tests (`src/review.rs` mod tests) pass without modification.
The behaviour-preservation gate: no test assertion changes.

### VH-3 — MCP protocol handshake

Integration test: spawn `doctrine serve --mcp` as a subprocess, send
`initialize` → `tools/list` → `tools/call` messages over stdio, assert valid
JSON-RPC responses. Use a temp project with a seeded slice target.

### VH-4 — MCP tool round-trips

For each review verb, call the MCP tool, then read the authored state from disk
and assert the mutation landed correctly. Covers the full
`deserialise → run_* → serialise → disk` pipeline.

### VH-5 — Baton CAS under batch mutation (deferred to execute phase)

Agent test run: an agent drives multiple review findings through the MCP server
in rapid sequence, verifying no lock contention failures or stale-baton
rejections. The existing per-review lock (ADR-007 D-C4a) should serialise these
correctly; the test validates this under realistic agent concurrency.

## 6. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Baton CAS under batch mutation | Medium | Per-review lock serialises concurrent writes (ADR-007 D-C4a). Verified in execute phase with agent test run (VH-5). |
| `Deserialize` on `Severity`/`Facet` clashes with existing `parse` methods | Low | Custom deserializer forwarding to `parse()` — or `#[serde(deserialize_with)]` on the MCP arg structs |
| `Serialize` on `PathBuf` in JSON | Low | Acceptable — path is human-readable and relative to project root |
| `with_turn` generic introduces turbofish at call sites | Low | Callers already provide closure type; compiler infers `T` from closure return |

## 7. Follow-ups

- **Session-scoped review context** — remember "current review" across MCP tool calls
- **Decomposition of `src/review.rs`** — extract `src/review/{types, engine, verbs, render, cache}.rs`
- **Other command suites** — memory, slice, backlog as MCP tools
- **MCP resources** — expose review state as readable documents
- **MCP prompts** — templated review workflows
- **HTTP transport** — for non-stdio MCP clients
