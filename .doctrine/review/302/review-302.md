# Review RV-302 — reconciliation of SL-227

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation audit). **Facet:** reconciliation.
**Self-audit** (raiser + responder driven via `--as`), with an external
adversarial pass by codex (GPT-5.5).

**Reviewed surface.** `review/227` (impl bundle, commit `d32f9100`) — the
immutable evidence ref projected by the concluded claude-arm dispatch drive
(PHASE-01 publication engine additions; PHASE-02 library veneer + full-complement
manifest; PHASE-03 minimal-projection flip). No candidate interaction branch was
created: every finding is reconcile-surface (design prose, selector registry,
accepted-decision record, backlog evidence) or a follow-up — none required a
code `fix-now`, so the immutable evidence ref was reviewed directly. Independent
green-verification ran in an isolated `audit-227` worktree (web/map/dist sourced
from edge per `mem_019f4c64…`).

**Lines of attack.**
1. **Behaviour-preservation (VA-1, the ADR-019 hinge).** PHASE-01 claims additive
   only — every pre-existing publication VT-1..6 and install-mechanism test BODY
   unchanged (not merely green). A weakened assertion would be a blocker.
2. **The crux derived gate (D7/§9).** `every_unprojected_embed_is_a_published_backing`
   must DERIVE the base from the shipped manifest (drift-guarded), be non-vacuous
   (`checked > 0`), and genuinely prove `{embedded_filenames()} − {base} ⊆
   {published}` — the pairing invariant, executable.
3. **`show` error-class fidelity (FR-003).** All four reachable classes mapped to
   distinct stderr + non-zero, never a silent empty; traversal genuinely rejected.
4. **NF-002 structural no-write.** No reachable fs-write/mutator from any library
   verb; the byte-unchanged test real (covers failure paths), not vacuous.
5. **Full-complement completeness (D7).** `publication/manifest.toml` = every
   embedded install asset minus the base, all licence=MIT (VT-3), no hollow entry,
   nothing silently unreachable.
6. **Minimal-projection result (FR-007/008, NF-004/005).** Fresh install =
   exactly {.gitignore, doctrine.toml, project-orientation.md}, no memory entity;
   lazy first-scaffold; harness adapter survives; project-governance.md absent.
7. **Mechanical conformance algebra.** `slice conformance` undeclared/undelivered
   — assessed against the projected (review/227) registry, not the stale primary
   tree (the pre-integration lag is itself a note, not a defect).

**Gate status carried in.** `slice verify-vt 227` exit 0 (all UNATTRIBUTABLE =
ISS-226 attribution artifact, non-halting — F-4); S1 regression clean; independent
`cargo test --bin doctrine` on review/227 = 3812 pass / 1 unrelated env-fail
(`test_support::doctrine_bin_returns_existing_executable`) / 1 ignored; `cargo
clippy --workspace` clean.

## Synthesis

**Closure story.** SL-227 delivers both paired contracts faithfully. The runtime
behaviour is verified-correct: independent `cargo test --bin doctrine` on
review/227 is green (3812 pass; the lone fail is `test_support::…existing_executable`,
an env artifact unrelated to the slice), clippy is clean, and the external
adversarial pass (codex/GPT-5.5) independently confirmed the load-bearing pieces —
the full projection complement is sound (73 entries, all MIT, every backing
resolves, no duplicate backings, no base/published overlap, nothing silently
unreachable), `show` distinguishes all four reachable error classes and rejects
traversal with a non-zero exit, and the named behaviour-preservation mechanism
tests were **not** semantically weakened (F-1's additive-only claim holds — the
only non-additive change to `publication.rs` is a doc-comment). The pairing
invariant (read path before the cut) is executable via the derived crux gate.

**Ten findings, zero blockers.** The audit divides cleanly:

