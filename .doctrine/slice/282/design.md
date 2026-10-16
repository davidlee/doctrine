<!-- doctrine:section sec-problem -->
## Design problem

Interactive `doctrine install` asks a string of questions in internal vocabulary,
most of which are not real choices, and hands the user nothing at the end
(IMP-516). It writes hymn starters nobody asked for, which freeze their slots
against framework updates (IMP-513, ISS-210, ISS-521). And once install has run,
nothing tells an agent or operator whether a checkout is installed, or whether
its install matches the binary in use (IMP-514).

This design makes install explain itself and ask once, moves hymn starters to an
explicit verb, and adds an advisory version-currency signal. Decisions:
DEC-378 to DEC-385. Evidence: `.doctrine/slice/282/research/research.md`.

Out of scope: hard version pinning, automatic reinstall, reference-doc currency
in existing clients (SL-242, IMP-378), model-key vocabulary (IMP-489), codex
agent defs (SL-283).

<!-- doctrine:section sec-model -->
## System model

```mermaid
flowchart LR
  subgraph cmd[command tier]
    install[install.rs<br/>InstallPlan · run]
    boot[boot.rs<br/>boot · boot --check · guard]
    status[status.rs]
    prompt[commands/prompt.rs<br/>resolve · customise]
  end
  subgraph leaf[leaf tier]
    state[install_state.rs<br/>Currency · classify · advisory]
    own[install_ownership.rs]
    dtoml[install_config]
  end
  install --> own
  install --> state
  install --> dtoml
  boot --> state
  status --> state
  prompt --> state
  install -. writes .-> tracked[(doctrine.toml<br/>[install] installed-version)]
  install -. writes .-> local[(.doctrine/state/install.toml)]
  state -. reads via shell .-> tracked
  state -. reads via shell .-> local
```

Two stores, one question each (DEC-378, DEC-379):

| store | tier | answers |
|---|---|---|
| `installed-version` in `.doctrine/doctrine.toml` `[install]` | tracked | which doctrine version the repo is on (advisory team pin) |
| `.doctrine/state/install.toml` | gitignored runtime | whether this checkout ran install, and with which binary |

`install_state` is a pure leaf: no file I/O. Command-tier callers read both
stores and pass the results in, so boot, status and prompt depend downward on it
(ADR-001). It is the only home of advisory text (STD-001).

<!-- doctrine:section sec-flow -->
## Interactive install

### Harness set

One resolver, `resolve_install_harnesses(explicit, root, registry)`, replaces
`detect_agents` + `install_harnesses` (DEC-381):

- `--agent` given → exactly those names.
- otherwise → each registry harness whose `detect` path exists, plus codex when
  `AGENTS.md` is present, unless claude is detected *and* `AGENTS.md` resolves
  to `CLAUDE.md` (the predicate moves verbatim out of `boot::resolve_harnesses`,
  `src/boot.rs:763-779`; a lone symlinked pair with no `.claude/` stays codex,
  and its test, `src/boot.rs:4884`, runs through both `install` and `boot install`).
- none found and interactive → ask once, listing registry harness names; none
  found under `-y` → skip harness legs and say so.

Boot wiring maps this set through `boot_names`; convergence consumes it as is.

### InstallPlan

`InstallPlan` (in `install.rs`) composes what already exists; it adds no
parallel model (DEC-379):

| part | source |
|---|---|
| base steps | `build_plan` → `Plan` |
| memory sync | `corpus::sync_corpus` (planned, reported) |
| boot wiring | harness set → `boot::plan_wire` (split out of `boot::wire`, below) |
| harness convergence | `with_wanted` / `survey` → ownership `Action`s |
| legacy starter removal | identity scan (§ Hymn starters) |
| gitignore block | `write_gitignore_block` |
| stamps | tracked key + local marker |
| currency | `install_state::classify` (downgrade gate) |

`boot::wire(root, exec, harnesses, dry_run) -> Result<()>` (`src/boot.rs:3102`)
prints and returns nothing, and it catches each harness's refresh error,
prints `refresh failed` and continues (`src/boot.rs:3307-3310`). It is split
into `plan_wire` (pure over the `@`-import line edits only) and `apply_wire`
(writes, prints the same lines, continues past a failed harness, and returns
per-harness results). The harness refresh (hooks, settings, registrations)
does harness I/O, so it stays in `apply_wire`, which reports what it touched
and what it refused; a refused wiring (printed manual fallback, foreign file
left in place) fails that harness (RV-422 F-4, F-7). `wire` becomes their composition and still returns
`Ok(())` after printing, so `boot install` is unchanged and its suites are the
behaviour-preservation proof; install maps any failed harness to a `Failed`
boot-wiring leg.

