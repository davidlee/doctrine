# CHR-039: Spike: probe Workflow-harness worktree-fork + arming for confined slice-drive

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

De-risks IDE-031 (Workflow-templated slice-driver). The coarse design there
assumes a dynamic **Workflow** script can drive the dispatch funnel by spawning a
`dispatch-orchestrator` (or worker) via `agent(isolation:'worktree')`. That
assumption rests on **unverified harness behaviour** — each probe below can kill or
confirm the whole approach, and every one is **SL-199-independent** (runnable now,
in parallel with SL-199 finishing).

Empirical, like the SL-199 §6 feasibility probe. Resolution = a findings note
answering SQ1–SQ5, feeding IDE-031's design.

## Ground truth

- Workflow scripts have **no filesystem/shell** (`docs/claude/workflows.md:285`
  constraint table) — the script cannot run `dispatch arm-spawn`. Confirmed.
- Full `agent()` opt surface (`isolation`, `schema`, `effort`, `model`, `agentType`,
  `phase`, `label`) + `WorkflowInput`/`Output`: `docs/claude/agent-sdk/typescript.md`
  (§Workflow ~L2001/2523; agent opts ~L1772-1790).
- SL-199 create-fork confined arm: fork iff `cwd_is_coord_root ∧ coord_in_dispatch
  ∧ base` (design §A). Depends on the `WorktreeCreate` payload cwd + an armed `base`.

## Probe questions

- **SQ1 (linchpin).** Does a Workflow-script `agent(isolation:'worktree')` spawn
  **fire the doctrine `WorktreeCreate` hook at all**, or does the runtime fork off
  session HEAD without invoking it? No hook ⇒ no confined-arm ⇒ no base-control ⇒
  coarse design dead as drafted.
- **SQ2.** If it fires, what **cwd** does the payload carry? The script runs in an
  isolated environment "separate from your conversation" (`workflows.md:272`) — can
  §A's `cwd_is_coord_root ∧ coord_in_dispatch ∧ base` ever be satisfied from there?
- **SQ3.** Can `base` be armed at all, given the script can't Bash? Tests whether a
  server-side `dispatch_arm` MCP is **mandatory** vs arming inside a spawned
  orchestrator agent's turn (the coarse fallback).
- **SQ4.** Do the funnel MCP tools (`dispatch_import`/`dispatch_conclude_phase`/
  `dispatch_reap`) run **clean inside a background workflow**, or stall on the
  allowlist gate? Subagents run `acceptEdits` + inherit the allowlist regardless of
  session mode; unallowlisted MCP/shell "can still prompt you mid-run", and headless
  has "no one to prompt" (`workflows.md:162-166`).
- **SQ5.** Is `taskBudget: {total}` (Alpha, `typescript.md:456`) actually live in
  this harness build for pacing a sub-orchestrator (inject remaining-token budget so
  the agent self-paces)?

## Method

Minimal throwaway Workflow (`ultracode`/`use a workflow`) spawning ONE trivial
`agent(isolation:'worktree')` inside a coord tree on a `dispatch/<NNN>` branch,
`base` pre-armed. Observe: fork location (`.worktrees/agent-*` vs session-HEAD
detached), branch, jail record, `WorktreeCreate` firing. Then a second agent that
calls a funnel MCP read to check SQ4 gating. Cross-ref the SL-199
dispatch-harness-findings probe matrix method.

## Outcome

Findings note (probe matrix + SQ1–5 verdicts) → resolves this chore, unblocks
IDE-031 design. If SQ1/SQ2 negative: IDE-031 pivots to server-side arming
(`dispatch_arm`) or drops `isolation:'worktree'` for a fully server-resolved funnel.

Related: [[SL-199]] (confined orchestrator + create-fork §A), IDE-031 (capability),
RFC-011 (token-efficiency benchmark).