- **Canon lag (F-1, F-2, F-3, F-5)** — design.md prose and the selector registry
  trail the as-built reality (command tier under `src/commands/`, not top-level;
  §9 flip-list omits the 4 migrated e2e tests; the seed-gate mechanism is
  design-silent; `main.rs` is a stale design-target while `guard.rs`/`mod.rs` are
  undeclared). All reconcile-surface (per-slice direct edit + selector registry),
  no code touched. The plan.toml VT `test_file` and the two added `src/commands/*`
  selectors were **already** corrected on review/227.
- **Gate-fidelity artifact (F-4)** — verify-vt's universal UNATTRIBUTABLE is the
  known ISS-226 attribution bug (refresh-base moved the fork-point), reproduced
  fresh here; the mandated tests demonstrably exist and pass. Non-halting; a
  harvest datum for ISS-226, not a delta defect.
- **Test-fidelity + one latent edge (F-6..F-10)** — surfaced mostly by codex. The
  runtime invariants HOLD, but three tests are weaker than the invariants they
  claim: VT-6 (NF-002 no-write) is **vacuous** — it asserts on a temp dir no verb
  ever touches (F-8); the crux reachability gate reads compiled embed on both
  sides, so it can false-green on a stale incremental build and does not follow
  design §8 R3's own disk-source staleness-proof (F-7); VT-5 proves harness
  *detection*, not adapter *installation*, survives the flip (F-10). Plus a
  duplicated base-backing literal (F-6) and a latent `library tree` prefix-collision
  drop (F-9, not currently triggered — all 73 addresses are files). All batched to
  IMP-312 (test hardening) and IMP-313 (the tree bug).

**Standing risks / tradeoffs consciously accepted.**
- The NF-002 no-write guarantee currently rests on *structural* read-only-ness
  (no write path exists by construction — codex-verified), NOT on VT-6, which is
  vacuous. Until IMP-312 lands, a future command-layer change could introduce a
  write path without reddening a test. Accepted for close because the invariant
  holds today; flagged as the highest-value follow-up.
- The crux reachability guarantee is robust **at close** (fresh build) but not in
  the incremental dev loop (F-7). The current published set is correct, so no
  reachability hole ships; the robustness gap is deferred to IMP-312.
- **Honestly deferred, not regressions** (design §7/§10, reconciled in
  slice-227.md under X-F6): SPEC-009 FR-009 (hymn customization verb, D6) and
  FR-010 (governance define verb, D4); SPEC-026 REQ-375 *unsupported-source-type*
  and *metadata-without-bytes* (D3, both need >1 adapter). These stay `pending`
  by design — not audit findings.

## Post-verification amendment — F-6/F-7/F-8 pulled to fix-now (2026-07-25)

After this ledger reached `done` (F-6/F-7/F-8 verified with a `follow-up`
disposition, deferred to IMP-312), the operator **elected to fix F-6/F-7/F-8
now**, before close, reversing the IMP-312 deferral for those three only (F-9→IMP-313
and F-10→IMP-312 stay deferred). The three verified findings are terminal by
construction (ADR-007 — no verb transitions out of `verified`), so their structured
`follow-up` disposition stands as the immutable audit-time record; this prose
amendment carries the reversal.

**Fix delivered on a candidate interaction branch**, not the immutable evidence
ref: `cand-227-fix-001` / `refs/heads/candidate/227/fix-001`, merge tip
`7684b705` (3-way of base `main 3e083d4` × source `review/227 d32f9100`), fix-now
commit **`30e538be`** — `fix(SL-227): harden reachability gates + VT-6 no-write
(RV-302 F-6/F-7/F-8)`. Green + clippy-clean on a fresh full build in the candidate
worktree; the four affected tests
(`families_partition_the_visible_command_tree` census→**53**,
`every_unprojected_embed_is_a_published_backing`,
`published_set_covers_the_full_projection_complement`,
`every_verb_leaves_the_repo_byte_unchanged`) pass, full suite 3850 pass.

