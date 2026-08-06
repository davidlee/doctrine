<!-- doctrine:section sec-1 -->
# Activation after retirement

## Current behaviour

A Claude project acquires doctrine's behaviour through three disjoint channels
today, and only one of them is a direct write:

| what | activates via | written by |
|---|---|---|
| ten hook entries across five events | `enabledPlugins` + marketplace registration → `plugins/doctrine/hooks/hooks.json` | Claude Code, from per-user state |
| the memory-sync `SessionStart` hook | direct merge into `.claude/settings.local.json` | `HookSpec::sync` → `install_claude_hook` (`src/corpus.rs:506`) |
| skills | the same directory-source plugin | Claude Code |

`install_refresh`'s Claude arm writes **no** hook at all — `let hook =
RefreshOutcome::None` (`src/boot.rs:1290`). SL-152 PHASE-06 removed the write
because it double-fired against the plugin. The merge core stayed; only the
Claude caller left. That vacated seam is where this slice lands.

## Target behaviour

Two direct-write channels, both project-inspectable, both doctrine's to author:

```
  doctrine install ──┬─→ boot::wire ──→ install_refresh(Claude)
                     │                    ├─ 7 × HookSpec → 11 entries
                     │                    │                ──→ .claude/settings{,.local}.json
                     │                    │                     (scope-selected; sibling swept)
                     │                    ├─ install_baseref (same scope-selected file)
                     │                    └─ install_mcp     (unchanged, .mcp.json)
                     │
                     ├─→ install_skills_direct ─→ .doctrine/skills/<id>   (canonical tree)
                     │                          └→ .claude/skills/<id> ─→ ../../.doctrine/skills/<id>
                     │
                     └─→ install_for_other ────→ npx skills add <repo>   (non-Claude, unchanged)

  doctrine memory sync install ─→ install_claude_hook(HookSpec::sync)  (same scope, same sweep,
                                                                        same announcement)
```

## The counts, stated once

Four numbers describe this change at four altitudes, and conflating them is what
let a verification criterion certify a partial install. Each is fixed here; no
section restates one on its own authority.

| altitude | count | what it counts |
|---|---|---|
| plugin entries | **10** | entries in `plugins/doctrine/hooks/hooks.json` — `SessionStart` 1, `WorktreeCreate` 1, `SubagentStart` 1, `SubagentStop` 1, `PreToolUse` 6 |
| specs | **7** | the `claude_hook_specs` registry — the six replacing plugin entries, plus the already-shipping `sync` |
| settings entries | **11** | the matcher sets summed: 1+1+1+1+1+4+2 |
| printed hook lines | **7** | one `RefreshOutcome` line per spec, beneath one scope-announcement line and up to three sweep rider lines (`sec-3`: removed / unreadable / not attempted are independent) |

Specs and plugin entries do not correspond one-to-one, and `sync` is the whole
reason: it never shipped in the plugin, having been direct-written since SL-018.
It joins the registry because it shares the file, the scope dial and the sweep.

Entries exceed specs because two specs carry multi-element matcher sets — four
for `worktree pretooluse`, two for `memory surface`. **Eleven is the number a
human counts in `/hooks`**, and it is the only one of the four that appears in an
acceptance criterion.

What leaves: doctrine's automated `claude plugin marketplace add` and
`claude plugin install` steps. What stays: `plugins/` as the canonical skill
source, `plugins/doctrine/hooks/hooks.json` as the published plugin's payload,
and `.claude-plugin/marketplace.json` published — the managed-policy escape
hatch `R9` names, installed by hand and documented as such.

## Why the three legs are separable

The hook leg and the skills leg share no code and no file. They are sequenced
together only because both are Claude activation and both are pointless while
the plugin still supplies them. The merge core (`plan_hook` and friends) is
JSON-entry surgery over a settings file; the skills channel is a canonical tree
plus proven-ownership symlinks. They meet only at the install command that runs
both, and at the cutover note that tells a user to take them together.

This matters for phasing: the hook leg can land, be verified live, and be used
before the skills leg starts.

## Code impact

| path | change |
|---|---|
| `src/boot.rs` | `HookSpec` gains an ordered matcher set and holds its args rather than a rendered command; five new specs; one shared ownership-predicate helper; scope resolution, the `CommandForm` axis threaded through `install_hook_to_file` / `plan_hook` / `annotate_fallback` / `fallback_for` to both arms, and the abandoned-scope sweep gated on the write landing; `EvictOutcome` (per spec) plus `SweepReport` (per file); `RefreshReport.hook` becomes a collection; `install_baseref` follows the scope and reports a stranded sibling override; the shared scope-announcement writer; new path/matcher/event constants |
| `src/install_config.rs` | the `ClaudeSettingsScope` enum, its `[install]` key, and the container-level `rename_all` that key needs to bind |
| `src/install.rs` | `reconcile_link` extracted from two byte-identical blocks; the restored direct skills channel; removal of the automated plugin steps and their transitive orphans |
| `src/corpus.rs` | **changed** — `run_sync_install` matches `HookWrite.written` and calls the shared announcement writer, so the scope and the sweep are reported on the routine path too |
| `plugins/doctrine/hooks/hooks.json` | none — it stays the published plugin's payload |
| `install/` | the cutover note, the escape-hatch instructions, the new config key, the command-form note and its POSIX-shell scope boundary |
| `.doctrine/spec/tech/011/` | `REQ-186`, via the REV at reconciliation |

## What this section does not settle

Whether the removal of the automated plugin steps also deletes their helper
functions (`select_marketplace_source`, `marketplace_action`,
`claude_plugin_install`, `enable_key`, `parse_registered_source`,
`refresh_failure_is_fatal`) or retains them behind an opt-in. Carried into the
write-seam section.
<!-- doctrine:section sec-2 -->
# The merge core: an ordered matcher set

Settles `R5` per `DEC-161`. Ownership stays proven by `command` alone;
`HookSpec` carries the matchers.

## The type

```rust
pub(crate) struct HookSpec {
    /// The resolved `current_exe()`, kept unrendered — the program half of the
    /// command depends on the scope, which is resolved per write (sec-3).
    exec: PathBuf,
    /// The fixed argument suffix, e.g. `memory sync`. Also the ownership key.
    args: &'static str,
    is_ours: fn(&str) -> bool,
    event: &'static str,
    /// The matcher tokens this spec's entries carry, in emission order.
    /// One-element for every spec that shipped before SL-250.
    matchers: &'static [&'static str],
}
```

`command` was a rendered `String` before this slice. It becomes `exec` plus
`args` because the command's program half now depends on the file being written
— see *The command form* below — and a spec is built once by `claude_hook_specs`
while that is resolved per write. Rendering is a function of the two:

```rust
/// Which program half a file gets. THE MERGE CORE'S AXIS — deliberately not
/// `ClaudeSettingsScope`, which is one thing that selects it (`sec-3`).
#[derive(Clone, Copy)]
enum CommandForm {
    /// `<abs exec> <args>` — for a gitignored file (`baked ⟺ gitignored`, SL-195 D2).
    Baked,
    /// `${DOCTRINE_BIN:-doctrine} <args>` — for a tracked one (POL-002).
    Portable,
}

fn command_for(spec: &HookSpec, form: CommandForm) -> String {
    match form {
        CommandForm::Portable => format!("{PORTABLE_EXEC} {}", spec.args),
        CommandForm::Baked => format!("{} {}", spec.exec.display(), spec.args),
    }
}
```

**Why the axis is the form and not the scope.** `plan_hook` is the *shared*
merge core: `install_codex_hook` writes `.codex/hooks.json` through it
(`src/boot.rs:1628-1634`), and Codex has no `ClaudeSettingsScope`. Threading a
Claude-settings-file enum through the Codex arm purely to select "baked" would
make the type stop denoting what its name and doc-comment say it denotes — the
same boundary error `sec-3` avoids one section on, in the other direction, and
on the same authority (ADR-001: a shared core should not acquire a caller's
domain concept).

The underlying invariant was never Claude-specific. `.codex/hooks.json` is
gitignored (`.gitignore:9`, `/.codex`), so `baked ⟺ gitignored` already holds
there for the identical reason it holds for `settings.local.json`. So Codex
answers `Baked` **honestly**, on its own file's tracking status, rather than
being handed a scope it does not have. The two Claude scopes map on:

| caller | file | tracked? | `CommandForm` |
|---|---|---|---|
| Claude, `Project` | `.claude/settings.json` | yes | `Portable` |
| Claude, `Local` | `.claude/settings.local.json` | no | `Baked` |
| Codex | `.codex/hooks.json` | no | `Baked` |

`ClaudeSettingsScope::command_form()` is the one-line mapping, and it lives with
the scope in `install_config`; `CommandForm` lives in `boot.rs` with the core
that consumes it.

**The wire that carries it.** The form is decided at the top and consumed at the
bottom, so every function between them widens by one parameter. Naming them,
because an unstated wire is how a specified rendering function ends up
reverse-deriving its input from a path:

| function | today | after |
|---|---|---|
| `install_hook_to_file` (`:1595`) | `(root, rel_path, spec, dry_run)` | `+ form` |
| `plan_hook` (`:1153`) | `(existing_json, spec)` | `+ form` |
| `annotate_fallback` (`:1578`) | `(outcome, hook_file, spec)` | `+ form` |
| `fallback_for` (`:1587`) | `(spec)` | `+ form` |
| `install_codex_hook` (`:1628`) | — | passes `CommandForm::Baked` |
| `install_claude_hook` (`:1617`) | — | passes `scope.command_form()` |

`install_hook_to_file` is the join: it is the sole path from either arm's
installer to `plan_hook`, and it also calls `annotate_fallback`. Deriving the
form back out of `rel_path` inside it is the rejected alternative — it would
make the merge core reverse-engineer a decision its caller already made, and
would re-couple the core to a path constant per harness.

`src/corpus.rs:520` is the sixth site: its `PrintedFallback` arm calls
`fallback_for(&spec)` and now needs the form too. It has it —
`write.scope.command_form()` off the `HookWrite` this section introduces.

`is_ours` stays a bare `fn` pointer rather than becoming a closure: the four new
predicates differ only in their argument string, and a `Box<dyn Fn>` would
change `HookSpec`'s shape for every existing caller to buy nothing.

## The invariant, restated

Before: *exactly one canonical, doctrine-sole entry per spec, per file.*
After: *exactly the canonical entry **set**, in matcher order, each
doctrine-sole, per file.*

Entry identity becomes the set. A hand-edit that deletes one of the four
`worktree pretooluse` entries is healed on the next install, because the owned
set no longer matches and the whole set is rewritten. That stays inside
never-clobber: only doctrine-owned hooks are dropped.

## `desired_entry` → `desired_entries`

```rust
fn desired_entries(spec: &HookSpec, form: CommandForm) -> Vec<Value> {
    let command = command_for(spec, form);
    spec.matchers
        .iter()
        .map(|matcher| serde_json::json!({
            "matcher": matcher,
            "hooks": [ { "type": "command", "command": command } ],
        }))
        .collect()
}
```

`fallback_for` takes the form alongside the spec and renders the whole set, so
the manual-repair snippet a malformed
settings file prints is complete rather than one sixth of the answer — and
prints the form that file would actually have received.

## `plan_hook`'s normalize

The no-write short-circuit generalises from a single-element slice pattern to a
positional match over the owned set:

```rust
let command = command_for(spec, form);
let owned = owned_positions(arr, spec.is_ours);
let canonical_set = owned.len() == spec.matchers.len()
    && owned.iter().zip(spec.matchers).all(|(&(ei, hi), matcher)| {
        entry_is_canonical(arr, ei, hi, &command, matcher) && hook_is_sole(arr, ei)
    });
