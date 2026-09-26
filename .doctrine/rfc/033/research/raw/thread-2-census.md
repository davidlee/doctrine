I now have all evidence. Writing the brief.

# Scout Brief: Doctrine's "learning surface" — surface inventory, duplication, projection mechanics

## Thread 2 — surface inventory

All counts from commands run this session, noted per row.

| surface | extent | lines | home path |
|---|---|---|---|
| boot snapshot (generated runtime) | 1 file, 30 `##` sections | 627 (`wc -l .doctrine/state/boot.md`) | `.doctrine/state/boot.md` |
| shipped memory corpus (`.md` bodies) | 35 real memory dirs (`find -type d`), 31 slug symlinks, 71 files | 1260 total (`find memory -name memory.md \| xargs wc -l`) | `memory/` |
| published reference docs | 17 `install/*.md`; 15 published under `reference/` (+`LICENSE`+`doctrine.toml.example`) | 2182 (`wc -l install/*.md`) | `install/` |
| skill masters | 39 skill dirs, 41 `.md` files | 5315 (`find .agents/skills -name '*.md' \| xargs wc -l`) | `.agents/skills/` |
| `.pi/skills` view | 39 symlinks (1 dir) → `.agents/skills/*` | 5315 (same bytes via symlink) | `.pi/skills/` |
| README | 1 | 343 (`wc -l README.md`) | `README.md` |
| web map frontend | source `web/map/src/*.ts` (22 files) + `index.html`; embed `dist/` | n/a (TS) | `web/map/`, `web/map/dist/` |
| local (client) memories | ~200+ items incl. `mem.signpost.project.orientation` | — | `.doctrine/memory/items/` |
| installed `.doctrine/*.md` copies | 9 reference docs | 889 (`wc -l .doctrine/*.md`) | `.doctrine/` |

### 1. boot.md projection sources

`.doctrine/state/boot.md` is 627 lines, regenerated (gitignored runtime — `.gitignore:41` `.doctrine/state/`). Projected by `boot_sequence()` (`src/boot.rs:104-138`) via `render_boot` (`src/boot.rs:143-160`). The 30 sections come from:

- `## Authority` ← `SourceKind::Static("authority.md")` (`src/boot.rs:108`), read from the **embed** via `install::asset_text` (`src/boot.rs:245`, `src/install.rs:537`). Verified: boot §Authority (lines 5-44) is byte-identical to `install/authority.md` (39 lines) bar one trailing blank.
- `## Routing & Process` ← `Static("routing-process.md")` (`src/boot.rs:110`); boot lines 46-133 == `install/routing-process.md` (86 lines).
- `## Commands` ← `SourceKind::CommandMap` (`src/boot.rs:113`), the injected `render_boot_map` (`src/commands/cli.rs:1048`).
- `## Governance (project)` ← `SourceKind::Governance` (`src/boot.rs:114`), reading `.doctrine/governance.md` from disk (`GOVERNANCE_REL`, `src/boot.rs:56`, produced at `src/boot.rs:248-251`).
- `## Accepted ADRs` / `## Active Policies` / `## Active Standards` ← `GovRows` over the entity corpus (`src/boot.rs:115-127`).
- `## Memory` ← `SourceKind::Memories` → `memory::boot_keys` (`src/boot.rs:128`, `src/memory.rs:2999`) — active signpost keys, uid fallback for keyless.
- `## Onboarding` ← `SourceKind::Onboarding` → `memory::onboarding_bodies(root)` (`src/boot.rs:131`, `src/memory.rs:3013`), selecting memories tagged `onboarding`, inlined verbatim.
- `## Model band` ← `Static("model-band.md")` (`src/boot.rs:135`).
- `## Invoking doctrine` ← `SourceKind::ExecPath` (`src/boot.rs:137`), deliberately last (build-volatile).

The four `Static` assets are the `install/`-relative names (`authority.md`, `routing-process.md`, `model-band.md`); boot never reads them from disk — always the embed (`src/boot.rs:245`, comment `src/boot.rs:242`). `install/manifest.toml` does **not** declare boot projection; it declares base backings (`[base] backings = [".gitignore","doctrine.toml","project-orientation.md"]`) and gitignore entries — projection is code-driven.

### 2. `memory/` — dirs vs symlinks reconciled

Two commands disagree by construction:
- `find memory -mindepth 1 -maxdepth 1 -type d` = **35** (does not follow symlinks — counts only real directories).
- `ls -d memory/*/` = **66** (the glob matches symlinks too, and `ls -d */` lists them as if dirs).
- `find memory -maxdepth 1 -type l` = **31** (all resolve; 0 broken).

