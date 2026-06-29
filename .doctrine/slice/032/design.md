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
- **trunk-side id minting** — ids allocate against the shared trunk baseline so a
  minter (the **orchestrator** on a coordination branch, or a **solo** agent) does
  not collide with ids already on trunk. (Note: under the funnel **workers do not
  mint** — the guard refuses it; minting is serial on the coordination branch.
  Trunk-ref allocation is therefore *best-effort* collision **reduction** against
  the fetched baseline, **not** a distributed lock — two unpushed concurrent
  branches can still collide.)
- with a **detect (`validate`) + reseat** backstop for the residual
  offline/unpushed-collision case
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
  tokens in `src/git.rs` are the frozen-frame verb enum + test
  helpers). D2a's "doctrine-driven commits" is therefore moot today.
- **Worktree detection.** `src/worktree.rs` does **not** expose a self-detection
  seam (X8): `verify_sibling_worktree` (private, `:286`) checks whether *two* trees
  share a `git-common-dir` — sibling verification, not "am I on a linked
  worktree?". So the memory warning cannot "reuse" an existing detector; this slice
  **adds** a shared `worktree::is_linked_worktree(root) -> Result<bool>`
  (`git rev-parse --git-dir` ≠ `--git-common-dir`) that `memory record` calls.
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
- **Per-namespace ids (X2)** — every kind numbers from `001` independently
  (`SL-001` + `ADR-001` + `REQ-001` all valid). Duplicate detection and reseat are
  **intra-kind** and key on the **canonical ref**, never a bare number.
- **Repo clippy denies** (`mem.pattern.lint.*`): no indexing-slicing, no
  `as` casts, no `HashSet/HashMap` (use BTree*), `expect`+reason over bare
  `allow`, string-assembly rules. Apply throughout.

## 4. Guiding Principles

- **Ride the existing seam** — the injectable `scan` closure is D3's whole
  mechanism; do not add an allocation framework.
- **Data in, not effects** — the shell resolves trunk ids and the worker flag and
  passes them in; the engine stays pure.
- **Closed-set classification, not fail-open** — `write_class` matches every
  command variant exhaustively (no wildcard), so an unclassified new verb is a
  compile error, not a silent read (X3/X4).
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
3. **`validate` + `reseat`** — new shell verbs over a **per-kind** integrity scan
   (ids are per-namespace — X2).
4. **Memory warning** — add `worktree::is_linked_worktree` (X8), call it from
   `memory record`.

### 5.2 Interfaces & Contracts

**Guard (D2a).**
```rust
// pure, unit-tested — no env, no io. Write carries the verb label for the message.
enum WriteClass { Read, Write(&'static str) }
fn write_class(cmd: &Command) -> WriteClass;   // EXHAUSTIVE match, NO wildcard (X4)
// shell
fn worker_mode() -> bool;            // DOCTRINE_WORKER == "1"
// main(), before the dispatch match:
if let (true, WriteClass::Write(verb)) = (worker_mode(), write_class(&cli.command)) {
    bail!("DOCTRINE_WORKER=1: refusing authored write `{verb}` — workers return a \
           source delta; doctrine-mediated writes funnel through the orchestrator.");
}
```
**`write_class` is exhaustive over every `Command` + subcommand variant — no
`_ => Read` arm (X4).** The enums are closed, so a wildcard buys nothing but a way
for a future write verb to ship unguarded; exhaustiveness makes adding a command a
**compile error until its class is decided** (deletes risk R-4). `Write` covers
**every authored / memory / runtime-state mutation** — not just the narrow
mint/anchor list (X3): `Slice {New, Design, Plan, Notes, Phases, Phase, Status}`,
`Adr {New, Status}`, `Spec {New, Req Add}` (Validate = read), `Backlog {New, Edit}`,
`Memory {Record, Verify}`, and the `Sync/Boot/Skills/Install` writers. This is a
deliberate **superset** of ADR-006 D2a's enumerated set, honouring the broader D2
("workers write none of it") — refusing more is safe, since a worker's only channel
is its source delta. `Show/List/Find/Retrieve/CheckAllowlist/Provision/validate` =
`Read`.

