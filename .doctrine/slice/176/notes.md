# Notes SL-176: Finish Axis B — slices/drift retirement

Durable per-slice scratchpad — tracked in git.

## Onboarding for design / review / plan agents

**Read first:** RFC-003 § "Finish Axis B" (`doctrine rfc show RFC-003`) — the
decision-of-record. Also its § "Status" (axis dispositions, what shipped), § "Layer 1"
(graph-effect is consumer policy), § "Layer 2" (derivable-not-relational law), and §
"The deciding principle — label vs role". This slice is the *implementation* of an
already-decided direction; design resolves the **mechanism**, not the *what*.

### Locked decisions (do NOT relitigate — RFC-003, 2026-06-29)

- Provenance = **one neutral `references(originates_from)` role** (not sub-roles).
  Subsumes shipped `scoped_from`; absorbs IMP-207's `spawned_from`. Any work → its origin.
- Fulfillment+completion = **`fulfils` label** + **`{full, partial}` degree facet**
  (facet, not role; not a coverage substrate).
- Old `slices` "addressed by" = **derived inbound of `fulfils`** (ADR-004), not stored.
- Cascade is **hint-not-auto** (F-6); degree does **not** aggregate (2 partials ≠ full).
- Governance ratification **deferred to reconciliation** — no ADR edits in design/plan.

### Ground truth — code anchors (verified this session, ~2026-06-29)

> Verify against a **fresh dev build** before trusting CLI behaviour — the jail's
> installed `~/.cargo/bin/doctrine` can lag source. (It *does* currently support
> `link --role`, confirmed.) See `mem.pattern.relation.relate-via-link-not-hand-authored-rows`.

- **`RELATION_RULES`** table: `src/relation.rs:335–615`. The single source of truth;
  `SPEC-018` points at it, never transcribes (storage rule).
- **`Role` enum** (`src/relation.rs:197`): `{Implements, ScopedFrom, Concerns}` — wire
  names + `from_str` at `:217–232`. `originates_from` work edits this enum (rename/add).
- **`scoped_from` rule row**: `src/relation.rs:355–366` — `sources:&[SL]`, target
  `Kinds(BACKLOG)`, `Tier::One`, `Writable`. The row `originates_from` generalizes.
- **`slices` rule row**: `src/relation.rs:500–508` — `sources: BACKLOG`, target
  `Kinds(&[SL])`, `Tier::One`, `Writable`, inbound `"slices"`. The label to **retire**.
- **`drift` rule row**: `src/relation.rs:547–555` — `sources: BACKLOG`, target
  `Unvalidated` (free-text), `Tier::One`. Entity rows fold; non-entity rows stay (deferred).
- **`Tier` / `LinkPolicy` / `TargetSpec`** defs: `src/relation.rs:248–325`. `Tier::One` =
  uniform `[[relation]]`; `Tier::Typed` = bespoke payload (e.g. `interactions`,
  `members`). **`fulfils`+degree likely needs a payload-bearing shape — study
  `interactions` (`:438–446`, free-text `type` re-read at render) as the facet precedent.**
- **`RelationRule` struct**: `src/relation.rs:304–325` (the 6 axes incl. role column +
  `inbound_name`). The table is declared in `RelationLabel` enum-discriminant order
  (lockstep golden VT-1) — new variants land at the source kind's axis-run tail.
- **Reader / inbound derivation**: `src/relation_graph.rs` (`outbound_for`, `in_edges`).
  The `slices`→`fulfils`-derived-inbound flip lives here.
- **CLI seam**: `src/commands/relation.rs` (`link`/`unlink`/`inspect`); `link --role`
  exists (SL-149) — the `--degree` axis is the new flag.

### SL-149 is the migration template

SL-149 ("References role grammar", done 2026-06-24) did the *exact* prior move:
collapsed `specs`/`requirements` → `references(role)`, re-keyed the target gate from
`(source,label)` to `(source,label,role)`, and migrated ~185 edges. **Read SL-149's
design + plan + migration phase** before designing this slice's migration — same
machinery, same lockstep-golden discipline, same "retain old rows through PHASE-N then
drop" cadence. `doctrine slice show SL-149`; `doctrine slice paths SL-149`.

### Census (live ~2026-06-29, from `doctrine relation census`)

`slices` 82 · `references(scoped_from)` 19 · `drift` 7 (2 resolved / 5 free-text) ·
`references(implements)` 95 · `references(concerns)` 108 · `related` 130. The retcon set
is IMP-207's 19 named rows (some currently counted under `slices`).

### Open design questions (for /design to resolve)

