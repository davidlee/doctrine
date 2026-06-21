# Review RV-131 — design of SL-116

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The accused: `SL-116` design — lift the 3539-line `src/worktree.rs` into a
per-machine `src/worktree/` folder. Pleaded as "pure mechanical, behaviour-
preserving." The Inquisition presumes the plea false until the seams confess.

**Lines of interrogation:**

1. **Completeness of the partition.** Does the 12-file layout assign a home to
   *every* top-level production item in `worktree.rs` (≈35 symbols + consts)? An
   orphan is a latent compile break the plea conceals. Cross-examine the seam map
   against the file's actual `grep` of items.

2. **Re-export checklist exactness.** The design corrected 9→8 (F-1 struck
   `gather_tree_clean`). Press it: is any *other* externally-consumed symbol
   missing from the 8? Is any of the 8 a phantom (named but not actually consumed
   by the 8 caller files)? Untouched caller compilation is the only proof.

3. **Visibility soundness.** The widen set `{resolve_common_dir, resolve_commit,
   gather_tree_clean}` and the single-machine private claims
   (`verify_sibling_worktree`/`enumerate_candidates`→provision,
   `primary_worktree`→subagent) must match the *real* call-site distribution
   across the post-split boundaries. A helper used by two machines but parked
   private in one file is an invisible heresy.

4. **De-interleaving hazard.** land/coordinate/fork are physically interleaved
   today. Any const or helper shared across the cut and not routed to `shared.rs`
   breaks silently.

5. **ADR-001 binding tier-map.** If `.doctrine/adr/001/layering.toml` exists and
   keys on module path, the `worktree`→`worktree/` split may demand entries the
   design never names — SL-132/SL-133 were burned for exactly this omission.

6. **Behaviour-preservation gate integrity.** The plea rests on 46 tests green
   *unchanged*. Probe whether per-machine relocation can truly keep every body
   byte-identical given the shared test helpers (`git`/`init_repo`) and the
   machine-specific ones — or whether some test reaches across the new boundaries
   and forces an edit (which would void the gate).

7. **Convention fidelity & storage discipline.** Does `mod.rs`'s role and
   `test_helpers.rs` match the cited `catalog/`/`priority/` precedent? Is the
   design authored in the correct tier with canonical ids and no derived data in
   prose?

External adversarial reviewer: **codex mcp (GPT-5.5)**, per project default.

## Synthesis

**Verdict: HERETICAL AS FIRST CONFESSED — now scourged clean.** The accused
pleaded "pure mechanical, behaviour-preserving." Under cross-examination the plea
shattered: the design was *not an executable partition*. Three mortal sins and two
lesser taints, all confessed, all reconciled in `design.md` before the tribunal
rose. Five findings, five terminal — `done · await=none`, no blocker survives to
gate the slice's close.

**The ordered penance, and its discharge:**

1. **F-3 (blocker) — the binding layering map.** The gravest heresy: the design
   never amended `.doctrine/adr/001/layering.toml`, the ADR-001 tier map the gate
   enforces. The split makes `worktree` a mixed umbrella demanding per-file
   sub-classification, and `coordinate` — which reaches upward into `slice` via
   `crate::slice::run_phases` (`worktree.rs:2035`) — is `command`, not the "engine"
   the draft wished it. The identical omission burned SL-132 (RV-121) and SL-133
   (RV-130 F-1); a third repeat would have been unforgivable. **Penance:** new
   §ADR-001 obligation requiring the extractor-generated `worktree::<file>` entries
   in-slice, `MixedUmbrella` green as an exit criterion. Discharged.

2. **F-1 (blocker) — the under-counted widen set.** The draft widened 3 private
   helpers; the true cross-file set is 7. The remedy is twofold: widen the genuine
   cross-machine helpers to `pub(super)`, and — the cleaner stroke — co-locate the
   impure `read_allowlist`/`ALLOWLIST_FILE` with their sole consumers in
   `provision.rs`, which also preserves `allowlist.rs` as a pure ADR-001 leaf.
   Discharged.

3. **F-2 (blocker) — the incomplete partition.** Seven top-level items had no home,
   two of them cross-machine. **Penance:** an exhaustive 42-item map, every symbol
   to a file with its visibility. Discharged.

4. **F-4 (major) — premature proof.** Byte-identical relocation was asserted as
   settled while the partition that makes it true was unspecified. Now stated
   contingent, with the four at-risk tests spot-checked. Discharged.

5. **F-5 (minor) — a stale "9-symbol" count.** Corrected to 8. Discharged.

**Acquittal:** the external re-export surface was tried and found *sound* — exactly
the 8 named symbols; `gather_tree_clean` is no caller's symbol, only a doc-comment
ghost in `git.rs:1168`. The Inquisition confirms it.

**Standing risk (tolerated, by design):** the `worktree → slice` upward coupling
edge persists — it is the slice's declared Non-Goal. This Inquisition does not
fix it; it only compels the layering map to *confess* it (coordinate = command).
A future coupling slice may break it.

**Lesson harvested:** splitting any module under a sub-classified `layering.toml`
umbrella is never "pure mechanical" — the binding tier map is a first-class
deliverable. Recorded to memory.

> **HERESIS URITOR; DOCTRINA MANET**
