# SL-158 notes — Trinary actionability (context bootstrap)

General-purpose bootstrap for any agent picking up SL-158. Durable trail; the
disposable `handover.md` points here. Authoritative design = `design.md`;
governing canon = **ADR-017**. This file frames + supersedes the earlier
exploration (which predates RFC-008/ADR-017 and is no longer the plan).

## STATUS: design locked pending user approval (2026-06-26)

RFC-008 resolved → **ADR-017 accepted**. Gating = an **inbound `needs` edge on an
unsettled record**. No new relation/role/axis, no projection, no "gating edge" to
build. The earlier P/E/L mechanism fork and OQ-1/OQ-2/OQ-3 are all **closed** by
ADR-017 — ignore them (history below, struck).

Design + scope reconciled, selectors recorded, SL-158→ADR-017 linked. Internal +
external (codex) adversarial passes done and integrated. Next routed step: **/plan**
(unless re-review of D6 or /inquisition is requested first).

## The four changes (see design.md for detail)

1. **D1 — trinary partition** (`src/priority/partition.rs`): add `StatusClass::Gating`
   (non-Workable, non-Terminal) + a `gating` field on `KindPartition`. Knowledge rows
   move unsettled states into `gating`; every other row `gating: &[]` (byte-identical).
2. **D2 — `needs`/`after` target-gate widening** (`src/commands/dep_seq.rs`): split the
   one `is_work_like` gate — **source** unchanged (records still can't author dep/seq),
   **target** widens to `is_admissible_dep_target` = work-like ∪ records. Governance
   excluded. *This is the delta ADR-017's prose missed* (see "Gate contradiction").
3. **D3 — estimate/value on records**: already works (no facet code). Inert via leverage,
   **live via optionality** once D6 lands. `risk` excluded (gated + `[facet]` collision).
4. **D6 — records author `references` (`concerns`)** (`src/relation.rs` RELATION_RULES):
   user-requested; records were illegally barred. Distinct from `shapes`-roles (IDE-022).

Canon (SPEC-001/PRD-011/SPEC-019) moves via **reconcile**, not hand-edited ahead of code.

## Gate contradiction (the load-bearing design finding)

ADR-017 claims the `needs` work-like gate is *source-only* ("a work item may target a
record today"). **False.** `src/commands/dep_seq.rs:59-65` gates the **target** as
work-like too → `doctrine needs SL-x QUE-1` is **refused today**. So the trinary
partition is NOT the sole engine delta — D2 (gate widening) is required to make the
inbound gate authorable. The *read/build* path (`graph.rs`) IS kind-agnostic; only the
*authoring gate* refuses records. **ADR-017 prose reconciled at close** (follow-up).

## Key engine findings (still valid — confirmed in code)

### partition.rs — binary → trinary
- `StatusClass { Workable, Terminal, Unrecognised }`; `status_class(kind, status)` is a
  `(prefix → KindPartition{workable,terminal})` lookup. Add `gating` set + check.
- VT-1 drift canary per kind: generalise `workable ∪ terminal == <KIND>_STATUSES` to the
  three-way cover.
- Knowledge kinds currently `workable:&[]`, `terminal:<KIND>_STATUSES` (all-Terminal).
- Per-kind settle boundary (matches `knowledge.rs:173-180`):
  - ASM `held`/`testing` → Gating; `validated`/`invalidated`/`obsolete` → Terminal
  - DEC `proposed` → Gating; `accepted`/`rejected`/`superseded` → Terminal
  - QUE `open` → Gating; `answered`/`obsolete` → Terminal
  - CON `active` → Gating; `waived`/`superseded`/`retired` → Terminal
- Tests that flip BY DESIGN (consumer revision, not regression):
  `every_knowledge_status_classifies_terminal_never_workable`,
  `knowledge_partitions_cover_the_real_vocabularies`. **Stays green:**
  `decision_accepted_diverges_hidden_from_status_class` (`accepted` ∈ terminal).

### channels.rs — ZERO code change (the elegant part)
- `eligible == Workable` → Gating off the worklist automatically.
- `blocked_by` keeps predecessors `!= Terminal` → a Gating predecessor blocks; settling
  → Terminal unblocks. **Settle→unblock is free.** No exhaustive `match` on StatusClass
  here — comparison predicates absorb the new variant.

### graph.rs — ZERO code change (kind-agnostic needs build)
- dep overlay fed by `needs`/`after` (`relation_graph::dep_seq_for`), kind-blind:
  `graph.rs:344` resolves any `needs` target + emits `prereq→src`. Records are scanned
  nodes (`catalog/scan.rs:199`, `integrity.rs:113`). So a `needs → record` edge builds
  with no change once D2 lets it be authored.
- **leverage DP flows dependent→prereq** (`graph.rs:513`). A record is always a prereq
  with no dep predecessors (can't author `needs`) → its base never propagates via
  leverage. **optionality** (`graph.rs:163` CONSEQUENCE_LABELS incl `References`,
  filtered by label not role) flows referrer-base → target → so D6 makes a record's base
  live via the targets it `references`.

### Surface output flips with NO code change (codex finding)
- `render.rs:184` prints class via `{:?}` (Debug) — Gating compiles + prints, no forced
  edit. But `survey --all` (bypasses eligibility, `surface.rs:136`), `explain`, `inspect`
  render `StatusClass`/score for any minted node → unsettled records flip `Terminal →
  Gating` in output. Intended; pin with a knowledge-in-priority golden (VT-8). Existing
  priority goldens cover work/backlog/review only (`tests/e2e_priority_golden.rs:105`).

## Codex adversarial pass — disposition (2026-06-26)

External (codex MCP, read-only). One MAJOR: my "record score never displayed" claim was
false (survey --all/explain/inspect surface it) — **corrected** in design.md. MINORs
(surface blast radius, missing knowledge golden) → folded as VT-8. Confirmed CORRECT: D2
required; `graph.rs` kind-agnostic; record→record needs excluded + no dep cycle;
`ensure_ref_resolves` accepts knowledge refs (KINDS table); D1 boundary matches consts.
D6 was added AFTER this pass — self-verified only (optionality wiring at graph.rs:163).

## Verification map (design.md §4)

VT-1 canary (3-way cover) · VT-2 class boundary · VT-3 gate blocks · VT-4 settle→unblock
· VT-5 record never eligible · VT-6 admissibility (`needs SL QUE` ok; gov refused; record
source refused) · VT-7 estimate round-trip · VT-8 knowledge-in-priority golden · VT-9
references authoring + optionality.

## Split out / follow-ups

- **IDE-022** — `shapes`-roles (semantic disambiguation, ADR-016). Different from D6.
- **IMP-183** — surface estimate/value in show/inspect for all estimable kinds.
- ADR-017 prose reconciliation (source-only premise correction) — at close.
- Outbound gating hub-view + deferred batch sugar (ADR-017 §3).
- Coordinate with IMP-033 (full cross-tier dep/seq) — D2 widens only to records.

## Guardrails / environment

- Behaviour-preservation: ordinary workable/terminal items unchanged; existing priority
  suites green; trinary reduces to binary where `gating == ∅`.
- Canon-first: SPEC-001/PRD-011/SPEC-019 via design→reconcile, not hand-edited.
- Jail: writes need `DOCTRINE_RESERVATION_FALLBACK=1`. `link` flag is `--role`. RW
  doctrine = build target (`./target/debug/doctrine`); `~/.cargo/bin/doctrine` is RO.
- Lint: `just check` inner loop / `just gate` before commit; plain `cargo clippy` (NOT
  `--all-targets`); repo denies `as`/`unwrap`/`expect`/`print_stdout`/`format_push_string`
  — build Strings via `Vec<String>` + concat (`mem.pattern.lint.*`).
- Shared index: path-limit `git add`/`commit`; watch `.git/index.lock` (other agents).

---
*## Dispatch spawning issues (2026-06-26)

**Claude-arm agent dispatch broken — WorktreeCreate hook does NOT fire.**

1. `isolation: worktree` on `subagent(agent: "dispatch-worker")` lands the worker in
   the **main worktree** (`edge` branch), not an isolated linked worktree. The
   `base` file in the arming dir (`<coord>/.doctrine/state/dispatch/spawn/base`) is
   correctly written by `arm-spawn`, but the hook that should read it and create a
   fresh worktree fork never executes.
2. This was confirmed across 3+ spawn attempts from the correct cwd
   (`<coord>/.doctrine/state/dispatch/spawn/`). Every time:
   - git-dir == git-common-dir == `.git` (main tree)
   - HEAD == `edge` branch tip (not a new dispatch branch)
   - The dirty files from the main tree are visible to the worker
3. **Workaround:** manually `worktree fork --worker` to create the isolated fork,
   then spawn the dispatch-worker agent with `cwd` pointing to the fork directory
   (no `isolation: worktree`). This works correctly — the agent runs in the
   isolated worktree, commits there, and returns.
4. This means the orchestrator must do both the fork AND the agent spawn (two-step
   instead of the designed one-step `arm-spawn + agent`). The `arm-spawn` step is
   unnecessary with this workaround since the fork creates its own base.

**Root cause hypothesis:** the WorktreeCreate hook is a doctrine feature that
requires integration with the harness's agent spawning machinery. In this
environment (pi harness), the hook is either not installed, not discoverable,
or not compatible with the agent spawning path.

**Impact on this dispatch:** PHASE-01 was completed via the workaround
(manual fork → cwd-spawn). All three phases will follow the same pattern.

---

*Superseded (pre-ADR-017): the "shapes-projection (P) vs gates-axis (E) vs Gates-label
(L)" fork, OQ-1 (name → `Gating`), OQ-2 (edge direction), OQ-3 (mechanism), and the
"graph.rs gating edge is the real second change" framing. ADR-017 closed all of them:
inbound `needs`, no new edge. Kept out of the live notes to avoid misleading the planner.*
