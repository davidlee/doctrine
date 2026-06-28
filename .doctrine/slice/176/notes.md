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
