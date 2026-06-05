# Design SL-013: memory skills install ergonomics + off-script skill-port record

## 1. Design Problem

Two deliverables, one tiny code surface:

1. **`--only-memory`** convenience selector for `doctrine skills install` —
   install just the memory layer (`record-memory` + `retrieve-memory`) without
   the verbose `--skill record-memory --skill retrieve-memory`.
2. **Record** the off-script skill port (item 1) and the resolved
   marketplace-symlink viability question (item 3) as durable doctrine history.

The flag must not hardcode skill ids in the CLI (scope §2). The marketplace
question was a test-then-decide; it is now decided (§6).

## 2. Current State

`doctrine skills install` (`src/skills.rs`) selects skills via
`select(all, ids, domains)` — empty filters match everything; `validate_filters`
rejects unknown ids/domains. The memory skills live in the **`doctrine`** domain
alongside every other process skill, so neither `--domain` nor any existing
filter isolates them. The only memory-only path today is the verbose `--skill`
pair, plus the `doctrine-memory` marketplace subset plugin.

The `doctrine-memory` plugin (`plugins/doctrine-memory/skills/`) enumerates the
memory subset as two **relative symlinks** into the canonical `doctrine` domain:

```
record-memory   -> ../../doctrine/skills/record-memory
retrieve-memory -> ../../doctrine/skills/retrieve-memory
```

The CLI embed (`PluginAssets`, `#[folder = "plugins/"]`) scans all of `plugins/`
and follows those symlinks, so `doctrine-memory/skills/<id>/SKILL.md` paths are
present in `PluginAssets::iter()`. They would collide on duplicate skill id, so
`discover()` skips `MARKETPLACE_ONLY_DOMAINS = ["doctrine-memory"]`
(test `discover_excludes_marketplace_only_domains` guards it).

## 3. Forces & Constraints

- **No hardcoded id list in the CLI** (scope §2): adding a third memory skill
  must not require a CLI edit.
- **No general tag/group taxonomy** (scope Non-Goals): the mechanism must be
  minimal, scoped to what `--only-memory` needs.
- **No parallel implementation** (CLAUDE.md): ride the existing
  `select`/`validate_filters`/`build_plan` path; reuse the grouping the
  `doctrine-memory` plugin already declares rather than inventing a second one.
- **Pure/imperative split** (slices-spec §Architecture): no disk/embed access in
  the pure layer — the derivation core takes its input as a path iterator.
- **Behaviour preservation**: existing `skills install` suites must stay green
  unchanged — the flag is an additive selector, no change to downstream planning.

## 4. Guiding Principles

The `doctrine-memory` plugin is the **single source of truth** for "what is the
memory subset". The flag reads that grouping back out of the embed; it does not
restate it. One selector in, the rest of the pipeline unchanged.

## 5. Proposed Design

### 5.1 System Model

`--only-memory` is sugar that resolves to the id set the `doctrine-memory`
subset domain enumerates, then feeds the **unchanged** selection pipeline:

```
--only-memory ──► resolve_install_ids() ──► skills=[…] ──► validate_filters ──► build_plan ──► select
                       │
                       └─ subset_ids(paths, "doctrine-memory")  (pure core)
```

`resolve_install_ids` is the **pure resolver**: it owns the derivation and the
empty-set bail as data-in logic, so both are unit-testable without the embed or
disk. The `--only-memory` ⊕ `--skill`/`--domain` exclusion is enforced one tier
up by clap (`conflicts_with_all`), so by the time the resolver runs the selectors
are already guaranteed disjoint.

### 5.2 Interfaces & Contracts

CLI (`src/main.rs`, `skills install` arg group):

```
--only-memory   Install only the memory skills (record-memory + retrieve-memory).
                Mutually exclusive with --skill / --domain.
```

The mutual exclusion is declared on the arg, not hand-rolled: clap
`#[arg(long, conflicts_with_all = ["skill", "domain"])]`. The conflict is
enforced at parse time (a usage error, not a runtime `bail`), and is asserted by
a clap-parse test (VT-04) rather than a code path inside `run_install`.