Executing the plan returns a `LegOutcome` per leg (`Done(Vec<Touched>)`,
`Refused(reason)` or `Failed(error)`), with
`Touched { path, change: Created | Changed | Removed }`. Today's runner catches
and prints leg errors and returns `Ok(())` (`src/install.rs:593-714`), and the
gitignore writer reports malformed markers as `Ok(())` (`src/install.rs:1366-1404`);
both now return their outcome instead, still printing the same progress and
error lines (POL-003).

Stamps are the last step and are written only when every leg is `Done`. A
leg with nothing to do, or deliberately skipped (no harness found under `-y`),
is `Done` with no `Touched` entries.
Otherwise install prints `install incomplete: <legs>; version not recorded` and
leaves both stamps as they were, so currency keeps reporting the old state.

### Output

```
$ doctrine install

Doctrine will set this project up for: claude, codex   (found .claude/, AGENTS.md;
                                                        choose others with --agent)

  • .gitignore            add a managed block so generated files stay out of git
  • .doctrine/            create it; your config is tracked, generated parts are ignored
  • CLAUDE.md, AGENTS.md  add a short section so your agent loads doctrine each session
  • claude, codex         install skills and agent definitions (generated, ignored)
  • doctrine.toml         record doctrine 0.47.0 as this repo's version

Proceed? [y/N/d] (d = show every step)
```

- `render_overview(&InstallPlan)` produces the bullets; lines appear only for
  parts with work to do (e.g. no gitignore line when the block is current).
- `d` prints `print_plan`, `print_forward_summary`, `print_ownership_plan`, then
  asks again. A new `prompt_choice` reads one answer into
  `Choice { Proceed, Abort, Details }`; only install loops on `Details`.
  `prompt_confirm` becomes `prompt_choice(..) == Proceed`, so its other callers
  (`boot install`, `src/boot.rs:2956`; memory sync, `src/corpus.rs:447,491`)
  keep their yes/no behaviour.
- A legacy-starter removal adds a bullet: `• .doctrine/hymns/  remove N unedited
  starter copies so their slots follow framework updates`.
- A downgrade adds a warning line above `Proceed?` (§ Version currency).
- `-y` prints the overview and proceeds; `--dry-run` prints overview and details
  and stops.
- The memory-sync, boot-wire, hymn-starter and per-harness prompts are removed,
  with `prompt_step` and the `a` answer.
- Progress lines after proceeding are unchanged (each leg still reports, POL-003).

### Commit hint

After a successful run (DEC-382):

```
Next:
  git add -- .gitignore .doctrine/doctrine.toml CLAUDE.md AGENTS.md .claude/settings.json
  git commit -m 'chore: install doctrine 0.47.0' -- .gitignore .doctrine/doctrine.toml CLAUDE.md AGENTS.md .claude/settings.json
```

- Paths: `Touched` paths not covered by the ignore contract install itself
  maintains (the managed block entries plus `.doctrine/state/`), deduplicated,
  in plan order, minus paths git ignores (one batched `git check-ignore`,
  DEC-382 as amended, RV-422 F-5). If that query fails, install says so and
  the hint lists every path.
- The commit is path-limited with the same list, so already-staged unrelated
  work stays out. One command per line, no shell operator, so the hint pastes
  into any shell including nushell (RV-422 F-10). Paths are root-relative and
  quoted only when needed; when the working directory is not the project root
  a `cd <root>` line comes first.
- Removed starters are listed; `git add -- <deleted path>` stages the deletion.
- Scope: doctrine's own writes (DEC-382 as amended, RV-421 F-11). The delegated `npx skills` leg (codex, pi;
  `skills_via = "universal"`) reports only success, so its copies under
  `.agents/skills/` cannot be listed; when that leg ran, the hint adds
  `also review .agents/skills/ (written by npx skills)` (same disclosure as
  `NPX_SKILLS_NOTE`, `src/install.rs:1334`).
- Message: "install doctrine X" when no tracked stamp existed before; "update
  doctrine to X" when it changes.
- No tracked change → `Nothing to commit.` Not a git work tree → no hint.
- Printed, never run.

### Scope variants

The CLI contract says `--global` installs to the user directory instead of the
project and `--only-memory` installs only the memory skills. Neither is a
project install, so neither writes stamps, runs the currency gate or prints a
commit hint; the overview renders whatever legs the plan holds.

