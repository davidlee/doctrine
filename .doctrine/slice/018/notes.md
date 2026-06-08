# SL-018 — implementation notes (durable)

Durable cross-phase facts. Disposable scratch lives in the gitignored phase
sheets (`state/slice/018/phases/`); the handover is also disposable. This file
survives — keep only what a future agent needs and can't re-derive cheaply.

## PHASE-01 — DONE (commit `5c6e0ce`)

The scripture gate. Landed:
- **ADR-002** (`.doctrine/adr/002/`, status `accepted`) — sanctions the
  **global / unanchored / path-scoped / derived** memory class, defined by the
  signature **`repo="" && anchor_kind=none`** (NOT a new `memory_type`), plus the
  **fourth staleness disposition**: that class is *evergreen / reference-grade*,
  decay-exempt, rendering a non-decaying **`reference`** state.
- **`doc/memory-spec.md`** amended at three consistent sites: § Scope & anchoring
  (scoped+unanchored carve-out), § Retrieval partition (`repo=""` admitted in
  every partition), § Retrieval Staleness (4th table row + `reference` added to
  the explicit-state enum).

## Corrected seam refs (supersede plan.toml / the old handover)

Verified against source this session — the authored `plan.toml` carries a couple
of stale line refs. These are the real ones:
- **`read_body` lives in `src/memory.rs:1059`**, signature `read_body(items_root,
  uid)` — NOT `retrieve.rs:780`. Single caller `retrieve.rs:772`; **no direct
  test caller** → safe to re-key to `root` for the cross-root (items→shipped)
  fallback.
- **`base_filter` (`retrieve.rs:169`) ALREADY admits `repo=""`** in any partition
  — documented at `:173-174` (review B20). So the "admission" work is a **golden
  test only**, no `base_filter` code change. (The dormant hatch is real: zero
  `repo=""` memories exist today because `record` always derives a non-empty repo
  via the write gate `memory.rs:753`.)
- **Staleness fix** (`retrieve.rs:310` `staleness()`): a global master has
  `scope.paths` set + a seeded `reviewed`, so today it falls to branch 2
  (reviewed-time) and **decays**. The `Reference` branch must be inserted **after
  branch 1 (attested), before branch 2**, keyed `m.scope.repo.is_empty() &&
  m.anchor.kind == AnchorKind::None`. `Staleness` enum at `:267`, `label()` at
  `:276` (+ its test at `:1716`).
- **Leaf gate:** `collect_memories` (`memory.rs:1069`) has direct test callers at
  `memory.rs:2896, 2900` — those prove behaviour-preservation, so the leaf stays
  byte-unchanged; `collect_all(root)` is added *over* it. `MEMORY_ITEMS_DIR` const
  at `memory.rs:708` (add `MEMORY_SHIPPED_DIR` beside).

## PHASE-02 — DONE (commit `a02fb26`; memory `92f498a`)

Retrieval reach for the global class. Landed in `src/memory.rs` + `src/retrieve.rs`,
444 unit + e2e green, behaviour-preservation gate held (SL-005/007/008 unchanged).

- **`collect_all(root)`** (`memory.rs`, over the byte-unchanged `collect_memories`
  leaf): unions `items/` + `shipped/`, dedup by uid **items-win**, via a
  `BTreeSet<uid>` (NOT HashSet — `clippy::disallowed-types`, see new memory
  `mem.pattern.lint.disallowed-types-collections`). shipped-absent ⇒ byte-identical
  to the leaf. `MEMORY_SHIPPED_DIR` const added at `memory.rs:712`.
- **Three callers switched** to `collect_all`: `load_query` (retrieve), `run_list`,
  `list_rows` (the boot seam) — shipped now surfaces in find/retrieve/list + boot.
- **`read_body` re-keyed to `root`** (`memory.rs`): try `items/`, fall back to
  `shipped/`. Needs a non-empty filter (safe_join succeeds on a missing dir → empty
  read), so a present-but-EMPTY items body falls through to shipped — benign
  (empty == missing per the show contract). `Loaded.items_root` dropped.
- **`Staleness::Reference`** evergreen disposition (`retrieve.rs`), via predicate
  `is_global_reference`.