Pure core (`src/skills.rs`):

```rust
/// Skill ids a marketplace subset domain enumerates, read from embedded paths.
/// `<domain>/skills/<id>/…` → {id}. Pure: caller supplies the path iterator,
/// so it is unit-testable without the embed or disk.
fn subset_ids<'a>(paths: impl Iterator<Item = &'a str>, domain: &str) -> BTreeSet<String>;
```

Pure resolver (`src/skills.rs`) — owns the derivation **and** the empty-set
bail, both unit-testable on synthetic paths with no embed or disk:

```rust
/// The subset domain whose enumerated skills `--only-memory` resolves to.
const MEMORY_SUBSET_DOMAIN: &str = "doctrine-memory";

/// Effective skill-id selection for `skills install`. When `only_memory`, derive
/// the subset from `paths` and bail loud if empty (the `select([]) == all` guard,
/// D3); otherwise pass `skills` through. clap guarantees `only_memory` excludes
/// explicit `--skill`/`--domain`, so no exclusion check is needed here.
fn resolve_install_ids<'a>(
    only_memory: bool,
    skills: &[String],
    paths: impl Iterator<Item = &'a str>,
    subset_domain: &str,
) -> anyhow::Result<Vec<String>> {
    if !only_memory {
        return Ok(skills.to_vec());
    }
    let ids = subset_ids(paths, subset_domain);
    if ids.is_empty() {
        bail!("--only-memory: no skills enumerated under '{subset_domain}'");
    }
    Ok(ids.into_iter().collect())
}
```

(Illustrative — `subset_ids`/`resolve_install_ids` consume `&str` items; the
caller materialises `PluginAssets::iter()`'s `Cow<str>` first. Exact lifetime
plumbing settled at implementation.)

`run_install` gains an `only_memory: bool` parameter and becomes a thin shell —
it supplies the live embed paths and threads the result through the unchanged
pipeline. Before `validate_filters`:

```rust
let live: Vec<String> = PluginAssets::iter().map(|p| p.as_ref().to_string()).collect();
let skills = resolve_install_ids(only_memory, skills, live.iter().map(String::as_str), MEMORY_SUBSET_DOMAIN)?;
```

### 5.3 Data, State & Ownership

No new persistent state. The derived id set is computed per-invocation from the
compile/runtime embed. The `doctrine-memory` plugin owns the group definition;
the CLI only reads it. No authored data is written.

### 5.4 Lifecycle, Operations & Dynamics

Unchanged downstream: the derived ids enter `validate_filters` (so a stale or
renamed subset id surfaces as `Unknown skill '<id>'`), then `build_plan` →
`select` → `execute` exactly as an explicit `--skill` pair would. `--dry-run`,
`--global`, `--agent`, and the confirm prompt all compose for free.

### 5.5 Invariants, Assumptions & Edge Cases

- **Empty-set guard (critical).** `select(ids=[])` means *all*. If the subset
  domain ever vanished, an unguarded empty derivation would silently install the
  **entire** catalog under a `--only-memory` flag. `resolve_install_ids` bails on
  empty — the flag fails loud, never installs more than asked. The bail lives in
  the **pure resolver** (not the impure shell), so VT-01 drives it on synthetic
  paths.
- **Cross-domain identity (the silent pact).** The derivation reads ids from the
  discover-**excluded** `doctrine-memory` paths, yet validation runs against the
  catalog where those ids reappear under the **`doctrine`** domain (the symlink
  targets). The mechanism therefore depends on a standing invariant: *every
  subset symlink basename equals the id of a real skill in an installable
  domain.* That identity is what carries derived ids through `validate_filters`;
  break it (a symlink renamed off its canonical) and the next bullet catches it.
- **Mutual exclusion.** `--only-memory` with `--skill`/`--domain` errors rather
  than unioning — combining a "just these" sugar with explicit selectors is
  ambiguous. Enforced declaratively by clap `conflicts_with_all` at parse time.
