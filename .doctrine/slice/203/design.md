# Design SL-203: Dissolve commands↔mcp_server dependency cycle

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

RSK-227's coupling map found `{commands, mcp_server}` as a 2-node SCC **separate
from the 23-node command core** (`backlog-227.md:94-96`). Dissolve it by
severing the one incidental edge, without touching the legitimate forward edge
and without behaviour change. This is the cheap warm-up (IMP-265) that proves
the cycle-breaking pattern before the larger integrity refactor (SL-204).

## 2. Current State

The SCC is exactly two edges:

- **forward** `commands::serve::run_serve → mcp_server::serve` (`serve.rs:28`) —
  a `serve` command starting its server. Legitimate; unchanged.
- **back** `mcp_server::tools::render_model_band_guidance →
  commands::prompt::model_keys` (`tools.rs:1365`) — the MCP `doctrine_onboard`
  tool reaching up into a CLI command handler for one prompt-key helper.
  Incidental. Severing it dissolves the SCC.

`model_keys(root, harness)` (`prompt.rs:278`) gathers the hymns corpus
(`install::embedded_hymns` + `embedded_seal_set` + `load_full_corpus`) and
filters `Band::Model` labels. The gather functions live in `install` (**command**
tier) because they bridge the RustEmbed `Assets` root — they cannot sink to a
leaf.

Both `commands` and `mcp_server` are **command** tier; the SCC is a sub-cycle
inside the 123-edge command tangle (`layering.toml` `[tangle_baseline] command`).

Plumbing: `serve()` consumes `McpConfig` at startup, extracts `config.path` into
`root`, discards the rest; only `&root` threads down the dispatch chain
`dispatch(req, &root)` → `handle_tools_call(.., root)` → `call_tool(.., root)` →
`render_onboard(root)` → `render_model_band_guidance(root)` (`tools.rs:409/455/
468/953/1355/1364`).

## 3. Forces & Constraints

- **ADR-001** — leaf ← engine ← command, no cycles; the layering gate ratchets
  the command tangle (monotone non-increasing).
- **Behaviour-preservation gate** (AGENTS.md) — the entity/serve machinery is
  shared; existing suites must stay green unchanged. `doctrine_onboard` output
  byte-identical.
- **RSK-227 concentration warning** — the gate is blind to *concentration*;
  offsetting a removed SCC edge with a new same-tier edge still passes at the
  budget, letting a god-module grow silently (`backlog-227.md:23-25`). A
  dissolution that adds no same-tier edge is strictly better than one that
  relocates the edge.
- **RustEmbed anchor** — `embedded_hymns`/`embedded_seal_set` bind `Assets`
  (`install.rs:20-22`), the crate-wide embed root; the corpus gather cannot
  leave the command tier.
- **STD-001** — the ratchet value is a single-sourced named constant in
  `layering.toml`; the test-header comment must not drift from it.

## 4. Guiding Principles

- Ride the established seam, don't invent one (DRY). The renderer-injection
  pattern already exists in-tree (`prompt::dispatch(cmd, command_map: fn() ->
  String)`, `prompt.rs:105`) and is the recorded doctrine answer for this exact
  situation (`mem.pattern.lint.back-edge-tangle-inject-fnptr`).
- Smallest diff that fully dissolves the coupling; no module moves, no
  `layering.toml` re-classification (R1 — avoid sprawl).
- Add **no** new same-tier edge (§3 concentration force).

## 5. Proposed Design

### 5.1 System Model

**Dependency inversion by fn-pointer injection.** `mcp_server` stops importing
`crate::commands` (and never reaches `crate::install`). The model-key producer
is supplied at serve-start by `commands::serve` — the layer that *already*
depends downward on `mcp_server` via the legit forward edge — and threaded to
the one tool arm that needs it. `model_keys` stays exactly where it is in
`commands::prompt`; nothing moves.

Post-change, `mcp_server` is corpus-agnostic: it knows neither the hymns corpus
nor `install`. The 2-node SCC becomes trivial; **both** its edges leave the
tangle count.