if canonical_set {
    return HookPlan { outcome: RefreshOutcome::None, new_json: None };
}
```

`entry_is_canonical` takes the command and the expected matcher rather than the
whole spec, because the matcher it compares against is now positional and the
command is now form-dependent. `owned_positions` returns array order, so the zip
requires the owned entries to appear **in matcher order**. They always do after a
doctrine write; a reordering hand-edit falls through to the rewrite, which is the
healing property, not a defect.

The write branch inserts the set contiguously at the first owned slot:

```rust
Some(&(first, _)) => {
    let survives = entry_has_foreign_hook(arr, first, spec.is_ours);
    drop_owned_hooks(arr, spec.is_ours);
    let ins = (first + usize::from(survives)).min(arr.len());
    for (k, entry) in desired_entries(spec, form).into_iter().enumerate() {
        arr.insert(ins + k, entry);
    }
    RefreshOutcome::Refreshed(command)
}
```

`ins + k` is always in bounds: `ins <= len` before the loop, and the array has
grown by exactly `k` by the time the `k`-th insert runs. The `None` branch
appends the set at the tail. Position relative to foreign entries is preserved
the same way the current single insert preserves it.

`drop_owned_hooks`, `owned_positions`, `entry_has_foreign_hook`, `hook_is_sole`
and `hook_array_mut` are **unchanged**. The matcher set is a strict
generalisation with N=1 as the existing case, which is what makes the
behaviour-preservation gate hold by construction.

## The command form

`DEC-163` makes the default scope `.claude/settings.json` — a file the design
itself describes as committed and travelling with the repo. The hook command
bakes `current_exe()`, a host absolute path. **That combination is a POL-002
breach, and it is one SL-195 already ruled on.**

SL-195 fixed the same defect on `.mcp.json` and left an invariant behind:
**baked ⟺ gitignored** (its design D2). Claude hooks were left baked *because*
they were gitignored, and `.mcp.json` was named "the sole committed POL-002
breach", closed by writing a portable literal instead of a path. Its acceptance
criterion is flat: *no absolute host path in any tracked file*. Flipping the hook
default to a committed file without touching the command form would reintroduce
the breach SL-195 closed, in the same installer, one key over.

So the invariant is kept rather than broken, and the fix rides the seam SL-195
already cut:

```rust
/// The portable doctrine invocation — shell-expanded at hook execution.
/// Shared with the `.mcp.json` entry, which has carried it since SL-195:
/// one literal, one constant (STD-001).
const PORTABLE_EXEC: &str = "${DOCTRINE_BIN:-doctrine}";
```

`MCP_COMMAND` (`src/boot.rs:549`) holds this literal today. It is renamed to
`PORTABLE_EXEC` rather than duplicated — same string, same meaning, now two
surfaces.

| scope | file | command | why |
|---|---|---|---|
| `Project` | `.claude/settings.json` (committed) | `${DOCTRINE_BIN:-doctrine} <args>` | no host path in a tracked file |
| `Local` | `.claude/settings.local.json` (gitignored) | `<abs exec> <args>` | baked ⟺ gitignored, unchanged from today |

**This works because hooks are shell form.** A command hook with no `args` key
has its `command` string passed to `sh -c`, which "tokenizes the string, expands
variables" (`hooks.md:341`). Doctrine emits no `args` key, so `${DOCTRINE_BIN:-doctrine}`
expands at execution. This is not inference from the `.mcp.json` mechanism, which
is different — Claude Code expands that one itself at load (`mcp.md:384`). It is
the empirically proven case: `plugins/doctrine/hooks/hooks.json` has shipped every
one of its ten entries in exactly this form, and those hooks fire.

**Stated scope: POSIX shells.** That expansion is a property of the *shell*, not
of hooks, and the cited paragraph says which shell: "`sh -c` on macOS and Linux,
Git Bash on Windows, or PowerShell when Git Bash isn't installed"
(`hooks.md:341`). `${VAR:-default}` is POSIX parameter expansion — `sh` and Git
Bash provide it, PowerShell does not, where `${…}` delimits a variable *name* and
the token resolves as one literally called `DOCTRINE_BIN:-doctrine`, unset, so
the command degrades to empty. It would degrade *silently*, since `PreToolUse`
hooks fail open (`mem.fact.claude.pretooluse-hook-fail-open`, `DEC-162`).

**Doctrine does not target Windows** (user ruling, 2026-08-06; the repo is
NixOS / bubblewrap, flake-defined per `AGENTS.md`). So this is a scope boundary
stated once, not a portability problem to engineer around — but it is stated,
because the empirical control above was gathered on a POSIX host and proves
`sh -c` expands the literal, not the unconditional claim.

**And the shared literal's two consumers do not carry the same guarantee.**
SL-195's `.mcp.json` use is expanded by the *client* at load (`mcp.md:384`,
`src/boot.rs:544-549`) and is therefore portable wherever Claude Code runs; the
hook use is expanded by whichever shell the platform picks. One constant, one
string, two mechanisms. `PORTABLE_EXEC` names the *form* — it does not promise
that the form's portability is the same on both surfaces, and this is the
paragraph where a later reader will look for that.

**Ownership owns both forms, always, in either file.** `is_doctrine_program`
gains one arm:

```rust
fn is_doctrine_program(program: &str) -> bool {
    let p = program.trim_end();
    let p = p.strip_suffix(" (deleted)").unwrap_or(p).trim_end();
    p == PORTABLE_EXEC || Path::new(p).file_name() == Some(OsStr::new("doctrine"))
}
```

One arm, and every predicate inherits it — they all funnel through this function.
It is a strict widening: `Path::new("${DOCTRINE_BIN:-doctrine}").file_name()`
yields the whole token, never `doctrine`, so the two arms cannot collide and no
foreign command becomes ours.

That arm is what makes the scope switch heal rather than orphan. An abspath entry
found in the file we are now writing is **owned but not canonical**, so it is
rewritten to the portable form; an abspath entry in the file we are abandoning is
**owned**, so the sweep evicts it. This is precisely `is_doctrine_mcp_entry`'s
posture (`src/boot.rs:1475-1480`), which owns the portable form *and* a legacy
abspath so SL-195's migration lands without double-registering. The same reasoning
applies here for the same reason, and the healing property `DEC-161` bought with
command-only ownership is what carries it.

One incidental gain. Shell form tokenizes, so a baked exec path containing a space
is already broken at execution today — the ownership predicates go to some trouble
to tolerate space-bearing program paths that `sh -c` would split anyway. The
portable form removes that hazard on the committed scope. It is not fixed for
`Local` and this slice does not fix it; noted so a later reader does not mistake
the predicates' tolerance for an execution-time guarantee.

## The seven specs

| event | command args | matchers | predicate |
|---|---|---|---|
| `SessionStart` | `prompt resolve --role orchestrator` | `startup\|clear` | `is_doctrine_emit_command` (exists) |
| `SessionStart` | `memory sync` | `startup\|clear` | `is_doctrine_sync_command` (exists) |
| `WorktreeCreate` | `worktree create-fork` | `*` | `is_doctrine_create_fork_command` (exists) |
| `SubagentStart` | `worktree nominate` | `dispatch-orchestrator` | new |
| `SubagentStop` | `worktree denominate` | `dispatch-orchestrator` | new |
| `PreToolUse` | `worktree pretooluse` | `Bash`, `Edit\|Write`, `Agent`, `Workflow` | new |
| `PreToolUse` | `memory surface` | `Read\|Edit\|Write`, `Bash` | new |

Seven rows, seven specs: the six that replace plugin entries plus the
already-shipping `sync`. Against the ledger in `sec-1` — the **ten** plugin
entries collapse onto those **six** specs as `DEC-162` records, `sync` rides
along because it shares the file, the scope dial and the sweep, and the seven
specs' matcher sets emit **eleven** settings entries.

The `SessionStart` emit spec is the **canonical current** command on the
canonical matcher, not the plugin's stale `boot --emit` on `*`. It is
`boot_emit`'s hook, not `HookSpec::boot`'s: `is_doctrine_boot_command` requires
the trailing arg to be literally `boot` (`src/boot.rs:891`), which `boot --emit`
fails, while `is_doctrine_emit_command` already recognises `LEGACY_EMIT_ARGS`
(`:934`). So writing the canonical form retires the stale entry by the healing
property, with no migration step. `HookSpec::boot` stays dead.

The Codex arm already ships an emit spec on `SESSION_MATCHER_CODEX`. Rather than
a second constructor, `boot_emit` takes its matcher set:

```rust
fn boot_emit(exec: &Path, matchers: &'static [&'static str]) -> Self
```

Two callers, two constants. The two share `is_doctrine_emit_command`, which is
safe because they write different files (`.codex/hooks.json` vs the Claude
settings file) and ownership is only ever evaluated within one file.

## The ownership predicates

Four of the five existing predicates are the same shape — strip a fixed arg
suffix, then a space, then check the program's file name. That shape is written
out four times today and would be written out four more. One helper:

```rust
/// Whether `cmd` is `<doctrine> <args>` — the shared suffix-strip ownership
/// shape. Poison-tolerant via `is_doctrine_program`.
fn is_doctrine_command(cmd: &str, args: &str) -> bool {
    cmd.trim()
        .strip_suffix(args)
        .and_then(|program| program.strip_suffix(' '))
        .is_some_and(is_doctrine_program)
}
```

Each predicate becomes a one-liner over it, preserving the `fn(&str) -> bool`
signature:

```rust
fn is_doctrine_nominate_command(cmd: &str) -> bool {
    is_doctrine_command(cmd, WORKTREE_NOMINATE_ARGS)
}
```

`is_doctrine_sync_command`, `is_doctrine_create_fork_command` and
`is_doctrine_emit_command` collapse onto it identically —
`is_doctrine_emit_command` as `[RESOLVE_EMIT_ARGS, LEGACY_EMIT_ARGS].iter()
.any(|args| is_doctrine_command(cmd, args))`.

`is_doctrine_boot_command` is deliberately **left alone**, and the reason is
sharper than "behaviour-preservation risk on dead code". Its `rsplit_once(char::is_whitespace)`
form (`src/boot.rs:891-896`) splits on the last whitespace character of *any*
kind, while the suffix-strip form above requires a literal single space. They
diverge: for `/x/doctrine<TAB>boot` the incumbent returns true and the helper
returns false. Same for a newline. They agree on repeated spaces only because
`is_doctrine_program` trims (`:903`).

So this is not an inconsistency being tolerated — it is **the one predicate whose
semantics would actually change**. The four the design does collapse are already
the strict single-space form today (`is_doctrine_sync_command` at `:922` among
them), which is why collapsing those is a refactor and collapsing this one would
be a behaviour change. That it also guards a spec nothing ships
(`HookSpec::boot` keeps its `expect(dead_code)`, `DEC-162`) makes the decision
free, but the separator divergence is the actual reason.

## Constants (STD-001)

```rust
const SETTINGS_LOCAL_REL: &str = ".claude/settings.local.json";  // was SETTINGS_REL
const SETTINGS_PROJECT_REL: &str = ".claude/settings.json";

