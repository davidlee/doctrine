# Design SL-018: Shipped orientation memory corpus

## 1. Design Problem

Doctrine's memory store serves **project-local capture** — git-anchored TOML+MD
entities recording what an agent learned working in *this* client repo. It does
not serve **framework orientation**: durable, repo-agnostic knowledge about *how
to drive doctrine itself* that should travel with the binary to every client.

Sibling project spec-driver ships ~86 flat `.md`+frontmatter memories
(repo-root `memory/`, force-included into its wheel) for exactly this. Doctrine
ships none. An agent dropped into a fresh client repo has the boot snapshot,
skills, and `doc/*`, but no **scope-retrievable** orientation corpus — nothing
that answers "I'm about to touch `install/manifest.toml`, what should I know"
through `doctrine memory retrieve --path-scope`.

This slice gives doctrine a shipped, doctrine-authored orientation corpus and a
materialize path to clients — modelled on (not ported from) spec-driver's corpus.

## 2. Current State

- **Memory entity** (SL-005/007/008): `.doctrine/memory/items/<uid>/{memory.toml,
  memory.md}`, committed (authored tier). Each carries scope (`paths`/`globs`/
  `commands`/`tags`), a `repo` coordinate, and a born git frame (`[git]` anchor).
- **Write gate** (`src/memory.rs:750-755`): a memory with a **non-empty `repo`**
  requires a born anchor — unanchorable ⇒ hard error. `record` always derives a
  non-empty `repo` from cwd git, so it can never mint a repo-empty memory.
- **Read path** (`src/memory.rs` parse): validates *shapes only* (schema_version
  == 1, closed vocab, non-empty title). It does **not** re-enforce
  scoped⇒anchored. A hand-authored `repo="", anchor_kind=none, scope.paths=[…]`
  memory parses fine.
- **Partition filter** (`src/retrieve.rs:172-174`): drops a memory whose `repo`
  is non-empty and ≠ the querying repo; **a `repo=""` memory is admitted in every
  partition** (the global hatch).
- **Scope match / staleness** (`retrieve.rs`): path-prefix matching is
  independent of `repo` and anchor; a scoped-but-unattested memory falls to the
  days-since-`reviewed` staleness metric (no crash).
- **Indexer** (`retrieve.rs:631`, `memory::collect_memories`): scans the single
  `items/` root.
- **Skills precedent** (SL-010): masters at repo-root `plugins/`, a *separate*
  `#[derive(RustEmbed)]`, materialized by a dedicated `doctrine skills` verb into
  a gitignored `.doctrine/skills` tree. NOT folded into `doctrine install` or
  `doctrine boot`.
- **Installer** (`doc/install-spec.md`): embeds `install/` via rust-embed, writes
  to `.doctrine/`, **never overwrites existing files** (a documented guarantee).

## 3. Forces & Constraints

- **The `repo` field is the cross-repo filter, by design.** A real repo-id means
  "scoped to that codebase" and *correctly* hard-filters out of other repos.
  Framework memories are repo-agnostic ⇒ `repo=""` is the honest encoding, not a
  hack.
- **The committed capture tree's invariants must stay intact** (behaviour-
  preservation gate). The scoped⇒anchored rule and `items/`'s contents may not
  change.
- **Install must keep its never-overwrite guarantee** — the corpus needs *refresh*
  (overwrite-to-source) semantics, which is the opposite. So it is not an install
  file.
- **No parallel implementation** — ride the skills materialize pattern and the
  existing memory entity/retrieval seams; do not fork a second memory format.
- **ADR-001 (module layering)**: leaf ← engine ← command, no cycles. New code is
  a thin command shell over a pure materializer.
- **Audience is the downstream agent driving doctrine, not the doctrine
  contributor.** Doctrine-repo development gotchas (rust/clippy/cargo) do not
  belong in the shipped corpus.

## 4. Guiding Principles

1. **One memory format, one retrieval path.** Shipped memories are native
   entities; they surface through `doctrine memory retrieve`/`list` and the boot
   snapshot like any other — that scoped-retrieval reach is the whole point.
2. **Tier honesty.** Shipped copies are *derived* (regenerable from the binary),
   so they are gitignored and live outside the committed capture tree.