- **Derived ids are validated.** They pass through `validate_filters`, so a
  symlink renamed out from under the subset domain is caught, not silently
  dropped.
- **Embed follows the symlinks.** Assumed (and relied on already by the
  `MARKETPLACE_ONLY_DOMAINS` collision guard): `PluginAssets::iter()` yields
  `doctrine-memory/skills/<id>/…`. A VT test asserts the live derivation equals
  `{record-memory, retrieve-memory}`, pinning the assumption.

## 6. Open Questions & Unknowns

**RESOLVED — marketplace-symlink viability (scope item 3).** Confirmed via
Claude Code plugin docs + install behaviour: installing `doctrine-memory` from
the marketplace clones/copies the **whole** marketplace repo locally, so the
sibling `plugins/doctrine/` subtree is present and the relative symlinks resolve
(dereferenced/copied into the plugin cache at install). The subset plugin and
the `MARKETPLACE_ONLY_DOMAINS` guard **stand** — no code deletion. Residual
confidence on the exact deref mechanics is soft, so the design treats "proven"
as a **manual install-smoke** (VT-05), not an automated claim.

No open questions remain.

## 7. Decisions, Rationale & Alternatives

- **D1 — Derive the group from the `doctrine-memory` plugin (chosen).** DRY: the
  subset plugin already declares exactly this group; reading it back out adds no
  second source. A third memory skill joins by symlink, no CLI edit.
  - *Alt A — hardcode `["record-memory","retrieve-memory"]`.* Simplest (~30 LOC)
    but the scope explicitly cautions against an id list in the CLI; rejected.
  - *Alt C — frontmatter `tags:` + `--tag` filter.* A general primitive touching
    `Meta`/`Entry`/`select`/two `SKILL.md`s (~80+ LOC); crosses the
    "no general taxonomy" Non-Goal; rejected.
- **D2 — Mutually exclusive with `--skill`/`--domain` (error).** Clearer than
  union; sugar + explicit selectors together has no obvious right semantics.
  Enforced declaratively by clap `conflicts_with_all` (parse-time usage error),
  not a hand-rolled runtime check — no parallel implementation.
- **D3 — Empty-derivation bails.** Guards the `select([]) == all` footgun. The
  bail lives in the **pure resolver** `resolve_install_ids`, not the impure shell,
  so it is reachable by a unit test (VT-01) on synthetic-empty input.
- **D4 — Keep the marketplace subset plugin** (per §6).

## 8. Risks & Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| `select([]) == all` installs everything under `--only-memory` | high | D3 empty-set guard: `resolve_install_ids` bails loud. The bail lives in the **pure resolver**, so VT-01 fires it on synthetic-empty paths — proven, not merely asserted. |
| Embed stops following the subset symlinks (toolchain change) | low | VT-02 live-derivation test fails fast, pinning the assumption |
| Marketplace deref behaviour differs from docs in some CC version | low | VT-05 manual install-smoke; subset is additive — worst case users fall back to `--only-memory` |
| Coupling flag → `doctrine-memory` plugin existence | low | Accepted & documented (D1/D4); the plugin is confirmed staying |

## 9. Quality Engineering & Validation

TDD, red/green/refactor. Tests:

- **VT-01 (pure)** `subset_ids` over synthetic paths → correct id set; an
  empty/absent domain → `{}`. **AND** `resolve_install_ids(only_memory=true, …,
  synthetic-empty paths, …)` **bails** — the D3 guard fires in the pure layer, no
  embed or disk.
- **VT-02 (live)** `resolve_install_ids(only_memory=true, &[], PluginAssets::iter(),
  MEMORY_SUBSET_DOMAIN)` against the real embed → `{record-memory,
  retrieve-memory}`; pins the embed-follows-symlinks assumption.