**SUPERSEDES the staleness ref above (lines ~33-38):** the bare key
`repo.is_empty() && anchor.kind==None` is WRONG — it catches the *scopeless*
default test fixtures (e.g. `staleness_no_anchor_no_date_is_unanchored`) and breaks
the SL-008 gate. The correct key carries the **scope floor** (ADR-002: the class is
path/glob/command-scoped; a scopeless `repo=""` memory is illegal/lint-target). The
shipped `is_global_reference` keys on **scoped ∧ `repo=""` ∧ anchor=none ∧ no
`verified_sha`**; branch placed after attested (1), before reviewed-time (3).
Canon (ADR-002 + design §5.4) outranks the plan/sheet snippet here.

## PHASE-03 — DONE (commits `c68fee3` feat, `5be7661` test)

The sync **write** path. New `src/corpus.rs`; boot's hook seam generalized; both
gitignore surfaces. 463 unit/bin + e2e green, clippy zero-warning, fmt clean.

- **`src/corpus.rs`** — `#[folder="memory/"] CorpusAssets` embed (repo-root
  `memory/`, committed, `.gitkeep`-seeded so it derives while empty); pure
  `plan_corpus(assets, children) -> CorpusPlan` (new/changed/unchanged/prune/skipped,
  idempotent, BTreeMap-keyed, never names `items/`); impure `sync_corpus(root,
  assets, dry_run) -> SyncReport` gather→plan→apply. INV-prune gate is `is_inv` =
  `scope.repo.is_empty() && anchor.kind==None` — **repo+anchor only, NO scope floor**
  (D8: shipped/ is doctrine-owned; the floor is PHASE-04 master-lint, NOT a prune
  gate — deliberately narrower than PHASE-02's `is_global_reference`).
- **Prune safety (Charge III):** stray files + unparseable/non-INV dirs classify as
  `Skipped` (never `Prune`), proven RED-first. Target is always
  `root.join(MEMORY_SHIPPED_DIR)`; `apply` only `remove_dir_all`s uids that passed
  parse∧INV∧absent-from-embed.
- **Hook seam generalized, not copied (no-parallel-impl).** `boot::HookSpec
  { command, is_ours: fn }` + `HookSpec::boot`/`::sync`; `plan_hook`/`fallback_for`/
  `install_claude_hook` are the generic core, `find_owned` takes `is_ours`. Boot's
  `plan_session_hook`/`fallback_snippet`/`install_refresh` keep exact signatures as
  thin callers → **33 boot tests passed byte-unchanged** (no STOP). `is_doctrine_sync_command`
  uses **suffix-strip `" memory sync"`** (two args ≠ boot's single-arg `rsplit_once`);
  disjoint from boot → two independent `SessionStart` entries (OQ-E).
- **Single live write path:** `run_sync` drives `sync_corpus` for both preview
  (`dry_run`) and apply — no dead `sync_corpus` (clippy `-D dead-code` bins+lib).
  `plan_session_hook` became `#[cfg(test)]` (production refresh now via
  `install_claude_hook`).
- **`SyncCommand::Install` dropped `--agent`** (sheet suggested it): the sync hook
  is Claude-`SessionStart`-only; codex has no equivalent. YAGNI vs dead surface.
- **gitignore both surfaces:** `.gitignore` + `install/manifest.toml` denylist add
  `.doctrine/memory/shipped/`. Verified live via `git check-ignore` (shipped ignored,
  items/ + repo-root `memory/` still tracked).
- **Empty-embed reality:** populate-from-embed (VT-4) proven at integration
  (synthetic assets through the full write path); binary e2e proves wiring + no-op
  + gitignore only. The real corpus is PHASE-05.
- **No-root e2e footgun:** `root::find` walks CWD→`/`; a stray `/tmp/.git` made
  default tempdirs resolve a root. e2e picks a marker-free base. Captured as memory
  `mem.pattern.testing.no-root-find-walk`.

## PHASE-04 — DONE (master authoring path + master-lint)

`record --global` mints a `repo=""`/anchor=none master under repo-root `memory/`
(`MEMORY_MASTERS_DIR`), reusing the uid-mint + schema validation and bypassing the
repo-anchor gate by explicit intent (`git::unanchored_frame`); the normal-path gate
is byte-unchanged. `corpus::lint_master` (test gate, not a runtime verb — F-3) asserts
per master: INV signature, valid `memory_type` ≠ `reference` literal, scope floor
(≥1 path/glob/command). `install::sync_hint()` points at the standalone `memory sync`
verb without orchestrating it; install never writes `shipped/`. Behaviour-preservation
gate (SL-005/007/008) held — only mechanical `global: false` field initializers added.

## PHASE-05 — DONE (the real corpus)

`triage.md` dispositions all 86 spec-driver memories (5 transferable / 17
topic-applicable / 64 inapplicable, each with rationale). 14 masters authored under
`memory/` (5 signpost, 4 concept, 3 pattern, 2 fact), every OQ-A topic ≥1 master, all
`repo=""`/anchor=none/scope-floored/typed/reviewed-seeded. `every_embedded_master_lints_clean`
runs master-lint over the full 14-master embed in `just check`. E2E
`sync_populates_the_shipped_corpus...` proves populate→idempotent→`retrieve` surfaces a
shipped master at `staleness: reference`→alias-dedup; boot snapshot lists all 14 live.

## PHASE-06 — DONE (commit `5c4adfc`; plan `75d388d`)

Post-closure maintenance: re-align the three *enumerating* signpost masters that
fell behind the surface SL-019..023 grew. Additive only (10 insertions, 3 files).

- **cli-command-map** (`mem_019e9a12139e7c52bb0ee2b82fb79868`) — added the
  `backlog` verb (`new · list · show · edit`).
- **file-map** (`mem_019e9a11cda27db19c0c75bafa453d5d`) — added the
  `.doctrine/spec/{product,tech}/` and `.doctrine/backlog/` entity dirs.
- **skill-map** (`mem_019e9a1200b6796094f9e31ff1666390`) — added `/spec-product`
  + `/spec-tech`.
- **Behaviour-preservation:** the other 11 masters byte-unchanged; `memory sync`
  reports **3 changed / 11 unchanged / 0 prune**. `every_embedded_master_lints_clean`
  + the SL-018 sync e2e stay green. No ADR/spec numbers enumerated (no-restate held).
- **4 memories dispositioned, all KEEP repo-scoped** (drift was in the signposts,
  not a concept gap; corpus stays lean at 14): `canonical-change-loop` already
  covered by lifecycle-start + core-loop; `descent-descends-from` +
  `shipped-not-reachable` are doctrine-dev (D7); `backlog.work-intake-membership`
  deferred (existence now surfaces via the two signpost edits).
- **Rollup correction (this session):** PHASE-04/05 were committed & audited at the
  original closure but a state-tree re-materialization had reset them to `planned`,
  giving a false `⚠ 3/5`. Re-flipped to `completed` before adding PHASE-06.

## Closure (SL-018)

Audit `overall=pass` (audit.md, HEAD `4e3cb65`): all PHASE-04/05 EX/VT aligned; F-1
(test-locality nuance) + F-3 (lint is a test gate by design) aligned, F-2 (ADR clock
one-day lag) tolerated cosmetic drift. Rollup `5/5`, lifecycle `done`, `⚠` divergence
cleared. State tree was re-materialized this session, so PHASE-01/02/03 were re-flipped
`completed` to match their committed work.

## Decisions carried forward

- **read_body re-keyed to `root`** (not a second `shipped_root` arg) → drops the
  now-purposeless `Loaded.items_root` field (`retrieve.rs:598`). `dead_code` is
  denied, so the field must be removed, not left.
- **items-win dedup is silent** — design says "logged at find debug"; the repo has
  no debug-log facility (`print_stdout` denied), so the dropped duplicate is
  silent. Acceptable (uid collisions are practically impossible — disjoint
  minting).

## Environment hazard (this session)

`SL-015` is a **shared branch with an active concurrent agent** (unrelated SL-017/
SL-013 work) doing stage-all + commit. It absorbed this slice's `plan.{toml,md}`
into one of its commits (`2ce9325`) and superseded the standalone `plan(SL-018)`
commit `517d4a6` — content was preserved, nothing lost. Lesson for the executor:
**commit your own `src/**` promptly and verify it's reachable**; don't leave work
uncommitted across long gaps. Leave the concurrent agent's files
(`slice/013/*`, SL-017 `src/lexical.rs` etc.) alone.

## Minor / open

- `doctrine adr new` stamped `created = "2026-06-05"` though the session date is
  `2026-06-06` (ADR-001 carries the same). The ADR clock appears to lag a day —
  not investigated; cosmetic. Flag only if dated ordering ever matters.