3. **Provenance honesty.** `repo=""`, `anchor_kind=none` — assert nothing false
   about a client's git.
4. **Orient toward, don't restate.** The corpus points at boot/skills/`doc/*`;
   its unique value is *per-scope* retrieval the static surfaces can't give.
5. **Smallest seam that works.** A doctrine-owned private subtree needs no
   symlink dance; wholesale regenerate beats skills' ownership trichotomy here.

## 5. Proposed Design

### 5.1 System Model

Three trees, one per storage tier — the alignment that resolves every tension:

```
authored  (committed, doctrine repo only):
  memory/<uid>/{memory.toml, memory.md}        ← the masters; embedded via a new RustEmbed

derived   (gitignored, every repo incl. doctrine's):
  .doctrine/memory/shipped/<uid>/{memory.toml, memory.md}
                                               ← materialized by `doctrine memory sync`; 2nd scan root

captured  (committed, client repo):
  .doctrine/memory/items/<uid>/...             ← UNCHANGED; anchor invariant intact
```

Data flow:

```
memory/ (repo-root masters)
   │  #[derive(RustEmbed)] #[folder="memory/"]   (compile-time, parallel to install/, plugins/)
   ▼
binary
   │  `doctrine memory sync`  (regenerate gitignored subtree, overwrite)
   ▼
.doctrine/memory/shipped/<uid>/…
   │  collect_memories({items/, shipped/})        (merged candidate set)
   ▼
`doctrine memory retrieve|find|list`  +  boot snapshot Memory section
```

### 5.2 Interfaces & Contracts

**New verb** (in the `memory` command family):

```
doctrine memory sync            # bring .doctrine/memory/shipped/ to match embedded masters (idempotent diff)
doctrine memory sync --dry-run  # print plan (new / changed / prune / unchanged), exit
doctrine memory sync --yes      # no prompt
doctrine memory sync install    # wire the SessionStart hook (M1) — mirrors `boot install`
```

Mirrors `doctrine skills install` / `boot install`'s shape. First-time setup runs
it after `doctrine install`; refresh-on-upgrade re-runs it (or the M1 hook does).
**M1 hook wiring** — OQ-E: a *separate* SessionStart entry for `memory sync`, vs
extending the existing `boot` SessionStart hook to chain `doctrine boot &&
doctrine memory sync`. Lean **separate entry** (cohesion; sync and boot are
independent surfaces). Decide in plan.

**New embed** (parallel to `install/`'s and `plugins/`'s):

```rust
#[derive(RustEmbed)]
#[folder = "memory/"]
struct CorpusAssets;   // the authored masters
```

**Indexer second root — preserve the leaf (gate).** `collect_memories(items_root)`
is shared by retrieve (`retrieve.rs:633`), `memory list` (`memory.rs:1092`), and
`list_rows` (`memory.rs:1109`, which `boot.rs:127` calls), and existing tests call
it **directly** (`memory.rs:2896-2900`). Changing its signature breaks the
behaviour-preservation gate. So it stays unchanged; add a composite over it:

```rust
// unchanged leaf — per-root primitive; existing tests keep calling this.
fn collect_memories(items_root: &Path) -> Result<Vec<Memory>>

// new composite — union the capture + shipped roots, dedup by uid (items wins).
fn collect_all(root: &Path) -> Result<Vec<Memory>> {
    let mut ms = collect_memories(&items_root(root))?;
    let shipped = shipped_root(root);
    if shipped.is_dir() {
        for m in collect_memories(&shipped)? {
            if !ms.iter().any(|x| x.uid == m.uid) { ms.push(m); }
        }
    }
    Ok(ms)
}
```

retrieve (633), `run_list` (1092), and `list_rows` (1109) switch to `collect_all`.
**Consequence (intended):** shipped memories then surface in `retrieve`, `find`,
`list`, **and the boot snapshot Memory index** (boot rides `list_rows`) — exactly
the orientation reach we want. Existing direct-`collect_memories` tests are
untouched (gate holds).

**Body reads across roots.** `read_body` (`retrieve.rs:780`) today joins
`items_root`. A shipped memory's body lives under `shipped/`; `read_body` must try
the capture root then fall back to the shipped root (a shipped uid is absent from
`items/`).

