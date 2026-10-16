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
    dtoml[dtoml / install_config]
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
  a standalone `AGENTS.md` is present and is not an alias of `CLAUDE.md` (the
  rule moved out of `boot::resolve_harnesses`, `src/boot.rs:774-779`).
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
| boot wiring | harness set → `boot::wire` |
| harness convergence | `with_wanted` / `survey` → ownership `Action`s |
| legacy starter removal | identity scan (§ Hymn starters) |
| gitignore block | `write_gitignore_block` |
| stamps | tracked key + local marker |
| currency | `install_state::classify` (downgrade gate) |

Executing it returns `Vec<Touched { path, change: Created | Changed | Removed, tier }>`;
each writer reports its own outcome. The commit hint reads that.

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
  asks again.
- A legacy-starter removal adds a bullet: `• .doctrine/hymns/  remove N unedited
  starter copies so their slots follow framework updates`.
- A downgrade adds a warning line above `Proceed?` (§ Version currency).
- `-y` prints the overview and proceeds; `--dry-run` prints overview and details
  and stops.
- The memory-sync, boot-wire, hymn-starter and per-harness prompts are removed,
  with `prompt_step` and the `a` answer. `prompt_confirm` remains the single
  primitive (still shared with `claude install`, SPEC-010).
- Progress lines after proceeding are unchanged (each leg still reports, POL-003).

### Commit hint

After a successful run (DEC-382):

```
Next: git add .gitignore .doctrine/doctrine.toml CLAUDE.md AGENTS.md .claude/settings.json \
        && git commit -m "chore: install doctrine 0.47.0"
```

- Paths: `Touched` entries whose tier is tracked, deduplicated, in plan order.
- Message: "install doctrine X" when no tracked stamp existed before; "update
  doctrine to X" when it changes.
- No tracked change → `Nothing to commit.` Not a git work tree → no hint.
- Printed, never run.

### Scope variants

- `--global`: no project `doctrine.toml` or checkout marker; no stamps, no
  currency gate, no commit hint.
- `--only-memory`: a partial install; writes neither stamp.

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

### Classifier

```rust
// install_state.rs (leaf, pure)
pub(crate) enum StampRead { Absent, Unreadable { path: PathBuf, reason: String }, Version(semver::Version) }

pub(crate) enum Currency {
  Current,
  CheckoutNotInstalled,                       // local Absent
  CheckoutOlder { checkout: Version },        // local < binary
  RepoOlder { repo: Version },                // tracked < binary
  RepoNewer { repo: Version },                // tracked > binary
  Unreadable { path: PathBuf, reason: String },
}

pub(crate) fn classify(tracked: &StampRead, local: &StampRead, binary: &Version) -> Vec<Currency>;
pub(crate) fn advisory(c: &Currency, binary: &Version) -> String;
```

- `classify` returns every finding (a checkout can be both older and on a
  lagging repo); empty means current. Order: unreadable, checkout, repo.
- Tracked `Absent` (pre-stamp install) classifies as `RepoOlder` against any
  binary, so the advice is to install and commit.
- `Unreadable` is never folded into `Absent` (STD-003); it carries the path and
  parse error.
- Version comparison is `semver` (new direct dependency); the binary version is
  `env!("CARGO_PKG_VERSION")`, parsed once by the caller.

### Advisory text

| finding | line |
|---|---|
| `CheckoutNotInstalled` | `doctrine isn't installed in this checkout: run \`doctrine install\`` |
| `CheckoutOlder` | `this checkout was installed by doctrine C (you have B): re-run \`doctrine install\`` |
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
- **`status`.** `BootSection` gains `currency: Vec<String>` (the advisory lines;
  empty when current) and the text view prints them under the boot line.
- Nowhere else.

### Downgrade gate (DEC-378)

Install classifies before executing. If the tracked stamp is newer than the
binary:

- interactive: the overview shows `! this repo is on doctrine R; you are running
  B. Proceeding will downgrade generated files.` and `Proceed?` defaults to N as
  usual;
- `-y`: refuse with that line plus `pass --allow-downgrade to proceed`;
- `--allow-downgrade`: proceed; the tracked stamp is rewritten to B and the
  commit hint names it.

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
- otherwise calls `project_starters` for that one slot: writes
  `.doctrine/hymns/<slot>.md` (framework body) and `<slot>.toml`
  (`replaces = "<slot>"`), each write-if-absent.
