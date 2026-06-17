# Design: SL-088 — Consolidated installer

## 1. Target CLI surface

```
doctrine install [--agent <name>...] [--skill <id>...] [--domain <name>...]
                 [--only-memory] [--global] [--dry-run] [-y] [-p <path>]
```

| Flag | Source | Purpose |
|------|--------|---------|
| `-p`, `--path` | existing | Explicit project root |
| `--dry-run` | existing | Print plan and exit |
| `-y`, `--yes` | existing | Skip all prompts |
| `-a`, `--agent` | `ClaudeCommand::Install` | Target agent(s); repeatable. Default: auto-detect |
| `-s`, `--skill` | `ClaudeCommand::Install` | Skill id(s); repeatable. Default: all |
| `-d`, `--domain` | `ClaudeCommand::Install` | Domain(s); repeatable. Default: all |
| `--only-memory` | `ClaudeCommand::Install` | Install only the memory skills. Conflicts with `--skill`/`--domain` |
| `-g`, `--global` | `ClaudeCommand::Install` | Install to user home instead of project |

## 2. Install flow

Root is detected once at the top. `--dry-run` prints the full plan (base +
forward steps) and exits. `-y` skips all prompts.

### Stage 1 — base manifest (always, no prompt)

Existing `install::run` logic: materialize embedded files into `.doctrine/`,
create dirs from manifest, append gitignore entries. Idempotent — skips
existing files, deduplicates gitignore entries.

### Stage 2 — forward-step summary (always printed)

A compact summary of the forward steps, one line each. Wording adapts to
`--dry-run` vs live:

```
# live (base install just ran)
Base install complete. Forward steps:

  memory sync  materialize shipped corpus into .doctrine/memory/shipped/
  boot         wire @-import into AGENTS.md/CLAUDE.md + session hooks
  skills       install skills + agent defs for claude
  skills       install skills for pi (delegates to npx)

# --dry-run (nothing executed yet)
Forward steps (not executed under --dry-run):

  memory sync  materialize shipped corpus into .doctrine/memory/shipped/
  ...
```

Step labels adapt to detected/selected agents. If no agents are detected and
none specified, skills steps are omitted (non-fatal — user may just want base
files).

### Stage 3 — forward steps with individual prompts

The forward steps run in least-invasive-first order: memory sync is purely
additive, boot modifies config files, skills install mutates agent paths and
may spawn `npx`.

Each step prompts `y/N/a`:

- `y` — yes to this step
- `N` — no (default, uppercase indicates default)
- `a` — yes to this and all remaining steps

Steps and their underlying calls:

| # | Prompt | Underlying call |
|---|--------|-----------------|
| 1 | `Materialize shipped memory corpus? [y/N/a]` | `corpus::sync_corpus(&root, &corpus::embedded_assets(), false)` |
| 2 | `Wire @-import + session hooks for detected harnesses? [y/N/a]` | `boot::wire(&root, &exec, &harnesses, false)` |
| 3+ | `Install skills for claude? [y/N/a]` | skills materialize + install agent-def + SubagentStart hook |
| ... | `Install skills for pi? [y/N/a]` | skills delegate to `npx skills` + install pi agent-def |

Boot wiring is independent of skill install. Harness labels in the boot
prompt are auto-detected from the filesystem (`.claude/`, `.codex/`), not from
`--agent` — the prompt uses the neutral phrase "detected harnesses" to avoid
confusion when `--agent` flags differ from what is present on disk. When no
harness directories exist, the boot prompt is skipped (no harnesses → nothing
to wire).

Agent-def install rides each skills step: `--agent claude` installs
`.claude/agents/dispatch-worker.md`, `--agent pi` installs
`.pi/agents/dispatch-worker.md`, both via canonical copy + symlink (existing
pattern).

Agent resolution in the consolidated `install` path is non-fatal: when no
`.claude/` directory exists and no `--agent` flag is given, skills steps are
skipped (empty agent list). This is a relaxed resolver distinct from
`skills::resolve_agents()`, which `bail!`s — the focused resolver is preserved
for the standalone `skills list` verb. The consolidated path calls a separate
`detect_agents()` that returns an empty `Vec` instead of erroring.

### 4. Prompt helper

