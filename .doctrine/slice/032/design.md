# Design SL-032: Worker-mode CLI guard and trunk-ref id allocation with reseat

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Build the doctrine-mediated enforcement half of ADR-006's worker-sole-writer
invariant (**D2a**) plus fork-safe id allocation (**D3**), so the orchestrator
funnel (SL-031) has a CLI it can trust under parallel dispatch:

- a **worker-mode guard** that hard-refuses authored writes when a worker is
  running, and
- **trunk-side id minting** so concurrent worktrees allocate disjoint ids,
- with a **detect (`validate`) + reseat** backstop for the offline-collision case
- and a **worktree-context warning** on `memory record` (ADR-006 amendment).

Out: raw-tree confinement (D2b, ADR-008); the funnel itself (SL-031).

## 2. Current State

- **Id allocation.** `entity::materialise` (`src/entity.rs:237`) dispatches a
  `Fresh` placement to `allocate_fresh`, which already accepts an injected
  `scan: impl FnMut() -> Vec<u32>` closure — `materialise` hardcodes it as
  `|| scan_ids(&tree_root)` (a local working-tree read). `candidate_id` takes
  `max+1`; the generic `Claim` (mkdir) retries on `AlreadyHeld` (lost race). The
  scan injection point is the seam D3 rides.
- **Engine contract** (`mem.system.engine.identity-claim-seam`): numeric callers
  stay **behaviour-preserving** — their suites are the gate; signatures may change
  mechanically, observable behaviour may not. Pure/imperative split: **no git,
  clock, rng, disk in the engine core — pass them in as inputs** (the date/uid
  pattern).
- **CLI dispatch.** One central `match cli.command` in `main()`
  (`src/main.rs:870`); each write verb calls a `*::run_*` handler.
- **Commits.** Doctrine creates **no** commits in production paths (the `commit`
  tokens in `src/git.rs` are the the external decision register event-store verb enum + test
  helpers). D2a's "doctrine-driven commits" is therefore moot today.
- **Worktree detection.** SL-029 shipped `GIT_DIR != GIT_COMMON` detection in
  `src/worktree.rs` — reused here, not reimplemented.
- **`validate`.** No top-level command. A per-kind `spec validate` exists
  (`SpecCommand::Validate`); dup-id detection is cross-kind, so it does not ride
  that surface.
- **Git helpers.** `src/git.rs` has `git_text`/`git_opt`/`run_git`,
  remote-selection, and submodule rejection — the trunk-ref ladder reuses them.

## 3. Forces & Constraints

- **ADR-006 D2a/D3** govern; D1 ("no configuration Rube Goldberg", solo path
  untouched) constrains the override surface and the no-trunk fallback.
- **Behaviour-preservation gate** — the numeric allocation suites must stay green
  **unchanged**.
- **Pure/imperative split** — the guard reads `DOCTRINE_WORKER` (impure) and the
  trunk scan reads git (impure); both live in the shell, not the engine core.
- **ADR-004** — relations are outbound-only and prose-only in v1; reseat cannot
  reliably auto-rewrite inbound citations.
- **Repo clippy denies** (`mem.pattern.lint.*`): no indexing-slicing, no
  `as` casts, no `HashSet/HashMap` (use BTree*), `expect`+reason over bare
  `allow`, string-assembly rules. Apply throughout.

## 4. Guiding Principles

- **Ride the existing seam** — the injectable `scan` closure is D3's whole
  mechanism; do not add an allocation framework.
- **Data in, not effects** — the shell resolves trunk ids and the worker flag and
  passes them in; the engine stays pure.
- **Fail open for reads, closed for the named writes** — the guard default-allows
  unknown verbs; the write set is explicit and tested.
- **Detect ≠ repair** — `validate` reports; `reseat` renumbers and *reports*
  danglers; neither silently mutates prose.

## 5. Proposed Design

### 5.1 System Model

```
        ┌────────────────────── shell (src/main.rs, src/*.rs handlers) ──────────────────────┐
        │  worker_mode():bool        trunk ladder + git::trunk_entity_ids() : Vec<u32>        │
        │  write_class(&Command)     (impure: env, git)                                       │
        └───────┬───────────────────────────────┬─────────────────────────────────────────────┘
                │ refuse if worker & write       │ trunk_ids passed as data
                ▼                                ▼
        (bail before dispatch)         entity::materialise(.., trunk_ids:&[u32])  ← pure union
                                               local ∪ trunk  → candidate_id → Claim(mkdir)
```

