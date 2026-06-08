# SL-018 — Audit (PHASE-04 + PHASE-05)

Conformance audit. Mode: post-implementation, tied to the slice. Reconciled
against `design.md`, `plan.toml`, ADR-002, and `doc/memory-spec.md`. Evidence
gathered at HEAD `4e3cb65` over a green `just check` (clippy zero-warning bins+lib,
fmt clean, 474 bin tests + e2e all pass).

Scope: PHASE-04 (master authoring path + master-lint) and PHASE-05 (author the
corpus). PHASE-01/02/03 were audited implicitly via the behaviour-preservation
gate and the end-to-end reach (they are the surface these phases ride).

## Gate state at audit time

- `just check` — GREEN. `cargo fmt` clean, `cargo clippy` zero warnings (bins+lib),
  `cargo test --bin doctrine` = 474 passed / 0 failed, integration suites
  (`e2e_memory_sync` 5, `e2e_memory_anchoring` 4, `e2e_skills_symlink` 1,
  `bm25_probe` 3, retrieve e2e 4) all green.
- **Behaviour-preservation gate (SL-005/007/008) — HELD.** `git diff main --
  src/memory.rs src/retrieve.rs` removes ZERO existing test/assertion lines; the
  `collect_memories` leaf signature is unchanged. The only edits to existing test
  bodies are the mechanically-required `global: false` field initializers (forced
  by the additive `RecordArgs.global` field) — no assertion altered or deleted.
  The SL-005/007/008 suites pass unchanged.

## PHASE-04 disposition table

