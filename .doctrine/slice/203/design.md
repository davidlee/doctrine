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
- **STD-001** — the ratchet value is single-sourced in `layering.toml`
  (`command`, read at runtime by the gate); no second authored copy exists. The
  `120` figures in the test file are historical SL-112 report prose, **not** a
  mirror of the constant, so they fall outside STD-001's scope (§8 R3).

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
- **Edge — cfg(test) callers.** All test callers live in one `#[cfg(test)] mod
  tests` (opens `tools.rs:1386`): **~13 direct `dispatch(&req, …)` call sites**
  (`tools.rs:1458/1471/1628/1643/1745/1760/1775/1790/1804/1818/1917/1935/1961`)
  **plus the `memory_dispatch` helper** (`tools.rs:1900`, itself calling
  `dispatch`, invoked ~10×). Each direct caller gains the arg passing the real
  `crate::commands::prompt::model_keys`; the helper gains the param and threads
  it (or supplies `model_keys` internally). The layering extractor **excludes
  `#[cfg(test)]` item/module bodies** (`architecture_layering.rs:120` rule;
  `visit_item_mod` early-return `:324-326`), so all of this is invisible to the
  gate — it can neither mask a failure nor forge a pass. Mechanical and
  cfg(test)-confined, but the full surface is **~14 signatures, not four** (a
  `/plan` scope note — a missed call site is a mid-phase compile failure).

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
- **R2 — silent mis-wire.** A **wrong same-signature** `ModelKeysFn` (a plain
  `fn` pointer cannot be null in safe Rust) would compile and pass the
  structure-only VT-1. Mitigated by the wiring guard (§9 VT-3): a test that
  drives `render_onboard` through the injected pointer and asserts a **known
  model key renders** in the section — asserting mere non-emptiness is inert
  (the empty-corpus placeholder always makes the section non-empty; see §9).
- **R3 — ratchet drift (non-goal, clarified).** The live baseline is
  single-sourced at `layering.toml:156` (`command = 123`), read at runtime by
  the gate — that value legitimately ratchets 123→121 in-slice. The `120`
  figures in `architecture_layering.rs` (lines 8, 22) are a **historical SL-112
  `PHASE-01 go/no-go report` snapshot**, not a live constant; STD-001 does not
  govern a dated narrative comment. **Leave the SL-112 report block untouched**
  (it is history) — editing it to 121 would make it neither the SL-112 value nor
  sourced, and there are two occurrences, not one. Not a slice objective.

## 9. Quality Engineering & Validation

- **VT-1 (primary, structural proof).** `cargo test --test
  architecture_layering` green with `[tangle_baseline] command = 121`. What the
  gate proves is the **scalar ratchet** `count_tangle_edges(command) ≤ 121`
  (Assertion 4); a surviving production back-edge holds the count at 123 > 121 →
  RED (load-bearing). The stronger claim that Tarjan no longer *groups*
  `{commands, mcp_server}` is **inferred** from the −2 drop, or shown explicitly
  via the `#[ignore] dump_real_graph` diagnostic (`architecture_layering.rs
  ~:1125`) — the standard gate emits a count, not a membership report. Expected
  exactly −2 (the 2-cycle's two edges), since the SCC is core-separate
  (RSK-227); verified empirically at execute-time, never hard-coded blind.
- **VT-2 (behaviour-preservation).** Full suite green, unchanged.
  `doctrine_onboard` output byte-identical.
- **VT-3 (wiring guard).** A test drives `render_onboard` /
  `dispatch("doctrine_onboard")` through an injected `ModelKeysFn` and asserts
  on **actual key content** — a known corpus model key appears as a `` - `…` ``
  bullet line — **not** merely that the section is present. (Section-non-empty is
  inert: `render_model_band_guidance` emits the `(no model keys in corpus)`
  placeholder under the always-present section header, `tools.rs:1366-1375`, so
  an empty producer would pass.) This proves the injected producer's output
  reaches the rendered section (R2). The one production supply site —
  `run_serve` coercing the real `crate::commands::prompt::model_keys` into
  `McpConfig` (`serve.rs:28`) — rests on compile-time fn-ptr coercion + VA-1, not
  VT-3; VT-3 proves the *thread*, not the *supply*. Add red/green if none exists.
- **VA-1.** `doctrine slice conformance SL-203` clean against the design-target
  selectors (§ code-impact).

Code-impact / design-target touch-set:

| path | change |
|---|---|
| `src/mcp_server/mod.rs` | `ModelKeysFn` type; `model_keys` field on `McpConfig`; `serve` threads it into `dispatch` |
| `src/mcp_server/tools.rs` | +param on `dispatch`/`handle_tools_call`/`call_tool`/`render_onboard`/`render_model_band_guidance`; drop the `crate::commands::prompt::model_keys` static call; **all ~13 cfg(test) `dispatch` callers + the `memory_dispatch` helper** gain the arg (§5.5); wiring guard test |
| `src/commands/serve.rs` | supply `model_keys` in the `McpConfig` literal |
| `.doctrine/adr/001/layering.toml` | ratchet `command = 123 → 121` |
| `tests/architecture_layering.rs` | **no source edit** — run the gate green at `command = 121` (VT-1); the `120` figures (lines 8, 22) are historical SL-112 report prose, not the live baseline (§8 R3) |

`src/commands/prompt.rs` is **unchanged** (`model_keys` stays put).

## 10. Review Notes

**Internal adversarial pass (design author).** No blocking flaws. Confirmed:

- `crate::commands::prompt::model_keys` (`pub(crate)`) coerces to `ModelKeysFn`;
  higher-ranked elided lifetimes match the fn-ptr type.
- `root::find(config.path, …)` moves only `config.path`; `config.model_keys`
  (Copy) stays accessible after the partial move.
- The back edge is a **single** import path (`grep crate::commands
  src/mcp_server/` → only `tools.rs:1365`); no transitive reach. `mcp_server`
  never imported `install`, so INV-1's corpus-agnostic claim is already-true
  for `install` and newly-true for `commands`.
- The −2 ratchet is an *expectation*, verified empirically by the extractor at
  execute-time (VT-1), never hard-coded blind.
- cfg(test) callers of `dispatch` are excluded from the gate graph, so they can
  reference the real `model_keys` without masking/forging VT-1.

Impl nicety for `/plan`: bind `let model_keys = config.model_keys;` before the
serve loop (Copy, but reads clearer than repeated `config.` access).

**External adversarial pass — RV-252 (inquisition; codex GPT-5.5 + Inquisitor).**
Core §7 decision (D-B over D-A) **survived** — no new same-tier edge; D-B
strictly better than D-A on the RSK-227 concentration axis (confirmed
independently). Four charges raised; all reconciled into this design:
- **F-1 (blocker)** — VT-3 was inert (asserted section-non-empty, which the
  empty-corpus placeholder always satisfies). Rewritten to assert on real key
  content; R2 re-characterised (wrong-fn, not null); supply-site coverage stated
  (§8 R2, §9 VT-3).
- **F-2 (major)** — R3/STD-001 misapplied to a historical SL-112 report comment;
  reframed as a non-goal, the report block left untouched (§3, §8 R3, §9 row).
- **F-3 (major)** — §5.5 undercounted the cfg(test) edit surface (4 → ~13
  callers + `memory_dispatch`); corrected (§5.5, §9 row).
- **F-4 (minor)** — VT-1 conflated ratchet-count with SCC-membership proof;
  tightened (§9 VT-1).
