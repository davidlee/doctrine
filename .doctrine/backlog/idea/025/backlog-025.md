# IDE-025: Selector-sourced write-allowlist jail mode: confine worker writes to design-target touch-set (anti-drift)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A stricter **jail mode** for the SL-182 confinement hooks, deferred as a future
config surface. SL-182's floor confines a worker to writing *anywhere inside its
worktree*. This mode tightens that to writing *only the paths the slice declared*
— sourced from the slice's **`design-target` selectors** (RFC-004 path-intent
primitive; the same touch-set the audit-time `slice conformance` delta diffs
against git actuals).

## The mechanism

The `PreToolUse(Edit|Write)` pathcheck already computes `realpath(file_path) ⊆
cwd`. In allowlist mode it adds a second predicate: `realpath` must also match one
of the worker's design-target selectors. The Bash-wall bwrap jail can't do this
per-path (a mount-ns rw-binds whole subtrees, not glob sets), so the allowlist is
an **Edit/Write-wall property**, not a bwrap property — and only as strong as the
Bash wall's coverage allows (a worker could still `Bash`-write inside the worktree
off-allowlist; the mode is anti-*accident*/anti-*drift*, layered, not a hard wall
for the Bash tool). Worth stating that boundary honestly when built.

## Why

Anti-**drift**: keeps an honest worker editing only its declared surface, catching
scope creep at write time instead of at audit. Ties confinement to **conformance**
— the selectors that already gate the audit become live write guards. Natural
insertion point is the OQ-3 per-worker policy schema: a `write_allowlist` /
`mode = "selector-strict"` field, with the selector set resolved from the slice +
phase at spawn and written into the worker's policy file.

## Costs / unknowns

- Selector resolution at spawn — the orchestrator must resolve the phase's
  design-target selectors and stamp them into the policy file (binding key:
  `agent_id`).
- Bash-wall gap — see above; the mode is honest only if framed as anti-drift, not
  a hard write boundary (the bwrap jail rw-binds the whole worktree).
- Glob matching in the hook (selector globs vs realpath).

## Correction — the floor CAN host an allowlist (added during SL-185 scoping)

The "mount-ns rw-binds whole subtrees, not glob sets" claim above is too strong
for the **Seatbelt** floor (SL-183). `seatbelt_profile` already emits a regex path
filter (`(allow file-write* (require-all (subpath …) (regex XCRUN_DB_REGEX)))`,
`jail.rs:506`). A selector allowlist is the same shape per-selector —
`(allow file-write* (require-all (subpath WT) (regex <glob→regex>)))` — and, being
a `file-write*` syscall floor, it governs **new-file creation under a glob** and
**every write incl. Bash**. That makes the Seatbelt form *stronger* than the
claude Edit/Write-tool predicate described here (which a `Bash` write slips). So
this mode is NOT claude-arm-only; the profile builder is a second insertion point
alongside the policy schema. bwrap can only approximate (per-file rw-bind of the
point-in-time set; new-file/atomic-rename break). See [[SL-185]] OQ-3.

## Decision basis

Out of scope for SL-182 (which lands the floor: confine-to-worktree). Pull forward
if observed agent drift (workers editing undeclared files) justifies it.

Refs: SL-182 (the confinement floor this extends), RFC-004 (path-intent selector
primitive), `slice conformance` (the audit-time consumer of the same selectors).
