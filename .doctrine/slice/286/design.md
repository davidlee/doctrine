<!-- doctrine:section sec-code-impact -->
## Code impact

| path | change |
|---|---|
| `src/codex_mcp.rs` (new, command tier) | the leg: `CodexUserPlan`, its supplied inputs (`FileRead`, `BakeFacts`, `CodexEnv`), pure `destination`/`bake`/`plan`, the user and project classifiers (`is_wrapper_form` is the one wrapper rule), `remove_wrapper`, consent text, report lines, `CodexCli` runner trait + process impl, `execute`, and the `read_plan` gather; moves the codex MCP pieces out of `boot.rs` (`CODEX_MCP_*` constants, `classify_codex_entry`, the project-file edit) (DEC-412, DEC-413, DEC-414) |
| `.doctrine/adr/001/layering.toml` | register `codex_mcp = "command"`; it imports only leaves (`fsutil`, `toml_edit`, `std`) — never `boot` or `install`, so callers pass the exec path in (DEC-412) |
| `src/main.rs` | register `codex_mcp` |
| `src/boot.rs` | codex arm of `install_refresh` drops the MCP leg (`mcp: NotApplicable`); remove `install_codex_mcp`, `plan_codex_mcp`, `codex_mcp_entry`, `codex_mcp_fallback_snippet` and their `wire()` lines and `mcp_rel` codex row; `run_install` (boot install) gathers the plan once, asks consent after its `Wire …?` prompt and before `wire`, runs the leg after `wire`, prints `preview` under `--dry-run`, and lets a refused leg stand as success — boot install keeps no stamp (DEC-414) |
| `src/install.rs` | `InstallPlan.codex_user_mcp: Option<CodexUserPlan>` (codex harness only); `run` resolves exec before the plan and `InstallPlan::build` calls the gather closure only for a project install with codex; overview bullet (`render_overview(plan, yes)`, labelled `codex`); consent through `ask_consent` after `confirm_install`; `Leg::CodexUserMcp` in `run_forward_steps` after boot wiring, its `Done` paths as `Change::Changed`, its `Refused` withholding the stamp; `preview` in `print_details` for `--dry-run`; *not* in `regenerate` (DEC-411, DEC-414) |
| `src/commands/cli.rs` | `-y` help on `install` and `boot install` states it grants user-level consent |
| `src/mcp_server/mod.rs`, `src/mcp_server/tools.rs`, `src/mcp_server/protocol.rs` | `ProjectRoot::resolve` (`Doctrine` / `Rootless { bare_root }`) over `root::find_opt` and `root::is_doctrine_project`; `InitializeResult.instructions`; `handle_initialize` takes the root option; `rootless_notice` names both causes; rootless list / call (RV-427 F-11) |
| `src/root.rs` | `find_opt(explicit, markers)`: an explicit path as-is, else the walk from CWD, a miss as `None` rather than an error; `DOCTRINE_DIR`; `is_doctrine_project(root)` — the one spelling rootless `serve --mcp` and the catalogue commands share (DEC-405) |
| `src/catalog/mod.rs` | `run_catalog_scan` / `run_catalog_graph` adopt `root::DOCTRINE_DIR` / `root::is_doctrine_project` instead of their own `.doctrine` literal (DEC-405) |
| `src/fsutil.rs` | `write_atomic_through_link`: resolve the link, then `write_atomic` its target — opt-in and used only by the wrapper removal, so plain `write_atomic` still replaces a link and a repo-committed symlink cannot redirect engine writes out of the tree (DEC-413; RV-427 F-7); `find_on_path(name, path_value)`: first executable match among **absolute** `PATH` elements, used for `codex` and `doctrine` (DEC-412) |
| `tests/common/mod.rs` | `doctrine_cmd` sets `CODEX_HOME` to a non-existent path, so no test can reach a real codex config; codex suites override it (RV-427 F-3) |
| `tests/e2e_codex_install.rs` | rewritten around a fake `codex` on PATH recording argv, temp `HOME`/`CODEX_HOME`; project retirement; consent / decline / `-y`; degradation; a stale user entry left byte-identical by fork regeneration with no `codex` call (DEC-411) |
| `tests/e2e_mcp_server.rs` | rootless handshake outside a project and in a git repo without `.doctrine/`; existing fixtures create `.doctrine/` where they relied on a bare root |
| `tests/e2e_install_flow.rs` | the codex `--yes` flow asserts the degraded line under the safe `CODEX_HOME` |
| `tests/e2e_worktree_provision.rs` | drives the DEC-411 seam: `worktree provision` → `install::regenerate`, which builds no codex user plan; its sibling-worktree and installed-source fixtures move to `common`, shared with the codex suite's byte-identical / no-call test |