Reconciliation: **66 = 35 real dirs + 31 slug symlinks.** Of the 35 dirs, 31 are id-named (`mem_<uid>/`) and 31 of *those* have a title-slug symlink sibling. The remaining **4 dirs are slug-named real dirs** — `mem.concept.backlog.work-intake-membership`, `mem.signpost.doctrine.concept-map`, `mem.signpost.doctrine.rec`, `mem.signpost.doctrine.rfc` — newly minted locally without a `mem_<uid>` id-dir sibling yet. So **35 distinct memory entities**, 26 of which are also reachable by slug path.

Keys (from `memory_key` in each `memory.toml`, 35 total) and types (`memory_type` histogram: **21 signpost, 8 concept, 4 pattern, 2 fact**):

- **signpost (21):** `mem.signpost.doctrine.{overview,file-map,lifecycle-start,skill-map,install,reference-docs,relating-entities,recording-memories,backlog,adrs,specs,requirements,audit,revisions,policies-standards,knowledge,review,dispatch,rfc,rec,concept-map}`
- **concept (8):** `mem.concept.backlog.work-intake-membership`, `mem.concept.doctrine.{storage-model,entity-engine,memory-model,routing-gate,boot-snapshot,reading-entities,hymn-cascade}`
- **pattern (4):** `mem.pattern.doctrine.{core-loop,conventions,tdd-loop,close-drift-discharge-rec}`
- **fact (2):** `mem.fact.doctrine.{cli-source-of-truth,storage-tiers}`

Total body lines: 1260 (per-file max 82 = `mem.signpost.doctrine.overview`).

### 3. `install/*.md` → `reference/*` address construction

17 files in `install/` (2182 lines). The address↔backing mapping is declared in `publication/manifest.toml` (a **separate embed root** from `install/` — `src/asset_source.rs:29` `#[folder = "publication/"]`; `publication/manifest.toml:5-7`). 18 `[[entry]]` blocks carry `address = "reference/…"` (17 distinct — `reference/claude-activation.md` appears **twice**: as a reference and once more). 15 of the 17 `install/*.md` get a `reference/` address; the two exceptions are:
- `install/project-orientation.md` — a **base backing**, installed write-if-absent to `.doctrine/project-orientation.md`, deliberately *not* a reference (`install/manifest.toml:[base]`; test at `src/install.rs:3806-3816` asserts exactly `{.gitignore, doctrine.toml, project-orientation.md}` and NO memory seed).
- `install/claude-activation.md` — published as `reference/claude-activation.md` *and* also present as an integration asset (the duplicate block).

The `<name>.md → reference/<name>.md` pattern is a per-entry declared mapping (`publication/manifest.toml:48-49` shows `address="reference/LICENSE", backing="LICENSE"`; `:273-274` `reference/design-payload-contract.md ← design-payload-contract.md`). Address construction: `LogicalAddress::parse` (`src/publication.rs:162`), admitted by `PublicationManifest::admit` (`src/publication.rs:317`), resolved by `Resolver::emit` (`src/publication.rs:495`). `src/commands/publication.rs:30-52` (`doctrine publication validate`) resolves+emits every entry. `doctrine library {list,tree,show}` is a pure read veneer over the same resolver (`src/commands/library.rs:2-13, 82-135`). The reachability invariant is *derived*, not curated: `{embedded_filenames()} - {base backings} ⊆ published backings` (`publication/manifest.toml:10-12`).

### 4. `.pi/skills/`

39 entries — **39 symlinks, 0 real files** (`find .pi/skills -maxdepth 1 -type l | wc -l`), each `-> ../../.agents/skills/<name>`. Total bytes = the masters: 41 `.md`, **5315 lines** (`find -L .pi/skills -name '*.md' | xargs wc -l`). (39 masters; `rigour` and `worktree` each carry a second `.md`.)

### 5. `README.md`

**343 lines** (`wc -l README.md`).

### 6. `web/map/` entry points and `doctrine onboard`

