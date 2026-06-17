# SL-084 scope notes

## Scope conversation (2026-06-17)

### pi subagent tool dependency

User raised: pi `subagent` tool depends on `pi-subagents` extension — not built-in.
Doctrine must handle this dependency:
- declare in install docs, or
- detect + refuse at dispatch time, or
- provide `pi -p` fallback (no agent-def contract, needs `env -C`/`cd` wrapping)

Tracked as RSK-3 and OQ-6.

### AGENTS.md separation question

OQ-1: bundle in this slice or split to follow-up? Mechanically simple but may
touch `doctrine boot`. Listed in scope for now; deferrable if design reveals
coupling.

---

# SL-081 dispatch field notes → SL-084

Recorded 2026-06-17 during SL-081 dispatch run. Concrete observations about
harness behaviour, jail interaction, and spawn gaps.

## pi subprocess vs pi subagent (relevant to OQ-2, scope §1)

**Finding:** The `/dispatch-subprocess` skill prescribes `codex exec` for pi
workers. In this bwrap jail:

- `codex exec --cd "$D" -s workspace-write` **edits source correctly** but
  **cannot commit** — the `workspace-write` sandbox blocks writes to `.git/`
  (`fatal: Unable to create '.../index.lock': Read-only file system`).
- `codex exec -s danger-full-access` was not attempted (too coarse).
- pi `subagent` tool (`agent: worker, cwd: "$D"`) **can commit** — it inherits
  the orchestrator's filesystem permissions, no additional sandbox layer.

**Consequence:** The `env -C "$D" codex exec` spawn template in
`dispatch-subprocess/SKILL.md` is **non-functional in bwrap jails** for the
commit step. Workers produce source deltas but can't create `S`.

**Mitigation attempt:** Manual `git apply` of the fork diff into the
coordination worktree, followed by orchestrator commit. Works but defeats the
funnel automation.

**Preference:** pi subagents (`subagent` tool) are the correct mechanism for pi
dispatch workers. The routing table should route pi → subagent, not subprocess.

## fork --base resolution (related to dispatch trunk resolution)

**Finding:** `doctrine worktree fork --base <sha>` resolved `<sha>` against
remote refs, not local. A local-only commit (on `dispatch/081`, not pushed)
failed with `fork-refused: base <sha> is not a commit`.

**Workaround:** Use branch names (`--base dispatch/081`) instead of commit
hashes. Works because local branches are resolvable.

**Note:** Same root cause as the coordinate issue below — `trunk_commit`
defaults to `origin/HEAD` before `main`.

## DOCTRINE_TRUNK_REF necessity (scope §4, RSK-1)

**Finding:** SSH push is disabled in the bwrap jail (`GIT_SSH`→disabled).
Local `main` is ahead of `origin/main` (plan.toml not pushed). Without
`DOCTRINE_TRUNK_REF=main`, `worktree coordinate` fails because the resolved
trunk (`origin/main`) lacks the plan.toml (committed locally at 4d0d71a).

**Consequence:** Every `doctrine worktree coordinate` and `doctrine slice phase`
invocation in this session required `DOCTRINE_TRUNK_REF=main`.

**Design note:** The trunk ladder (`DOCTRINE_TRUNK_REF` → `origin/HEAD` →
`main` → `master`) is correct per ADR-012, but the `origin/HEAD` priority
before `main` breaks when the remote is stale. Consider: can the ladder prefer
the local branch when it's ahead of the remote?

## Worker marker vs e2e tests

**Finding:** The 3 `e2e_adr_cli_golden` tests fail in worker-marked forks
because the marker causes `doctrine adr status` (a write-classed verb) to
refuse. The tests create temp dirs and invoke the doctrine binary, which
detects the fork marker.

**Consequence:** `just check` (which runs all tests + e2e) cannot be the
baseline-verify for worker forks. The 3 e2e failures are expected and harmless
for source editing, but they block the green gate.

**Observation:** The worktree skill's baseline-verify says "An unbuildable fork
is fixed in provisioning, never handed off." The fork *is* buildable — the e2e
failures are marker-mediated, not build failures. The skill doesn't distinguish
these.

## Cross-phase compile ripple (PHASE-02)

**Finding:** PHASE-02 (CatalogKey/CatalogEdgeLabel introduction) caused
compile errors in:
- `src/map_server/routes.rs` — `NodeKey::Entity(key)` → `Numbered(key)`
- `src/relation_graph.rs` — `entity_kinds: BTreeMap<EntityKey, &Kind>`,
  `edge.source` and `edge.label` type changes

These are PHASE-06 and PHASE-07 territory. The plan assumes PHASE-02's
hydrate.rs changes don't ripple into routes/relation_graph, but
`CatalogEntity.key` changing from `EntityKey` to `CatalogKey` touches every
consumer.

**Mitigation:** Applied minimal compile fixes (mechanical type updates).
PHASE-06 and PHASE-07 will find less novel work but the fixes are
behaviourally equivalent to what they would have done.

**Note for dispatch:** When batch file sets are declared, transitive compile
dependencies (other files importing changed types) may need attention. The
"file-disjoint" optimization is valid for source *editing* but may need
orchestrator compile-fix followup.

## Pi subagent model selection (scope §3)

**Finding:** The `subagent` tool's `model` parameter was not needed — the
default worker model performed correctly. The dispatch-worker agent definition
(in `.pi/agents/`) should document the model override surface but not
hardcode a default.

## AGENTS.md leakage observed (scope §4)

**Finding:** `AGENTS.md` includes: "default reviewer: codex mcp — use default
(GPT-5.5) for external adversarial reviews. Opus sub-agent is also useful for
variety on subsequent passes." This is read by pi agents and is nonsensical in
the pi context (pi has no `codex mcp`, no Opus sub-agent). During this
session the instruction was harmless but confirms the leakage described in
SL-084's context.