1. `fulfils` storage tier + write seam — tier-1+degree-column vs tier-2 typed payload;
   `LinkPolicy` Writable+`--degree` vs typed verb. (The facet threading is the novel work.)
2. `scoped_from` → `originates_from`: rename-in-place vs add-and-migrate.
3. `fulfils` inbound rendering — degree-aware ("fulfilled by (partial)")?
4. Author-at-the-mutable-end: enforced invariant or convention?
5. Close-cascade hint (`doctor`/`/close` reading `fulfils(full)`) — in-scope minimal hint
   or spun out? Hint-not-auto either way.
6. `drift` entity-row reclassification — exact rows, and the "feeds into" → dep/seq move.

### Gotchas

- **No parallel implementation** — ride the existing relation seam; behaviour-preservation
  gate (entity-engine suites stay green unchanged).
- **Immutable ids** — `PHASE-NN` / `EN-/EX-/VT-` append-only.
- **Relations via `doctrine link`**, never hand-authored `[[relation]]` rows.
- Tangential: `src/supersede.rs` has a known RECORD-supersede tier drift (IMP-095, one of
  the 19 retcon rows) — *not* this slice's concern, but IMP-095's edge gets relabelled here.

---

## Design session — COMPLETE, locked (2026-06-29)

`design.md` written + adversarially reviewed (internal SR-1..12 + external codex F1..6),
all integrated. Six open questions resolved (see design.md decision ledger). **Lifecycle
still `design`** — NOT moved to `plan` yet; user wants a **second codex pass** on the
revised design first (the F3/F4 fixes added new mechanism).

**Commits:** `96fa4edf` (design + scope + selectors + IMP-210/IMP-156 follow-ups) ·
`27bd3321` (codex F1–F6 integration). `.doctrine` committed promptly. No code touched →
`doctrine check gate` N/A (design only).

**Resolutions (one-liners; design.md is authority):** Q1 tier-1 + `degree:Option<Degree>`
column · Q2 rename `ScopedFrom`→`OriginatesFrom` in place, parallel-naming with REV label
accepted · Q3 degree-aware inbound (per-target) · Q4 author-end = convention + source-set
fence · Q5 cascade spun out → IMP-210 · Q6 drift mapping named, re-census at exec.

**Codex grew the scope (verified in source — durable, memory-worthy at execution):**
- **F1** `append_relation_row` is append-or-`Noop` (`relation.rs:911/949`) — **no mutation/
  upsert path**. Degree set at author; change = `unlink`+`relink`; conflicting-degree relink
  errors. F2 → `validate` uniqueness invariant on `(source,fulfils,target)`.
- **F3** `RelationGroup = (RelationKey, Vec<String>)` (`relation_graph.rs:521`) flattens
  targets to bare strings — per-target degree needs a **data-model change**
  (`RelationTargetView{target,degree}`), not a render-side index. Touches all inspect
  render/JSON paths.
- **F4 (the consequential one)** retiring `slices` is **NOT vocab-table-local**: live
  consumers are **`src/priority/graph.rs:190/201`** (backlog optionality scoring — reference
  + consequence label) and **`src/backlog.rs`** show/JSON/lifecycle-findings. Re-point all at
  `fulfils`, scoring numbers held identical. *Reusable gotcha for any future relation-label
  retirement: grep priority/graph + backlog show/json/lifecycle, not just `RELATION_RULES`.*
- **F5** `scoped_from` hardcoded in `slice.rs:1677/1758`, `backlog.rs:1428/1447/1581`,
  `cli.rs:552` (show/JSON/help) — public field renames.

**Selectors:** 14 `design-target` recorded (incl. the F4/F5 additions
`src/priority/graph.rs`, `src/commands/cli.rs`).

**Next:** second codex pass (fresh agent) → if clean, `doctrine slice status 176 plan` +
`/plan`. If the priority re-point (R10) or `RelationTargetView` change (R11) draws blood,
back to design.

## Second codex pass — DONE, verdict RETURN-TO-DESIGN (2026-06-29)

Fresh independent codex thread, hostile read-only. Four findings, all verified in source,
all integrated into design.md (new "external pass 2" subsection + section callouts G1–G4).
Lifecycle **stays `design`** — NOT moved to plan.

