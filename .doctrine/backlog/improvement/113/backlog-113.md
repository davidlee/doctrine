# IMP-113: MCP review-output token efficiency: skip formatted on Listed/Status, summary view for show, opt-in limit for list

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The review MCP verbs serialize the whole `ReviewOutput` over the wire
(`handle_tools_call` → `serde_json::to_string(&out)`). The verb *descriptions*
are tight; the *payloads* leak tokens. Three findings, smallest blast radius
first. Sibling of ISS-033 — same seam (MCP reusing CLI-shaped structs without
minding what the wire actually needs). `ReviewOutput` is `Serialize`-only (no
round-trip), so wire-shape changes never touch the CLI, which renders via
`print_review`.

## 1 — `formatted` double-emit on `Listed` / `Status` (defect)

`#[serde(skip)]` guards `formatted` on the `Showed`/`Unlocked` variants but NOT
on `Listed`/`Status` (`review.rs:354,376`). So MCP returns the structured data
*and* the pre-rendered human string carrying the same data again — e.g.
`review_status` emits `stale_paths: [...]` then repeats all paths inside
`formatted: "...cache: stale (.gitignore, ...)"`. ~2× redundancy on every list
and status call. `formatted` is a CLI render cache; no MCP consumer needs it.

**Fix:** add `#[serde(skip)]` to `formatted` on `Listed` and `Status` — making
all four render-carrying variants consistent. One line each.

## 2 — `review_show` has no projection (gap)

Always emits the full `body` (entire brief markdown) plus every finding's full
`detail` + `response` prose (RV-029's show was ~5KB). To ask "which findings are
open?" an agent must swallow the whole brief. `format:"json"` is *more* verbose,
not less.

**Fix (MCP-local):** optional `view` param — `full` (default, unchanged) |
`summary`. In `summary` the `review_show` arm blanks `body` and clears each
finding's `detail` + `response`, keeping id / status / severity / title /
disposition. Engine (`run_show`) untouched; the arm trims the returned `Showed`
before serialization.

## 3 — `review_list` is uncapped (gap)

Returns every RV (33+ today), growing linearly. Default stays uncapped (no
surprising behaviour change, no silent truncation); the agent opts in.

**Fix (MCP-local):** optional `limit` integer — when set, the `review_list` arm
truncates `rows` to the first `limit`. Opt-in ⇒ the cap is agent-requested, not
a silent engine cap (boot: no silent caps). Shared `listing` engine untouched.

## Verification

- #1 red: a test serializing a `review_list` / `review_status` MCP call and
  asserting the JSON has no `formatted` key. Currently present.
- #2: `review_show` `{view:"summary"}` returns empty `body` and findings with
  empty `detail`/`response`; `full` (and default) unchanged.
- #3: `review_list` `{limit:2}` returns 2 rows; no limit returns all.
- Regression: `print_review` (CLI) output unchanged — `formatted` is read via
  field access, not serde; `ReviewOutput` has no `Deserialize`.
