# Notes SL-240: Forward-sync live checkouts on ref advance

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Abandoned (2026-08-15)

Abandoned with the dispatch funnel, which the defect class was exclusive to.

- The mechanism — `commit_on_behalf` → `git::update_ref_cas`, working-tree-free
  CAS landing under a live checkout — lives only in `src/dispatch.rs` and
  `src/mcp_server/dispatch.rs`. Solo (`/execute`) and `/worktree` fork paths never
  call it: fork = `git branch` + `git worktree add`; land = `git merge --no-ff`.
- The one non-dispatch `update_ref_cas` caller, `replay_ref` (projection journal),
  advances never-checked-out refs — already out of scope here.
- The underlying invariant — advancing a ref with a live checkout leaves a stale
  index / phantom staged reversal — survives the capsule model: RFC-025 migration
  step 4 keeps one CAS ref advance in the control plane. Carried forward as QUE-220.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