Entry: `web/map/index.html` + `web/map/src/app.ts` (22 `.ts` files incl. tests), built to `web/map/dist/` (gitignored). The binary embeds `dist/` via `src/map_server/assets.rs:14` (`#[folder = "web/map/dist/"]`), served by `serve_embedded` (`src/map_server/assets.rs:33`). `doctrine serve` (`src/commands/serve.rs`) starts the **MCP** server, not the map. The map server is reached via `doctrine map serve` and `doctrine onboard`:
- `src/commands/cli.rs:138-139` declares `Onboard`; dispatch at `src/commands/cli.rs:1950` → `crate::commands::map::run_onboard()`.
- `run_onboard` (`src/commands/map.rs:102`) calls `run_serve(None, onboard_args())`; `onboard_args` (`src/commands/map.rs:91-99`) sets `focus: Some("mem.signpost.doctrine.overview")` (`ONBOARDING_MEMORY_KEY`, `src/commands/map.rs:27`) with `open: true`.
- `run_serve` (`src/commands/map.rs:55`) hydrates the catalog and hands a `Config` to `crate::map_server::serve` (`src/commands/map.rs:83`), which serves the embedded `dist/` assets.

---

## Thread 2 — duplication evidence

**A. `mem.signpost.doctrine.overview` body (82 lines)** — duplicated verbatim in:
- `memory/mem_019e9a11b3797af3a8833c67acfa69bf/memory.md` (the source; key/id at `memory.toml:1-2`).
- boot snapshot §Onboarding, **`.doctrine/state/boot.md:390-480`** (inlined by `memory::onboarding_bodies`, `src/memory.rs:3013`; heading at `:391`).
- The boot context header injected in this session (the `# Doctrine overview` block) — same bytes.
- **Not** `install/project-orientation.md`: that template's §Project Purpose etc. are *stubs* (`install/project-orientation.md:11-12` comments), while boot.md:481-600 is a **different** memory — `mem.signpost.project.orientation` (local item `mem_019ef1ae52c27ac2867b91db044f62a1`, tags `["onboarding"]`). So the onboarding tag selects **two** bodies into boot §Onboarding.

**B. `mem.signpost.doctrine.reference-docs` vs `doctrine library tree`** — the memory (`memory/mem_019ec92b10037850817507044f0f99ef/memory.md`) names 4 docs: `using-doctrine.md`, `glossary.md`, `dispatch-mechanics.md` (and implicitly the `reference/` tier). `doctrine library tree` (run this session) lists **18 reference entries across many more files** — `authority.md`, `authority-model.md`, `boot-footer.md`, `design-payload-contract.md`, `design-run-obligations.md`, `design-run-stages.md`, `routing-process.md`, `review-ledger.md`, `harvest.md`, `governance.md`, `model-band.md`, `shipped-corpus-authoring.md`, `claude-activation.md`, `LICENSE`, `doctrine.toml.example`. The memory's harder invariant (it claims the *general* authorities are two docs) has drifted from the manifest's 18-row reality; the memory does not enumerate the domain docs it could. No `file:line` in the memory corresponds to the manifest's list.

**C. `mem.signpost.doctrine.skill-map` vs the actual skill list** — the memory (`memory/mem_019e9a1200b6796094f9e31ff1666390/memory.md`) deliberately **does not** list skills; it defers: "The routing table is authoritative in boot.md" (`:7`) and hands to wikilinks. So there is no enumerated duplication — but the *skill set* is restated in at least three surfaces: boot §Routing table (`src/boot.rs:106-137`, ~40 rows), `install/routing-process.md:1-86` (source), and boot's §"useful commands"/skill-map pointer. 39 skills exist (`.agents/skills/`), 39 symlinked (`.pi/skills/`); boot's routing table names ~40 route targets. Any skill add/rename must touch `install/routing-process.md`, boot regeneration, and the skill-map wikilink targets.