```rust
/// Returns `true` if the user wants to proceed. `all_yes` is set to `true`
/// when the user picks "a" (yes to all remaining).
fn prompt_step(question: &str, yes: bool, all_yes: &mut bool) -> io::Result<bool> {
    if yes || *all_yes {
        return Ok(true);
    }
    let mut stdout = io::stdout();
    write!(stdout, "\n{question} ")?;
    stdout.flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    match line.trim().to_lowercase().as_str() {
        "y" => Ok(true),
        "a" => {
            *all_yes = true;
            Ok(true)
        }
        _ => Ok(false), // "n", "", anything else → no
    }
}
```

## 3. Module changes

### `src/main.rs`

- `Command::Install` gains `agent`, `skill`, `domain`, `only_memory`, `global`
- `Command::Claude` and `enum ClaudeCommand` removed entirely
- `SkillsCommand::Install` variant removed; `SkillsCommand::List` kept
  (hidden deprecated alias)
- Match arm: `Command::Install { .. }` → `install::run(path, dry_run, yes,
  &agent, &skill, &domain, only_memory, global)`
- `worker_guard` write label: `Command::Install { .. }` already labelled
  `Write("install")` — no change needed

### `README.md`

All five `claude install` references must be updated to the consolidated
surface before the command is removed:

- **L44:** `npx skills add davidlee/doctrine  # or \`doctrine install --agent claude\` for claude code only`
- **L84:** `doctrine install --agent claude --yes # skills + dispatch-worker agent + SubagentStart hook into .claude`
- **L86:** `doctrine install --agent universal --yes`
- **L104:** `doctrine install --agent claude --only-memory -y`
- **L153:** `doctrine install --agent claude            # from binary, or`

### `src/install.rs`

`run()` signature expands:

```rust
pub(crate) fn run(
    path: Option<PathBuf>,
    dry_run: bool,
    yes: bool,
    agents: &[String],
    skills: &[String],
    domains: &[String],
    only_memory: bool,
    global: bool,
) -> anyhow::Result<()>
```

Internally:
1. Load manifest, detect root, build base plan, print it
2. If `!dry_run`, execute base plan
3. Resolve agents via a relaxed `detect_agents(agents, &root)` — returns empty
   `Vec` when no `.claude/` and no `--agent` (never errors), distinct from
   `skills::resolve_agents()`
4. Print forward summary — skills steps omitted when agent list is empty
5. Walk forward steps with `prompt_step` — boot prompt skipped when no
   harness directories are detected

Agent-def install requires the pi embed path. SL-084 places
`install/agents/pi/dispatch-worker.md` in the embed. The install code reads it
via the existing `install::embedded_asset` accessor and writes it to
`.doctrine/agents/pi/dispatch-worker.md` (canonical) + symlinks
`.pi/agents/dispatch-worker.md` at it.

### `src/skills.rs`

- `run_install()` stays `pub(crate)` — now called from `install.rs`, not from
  `main.rs`
- `InstallArgs` unchanged; the CLI arg parsing moves to `main.rs`
- `resolve_agents()` stays focused (bails on no agent) — the relaxed
  `detect_agents()` in `install.rs` handles the "empty list is fine" policy
- The per-agent skills install logic (materialise canonical + symlink for
  Claude, delegate `npx skills` for others) is extracted from `execute()` into
  two callable functions:
  - `install_for_claude(root, &catalog, &selected, global) -> Result<()>`
  - `install_for_other(agent_name, &catalog, &selected, global, runner) -> Result<()>`

### `src/corpus.rs`

- `run_sync()` unchanged for `doctrine memory sync`
- `sync_corpus()` and `embedded_assets()` already `pub(crate)` — called
  directly from `install.rs`

### `src/boot.rs`

- `run_install()` unchanged for `doctrine boot install`
- `wire()` already `pub(crate)` (called internally by `run_install`) — called
  directly from `install.rs`

## 4. Agent-def install (generalized)

SL-084 creates `install/agents/pi/dispatch-worker.md` as an embed asset. The
consolidated install writes agent defs using the same canonical-copy + symlink
pattern as the Claude dispatch-worker agent (SL-056 PHASE-11).

Canonical paths differ by agent to avoid collisions:
- Claude: `.doctrine/agents/dispatch-worker.md` (flat, existing path)
- Pi:     `.doctrine/agents/pi/dispatch-worker.md` (namespaced)