Today both diverge from that contract: `--global` still runs the project base
plan (`src/install.rs:207-236`; ISS-524) and `--only-memory` is unwired, so it
runs a full install (`src/install.rs:189-201`; ISS-523). This slice does not
re-scope either mode. Until they are fixed, a run under either flag can write
project files and leave the checkout classified as not installed; the advice
(run a plain `doctrine install`) is correct for that state.

<!-- doctrine:section sec-currency -->
## Version currency

### Stores

Tracked (DEC-378): `installed-version = "0.47.0"` in the `[install]` table of
`.doctrine/doctrine.toml`, read through `InstallConfig` (`src/install_config.rs:72`)
as `installed_version: Option<String>`. Install writes it with a single-key
`toml_edit` edit that preserves every other byte of the file, creating the
`[install]` table only when absent. Nothing in doctrine wrote this file before;
the edit is confined to one helper, `write_installed_version`.

Local (DEC-379): `.doctrine/state/install.toml`, written only by install:

```toml
version = "0.47.0"
harnesses = ["claude", "codex"]
```

Path constant `INSTALL_MARKER_REL` (precedent: `BOOT_REL`). Covered by the
existing `.doctrine/state/` gitignore entry.

Marker lookup (DEC-379 as amended, RV-421 F-1) is one command-tier helper,
`read_install_marker(root) -> (StampRead, MarkerSource)`:

1. `<root>/INSTALL_MARKER_REL`, when present (`MarkerSource::Checkout`);
2. otherwise, when `root` is a linked git worktree (`git rev-parse
   --git-common-dir` differs from `--git-dir`), the main worktree's marker
   (`MarkerSource::Main(path)`); linked worktrees and dispatch forks never
   receive `.doctrine/state/` (fork provisioning withholds the state tier).
   The main root is the common dir's parent, accepted only if that directory's
   `.git` is the common dir itself or a gitfile pointing to it. Otherwise (e.g.
   `git init --separate-git-dir`, where the common dir sits outside any
   checkout) the main root cannot be located: `MarkerSource::Unlocated`, and
   the `CheckoutNotInstalled` line says so (`… (main worktree not located)`)
   rather than silently reporting this checkout alone (STD-003);
3. otherwise `Absent`.

Outside git, or when git is unavailable, only step 1 applies (POL-002). When
the source is `Main`, the checkout advisories name that root
(`… run \`doctrine install\` in <main root>`); `advisory` takes the source as an
argument, so the text stays in the leaf.

### Classifier

```rust
// install_state.rs (leaf, pure)
pub(crate) enum StampRead { Absent, Unreadable { path: PathBuf, reason: String }, Version(semver::Version) }

pub(crate) enum Currency {
  Current,
  CheckoutNotInstalled,                       // local Absent
  CheckoutOlder { checkout: Version },        // local < binary
  CheckoutNewer { checkout: Version },        // local > binary
  RepoUnstamped,                              // tracked Absent (pre-stamp install)
  RepoOlder { repo: Version },                // tracked < binary
  RepoNewer { repo: Version },                // tracked > binary
  Unreadable { path: PathBuf, reason: String },
}

pub(crate) fn classify(tracked: &StampRead, local: &StampRead, binary: &Version) -> Vec<Currency>;
pub(crate) fn advisory(c: &Currency, binary: &Version, marker: &MarkerSource) -> String;
```

- `classify` returns every finding (a checkout can be both older and on a
  lagging repo); empty means current. Order: unreadable, checkout, repo.
- Tracked `Absent` (pre-stamp install) classifies as `RepoUnstamped`: it reads
  as older (DEC-378) but has no version to quote. When `CheckoutNotInstalled`
  is also found, `RepoUnstamped` is dropped: installing writes the stamp and the
  commit hint carries it.
- `CheckoutNewer` covers the stale-`PATH` case (an older binary shadowing the
  one that installed this checkout) whether or not the tracked stamp is
  committed.
- `Unreadable` is never folded into `Absent` (STD-003); it carries the path and
  parse error.
- Version comparison is `semver` (new direct dependency); the binary version is
  `env!("CARGO_PKG_VERSION")`, parsed once by the caller.

### Advisory text

