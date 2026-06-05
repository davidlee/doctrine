# SL-015 Audit — Spec entity v1 (product + technical specs)

**Mode:** conformance (post-implementation, slice-tied).
**Evidence date / baseline:** SL-015 work committed at `65c6792`
(`doc(SL-015): canon sweep + close-out (PHASE-06)`); gate re-run green at repo
HEAD. Reconciled against `design.md` (canonical), `slice-015.md` (scope),
`plan.toml` (EX/VT), and the swept `doc/*` canon.

> Hand-authored — no `slice audit` scaffold yet (CLAUDE.md known gap).

## Scope of audit

All six phases, `completed` in the state tree; `slice list` rollup **6/6**
(`⚠` = hand-edited `slice-015.toml status = proposed` diverges from the rollup —
the lifecycle-transition gap `/close` reconciles). Mechanism shipped end-to-end:
`requirement` peer entity + `spec new` / `list` / `req add` / `show` / `validate`.

## Evidence gathered

1. **Gate green at HEAD.** `just check` → **404 lib/bin + 3 + 1 e2e passing, 0
   failed**; `cargo clippy` (bins/lib) zero warnings; fmt clean. No `src` change in
   PHASE-06 (doc/skills/memory only).
2. **Behaviour-preservation gate.** `entity.rs` + slice/adr/memory suites green
   **unchanged**; `meta.rs` additive only (the `#members` cell rides the generic
   `render_table`, the shared 4-column `format_list` untouched — notes F-3).
3. **End-to-end CLI smoke** (throwaway temp project, built binary) — all §5.2
   contracts observed:
   - `spec new product|tech` → `PRD-001` / `SPEC-001`, per-subtype trees.
   - `spec req add` auto-labels by kind: functional → `FR-001`/`FR-002`, quality →
     `NF-001`; reserves `REQ-001..003`; prints `Added FR-001 (REQ-001) to PRD-001`.
   - `spec list` → per-subtype rows `id status slug #members` (product `#members=3`).
   - `spec show PRD-001` → identity line + `spec-NNN.md` prose **verbatim** +
     Requirements section (`### FR-001 (REQ-001) — <title>`, then
     `slug · kind · status`), members in `order`. Absent `description` ⇒ no
     statement line (notes D-P4-1).
   - `spec validate` clean → `validate: corpus clean`, exit 0.
   - **Failure path:** injected dangling member FK (`REQ-999`) → hard, exit 1;
     hand-made orphan requirement → hard on corpus, exit 1; **scoped**
     `spec validate SPEC-001` suppresses the orphan check, still flags the dangling
     FK (exit 1) — scope semantics correct (§5.4 / external-review item 3).
   - Bare numeric `spec show 001` → `is not a canonical spec ref` (exit 1) — C4.
4. **PHASE-06 canon sweep.** Grep gate
   `grep -rnE 'rows, not artefacts|SPEC-[0-9]+\.(FR|NF)' doc/` returns **only**
   `doc/spec-entity-spec.md:12` and `:121` — both deliberate superseding callouts
   ("old draft said `SPEC-110.FR-001`; shipped uses `REQ-NNN`"). No live
   compound-key / rows-not-artefacts creed. Skills (`spec-product`/`spec-tech`)
   dropped "not yet structural" in **both** the tracked source
   (`plugins/doctrine/skills/`) and the installed copy (`.doctrine/skills/`).

## Findings & dispositions

| # | Expected (cite) | Observed | Disposition |
|---|---|---|---|
| A-1 | §5.2 CLI contracts (new/req add/show/validate/list, canonical-ref, scope) | All observed in smoke incl. failure + scoped paths | **aligned** |
| A-2 | §9 named test titles (`validate_flags_*`) | Behaviour covered in `registry.rs` under different names (`dangling_member_fk_is_flagged`, `clean_corpus_yields_no_findings`, …) **plus** an extra `validate_runs_orphan_only_on_a_corpus_pass` | **aligned** — §9 titles are illustrative, not normative; coverage is complete + superset |
| A-3 | §9 behaviour gate — entity/slice/adr/memory suites green unchanged | 404 passing; meta.rs additive (F-3) | **aligned** |
| A-4 | Scope closure intent — req add edit-preserving row+prose; round-trips comments/unknown keys | `spec_req_add_is_edit_preserving` green; `toml_edit` append (notes F-5) | **aligned** |
| A-5 | §5.6/C1/C3 — full four-file canon sweep, grep gate clean | spec-entity-spec rewritten; entity-model/relation-index/glossary reconciled; gate returns only intended residue | **aligned** |
| A-6 | §5.3 declares `sources: Vec<Source>` but never defines `Source` (notes D-3) | Resolved `Source { language, identifier, module? }` now in the durable canon `doc/spec-entity-spec.md` (Metadata + Serde) + notes D-3; **`design.md` §5.3 itself still terse** | **tolerated drift** — the binding definition lives in evergreen canon + notes; `design.md` is the locked point-in-time slice design, not re-opened for a field already reconciled downstream. No live defect |
| A-7 | §5.5 / known risks — torn two-tree write leaves orphan requirement | `spec req add` non-transactional by design; orphan left uncommitted; `spec validate` flags hard (smoke-confirmed) | **aligned** — consciously accepted; detection backstop present |
| A-8 | §10 residuals — label/order TOCTOU; auto-label cross-merge collision | Detection-only; `spec validate` duplicate-label is the hard backstop (`duplicate_label_within_a_spec_is_flagged`) | **aligned** — accepted residual with backstop |
| A-9 | §5.2/§7 D-Q4 — `spec req link` (reuse existing requirement under a 2nd spec) | Not built; storage admits it (a 2nd `members.toml` row), named-deferred | **follow-up slice** — designed-deferred, explicitly named |
| A-10 | Scope Follow-Ups — `feature` DAG + cycle validation, relation-index cache, coverage queries, revision subtype, importer, sync, `*.rendered.md` (`--write`), spec lifecycle/approval | None built; all design-acknowledged deferrals | **follow-up slice** — owned future work, named in scope §Follow-Ups |
| A-11 | Boot snapshot Memory index should list `mem.system.spec.composition-seam` | New memory recorded but not yet in `boot.md` governance index (≤2-session lag) | **fix at close** — `doctrine boot` regenerate so the new seam surfaces (the `/canon` freshen ritual) |
| A-12 | Clean tree for memory attestation | `doctrine memory verify` refused (working tree carried unrelated SL-017 edits mid-audit; an external `218a7cf` then landed) | **aligned** — memory is recorded (VT-3 met); attestation is optional polish, run post-close on a clean tree |

No finding routes to "fix now" inside the mechanism — the slice is behaviourally
complete and canon-consistent. A-6 is the only drift, consciously tolerated with
rationale; A-11 is a one-command housekeeping step folded into close.

## Harvested durables

The durable decisions/findings are already in `notes.md` (D-1..D-P6-4, F-1..F-P6)
and the new memory `mem.system.spec.composition-seam` (the spec↔requirement
identity + edge seam, distinct from `mem.system.engine.identity-claim-seam`). No
additional harvest required.

## Closure recommendation

**Audit-ready.** Mechanism conforms to `design.md` §5; canon (`doc/*`) reconciled
to shipped reality with a clean grep gate; behaviour gate green; every deferral is
design-acknowledged and named. `/close` actions: (1) `doctrine boot` regenerate
(A-11); (2) reconcile `slice-015.toml status` proposed → done (clears the `⚠`);
(3) final close commit. Then optionally `doctrine memory verify` on a clean tree.