**D. Storage rule / storage tiers / "read via show"** — restated in 8 files (grep over `memory/ install/ .doctrine/ README .agents/`):
- `mem.fact.doctrine.storage-tiers` — `memory/mem_019e9a12c5a97d00b6da9ad9686c9945/memory.md:1-11` (canonical) and its title `memory.toml:6`.
- `mem.concept.doctrine.storage-model` — `memory/mem_019e9a1234ff7e619d865592b0042cb9/memory.md:3`.
- `mem.signpost.doctrine.overview` — `memory/mem_019e9a11b3797af3a8833c67acfa69bf/memory.md:25-26,43,71` (three restatements incl. the table row and the Conventions bullet).
- `mem.signpost.doctrine.file-map` — `memory/mem_019e9a11cda27db19c0c75bafa453d5d/memory.md:44`.
- `install/using-doctrine.md:177,188-189` and `install/glossary.md:109-110`.
- `.agents/skills/canon/SKILL.md:32`.
- boot snapshot picks all of these up: `.doctrine/state/boot.md:415-416,433,461,518,536,590` (the same facts appear because boot inlines the overview memory **and** the boot's own §Memory table).
- "read via `show`" specifically: `memory/mem_019ec92b10007cf3b62bd5d02bce083c/memory.md:3,21`; `memory/mem_019ec92b10037850817507044f0f99ef/memory.md:4,33`; `memory/mem_019ec92b0ffc79d294a49559db9aa12a/memory.md:11`; `install/agents/claude/capsule-phase-planner.md:28`; `install/using-doctrine.md:210`; boot `.doctrine/state/boot.md:499`.

**E. "CLI is source of truth / don't guess the command shape"** — **23 files** (grep):
- Canonical: `mem.fact.doctrine.cli-source-of-truth` (`memory/mem_019e9a12b09472b3a99ac6cb071a6756/memory.md:1-4`) + title `memory.toml:6`.
- Restated per-verb in **12 shipped memories**: overview `memory/mem_019e9a11b3797af3a8833c67acfa69bf/memory.md:27-28,49,70`; reading-entities `…10007cf3…:21`; revisions `…301478…:17`; rfc `mem.signpost.doctrine.rfc/memory.md:16`; review `…d279444273b093816ccbb8e5da64…:12`; knowledge `…d2791edc70e2a2a1caeb2521aad0…:43`; policies-standards `…30177c40…:13`; relating-entities `…100472…:9`; concept-map `mem.signpost.doctrine.concept-map/memory.md:17`; recording-memories `…100678…:8`; rec `mem.signpost.doctrine.rec/memory.md:16`; requirements/coverage `…301176…:9`; install `…0ffc79d294a49559db9aa12a…:31`; lifecycle-start `…11e8337613bdf8e96f75a9e6b2…:10`; conventions `…128a0674…:19`.
- `install/using-doctrine.md:210`, `install/shipped-corpus-authoring.md:28`, `install/routing-process.md:59`.
- Skills: `route`, `elicit:15`, `backlog:14`, `canon:13`, `knowledge:16`.
- boot snapshot: `.doctrine/state/boot.md:417-418,439,460,495`.

**F. Core loop (slice → design → plan → execute → audit → close)** — restated in:
- `mem.pattern.doctrine.core-loop` (`memory/mem_019e9a12789f7ac39c0841f4d976503b/memory.md:12,30`).
- `mem.signpost.doctrine.lifecycle-start` (`memory/mem_019e9a11e8337613bdf8e96f75a9e6b2/memory.md:1,3,5` — longer ordering `route → slice → design → plan → phase → audit → reconcile → close`).
- `mem.signpost.doctrine.audit` (`memory/mem_019ec92b30127db0aac7eb7badb1cbf2/memory.md:9` — `route → slice → design → plan → phase-plan → execute → audit → close`).
- `mem.signpost.doctrine.overview` (`memory/mem_019e9a11b3797af3a8833c67acfa69bf/memory.md:10`).
- `install/routing-process.md:49-56` (the `**Core process:**` paragraph).
- `install/using-doctrine.md:10`.
- boot: `.doctrine/state/boot.md:400,484,598` (three phrasings).

**G. Route/routing-gate concept** — restated in:
- `mem.concept.doctrine.routing-gate` (`memory/mem_019e9a1266537c738a007fa6a10a6399/memory.md:1,3,8,20,27` — "Route before you act", "No code without an approved plan").
- `mem.pattern.doctrine.core-loop` `memory/mem_019e9a12789f7ac39c0841f4d976503b/memory.md:12,30`.
- `mem.signpost.doctrine.skill-map` `memory/mem_019e9a1200b6796094f9e31ff1666390/memory.md:7`.
- `mem.concept.doctrine.boot-snapshot` `memory/mem_019ec92b0fff76e1935b222348938d7f/memory.md:7,40`.
- `mem.signpost.doctrine.backlog` `…300a7df2…:32`; `mem.signpost.doctrine.install` `…0ffc79d294a49559db9aa12a…:15`.
- `install/routing-process.md:1-3` (the `**Route before you act.**` opener — the boot source).
- `install/doctrine.toml.example:33`.
- `install/using-doctrine.md`, `install/shipped-corpus-authoring.md`.
- `.agents/skills/route/SKILL.md:8,30`; `canon/SKILL.md`; `elicit/SKILL.md`.
- boot: `.doctrine/state/boot.md:47-49,429,497,511,594`.

---

## Thread 2 — projection and materialisation mechanics

**RustEmbed `#[folder]` roots in `src/`** (5 roots):
- `src/install.rs:20` `#[folder = "plugins/"]`
- `src/map_server/assets.rs:14` `#[folder = "web/map/dist/"]`
- `src/corpus.rs:44` `#[folder = "memory/"]`
- `src/asset_source.rs:21` `#[folder = "install/"]`
- `src/asset_source.rs:29` `#[folder = "publication/"]`

**Matching `flake.nix` `srcWithDist`** (`flake.nix:345-357`): because crane's `cleanCargoSource` (`flake.nix:343`) strips non-`.rs/.toml/.lock` assets, `srcWithDist` removes then re-grafts each embed root back:
- `flake.nix:348-349` `rm -rf`/`mkdir -p` for `plugins install memory web/map/dist templates publication`;
- `flake.nix:351` `cp -R ${./install}/.`;
- `flake.nix:352` `cp -R ${./memory}/.`;
- `flake.nix:353-354` templates, `publication`;
- `flake.nix:355` `cp -R ${webDist}/. $out/web/map/dist/`.
Note the six grafted names vs the five `#[folder]` roots: `templates/` is grafted but there is no `#[folder="templates/"]` (it backs `include_str!`/starters, not an embed root), and `plugins/` is grafted via `flake.nix`'s `plugins` list.

**`memory sync` and the derived `shipped/` tree:**
- `doctrine memory sync` materialises the embedded `memory/` corpus into gitignored `.doctrine/memory/shipped/` (`src/corpus.rs:7-24, 386`; `MEMORY_SHIPPED_DIR`, `apply` at `src/corpus.rs:398-420`).
- Ignore rule lives in **two** places: `.gitignore:46` `.doctrine/memory/shipped/` and the projection manifest `install/manifest.toml:44` (`[gitignore] entries`), with the rationale at `install/manifest.toml:32-44` (ignore contents narrowly, never blanket `.doctrine/memory/*`).
- Bounded prune (`src/corpus.rs:21-24`): a shipped dir is removed only when it parses as doctrine-owned; non-parsing children are disclosed, never silently dropped (STD-003).
- Verified present: `.doctrine/memory/shipped/<uid>/memory.{toml,md}` for the 35 uids.

**Write-if-absent vs published-on-demand** (`install/manifest.toml`):
- `[base] backings = [".gitignore","doctrine.toml","project-orientation.md"]` (`install/manifest.toml:11`) — the ONLY eagerly-projected assets, landed **write-if-absent** to `.doctrine/<key>` (rationale `install/manifest.toml:6-11`; key set asserted against `BASE_BACKINGS` at `src/install.rs:80`; test `src/install.rs:3806-3816`).
- Everything else embedded is **published on demand** via the publication manifest — reachable through `doctrine library show`/`serve`, never copied (`install/manifest.toml:6-9`; mechanics `src/publication.rs:495`, `src/commands/library.rs:82-135`). `[memory] seed_items = []` (`install/manifest.toml:24`) retires the eager orientation-memory seed (`install/manifest.toml:20-24`).
- `[root_markers]` (`install/manifest.toml:56-62`) and `[hymns] seal/expose` (`install/manifest.toml:50-53`) are the other two manifest sections.

---

## Thread 2 — hotspots

- `src/boot.rs` (8272 lines) — the single projector; any surface change (new section, new source, new Static name) lands here in `boot_sequence` (`:104-138`) and its golden tests. Highest blast radius.
- `install/routing-process.md` (86 lines) — feeds boot §Routing **and** is itself a published reference; the routing table lives here, so a skill add/rename edits it plus boot regeneration plus `skill-map` wikilinks.
- `publication/manifest.toml` — every new/changed embedded asset needs an `[[entry]]` here or reachability breaks; the duplicate `claude-activation.md` row shows curation drift is already present.
- `memory/mem_019e9a11b3797af3a8833c67acfa69bf/` (`overview`) — 82-line body restating pillars/storage/CLI/core-loop; inlined verbatim into boot §Onboarding and the session header. Its restatements (D/E/F/G) are the densest duplication node.
- `memory/mem_019e9a12b09472b3a99ac6cb071a6756/` (`cli-source-of-truth`) — the rule restated in 23 files; unifying it means editing 12 per-verb memories + skills + `install/*.md` + boot.
- `install/using-doctrine.md` (299 lines) — restates storage tiers, core process, read-via-show; also has a divergent installed copy (`.doctrine/using-doctrine.md`, 114 lines) — a live authored/projected divergence to reconcile.
- `.doctrine/governance.md` (edited, 155 lines) — user-owned layer inlined verbatim into boot §Governance; re-embed/regeneration matters because boot is gitignored runtime.
- `flake.nix:345-357` — must be kept in lockstep with `src/` `#[folder]` roots; a new embed root added in Rust but not grafted here ships asset-incomplete under Nix.

`[SL-201]`: surface census and duplication evidence for the learning-surface thread.