**Trunk-ref allocation (D3).**
```rust
// src/git.rs (impure shell)
fn trunk_tree_ish(root: &Path) -> anyhow::Result<Option<String>>;  // peeled, ladder below
fn trunk_entity_ids(root: &Path, kind_dir: &str) -> anyhow::Result<Vec<u32>>;
//   = ls-tree -d --name-only <tree-ish> -- <kind_dir>/   (kind_dir is ALREADY
//     repo-relative incl. `.doctrine/` — X1: do NOT re-prepend `.doctrine/`);
//     parse numeric basenames; Ok(vec![]) when no trunk / dir absent / bare repo.
// src/entity.rs — extract a genuinely PURE helper; materialise stays the (already
// diskful) writer, gaining only a data param (X5):
fn next_id(local: &[u32], trunk: &[u32]) -> u32;   // pure: max(union) + 1, unit-tested
fn materialise(.., trunk_ids: &[u32]) -> ..;       // Fresh arm: candidate = next_id(&scan_ids(..)?, trunk_ids)
//   trunk_ids INERT for InExisting / named (no alloc). Existing call sites pass
//   `&[]` (mechanical arg update — behaviour assertions unchanged, F5).
```
Ladder (`trunk_tree_ish`): `DOCTRINE_TRUNK_REF` → `origin/HEAD` → `main` →
`master`; each candidate resolved by **`git rev-parse --verify --quiet
<ref>^{commit}`** — the `^{commit}` peel means a ref naming a blob/tag fails *here*,
not silently later at `ls-tree` (X6). First to peel wins; else `Ok(None)`.
**Asymmetry (F4):** an **explicitly set** `DOCTRINE_TRUNK_REF` that fails to peel is
a **hard error** (don't mask a misconfiguration); only its *absence* falls through.
`ls-tree` reads the object DB — **the tree-ish need not be checked out**.

**`validate` + `reseat` (D3 fallback).** Ids are **per-namespace** — `SL-001`,
`ADR-001`, `REQ-001` coexist legitimately (X2). So "duplicate id" means **two
entities *of the same kind* asserting the same canonical id**, and the verbs key on
the **canonical ref**, never a bare number:
```
doctrine validate                     # exit 1 + report if any integrity violation
doctrine reseat <CANONICAL_REF> [--to <NNN>]   # e.g. `reseat SL-031`; exit 1 + dangler list
```
`validate` rules (per kind): (a) dir basename `NNN` == toml `id`; (b) no two
entities of a kind share an `id`; (c) each alias symlink (`SL-031-slug`, `mem.*`)
**targets the dir whose toml id matches the alias's encoded id** — target equality,
not mere resolvability (X7).

### 5.3 Data, State & Ownership

- **No new persistent state.** Trunk ids are read-through (never cached to disk).
  The worker flag and trunk ref are ambient env, owned by the orchestrator that
  spawns the worker / runs allocation.
- **Override env** `DOCTRINE_TRUNK_REF` — sibling to `DOCTRINE_WORKER`; documented,
  no config file (D1, no Rube Goldberg).
- **Reseat** mutates exactly: the entity dir name (`git mv`-free filesystem rename),
  its `id` toml field, and its id/alias symlink — the canonical-id triple. It owns
  nothing else; prose citations are reported, not owned. **Target guard:** `--to
  <NNN>` that names an occupied id is refused (no clobber); default `--to` = the
  next free trunk-aware id (§5.2). **Runtime-tier guard (F3):** if gitignored
  runtime phase state exists for the id (`.doctrine/state/slice/NNN/`, the `phases`
  symlink), reseat **refuses** — that state is keyed by id and reseat does not own
  the disposable tier; the human clears/rebuilds it first. (Reseat targets
  freshly-minted, pre-execution collisions in practice.)

### 5.4 Lifecycle, Operations & Dynamics

- **Allocation under dispatch.** The **orchestrator** (never a worker — guard
  refuses) mints trunk-side (trunk ref = the id baseline) *before* a worktree forks
  (ADR-006 D3); the union scan clears any id present on trunk but absent from the
  minter's local branch (e.g. a coordination branch behind trunk). One `ls-tree`
  per `new` — cheap, and only ever in non-worker context (the guard short-circuits
  workers before dispatch, so no wasted git call under a worker).
- **Offline collision → detect → reseat.** A solo agent on an offline branch (no
  trunk reachable) falls back to local-only scan and may mint a stale id; after
  merge, two entities of the kind assert the same id. `doctrine validate` flags it
  (intra-kind); the human runs `doctrine reseat SL-NNN`, which allocates the next
  free id (trunk-aware §5.2) and prints inbound citations to fix by hand.
- **Worker write attempt.** `DOCTRINE_WORKER=1` + any `Write`-classed verb → bail
  before dispatch; nonzero exit; the worker’s source delta is the only channel.
- **Memory record in a worktree.** `worktree::is_linked_worktree(root)` true →
  stderr warning (squash-orphan risk) → proceed (non-blocking; solo-in-worktree is
  blessed, D6a).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (behaviour preservation, F2).** The gate is the **existing numeric test
  fixtures** (no remote / `origin == HEAD`), where `trunk_ids ⊆ local_ids` ⟹
  `local ∪ trunk == local` ⟹ their behaviour assertions stay green unchanged. This
  is **not** a universal subset claim: when a real `origin/HEAD` is *ahead* of
  local, the union deliberately skips past already-pushed ids — an **intended**
  collision-avoidance change, not a regression (and absent from the fixtures, so
  the gate holds).
- **INV-2 (no NEW impurity, X5).** `materialise` is *already* diskful (it is the
  writer); this slice adds **no** new env/git/disk to `entity.rs` — only a `&[u32]`
  data param — and extracts the genuinely pure `next_id(local, trunk)` for unit
  testing. The claim is "no new impurity," not "materialise is pure."
- **INV-3 (reads always open).** The guard never refuses a `Read`-classified verb.
- **EDGE — no trunk ref** (no remote / detached / bare / fresh repo): empty trunk
  set → local-only allocation. Defined terminus, not an error.
- **EDGE — stale `origin/HEAD`**: collision possible; accepted by ADR-006 D3 and
  backstopped by `validate`+`reseat` — **not** designed out.
- **EDGE — submodule**: trunk read inherits `git.rs`'s existing submodule guard;
  no new exposure.
- **EDGE — reseat vs runtime state (F3)**: an id with live gitignored phase state
  is refused (§5.3) — reseat does not migrate the disposable tier.
- **EDGE — explicit bad override (F4)**: a set-but-unresolvable
  `DOCTRINE_TRUNK_REF` errors hard; only absence falls through the ladder.

## 6. Open Questions & Unknowns

- **OQ-1 (PHASE-03 detail).** Exact `validate` rule set: v1 = §5.2 (a)/(b)/(c) —
  intra-kind dir==id, no duplicate id within a kind, alias target equality. May
  tighten during PHASE-03; not load-bearing for SL-031.
- **OQ-2 (resolved).** Override = `DOCTRINE_TRUNK_REF` env; read via `ls-tree` on a
  peeled tree-ish (no checkout); no-trunk → local fallback.
- **OQ-3 (resolved).** `validate` is a new top-level verb scanning **each kind's**
  namespace (X2 — ids are per-namespace, so the scan is intra-kind, not cross-kind).
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
  `DOCTRINE_WORKER`; honours D1 (no configuration Rube Goldberg). An explicitly set
  ref that fails to peel (`^{commit}`) errors hard rather than silently falling
  through (F4/X6) — silence would mask a misconfiguration as a phantom collision.
- **D6 — exhaustive `write_class`, superset of D2a (X3/X4).** Match every command
  variant with no wildcard; classify every authored/memory/runtime mutation as
  `Write`, not just D2a's mint/anchor list. Honours the broader D2 ("workers write
  none of it") and makes an unclassified new verb a compile error. *Alt rejected:*
  the narrow D2a-only set + `_ => Read` (lets `slice design`/`memory verify`/runtime
  writers slip through, and ships future verbs unguarded).
- **D7 — per-namespace integrity; canonical-ref verbs (X2).** `validate` scans each
  kind independently; `reseat` takes `SL-NNN`, not a bare id. *Alt rejected:*
  cross-kind dup detection (incoherent — `SL-001`/`ADR-001` legitimately coexist)
  and `reseat <id>` (ambiguous target).

## 8. Risks & Mitigations

- **R-1 (engine gate).** Allocation is shared machinery → mitigate by INV-1 +
  running the existing numeric suites unchanged as the PHASE-02 exit gate.
- **R-2 (trunk absent / unreachable).** → defined empty-set fallback (EDGE), solo
  path byte-identical.
- **R-3 (reseat vs prose citations).** → renumber the triple, *report* inbound
  citations, exit nonzero so the human finishes (D4). No silent corruption.
- **R-4 (classifier drift) — DELETED by D6/X4.** Exhaustive matching makes an
  unclassified new verb a *compile error*; there is no drift surface left to
  mitigate.
- **R-5 (D2b residual).** A worker can still raw-edit main — out of scope, ADR-008.
  This slice rests on the CLI guard + prompt contract, as ADR-006 states.

## 9. Quality Engineering & Validation

Per ADR-006 Verification, all **new** tests. The gate is **behaviour
preservation**, not file-immutability: existing `materialise` call sites take a
mechanical `&[]` arg update (permitted by the seam contract); no existing
behaviour **assertion** changes (F5).

- **Guard:** `write_class` unit table asserts a class for **every** `Command` +
  subcommand variant (exhaustive — the compiler enforces totality, X4); for each
  `Write` verb, `DOCTRINE_WORKER=1` ⟹ nonzero exit + verb-named refusal; a
  representative `Read` verb ⟹ unaffected.
- **`next_id` (pure):** unit table over (local, trunk) — empty/empty, trunk-ahead,
  local-ahead, overlap — asserts `max(union)+1`.
- **Trunk allocation:** fixture repo with committed ids ahead of the working tree
  ⟹ next id clears the trunk max; `kind_dir` passed un-prefixed (X1 regression
  test: a planted `.doctrine/slice/NNN` is found); no-trunk fixture ⟹ local-only;
  bad-`^{commit}` override ⟹ hard error (X6); **existing numeric suites green
  unchanged** (INV-1, PHASE-02 exit gate).
- **`validate`:** planted dir/`id` mismatch, planted intra-kind duplicate, and a
  planted **mis-targeted alias** (X7) ⟹ exit 1 + named report; clean corpus ⟹
  exit 0.
- **`reseat`:** colliding `SL-NNN` ⟹ triple renumbered to the next free id, alias
  target correct, inbound-citation danglers listed, exit nonzero; refuses an
  occupied `--to`; refuses an id with live runtime phase state (F3).
- **Memory warning:** `is_linked_worktree` fixture (linked worktree) ⟹ stderr
  warning + record still succeeds; primary tree ⟹ silent.

**Phasing:** PHASE-01 guard (pure leaf, SL-031's hardest dep) → PHASE-02
trunk-ref allocation (engine-gate risk) → PHASE-03 validate+reseat (needs §5.2's
trunk-aware free-id pick) → PHASE-04 memory warning (adds + shares
`is_linked_worktree`, X8).

## 10. Review Notes

### Self-adversarial pass (integrated)

- **F1 — overclaim corrected.** Trunk-ref allocation is best-effort collision
  *reduction*, not a distributed lock; workers never mint (guard refuses), so the
  "concurrent worktrees allocate disjoint ids" framing was wrong. §1, §5.4 fixed.
- **F2 — INV-1 restated.** The gate is the test fixtures (origin==HEAD), not a
  universal `trunk ⊆ local`; ahead-origin skip is an intended change. §5.5 fixed.
- **F3 — reseat vs runtime tier.** Reseat refuses an id with live phase state
  (it does not own the disposable tier). §5.3, §5.5 added.
- **F4 — explicit bad override errors hard;** only absence falls through. §5.2,
  §5.5, §7-D5.
- **F5 — gate is behaviour preservation, not file immutability** — mechanical
  call-site arg updates are allowed. §9 fixed.
- **F6 — minor:** guard message carries the verb label (`Write(&'static str)`);
  `reseat --to` refuses an occupied target; `trunk_ids` inert outside `Fresh`;
  per-`new` `ls-tree` runs only in non-worker context.

### External adversarial pass (codex MCP — gpt-5.x, read-only)

All eight findings verified against source and **accepted**; none spurious. The
two BLOCKERs invalidated load-bearing assumptions the self-pass missed.

- **X1 (BLOCKER, accepted).** `trunk_entity_ids` double-prefixed `.doctrine/` —
  every `Kind.dir` already includes it (`src/slice.rs:38`, `src/spec.rs:51`,
  `src/backlog.rs:60`). Fix: pass `kind.dir` straight to `ls-tree`. §5.2; X1
  regression test in §9.
- **X2 (BLOCKER, accepted).** Ids are **per-namespace** (`SL-001`+`ADR-001`
  coexist), so "cross-kind dup-id" was incoherent and `reseat <bare-id>` had no
  unique target. Fix: intra-kind detection; verbs key on the canonical ref
  (`reseat SL-NNN`). §3, §5.1, §5.2, §6-OQ3, §7-D7.
- **X3 (MAJOR, accepted).** Write set omitted `slice design/plan/notes/phases/
  phase`, `memory verify`, sync/install writers — D2 ("workers write none of it")
  not enforced. Fix: classify every mutation as `Write` (superset of D2a). §5.2,
  §7-D6.
- **X4 (MAJOR, accepted).** `_ => Read` wildcard lets a future write verb ship
  unguarded. Fix: exhaustive match, no wildcard → compile error. §4, §5.2, §7-D6;
  deletes R-4.
- **X5 (MAJOR, accepted).** `materialise` is already diskful — "pure engine" was
  hand-waving. Fix: extract pure `next_id(local, trunk)`; reframe INV-2 as "no new
  impurity." §5.2, §5.5.
- **X6 (MAJOR, accepted).** `rev-parse --verify` accepts a blob/tag → explicit
  override fails late at `ls-tree`, contradicting hard-error-at-resolution. Fix:
  peel `^{commit}` at resolution. §5.2, §7-D5.
- **X7 (MAJOR, accepted).** "symlinks resolve" misses a stale alias pointing at
  *some* live dir. Fix: validate alias **target equality** vs the id. §5.2(c), §9.
- **X8 (MAJOR, accepted).** `worktree.rs` has no self-detection seam
  (`verify_sibling_worktree` @286 is sibling verification) — "reuse" was false.
  Fix: add shared `worktree::is_linked_worktree(root)`. §2, §5.1, §5.4, §9.
