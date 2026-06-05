# Audit SL-013 — memory install flag + off-script skill-port record

Conformance audit of commit `dee25ea` (PHASE-01) against the re-locked
`design.md` (`cb2ba05`, all 5 inquisition charges dispositioned, Part VI) and the
§9 VT roster (authoritative). Code surface: `src/skills.rs` (pure resolver +
thin shell) and `src/main.rs` (clap arg + thread).

**Verdict: acceptable — ready for `/close`.** All six VTs discharged. The
HIGH-severity `select([]) == all` guard (D3) is proven, not asserted: the bail
lives in the pure `resolve_install_ids` and was empirically driven red. The
marketplace-symlink mechanism (VT-05) is verified by reproducing the install
path (clone-whole-repo + deref-copy). Two cosmetic nits, no defects.

## Evidence

- **Gate PASS.** `just check` exit 0; 436 `--bin doctrine` unit tests +
  3 bm25_probe + 3 e2e_memory_anchoring + 1 e2e_skills_symlink green. (First run
  needed `touch tests/*.rs` for the stale-`CARGO_BIN_EXE` env trap — F2, env not
  code; `mem.pattern.testing.stale-cargo-bin-exe`.)
- **VT→test trace** (all automated, green):
  - **VT-1** ← `subset_ids_extracts_only_the_named_domain`,
    `subset_ids_absent_domain_is_empty`, **`resolve_install_ids_bails_on_empty_derivation`** (the D3 bail, pure, synthetic paths).
  - **VT-2** ← `resolve_install_ids_live_embed_yields_the_memory_pair` (live embed
    → `{record-memory, retrieve-memory}`; pins embed-follows-symlinks).
  - **VT-3** ← `only_memory_selects_exactly_the_two_canonical_skills`
    (derive → `validate_filters` → `select` = exactly the two `doctrine`-domain
    entries; drives the cross-domain identity invariant §5.5).
  - **VT-4** ← `only_memory_conflicts_with_skill`, `only_memory_conflicts_with_domain`
    (parse-time clap conflict) + `only_memory_alone_parses` (positive control).

## Findings

### A-1 🟢 Charge I penance landed — the safety guard is red-able, not vacuous
The inquisition's MAJOR charge was that the D3 empty-set bail was stranded in the
impure shell, asserted-but-unprovable. **Verified fixed.** The bail is in the
pure `resolve_install_ids` (`src/skills.rs:224`), fed synthetic paths by
`resolve_install_ids_bails_on_empty_derivation`. Confirmed non-vacuous by
controlled mutation: deleting the `if ids.is_empty() { bail!(…) }` block made the
test **FAIL** (`assertion failed: …is_err()`, `skills.rs:927`); reverted via
`git checkout`. The guard against `--only-memory` silently installing the entire
catalog is genuinely proven.
- **Disposition:** aligned. No action.

### A-2 🟢 VT-05 marketplace install-smoke — mechanism verified
Design §6/D4 keeps the `doctrine-memory` subset plugin on the bet that Claude
Code clones the whole marketplace repo locally, so the relative symlinks
(`../../doctrine/skills/<id>`) resolve at install. Confidence was "soft" → a
manual smoke. Reproduced the load-bearing install path in-jail:
1. **Repo-side preconditions.** Both subset entries are git mode `120000` real
   symlinks (travel with a clone), relative-targeted at the **sibling**
   `plugins/doctrine/` subtree; both resolve to a real `SKILL.md`.
   `marketplace.json` lists `doctrine` and `doctrine-memory` from the **same**
   `./plugins/...` repo — one clone carries both subtrees.
2. **Clone-whole-repo (what CC does).** `git clone file://…/.git` into a temp
   dir; in the fresh clone both `doctrine-memory/skills/<id>/SKILL.md` resolve
   through the symlinks (sibling `doctrine/` present) — `name: record-memory`,
   `name: retrieve-memory` read back.
3. **Deref into the plugin cache.** `cp -rL plugins/doctrine-memory …` yields
   **real** `SKILL.md` files — the consumer needs no symlink-following to see the
   subset.
4. **CLI resolver agrees.** VT-2/VT-3 (live embed) re-run green.
- **Limitation (recorded, not a gap):** the live Claude Code
  `/plugin marketplace install` step was **not** executed — it would mutate the
  real `~/.claude`. It is substituted by the faithful mechanical reproduction
  above; the runtime behaviour it relies on (clone + deref) is what was proven.
- **Disposition:** aligned. The mechanism stands. If a future CC version
  diverged, the subset is **additive** — users fall back to
  `--only-memory` (this slice's flag) or `--skill record-memory --skill
  retrieve-memory`; no capability is lost (design §8 row 3).

### A-3 🟢 VT-06 deliverable-2 record — reads true
Deliverable 2 (record the off-script skill port, scope item 1) is discharged by
`slice-013.md` Context + Scope §1 (the 18-skill port, plugin manifests, the
`doctrine-memory` subset + `MARKETPLACE_ONLY_DOMAINS` guard, and the durable
session decisions). Read it; it accounts for what landed on `main` this session.
No further authoring owed (inquisition Charge II disposition).
- **Disposition:** aligned. `/close` makes the final attestation.

### A-4 🔵 Orphan doc line on `InstallArgs`
`src/skills.rs:743` is a stranded `/// `doctrine skills install`.` — the old
`run_install` doc left behind when `InstallArgs` was inserted above the fn. It
now doubles the struct's real doc line (`:744`) and `run_install` (`:758`) has no
doc of its own. Cosmetic only, no behaviour.
- **Disposition:** fix (trivial). `/close` lands a commit anyway — drop the
  orphan line there. Not a defect; no route-back warranted.

### A-5 🔵 `subset_ids` mirrors `discover`'s path-split rather than sharing it
`subset_ids` (`:200`) and `discover` (`:128`) both pattern-match
`[domain, "skills", id, ..]` over a freshly-collected `Vec`. The duplication is
small and deliberate (design §5.5: derivation keeps the discover-**excluded**
domain, so it cannot ride `discover`). Acceptable.
- **Disposition:** tolerated drift. Extracting a shared splitter is not worth the
  coupling; the two readers want different domain-filtering.

## Durable harvest

- `mem.pattern.lint.cli-handler-args-struct` — recorded in PHASE-01 (F1): adding
  a flag pushed `run_install` past clippy's arg/bool ceilings; fix was the
  `InstallArgs` struct mirroring `memory::RecordArgs`. Already in the store.
- No new memory owed. F2 (stale `CARGO_BIN_EXE`) is already
  `mem.pattern.testing.stale-cargo-bin-exe`.

## Reconciliation note for `/close`

Rollup is `1/1 ⚠` — the ⚠ is the hand-status `proposed` diverging from
phases-complete. `/close` reconciles slice status `proposed` → terminal, attests
VT-06, and may fold A-4's one-line doc cleanup into the final commit.