`README.md` changes at reconcile, not in a phase (see Revision).

Design-target selectors record exactly these paths.

<!-- doctrine:section sec-leg -->
## The codex user MCP leg

### Plan (pure)

`plan(user: FileRead<'_>, project: FileRead<'_>, facts: &BakeFacts, env: &CodexEnv)
-> CodexUserPlan`. No I/O. The shell reads each file into one of three states —
`Absent`, `Unreadable`, `Text` — because an unreadable file must not read as a
fresh install (DEC-412). It supplies the bake facts (canonical `exec`, the PATH
`doctrine` entry and its canonical form) and the env facts (`CODEX_HOME` value and
whether it names an existing directory, `HOME`, `codex` on PATH). The plan keeps
what each step resolved, so a late degradation still names the file and the
command it reached.

0. **Applicability.** Only a project install (`plan.is_project_install`:
   not `--global`, not `--only-memory`) with the codex harness builds this plan
   (RV-427 F-10).
1. **Destination** (DEC-408, DEC-412). Pure `destination(&CodexEnv)`, which the
   shell calls before reading the user file and `plan` reuses. `CODEX_HOME` set,
   absolute and existing → `$CODEX_HOME/config.toml`; set but empty, relative or
   non-existent → `Degraded(BadCodexHome)` (codex's handling of each is unprobed,
   RV-427 F-9; existence is a supplied fact). Unset → `$HOME/.codex/config.toml`;
   `HOME` unset, empty or relative → `Degraded(NoHome)` — a relative anchor
   resolves against the CWD, which inside a project is the project file read as
   the user entry (DEC-414). The file arrives as a `FileRead`: absent → `Absent`;
   unreadable → `Degraded(UserUnreadable)`; present but not valid TOML →
   `Degraded(UserUnparseable)` (RV-427 F-4). The plan always carries the resolved
   path for the report.
2. **Bake rule** (DEC-404, RV-427 F-8/F-18). Bake only the doctrine the user gets
   by typing `doctrine`. The shell looks `doctrine` up on the install-time
   `PATH` (`find_on_path(name, path_value)` over the supplied PATH value: first
   executable match among **absolute** elements, split with
   `std::env::split_paths`; relative and empty elements are skipped, so the baked
   command is always absolute — RV-427 F-19) and canonicalises both it and
   `exec`:
   - same file → bake the **PATH entry's own path**, not resolved through symlinks
     (e.g. `~/.nix-profile/bin/doctrine`, not its `/nix/store` target;
     mitigates RSK-234);
   - different file, or no `doctrine` on PATH → `Degraded(ExecNotOnPath)`: a dev
     build (`./target/debug/doctrine` on any checkout, fork or fixture) or an
     installed binary run by absolute path. The report names both paths;
   - `exec` cannot be resolved → `Degraded(ExecUnresolved)`.

   The install-time PATH is not guaranteed to be the user's interactive one: an
   agent or sandbox may run install with a constructed PATH. The outcome is still
   safe. The bake is an absolute path to the running binary, shown in the prompt and
   report, and a wrong choice is repaired by rerunning install from the intended
   binary (RV-427 F-20). EVD-032's failure is the codex daemon's PATH at *launch*,
   which the absolute bake sidesteps.