- **VT-03 (pure integration)** the derived ids fed through `validate_filters` +
  `build_plan`/`select` against the catalog select **exactly** those two skills —
  no more, no less. Drives the cross-domain identity invariant (§5.5) without the
  IO-bound `run_install`.
- **VT-04 (clap parse)** `--only-memory --skill X` (or `--domain Y`) → clap
  conflict/usage error at parse time (asserts `conflicts_with_all`).
- **VT-05 (manual)** marketplace install-smoke: install `doctrine-memory` from
  the marketplace, confirm both memory skills resolve. Recorded in `audit.md`,
  not automated.
- **VT-06 (record, deliverable 2)** the off-script skill-port history (scope
  item 1) is **already discharged** by the `slice-013.md` Context + Scope
  sections and this design's history; no further authoring is owed. `/close`
  attests the record exists rather than producing it — closure must not pass
  with deliverable 2 silently unaddressed.
- Behaviour-preservation: existing `skills install` suites stay green unchanged.

`cargo clippy` zero warnings (bins/lib, no `--all-targets`); `just check` before
each commit.

## 10. Review Notes

### Adversarial pass (pre-lock)

- **R1 (critical, resolved) — embed-follows-symlinks was unproven.** The whole of
  mechanism B (§5.1) rests on `PluginAssets::iter()` descending the
  `doctrine-memory/skills/*` symlinks. The existing
  `discover_excludes_marketplace_only_domains` test does **not** prove this — its
  "no doctrine-memory domain" assertion passes vacuously if the embed never
  included those paths. `walkdir` defaults to `follow_links=false`, so this was a
  real failure mode: empty derivation → D3 bail → `--only-memory` permanently
  broken. **Verified empirically** (throwaway probe, since removed): rust-embed 8
  with `debug-embed` *does* embed
  `doctrine-memory/skills/{record-memory,retrieve-memory}/SKILL.md`. Assumption
  holds; VT-02 now pins it as a standing regression guard. The probe also
  confirmed the plugin's `README.md`/`.claude-plugin/plugin.json` are embedded
  but correctly excluded by the `<domain>/skills/<id>/…` match in `subset_ids`.
- **R2 — mutual-exclusion scope.** Confirmed the exclusion is only vs
  `--skill`/`--domain`; `--only-memory` composes freely with
  `--agent`/`--global`/`--dry-run`/`-y`. No over-broad rejection.
- **R3 — derived-id validation.** Confirmed derived ids still flow through
  `validate_filters`, so a renamed canonical skill surfaces as `Unknown skill`
  rather than silently shrinking the install set.

### Inquisition pass (formal, 2026-06-06)

Hostile pass on the locked design — full findings in `inquisition.md`. Verdict:
1 MAJOR, 4 MINOR, all accepted; penances integrated above. Summary:

- **I (MAJOR) — testability of the safety guard.** The D3 empty-bail, D2
  exclusion, and the integration assertion were stranded in IO-bound
  `run_install`; §8 claimed "covered via the pure core" when the bail was
  actually impure and unreachable by test. **Fixed:** extracted the pure
  `resolve_install_ids` (derive + empty-bail), moved D2 to clap
  `conflicts_with_all`, re-pointed VT-01 (bail), VT-03 (pure derive→select), and
  VT-04 (clap parse) at testable seams (§5.2, §8, §9).
- **II (MINOR)** — deliverable 2 (off-script port record) was declared then never
  given an acceptance surface. **Fixed:** VT-06 attests it is discharged by the
  scope doc; `/close` verifies, does not re-author.
- **III (MINOR)** — VT-id collision (manual smoke called both VT-03 and VT-05).
  **Fixed:** §9 authoritative, manual smoke = VT-05; §6/§8 corrected.
- **IV (MINOR)** — the cross-domain identity invariant (subset symlink basename ==
  canonical id in an installable domain) was unstated. **Fixed:** named in §5.5.
- **V (MINOR)** — hand-rolled exclusion duplicated clap. **Fixed:** clap
  `conflicts_with_all` (§5.2, D2).

No unresolved findings. Design re-locked.
