# SL-229 — Pre-design research stage v1 — Design

> A persisted research artefact that exists *before* design.md, produced by a
> pre-design round of two read-only research threads, governed by one new
> unrouted skill (`/research`), stamped with a `ContentSet` staleness baseline
> by one new idempotent CLI verb (`doctrine slice research <id>`), and consumed
> via short advisory hooks in `/slice` `/design` `/plan` `/phase-plan`.
> Grounded in the dogfood research round run on this slice itself:
> `research/research.md` (✓-verified claims; citations below trace to it).

## Problem

RFC-011 shows tokens bleed in orientation: inquisition finds design assumptions
not grounded in code, plan selectors mis-represent the touch-set, plan
assumptions fail at implementation (IDE-044 § Problem). The missing piece is an
evidence artefact design/plan can cite instead of recall.

## Decisions

- **D1 — Storage: `.doctrine/slice/NNN/research/` direct**, riding the
  pre-existing SL-055 convention. `.gitignore:48`, `Tier::Research`
  (`src/worktree/allowlist.rs:69`), and the `doctor_checks.rs:334` prose-cite
  skip all pre-exist; zero new machinery. Tier impurity is precedented
  (`handover.md`, `inquisition.md`). Gitignored ⇒ per-worktree ⇒ no dispatch
  split-brain; harvest at close is explicit. **Supersedes** the scope's
  original state-tier+symlink wording (the dogfood round surfaced the existing
  convention and empirically showed the symlink route needs a gitignore fix —
  research.md § Cross-thread finding).
- **D2 — Verb: one idempotent subverb** `doctrine slice research <id>`
  (SPEC-013 grammar; extends `SliceCommand`, `src/slice.rs:158`). Absent →
  mint + stamp; present → print drift advisory; `--restamp` → re-baseline.
  No `paths`/`show` surfaces in v1 (artefact sits in the slice folder;
  `slice paths` finds it).
- **D3 — One artefact doc** (`research.md`) with mandated thread sections;
  cross-thread findings were the highest-value dogfood output and need a
  single home. Researchers return over stdout and never write; the
  orchestrating agent assembles regardless, so per-thread docs buy nothing.
- **D4 — Staleness advisory surfaces in the skill flow only** (v1). No
  `status`/`reports next` wiring; trivial to add later once the engine fn
  exists, judged by RFC-011 evals (scope R1).
- **D5 — Raw thread output is piped to `research/raw/<thread>.md`**, never
  inlined into tool results. Files are selectively readable and greppable
  during verification and refresh rounds; the curated doc distills and points
  at raw for bulk (dogfood: 92-line governance raw → 5-line n/a list +
  pointer).
- **D6 — Advisory, never a gate** (ADR-003): hooks use "run/consult"
  phrasing; the verb always exits 0 on drift. Harder gating would need an
  ADR, not a skill edit.

## Artefact contract (owned by the `/research` skill)

`plugins/doctrine/skills/research/SKILL.md` — doctrine domain, unrouted (no
routing row, no boot change). Frontmatter `name: research` = dir name
(SPEC-010); `description` written as the real trigger.

The skill body owns:

1. **Shape.** `research.md` skeleton: header (producers, baseline pointer);
   verification legend; *Thread 1 — governance applicability* (binding
   constraints / checked-not-applicable **with stated reasons** / revision
   candidates); *Thread 2 — code map* (hotspots, cited facts, naming
   precedents); *Cross-thread findings*; *Design-input deltas*. Extra sections
   free-form. Refresh rounds revise sections in place.
2. **Citation forms.** Durable ids for governance (STD-002); `file:line` for
   every code claim. An uncited claim is unverifiable by definition.
3. **Verification discipline.** ✓ = claim verified by the *consuming* agent
   (grep/read of the cited site); unmarked = researcher claim. Design/plan may
   only load-bear ✓ rows or rows verified at point of use. Verification is
   asymmetric — verify what you lean on (~1 grep/claim observed).
4. **Runner deferral + raw capture.** "Spawn the project's research agents,
   one per thread, read-only, stdout → `research/raw/<thread>.md`." What a
   research agent *is* lives in project governance (governance § research /
   CLAUDE.md), never in doctrine (POL-002; no executable seam). **Graceful
   degradation:** project defines no research agents → the orchestrating
   agent runs the threads itself, or skips and says so in the artefact
   header. Researchers never write files or memories (memory distillation =
   harvest-time, later slice).
5. **Prompt duties.** Thread prompts must demand: the citation forms, the
   structured not-applicable form, output in the artefact's section shape,
   no preamble (known cheap-model failure; assembler strips).

ADR-005 restate line: the skill names the verb and when to run it; flag syntax
and baseline format belong to `--help` and code. The artefact skeleton is
*owned* here, so spelling it out is definition, not restatement.

## CLI verb + engine

**Command surface** (pattern: coverage wiring, research.md § coverage-verb):

- `SliceCommand::Research { id, restamp }` (`src/slice.rs`), dispatch via the
  existing `Command::Slice` arm (`cli.rs:1427`).
- `guard.rs:74+`: `Write("slice research")` arm (bare invocation may create).
- Behaviour: dir absent → `create_dir_all` + stamp `baseline.toml`; present →
  compute current set, `baseline.diff(&current)`, print fresh-or-drift
  advisory (per-path changed/added/removed); `--restamp` → overwrite baseline.
  Always exit 0 (D6).