| finding | line |
|---|---|
| `CheckoutNotInstalled` | `doctrine isn't installed in this checkout: run \`doctrine install\`` |
| `CheckoutOlder` | `this checkout was installed by doctrine C (you have B): re-run \`doctrine install\`` |
| `CheckoutNewer` | `this checkout was installed by doctrine C, newer than yours (B): upgrade doctrine or check which one is on PATH` |
| `RepoUnstamped` | `this repo has no recorded doctrine version: run \`doctrine install\` and commit` |
| `RepoNewer` | `this repo is on doctrine R and yours is B: upgrade doctrine` |
| `RepoOlder` | `your doctrine B is newer than this repo's R: run \`doctrine install\` and commit to move the repo up, or use R` |
| `Unreadable` | `cannot read <path> (<reason>): fix or remove it, then run \`doctrine install\`` |

### Sites (DEC-380)

- **Session hook.** Unbanded `prompt resolve` appends the advisory lines after
  the cascade, outside the boot snapshot bytes, only when findings exist. Same
  for the `--json` hook envelope (appended to the cascade text before
  `session_start_hook_json`).
- **`doctrine boot` / `boot --check`.** Print the lines after their own output.
  `--check` exit status is unchanged (currency is advisory).
- **`status`.** `BootSection` gains `currency: Vec<Currency>` (serialised as
  structured findings: kind plus versions or path; empty when current). Only the
  text view renders them, via `advisory`, under the boot line.
- Nowhere else.

### Downgrade gate (DEC-378)

Install classifies before executing. If the tracked stamp or the local marker
is newer than the binary (`RepoNewer` or `CheckoutNewer`; R below is the newer
of the two):

- interactive: the overview shows `! this project was installed by doctrine R;
  you are running B. Proceeding will downgrade generated files.` and `Proceed?`
  defaults to N as usual;
- `-y`: refuse with that line plus `pass --allow-downgrade to proceed`;
- `--allow-downgrade`: proceed; the local marker records B. The tracked stamp
  is preserved only when it is newer than B (one stale binary does not lower
  the team's recorded version, so `RepoNewer` keeps reporting until the binary
  is upgraded); when it is absent or older (only the local marker was newer),
  it is written as B like any successful install.

<!-- doctrine:section sec-guard -->
## Boot guard

`BOOT_REF_GUARD` (`src/boot.rs:812`) gains one sentence (DEC-383):

> If you have NOT seen `BOOT-SENTINEL: doctrine-governance-snapshot` anywhere in
> your context (system prompt or preceding messages), you MUST read the file
> referenced above now. If you HAVE seen it, you MUST NOT — the content is
> already in context. **If that file does not exist, doctrine is not set up in
> this checkout: tell the user to install doctrine and run `doctrine install`.**

Upgrade in place: `PRIOR_BOOT_REF_GUARDS: &[&str]` holds earlier guard texts
(today: the current one). `plan_boot_import`:

- current guard present → `Present`, no write;
- a prior guard present → replace it with the current guard (`Add`);
- bare `@` line → existing `upgrade_bare_boot_ref`;
- otherwise as today.

The `Present` test matches the emitted text, not just the sentinel, so the
comparator follows the output. Each future guard edit appends the outgoing text
to the list.

<!-- doctrine:section sec-hymns -->
## Hymn starters

### Install leg removed

The starter leg in `run_forward_steps` (`src/install.rs:633-663`) and
`[hymns].expose` (`install/manifest.toml:103`) are deleted, with
`embedded_expose_set`. Fresh installs write no hymn files (REQ-355).

### `doctrine prompt customise <slot>`

New `PromptCommand::Customise { slot, path }`, clap alias `customize` (DEC-384):

- `slot` is a framework slot label, e.g. `role/worker`.
- unknown slot → error listing framework slots; sealed slot (`[hymns].seal`) →
  refusal naming the seal.
- no tracked `installed-version` → refusal: `run \`doctrine install\` first`
  (DEC-384 as amended, RV-421 F-18). Nothing the verb writes can then precede
  the one-time legacy cleanup, which would delete it as an unedited copy.
- otherwise calls `project_starters` for that one slot: writes
  `.doctrine/hymns/<slot>.md` (framework body) and `<slot>.toml`
  (`replaces = "<slot>"`), each write-if-absent.
- an existing file is left alone and reported.
- prints the written paths and: `this copy replaces the framework text for
  <slot>; it no longer receives framework updates`.

`project_starters`' signature stays; the verb passes a one-element slot list.

### Legacy starter removal

A one-time migration (DEC-384 as amended): it runs only while the tracked
`installed-version` is absent, i.e. on the first install after an install that
predates the stamp. Once the stamp exists nothing is removed, so a starter
written by `prompt customise`, byte-identical by construction, survives every
later install (RV-421 F-16).