- **F-6 (nit)** — `publication.rs` VT-3 no longer hardcodes the `BASE_BACKINGS`
  literal; it and the install crux gate now delegate to a single shared
  disk-source assertion in `asset_source.rs`.
- **F-7 (major)** — the crux reachability gate
  `every_unprojected_embed_is_a_published_backing` now derives its base from a
  **disk source** (mirroring `shipped_manifest_admits_from_disk_source`, CHR-014),
  closing the stale-incremental-build false-green; twin gate VT-3 hardened
  identically (chosen scope **B**: harden both twins + dissolve F-6).
- **F-8 (major)** — VT-6 `every_verb_leaves_the_repo_byte_unchanged` rewritten
  **non-vacuous**: populated repo root, byte-tree snapshot before/after, all four
  failure classes asserted incl. malformed-policy load `Err`.

**Supersedes** the two "Standing risks" bullets below (NF-002 no-write rests on a
vacuous VT-6; crux reachability robust only at close): both are now closed in
candidate. The candidate is admitted `--review RV-302` and is the `close_target`.

## Reconciliation Brief

Surface I reviewed: **`review/227` @ `d32f9100`** (immutable impl bundle). Findings
were reconcile-surface or captured follow-ups; **post-audit, F-6/F-7/F-8 were pulled
to fix-now on candidate `cand-227-fix-001` `30e538be`** (see amendment above) — the
remaining findings need no code `fix-now`.

### Per-slice (direct edit — design.md / slice-227.toml)

- **design.md §5.1 system model + §5.2 (F-1):** the command tier is
  `src/commands/{cli,library,guard,mod}.rs`, not top-level `main.rs` / `src/library.rs`
  (ADR-001 forbids an unclassified top-level command module). Update the §5.1
  diagram and the §5.2 module names/`main.rs` wiring prose to match. The selector
  registry and plan.toml VT `test_file` are already correct on review/227 — this
  is the prose mirror only.
- **design.md §9 "Install (changed)" flip list (F-2):** add the 4 migrated
  `tests/e2e_{policy,knowledge,standard,revision}_install_commit.rs` (eager-install
  → lazy-first-scaffold, each strengthened with a "bare install must NOT eagerly
  scaffold" negative assertion).
- **design.md §5.3 / D8 (F-3):** record the chosen seed-gate mechanism as an
  accepted decision — `install/manifest.toml [memory].seed_items = []` +
  `seed_authoring_memories`'s existing `is_empty()` early-return — so the empty
  `[memory]` block reads as deliberate, not an omission.

### Selector registry (the `doctrine slice selector` verb — load-bearing for conformance; §5.2/§6 are the mirror)

- **F-5 / F-2:** `doctrine slice selector rm src/main.rs` (undelivered stale
  design-target); `doctrine slice selector add src/commands/guard.rs` and
  `src/commands/mod.rs` (undeclared mechanical library-module glue). Optionally add
  the 4 e2e test files as `scope-relevant` so they stop reading as undeclared.
  These edit `slice-227.toml`, which `slice conformance` reads — a prose-only fix
  leaves conformance red.

### Backlog (already captured during this audit — harvest, no reconcile write)

- **F-4 → ISS-226:** annotated with the fresh universal-UNATTRIBUTABLE reproduction.
- **F-10 → IMP-312** (test-fidelity hardening — VT-5 adapter-install); **F-9 →
  IMP-313** (latent `library tree` prefix-collision). **F-6/F-7/F-8 fixed-in-candidate**
  `cand-227-fix-001` `30e538be` (post-audit fix-now — see amendment), trimmed from IMP-312.

### Off-surface (no brief item — recorded so reconcile does not chase them)

- No `plan.toml` `EN-/EX-/VT-` edits (immutable-append; the test_file correction
  already rode the dispatch drive, not a reconcile edit).
- The deferred SPEC-009/SPEC-026 items are already reconciled in slice-227.md
  scope; no further governance/spec REV is warranted by this audit.