| id | criterion | status | evidence | disposition |
|---|---|---|---|---|
| EX-1 | `record --global` mints `repo=""`/anchor=none master into repo-root `memory/`, reusing uid-mint+schema validation, bypassing the repo-anchor gate by explicit intent; normal-path gate unchanged | met | `src/memory.rs` `run_record`: `--global` branches the frame to `git::unanchored_frame()` (suppresses capture) and the target dir to `MEMORY_MASTERS_DIR` ("memory") vs `MEMORY_ITEMS_DIR`; the constraint-4 bail is unchanged (only its precondition `frame.repo` is now empty for global). Test `record_global_mints_an_unanchored_master_under_memory_not_items` (passes inside a born `GitScratch` repo). `git::unanchored_frame` = `none_frame` (single construction site). | aligned |
| EX-2 | master-lint asserts per master: INV signature (`repo=""`, `anchor_kind=none`); valid `memory_type` and NOT `reference` literal; scope floor (≥1 path/glob/command, never tag-only) | met | `corpus::lint_master` (`src/corpus.rs:180`): checks raw `memory_type=="reference"` before parse → `ReferenceType`; parses → `NonEmptyRepo`/`Anchored`/`ScopeFloor`/`Schema`. Distinct `Violation` variants. `is_inv` (prune gate) stays repo+anchor only — lint is layered on top, as designed (notes PHASE-03). | aligned |
| EX-3 | OQ-C: `doctrine install` prints a hint to run `memory sync` (no orchestration); verb stays standalone | met | `install::sync_hint()` ("Next: run `doctrine memory sync`…") printed at end of `install::run`. Tests `install_hints_at_the_standalone_memory_sync_verb` + `install_writes_no_shipped_tree` (install creates `items/`, never `shipped/`). | aligned |
| EX-4 | `just check` green; SL-005/007/008 unchanged & green (record's normal path untouched) | met | See gate state above. | aligned |
| VT-1 | `record --global` produces master `repo=""`/anchor=none under `memory/`; normal record still requires+derives non-empty repo+anchor (gate intact) | met | `record_global_mints_an_unanchored_master_under_memory_not_items` asserts `m.scope.repo==""`, `m.anchor.kind==None`, dir under `memory/` not `items/`. Existing `repo_scoped_record_in_a_non_git_dir_errors_and_writes_nothing` (gate) still green. | aligned |
| VT-2 | master-lint catches each planted defect (non-empty repo / present anchor / reference literal / tag-only scope) with a clear signal | met | `lint_flags_a_non_empty_repo`, `lint_flags_a_present_anchor`, `lint_flags_a_reference_type_with_a_dedicated_signal` (asserts NOT a generic Schema bail), `lint_flags_a_tag_only_scope`, plus `lint_flags_an_unknown_type_as_schema` — all green. `lint_passes_a_clean_master` is the positive control. | aligned |
| VT-3 | install-hint references `memory sync`; install does not run sync (no shipped/ written) | met | `install_hints_at_the_standalone_memory_sync_verb` + `install_writes_no_shipped_tree`. E2E `full_install_gitignores_the_shipped_corpus` confirms install wires the denylist but does not materialize. | aligned |

## PHASE-05 disposition table

| id | criterion | status | evidence | disposition |
|---|---|---|---|---|
| EX-1 | triage table dispositions ALL 86 spec-driver memories into transferable/topic-applicable/inapplicable, with drop rationale | met | `triage.md`: 86 numbered rows, each a/b/c with a target slug or drop rationale; totals 5(a)+17(b)+64(c)=86. Drop rationales name spec-driver-internal / stack-specific (Python/Typer/Textual/pylint) / doctrine-dev gotcha. | aligned |
| EX-2 | corpus authored under `memory/` covering every OQ-A topic — every topic ≥1 master; each carries `repo=""`/anchor=none, ≥1 path/glob/command scope, valid type ≠ reference, reviewed seeded | met | 14 masters under `memory/mem_*`. Type tally: 5 signpost, 4 concept, 3 pattern, 2 fact (=14). `grep`: 0 with non-empty repo, 0 `reference` literal, 0 missing `reviewed` (all `2026-06-06`). Spot-check: overview carries `paths=[".doctrine/"]`+`commands=["doctrine"]`, `anchor_kind="none"`. Skeleton coverage table in `triage.md` maps every OQ-A topic → ≥1 slug. | aligned |
| EX-3 | master-lint passes over the WHOLE real corpus (schema+INVs+scope floor+type); corpus orients toward boot/skills/doc, not restating | met | `every_embedded_master_lints_clean` iterates `embedded_assets()` and lints each — green over the now-14-master embed. Bodies read as pointers (e.g. file-map references `.doctrine/` layout, lifecycle-start names route→…→close). | aligned |
| EX-4 | populated embed ships: `memory sync` lands corpus in shipped/ (gitignored); `retrieve --path-scope` surfaces a shipped master; boot snapshot lists shipped masters; `just check` green | met | E2E `sync_populates_the_shipped_corpus_then_is_idempotent_and_retrievable`: ≥12 masters under `shipped/`, only `mem_` uid dirs (no alias dupes), re-sync inert, `retrieve --command doctrine` surfaces `mem.signpost.doctrine.overview` with `staleness: reference`. `full_install_gitignores_the_shipped_corpus` (denylist). Live: `boot` snapshot Memory section lists all 14 shipped masters (verified at audit). | aligned |
| VT-1 | triage table has 86 rows each dispositioned; spot-check confirms drops + kept topics map to skeleton | met | `triage.md` totals reconcile to 86; skeleton coverage table present. Spot-check rows 79/80/81 (file-map/lifecycle/overview, type (a)) map to doctrine slugs; row 77 (workflow-commands) → cli-command-map authored as signpost not reference (Charge VIII). | aligned |
| VT-2 | coverage assertion: every OQ-A topic resolves to ≥1 master; master-lint green over full corpus (not trivial/empty) | met | `triage.md` "Skeleton coverage" — 14 OQ-A topics each → a master slug, none uncovered. `every_embedded_master_lints_clean` green over 14 masters. E2E asserts `masters.len() >= 12`. The slice does not pass on an empty corpus (Charge XII). | aligned |
| VT-3 | E2E: fresh repo → sync → shipped populated+gitignored → retrieve surfaces a shipped master, body renders (read_body fallback), non-decaying `reference` staleness; boot lists it; foreign items/ untouched | met (see finding F-1) | E2E covers populate+gitignore+retrieve+`staleness: reference`+idempotency+alias-dedup. Boot listing verified live (14 masters). `items/` untouched is structural: `sync_corpus` targets only `root.join(MEMORY_SHIPPED_DIR)` and `apply` only writes/prunes under `shipped/` — never an `items/` path (`plan_never_names_items_path`). `read_body` cross-root fallback is PHASE-02-tested (`read_body` items→shipped). | aligned |

## Findings

- **F-1 (evidence-granularity, aligned).** PHASE-05 VT-3 enumerates several sub-
  assertions in one E2E. Two are not asserted inside the single `e2e_memory_sync`
  test body but are covered elsewhere: (a) **boot snapshot lists a shipped master**
  — covered structurally (`boot.rs:127`→`list_rows`→`collect_all`, PHASE-02) and
  confirmed live at audit (all 14 masters in a temp repo's `boot.md` Memory
  section); (b) **a foreign `items/` file is untouched** — covered structurally
  (sync only ever targets `shipped/`; `plan_never_names_items_path`) rather than by
  a planted-items/-file E2E. Disposition: **aligned** — the guarantees hold and are
  evidenced; no behavioural gap, only a test-locality nuance. Not worth a
  belt-and-braces E2E addition.

- **F-2 (cosmetic, tolerated drift).** `doctrine adr new` stamped ADR-002
  `created = "2026-06-05"` while the session date was `2026-06-06` (ADR-001 carries
  the same one-day lag); the corpus masters carry `created="2026-06-05"`,
  `updated`/`reviewed="2026-06-06"`. The ADR clock appears to lag a day. Disposition:
  **tolerated drift** — cosmetic, pre-existing (not introduced by SL-018), and the
  evergreen class is decay-exempt so dated ordering does not affect staleness or
  the recency tiebreak materially. Flag only if dated ordering ever becomes
  load-bearing. (Already noted in `notes.md`; recorded here for closure.)

- **F-3 (design-honesty, aligned).** master-lint is a `#[cfg(test)]` validation gate
  this slice, not a runtime command surface — exactly as PHASE-04 designed (notes:
  "load-bearing over the real corpus in PHASE-05, still via the test gate"). The
  enforcement path for the real corpus is `every_embedded_master_lints_clean`, which
  runs in the normal `cargo test`/`just check` gate. Disposition: **aligned** —
  intentional; a runtime `memory lint` verb is out of scope (no plan criterion asks
  for it).

## Harvested memories (doctrine's own `items/`, not the shipped corpus)

Contributor-facing doctrine-dev gotchas — correctly kept in `items/` (design D7),
NOT shipped to downstream drivers:

- `mem.pattern.embed.rustembed-recompile-and-symlinks`
  (`mem_019e9a21f97a7d228c78013b3e8323c0`) — RustEmbed embeds at compile time
  (edit `memory/` ⇒ rebuild before any e2e; compounds the stale-`CARGO_BIN_EXE`
  footgun) AND follows symlinks (`mem.<key>` aliases double each master ⇒
  `gather_assets` must filter to canonical uid dirs).
- `mem.system.memory.global-master-authoring`
  (`mem_019e9a2211af7c929d791f4b3d0e64af`) — the `record --global` authoring seam:
  born-frame suppression via `git::unanchored_frame`, write into repo-root `memory/`
  (`MEMORY_MASTERS_DIR`), ride past the repo-anchor gate by explicit intent; embed →
  `memory sync` → `shipped/` → `collect_all` reach; master-lint invariants.

## Closure readiness

PHASE-04 and PHASE-05 EX/VT criteria are all **met / aligned**. No `fix now`, no
`follow-up slice`, no genuinely-unmet criterion. The one tolerated drift (F-2) is
cosmetic and pre-existing. Behaviour-preservation gate held. Ready for `/close`.

## PHASE-06 addendum (master-corpus drift reconciliation) — overall PASS

Appended after the original closure; the slice was reopened (`done → in_progress`)
to add a maintenance phase. Evidence at HEAD `5c4adfc` over a green `just check`.

- **EX-1 (re-align) — met.** cli-command-map names the `backlog` verb; file-map
  names `.doctrine/spec/{product,tech}/` + `.doctrine/backlog/`; skill-map names
  `/spec-product` + `/spec-tech`. Each master-lint-clean.
- **EX-2 (dispositions) — met.** All four orientation-grade memories authored
  since closure are dispositioned KEEP-repo-scoped, with rationale (phase sheet
  `## Decisions`; notes PHASE-06). Corpus stays lean at 14 (target ~12-18).
- **EX-3 (ship + behaviour-preservation) — met.** `memory sync` = 3 changed /
  11 unchanged / 0 prune (other 11 masters byte-unchanged); `just check` green.
- **VT-1 — met.** `every_embedded_master_lints_clean` + `sync_populates_the_shipped_corpus...`
  green after a rebuild (rust-embed compile-time embed honoured).
- **VA-1 — met.** Read-back confirms the three masters name the current surface
  and enumerate no ADR/PRD/REQ/SL numbers (no-restate principle held).

**Reconciliation note:** the pre-PHASE-06 `⚠ 3/5` was a false divergence — a
state-tree re-materialization had reset PHASE-04/05 (committed & audited at the
original closure) to `planned`. Re-flipped to `completed`; this is an inherent
property of disposable runtime state, not a defect.

No findings, no follow-up. Ready for re-`/close`.