During that install, where the starter leg ran (before convergence bakes worker defs
from the disk corpus), `identical_starters(disk_root, embedded, seal)` finds each
`.doctrine/hymns/<slot>.md` snippet file (the band/label paths the corpus loader reads; `README.md` and other non-snippet files are never candidates) such that:

1. a framework snippet exists at the same slot path, and its bytes equal the
   disk file's bytes; and
2. the sidecar `<slot>.toml` is absent, or its bytes equal exactly what
   `project_starters` writes for that slot.

Each match (and its sidecar) is removed and reported:
`removed unedited starter .doctrine/hymns/<slot>.md (framework text applies again)`.
Anything else under `.doctrine/hymns/` is untouched. Removals are `Touched`
entries outside the ignore contract, so they appear in the commit hint.

In this repo that removes `harness/claude`, `harness/cursor`,
`model/anthropic/claude-sonnet-4`, `model/deepseek/_default`, `role/worker`,
`preamble/core` and `model/adherence/low` (ISS-521). `role/orchestrator`,
`project/*` and `README.md` stay.

<!-- doctrine:section sec-code-impact -->
## Code impact

| path | change |
|---|---|
| `src/install_state.rs` (new, leaf) | `StampRead`, `Currency`, `classify`, `advisory`; unit-tested exhaustively |
| `.doctrine/adr/001/layering.toml` | register `install_state = "leaf"` |
| `src/install.rs` | `InstallPlan`, `LegOutcome` + `Touched` (leg runner and gitignore writer return outcomes); `render_overview`; `prompt_choice` + single `Proceed? [y/N/d]` (`prompt_confirm` an adapter); delete `prompt_step`, the per-step prompts and the starter leg; `resolve_install_harnesses` replaces `detect_agents` + `install_harnesses`; `identical_starters` removal (gated on absent stamp); `read_install_marker` (linked-worktree fallback); stamp writes last, only on full success; downgrade gate; path-limited commit hint |
| `src/install_config.rs` | `installed_version: Option<String>` |
| `src/install_config.rs` | `write_installed_version` — comment-preserving `toml_edit` single-key write |
| `src/boot.rs` | `BOOT_REF_GUARD` sentence; `PRIOR_BOOT_REF_GUARDS` + `plan_boot_import` upgrade; `wire` split into `plan_wire` (import edits) + `apply_wire` (writes, harness refresh, refused-wiring report); AGENTS.md detection rule moved to the shared resolver (boot keeps calling it for `boot install`); currency lines in `boot` / `boot --check` |
| `src/status.rs` | `BootSection.currency` (structured in JSON); text view lines |
| `src/commands/prompt.rs` | `Customise { slot, path }` (alias `customize`); currency lines after unbanded `resolve` |
| `src/commands/cli.rs` | `--allow-downgrade` on `install` |
| `install/manifest.toml` | remove `[hymns].expose` |
| `Cargo.toml` | `semver` direct dependency |
| `.doctrine/hymns/` (this repo) | seven identical copies removed by running install (dogfood) |
| `src/git.rs` | `ignored_among` — one batched `git check-ignore` for the commit hint |
| `src/commands/guard.rs` | classify `prompt customise` as a write |
| `src/main.rs` | register `install_state` |
| `src/worktree/mod.rs`, `src/worktree/shared.rs` | `resolve_common_dir` crate-visible for the marker lookup |
| `tests/e2e_install_*.rs` | binary-driven flow, hint, currency, downgrade, legacy removal, customise |
| `tests/common/mod.rs`, `tests/e2e_prompt_resolve_golden.rs`, `tests/e2e_claude_install.rs` | shared stamp/digest helpers; golden and claude suites set up starters via `prompt customise` |

Design-target selectors record exactly these paths.

<!-- doctrine:section sec-revision -->
## Revision

One REV, drafted at reconcile and landed as a slice output (DEC-385, DEC-376
precedent):

- **SPEC-009** — interactive flow (overview, one prompt, real choices only,
  commit hint); tracked stamp and local marker; currency advisory and its sites;
  downgrade gate; `[hymns].expose` retired; REQ-355 activated.
- **SPEC-011 REQ-185** — guard content includes the not-installed clause;
  prior guards upgrade in place.
- **SPEC-023 REQ-322 / REQ-329** — the self-replaces sidecar is written by
  `prompt customise`, not install-time projection.