Three near-independent additions plus one reuse:
1. **Guard** — pure classifier in the shell, gate in `main()`.
2. **Trunk-ref allocation** — impure resolver in `git.rs`; pure union in the engine.
3. **`validate` + `reseat`** — new shell verbs over an integrity scan.
4. **Memory warning** — reuse `worktree.rs` detection in `memory record`.

### 5.2 Interfaces & Contracts

**Guard (D2a).**
```rust
// pure, unit-tested — no env, no io
enum WriteClass { Read, Write }
fn write_class(cmd: &Command) -> WriteClass;
// shell
fn worker_mode() -> bool;            // DOCTRINE_WORKER == "1"
// main(), before the dispatch match:
if worker_mode() && matches!(write_class(&cli.command), WriteClass::Write) {
    bail!("DOCTRINE_WORKER=1: refusing authored write `{verb}` — workers return a \
           source delta; doctrine-mediated writes funnel through the orchestrator.");
}
```
Write set classified `Write`: `Slice/Adr/Spec/Backlog New`, `Memory Record`,
`Adr/Slice Status`, `Spec Req Add`, `Backlog Edit`. All else `Read`.

**Trunk-ref allocation (D3).**
```rust
// src/git.rs (impure shell)
fn trunk_ref(root: &Path) -> Option<String>;     // ladder, below
fn trunk_entity_ids(root: &Path, kind_dir: &str) -> anyhow::Result<Vec<u32>>;
//   = ls-tree -d --name-only <ref> -- .doctrine/<kind_dir>/ ; parse numeric basenames;
//     Ok(vec![]) when no trunk ref / dir absent / bare repo.
// src/entity.rs (pure engine) — signature widens, behaviour preserved
fn materialise(kind, claim, project_root, request, inputs, trunk_ids: &[u32]) -> ..;
//   Fresh scan closure becomes: || Ok(scan_ids(&tree_root)?.into_iter()
//                                      .chain(trunk_ids.iter().copied()).collect())
```
Ladder (`trunk_ref`): `DOCTRINE_TRUNK_REF` (if set & resolves) → `origin/HEAD` →
`main` → `master`; first to `git rev-parse --verify --quiet` wins; else `None`.
`ls-tree` reads the object DB — **the ref need not be checked out**.

**`validate` + `reseat` (D3 fallback).**
```
doctrine validate                     # exit 1 + report if any integrity violation
doctrine reseat <ID> [--to <NNN>]     # renumber a colliding entity; exit 1 + dangler list
```

### 5.3 Data, State & Ownership

- **No new persistent state.** Trunk ids are read-through (never cached to disk).
  The worker flag and trunk ref are ambient env, owned by the orchestrator that
  spawns the worker / runs allocation.
- **Override env** `DOCTRINE_TRUNK_REF` — sibling to `DOCTRINE_WORKER`; documented,
  no config file (D1, no Rube Goldberg).
- **Reseat** mutates exactly: the entity dir name (`git mv`-free filesystem rename),
  its `id` toml field, and its id/alias symlink — the canonical-id triple. It owns
  nothing else; prose citations are reported, not owned.

### 5.4 Lifecycle, Operations & Dynamics

- **Allocation under dispatch.** Orchestrator mints trunk-side (trunk ref = the id
  baseline) *before* a worktree forks (ADR-006 D3); union scan guarantees a fresh
  id even if the local fork tree is behind.
- **Offline collision → detect → reseat.** A sandbox-denied worker falls back to
  local-only scan (no trunk reachable) and may mint a stale id; after merge,
  `doctrine validate` flags the duplicate; the human runs `doctrine reseat <ID>`,
  which allocates the next free id (trunk-aware §5.2) and prints inbound citations
  to fix by hand.
- **Worker write attempt.** `DOCTRINE_WORKER=1` + an authored-write verb → bail
  before dispatch; nonzero exit; the worker’s source delta is the only channel.
- **Memory record in a worktree.** Detect `GIT_DIR != GIT_COMMON` → stderr warning
  (squash-orphan risk) → proceed (non-blocking; solo-in-worktree is blessed, D6a).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (behaviour preservation).** Solo mode ⟹ `trunk_ids ⊆ local_ids` ⟹
  `local ∪ trunk == local` ⟹ numeric suites green **unchanged**.
- **INV-2 (pure engine).** No env/git/disk added to `entity.rs` core — only a
  `&[u32]` data param.
- **INV-3 (reads always open).** The guard never refuses a `Read`-classified verb.
- **EDGE — no trunk ref** (no remote / detached / bare / fresh repo): empty trunk
  set → local-only allocation. Defined terminus, not an error.