const PORTABLE_EXEC: &str = "${DOCTRINE_BIN:-doctrine}";         // was MCP_COMMAND

const EVENT_SESSION_START: &str = "SessionStart";
const EVENT_WORKTREE_CREATE: &str = "WorktreeCreate";
const EVENT_SUBAGENT_START: &str = "SubagentStart";
const EVENT_SUBAGENT_STOP: &str = "SubagentStop";
const EVENT_PRE_TOOL_USE: &str = "PreToolUse";

const SESSION_MATCHERS: &[&str] = &[SESSION_MATCHER];
const SESSION_MATCHERS_CODEX: &[&str] = &[SESSION_MATCHER_CODEX];
const WORKTREE_CREATE_MATCHERS: &[&str] = &[WORKTREE_CREATE_MATCHER];
const SUBAGENT_MATCHERS: &[&str] = &["dispatch-orchestrator"];
const PRETOOLUSE_MATCHERS_WORKTREE: &[&str] = &["Bash", "Edit|Write", "Agent", "Workflow"];
const PRETOOLUSE_MATCHERS_SURFACE: &[&str] = &["Read|Edit|Write", "Bash"];
```

Renaming `SETTINGS_REL` is mechanical and compiler-checked, and it is worth the
test-file churn: once doctrine writes either of two settings files, a constant
named "the settings file" is an ambiguity that will be misread. **The rename is
not the whole change to those tests** — the default scope also flips, which
rewrites assertions rather than identifiers. `sec-7` enumerates that separately,
because collapsing the two is what would blind the behaviour-preservation gate.

`MCP_COMMAND` → `PORTABLE_EXEC` is the same kind of rename for the same reason:
once the literal serves hooks as well as `.mcp.json`, a constant named for one
consumer misdescribes it. Its three existing consumers — `desired_mcp_entry`,
the `plan_mcp` comparator and `is_doctrine_mcp_entry` — are unaffected in
behaviour.

The event names are literals inline today. Naming them is STD-001's ask and
removes the chance of a typo producing a silently-inert hook under a
misspelled event key.
<!-- doctrine:section sec-3 -->
# Scope selection and the abandoned-scope sweep

`DEC-163` (a sticky key, no flag) and `DEC-164` (writing one scope evicts the
other) are one mechanism: doctrine writes exactly one of two files, and the
merge core's one-canonical-entry-set invariant extends from *within a file* to
*across the pair doctrine writes*.

## The key

```toml
# .doctrine/doctrine.toml
[install]
claude-settings-scope = "project"   # or "local"
```

Read through the existing `load_doctrine_toml` seam; `[install]` already exists
(`InstallConfig`, `src/install_config.rs`) carrying `repo`, so this is a second
field on a live table rather than a new config surface.

**Two attributes are needed, not one, and only one of them is on the enum.**

```rust
// src/install_config.rs — the CONTAINER acquires the kebab-case rename.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", default)]   // `default` is already there; `rename_all` is new
pub(crate) struct InstallConfig {
    pub(crate) repo: String,
    pub(crate) claude_settings_scope: ClaudeSettingsScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]            // governs the VALUES: `project` / `local`
pub(crate) enum ClaudeSettingsScope {
    /// `.claude/settings.json` — committed, reviewable, travels with the repo.
    #[default]
    Project,
    /// `.claude/settings.local.json` — private to one checkout.
    Local,
}
```

`rename_all` on the enum renames its **variants**, so it is what makes the values
`project` and `local` parse. It does nothing for the **field name**. Without the
container attribute the field binds from `claude_settings_scope`, and the
documented spelling is silently discarded — `InstallConfig` carries
`#[serde(default)]` and neither it nor `DoctrineToml` sets `deny_unknown_fields`,
so an unknown key is not an error and not a warning.

That failure mode is this slice's own, and `DEC-164` makes it worse than an
ignored key usually is: the user writes `claude-settings-scope = "local"`, the
field falls to its `Project` default, doctrine writes the project file, **evicts
their local entries**, and the announcement line truthfully reports a target they
did not choose.

`InstallConfig` has never needed a container `rename_all` because its one field,
`repo`, is a single word. The precedent to follow is `DispatchConfig`
(`src/dispatch_config.rs:50-51`), which carries exactly this pair and is what
makes `[dispatch] preferred-subprocess-harness` bind — the table `DEC-163` names
as its model. `:93` shows the per-field alternative if a future key needs it.

An absent key and an absent `[install]` table both yield `Project`, which is the
flip `OQ-1` settled.

**Layering.** The enum lives in `install_config` — it is config vocabulary, and
that module is a pure leaf with no IO. The mapping onto a path stays in
`boot.rs`, which owns the path constants:

```rust
const fn settings_rel(scope: ClaudeSettingsScope) -> &'static str {
    match scope {
        ClaudeSettingsScope::Project => SETTINGS_PROJECT_REL,
        ClaudeSettingsScope::Local => SETTINGS_LOCAL_REL,
    }
}
```