<!-- doctrine:section sec-risks -->
## Risks and residuals

| risk | mitigation |
|---|---|
| Stamp write damages a hand-edited `doctrine.toml` | single-key `toml_edit`; test: this repo's commented file round-trips byte-identical outside the key |
| Legacy removal deletes a customisation | runs only while `installed-version` is absent (one-time migration), and `prompt customise` refuses while it is absent, so customise output is never a candidate; exact byte identity on body and sidecar; non-snippet files never candidates; each removal reported and shown in the commit hint |
| Advisory sites miss projects rooted only by `pyproject.toml` / `justfile` | pre-existing marker divergence between install and readers; deferred to ISS-522 |
| Bake reads a different disk corpus after removal | removal runs before convergence, where the starter leg ran; e2e asserts a second install is a no-op |
| Rollout noise ("newer than repo") | intended; clears once the stamp is committed |
| Same-version builds indistinguishable | release discipline: behaviour-changing slices get at least a patch release |
| SL-283 also changes the bake | land SL-282 first |
| Existing clients keep old reference-doc copies | out of scope (SL-242, IMP-378) |

<!-- doctrine:section sec-verification -->
## Verification

Unit (pure):

- `install_state::classify` — every combination of tracked / local in
  {Absent, Unreadable, older, equal, newer} against a fixed binary; order of
  findings; Unreadable never reported as Absent; tracked Absent → `RepoUnstamped`,
  dropped beside `CheckoutNotInstalled`; local newer → `CheckoutNewer`.
- `advisory` — one line per finding, constants only.
- `prompt_choice` — `y` / `n` / `d` / empty map to Proceed / Abort / Details /
  Abort; `prompt_confirm` keeps its yes/no results.
- `plan_boot_import` — current guard → Present; prior guard → replaced; bare
  line → upgraded; no block → created.
- `identical_starters` — identical + default sidecar → match; identical + no
  sidecar → match; edited body, edited sidecar, non-snippet file → no match.
- `write_installed_version` — comments and ordering preserved; table created
  when absent.

E2e (built binary, `tests/common`):

- fresh repo, interactive with `y`: overview, one prompt, commit hint naming
  exactly the tracked paths; no hymn files; both stamps written.
- `d` then `y` proceeds; `d` then `N` writes nothing; `N` writes nothing.
- commit hint: an unrelated staged file stays out of the printed commit's
  pathspec; run from a nested directory → a leading `cd <root>` line.
- forced leg failure (boot-wire error, malformed gitignore markers): progress
  shows the error, `install incomplete` printed, neither stamp written.
- `-y` prints overview, no prompt; `--dry-run` writes nothing (digest check).
- AGENTS.md-only repo: codex boot import and codex skills from one harness set.
- lone `CLAUDE.md`↔`AGENTS.md` symlink pair, no `.claude/`: codex, via both
  `install` and `boot install`.
- second install: `Nothing to commit.`
- currency: marker absent / older / newer, stamp older / newer / missing /
  unreadable → expected line from `prompt resolve` (after the snapshot; snapshot
  bytes unchanged) and `boot`; structured finding in `status --json`.
- downgrade (stamp newer, and marker newer with stamp absent): interactive
  warns; `-y` refuses; `--allow-downgrade` proceeds and rewrites the local
  marker; the tracked stamp follows the rule above.
- `prompt customise role/worker` in a pre-stamp repo refuses (`run doctrine
  install first`) and writes nothing.
- `prompt customise role/worker` writes starter + sidecar; sealed slot refused;
  existing file left alone.
- legacy starters: identical copies removed and reported; edited copy kept;
  `adherence/low` emits once afterwards.
- `prompt resolve --role worker` after a fresh install contains the framework
  worker contract.
- linked worktree of an installed checkout: no checkout advisory; of an
  uninstalled one: the advisory names the main root; a worktree with its own
  marker uses it; `--separate-git-dir` layout: `main worktree not located`.
- boot wiring with one harness's refresh failing (caught inside `apply_wire`):
  `boot install` output unchanged; install reports the leg failed and writes
  no stamps.
- `--allow-downgrade` with only the local marker newer: tracked stamp absent →
  written as B; tracked older → rewritten to B; tracked newer → kept.
- `--global` and `--only-memory`: no stamps, no currency gate, no commit hint.
- install → `prompt customise role/worker` → install: both files remain.
- legacy removal does not run once `installed-version` exists.

Dogfood: run install in this repo; the seven copies go; commit the result.

