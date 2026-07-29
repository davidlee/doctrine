# Notes SL-144: ADR-005 full compliance: reference-doc IA, user hooks, restate-line audit

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open

## Design staleness — projection policy changed under §5.3 (2026-07-29)

Surfaced incidentally while landing IMP-263 (a `using-doctrine.md` edit). **The
design's shipping premise is now false**, and the falsehood is load-bearing for
§5.3, §5.2's reset column, and two §9 VTs. Recorded here rather than edited into
the reviewed design; `/design` owns the correction.

**What changed.** SL-227 / ADR-019 (`minimal-projection`) replaced eager
projection with **publish-on-demand**. `build_plan` step 2 now projects only
`install/manifest.toml` `[base].backings` = `{.gitignore, doctrine.toml,
project-orientation.md}`; every other embedded asset is *published*, reached via
`doctrine library show reference/<file>`. See `src/install.rs` step 2 comment
(FR-007/FR-008, D8) and `src/install.rs:4003` ("Governance is no longer
projected").

**What it invalidates.**

1. **OQ-2's resolution and §5.3 point 1** — "'shipped' ≡ 'exists under
   `install/`', copied write-if-absent, no allowlist" is exactly the policy
   ADR-019 retired. The `[base].backings` list *is* the per-file allowlist OQ-2
   concluded did not exist.
2. **§5.3 semantic currency + IDE-030** — dissolved, not deferred. Nothing is
   copied, so no client can hold a stale copy; `library show` streams the
   installed binary's embed. Currency is automatic, the same argument §5.1
   already makes for clap-derived boot sections. Re-check IDE-030 for mootness.
3. **The pointer form (§5.3 point 2)** — a "machine-checkable pointer class"
   defined as *an explicit filename reference* now points at nothing resolvable
   in a client: bare `using-doctrine.md` names no file there. The canonical
   pointer is the **invocation** `doctrine library show reference/<file>`. This
   is §5.1's own derived-vs-authored rule applied to docs — cite the serving
   verb, never a path. Every Tier-1 pointer the slice plans to *add* inherits
   this, as do the ~12 skills that currently cite the bare filename and boot's
   reference-docs line.
4. **Shipping oracle identity (§5.1)** — publication is governed by
   `publication/manifest.toml` (logical address `reference/using-doctrine.md` ←
   backing `using-doctrine.md`), a *different* file from the
   `install/manifest.toml` the design names. Two manifests, two roles.
5. **§9 fresh-install VT is now backwards** — it asserts `glossary.md`,
   `using-doctrine.md`, `model-band.md` are **present in `.doctrine/`**. Under
   minimal projection their presence is the defect, not the pass condition.
   Correct assertion: absent from projection, resolvable via `library show`.
6. **§2 / D4 orphan count** — the design knows one orphan (`boot-footer.md`).
   This repo carries **8** tracked orphans with no `src/` read path:
   `boot-footer.md`, `dispatch-mechanics.md`, `glossary.md`, `harvest.md`,
   `model-band.md`, `review-ledger.md`, `routing-process.md`,
   `using-doctrine.md`. D4's delete-the-residue logic generalises to all 8.
   **Live, keep:** `.doctrine/governance.md` (boot reads `GOVERNANCE_REL`) and
   `.doctrine/project-orientation.md` (a backing).
7. **§5.2 hook table** — the `model-band.md` row's "`install/model-band.md` →
   `.doctrine/…`" mechanism and "Reset: restore from `install/`" describe copy
   semantics that no longer run. `governance.md`'s row survives on the boot read
   path, but its "re-install re-seeds" reset does not — it is no longer a backing.
8. **Second-order symptom worth citing as evidence** — IMP-252 added a doctor
   prose-cite **exclusion** for `.doctrine/glossary.md`: noise suppression for an
   orphan that should have been deleted. The residue is already costing
   maintenance elsewhere.

**Unchanged.** The re-embed footgun (R2), the restate-line objectives (3, 5), and
the tier model itself (§5.1 Tier 0/1/2) survive intact — only the *mechanism* by
which a Tier-1 doc reaches a reader changed, from a copied file to a verb.

Friction record: `019fad6c-b50a-7d41-9987-bf5498c04eee` (supersedes a
wrong-cause record, `019fad63-…`).
