# integrity::KINDS is the corpus-wide id table — a referencing view over the engine Kind consts

`src/integrity.rs::KINDS` is the **one** place every numbered kind's identity is
assembled for generic id operations — backing `validate` (per-kind id-integrity
scan) and `reseat` (canonical-ref → kind dispatch via `kind_by_prefix`). It covers
the 12 numbered kinds (SL, ADR, POL, STD, PRD, SPEC, REQ, ISS, IMP, CHR, RSK, IDE).
Reuses `meta::read_meta(tree_root, stem, id)` and `entity::scan_ids` — no engine touch.

**Post-SL-031 shape (F-2/F-5 dedup landed).** `KindRef` is now a *referencing view*:
`{ kind: &'static entity::Kind, stem: &str, state_dir: Option<&str> }`. The canonical
`prefix` and tree `dir` are **no longer copied** — they derive from the owning
module's `entity::Kind` const (`kind: &SLICE_KIND`, `&ADR_KIND.kind`, …), so a
dir/prefix change in the const flows through with no second copy to drift (closes the
old SL-032 F-2 raw-parallel-copy hazard). The two facts the engine leaf does not
carry stay on `KindRef`: `stem` (names the toml file, `slice` → `slice-007.toml`;
deliberately distinct from `prefix`) and `state_dir` — `Some(".doctrine/state/slice")`
for slice, `None` for the rest — the gitignored runtime phase-state tree `reseat`
refuses to strand (F3). `state_dir` replaced the old `has_runtime_state` bool +
hardcoded path (F-5).

**Drift surface (R-b) — NOT closed, and provably can't be cheaply.** Membership in
KINDS is still hand-maintained: a future numbered kind whose `Kind` const exists but
whose `KindRef` row is missing **silently escapes `validate`**. There is no
compile-time gate forcing registration (unlike `write_class`'s exhaustive match). The
`kinds_table_*` unit test is a **literal prefix pin** (asserts the current 12), not a
set-equality guard — add a 13th kind and forget KINDS and the test stays green.
SL-031's scope aspired to a "set-equality guard (KINDS ⟺ the Kind consts)" but
reframed it to the literal pin via **C-IV**: Rust has no const reflection, so the set
of all `Kind` consts can't be enumerated to diff against KINDS. Closing this for real
needs a macro/inventory pattern, not a test. (Re-confirmed by the SL-031 audit, F-7.)

**Why:** future agents adding a numbered entity kind, or building any corpus-wide id
tool (audit, renumber, cross-kind report), will look for a registry — KINDS is it,
and they must know its membership guard is advisory, not enforced.

**How to apply:** adding a numbered kind ⟹ add its `KindRef` row to `integrity::KINDS`
(point `kind` at the module's `Kind` const; set `state_dir` only if it owns runtime
state) **and** update the `kinds_table_*` pin — nothing else will catch the omission.
Building a corpus-wide id operation ⟹ iterate `KINDS`, read `k.kind.prefix`/`k.kind.dir`,
don't re-derive. Memory is a *named* kind (no numeric id) and is intentionally absent
(D-A). See [[mem.pattern.entity.edit-preserving-status-transition]] (reseat's toml-id
rewrite) and [[mem.system.engine.identity-claim-seam]] (the engine's two identity shapes).