**Materializer (pure / impure split, ADR-001):**

```rust
// pure: given embedded assets, produce the set of (relpath, bytes) to write.
fn plan_corpus(assets) -> Vec<MaterialisedFile>
// impure shell: rm-rf the shipped subtree, write the plan, report.
fn sync_corpus(root, plan) -> Result<SyncReport>
```

shipped/ is a **wholly doctrine-owned, gitignored** subtree (like `.doctrine/
state/`) — **it never holds local content** (D8). Sync is **idempotent /
diff-based**, not a blind regenerate: write masters that are new or changed,
**prune** shipped entries whose master no longer exists, no-op when identical.
This is cheaper than skills' ownership trichotomy (shipped/ is doctrine-only, so
no foreign-file question) and cheap enough to run **every session** via the M1
hook. **Safety**: the target is *always* `<validated-root>/.doctrine/memory/
shipped`, computed from `root::find`, never a user-supplied path; sync only ever
touches that one derived subtree, never `items/`.

### 5.3 Data, State & Ownership

A **shipped/global orientation memory** master:

```toml
memory_uid  = "mem_…"        # stable, hand-assigned once (uuid v7); the dir name
memory_key  = "mem.signpost.doctrine.overview"
schema_version = 1
memory_type = "signpost"     # signpost | concept | pattern | fact | reference(→ as a memory_type? see OQ-1)
status      = "active"
title       = "…"
summary     = "…"
created = "2026-06-…"; updated = "2026-06-…"

[scope]
paths    = ["…"]             # e.g. install/manifest.toml, .doctrine/slice
globs    = []
commands = []                # e.g. "doctrine slice", "doctrine memory"
tags     = ["doctrine", "orientation", …]
workspace = "default"
repo = ""                    # ← GLOBAL: admitted in every partition
repo_id_kind = "none"; repo_id_confidence = "low"

[git]
anchor_kind = "none"         # ← UNANCHORED: asserts nothing about client git
# commit/tree/ref_name/checkout_state_id/base_commit/verified_sha all empty

[review]
verification_state = "unverified"
[trust]
trust_level = "medium"
[ranking]
severity = "…"; weight = …
```

**Ownership.** Masters: authored, committed, doctrine repo. shipped/: derived,
gitignored, regenerated, never hand-edited. items/: client-captured, untouched.

**uid strategy.** Masters carry pre-minted stable uids (a `memory_key` is the
human handle). `record` can't mint these (it forces an anchor), so masters are
**authored files**, like templates — minted once at authoring (uuid v7), then
stable forever. Same uid in every client = same framework knowledge; correct.

### 5.4 Lifecycle, Operations & Dynamics