### 5.2 Interfaces & Contracts

New type + config field, in `mcp_server/mod.rs` (refers only to `std` +
`anyhow` — no command import):

```rust
/// Producer for the model-band self-ID key list, injected by the serve entry
/// so mcp_server needn't reach up into the command tier (SL-203, ADR-001).
pub(crate) type ModelKeysFn = fn(&std::path::Path, Option<&str>) -> anyhow::Result<Vec<String>>;

pub(crate) struct McpConfig {
    pub(crate) path: Option<PathBuf>,
    pub(crate) model_keys: ModelKeysFn,
}
```

Supply site — `commands::serve::run_serve` (`serve.rs:28`):

```rust
crate::mcp_server::serve(crate::mcp_server::McpConfig {
    path: args.path,
    model_keys: crate::commands::prompt::model_keys,   // fn item → fn ptr
})
```

Threaded chain (fn-ptr rides beside `root`, confined to the onboard arm):

```rust
tools::dispatch(&request, &root, config.model_keys)          // serve() loop
fn dispatch(req, root: &Path, model_keys: ModelKeysFn) -> JsonRpcResponse
fn handle_tools_call(id, params, root, model_keys) -> JsonRpcResponse
fn call_tool(id, params, root, model_keys) -> anyhow::Result<String>
    "doctrine_onboard" => render_onboard(root, model_keys),  // only arm using it
fn render_onboard(root, model_keys) -> anyhow::Result<String>
fn render_model_band_guidance(root, model_keys) -> anyhow::Result<String>
    { let keys = (model_keys)(root, None)?; /* unchanged rendering */ }
```

The 3 intermediaries (`dispatch`, `handle_tools_call`, `call_tool`) are pure
routers that pass the pointer through. The ~40 `root.to_path_buf()` tool arms
are untouched. `render_model_band_guidance`'s body is unchanged except the call
target: `crate::commands::prompt::model_keys(root, None)` → `(model_keys)(root,
None)`.

### 5.3 Data, State & Ownership

`ModelKeysFn` is a `Copy` fn-pointer — no ownership/lifetime cost, safe to copy
across serve-loop iterations. `commands::serve` owns the choice of concrete
producer; `mcp_server` owns only the type and call. No global/ambient state
(pure-shell rule preserved).

### 5.4 Lifecycle, Operations & Dynamics

Resolution stays **lazy per onboard call** (root-parameterized), identical to
today. No precompute-at-startup (rejected D-C, §7) — that would change timing
without saving any plumbing.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** — `mcp_server` production code holds zero `crate::commands` and zero
  `crate::install` import paths after the change. Proven structurally by VT-1.
- **INV-2** — `doctrine_onboard` output is byte-identical (VT-2).
- **Edge — cfg(test) callers.** The in-module `#[cfg(test)]` tests call
  `dispatch(&req, &root)` (`tools.rs:1458/1471/1628/1643`); they gain the arg
  and pass the real `crate::commands::prompt::model_keys`. The layering
  extractor **excludes `#[cfg(test)]` item/module bodies** (`architecture_
  layering.rs:120,315,326`), so this reference is invisible to the gate — it
  can neither mask a failure nor forge a pass.

## 6. Open Questions & Unknowns

All resolved during design:

- **OQ-1** (slice: where does `model_keys` belong?) → **resolved**: it stays in
  `commands::prompt`. The premise that it should *move* to a leaf is unsound —
  the corpus gather is anchored to the `install`/RustEmbed command tier, so a
  moved `model_keys` would merely relocate the upward reach (see D-A, §7).
- **ASM-1** (slice: `model_keys` cleanly liftable to leaf) → **falsified**: the
  filter is pure but the gather is not liftable (`Assets` anchor). Superseded by
  the injection approach.
- **D1** (how far does the pointer thread?) → **resolved**: thread it as a param
  beside `root`, confined to the onboard arm (~5 signatures). A `ToolCtx { root,
  model_keys }` seam replacing `root` was rejected (D-D, §7).

