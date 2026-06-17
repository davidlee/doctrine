# Notes SL-090: Wire link/unlink CLI and template for memory relations

## 2026-06-17 — Inquisition complete (RV-059)

RV-059 tried the design on 8 charges. All resolved. Key findings:

- **F-1 (blocker): D4 contradicts scope.** Scope says "items/ first, shipped/
  fallback"; D4 tightened to "items/ only" with no delta annotation. Must be
  reconciled before /plan.
- **F-2 (design-wrong): Shipped/ clone dead end.** "clone to items/ first"
  prescribes an unavailable verb. Fix depends on F-1 resolution.
- **F-4 (major): D6 fork.** Design must commit to duplicating F1 guard in
  memory.rs — RelationLabel is vocabulary-bound; memory labels are raw.
- **F-3/F-5/F-6/F-7/F-8 (minor/nit):** Import direction, test disambiguation,
  path joiner, help text, stale handover — all straightforward fixes.

The design is salvageable with small corrections. See RV-059 synthesis for the
ordered penance.

Durable pattern recorded: `mem.pattern.link.memory-label-fork` — memory write
path labels are free-form; RELATION_RULES vocabulary is closed to memory edges.

## 2026-06-17 — Plan revised after critical review

Five corrections applied to scope, design, and plan:

1. **Scope amended**: "items/ first, shipped/ fallback" → "resolve the writable
   `items/<uid>/memory.toml` path (shipped/ is read-only)". Eliminates the
   scope-plan contradiction.
2. **D4 error message fixed**: removed "clone to items/ first" (unavailable verb)
   → honest message naming `doctrine memory sync` and the items/ remedy.
3. **UidPrefix in shipped/ dropped**: PHASE-01 no longer scans shipped/ for
   prefix disambiguation — shipped/ is read-only, error immediately.
4. **PHASE-03 target classification specified**: two-step try/catch —
   `parse_canonical_ref` succeeds → `ensure_ref_resolves`; fails → free text
   passthrough. Behavior-preservation invariant documented.
5. **Key symlink mechanism documented**: items/ key→uid symlinks are what make
   `MemoryRef::Key` resolution work; noted in plan.md and PHASE-01 assumptions.

## 2026-06-17 — Plan authored

Three sequential phases, bottom-up: resolution → manipulation → CLI fork.

- **PHASE-01** (`resolve_memory_toml_path`): items-first/shipped-fallback
  existence check, shipped-only refusal, uid-prefix disambiguation.
- **PHASE-02** (`append_memory_relation`/`remove_memory_relation`): raw
  toml_edit with free-form labels, duplicated F1 guard, empty-label/target
  refusal. Reuses `AppendOutcome`/`RemoveOutcome` from relation.rs (leaf→leaf).
- **PHASE-03** (link/unlink fork): memory branch before `parse_canonical_ref`,
  best-effort target validation (canonical refs must resolve; free-text/memory
  uid pass through), CLI help text update.

Each phase is independently testable. The fork in PHASE-03 is the thinnest
possible integration — a single `if let Ok(mref)` block per function.

Status transitioned: design → plan. Next: `/phase-plan` → `/execute`.
