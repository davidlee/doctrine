# Authoring doctrine entities cannot be fanned out via /dispatch

The `/dispatch` orchestrator funnels **source deltas** from worktree-isolated
workers: workers write **source only**, the orchestrator is the sole
doctrine-mediated writer, and the **R-5 belt halts** on any worker delta that
touches `.doctrine/` authored trees (`git diff --name-only B..S` under
`.doctrine/` ⇒ report+halt). Worker-mode also disarms the doctrine CLI's write
verbs.

So authoring doctrine **entities** — specs (`spec new`/spine TOML/`spec req add`),
ADRs, backlog items, requirements, memory — produces **zero source delta** and is
exactly what R-5 rejects. A spec/entity-authoring fan-out (e.g. SL-021 PHASE-04,
~13 specs) is therefore **not** a `/dispatch` target.

Use instead:
- **Plain `Agent` sub-agents** (no worktree isolation) that draft or author, with
  the orchestrator running the CLI writes + gates (`spec validate`, VH gates,
  commit). Heavy reads live in the disposable sub-context; the orchestrator stays
  lean. This is how SL-021 PHASE-02 authored the exemplar trio.
- or **serial `/execute`** when order is forced (e.g. `parent` FK must resolve:
  root → container → component).

`/dispatch` fits parallel **source/code** changes (file-disjoint batches), not
entity authoring. Established SL-021 PHASE-02, 2026-06-11.
