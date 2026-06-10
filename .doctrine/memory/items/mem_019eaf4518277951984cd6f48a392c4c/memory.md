# Numbered-kind identity is scattered; integrity::KINDS is the single corpus-wide id table

The trio a generic id operation needs — canonical **prefix** (`SL`), tree **dir**
(`.doctrine/slice`), and the toml-filename **stem** (`slice` → `slice-007.toml`) —
travels together **nowhere** in the kind-owning modules. Each declares its own
`Kind`/`GovKind` (slice passes the `"slice"` stem as a bare literal; adr/policy
carry `stem` on `GovKind`; spec/requirement/backlog scatter it further). The stem
is deliberately distinct from `Kind.prefix` (`meta.rs` doc): stem names the file,
prefix names the canonical id.

`src/integrity.rs::KINDS` (SL-032 PHASE-03) is the **one** place that trio is
assembled — a `const &[KindRef { prefix, dir, stem, has_runtime_state }]` for all
11 numbered kinds (SL, ADR, POL, PRD, SPEC, REQ, ISS, IMP, CHR, RSK, IDE). It backs
`validate` (per-kind id-integrity scan) and `reseat` (canonical-ref → kind dispatch
via `kind_by_prefix`). Reuses `meta::read_meta(tree_root, stem, id)` (the bare-int
`id` reader) and `entity::scan_ids` — no engine touch.

**Drift surface (R-b, accepted):** a future numbered kind **not added to KINDS**
silently escapes `validate` — there is no compile-time gate forcing registration
(unlike `write_class`'s exhaustive match, X4). The `kinds_table_*` unit test pins
the current 11 but does not force a new kind in.

**Correction (SL-032 review F-2/F-5):** "single corpus-wide table" overstates it —
KINDS' `prefix`/`dir` are a **raw parallel copy** of each module's `entity::Kind`
const, linked by nothing. The real single-source registry both `validate` and
SL-031's trunk-mint wiring derive from is **deferred to SL-031** (the second
consumer that shapes it); until then KINDS is the interim assembly point. Do NOT
pre-build the registry to "fix" the duplication — see
[[mem.thread.sl-031.kind-registry-dedup]].

**Why:** future agents adding a numbered entity kind, or building any corpus-wide
id tool (audit, renumber, cross-kind report), will look for a registry and find
none scattered — KINDS is it.

**How to apply:** adding a numbered kind ⟹ add its `KindRef` row to
`integrity::KINDS` (and update the `kinds_table_*` test). Building a corpus-wide id
operation ⟹ iterate `KINDS`, don't re-derive dirs/stems. Memory is a *named* kind
(no numeric id) and is intentionally absent (D-A). See
[[mem.pattern.entity.edit-preserving-status-transition]] (reseat's toml-id rewrite)
and [[mem.system.engine.identity-claim-seam]] (the engine's two identity shapes).