Putting the path in `install_config` would give a pure leaf domain knowledge it
does not otherwise have (ADR-001, and the module's own doc).

## Where the scope is resolved

**Inside `install_claude_hook`, not at its call sites.**

```rust
pub(crate) fn install_claude_hook(
    root: &Path,
    spec: &HookSpec,
    dry_run: bool,
) -> anyhow::Result<HookWrite> {
    let scope = crate::dtoml::load_doctrine_toml(root)?.install.claude_settings_scope;
    let written = install_hook_to_file(
        root, settings_rel(scope), scope.command_form(), spec, dry_run,
    )?;
    // Sweep only if the canonical set actually landed. `PrintedFallback` is an
    // `Ok` value, so the `?` above cannot see this failure — see below.
    let evicted = if matches!(written, RefreshOutcome::PrintedFallback { .. }) {
        EvictOutcome::NotAttempted
    } else {
        evict_hook_from_file(root, settings_rel(scope.sibling()), spec, dry_run)?
    };
    Ok(HookWrite { scope, written, evicted })
}
```

This is the decisive factoring. `install_claude_hook` has two callers — the
seven-spec loop in `install_refresh` and `memory sync install`
(`src/corpus.rs:506`) — and the second is exactly the routine, flagless install
`DEC-163` argues about. If the scope were a parameter, `run_sync_install` could
forget it and re-create the entry in the abandoned file on every memory-sync
install. Resolving inside makes that unspellable, so `memory sync install`
inherits the scope, the command form and the sweep without electing to.

The cost is one small TOML read per spec. That is not worth a parameter whose
omission is the defect.

**`corpus.rs` does change, and the earlier claim that it does not was wrong in
the dangerous direction.** `run_sync_install` matches `install_claude_hook`'s
value directly against `RefreshOutcome::Wired` / `Refreshed` / `None` /
`PrintedFallback` (`src/corpus.rs:506-524`), which does not compile against
`HookWrite`. The compile break is trivial; what it was hiding is not. Inheriting
the sweep *silently* means the documented ritual after any shipped-memory edit
would resolve a scope, write to it, delete owned entries from the sibling, and
say nothing about either — the highest-frequency install path performing this
slice's one destructive write with no output. `DEC-164` calls the rider "what
keeps a destructive-looking write honest"; `DEC-163` singles out this very caller
as the reason the scope is a sticky key. Inheriting the mechanism without the
report breaches both on their own terms.

So `run_sync_install` matches `write.written` for its four existing arms, and the
announcement is not `wire`'s to own — see *The announcement* below.

**Write before evict — and only if the write landed.** Ordering activation
before removal leaves an interrupted run firing twice rather than not at all,
the same reasoning `DEC-167` applies to the human cutover, at file granularity.
But **ordering alone does not buy that property**, and the earlier text claiming
it did was wrong in the one direction this slice cannot afford.

A failed write is not an error. `plan_hook` turns malformed JSON, or a
wrongly-typed `hooks`/`hooks.<event>`, into `RefreshOutcome::PrintedFallback`
(`src/boot.rs:1159-1168`, `:1170-1178`) — an `Ok` value carried out through
`install_hook_to_file` (`:1611`). So a bare `?` sequences the sweep *after* a
write that did not happen, and the sweep then succeeds, because the sibling is
perfectly readable. Reachable, and by this slice's own choices: an operator on
`Local` today has working hooks in `settings.local.json`; `DEC-163` makes
`Project` the default and `sec-3` makes `settings.json` the committed,
hand-editable file this repo already tracks (`sec-6`). One malformed committed
file on the first install after this slice, and doctrine writes nothing, deletes
eleven working entries from the sibling, and lands on **zero activation** —
verbatim the state `DEC-167` orders the acts to walk away from, produced by the
ordering meant to prevent it.

The guard is the one line above. `Wired` / `Refreshed` / `None` all mean the
canonical set is present in the target; only `PrintedFallback` means it is not,
and only the first three earn the right to sweep. `EvictOutcome::NotAttempted`
is what the fourth reports — not `Nothing`, because "we chose not to sweep" and
"there was nothing to sweep" are different facts, and the operator needs the
first one to understand why both files still carry entries.

This is **not** `F-4`'s case and `EvictOutcome::Unreadable` does not cover it:
there the *sibling* could not be read, here the sibling reads fine and the sweep
would succeed. Success is what makes it destructive.

## The sweep

Drop-only. `plan_hook` minus the insert:

```rust
/// The outcome of ONE SPEC's sweep. Four states, mutually exclusive at this
/// granularity: a given spec either swept, found nothing, could not, or was
/// not asked to. "Could not read it", "nothing to do" and "not attempted" are
/// different facts about the file being abandoned, and collapsing any pair of
/// them makes a failed or skipped sweep indistinguishable from an empty one.
enum EvictOutcome {
    /// Nothing owned was there — the legitimate no-op.
    Nothing,
    /// `n` owned entries removed.
    Removed(usize),
    /// This spec's `hooks.<event>` could not be swept: malformed JSON, or a
    /// `hooks` / `hooks.<event>` of the wrong type. Left untouched, and SAID SO.
    Unreadable,
    /// Not attempted, because this spec's write to the TARGET did not land.
    /// Sweeping would remove the only working copy (see above).
    NotAttempted,
}

/// The abandoned-scope half of DEC-164: drop this spec's owned entries from
/// `existing_json` without inserting. Same ownership predicate, so a foreign
/// entry is never at risk; same fail-soft, so a malformed file is left untouched
/// — but fail-soft here is REPORTED, because the entries are still firing.
fn plan_evict(existing_json: Option<&str>, spec: &HookSpec) -> EvictPlan {
    let Some(mut value) = parse_settings(existing_json) else { return EvictPlan::UNREADABLE };
    let Some(arr) = hook_array_mut(&mut value, spec.event) else { return EvictPlan::UNREADABLE };
    let count = owned_positions(arr, spec.is_ours).len();
    if count == 0 { return EvictPlan::NOTHING; }
    drop_owned_hooks(arr, spec.is_ours);
    EvictPlan { outcome: EvictOutcome::Removed(count), new_json: serde_json::to_string_pretty(&value).ok() }
}
```

`parse_settings(existing_json) -> Option<Value>` is the fail-soft parse both
planners now share — absent or empty yields an empty object, malformed yields
`None`. Extracting it is what keeps `plan_evict` from being a second copy of
`plan_hook`'s prologue. An **absent** sibling parses to an empty object and
reaches `Nothing`, not `Unreadable`; only a file that exists and cannot be
understood is unreadable.

**Why the third state earns its keep.** The write path already invests in this:
`plan_hook` turns the same two `None` conditions into `PrintedFallback`
(`src/boot.rs:1159-1168`), a diagnostic complete enough to repair by hand — and
`sec-2` deliberately upgrades it so the snippet covers the whole matcher set
rather than one entry of it. A two-valued sweep would spend that care on the file
being written and none on the file being abandoned, having argued below that the
sweep exists precisely because doctrine can reach both.

The concrete state it prevents: a user with a hand-edited, slightly broken
`.claude/settings.local.json` switches to project scope. Doctrine writes eleven
entries to `settings.json`, cannot parse the sibling, and — under a silent
no-op — reports success. Every hook fires twice, permanently. That is verbatim
the condition `DEC-164` refused to leave as documentation, on the ground that
"the failure is silent … it presents as mild slowness and nothing else" and
"nothing would report it", with the doctor leg that might have caught it gone to
IMP-407. Engineering the mechanism and then silencing its failure would reproduce
the rejected state rather than fix it.

Fail-soft on a file being *left* stays the right safety posture — it is a weaker
obligation than fail-soft on a file being written, and correspondingly a stronger
reporting one.

Three further properties fall out of reusing the predicate rather than writing a
new one:

- **Never-clobber is untouched.** Eviction is gated by exactly the predicate
  that protects foreign entries during a normal merge.
- **It inherits the healing property.** Because `DEC-161` keeps ownership
  command-only, a stale-matcher entry in the abandoned file is still recognised
  as ours and still evicted. Under the rejected `(command, matcher)` ownership
  it would have been orphaned in the file doctrine is walking away from —
  the worst possible place for an unreachable double-fire.
- **It never inserts.** The file being left is only ever made smaller.

`evict_hook_from_file(root, rel_path, spec, dry_run)` is `install_hook_to_file`
with `plan_evict` in place of `plan_hook`; both write atomically through
`fsutil::write_atomic`, and neither creates a file that does not exist — a sweep
of an absent sibling is a clean no-op. It needs no `CommandForm`: eviction is by
ownership predicate, and `DEC-161` keeps ownership command-only, so both forms
are owned in either file — which is exactly what makes the scope switch heal.

## Folding seven sweeps into one file's report

The seven specs sweep **one** sibling file, so the per-spec outcomes have to
fold. They do not fold into an `EvictOutcome`, and the shape matters:

```rust
/// What the whole Claude arm did to the abandoned file. A COUNT AND TWO FLAGS,
/// not a sum type: at file granularity these facts are independent — a run can
/// remove two entries AND fail to sweep a third event AND skip a fourth.
#[derive(Default)]
struct SweepReport {
    /// Total owned entries removed, summed across specs.
    removed: usize,
    /// Some spec's `hooks.<event>` could not be swept — entries may still fire.
    unreadable: bool,
    /// Some spec's sweep was skipped because its write did not land.
    skipped: bool,
}
```

**Why an absorbing fold is wrong.** `EvictOutcome::Unreadable` is a fact about
one **spec**, not about the file: `hook_array_mut` navigates to
`hooks.<event>` and returns `None` when *that event's* value is the wrong type
(`src/boot.rs:1127-1137`), leaving every other event in the same well-formed
file perfectly sweepable. The reachable state, over one sibling:

- `.claude/settings.local.json` is valid JSON; `hooks.SessionStart` is a proper
  array holding an owned entry; `hooks.SubagentStart` has been hand-edited to a
  string.
- spec `sync` (`SessionStart`) → the file **is written** → `Removed(1)`.
- spec `nominate` (`SubagentStart`) → `hook_array_mut` returns `None` →
  `Unreadable`.

Under an absorbing rule the `Removed(1)` is discarded and the run reports only
that it could not sweep a file it just deleted an entry from. That breaches
`DEC-164` on the clause this design quotes twice — the installer "says what it
removed and from where" — and it breaches `plan_evict`'s own doc-comment, which
promises the fail-soft is *reported* rather than substituted for the write. The
precision `plan_evict` invests in one level down (an absent sibling reaches
`Nothing`, not `Unreadable`) would be spent and then thrown away by the fold
above it.

So the rider prints **every** line that is true, not the most severe one.

## The announcement

`DEC-163` drops the `--scope` flag, so discoverability has to come from the
installer. One line, early, before any hook line:

```
  claude: hooks → .claude/settings.json  ([install] claude-settings-scope in .doctrine/doctrine.toml)
```

then a rider — **one line per true fact, not one line total**, because
`SweepReport`'s three fields are independent:

```
  claude: evicted 2 stale hook entries from .claude/settings.local.json
  claude: could not sweep part of .claude/settings.local.json (malformed) — stale doctrine hooks may still fire there
  claude: did not sweep .claude/settings.local.json — .claude/settings.json could not be written, so there is nothing to replace it with
```

The third is the `F-12` guard speaking. It reads as the warning it is: not
"cleanup skipped" but "your activation is still in the other file, and it is the
only copy" — which, alongside the malformed-target fallback snippet printed by
the write path, is the whole state the operator needs. The failure mode the
guard removes was the *opposite* pairing: a fallback snippet next to
"evicted 11 stale hook entries", which reads like routine cleanup and is in fact
the announcement that nothing fires any more.

**The seam is shared, not `wire`-local.** Siting it on `boot::wire` was the
mistake behind the `corpus.rs` claim above: `run_sync_install` never enters
`wire`, so the routine path would have inherited the sweep and none of the
reporting. One writer, two callers:

```rust
/// The DEC-163 announcement and the DEC-164 rider. Called by `wire` once per
/// Claude arm before its hook lines with the folded `SweepReport`, and by
/// `run_sync_install` with its single `HookWrite`'s outcome lifted through
/// `SweepReport::from` — so the destructive write is reported on BOTH paths.
pub(crate) fn write_scope_report(
    out: &mut dyn Write,
    tag: &str,
    scope: ClaudeSettingsScope,
    swept: &SweepReport,
) -> io::Result<()>
```

Taking `&mut dyn Write` rather than reaching for stdout keeps it testable and
matches how `install.rs`'s link legs already thread output. `print_stdout` is
denied (`Cargo.toml:228`); both callers pass the handle they already hold, so no
new output surface is created.

The rider is what makes the difference visible on the path that matters most:
`doctrine install` is the documented ritual after any shipped-memory edit, so
`memory sync install` is the highest-frequency caller of the slice's one
destructive write.

## Reporting shape

`RefreshReport.hook: RefreshOutcome` becomes a collection, since one Claude
refresh now performs seven merges:

```rust
struct RefreshReport {
    /// One outcome per spec, in emission order. The Codex arm carries exactly one.
    hooks: Vec<RefreshOutcome>,
    /// The scope written and what the sibling sweep found, folded across specs.
    /// `None` on the Codex arm, which has one settings file.
    claude_scope: Option<(ClaudeSettingsScope, SweepReport)>,
    baseref: BaseRefOutcome,
    mcp: RefreshOutcome,
    append_system: AppendSystemOutcome,
    extension: ExtOutcome,
    mcp_extension: ExtOutcome,
    spike_warning: bool,
}
```

`wire`'s existing `match report.hook { … }` becomes the same match inside a
`for` — the four `RefreshOutcome` arms and their message strings are unchanged,
which keeps the Codex arm's output byte-identical.

## `install_baseref` follows the scope

`install_baseref` (`src/boot.rs:1428`) is the third consumer of the settings
path and is not a hook — it sets `worktree.baseRef = "head"`. Calling it
"unchanged" and renaming its constant to `SETTINGS_LOCAL_REL` would leave
doctrine writing **two** Claude settings files per install: eleven hook entries
into `settings.json` and one key into `settings.local.json`. That contradicts
this section's opening sentence — doctrine writes exactly one of two files — and
puts a key outside the pair the sweep reasons about.

So it takes `settings_rel(scope)` like everything else. The value is project
semantics, not machine state: every collaborator on a repo wants forks taken from
head, and unlike a baked exec path there is nothing per-machine in `"head"` to
keep out of git.

**The sweep does not chase it, and the stale copy is not inert.** Eviction is
spec-keyed — it walks `hooks.<event>` arrays by ownership predicate — and
`worktree.baseRef` is a top-level key with no spec. Extending a destructive path
to non-hook keys is the wrong trade, so the key is not swept. But the earlier
justification for not sweeping it — that the stale copy "carries the identical
value doctrine would write, so it cannot disagree with the live one" — assumed
the value is invariant, and doctrine is built on it not being.

`BaseRefOutcome::Conflict(String)` (`src/boot.rs:1338-1340`) is documented as
"Present with a NON-`head` value; left untouched, reported (no clobber — a
per-operator deliberate override is respected, §8.3)". So the reachable state is:

