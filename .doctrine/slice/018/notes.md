# SL-018 — implementation notes (durable)

Durable cross-phase facts. Disposable scratch lives in the gitignored phase
sheets (`state/slice/018/phases/`); the handover is also disposable. This file
survives — keep only what a future agent needs and can't re-derive cheaply.

## PHASE-01 — DONE (commit `5c6e0ce`)

The scripture gate. Landed:
- **ADR-002** (`.doctrine/adr/002/`, status `accepted`) — sanctions the
  **global / unanchored / path-scoped / derived** memory class, defined by the
  signature **`repo="" && anchor_kind=none`** (NOT a new `memory_type`), plus the
  **fourth staleness disposition**: that class is *evergreen / reference-grade*,
  decay-exempt, rendering a non-decaying **`reference`** state.
- **`doc/memory-spec.md`** amended at three consistent sites: § Scope & anchoring
  (scoped+unanchored carve-out), § Retrieval partition (`repo=""` admitted in
  every partition), § Retrieval Staleness (4th table row + `reference` added to
  the explicit-state enum).

## Corrected seam refs (supersede plan.toml / the old handover)

Verified against source this session — the authored `plan.toml` carries a couple
of stale line refs. These are the real ones:
- **`read_body` lives in `src/memory.rs:1059`**, signature `read_body(items_root,
  uid)` — NOT `retrieve.rs:780`. Single caller `retrieve.rs:772`; **no direct
  test caller** → safe to re-key to `root` for the cross-root (items→shipped)
  fallback.
- **`base_filter` (`retrieve.rs:169`) ALREADY admits `repo=""`** in any partition
  — documented at `:173-174` (review B20). So the "admission" work is a **golden
  test only**, no `base_filter` code change. (The dormant hatch is real: zero
  `repo=""` memories exist today because `record` always derives a non-empty repo
  via the write gate `memory.rs:753`.)
- **Staleness fix** (`retrieve.rs:310` `staleness()`): a global master has
  `scope.paths` set + a seeded `reviewed`, so today it falls to branch 2
  (reviewed-time) and **decays**. The `Reference` branch must be inserted **after
  branch 1 (attested), before branch 2**, keyed `m.scope.repo.is_empty() &&
  m.anchor.kind == AnchorKind::None`. `Staleness` enum at `:267`, `label()` at
  `:276` (+ its test at `:1716`).
- **Leaf gate:** `collect_memories` (`memory.rs:1069`) has direct test callers at
  `memory.rs:2896, 2900` — those prove behaviour-preservation, so the leaf stays
  byte-unchanged; `collect_all(root)` is added *over* it. `MEMORY_ITEMS_DIR` const
  at `memory.rs:708` (add `MEMORY_SHIPPED_DIR` beside).

## PHASE-02 — DONE (commit `a02fb26`; memory `92f498a`)

Retrieval reach for the global class. Landed in `src/memory.rs` + `src/retrieve.rs`,
444 unit + e2e green, behaviour-preservation gate held (SL-005/007/008 unchanged).

- **`collect_all(root)`** (`memory.rs`, over the byte-unchanged `collect_memories`
  leaf): unions `items/` + `shipped/`, dedup by uid **items-win**, via a
  `BTreeSet<uid>` (NOT HashSet — `clippy::disallowed-types`, see new memory
  `mem.pattern.lint.disallowed-types-collections`). shipped-absent ⇒ byte-identical
  to the leaf. `MEMORY_SHIPPED_DIR` const added at `memory.rs:712`.
- **Three callers switched** to `collect_all`: `load_query` (retrieve), `run_list`,
  `list_rows` (the boot seam) — shipped now surfaces in find/retrieve/list + boot.
- **`read_body` re-keyed to `root`** (`memory.rs`): try `items/`, fall back to
  `shipped/`. Needs a non-empty filter (safe_join succeeds on a missing dir → empty
  read), so a present-but-EMPTY items body falls through to shipped — benign
  (empty == missing per the show contract). `Loaded.items_root` dropped.
- **`Staleness::Reference`** evergreen disposition (`retrieve.rs`), via predicate
  `is_global_reference`.

**SUPERSEDES the staleness ref above (lines ~33-38):** the bare key
`repo.is_empty() && anchor.kind==None` is WRONG — it catches the *scopeless*
default test fixtures (e.g. `staleness_no_anchor_no_date_is_unanchored`) and breaks
the SL-008 gate. The correct key carries the **scope floor** (ADR-002: the class is
path/glob/command-scoped; a scopeless `repo=""` memory is illegal/lint-target). The
shipped `is_global_reference` keys on **scoped ∧ `repo=""` ∧ anchor=none ∧ no
`verified_sha`**; branch placed after attested (1), before reviewed-time (3).
Canon (ADR-002 + design §5.4) outranks the plan/sheet snippet here.

## Decisions carried forward

- **read_body re-keyed to `root`** (not a second `shipped_root` arg) → drops the
  now-purposeless `Loaded.items_root` field (`retrieve.rs:598`). `dead_code` is
  denied, so the field must be removed, not left.
- **items-win dedup is silent** — design says "logged at find debug"; the repo has
  no debug-log facility (`print_stdout` denied), so the dropped duplicate is
  silent. Acceptable (uid collisions are practically impossible — disjoint
  minting).

## Environment hazard (this session)

`SL-015` is a **shared branch with an active concurrent agent** (unrelated SL-017/
SL-013 work) doing stage-all + commit. It absorbed this slice's `plan.{toml,md}`
into one of its commits (`2ce9325`) and superseded the standalone `plan(SL-018)`
commit `517d4a6` — content was preserved, nothing lost. Lesson for the executor:
**commit your own `src/**` promptly and verify it's reachable**; don't leave work
uncommitted across long gaps. Leave the concurrent agent's files
(`slice/013/*`, SL-017 `src/lexical.rs` etc.) alone.

## Minor / open

- `doctrine adr new` stamped `created = "2026-06-05"` though the session date is
  `2026-06-06` (ADR-001 carries the same). The ADR clock appears to lag a day —
  not investigated; cosmetic. Flag only if dated ordering ever matters.