- an existing file is left alone and reported.
- prints the written paths and: `this copy replaces the framework text for
  <slot>; it no longer receives framework updates`.

`project_starters`' signature stays; the verb passes a one-element slot list.

### Legacy starter removal

During install, where the starter leg ran (before convergence bakes worker defs
from the disk corpus), `identical_starters(disk_root, embedded, seal)` finds each
`.doctrine/hymns/<slot>.md` snippet file (the band/label paths the corpus loader reads; `README.md` and other non-snippet files are never candidates) such that:

1. a framework snippet exists at the same slot path, and its bytes equal the
   disk file's bytes; and
2. the sidecar `<slot>.toml` is absent, or its bytes equal exactly what
   `project_starters` writes for that slot.

Each match (and its sidecar) is removed and reported:
`removed unedited starter .doctrine/hymns/<slot>.md (framework text applies again)`.
Anything else under `.doctrine/hymns/` is untouched. Removals are `Touched`
entries with tier tracked, so they appear in the commit hint.

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
| `src/install.rs` | `InstallPlan` + `Touched`; `render_overview`; single `Proceed? [y/N/d]`; delete `prompt_step`, the per-step prompts and the starter leg; `resolve_install_harnesses` replaces `detect_agents` + `install_harnesses`; `identical_starters` removal; stamp writes; downgrade gate; commit hint |
| `src/install_config.rs` | `installed_version: Option<String>` |
| `src/dtoml.rs` (or `install_config.rs`) | `write_installed_version` — comment-preserving `toml_edit` single-key write |
| `src/boot.rs` | `BOOT_REF_GUARD` sentence; `PRIOR_BOOT_REF_GUARDS` + `plan_boot_import` upgrade; AGENTS.md detection rule moved to the shared resolver (boot keeps calling it for `boot install`); currency lines in `boot` / `boot --check` |
| `src/status.rs` | `BootSection.currency`; text view lines |
| `src/commands/prompt.rs` | `Customise { slot, path }` (alias `customize`); currency lines after unbanded `resolve` |
| `src/commands/cli.rs` | `--allow-downgrade` on `install` |
| `install/manifest.toml` | remove `[hymns].expose` |
| `Cargo.toml` | `semver` direct dependency |
| `.doctrine/hymns/` (this repo) | seven identical copies removed by running install (dogfood) |
| `tests/e2e_install_*.rs` | binary-driven flow, hint, currency, downgrade, legacy removal, customise |

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
| Legacy removal deletes a customisation | exact byte identity on body and sidecar; non-snippet files never candidates; each removal reported and shown in the commit hint |
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
  findings; Unreadable never reported as Absent.
- `advisory` — one line per finding, constants only.
- `plan_boot_import` — current guard → Present; prior guard → replaced; bare
  line → upgraded; no block → created.
- `identical_starters` — identical + default sidecar → match; identical + no
  sidecar → match; edited body, edited sidecar, non-snippet file → no match.
- `write_installed_version` — comments and ordering preserved; table created
  when absent.

E2e (built binary, `tests/common`):

- fresh repo, interactive with `y`: overview, one prompt, commit hint naming
  exactly the tracked paths; no hymn files; both stamps written.
- `d` shows details then re-prompts; `N` writes nothing.
- `-y` prints overview, no prompt; `--dry-run` writes nothing (digest check).
- AGENTS.md-only repo: codex boot import and codex skills from one harness set.
- second install: `Nothing to commit.`
- currency: marker absent / older, stamp older / newer / missing / unreadable →
  expected line from `prompt resolve` (after the snapshot; snapshot bytes
  unchanged), `boot`, `status --json`.
- downgrade: interactive warns; `-y` refuses; `--allow-downgrade` proceeds and
  rewrites the stamp.
- `prompt customise role/worker` writes starter + sidecar; sealed slot refused;
  existing file left alone.
- legacy starters: identical copies removed and reported; edited copy kept;
  `adherence/low` emits once afterwards.
- `prompt resolve --role worker` after a fresh install contains the framework
  worker contract.

Dogfood: run install in this repo; the seven copies go; commit the result.