The asymmetry is intentional: Claude shipped first with a flat canonical path
(SL-056 PHASE-11); that path is grandfathered to avoid breaking existing
installs. New agents use namespaced canonical paths under
`.doctrine/agents/<name>/` to prevent collisions as more agent types are
added. The embed sources are namespaced for both (`install/agents/claude/`
and `install/agents/pi/`) — the flat canonical for Claude is a compat
concession, not a pattern to replicate.

Link targets:
- Claude: `.claude/agents/dispatch-worker.md` → `../../.doctrine/agents/dispatch-worker.md`
- Pi:     `.pi/agents/dispatch-worker.md` → `../../.doctrine/agents/pi/dispatch-worker.md`

A single `install_agent_def(root, agent_name, canon_subdir, embed_asset, global)`
function handles both. `canon_subdir` is `None` for Claude (flat) and
`Some("pi")` for pi (namespaced). Reuses `classify_link`/`write_link`/
`relative_target` from `skills.rs` — no parallel symlink impl.

## 5. Test strategy

### Unit tests (new, `src/install.rs`)

- `prompt_step` returns true for `"y"`, `"a"`, when `yes=true`, when
  `all_yes=true`
- `prompt_step` returns false for `"n"`, `""`, `"no"`, `"x"`
- `prompt_step` sets `*all_yes = true` on `"a"` and returns true
- Forward summary lists correct steps based on detected agents
- Agent auto-detection defaults to Claude when `.claude/` exists

### Integration tests (new)

- `doctrine install --dry-run` prints base + forward plan, exits 0
- `doctrine install -y` (in temp dir with `.claude/`) completes all steps
  without interaction
- `doctrine install` with all "n"/empty inputs does base only
- `doctrine install --agent pi --dry-run` prints pi delegation plan
- Standalone `doctrine memory sync`, `doctrine boot install`,
  `doctrine skills list` still work

### Removal tests

- `doctrine --help` shows no `claude` subcommand
- `doctrine claude install` yields "error: unrecognized subcommand"
- `doctrine skills install` yields "error: unrecognized subcommand" (the
  hidden `skills list` still works)

### Migration of existing e2e tests

- `tests/e2e_claude_install.rs` — rewritten to drive the consolidated
  `doctrine install --agent claude --yes` surface. The same assertions are
  preserved: SubagentStart hook is merged into `.claude/settings.local.json`,
  the dispatch-worker agent def resolves at `.claude/agents/`, and the hidden
  `skills install` alias is removed. The test file is renamed accordingly
  (e.g. `tests/e2e_install_claude.rs`).
- `tests/e2e_worker_guard.rs` — goldens updated from `claude install` to
  `install`. The test function `claude_install_and_skills_alias_refuse_in_worker_mode`
  is renamed to cover the consolidated verb. The hidden `skills install`
  alias path is removed (that verb no longer exists).

## 6. Edge cases

- **No `.claude/` and no `--agent`:** Agent resolution returns an empty list
  (non-fatal). Skills steps are skipped. Base install + memory + boot still
  run (boot auto-detects harnesses independently). The standalone `claude
  install` (now removed) was the only path that needed the hard error.
- **`--only-memory` with `--agent pi`:** The skills install step only
  installs `record-memory` + `retrieve-memory` for pi via `npx skills
  --skill record-memory --skill retrieve-memory`. clap enforces
  `conflicts_with_all = ["skill", "domain"]`.
- **`--global`:** All paths (canonical dir, agent links) anchor at `$HOME`.
  Boot wiring skips the SubagentStart hook (project-local only). Gitignore
  entries anchor at `$HOME` — existing behavior, unchanged.
- **Dry-run with forward steps:** Printed but not executed. User sees
  exactly what would happen.
- **Partial failure:** If a forward step fails, print the error and
  continue to the next step. The base install is already done.
- **Worker mode:** `worker_guard` already refuses `Write("install")` — no
  change needed.

## 7. Sequence with SL-084

SL-084 creates the pi agent-def content (`install/agents/pi/dispatch-worker.md`
in the embed) and updates dispatch skills. SL-088 adds the install path for it.
SL-084's PHASE-04 VA-3 states "`.pi/agents/dispatch-worker.md` is not installed
by `doctrine claude install`" — correct; it's installed by the consolidated
`doctrine install --agent pi`, which is this slice.

No dependency edge needed — SL-084's content is already authored; SL-088 just
needs the embed file to exist at the expected path.