1. An operator sets `worktree.baseRef = "main"` in `.claude/settings.local.json`.
   Doctrine respects it and reports `Conflict` on every install.
2. This slice ships, default scope `Project`. `install_baseref(root, Project, ..)`
   reads `.claude/settings.json`, finds no `worktree` key, writes `"head"`,
   reports `Set`.
3. The override survives untouched in the abandoned file.

**And it wins.** Claude settings precedence is explicit: *Local* "overrides
project and user settings" (`docs/claude/settings.md:56-57`); only permission
rules merge rather than override, and `worktree.baseRef` is a scalar. So the
stranded `"main"` continues to govern and doctrine's fresh `"head"` is the inert
one — the exact inverse of what the removed sentence claimed. The operator's
`Conflict` signal disappears at the same moment, because doctrine is now reading
a file the override is not in.

What rides on it is why this is not cosmetic. `worktree.baseRef` governs where
the Claude dispatch arm forks from (SL-064 §8), and a wrong fork base is an
already-bitten failure here — `mem.signpost.doctrine.dispatch-claude-arm-wrong-base`
is a standing signpost in the boot snapshot.

**So the key is not swept, but it is read and reported.** `install_baseref`
inspects the sibling for a `worktree.baseRef` and the report carries what it
found:

```
  claude: worktree.baseRef in .claude/settings.local.json is "main", not "head" —
    local settings override project settings, so it still governs; remove it there
    or set [install] claude-settings-scope = "local"
```

This keeps no-clobber intact — a deliberate override is still never deleted by
doctrine — while restoring the signal the scope flip would otherwise silence.
It is the same obligation `F-2` and `F-4` established for the hook leg: a
scope-changing act is legible. `sec-6`'s `R10` note says the sweep touches
doctrine-owned **hook entries**, which stays literally true.

## What this leaves for `R8`

Nothing. The sweep reaches only the two settings files. The plugin's entries
load through `enabledPlugins` plus per-user marketplace registration — state
this slice reads and writes nothing of by Non-Goal — so `R8` stays
documentation and cannot be otherwise (`DEC-167`). The asymmetry is the point:
`R10` is engineered because both settings files are doctrine's to write; `R8` is
documented because the plugin's activation state is not.
<!-- doctrine:section sec-4 -->
# The write seam, and the retirement act

## Where the seven specs are written

`install_refresh`'s Claude arm (`src/boot.rs:1284`), whose `hook` field has been
`RefreshOutcome::None` since SL-152 PHASE-06:

```rust
Harness::Claude => {
    let specs = claude_hook_specs(exec);
    let mut hooks = Vec::with_capacity(specs.len());
    let mut swept = SweepReport::default();
    let mut scope = ClaudeSettingsScope::default();
    for (i, spec) in specs.iter().enumerate() {
        let write = install_claude_hook(root, spec, dry_run).with_context(|| {
            format!(
                "Claude hook {}/{} ({} {}) failed; {} of {} specs were written before it",
                i + 1, specs.len(), spec.event, spec.args, i, specs.len(),
            )
        })?;
        scope = write.scope;
        swept.absorb(write.evicted);
        hooks.push(write.written);
    }
    let baseref = install_baseref(root, scope, dry_run)?;  // same scope-selected file
    let mcp = install_mcp(root, dry_run)?;                 // unchanged, .mcp.json
    Ok(RefreshReport { hooks, claude_scope: Some((scope, swept)), baseref, mcp, .. })
}
```

`SweepReport::absorb` folds each per-spec `EvictOutcome` in: `Removed(n)` sums
into `removed`, `Unreadable` sets the `unreadable` flag, `NotAttempted` sets
`skipped`, `Nothing` is the identity. **No state is absorbing** — the fold
accumulates independent facts rather than electing the most severe one, for the
reason `sec-3` gives: `Unreadable` is per-spec, so it cannot stand for the file.

```rust
/// The Claude hook registry — the single enumeration of what doctrine activates.
/// Order is emission order in the settings file and in the installer's output.
fn claude_hook_specs(exec: &Path) -> Vec<HookSpec> {
    vec![
        HookSpec::boot_emit(exec, SESSION_MATCHERS),
        HookSpec::sync(exec),
        HookSpec::create_fork(exec),
        HookSpec::nominate(exec),
        HookSpec::denominate(exec),
        HookSpec::pretooluse(exec),
        HookSpec::memory_surface(exec),
    ]
}
```

One list, in one place. It is what a later conformance check would read to keep
the published `hooks.json` honest — that check is IMP-407's, but the registry it
needs is this slice's, and building it as a function rather than seven scattered
calls is what makes IMP-407 cheap.

`HookSpec::create_fork`'s `expect(dead_code)` comes off: it acquires a
production caller here. `HookSpec::boot` keeps its own — nothing ships it
(`DEC-162`).

`install_mcp` is untouched, and `install_baseref` changes only in taking the
scope (`sec-3`). They remain the worked example of a non-hook key riding *beside*
the merge core as a separate pure planner plus shell, and nothing about seven
specs changes that.

**Failure isolation.** `wire` already isolates one harness's failure from the
others (`src/boot.rs:1951`, `wire_isolates_one_harness_failure` at `:4803`). The
seven-spec loop uses `?`, so a mid-loop failure aborts the Claude arm with some
hooks written and some not. That is the correct shape: the loop's only failure
modes are IO on a file doctrine has just successfully created, every write is
atomic and independently idempotent, and a re-run completes the set. Swallowing
per-spec errors would hide a half-activated install, which is the failure class
this whole slice exists to stop being silent about.

**But the `?` must carry the boundary with it.** On abort at spec *k* the
`RefreshReport` is never constructed, so the accumulated `hooks` vector is
dropped with it — the operator learns the arm failed and not how far it got. For
most slices that is unremarkable; here the abort leaves exactly the
half-activated install the paragraph above invokes to justify the `?`, and the
sweep folded into each write means the intermediate state straddles both files:
specs 1..k-1 written *and* swept, specs k..7 unwritten with their stale entries
still live in the abandoned one. `sec-6` raises the stakes for one member of that
set — an absent `WorktreeCreate` hook changes dispatch semantics without saying
so.

Hence the `with_context` above: the error names the spec that failed and how many
landed before it. That is deliberately cheaper than surfacing a partial report —
a half-built `RefreshReport` would have to be interpretable by `wire`'s success
path, which buys a type for a condition whose remedy is unconditional. **The
operator's only obligation on failure is to re-run**, and the context line exists
so the interim state is legible rather than guessed at.

## Removing the plugin activation

This is the act the slice is named for. `doctrine install` stops touching
Claude's plugin machinery.

Deleted from `src/install.rs`:

- the `run()` Claude branch's steps 1 and 2 — marketplace add/refresh and plugin
  install/update, with their prompts (`:445`–`:518`);
- the trailing "Claude Code requires the doctrine plugin" reminder and the
  `skipped_marketplace` / `skipped_plugin` tracking that feeds it;
- `claude_plugin_add_marketplace`, `claude_plugin_install`, `claude_plugin_update`,
  `claude_plugin_has`, `claude_cmd_stdout`;
- `MarketplaceSource` + `as_arg`, `select_marketplace_source`, the `--dev`
  manifest precondition checks, `enable_key`, `parse_registered_source`,
  `RegisteredSource`, `MarketplaceAction`, `marketplace_action`,
  `refresh_failure_is_fatal`;
- their tests: `enable_key_is_qualified_doctrine`,
  `parse_registered_source_reads_directory_and_github`,
  `parse_registered_source_absent_or_sibling_is_none`,
  `marketplace_action_add_skip_refresh`, `refresh_failure_is_fatal_only_on_refresh`,
  and the `--dev` precondition cases.

**The transitive closure, which the above does not reach.** Each of these is
reachable *only* from code deleted above, verified by grepping its callers rather
than its absence:

| orphan | line | sole reader once the cut lands |
|---|---|---|
| `MarketplaceManifest` | `:672` | `:743`, inside `select_marketplace_source` |
| `ManifestPlugin` | `:679` | `MarketplaceManifest` itself |
| `select_plugin` | `:687` | `:750`, the `--dev` precondition |
| `MARKETPLACE_MANIFEST_REL` | `:703` | `:735`, the same precondition |
| `source_matches` | `:830` | `:849`, `marketplace_action` |
| `claude_list_has` | `:765` | `:780`, `claude_plugin_has` |

And four tests beyond the five listed above: `select_plugin_picks_by_name_not_first`
(`:3037`), `plugin_presence_is_exact_not_substring` (`:3059`),
`marketplace_presence_is_exact_token` (`:3075`), `source_default_is_github_slug`
(`:3084`).