- **EDGE — stale `origin/HEAD`**: collision possible; accepted by ADR-006 D3 and
  backstopped by `validate`+`reseat` — **not** designed out.
- **EDGE — submodule**: trunk read inherits `git.rs`'s existing submodule guard;
  no new exposure.

## 6. Open Questions & Unknowns

- **OQ-1 (PHASE-03 detail).** Exact `validate` rule set: v1 = (a) dir basename ==
  toml `id`; (b) no duplicate canonical id within a kind; (c) entity symlinks
  resolve. May tighten during PHASE-03; not load-bearing for SL-031.
- **OQ-2 (resolved).** Override = `DOCTRINE_TRUNK_REF` env; read via `ls-tree`
  (no checkout); no-trunk → local fallback.
- **OQ-3 (resolved).** `validate` is a new top-level verb (cross-kind), not a
  per-kind rider.
- **OQ-4 (resolved).** Reseat = renumber the canonical-id triple + report
  danglers; no auto prose-rewrite (R-3).
- **OQ-5 (resolved).** Allocation composition = injected union at the existing
  `scan` seam.

## 7. Decisions, Rationale & Alternatives

- **D1 — guard at a central `main()` gate over a pure classifier.** Auditable,
  one diff site, respects the pure/imperative split (env read once in the shell).
  *Alt rejected:* per-handler guard calls (a new write verb silently ships
  unguarded); engine-seam guard (forces env into the near-pure engine).
- **D2 — trunk allocation via the existing injected scan, union semantics.**
  Minimal, behaviour-preserving (INV-1), no allocation framework. *Alt rejected:*
  replacing the local scan with a trunk-only scan (breaks solo + uncommitted ids).
- **D3 — trunk ids read by `ls-tree`, never cached.** Stateless, correct under a
  moving trunk; no cache-invalidation surface. *Alt rejected:* a derived id cache
  (new staleness class for no benefit).
- **D4 — reseat reports danglers, does not rewrite prose.** Bounded by ADR-004
  (outbound-only prose relations); silent rewrite would risk corrupting citations.
- **D5 — `DOCTRINE_TRUNK_REF` env override, no config file.** Consistent with
  `DOCTRINE_WORKER`; honours D1 (no configuration Rube Goldberg).

## 8. Risks & Mitigations

- **R-1 (engine gate).** Allocation is shared machinery → mitigate by INV-1 +
  running the existing numeric suites unchanged as the PHASE-02 exit gate.
- **R-2 (trunk absent / unreachable).** → defined empty-set fallback (EDGE), solo
  path byte-identical.
- **R-3 (reseat vs prose citations).** → renumber the triple, *report* inbound
  citations, exit nonzero so the human finishes (D4). No silent corruption.
- **R-4 (classifier drift).** A new write verb omitted from `write_class` ships
  unguarded → mitigate: the classifier is one central `match` a reviewer sees, and
  each verb's write/read class is unit-asserted.
- **R-5 (D2b residual).** A worker can still raw-edit main — out of scope, ADR-008.
  This slice rests on the CLI guard + prompt contract, as ADR-006 states.

## 9. Quality Engineering & Validation

Per ADR-006 Verification, all **new** tests (no existing-test edits — the gate):

- **Guard:** for each write-classed verb, `DOCTRINE_WORKER=1` ⟹ nonzero exit +
  refusal message; a representative read verb ⟹ unaffected. `write_class` unit
  table covers every `Command` variant.
- **Trunk allocation:** fixture repo with committed ids ahead of the working tree
  ⟹ next id clears the trunk max; no-trunk fixture ⟹ local-only; **existing
  numeric suites green unchanged** (INV-1, PHASE-02 exit gate).
- **`validate`:** planted dir/`id` mismatch and planted duplicate ⟹ exit 1 +
  named report; clean corpus ⟹ exit 0.
- **`reseat`:** colliding entity ⟹ triple renumbered to the next free id, symlink
  resolves, inbound-citation danglers listed, exit nonzero.
- **Memory warning:** worktree-context fixture ⟹ stderr warning + record still
  succeeds; non-worktree ⟹ silent.

**Phasing:** PHASE-01 guard (pure leaf, SL-031's hardest dep) → PHASE-02
trunk-ref allocation (engine-gate risk) → PHASE-03 validate+reseat (needs §5.2's
trunk-aware free-id pick) → PHASE-04 memory warning (smallest; rides the worktree
detection).

## 10. Review Notes

(Adversarial pass appended after the draft is committed.)