3. **User entry** (DEC-406), classified by table name:
   - absent → `Add`;
   - a table with `command` a string and `args` a string array →
     `Current` when command == the bake, args == `["serve","--mcp"]` and
     `enabled` is not `false` (every other key survives untouched); else
     `Replace { old, dropped }` (a disabled entry is replaced and re-enabled,
     which the prompt states; RV-427 F-5);
   - otherwise (not a table, a `url` transport, wrong types) →
     `Degraded(UserMalformed)`.

   `Replace` carries `dropped`: every key other than `command`/`args`, named the
   way the prompt and report show it (EVD-033: `mcp add` drops them). `Add`
   carries nothing — there is no entry to lose (DEC-412).
4. **Project entry** (DEC-406, form-classified):

   | project `[mcp_servers.doctrine]` | class | effect |
   |---|---|---|
   | absent | — | nothing |
   | `command = "sh"`, `args = ["-c", line]`, line in `CODEX_MCP_WRAPPER_FORMS` — whatever other keys it carries | `OwnedWrapper` | removed with the user write; extra keys listed as removed (RV-427 F-2) |
   | `command` contains `/` (absolute or relative), or a `url` that is a non-empty string | `Override` | untouched; reported; not incomplete |
   | `command` is a bare name | `PathShadow` | untouched; warned; holds incomplete |
   | empty `command`, an empty or non-string `url`, wrong types, not a table | `Malformed` | untouched; cause reported; holds incomplete |
   | unparseable project file | `Unreadable` | untouched; disclosed; holds incomplete |

   Wrapper recognition is table-level `is_wrapper_form` — `command` is `sh`,
   `args` is exactly `["-c", line]` with the line `normalise`d into
   `CODEX_MCP_WRAPPER_FORMS` — one rule shared by this classifier and the removal,
   so "ours" cannot mean two things (DEC-412). Every extra key is named as
   removed, except doctrine's own `env_vars = ["DOCTRINE_BIN"]`.

5. **Codex CLI.** `codex` absent from PATH, with a user write pending →
   `Degraded(NoCodexCli)`.

The plan *proposes a change* when the user entry is `Add`/`Replace` or the project
entry is `OwnedWrapper`, nothing is degraded, and the project entry does not block:
`PathShadow`, `Malformed` and `Unreadable` each mean execute writes nothing, so
nothing is proposed and no question is asked (DEC-412). `Override`, `Current` and
an absent entry block nothing.

### Consent (DEC-403)

Asked only when the plan proposes a change, after install's `Proceed?` and before
any leg runs. `ask_consent(plan, yes, ask)` holds the policy: no proposal →
`NotAsked`, `-y` → `Granted`, otherwise the whole block goes to
`install::prompt_confirm` (it ends on its own ask line, so there is no
body/question split — DEC-414). `--dry-run` (of `install` and `boot install`)
shows the same block under `Codex user MCP (not asked under --dry-run):` without
asking and never runs the leg. Default No.

```text
Codex: register doctrine's MCP server in your user config?
  file     /home/u/.codex/config.toml  [mcp_servers.doctrine]
  runs     codex mcp add doctrine -- /home/u/.local/share/doctrine/bin/doctrine serve --mcp
  scope    every codex session on this machine, in every project
  replaces doctrine serve --mcp                         (only when replacing)
  drops    enabled = false, env.FOO                     (only when keys are dropped;
                                                         enabled = false means re-enabled)
  removes  the old doctrine wrapper in .codex/config.toml (with env.FOO)  (only when present;
                                                             extra keys it held named
                                                             in " (with …)")
  Codex asks you to trust each new folder before doctrine connects there.
Write user config? [y/N]
```