This is not tidiness. The workspace denies `dead_code`, so an implementer working
the inventory literally does not ship a stale helper — they ship a **red gate**,
and then re-derive this closure themselves at the point the design was supposed
to have supplied it.

Deleted from the CLI: `InstallArgs.dev` (`src/install.rs:187`) and the `--dev`
flag (`src/commands/cli.rs:126`, threaded at `:1500`/`:1511`). Its sole consumer
is `select_marketplace_source`; with the marketplace step gone the flag has no
remaining meaning, and keeping a flag that selects between two sources for a
registration nobody performs is worse than removing it.

**`DOCTRINE_MARKETPLACE` goes**, and the condition is answerable now rather than
carried into implementation. Its three readers are `:457` (inside the deleted
step 1), `:699` (`enable_key`) and `:751` (the `--dev` check). All three are
deleted above, so nothing reads it.

**`[install] repo` survives.** Its other consumer is `delegate_argv` on the npx
path (`install_for_other`, `src/install.rs:423` → `:1989`, called `:2058`), which
`OQ-2` left unchanged for non-Claude agents.

## The claims the retirement falsifies

`sec-5` catches one of these on the gitignore leg and the same reasoning reaches
three more. The retirement is the event that invalidates them and this design is
the artefact that knows it, so they are named here rather than found later:

1. **`src/install.rs:2296-2301`** — a banner reading "Hooks plugin leg — install
   the doctrine Claude plugin as a skills-directory plugin so hooks … auto-load
   without a marketplace install step". There is no code under it; the next item
   is `#[cfg(test)] mod tests_hymns`. It already misdescribes the file, and after
   this slice it asserts the opposite of the slice's name. **Delete it** — a
   reader has *more* reason to believe it after the retirement, not less.
2. **`src/install_config.rs:2-8`** — the module doc says `[install]`
   parameterises "the printed post-install delegation instructions", naming the
   marketplace command this slice deletes; and the field added here is activation
   scope, not a printed instruction. Both halves go stale in one slice. Mirrored
   at **`src/dtoml.rs:44-45`**.
3. **`src/boot.rs:1614-1616`** — `install_claude_hook`'s own doc says it merges
   into `.claude/settings.local.json`. That is the single sentence `sec-3`'s
   scope dial most directly falsifies, in the function `sec-3` rewrites.
   Adjacent and lower-stakes: `HookSpec`'s doc (`:960-964`) and `plan_hook`'s
   (`:1147-1152`) both describe a `SessionStart` hook — already loose with
   `create_fork` on `WorktreeCreate`, and wrong once five events ship.

Plus the one `sec-5` already has: `ensure_gitignored`'s doc claiming `skills
install` reuses it.

## What deliberately survives

| artefact | why |
|---|---|
| `plugins/` | the canonical skill source for every channel (IMP-400 `OQ-1`) |
| `plugins/doctrine/hooks/hooks.json` | the published plugin's payload — `R9`'s managed-policy escape hatch |
| `.claude-plugin/marketplace.json` | published, not projected — the sanctioned ADR-019 posture |
| `SpawnSeamSymmetry` (`src/doctor_checks.rs:673`) | its input is `hooks.json`, which survives; the check needs **no** change |
| pre-existing `enabledPlugins` entries and marketplace registrations | migration is out of scope; IMP-400 stays open carrying it |

`R6` is withdrawn on exactly this: retirement was thought to blind
`SpawnSeamSymmetry`, but the file it parses is authored, shipped, and kept. What
retirement actually leaves behind is un-policed drift between that manifest and
the `claude_hook_specs` registry — already live, and IMP-407's Leg 2.

## Consequence for the install flow

The Claude branch of `run()` reduces to what it should always have been: agent
defs, the probe def, workflows, and skills. The plugin prompts disappear from
the interactive flow, which removes two of the questions a cold install asks.

The manual commands move into `install/` as `R9`'s documented escape hatch —
not as vestigial text, but as the named remedy for a
`strictPluginOnlyCustomization` environment where direct-written hooks are
blocked and only plugin-sourced ones load.
<!-- doctrine:section sec-5 -->
# The Claude skills channel

`OQ-2b`: rebuild the binary-sourced canonical tree plus proven-ownership
symlinks — the channel SPEC-010 responsibilities 3–6 already describe, deleted
at `347197e8` with no amendment.

## What survived the deletion, and what did not

Every helper the channel needs is **live in `src/install.rs` today**, because the
agents and workflows legs use them:

| helper | line | used by |
|---|---|---|
| `install_base` | `:1816` | agents, workflows |
| `relative_path` / `relative_target` | `:1859` / `:1877` | agents, workflows |
| `classify_link`, `Link`, `ForeignReason`, `foreign_reason` | `:1918` (`:1892`, `:1883`, `:1981`) | agents, workflows |
| `write_link`, `staging_path` | `:1964` (`:1954`) | agents, workflows |
| `Entry`, `discover` | `:1670`, `:1731` | the npx delegate path |
| `select_for_install` | `:2079` | the npx delegate path |
| `PluginAssets` | `:22` | the RustEmbed root, top of file |

Only the orchestration went: `install_for_claude`, `claude_dir`,
`canonical_dir`, `claude_links`, `materialise_canonical`, `copy_skill`, and the
`Canonical` struct. Recoverable verbatim from `git show 347197e8^:src/skills.rs`.

**They return to `src/install.rs`, not to a new module.** Every helper above is
private to that file; a `skills_claude` module would have to widen six
private items to `pub(crate)` to buy separation for ~150 lines. `install.rs` is
already long, but splitting it is a distinct chore over the whole file, not a
boundary to draw around one leg.

## `DEC-166`: the extracted trichotomy

The link-reconcile block is byte-identical at `src/install.rs:2179-2191`
(agents) and `:2279-2291` (workflows). Skills would be the third copy, not the
fourth-copy *risk* `T3` framed. One helper, taking exactly the inputs the
existing block already has:

```rust
/// Reconcile one managed link by proven ownership, reporting the action.
/// The single trichotomy: agents, workflows and skills all call this.
fn reconcile_link(
    name: &str,
    dest: &Path,
    target: &Path,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    match classify_link(name, dest, target) {
        Link::Create { .. } => {
            write_link(dest, target)?;
            writeln!(out, "  linked    {name}")?;
        }
        Link::Relink { .. } => {
            write_link(dest, target)?;
            writeln!(out, "  relinked  {name}")?;
        }
        Link::KeepForeign { reason, .. } => {
            writeln!(out, "  kept      {name} ({})", foreign_reason(&reason))?;
        }
    }
    Ok(())
}
```

The boundary is deliberate and `DEC-166` fixes it: no materialise callback, no
trait, no absorption of the agents leg's `"claude" => claude_agents_dir, _ =>
pi_agents_dir` branching. The three legs' materialise phases are genuinely
unlike — single file with a hymns bake, single file plain, whole directory tree
— and a generic driver would be absorbing precisely the part that varies, for
three callers.

Two call sites shrink from thirteen lines to one; skills gets the third for free.

## The restored channel

```rust
/// The canonical skills tree (project-local, or under `$HOME` with `global`).
fn skills_canonical_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".doctrine/skills"))
}

/// The Claude skills link dir. The only Claude-specific element in the leg.
fn claude_skills_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".claude/skills"))
}