**Baseline** (`research/baseline.toml`): `slice`, `date`, `[hashes]` — the
serialize→recompute→diff pattern from the review warm-cache
(`review.rs:2500`, `2540-2542`). Fixed path set: `slice-NNN.md`, `design.md`,
`plan.md`, `plan.toml` under `.doctrine/slice/NNN/` (repo-relative keys;
plan.toml added at plan time — criterion edits are frequently TOML-only and
phase-plan refresh is exactly the consumer that cares).
Absence-is-defined does the lifecycle work: pre-design mint records only the
scope doc; design.md appearing later = `added` drift.

**Engine:** new leaf `src/research.rs` (~100 lines; ADR-001: leaf, no command
imports): `RESEARCH_DIR`/`BASELINE_FILE` constants (STD-001),
`research_dir(root, id)`, `mint(root, id, date)`, `check(root, id) ->
SetDrift`. Pure over passed-in date (date/uid pattern); fs at the thin edges
mirroring `contentset::compute`. `contentset::is_stale_against`: consume it or
**remove** it (`diff` already carries per-path detail the advisory needs) —
resolved at implementation; the `dead_code` suppression
(`contentset.rs:114-118`) does not survive this slice either way.

## Consumption hooks

One-to-three-line advisory edits (D6 phrasing), each pointer + consumption
note:

- `/slice` (Next): before `/design`, run the pre-design round per `/research`.
- `/design` (Explore context): `doctrine slice research <id>`; artefact absent
  → run the round first. Thread 1 stands in for the bulk of the `/canon`
  sweep. Assertions cite `research.md`; load-bear ✓ rows only.
- `/plan`: check the advisory; draft selectors from the Thread-2 hotspot map;
  the design-time selector dry-run remains the checking half. (Rationale
  cites SL-180; **shipped hook text stays project-neutral** — POL-002.)
- `/phase-plan`: check the advisory; on drift refresh only affected thread
  sections, then `--restamp`.

`/research` carries the inverse pointers (who invokes, when) — single-sourced.

## Install-side touches

- `install/governance.md`: stub *§ Research agents* — the socket the skill
  points at (commands, models, expectations; project-defined).
- `install/glossary.md`: one slice-dir layout line for `research/`
  (`research.md`, `raw/`, `baseline.toml`).
- No manifest change (SPEC-009); no routing/boot change (SPEC-011).

## Code impact summary (design-target)

| Path | Change |
|---|---|
| `plugins/doctrine/skills/research/SKILL.md` | new — the conventions contract |
| `plugins/doctrine/skills/slice/SKILL.md` | hook |
| `plugins/doctrine/skills/design/SKILL.md` | hook |
| `plugins/doctrine/skills/plan/SKILL.md` | hook |
| `plugins/doctrine/skills/phase-plan/SKILL.md` | hook |
| `src/research.rs` | new leaf — mint/check engine |
| `src/main.rs` | `mod research;` |
| `src/slice.rs` | `SliceCommand::Research` variant + dispatch |
| `src/commands/guard.rs` | write-class arm |
| `src/contentset.rs` | suppression removal (or fn removal) |
| `install/governance.md` | § Research agents stub |
| `install/glossary.md` | layout line |

Excluded by design: `.agents/skills/**` / `.doctrine/skills/**` (install-
derived mirrors, regenerated by the ritual, not authored).

**Post-authoring ritual** (embed footgun,
`mem.pattern.distribution.skill-refresh-command`): `touch src/install.rs &&
cargo build && ./target/debug/doctrine install -s research -y` (and `-s slice
-s design -s plan -s phase-plan` for the hooks). A bare `cargo build` after a
`plugins/`-only edit is a silent no-op.

## Verification alignment

- **VT:** baseline round-trip; absence semantics (design.md missing at mint →
  `added` on later check); drift classes (changed/removed); `--restamp`
  overwrite; clap parse test for the variant; guard write-class test.
  Behaviour-preservation: existing suites (contentset, review warm-cache)
  green unchanged.
- **VA:** post-ritual, the installed copies exist and match
  (`.doctrine/skills/research/`, agent symlink dirs); hook edits present in
  installed copies. (Not via `doctrine skills list` — deprecated alias,
  SL-088.)
- **VH:** this slice's own dogfood round is the pre-design evidence (this
  design cites it). Closure: one further real slice driven through the round,
  observations to RFC-011 case-notes.

## Open questions / risks

- OQ-1 (one vs two docs) → closed, D3. OQ-2 (verb shape) → closed, D2.
  OQ-3 (advisory surfacing) → closed, D4.
- **R1** (hooks under-deliver without enforcement) — carried; mitigation is
  the verb making absence/drift visible in-flow + RFC-011 evals. Escalation to
  gating requires an ADR (D6).
- **A1** (prose runner deferral suffices for both arms) — carried; pi scripts
  exist, claude arm uses subagents or `claude -p`.
- **A2** (new): the fixed baseline path set (scope/design/plan) is enough for
  v1; phase sheets and source files are deliberately outside the staleness
  domain (research goes stale against *intent* docs, not implementation).