## 7. Decisions, Rationale & Alternatives

- **D-B (chosen) — fn-pointer injection through `McpConfig`.** The established
  doctrine pattern (`mem.pattern.lint.back-edge-tangle-inject-fnptr`, high
  trust) with live in-file precedent (`prompt::dispatch`). Adds no same-tier
  edge; `mcp_server` ends corpus-agnostic; no module move, no `layering.toml`
  re-classification.
- **D-A (rejected) — extract `model_keys` to a leaf (slice's stated
  hypothesis, SL-016 pattern).** The corpus gather binds `install` (command,
  RustEmbed `Assets`), so the caller still reaches into the command tier —
  relocating the back-edge to `mcp_server → install` rather than dissolving it,
  and adding a mixed-umbrella `layering.toml` entry
  (`mem.pattern.lint.module-split-needs-layering-entry`). Its honest form
  reduces to D-B at the gather boundary. `install` is likely *in* the 23-node
  core, so the relocated edge feeds exactly the concentration RSK-227 warns
  against.
- **D-C (rejected) — precompute `Vec<String>` into `McpConfig` at startup.**
  Still must thread the result through the same dispatch chain (no plumbing
  win), and changes resolution timing from per-call to startup for no benefit.
- **D-D (rejected) — `ToolCtx { root, model_keys }` replacing the threaded
  `root`.** Cohesive, but rewrites ~40 `root` call sites for a one-field gain.
  Overkill for the warm-up (R1). Revisit only if a second injected dep lands.

## 8. Risks & Mitigations

- **R1 (slice) — extraction sprawl.** Mitigated by choosing D-B (zero module
  moves) over D-A.
- **R2 — silent mis-wire.** A null/wrong `ModelKeysFn` would compile and pass
  the structure-only VT-1. Mitigated by the wiring guard (§9): a test that
  drives `render_onboard` through the injected pointer and asserts the model-key
  section renders non-empty.
- **R3 — ratchet drift.** The test-header comment is already stale (120 vs
  baseline 123, `architecture_layering.rs:11-12`). Fix it to 121 in-slice
  (STD-001 single-source hygiene) while touching the gate.

## 9. Quality Engineering & Validation

- **VT-1 (primary, structural proof).** `cargo test --test
  architecture_layering` green with `[tangle_baseline] command = 121`; Tarjan no
  longer groups `{commands, mcp_server}`. Load-bearing: a surviving production
  back-edge holds the count at 123 > 121 → RED. Expected exactly −2 (the
  2-cycle's two edges), since the SCC is core-separate (RSK-227).
- **VT-2 (behaviour-preservation).** Full suite green, unchanged.
  `doctrine_onboard` output byte-identical.
- **VT-3 (wiring guard).** A test drives `dispatch("doctrine_onboard")` /
  `render_onboard` through the injected `ModelKeysFn` and asserts the model-key
  section is present and non-empty (R2). Add red/green if none exists.
- **VA-1.** `doctrine slice conformance SL-203` clean against the design-target
  selectors (§ code-impact).

Code-impact / design-target touch-set:

| path | change |
|---|---|
| `src/mcp_server/mod.rs` | `ModelKeysFn` type; `model_keys` field on `McpConfig`; `serve` threads it into `dispatch` |
| `src/mcp_server/tools.rs` | +param on `dispatch`/`handle_tools_call`/`call_tool`/`render_onboard`/`render_model_band_guidance`; drop the `crate::commands::prompt::model_keys` static call; cfg(test) `dispatch` callers gain the arg; wiring guard test |
| `src/commands/serve.rs` | supply `model_keys` in the `McpConfig` literal |
| `.doctrine/adr/001/layering.toml` | ratchet `command = 123 → 121` |
| `tests/architecture_layering.rs` | fix stale header comment `120 → 121` |

`src/commands/prompt.rs` is **unchanged** (`model_keys` stays put).

## 10. Review Notes

(pending adversarial pass)
