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
--only-memory ──► memory_subset_ids() ──► skills=[…] ──► validate_filters ──► build_plan ──► select
                       │
                       └─ subset_ids(PluginAssets::iter(), "doctrine-memory")  (pure core)
```

### 5.2 Interfaces & Contracts

CLI (`src/main.rs`, `skills install` arg group):

```
--only-memory   Install only the memory skills (record-memory + retrieve-memory).
                Mutually exclusive with --skill / --domain.
```

Pure core (`src/skills.rs`):

```rust
/// Skill ids a marketplace subset domain enumerates, read from embedded paths.
/// `<domain>/skills/<id>/…` → {id}. Pure: caller supplies the path iterator,
/// so it is unit-testable without the embed or disk.
fn subset_ids<'a>(paths: impl Iterator<Item = &'a str>, domain: &str) -> BTreeSet<String>;
```

Impure wrapper (`src/skills.rs`):

```rust
/// The subset domain whose enumerated skills `--only-memory` resolves to.
const MEMORY_SUBSET_DOMAIN: &str = "doctrine-memory";

fn memory_subset_ids() -> anyhow::Result<Vec<String>> {
    let paths: Vec<String> = PluginAssets::iter().map(|p| p.as_ref().to_string()).collect();
    let ids = subset_ids(paths.iter().map(String::as_str), MEMORY_SUBSET_DOMAIN);
    if ids.is_empty() {
        bail!("--only-memory: no skills enumerated under '{MEMORY_SUBSET_DOMAIN}'");
    }
    Ok(ids.into_iter().collect())
}
```

(Illustrative — `subset_ids` consumes `&str` items; the wrapper materialises
`PluginAssets::iter()`'s `Cow<str>` first. Exact lifetime plumbing settled at
implementation.)

`run_install` gains an `only_memory: bool` parameter. Before `validate_filters`:

```rust
let skills: Vec<String> = if only_memory {
    if !skills.is_empty() || !domains.is_empty() {
        bail!("--only-memory cannot be combined with --skill or --domain");
    }
    memory_subset_ids()?
} else {
    skills.to_vec()
};
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
  **entire** catalog under a `--only-memory` flag. `memory_subset_ids` bails on
  empty — the flag fails loud, never installs more than asked.
- **Mutual exclusion.** `--only-memory` with `--skill`/`--domain` errors rather
  than unioning — combining a "just these" sugar with explicit selectors is
  ambiguous.
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
as a **manual install-smoke** (VT-03), not an automated claim.

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
- **D3 — Empty-derivation bails.** Guards the `select([]) == all` footgun.
- **D4 — Keep the marketplace subset plugin** (per §6).

## 8. Risks & Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| `select([]) == all` installs everything under `--only-memory` | high | D3 empty-set guard bails loud (VT covers it via the pure core) |
| Embed stops following the subset symlinks (toolchain change) | low | VT-02 live-derivation test fails fast, pinning the assumption |
| Marketplace deref behaviour differs from docs in some CC version | low | VT-03 manual install-smoke; subset is additive — worst case users fall back to `--only-memory` |
| Coupling flag → `doctrine-memory` plugin existence | low | Accepted & documented (D1/D4); the plugin is confirmed staying |

## 9. Quality Engineering & Validation

TDD, red/green/refactor. Tests:

- **VT-01 (pure)** `subset_ids` over synthetic paths → correct id set; an
  empty/absent domain → `{}` (drives the D3 guard).
- **VT-02 (live)** `memory_subset_ids()` against the real embed →
  `{record-memory, retrieve-memory}`; pins the embed-follows-symlinks assumption.
- **VT-03 (integration)** `run_install`/`build_plan` with `only_memory=true`
  selects **exactly** those two skills — no more, no less.
- **VT-04** `--only-memory` + `--skill` (or `--domain`) → error.
- **VT-05 (manual)** marketplace install-smoke: install `doctrine-memory` from
  the marketplace, confirm both memory skills resolve. Recorded in `audit.md`,
  not automated.
- Behaviour-preservation: existing `skills install` suites stay green unchanged.

`cargo clippy` zero warnings (bins/lib, no `--all-targets`); `just check` before
each commit.

## 10. Review Notes

(Adversarial pass appended below before locking.)