/// Refresh the canonical tree once, then reconcile links into it from every
/// target directory. Parameterised over `link_dirs` per `OQ-9`/`DEC-166`:
/// materialise runs ONCE, the link phase loops.
pub(crate) fn install_skills_direct(
    root: &Path,
    selected: &[&Entry],
    link_dirs: &[PathBuf],
    global: bool,
    dry_run: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let canon_dir = skills_canonical_dir(root, global)?;
    writeln!(out, "agent claude (direct):")?;
    for entry in selected {
        let canon = canon_dir.join(&entry.id);
        writeln!(out, "  skill     {} → {}", entry.id, canon.display())?;
        if dry_run { continue; }
        materialise_canonical(entry, &canon)?;
        for link_dir in link_dirs {
            let dest = link_dir.join(&entry.id);
            let target = relative_target(link_dir, &canon_dir, &entry.id);
            reconcile_link(&entry.id, &dest, &target, out)?;
        }
    }
    Ok(())
}
```

Called from `run()`'s Claude branch with **one** entry:
`&[claude_skills_dir(root, args.global)?]`. Shipping `.agents/skills` as the
second target is IMP-406's, and it is then a call-site change, not a mechanism
change — which is exactly the scope `OQ-9` set.

The name is `install_skills_direct`, not `install_for_claude`: once `link_dirs`
is a parameter nothing in the body is Claude-specific, and a Claude-named
function would mislead IMP-406's author the way `claude_links` misled `T3`'s
reading of the recovered code.

## Two changes from the recovered code

**The double classify goes.** `install_for_claude` computed a `Vec<Link>` up
front via `claude_links`, then re-classified each one at mutation time inside
the loop — the second classify being the correct one (a stale plan is a race).
The restored leg classifies once, at mutation time, which is what
`reconcile_link` does and what the agents and workflows legs already do. The
up-front vector had no other consumer.

**`Canonical` goes.** It existed to carry a plan between `build_plan` and
`execute` in the old plan/execute split. The consolidated `install::run()`
dispatch has no such split, and the struct degenerates to `entry.id` plus a path
derived from it.

`materialise_canonical` and `copy_skill` return **verbatim**. Their
remove-then-rename swap, `.tmp-<id>` staging, and `symlink_metadata`-not-`exists`
leftover clearing are the pass-2 F4 reasoning SPEC-010 responsibility 5 records;
re-deriving any of it would be re-litigating a settled design.

## Gitignore

No call needed. `install/manifest.toml:46` already lists `.doctrine/skills/*`,
so the install manifest's gitignore leg lands the entry. `ensure_gitignored`'s
doc-comment still claims `skills install` reuses it — that claim is stale and
should be corrected to name the manifest, or the comment will send the next
reader looking for a call site that should not exist.

## Governance

Nothing here amends SPEC-010. `OQ-2b` restores what its responsibilities 3–6
already describe, and `DEC-166`'s parameterisation sits below spec altitude —
one target driven still "reconciles a relative agent symlink into it". The
pre-existing divergence closes by conformance, verified at close (`DEC-171`).
<!-- doctrine:section sec-6 -->
# Cutover and documentation

## The prescribed order (`DEC-167`)

1. Run the install that writes the settings hooks.
2. Disable the doctrine plugin by hand.

Between the two acts every hook fires twice. That is the intended cost. The
reverse order leaves the repo with **no** activation, and absence is the more
dangerous state: `isolation: worktree` teardown is conditional on
`WorktreeCreate` firing, so an inert hook changes dispatch's semantics without
saying so. Order toward the degraded state, never the absent one.

## The two notes are not symmetrical

**`R10` — a description, not an instruction.** Doctrine sweeps the abandoned
scope and reports what it removed, so the note exists only so a user who sees
`evicted 2 stale hook entries from .claude/settings.local.json` does not read it
as doctrine eating a hook they placed by hand. It should say what the sweep
touches (doctrine-owned entries, in the other of doctrine's two settings files,
never anything foreign) and what it does not.

**`R8` — pure documentation, and it cannot be otherwise.** The plugin's entries
load through `enabledPlugins` plus per-user marketplace registration, which this
slice reads and writes nothing of by Non-Goal. The ownership sweep cannot reach
them. So: *when you take the new activation, disable the doctrine plugin, or
every hook fires twice.*

The asymmetry is worth stating in the user-facing text, not just here: `R10` is
engineered because both settings files are doctrine's to write; `R8` is
documented because the plugin's activation state is not.

## What `install/` acquires

- **Activation, plainly stated.** Which file doctrine writes, which hooks it
  contains, and that Claude Code hot-reloads settings files — `hooks` explicitly
  included — so no restart is needed (`R1`, which inverted into an argument
  *for* the change: the plugin path needs `/reload-plugins`, whose reliability
  the memory corpus already doubts).
- **The scope key.** `[install] claude-settings-scope`, its two values, its
  default, and why a collaborator on a client project might set `local`.
  Mirrored into `install/doctrine.toml.example` (`:73`, which already carries a
  commented `[install]` block) and `install/doctrine.toml` (`:24`).
- **The command form, and `DOCTRINE_BIN`.** The committed scope writes
  `${DOCTRINE_BIN:-doctrine}`, so a reader who opens `.claude/settings.json` and
  expects a path finds a variable instead. The note has to say why (no host path
  in a committed file, SL-195, POL-002) and what to do when `doctrine` is off the
  harness `PATH` — set `DOCTRINE_BIN`, the same override `.mcp.json` has taken
  since SL-195. The `local` scope keeps the baked absolute path, and the reason
  it can is that the file is gitignored.
- **The cutover note** — the two risks above, in the prescribed order.
- **`R9`'s escape hatch.** For a `strictPluginOnlyCustomization` environment,
  where managed settings block hooks from user and project sources so they may
  come only from plugins: install the doctrine plugin by hand —
  `claude plugin marketplace add davidlee/doctrine` then
  `claude plugin install doctrine@doctrine --scope project`. These are the two
  commands `sec-4` removes from the installer; they become the documented remedy
  rather than an automated default.

  The honest framing matters here. Direct-write sheds two plugin-only failure
  modes (orphaned marketplace registration, plugin blocklist) but keeps
  `disableAllHooks`, `allowManagedHooksOnly` and folder trust, and **acquires**
  `strictPluginOnlyCustomization`. Fewer failure modes, not a subset — the docs
  must say so rather than overclaim.

## This repo's own cutover

`.gitignore:4` already carries `!/.claude/settings.json` alongside `/.claude/*`,
so the project settings file is already tracked here — `A3` is satisfied and
needs no edit. That is a benefit-realisation fact, not an activation one: Claude
Code reads the file regardless of whether git tracks it (`DEC-167`).

It is also the fact that made the command form non-negotiable. This repo will
commit the eleven entries doctrine writes, so on the first install after this
slice, `git diff` is the check that the portable form actually landed: eleven
`${DOCTRINE_BIN:-doctrine}` commands and no `/home/…` anywhere. That is SL-195's
own acceptance criterion — *no absolute host path in any tracked file* — now
covering hooks as well as `.mcp.json`, and this repo is the first place it is
exercised.

`R3` is downgraded — the human hand-repairs their own activation across the
cutover. What the design owes is only that the moment activation flips is
*stated*: it flips on the first `doctrine install` (or `doctrine boot install`)
run from a binary carrying this slice. There is no compatibility window in which
both channels are tolerated by design; there is only the interval between the
two cutover acts, which double-fires.

## Backlog disposition at close

- **IMP-400** — reduced to its migration leg and left **open**, not closed.
  Migration (`OQ-4`) is out of scope, and the doctor leg is IMP-407's. Two
  counts, not one.
- **IMP-407** — confirmed still open and still sequenced `after` this slice,
  carrying `R2`, `R7`, `OQ-5`, and the manifest-vs-registry conformance leg
  `R6`'s withdrawal surfaced.
- **IMP-406** — the `.agents/skills` neutral target, sequenced `after`; this
  slice ships only the parameterisation.
- **CHR-045** (*bump `plugin.json` version when the skill set changes*) — **no
  longer moot**: the manifest survives as `R9`'s escape hatch, so a stale
  published plugin is still a real defect. Resolve or explicitly retain.
- **IMP-234, CHR-037** — assessed for overlap.
<!-- doctrine:section sec-7 -->
# Verification

## The behaviour-preservation gate

This slice makes **two** kinds of change to the existing suites, and they must be
kept apart. Class (i) is the generalisation, where unchanged-green is the proof.
Class (ii) is the default-scope flip and the report-shape change, where specific
assertions are knowingly rewritten. Collapsing them into "a compiler-checked
rename" was the error worth naming: once a large expected red is baked into the
same suite for an unrelated reason, "any red here means the generalisation
narrowed something" stops being true, and that inference is the only thing the
design offers in place of testing the generalisation directly.

**Class (i) — preserved by construction. Any red is a real narrowing.**

- `corpus.rs`'s memory-sync hook and the Codex arm are the N=1 case of the
  matcher set, so `DEC-161`'s shape satisfies the gate by construction rather
  than by testing.
- `check_spawn_seam_symmetry_passes_the_shipped_hooks_config`
  (`src/doctor_checks.rs:1983`) passes **unmodified**. Its input survives this
  slice, so "needs no change" is the criterion.
- The `.mcp.json` planner: `MCP_COMMAND` → `PORTABLE_EXEC` is a pure rename with
  no behaviour attached, so `plan_mcp`'s tests change in that identifier only.
  This one really is the rename the old text claimed for everything.

**Class (ii) — knowingly rewritten. Enumerated so an expected red is
distinguishable from a real one.**

| test | line | why it changes |
|---|---|---|
| `install_refresh_writes_settings_and_respects_dry_run` | `:3904` | asserts `commands(&json).is_empty()` on the local file, commented "no boot hook wired for Claude (ships via plugin)". The Claude arm now writes eleven entries to the *project* file. **The assertion inverts** — it is not renamed |
| `install_claude_hook_wires_boot_and_sync_as_two_entries` | `:4180` (reads `:4190`, `:4204`) | reads `root.join(SETTINGS_REL)`; the correct constant is `SETTINGS_PROJECT_REL`, **not** `SETTINGS_LOCAL_REL`. A mechanical rename lands red with the wrong constant |
| `install_claude_hook_sync_respects_dry_run` | `:4216` (`:4223`) | same class |
| `install_claude_create_fork_hook_appends_worktreecreate_leaves_sessionstart_intact` | `:4250` | same class |
| `wire_adds_import_and_hook_then_is_idempotent` | `:4755` (`:4766`, `:4782`, `:4797`) | same class, plus the report shape |
| the malformed-settings case | `:4809` | same class |
| the hymns/boot case reading settings | `:5523` | same class |
| every `out.hook` assertion — **all three of them** | `:3916`, `:3924`, `:3953` | `RefreshReport.hook` becomes `hooks: Vec<RefreshOutcome>` |
| ↳ of which `:3953` is the **Codex arm** assertion | `:3953` | `assert!(matches!(out.hook, Wired(_) \| None))` in `install_refresh_writes_settings_and_respects_dry_run`. It becomes an index or a `first()`. This is the single site testing `sec-4`'s "the Codex arm's output stays byte-identical", and the site where `CommandForm::Baked` is exercised on that arm — call it out rather than let it ride "and neighbours" |
| the `worktree.baseRef` assertions | in `:3904` and neighbours | `install_baseref` follows the scope, so the key is read from the project file |

**Deliberately absent from that row:** `:3926` and `:3948` assert `out.mcp`, and
`:5303` asserts `out.mcp_extension`. Neither field changes shape, so neither
test changes for this row's reason. An earlier draft cited all three, which
inverted the row's purpose — a table that exists so an uncited red reads as
class (i) has to be exact, or its slack is the diagnosis it was meant to
provide. The `out.hook` sites are exactly three; they are all listed.

Every entry above is a **path or shape** change with the same underlying
behaviour. None of them is a matcher-set behaviour change, which is the point of
separating the tables: after this slice, a red in class (i) still means what
`sec-2` says it means.

The deliberate output changes: `wire`'s Claude arm now prints **seven** hook
lines where it printed none, beneath one scope-announcement line and up to three
sweep rider lines. The Codex arm's output stays byte-identical — the
`RefreshOutcome` match arms and their strings are unchanged, only the loop around
them is new, and `claude_scope` is `None` there so no announcement is written.

## New test cases

**Matcher set (`src/boot.rs`)**

- `plan_hook_emits_one_entry_per_matcher` — a four-matcher spec into empty
  settings yields four entries, in matcher order, each with the one command.
- `plan_hook_is_a_no_op_on_the_canonical_set` — re-planning the above returns
  `RefreshOutcome::None` and no JSON. The idempotence property the whole core
  rests on.
- `plan_hook_heals_a_partial_hand_edit` — delete one of four entries; the plan
  rewrites all four. Entry identity is the set.
- `plan_hook_heals_a_stale_matcher` — an owned entry on a retired matcher is
  recognised (command-only ownership) and rewritten onto the canonical set. This
  is the property `DEC-161` chose the shape *for*; it is the test that would go
  red under the rejected `(command, matcher)` ownership.
- `plan_hook_preserves_foreign_entries_around_a_matcher_set` — foreign entries
  before and after the owned block survive, and the set inserts at the first
  owned slot.
- `plan_hook_preserves_a_foreign_hook_sibling` — the existing single-matcher
  case, re-run with N>1: an entry carrying both an owned and a foreign hook
  keeps the foreign one and the set inserts after it.
- `single_matcher_specs_are_unchanged` — `HookSpec::sync` produces byte-identical
  JSON to the pre-slice output. The gate, asserted rather than assumed.

**Ownership predicates**

- `is_doctrine_command_recognises_each_new_spec` — the four new arg strings,
  each with a spaced program path and each with the `/proc/self/exe`
  ` (deleted)` poison suffix.
- `predicates_are_pairwise_disjoint` — no predicate matches another spec's
  command. This is what stops the seven specs clobbering one another in one
  file, and with seven specs sharing `SessionStart` and `PreToolUse` it is no
  longer self-evident. Worth asserting over the registry rather than by hand, so
  an eighth spec cannot join without being checked.

**Scope and sweep**

- `absent_key_defaults_to_project_scope` — an absent key and an absent
  `[install]` table both yield `Project` (`src/install_config.rs`).
- `scope_key_binds_from_its_documented_spelling` — parses the literal TOML text
  `[install]\nclaude-settings-scope = "local"` and asserts `Local`. **Written
  against parsed text, never against the constructed enum**: with no container
  `rename_all` the key is silently discarded and the field falls to `Project`,
  which an enum-level test cannot see. A companion asserts the underscore
  spelling does *not* bind, so the test fails if someone "fixes" it by adding a
  serde alias instead.
- `scope_key_selects_the_settings_file` — `local` writes
  `.claude/settings.local.json`, `project` writes `.claude/settings.json`.
- `writing_one_scope_evicts_the_other` — **the `R10` behavioural criterion.**
  Seed an owned entry in `settings.local.json`, install at project scope, assert
  the entry is gone from local, present in project, and the eviction count is
  reported.
- `eviction_spares_foreign_entries` — a foreign hook in the abandoned file
  survives the sweep. Never-clobber, on the destructive path.
- `eviction_heals_a_stale_matcher_in_the_abandoned_file` — a stale-matcher owned
  entry in the file being left is still evicted. The healing property, applied
  where it matters most: an orphan there is unreachable by any later install.
- `eviction_never_inserts` — the abandoned file only ever shrinks; a sweep of an
  absent sibling creates nothing.
- `memory_sync_install_honours_the_scope_key` — drives `run_sync_install`, not
  `install_claude_hook`, so it proves the resolution-inside-the-installer
  decision rather than restating it.
- `memory_sync_install_reports_scope_and_eviction` — the `F-2` criterion. Drives
  `run_sync_install` with a seeded owned entry in the sibling and asserts its
  captured output carries both the announcement and the rider. Without this the
  destructive write is silent on the highest-frequency path and every other test
  still passes.
- `plan_evict_is_fail_soft_on_malformed_json` — malformed settings are left
  untouched, no write, no panic.
- `a_failed_sweep_is_reported_not_silent` — malformed sibling and wrongly-typed
  `hooks.<event>` both yield `EvictOutcome::Unreadable`, and the installer prints
  the file it could not sweep. The preceding test asserts only the non-clobber
  half and passes under a silent no-op; this is the half that distinguishes
  "could not sweep" from "nothing to sweep".
- `a_failed_target_write_does_not_trigger_the_sweep` — **the `F-12` criterion,
  and the one that would go red today.** Seed a malformed `.claude/settings.json`
  and a well-formed `.claude/settings.local.json` carrying the owned entries;
  install at `Project` scope. Assert the local entries **survive**, the outcome
  is `EvictOutcome::NotAttempted`, and the installer says it did not sweep and
  why. Without the guard the sweep succeeds and the assertion on surviving
  entries fails — which is the point: the failure this test catches is a
  *successful* sweep.
- `a_partly_successful_sweep_reports_both_facts` — **the `F-15` criterion.**
  One sibling, valid JSON, `hooks.SessionStart` a proper array with an owned
  entry, `hooks.SubagentStart` hand-edited to a string. Assert the folded
  `SweepReport` carries `removed == 1` **and** `unreadable == true`, and that
  both rider lines print. `a_failed_sweep_is_reported_not_silent` above is
  written against a single spec and cannot reach this state — it passes under an
  absorbing fold.
- `baseref_follows_the_scope` — `worktree.baseRef` lands in the same file as the
  hooks under each scope, so doctrine authors one Claude settings file, not two.
- `a_stranded_baseref_override_is_reported` — **the `F-14` criterion.** Seed
  `worktree.baseRef = "main"` in `.claude/settings.local.json`, install at
  `Project`. Assert the override is **not** removed (no-clobber survives), the
  project file gets `"head"`, and the report names the stranded value — because
  local precedence means the stranded one still governs
  (`docs/claude/settings.md:56-57`).

**Command form and portability**

- `project_scope_writes_the_portable_command` — every one of the eleven entries
  carries `${DOCTRINE_BIN:-doctrine}` and **no** entry contains the test's
  absolute exec path. This is SL-195's INV-1 (*no absolute host path in any
  tracked file*) asserted for hooks.
- `local_scope_keeps_the_baked_path` — the gitignored file still gets
  `<abs exec>`, so `baked ⟺ gitignored` holds in both directions rather than one.
- `ownership_spans_both_command_forms` — `is_doctrine_program` recognises the
  portable literal and an abspath, including the ` (deleted)` poison suffix on
  the latter. The single arm five predicates inherit.
- `a_scope_switch_rewrites_the_command_form` — an abspath entry in the file now
  being written is owned-but-not-canonical, so it is **rewritten** to the
  portable form rather than duplicated; and an abspath entry in the abandoned
  file is **evicted** rather than orphaned. This is the pair that makes the
  migration land, and it is the test that would go red if the new
  `is_doctrine_program` arm were dropped.
- `the_codex_arm_renders_the_baked_form` — **the `F-13` criterion.**
  `install_codex_hook` passes `CommandForm::Baked` and `.codex/hooks.json` comes
  out byte-identical to pre-slice output. `sec-4` asserts that arm is untouched;
  this is the only test that says so, on the arm the `CommandForm` axis exists
  to keep honest. It pairs with `:3953` in the class-(ii) table.

**Skills channel (`src/install.rs`)**

- `install_skills_direct_materialises_and_links` — canonical tree at
  `.doctrine/skills/<id>`, relative symlink at `.claude/skills/<id>` whose value
  is `../../.doctrine/skills/<id>`.
- `install_skills_direct_is_idempotent` — a second run relinks without churn.
- `install_skills_direct_keeps_a_foreign_skill_dir` — a real directory at the
  link path is kept and warned, never replaced.
- `install_skills_direct_heals_a_dangling_owned_link` — a link whose value
  equals our target but whose canonical is missing is healed, not recreated as
  foreign.
- `install_skills_direct_links_every_target_dir` — driven with **two** link
  dirs. This slice ships one, but the loop is the deliverable `OQ-9` scopes, and
  an untested loop driven with N=1 is indistinguishable from a hard-coded target.
- `reconcile_link` — the agents and workflows legs' existing link tests are the
  extraction's proof; they must pass unchanged.

## Verification by human (`VH`)

**A cold install activates.** In a scratch project with no plugin registration
and no marketplace entry, `doctrine install` leaves hooks that actually fire.
This is the claim the whole slice rests on and no test can make it: `/hooks` is
the only surface reporting which hooks are live and which file each came from,
and it is interactive-only.

**Preconditions**, so the run is reproducible rather than recounted: a scratch
project, no `enabledPlugins` entry for doctrine, no marketplace registration, the
scope key absent (exercising the `Project` default), and a binary built from this
slice.

**Evidence to capture:** the `/hooks` output showing **all eleven entries**
sourced from `.claude/settings.json` — eleven, per the `sec-1` ledger, not seven;
seven is the spec count and a criterion written against it would pass a
four-entries-short install — plus one observed effect per event class: a session
boot emitting, a `WorktreeCreate` fork, and a `PreToolUse` surfacing.

**Where it lands.** The same sentence this section already writes for `VA`, and
for the same reason, applied to the leg that needs it more: **the `/hooks`
transcript and the three observed effects are transcribed into the reconciliation
brief**, verbatim and with the preconditions above stated alongside them. A
criterion over an interactive-only surface with no named sink produces evidence
that exists in one agent session and nowhere an auditor can reach
(`mem_019fd1d862887d42b7a1f88c28fd28a7`). The interaction is sharper here than
usual: `/hooks` reports live hooks, and in the prescribed scratch project the
`.claude/settings.json` it reports from is itself untracked — so pasting the
transcript into the brief is the only thing that makes it re-derivable.

A `VH` leg is the honest mode here. `R7` is the reason, and it moved to IMP-407
with the doctor leg: doctrine can verify plausibility, not activation. That is
also why the sink matters: with no automated check behind it, the transcript is
the whole of the evidence.

## Verification by agent (`VA`)

**SPEC-010 conformance.** Responsibilities 3–6 confirmed true of the restored
code — derived canonical `.doctrine/skills/<id>` tree, relative agent symlink
reconciled by proven ownership, `.tmp-<id>` staging with remove-then-rename, and
the self-enforced `.doctrine/skills/*` gitignore. `DEC-171` makes this a
conformance claim rather than a REV amendment, which is the stronger of the two:
an amendment edits text to fit what was built, a conformance claim checks the
built thing against text written before it.

The evidence must be re-derivable at audit, so it belongs in the reconciliation
brief with file/line citations — not in a runtime artefact that disappears.

## Governance

The REV amends **SPEC-011 / `REQ-186`** alone. It reads:

> `boot install` merges a `<exec> boot` SessionStart hook into Claude
> settings.local.json, refreshing a stale owned copy and preserving every
> foreign hook and key.

Invalidated on four axes: not one hook (seven specs emitting eleven entries
across five events), not `settings.local.json` (a scope-selected project
default), not `<exec>` (the committed scope writes the portable
`${DOCTRINE_BIN:-doctrine}`, `sec-2`), and a new obligation the requirement says
nothing about (the abandoned-scope sweep).

`QUE-209` — whether the REV widens `REQ-186` or adds new requirements for the
newly-governed hook set and the scope key — is **deferred to reconciliation**,
where the REV is authored and requirement granularity is the natural call. The
draft does not settle it.

RFC-018 (*Claude harness field notes*) takes anything empirical the slice
learns, including the `strictPluginOnlyCustomization` asymmetry.
