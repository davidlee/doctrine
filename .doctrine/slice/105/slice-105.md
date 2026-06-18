# after edge removal: --remove flag and --prune verb for dangling after edges

## Context

`doctrine after <SRC> <TGT>` appends soft-sequence edges — idempotent, never removes.
Once a predecessor is resolved (terminal), the `after` edge is dangling: the ordering
engine drops it and the `backlog list` footer reports it as a noisy `overrides:` line.
There is no CLI verb for removal; the only fix is hand-editing the TOML, which
violates "use the verb, not hand-edit".

Current state: 15 dangling `after` edges across ~12 open items (2026-06-18 scan).

## Scope & Objectives

1. **`doctrine after <SRC> <TGT> --remove [--rank N]`** — remove `after` edges from
   SRC matching `to == TGT`. Without `--rank`: removes all edges to TGT. With
   `--rank N`: removes only edges where `rank ≤ N`. Missing edge is an error
   (removal is deliberate, not silently ignored).

2. **`doctrine after <SRC> --prune`** — drop every dangling `after` edge from SRC:
   any edge whose TGT is absent or resolved/closed. Probes each target, removes
   dangling edges, reports each dropped edge with rank and reason. No-op prints
   "nothing to prune".

3. **`dep_seq::remove_after`** — the edit-preserving `toml_edit` removal core, pure
   (no disk/clock). Removes matching `{ to, rank }` inline tables from the `after`
   array, filtered by optional rank ceiling. F-1 refuse: absent array bails.
   IO wrapper `dep_seq::remove` follows the read→parse→core→write-once pattern.

4. **`resolve_dep_seq_src_path`** — refactor: extract source-only resolution from
   the existing `resolve_dep_seq_src` so prune can validate source without target.

5. **Tests**: unit tests in `dep_seq` (VT: removal, rank ceiling, no-match, F-1
   refuse, edit-preservation), e2e CLI goldens for `--remove` and `--prune`.

## Affected surface

| Layer | File | Change |
|-------|------|--------|
| Leaf | `src/dep_seq.rs` | Add `remove_after` core + `remove` IO wrapper; extract `resolve_dep_seq_src_path` |
| Command | `src/main.rs` | `Command::After`: add `--remove`/`--prune` flags, make `target` optional; wire `run_after_remove` / `run_after_prune`; refactor `resolve_dep_seq_src` to call new path helper |
| Backlog command | `src/main.rs` | `BacklogCommand::After`: parallel flags + optional `to` |
| Backlog engine | `src/backlog.rs` | Branch `run_after` on remove/prune modes |

## Non-Goals

- **Auto-eviction on append**: removing a newly-contradicted edge at append time
  requires ordering-engine knowledge in the write path — deferred.
- **`needs --remove`**: needs removal is a separate higher-risk operation (needs are
  hard deps, removal could create cycles) — out of scope.
- **Prune-all across the project**: `--prune` is per-source; a project-wide prune
  is deferred.
- **Modifying resolved items' own `after` edges**: the `overrides:` warnings come
  from *open* items pointing TO resolved items. The resolved item's own `after`
  edges are inert and out of scope.

## Verification

- Unit: `dep_seq` module — removal from array, no-op, F-1 refuse, prune-all
- E2E: `doctrine after <SRC> <TGT> --remove` golden output
- E2E: `doctrine after <SRC> --prune` golden output (drops all dangling)
- Manual: clean sweep of the 15 current overrides → `backlog list` footer empty

## Follow-Ups

- `doctrine needs --remove` (higher risk — cycle possibility)
- Project-wide `doctrine after --prune-all`
- Auto-eviction: when `--remove` creates a contradiction, surface it (soft cycle
  detection at write time)
