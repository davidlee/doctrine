# Notes SL-032: Worker-mode CLI guard and trunk-ref id allocation with reseat

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-02 — trunk-ref id allocation (done)

Mechanised ADR-006 D3. The trunk read seam + pure union allocation; no production
trunk wiring (that is SL-031). All `materialise` call sites pass `&[]` (INV-1
trivially holds). Existing numeric allocation suites green unchanged (EX-4/VT-6).

**Shape shipped**
- `entity::next_id(local, trunk)` — pure `max(local ∪ trunk)+1`; unions then
  delegates to `candidate_id` (one max-impl, D-a). `next_id(local, &[]) ==
  candidate_id(local)` — proven by a test, the INV-1 anchor.
- `materialise(.., trunk_ids: &[u32])` + `allocate_fresh(.., trunk_ids, scan)` —
  trunk read once outside the retry loop (D-b); the scan closure stays per-retry.
  Inert for `InExisting`.
- `git::trunk_entity_ids(root, kind_dir)` — `ls-tree -d --name-only <tree> --
  <kind_dir>/`, trailing numeric basename parsed, non-numeric dropped. `kind_dir`
  is already prefixed (X1) — no re-prepend. None/absent ⇒ `Ok(vec![])`.
- `git::trunk_tree_ish(root)` — peeled ladder `DOCTRINE_TRUNK_REF` → `origin/HEAD`
  → `main` → `master`. Ladder exhausted ⇒ `Ok(None)`.

**Deviations from the phase sheet (for the close-out audit)**
- **D-1: T1 split into `trunk_tree_ish` (shell) + `trunk_ladder(root, explicit:
  Option<&OsStr>)` (core).** The crate `forbid`s `unsafe_code`, and edition-2024
  `std::env::set_var` is `unsafe` — so the explicit-override asymmetry (F4/VT-5)
  is untestable if the env read sits inline. Split the env read (the only
  impurity) into the thin public shell `trunk_tree_ish` (locked §5.2 signature,
  unchanged) and an env-injected `trunk_ladder` core, exactly the project's
  pure/imperative split (CLAUDE.md: "pass env in as inputs"). VT-5/5b now drive
  `trunk_ladder` with the ref injected — no process-env mutation, no serialising
  lock. Public seam SL-031 calls is identical.
- **D-2: T4 call-site list was incomplete — `spec.rs:651` and `spec.rs:1112`
  also call `materialise`** (PRD/spec subtype minting). Both updated to `&[]`.
  The diagnostics surfaced them; the sheet enumerated only governance/requirement/
  backlog/slice. Worth a grep, not a hand-list, next time.
- **D-3: T5 (item-level `#[cfg_attr(not(test), expect(dead_code))]`) was
  unnecessary.** `git.rs` already carries a module-level
  `#![cfg_attr(not(test), expect(dead_code, …))]` (from SL-007) that covers every
  dead-in-non-test item, the new trunk helpers included. The VT fixtures reference
  them under `cfg(test)`, so nothing is unfulfilled. No per-item attr added.

Gate: `cargo test` all suites green, `cargo clippy` zero warnings, `cargo fmt`
clean.