- **G1 [BLOCKER, open]** the §A′.1 backlog re-point is **not a label swap** — it needs an
  inbound read-path `backlog show` structurally lacks. `format_show`/`format_json` are
  pure-on-own-tier1, inbound deferred to the registry surface by ADR-004
  (`backlog.rs:1363-1365`); reads at `:1420`/`:1574`/`:2201` read the item's *own outbound*
  `slices`, which migration deletes. `fulfilled by` = derived inbound + an ADR-004 posture
  reversal. **Resolution required at design:** (a) show/json gain scan-derived inbound, or
  (b) drop the line, defer to `inspect`. `doctor` (`:2201`) is tractable (already scans);
  show/json are the open call. *Drives the verdict.*
- **G2 [MAJOR, open]** validate uniqueness invariant has no home — `validate_relations`
  (`relation_graph.rs:341+`) reports only danglers/corruption, `CatalogEdge` carries no
  degree. Needs a named finding class + seam choice.
- **G3 [MAJOR, flag]** `originates_from` source/target widening flips shipped rule-contract
  tests (`relation.rs:2696`/`:2743`/VT-2 `:1457`) — deliberate *content*, mis-framed under
  "machinery green unchanged." Re-classed; no mechanism breakage.
- **G4 [MINOR]** unknown-role diagnostic string `commands/relation.rs:42-47` hardcodes
  `scoped_from` (distinct from `cli.rs:552` clap help). Census add.

**Reusable gotcha (memory-worthy at execution):** retiring/renaming a relation label is
NEVER vocab-table-local. Beyond `priority/graph.rs` + `backlog.rs` show/json/lifecycle
(F4), grep also **`lazyspec.rs` `map_edge`** (web-graph), **`commands/relation.rs`**
(runtime role-parse diagnostics, distinct from clap help), and any surface that reads the
*item's own outbound* for an edge that is migrating to the *other* endpoint — those silently
need an inbound-derivation read-path, not a swap (G1).

## Design session — G1–G4 resolved (2026-06-29)

- **G1 → (a), user-locked** (D-backlog-inbound): `backlog show`/`json` gain the
  `inspect`-style **derived-inbound** read-path (`in_edges`+`inbound_role_index`, threaded
  with `root`); `slices` outbound read **removed**, not swapped; `fulfilled by: SL (degree)`
  from derived inbound. **ADR-004-consistent** (inbound always derived; ADR defers the
  reverse *field*, not a derived render). `backlog show` becomes **corpus-aware** —
  deliberate refinement of the `:1363-65` item-local posture. `doctor` (`:2201`) same set.
- **G2 → D-uniqueness-seam**: identity's `source` is one entity → duplicate logical edge is
  two `fulfils` rows in **one toml** → enforce **locally at `read_block`** (new
  `DuplicateEdge` finding, degree-agnostic `(label,role,target)` match). NOT corpus
  `validate_relations`; no degree thread into `CatalogEdge`. Write-seam degree-conflict error
  = author-time guard; `read_block` = at-rest backstop.
- **G3 → content**: widening flips `relation.rs:2696`/`:2743`/VT-2 `:1457` — deliberate
  rule-contract content, enumerated at plan (§C). No mechanism breakage.
- **G4 → §A′.2** row: `commands/relation.rs:42-47` role-error string.

design.md fully reconciled (ledger rows D-backlog-inbound + D-uniqueness-seam added; §A.5,
§A.6, §A′.1, §A′.2, §C, P2/P3 phasing, Status, both review subsections all consistent).

**Next:** OPTIONAL one confirming third external pass on the G1(a) backlog-inbound mechanism
(it is the one genuinely new read-path), else **lock → `slice status 176 plan` → `/plan`**.
User to choose third-pass vs lock. Commits: `96fa4edf` → `27bd3321` → `20e86c8d` →
`560d4a1a` (pass-2) → (this G1–G4 resolution).

## Design locked → plan; HALTED on a priority-scoring decision (2026-06-29)

User chose **lock → `/plan`** (no third pass). Lifecycle moved `design`→`plan`; 4-phase
`plan.toml` + `plan.md` authored and committed (engine / storage / surfaces / migration;
governance ratification deferred to reconcile, not a plan phase).

**Plan finalisation HALTED.** The critical pass (plan skill step 7) grounded design **R10**
in source and found it **false**: the `Slices`→`Fulfils` priority re-point does NOT preserve
optionality numbers — the edge direction flips (`slices` item→SL vs `fulfils` SL→item), so
the credited node flips. Full context + the user's decisive guidance + the resolution
direction + the one remaining open scope question are dumped in
**`decision-priority-optionality.md`** — the fresh agent resolves that BEFORE finalising the
plan. Resolution direction is settled (credit the item from the slice's facets, degree-
weighted; delete R10's preservation claim); open question is whether degree-weighting is
in-scope here or a follow-up.
