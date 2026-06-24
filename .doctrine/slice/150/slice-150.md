# Family-grouped help + boot-map projection

## Context

`doctrine --help` is a flat, unsorted list of ~60 top-level commands — opaque to
humans and offering no routing structure to agents. `doctrine --help --commands`
(IMP-166) adds full per-kind verb tables, but is too heavy (~150 lines) to load
on every agent boot, and most of its bulk is the repeated CRUD spine
(`new/list/show/paths/status`) — high tokens, near-zero marginal information.

Agent onboarding to the command surface must live in the CLI (generated from the
clap command tree) so it cannot drift from the real commands. This is a
tokens-vs-breadth/clarity exercise: the same command tree must serve a scannable
human reference and a dense, routing-grade boot map.

Origin: design conversation following closed IMP-166. Adjacent open item IMP-135
(CLI help consistency pass) may be partially subsumed — coordinate, do not fold.

## Scope & Objectives

1. **Command taxonomy** — classify every top-level command into one of 8
   families: `change`, `governance`, `knowledge`, `relations`, `facets`,
   `reports`, `explore`, `infra`. The classification is a single source the two
   renderings share.

2. **Factored CRUD spine** — declare the shared entity-kind spine
   (`new/list/show/paths`, `+status` where lifecycle applies) once; surface only
   the *distinctive*, semantic-bearing verbs per kind (e.g. `slice design/plan/
   phases`, `review raise/dispose/contest`, `revision change/approve/apply`).

3. **Human rendering** — regroup `doctrine --help` into the 8 families as a
   scannable table with one-line glosses (not token-budgeted).

4. **Boot-map rendering** — a dense projection (families + distinctive verbs,
   spine stated once, no glosses; ~20 lines) suitable for inlining into the
   governance snapshot via `doctrine boot`.

5. **Drift-guard test** — assert every clap top-level command classifies into
   exactly one family; fail when a newly added command is unclassified.

Both renderings walk the *same* clap command tree → cannot drift from reality.

## Non-Goals

- Renaming, adding, or removing any command or verb.
- Changing `--help --commands` output shape (IMP-166 stays as the lazy full
  reference; only its boot-loading role is displaced by the boot map).
- Reworking per-subcommand (`doctrine <cmd> --help`) clap help.
- A general help-text consistency/copy pass (that is IMP-135's lane).
- Deciding whether/how the boot map actually lands in the snapshot body vs a new
  section heading — design will settle the boot wiring, but no change to the
  routing table or other snapshot sections.

## Affected Surface

- `src/commands/cli.rs` — `render_top_level_help` (~597), `render_commands_table`
  (~672), `HelpEntry`/`VerbEntry` structs; home of the taxonomy + both
  renderings.
- `src/boot.rs` — `boot_sequence`/`build_sections` (~98/317); boot-map as a
  generated section inlined like "Accepted ADRs".
- `src/main.rs` — top-level help intercept (~185); touched only if the boot map
  gets an interactive flag.

## Risks / Assumptions / Open Questions

- **R1** Boot snapshot is a governance contract every agent loads; the
  byte-stable / `boot --check` invariants (cf. IMP-123) must be honoured.
- **A1** The clap command tree is enumerable at runtime for the drift test
  (IMP-166 already walks it for `--commands`, so the seam exists).
- **OQ1** *(resolved)* Boot-map granularity = families **+ distinctive verbs**
  (auto-derived `verbs − spine`); infra verb-expansion suppressed (D7).
- **OQ2** *(resolved)* Family taxonomy settled — all 44 commands partitioned
  across 8 families (`reports` not `views`; `explore` holds search/inspect/
  relation/concept-map/map; `catalog`→infra). See design §5.2 `FAMILIES`.
- **OQ3** *(resolved)* Boot map gets BOTH an interactive entry point
  (`doctrine --help --map`) and a `SourceKind::CommandMap` snapshot section,
  one `render_boot_map()` behind both (D3).
- **OQ4** *(open — design §6 OQ-3)* `--map` flag name overloads the `map`
  command. User decision before execute.
- **OQ5** *(open — design §6 OQ-1)* D8 shared-width grouped help: plain-text
  render vs extend `listing`. Settles at execute phase 1.

## Verification / Closure Intent

- Drift-guard test green: total clap commands == sum over families, no command
  in two families, no command unclassified.
- `doctrine --help` renders the 8 families with glosses; spine stated once.
- Boot map renders dense (~20 lines), inlines cleanly, `boot --check` clean.
- `just gate` green; behaviour-preservation of existing help/boot suites.

## Follow-Ups

- Revisit IMP-135 scope once this lands (may be partially closed by it).
