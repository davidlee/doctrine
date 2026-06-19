# SL-117 Design — `claude-force-subprocess-dispatch`

## Current behaviour

The dispatch router (`dispatch/SKILL.md` step 3) selects the worker arm by
env-marker inference alone: a Claude orchestrator uses `/dispatch-agent` (native
`Agent` subagent tool), a codex/pi orchestrator uses `/dispatch-subprocess`
(subprocess spawn). There is no config key to override this — a project cannot
tell a Claude orchestrator to dispatch subprocess workers even when that would be
preferred (e.g. for pi RPC structured outcomes, process isolation, or
reproducibility).

The existing `[dispatch] preferred-subprocess-harness` key (IMP-101,
`dispatch_config.rs`) only selects *between* subprocess harnesses (codex vs pi)
once the subprocess arm is already chosen — it does not influence arm selection.

## Target behaviour

A new `[dispatch] claude-force-subprocess-dispatch` boolean key lets a project
override the default: when `true`, Claude orchestrators route workers to
`/dispatch-subprocess` instead of `/dispatch-agent`. When `false` (absent or
explicit), behaviour is unchanged — env-marker inference rules.

On non-Claude orchestrators (codex/pi) the key is inert — they have no native
subagent to override and always use `/dispatch-subprocess`.

The existing `preferred-subprocess-harness` key remains the authority over codex
vs pi *within* the subprocess arm. The two keys are orthogonal layers:

| Key | Decision |
|---|---|
| `claude-force-subprocess-dispatch` | Which **arm**: agent vs subprocess |
| `preferred-subprocess-harness` | Which **harness within subprocess arm**: codex vs pi |

## Routing decision tree

```
if claude_force_subprocess_dispatch == true:
    → /dispatch-subprocess (respect preferred_subprocess_harness)
else:
    → env-marker inference:
        orchestrator is Claude  → /dispatch-agent
        orchestrator is codex/pi → /dispatch-subprocess
```

The orchestrator detects its own harness by the same env-marker logic used in
`boot.rs` `resolve_harnesses`: presence of `.claude/` → Claude, otherwise
codex/pi. In practice the orchestrator LLM knows its harness implicitly (the
skill is read by the agent that *is* the harness), so the detection is
self-evident at routing time.

## Code impact

### `src/dispatch_config.rs` — one new field

```rust
/// Force Claude orchestrators to use the subprocess dispatch arm
/// (codex/pi) even though the native `Agent` subagent tool is available.
/// Defaults to `false` (use native subagents where available).
/// Inert on non-Claude orchestrators.
#[serde(default)]
pub(crate) claude_force_subprocess_dispatch: bool,
```

Added to `DispatchConfig`. No new enum — it's a `bool` with serde `default` (`false`).
`SubprocessHarness` is untouched.

### `.agents/skills/dispatch/SKILL.md` — step 3 prose

Replace the inference-only routing prose with the config-first chain:

> If `claude-force-subprocess-dispatch` is `true`, route workers via
> `/dispatch-subprocess` (respect `preferred-subprocess-harness` for codex vs pi).
> Otherwise, route per env-marker inference: Claude orchestrators →
> `/dispatch-agent`, codex/pi orchestrators → `/dispatch-subprocess`.

### Other files — no change

- `src/dtoml.rs` — `DispatchConfig` already derives `Default`; the new field
  deserialises automatically via the outer struct's `#[serde(default)]`.
- `src/main.rs` — no programmatic consumer; the orchestrator LLM reads config via
  skill prose.
- `doctrine.toml` — no required change; the project may optionally set the key.

## Verification alignment

| What | How |
|---|---|
| Parse `true` | Unit test: `claude-force-subprocess-dispatch = true` → field `true` |
| Parse `false` | Unit test: `claude-force-subprocess-dispatch = false` → field `false` |
| Absent key | Unit test: empty `[dispatch]` or missing key → field `false` (serde default) |
| Absent table | Existing `absent_tables_yield_defaults` test still passes |
| Round-trip through `dtoml::parse` | New test: populated `[dispatch]` with both keys parses correctly |
| Combined keys | Unit test: `claude-force-subprocess-dispatch = true` + `preferred-subprocess-harness = "pi"` in same `[dispatch]` table → both fields correct |

## Design decisions

| Decision | Rationale |
|---|---|
| **Bool, not enum** | The override is binary: force subprocess or don't. A two-value enum (`"native"` / absent) is a verbose bool with no upside. |
| **Name `claude-force-subprocess-dispatch`** | Says what it does: forces Claude to subprocess dispatch. Scoped by the `[dispatch]` table. |
| **Inert on codex/pi** | Codex/pi have no native subagent to override; making the key conditional on the orchestrator prevents false signals. |
| **Separate key, not merged enum** | `preferred-subprocess-harness` owns the codex-vs-pi choice within the subprocess arm. Merging into a single three-variant enum would conflate orthogonal decisions (arm selection vs harness selection) and force every config to restate the subprocess preference even when native subagents are the default. |
| **Default `false`** | Preserves current behaviour exactly — no project is forced to change. |
| **No CLI flag** | Non-goal per slice scope; `worktree fork` already has its own concerns. |

## Assumptions & risks

- **`preferred-subprocess-harness` consumption (IMP-101 scope).** The
  `dispatch-subprocess/SKILL.md` currently presents both spawn templates (codex
  and pi) as separate labeled blocks with no config-driven selection prose.
  Wiring `preferred-subprocess-harness` into that skill's routing is IMP-101's
  responsibility. SL-117 only needs the dispatch router step 3 to *direct* the
  orchestrator to `/dispatch-subprocess`; the sub-arm harness selection within
  that skill is a separate, pre-existing concern.

## References

- **ADR-011** (D3) — Claude's `Agent` tool is the first-class subagent arm,
  not a degraded rung. This design adds an opt-out, not a demotion.
- **IMP-101** — landed `preferred-subprocess-harness` and `dispatch_config.rs`.

## Remaining open questions

None. The design is small, bounded, and has no unresolved dependencies.

## Non-goals (per slice scope)

- Changing the subprocess spawn template (codex/pi) — IMP-101 covers that.
- Adding a CLI `--harness` override flag on `worktree fork`.
- Generalising the `Harness` enum in `boot.rs` — dispatch_config's concept is narrower and independent.