A removal-only plan (user entry already `Current`, the project wrapper still ours)
writes nothing to the user file, so it asks the narrower question (DEC-412):

```text
Codex: remove doctrine's old project MCP wrapper?
  removes  the old doctrine wrapper in .codex/config.toml (with env.FOO)
Remove it? [y/N]
```

`boot install` asks the same question after its own `Wire doctrine boot …?`
prompt and before `wire`, with the same `-y` rule; the leg runs after `wire`
(DEC-414).

### Execute (thin shell)

`execute(plan, consent, project_config, cli, out)` returns `Done(paths)` or
`Refused(reason)`, and install maps that to `LegOutcome` (DEC-414). The project's
`.codex/config.toml` is a parameter: the plan carries no project root (DEC-413).
Consent is `Granted | Declined | NotAsked`; with a proposal, anything but
`Granted` takes the declined path.

- **Declined** → no change in either layer (the wrapper stays, so codex keeps its
  current server). Print the `codex mcp add …` line and the EVD-032 warning.
  `Done` with nothing touched.
- **Consented** → run `codex mcp add …` through an injected `CodexCli` runner.
  On success, remove an `OwnedWrapper` from the project file: `execute` re-reads
  that file and strips only an entry `classify_project` still calls
  `OwnedWrapper` (pure `remove_wrapper`, boot's `toml_edit` idiom), so a file
  changed since the plan is refused, never removed on a guess. An implicit
  `mcp_servers` table goes with its last server, a user-written one stays, and the
  file is never deleted (DEC-413). The write is
  `fsutil::write_atomic_through_link`, opt-in here alone: a linked
  `.codex/config.toml` keeps its link, while plain `write_atomic` still replaces
  a link, so a repo-committed symlink cannot redirect engine writes out of the
  tree (DEC-413; RV-427 F-7). `Done` touches the project file only; the user file
  is outside the repo and is never a commit-hint path. `add` fails → `Refused`,
  codex's stderr indented beneath the line, project file untouched. `add`
  succeeds but the wrapper removal fails → `Refused`, with "user entry written;
  the old project wrapper still overrides it here; rerun install", the cause
  indented beneath it and the daemon hint (a rerun sees `Current` +
  `OwnedWrapper` and asks again; RV-427 F-6). A removal-only failure reports
  `could not remove the old doctrine wrapper from …` with the cause and no hint
  (DEC-413).
- **Degraded** → no writes. Print `codex: user MCP registration skipped — <cause>`,
  with the `run:` line only when a bake resolved (DEC-412). `LegOutcome::Done`
  with nothing touched: degradation is an environment fact (no codex CLI in the
  jail, no home), not an unfinished install, so the version stamp still records
  (RV-427 F-1; DEC-408).
- **PathShadow / Malformed / Unreadable** (project entry) → no writes. Print the
  cause only — no snippet: a manual user registration would still be shadowed by
  the project entry here (DEC-413). `LegOutcome::Refused`: install continues, and
  the closing line is `install incomplete: codex user MCP; version not recorded`
  (the existing `incomplete_line` format).
- **Current, no project change** → `Done`, one "current" line.
- **Override** never blocks the user write. It is reported alongside it.

The daemon hint follows every write — a user-entry write or a project-wrapper
retirement, since removing the project entry changes codex's effective config too:
an already-running codex daemon keeps its old config until restarted (EVD-032).

<!-- doctrine:section sec-model -->
## System model

```mermaid
flowchart LR
  subgraph machine
    U["user config<br/>$CODEX_HOME/config.toml<br/>[mcp_servers.doctrine]<br/>doctrine's default slot"]
  end
  subgraph project
    P[".codex/config.toml<br/>[mcp_servers.doctrine]?<br/>user's override slot"]
  end
  D["codex daemon"] -->|"project entry if present,<br/>else user entry"| S["doctrine serve --mcp"]
  P -.->|wins when present| D
  U -.-> D
  I["doctrine install<br/>(codex harness)"] -->|"read (parse file)"| U
  I -->|"write: codex mcp add"| U
  I -->|"read; remove owned wrapper"| P
```

Two layers with different owners:

| layer | role | doctrine reads | doctrine writes |
|---|---|---|---|
| user | doctrine's default slot (DEC-406) | parses the file directly; `codex mcp get` shows the *effective* entry, not the user layer (ASM-015 invalidated) | `codex mcp add doctrine -- <exec> serve --mcp` (EVD-033), after consent |
| project | the user's override slot | parses `.codex/config.toml` (existing code) | removes only an owned wrapper (`CODEX_MCP_WRAPPER_FORMS`), after consent |

One new install leg, **codex user MCP**, owns both layers' MCP handling. It runs
only from an explicit `doctrine install` or `doctrine boot install` (DEC-411),
never from worktree regeneration or provisioning. The per-project harness refresh
(`install_refresh`'s codex arm) stops touching MCP entirely. It keeps the codex
hooks.

<!-- doctrine:section sec-problem -->
## Design problem

Interactive codex starts MCP servers from a long-lived app-server daemon whose
PATH is frozen at daemon start (EVD-032). The project-level `sh -c` wrapper that
`boot install` writes today (DEC-323) depends on that PATH, so on a machine where
the daemon's PATH lacks doctrine, codex reports a handshake failure. Only an
absolute `command` survives, and POL-002 forbids writing one into a file a client
repo may track.

The user-level codex config is not such a file. This design moves doctrine's codex
MCP registration there, with the user's consent, through codex's own CLI. It
retires doctrine's project-level wrapper, which would otherwise shadow the
user-level entry, since codex prefers the project layer. It keeps the project
layer free for per-project overrides. It also makes `serve --mcp` harmless in the
directories a machine-wide entry now reaches.

Governing records: DEC-403 (user level, with consent), DEC-404 (bake guard),
DEC-405 (rootless serve), DEC-406 (ownership per layer), DEC-407 (report),
DEC-408 (degradation), DEC-409 (`DOCTRINE_BIN` dropped), DEC-410 (out-of-repo
write rule), DEC-411 (explicit install only), EVD-032, EVD-033, EVD-034
(invalidates ASM-014).

<!-- doctrine:section sec-report -->
## Report and dry-run

The leg prints its own lines (DEC-407 as amended); `RefreshReport.mcp` is
`NotApplicable` for codex and `mcp_rel` loses its codex row. Examples:

User write plus retired wrapper:

```text
codex: registered doctrine MCP server in /home/u/.codex/config.toml
  command /home/u/.local/share/doctrine/bin/doctrine serve --mcp
codex: removed the old doctrine wrapper from .codex/config.toml
codex: a running codex app-server keeps its old config until restarted
```

Replace, keys dropped:

```text
codex: replaced doctrine MCP command in /home/u/.codex/config.toml
  was     doctrine serve --mcp
  now     /home/u/.local/share/doctrine/bin/doctrine serve --mcp
  dropped enabled = false
```

Project override in effect (user write still done or current):

```text
codex: project override in effect: ./bin/doctrine (.codex/config.toml) — codex uses it here instead of the user entry
```

PATH-dependent shadow:

```text
codex: .codex/config.toml has a doctrine entry with command "doctrine"; it overrides the user entry here
  and needs doctrine on the codex daemon's PATH, which may fail (EVD-032). Use an absolute path, or remove it.
install incomplete: codex user MCP; version not recorded
```

Degraded (the install still completes):

```text
codex: user MCP registration skipped — codex is not on PATH
  run: codex mcp add doctrine -- /home/u/.local/share/doctrine/bin/doctrine serve --mcp
```

Each degradation names its own cause: `CODEX_HOME is not an existing absolute
directory`, `HOME is unset or not an absolute path`, `<file> could not be read`,
`<file> is not valid TOML`, `<file> has a doctrine entry doctrine cannot
interpret`, `this doctrine (<exec>) is not the doctrine on PATH (<on_path>)` /
`is not on PATH`, `the running doctrine's path could not be resolved`. The
`run:` line appears only when a bake resolved (DEC-412).

Declined:

```text
codex: user MCP registration declined; nothing changed
  interactive codex may fail to start doctrine if its daemon's PATH lacks doctrine (EVD-032)
  to register later: codex mcp add doctrine -- /home/u/.local/share/doctrine/bin/doctrine serve --mcp
```

Already current, nothing to do in either layer:

```text
codex: doctrine MCP server already registered in /home/u/.codex/config.toml
```

A write that did not land (DEC-413):

```text
codex: codex mcp add failed: <codex's stderr, later lines indented two spaces>
codex: could not remove the old doctrine wrapper from .codex/config.toml: <cause>
codex: .codex/config.toml has a doctrine entry doctrine cannot interpret; it overrides the user entry here. Fix or remove it.
codex: .codex/config.toml could not be read or parsed; fix it and rerun install.
```

The report states what was written, never that codex has loaded it (POL-003).

**Dry-run** builds the plan through the one gather — `read_plan`: the environment
through an injected lookup, both files, `PATH` and the canonical paths from disk
(DEC-414) — and invokes no `codex`. It prints the consent block under
`Codex user MCP (not asked under --dry-run):` for `install` and `boot install`,
and never writes either layer. The overview gains one bullet when the plan
proposes a change: `codex  register doctrine's MCP server in your user codex
config`, followed by "(asks first)" interactively or "(-y: consenting)" under
`-y` (RV-427 F-12).

<!-- doctrine:section sec-revision -->
## Revision and reconcile

At reconcile, one REV against SPEC-011:

- **REQ-484** rewritten. Install, with consent, registers doctrine in the codex
  user config via `codex mcp add` with an absolute command. It owns the user table
  by name and removes doctrine's project-level wrapper. It classifies other
  project entries by form, discloses and degrades rather than failing, runs only
  on explicit install, and reports what was written, never activation.
- A requirement for rootless `serve --mcp` (DEC-405), placed where `serve --mcp`'s
  behaviour is specified (to locate at reconcile).

Supersessions, recorded with `doctrine supersede` at reconcile: DEC-323 and
DEC-332 by DEC-406, and DEC-325 by DEC-407.

README, also at reconcile:

- the user-level entry and its scope;
- the consent prompt, and that `-y` grants it;
- the project-level override slot and how to use it;
- codex's per-folder trust prompt;
- the codex CLI as a declared dependency of the codex harness;
- removal of the `sh` dependency declaration.

<!-- doctrine:section sec-risks -->
## Risks and residuals

| risk | mitigation / residual |
|---|---|
| A hand-written user entry is replaced | consent prompt shows old → new and dropped keys; `-y` prints them; overrides belong in the project layer |
| A dev build is baked machine-wide | DEC-404: only the PATH `doctrine` is baked; dev builds degrade with the command printed |
| `doctrine` not on the user's PATH | degrade with the reason and the pasteable command (fail safe) |
| A `/nix/store` path breaks after garbage collection | the MCP entry bakes the PATH (profile) path; codex hooks still bake `current_exe` (RSK-234) |
| `codex mcp add` flags or semantics change | read path is independent; runner failure → `Refused` with codex stderr; EVD-033 pins 0.159.1 behaviour |
| Codex never shows the rootless `instructions` | EVD-034 invalidates ASM-014: the empty tool list is the only signal the user sees; `tools/list` is `[]` regardless, which is what DEC-405 relies on |
| Regeneration writes user config | DEC-411; e2e asserts regeneration leaves a temp user config byte-identical |
| Jail installs have no codex / a different home | degrade with the pasteable line; install still completes and stamps |
| Tests reach the developer's real codex config | `doctrine_cmd` points `CODEX_HOME` at a non-existent path (RV-427 F-3) |
| A `PathShadow` keeps install incomplete | intended, with the cause named (partially resolves ISS-528); an absolute override does not |
| Codex behaviour beyond tested versions | probes cover CLI 0.159.1 and daemon 0.163.0/0.164.0 |

<!-- doctrine:section sec-serve -->
## Rootless serve (DEC-405)

`mcp_server::serve` resolves `ProjectRoot::resolve` over `root::find_opt` and
`root::is_doctrine_project`: a discovered root carrying `.doctrine/` gives
`Doctrine(root)`; no root, or a root without `.doctrine/`, gives
`Rootless { bare_root }`. Rootless:

- `initialize` succeeds; its `instructions` say `no doctrine project here (no
  project root)` when the walk found nothing, or `no doctrine project here (no
  .doctrine/ under <root>)` when a root was found without `.doctrine/`. Codex
  never displays MCP initialize instructions (EVD-034, invalidating ASM-014), so
  the empty tool list is the only signal a user sees.
- `tools/list` returns `[]`.
- `tools/call` returns a JSON-RPC error carrying the same `rootless_notice` text
  (defensive; no tool is advertised).

With `Doctrine(root)`, behaviour is unchanged. `--path` keeps its meaning: an
explicit root without `.doctrine/` is also rootless.

<!-- doctrine:section sec-verification -->
## Verification

Unit (pure, `codex_mcp`):

- `plan` matrix:
  - user entry: absent / current / replace / replace with dropped keys / malformed;
  - project entry: absent / owned wrapper / absolute override / relative override /
    bare-name shadow / empty command / wrong types / unparseable;
  - user entry also: disabled (`enabled = false`) / `url` transport;
  - project entry also: wrapper with extra keys / `url` transport;
  - destination: `CODEX_HOME` unset / empty / relative / missing; no `HOME`;
    user file unreadable / unparseable;
  - applicability: `--global` / `--only-memory` build no plan;
  - bake rule (given PATH facts): PATH `doctrine` is the running binary → baked
    as the PATH path; PATH entry is a symlink → the link path, not the target;
    a dev build not on PATH → refused; no `doctrine` on PATH → refused; a
    relative or empty PATH element holding `doctrine` is skipped; exec
    unresolved → degraded;
  - `codex` absent.
- Consent text and report lines per outcome (constants, STD-001).

Shell (`execute` with a fake `CodexCli`):

- consented: one `add` with the exact argv, then wrapper removed;
- `add` fails: `Refused`, project untouched;
- `add` succeeds, wrapper removal fails: `Refused` with the rerun line;
- symlinked project config: link survives the removal;
- declined: zero calls, nothing written;
- degraded: zero calls, `Done`, the version stamp is recorded.

E2E (binary-driven, fake `codex` on PATH, temp `HOME`/`CODEX_HOME`):

- `install -y`: argv recorded and the project wrapper removed;
- interactive decline: nothing changed;
- `--dry-run`: no `codex` invocation and no writes;
- `worktree provision` of a fork (→ `install::regenerate`): the user config is
  byte-identical and there is no `codex` call (DEC-411);
- `boot install -y`: same as install;
- the default `doctrine_cmd` environment never invokes `codex` (a fake `codex`
  on PATH records no call).

MCP:

- rootless handshake (no root; git repo without `.doctrine/`): `tools/list` is
  `[]`, `instructions` names the cause;
- a doctrine project is unchanged.

VH:

- interactive codex in this repo connects after `doctrine install`, with a daemon
  whose PATH lacks doctrine;
- VH-2 (EVD-034): codex shows no MCP initialize instructions; `/mcp` lists the
  server as connected with 0 tools, the only user-visible signal DEC-405 relies on.