- **Author / revise** a master → bump `updated` → commit in doctrine repo.
- **Ship**: `cargo build` re-embeds; new binary carries the new corpus.
- **Materialize**: client runs `doctrine memory sync` (after `install`, or after
  upgrading the binary) → shipped/ brought to match the embedded masters
  (idempotent diff). **M1 — auto-refresh**: a SessionStart hook runs `memory
  sync` each session (mirrors `boot install`'s hook), so shipped/ self-heals on
  binary upgrade with no manual step. Idempotent ⇒ near-zero cost when in sync.
- **Retrieve**: merged into every `find`/`retrieve`/`list` and the boot snapshot.
- **Local orientation / override (M3, deferred)**: a client's own framework-level
  memories do **not** go in shipped/ — they live in committed `items/`
  (repo=<client>, unscoped/tag-scoped, unanchored), so sync never touches them and
  `collect_all` composes them automatically. Shadowing a *shipped* memory by
  `memory_key` is the future extension point (§7 D8).
- **Staleness**: `repo=""`, no anchor, no `verified_sha` ⇒ the days-since-
  `reviewed` metric. Evergreen corpus ⇒ this is weak signal; acceptable. The
  result renders an explicit state (never a silent hide).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV**: shipped memories always carry `repo=""` AND `anchor_kind=none`. (A
  master with a non-empty repo would self-exclude from clients; with an anchor it
  would lie.) → enforce with a `memory sync` validation pass / a master lint test.
- **INV (D8): shipped/ holds ONLY doctrine-authored masters** — never local
  content. This is what makes idempotent diff-sync safe and "not overwritten" a
  *structural guarantee*: local orientation/override lives in `items/`, which sync
  never touches. Violating this (a user dropping a file in shipped/) is the only
  way sync could prune local work; the tier convention + gitignore + docs prevent
  it. (M3 override/suppress builds on `items/` + key-precedence, not shipped/.)
- **INV**: shipped/ is gitignored in every repo; never committed. **Two surfaces**
  (the `mem.pattern.install.authored-entity-wiring` trap, inverted): (a) doctrine's
  **own** `.gitignore` negates `.doctrine/memory/` (line 17) and only re-ignores
  `index|embeddings|state/*` — `.doctrine/memory/shipped/` must be added beside
  them or it is committed-by-default here; (b) the client **manifest**
  `[gitignore].entries += ".doctrine/memory/shipped/"` for the denylist model.
- **INV**: `items/` and its scoped⇒anchored rule are unchanged (gate).
- **Edge — uid collision** items/ vs shipped/: practically impossible (disjoint
  minting), but defined: **items/ wins**, shipped/ duplicate dropped (committed
  capture outranks a shipped default). Logged at `find` debug, not an error.
- **Edge — sync over a stale shipped/**: wholesale regenerate ⇒ removed masters
  vanish, edited masters refresh. Correct by construction.
- **Edge — `repo=""` global memories pollute focused queries?** They are
  *path/command/tag*-scoped, so a scope-bearing query only surfaces them on a
  match; a bare `--query` already returns globals. Same behaviour as any global
  memory today.

## 6. Open Questions & Unknowns

- **OQ-A (corpus topic skeleton).** The triage of spec-driver's 86 → doctrine's
  set is *content*, executed in a later phase. The design fixes the **axes**:
  keep = framework-orientation for a client *driving* doctrine; drop =
  spec-driver-internal, stack-specific (Python/Typer/Textual/pylint), and
  **doctrine-repo development gotchas** (rust/clippy/cargo — those stay in
  doctrine's own `items/`, they are not for downstream users). Provisional
  doctrine skeleton (≈12-18 memories, not 86):
  - `signpost`: overview · file-map/layout · lifecycle-start (route→slice→design
    →plan→phase→audit→close) · skill/route map
  - `concept`: storage model + the storage rule · entity engine · memory model
    (capture vs this shipped corpus) · the routing gate
  - `pattern`: the core loop · conventions (conventional commits, pure/imperative
    split, behaviour-preservation gate, immutable PHASE/EN-EX-VT ids) · TDD
    red/green/**refactor**
  - `fact`: CLI-is-source-of-truth (don't guess ids/flags) · authored vs
    runtime vs derived tiers
  - `reference`: CLI command map
- **OQ-B (`reference` memory_type).** spec-driver uses a `reference` type;
  doctrine's enum is `concept|fact|pattern|signpost|system|thread` — no
  `reference`. Map references onto `signpost`, or add `reference` to the enum? Lean
  **map onto `signpost`** (no enum churn). Confirm in design review.
- **OQ-C (sync autorun).** Does `doctrine install` orchestrate `memory sync`, or
  is it a documented separate step? Lean **install prints a hint**; keep the verb
  standalone (skills parity). Decide before plan.
- **OQ-D (`reviewed` seeding).** Seed shipped masters' `[review].reviewed` to the
  authoring date so staleness has a baseline, or leave empty (→ unanchored)? Lean
  **seed**, so the day-metric is meaningful.
- **OQ-E (M1 hook wiring).** Separate SessionStart entry for `memory sync` vs
  extending boot's hook (§5.2). Lean **separate**. Plan-level.

## 7. Decisions, Rationale & Alternatives

- **D1 — native entities, not a flat format.** Rationale: scoped retrieval + boot
  listing + one format. *Alt*: spec-driver-style flat `.md` read directly —
  rejected: needs a new read surface, no scope retrieval, two formats.
- **D2 — `repo=""` global class.** Rationale: repo-id is the cross-repo filter;
  repo-agnostic memories must be empty to be admitted everywhere. *Alt*: anchor to
  doctrine's real remote — rejected: hard-filtered out of every client (dead on
  arrival). *Alt*: anchor to client — impossible (minted upstream).
- **D3 — derived + gitignored (`shipped/`), not committed `items/`.** Rationale:
  keeps the capture tree + invariant pristine; dissolves update-propagation;
  honest tiering. *Alt*: native-in-`items/` — rejected: pollutes capture tree,
  bends scoped⇒anchored, resurrects the merge/override problem.
- **D4 — dedicated `memory sync` verb.** Rationale: skills precedent; install's
  never-overwrite vs corpus's refresh are opposite contracts; boot is per-session
  and single-file (wrong cohesion). *Alt*: fold into install/boot — rejected
  (above).
- **D5 — masters at repo-root `memory/` + separate embed.** Rationale: under
  `install/` the scaffolder would write them to committed `items/`; skills already
  put masters at repo-root `plugins/` for this reason; mirrors spec-driver. *Alt*:
  `install/memory/` — rejected (double-handling).
- **D6 — idempotent diff-sync over a doctrine-private tree, no ownership
  trichotomy.** Rationale: shipped/ is doctrine-only (D8), so there is no
  foreign-file question skills' classify_link existed to solve; diff (not blind
  rm) makes it cheap enough for the per-session M1 hook. *Alt*: blind
  wholesale-regenerate — rejected: wasteful every session, and only "safe" by the
  same D8 convention anyway.
- **D7 — corpus audience = downstream driver; exclude doctrine-dev gotchas.**
  Rationale: a client agent drives doctrine, doesn't build it; rust/clippy
  gotchas are noise to them and already live in doctrine's `items/`.
- **D8 — local orientation/override lives in `items/`, never `shipped/`** (the
  consult outcome). Rationale: makes "sync never overwrites local work" a
  *structural guarantee* (sync only touches shipped/), and needs no new store —
  a repo-specific unscoped+unanchored memory in `items/` is already legal and
  `collect_all` already composes it. Shadowing a shipped memory by `memory_key`
  is the **deferred M3** extension (collect_all dedups by uid today; key-
  precedence is the seam). *Alt*: a writable shipped/ with merge semantics —
  rejected: resurrects the override/clobber problem this slice exists to avoid.
- **D9 — M1 auto-refresh via a SessionStart `memory sync` hook** (consult
  outcome). Rationale: answers "take effect in installer" without a manual step;
  idempotent sync ⇒ near-zero cost. M2 (staleness reaction) and M3
  (override/suppress) are **deferred to a follow-up slice + a behaviour-hooks
  ADR** — distinct concern, high blast radius, must not bloat this slice.

## 8. Risks & Mitigations

- **R1 — a second memory format creeps in by accident.** *Mitigation*: D1 — one
  format; masters are the same `memory.toml`+`md` schema, only field *values*
  differ. A master-lint test asserts the schema + INVs.
- **R2 — globals dilute scoped queries.** *Mitigation*: they are scoped (path/
  command/tag); a scope-bearing query only matches on hit — identical to existing
  global-memory behaviour. Covered by a retrieval test.
- **R3 — spec/impl drift (the new class outside §306).** *Mitigation*: the
  memory-spec amendment + ADR formalize the class; a golden test pins the
  repo=""/anchor=none admission path.
- **R4 — sync clobbers a client's real file.** *Mitigation*: shipped/ is
  doctrine-owned & gitignored; sync touches only that subtree; `--dry-run` shows
  the plan; never writes to `items/`.
- **R5 — corpus rots as doctrine evolves.** *Mitigation*: out-of-scope ownership/
  cadence follow-up (slice); the corpus is small + evergreen by construction.
- **R6 — foundational-process drift if the hook/override layer is mis-seamed**
  (the consult concern). *Mitigation*: D8 makes "sync never clobbers local work" a
  structural guarantee (local → `items/`, sync → shipped/ only); D6 idempotent
  sync is the per-session safe refresh; M2/M3 are explicitly deferred to a
  dedicated slice + behaviour-hooks ADR rather than improvised here. The seam
  (collect_all uid-dedup → future key-precedence) is documented, not foreclosed.

## 9. Quality Engineering & Validation

- **Behaviour-preservation gate**: existing memory suites (SL-005/007/008) pass
  **unchanged** — proves capture/retrieval semantics intact.
- **New unit tests**:
  - `plan_corpus` materializer (pure) — new/changed/prune/unchanged plan from
    embedded assets vs an on-disk shipped/ state (idempotency: identical input ⇒
    all-unchanged, zero writes; removed master ⇒ prune; never references items/).
  - `memory sync install` wires a SessionStart hook idempotently (M1) — mirror
    boot's hook-install test.
  - `collect_all` — union, items/-wins dedup, shipped-root-absent → equals
    `collect_memories(items)`. (Existing `collect_memories` tests stay unchanged
    — gate proof.)
  - `list_rows`/boot include shipped memories once a shipped root exists.
  - `base_filter` admits `repo=""` shipped memory in an arbitrary client
    partition (the global hatch — extend existing B20 test).
  - scope-match + staleness for a `repo="", anchor=none, paths=[…]` memory
    (explicit state, no crash).
  - master-lint: every embedded master satisfies schema + INVs (repo="",
    anchor=none).
  - gitignore wiring: a fresh install adds `.doctrine/memory/shipped/` (client
    manifest); a doctrine-repo check asserts the same path is in `.gitignore`.
- **E2E** (`memory sync`): fresh repo → sync → shipped/ populated, gitignored;
  `retrieve --path-scope <p>` surfaces a shipped memory; re-sync overwrites; foreign
  `items/` file untouched. (Mind `mem.pattern.testing.stale-cargo-bin-exe`.)
- **`just check`** clean (clippy zero-warning, bins/lib only).

## 10. Review Notes

### Adversarial self-review (round 1) — integrated

- **F1 (correctness): boot/list would not see shipped memories, and naïvely
  fixing it breaks the gate.** `boot.rs:127`→`list_rows`→`collect_memories`
  (single root); the same leaf is called directly by existing tests
  (`memory.rs:2896-2900`). Resolved §5.2: keep the leaf unchanged, add a
  `collect_all` composite used by retrieve/list/list_rows; existing tests
  untouched, boot gains shipped memories as a wanted consequence.
- **F2 (verified safe): repo-root `memory/` is outside `.doctrine/*`** → committed
  by default; no negation needed there.
- **F3 (correctness): gitignore trap, inverted.** `.doctrine/memory/shipped/` is
  committed-by-default in *doctrine's own* repo (under the `!.doctrine/memory/`
  negation). Two surfaces now in §5.5 INV + §9 + §affected-surface: doctrine's
  `.gitignore` AND the client manifest.
- **F4 (correctness): cross-root body reads.** `read_body` joins `items_root`
  only; must fall back to `shipped/`. Noted §5.2.
- **F5 (safety): wholesale `rm`.** Constrained to the computed validated-root
  subtree only. Noted §5.2.

### Doctrinal alignment

- **ADR-001 (layering)** honoured: `collect_all`/`plan_corpus` are pure
  engine-leaf helpers; `memory sync` is a thin command shell. No new cycle.
- **Storage rule**: masters are authored (TOML+MD), shipped/ is derived; no
  queried data in prose. Consistent.
- **Behaviour-preservation gate**: SL-005/007/008 suites unchanged (§9).
- **Spec authority**: the new class contradicts memory-spec §306 prose
  (*"unanchored permitted only for unscoped"*); the planned amendment + ADR are
  the reconciliation, not an improvisation — flagged, not normalized.

### Consult outcome (M1/M2/M3 boundary)

User raised repo-local behaviour hooks (update/sync memories on code/fact change)
and the "local changes not overwritten" risk. Resolved via `/consult`: SL-018 =
**mechanism + M1 refresh hook**; M2 (staleness reaction) and M3 (override/
suppress) deferred to a follow-up slice + a behaviour-hooks ADR. The foundational
fix taken now is **D8** (local content never in shipped/ ⇒ "not overwritten" is
structural) and **D6** (idempotent diff-sync, hook-safe). Seam left open, not
foreclosed: `collect_all` uid-dedup → future `memory_key` precedence for M3.

### Open for external pass

OQ-A (topic skeleton), OQ-B (`reference` type), OQ-C (sync autorun), OQ-D
(`reviewed` seeding), OQ-E (M1 hook wiring) remain — none architectural; all
decidable in plan or by an inquisition.
