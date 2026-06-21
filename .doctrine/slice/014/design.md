# SL-014 Design: Codex SessionStart-emit boot wiring

## D1. Current vs Target

**Current:** `boot install --agent codex` writes the `@`-import into `AGENTS.md` (dead — codex
doesn't expand `@`-imports, confirmed live in SL-011 closure) plus pi extensions. No hook
is written.

**Target:** `boot install --agent codex` writes a `SessionStart` hook into `.codex/hooks.json`
with command `<exec> boot --emit`. The hook's stdout (the boot snapshot body) is injected by
codex as developer context per the official docs — zero lag, the proven spike mechanism.

- The `@`-import line in `AGENTS.md` is **untouched** — harmless dead content for codex (the
  guard block is ~160 bytes of committed noise that codex never expands). `import_targets` still
  maps `Harness::Codex → AGENTS.md`, so a fresh `boot install` writes the guard block even when
  codex is the only detected harness. Removing it cleanly requires a separate un-import operation
  that crosses into the Claude import path — out of scope. The block stays; codex ignores it.
- The Claude path (`settings.local.json` hook) is untouched.
- The pi path (extensions + append-system symlink) is untouched.

## D2. Three changes

| # | Change | Surface |
|---|---|---|
| D2.1 | `doctrine boot --emit` | New CLI flag: regenerate + print snapshot to stdout |
| D2.2 | Codex hook merge in `install_refresh` | Codex arm writes a `SessionStart` entry into `.codex/hooks.json` via the existing `plan_hook` core |
| D2.3 | Trust surfacing | `boot install` prints one-time trust instruction after writing the hook |

## D3. `boot --emit` (D2.1)

A `--emit` flag on the bare `boot` verb (sibling to `--check`; mutually exclusive). It
regenerates the snapshot (writes `boot.md` if changed, preserving the content-diff cache key),
then prints the rendered snapshot bytes to stdout. Deterministic, no clock.

- **`src/boot.rs`: `run_emit(root, exec)`** — calls a shared `build_and_render(root, exec)`
  (extracted from `regenerate`'s internals) to produce the snapshot content once, then
  `write_if_changed` to disk AND `write!(stdout, "{content}")` to stdout — same bytes to both
  sinks, no double-render, no trailing extra newline from `println!`.
- **`build_and_render(root, exec) -> String`** — extracted private helper: `render_boot(&build_sections(root, exec))`.
  `regenerate` becomes `build_and_render` + `write_if_changed`. `run_emit` = `build_and_render` +
  `write_if_changed` + `write!(stdout, ...)`.
- **`src/commands/cli.rs`:** `Boot` variant gains `--emit` flag (`conflicts_with = "check"`).

One render, exact same bytes to both sinks.

## D4. Codex hook merge (D2.2)

The `.codex/hooks.json` JSON structure is identical to Claude's `.claude/settings.local.json`
— `hooks.<event>[]` carrying `{matcher, hooks: [{type, command}]}`. So the existing
`plan_hook` merge core works unchanged.

### D4.1 New artefacts

| Artefact | Purpose |
|---|---|
| `CODE_HOOKS_REL = ".codex/hooks.json"` | Target path (gitignored, mirroring `SETTINGS_REL`) |
| `HookSpec::boot_emit(exec)` | Constructor: `SessionStart`, matcher `startup\|resume\|clear\|compact`, command `<exec> boot --emit` |
| `is_doctrine_emit_command(cmd)` | Ownership predicate: suffix-strip ` boot --emit`, check program is `doctrine` (pairwise-disjoint from boot/sync/stamp predicates) |
| `install_codex_hook(root, spec, dry_run)` | Thin wrapper → writes to `CODE_HOOKS_REL` |
| `install_hook_to_file(root, rel_path, spec, dry_run)` | **Extracted** private helper — DRY: both `install_claude_hook` and `install_codex_hook` call it, avoiding parallel implementation |

### D4.2 Matcher

Codex `SessionStart` matcher events are `startup|resume|clear|compact` — all four sources,
per the official docs. Broader than Claude's `startup|clear` because codex doesn't have
Claude's `resume`-triggers-`SessionStart` ambiguity (codex fires `SessionStart` on all four
explicitly).

### D4.3 Ownership predicate

```
is_doctrine_emit_command(cmd):
  strip_suffix(" boot --emit") → program
  is_doctrine_program(program)   # shared, poison-tolerant
```

Pairwise-disjoint from the three existing predicates (boot has no `--emit` suffix; sync has
no `--emit` suffix; stamp has no `--emit` suffix → no overlap).

**Full command matrix** (VT: positive/negative tests for every row):

| Command | Owned by emit? | Reason |
|---|---|---|
| `/abs/doctrine boot --emit` | yes | canonical shape |
| `/abs/doctrine boot --emit` (trailing spaces) | yes | `trim()` in `strip_suffix`? No — `strip_suffix` is exact. Trim first. |
| `/path with spaces/doctrine boot --emit` | yes | spaces in exec path, suffix-strip |
| `/abs/doctrine (deleted) boot --emit` | yes | poison-tolerance via `is_doctrine_program` |
| `/abs/doctrine boot` | no | matches `is_doctrine_boot_command` only |
| `/abs/doctrine boot --emit --verbose` | no | extra arg → `strip_suffix` fails |
| `sh -c 'doctrine boot --emit'` | no | program is `sh`, not `doctrine` |
| `/abs/doctrine memory sync` | no | matches `is_doctrine_sync_command` only |
| `/abs/doctrine worktree marker --stamp-subagent` | no | matches `is_doctrine_stamp_command` only |
| `/abs/other-tool boot --emit` | no | file name is `other-tool`, not `doctrine` |

### D4.4 Merge behaviour (inherited from `plan_hook`)

- **No existing hook** → append at tail (`Wired`)
- **Owned hook exists, stale exec path** → normalize to one canonical entry (`Refreshed`)
- **Owned hook exists, already canonical** → no-op (`None`)
- **Multiple owned hooks** → drop all, insert one canonical (`Refreshed`)
- **Foreign `SessionStart` hooks** → preserved (entry survives the drop)
- **Malformed JSON / wrong-typed `hooks`/`SessionStart`** → `PrintedFallback { hook_file, snippet }`, no clobber
- **Poison tolerance** → `(deleted)` suffix is stripped by `is_doctrine_program` (shared)

**`PrintedFallback` carries path + snippet.** Currently `RefreshOutcome::PrintedFallback` is a
unit variant hardcoded to `.claude/settings.local.json` + `HookSpec::boot` snippet. It becomes a
struct variant carrying `hook_file: &'static str` and `snippet: String`. `install_hook_to_file`
accepts a `fallback_hint: fn(&HookSpec) -> String` closure or the spec itself, so the codex
caller supplies `CODE_HOOKS_REL` + `fallback_for(&HookSpec::boot_emit(exec))`. `wire()` uses
these fields, no harness-branching in the printer.

### D4.5 `install_refresh` Codex arm (modified)

Currently:
```rust
Harness::Codex => {
    let append_system = install_append_system(root, dry_run)?;
    let extension = install_pi_extension(root, exec, dry_run)?;
    let mcp_extension = install_mcp_extension(root, exec, dry_run)?;
    Ok(RefreshReport { hook: RefreshOutcome::None, ... })
}
```

After:
```rust
Harness::Codex => {
    let hook = install_codex_hook(root, &HookSpec::boot_emit(exec), dry_run)?;
    let append_system = install_append_system(root, dry_run)?;
    let extension = install_pi_extension(root, exec, dry_run)?;
    let mcp_extension = install_mcp_extension(root, exec, dry_run)?;
    Ok(RefreshReport { hook, ... })
}
```

### D4.6 `install_hook_to_file` (extracted DRY helper)

```rust
fn install_hook_to_file(
    root: &Path,
    rel_path: &str,
    spec: &HookSpec,
    dry_run: bool,
) -> anyhow::Result<RefreshOutcome> {
    let path = root.join(rel_path);
    let existing = fs::read_to_string(&path).ok();
    let plan = plan_hook(existing.as_deref(), spec);
    if let (Some(json), false) = (&plan.new_json, dry_run) {
        // ... create_dir_all + write_atomic ...
    }
    // When plan.outcome is PrintedFallback, annotate it with rel_path + the spec's fallback
    Ok(annotate_fallback(plan.outcome, rel_path, spec))
}
```

Where `annotate_fallback` maps `PrintedFallback` → `PrintedFallback { hook_file: rel_path,
snippet: fallback_for(spec) }`. The existing `install_claude_hook` becomes a one-line call to
`install_hook_to_file(root, SETTINGS_REL, spec, dry_run)`. `install_codex_hook` is
`install_hook_to_file(root, CODE_HOOKS_REL, spec, dry_run)`.

## D5. Trust surfacing (D2.3)

Codex requires the project `.codex/` layer to be trusted before project-local hooks load.
`boot install` cannot automate this (codex security model).

### D5.1 Output policy

Trust instructions print whenever `RefreshOutcome` is `Wired` or `Refreshed` (a hook was
written or normalised). On `None` (already canonical), trust has already been dealt with. On
`PrintedFallback`, the fallback snippet instruction covers it. On `dry_run`, trust
instructions print as informational (tagged `[dry-run]`).

```
  codex: wired hook: <exec> boot --emit
  codex: wrote .codex/hooks.json. To activate:
    1. Ensure [features] hooks = true in .codex/config.toml.
    2. Start codex in this project and accept the project trust prompt.
    3. Run /hooks in codex to trust the doctrine hook.
```

Step 1 (`[features] hooks = true`) is documented but NOT automated — parsing `.codex/config.toml`
is TOML-merge territory that `boot install` does not enter. If hooks are disabled, the hook
simply doesn't fire; the trust instructions warn the user to check this prerequisite.

### D5.2 Spike hook coexistence warning

The current repo's `.codex/hooks.json` carries a spike entry (shell wrapper, same matcher).
On first `boot install`, `plan_hook` preserves it as foreign — both hooks fire concurrently,
injecting the snapshot twice. `install_refresh` detects this condition (a foreign `SessionStart`
entry with the same matcher exists alongside the newly-written doctrine entry) and prints:

```
  codex: warning — a foreign SessionStart hook with the same matcher
    (startup|resume|clear|compact) exists alongside the doctrine hook.
    Both will fire concurrently. Remove the old entry to avoid
    duplicate snapshot injection.
```

Detection: after writing the canonical hook, re-parse `.codex/hooks.json` and check whether
the `SessionStart` array has >1 entry with the same matcher as our spec.

User-scope (`~/.codex/hooks.json`) is a documented alternative but NOT automated.

## D6. Module boundaries

```
src/boot.rs (existing)
├── run_emit (new — thin shell over regenerate + render_boot)
├── CODE_HOOKS_REL (new const)
├── HookSpec::boot_emit (new constructor)
├── is_doctrine_emit_command (new predicate — suffix-strip)
├── install_hook_to_file (extracted private helper — DRY)
├── install_codex_hook (new — thin caller over install_hook_to_file)
├── install_refresh Codex arm (modified — +hook leg)
└── fallback_for (unchanged — generic over HookSpec)

src/commands/cli.rs
└── Boot { --emit } (new flag, conflicts_with = "check")
```

No new imports. No new modules. `install_hook_to_file` extraction is the only refactor —
behaviour-preserving for the Claude callers.

## D7. Verification

| Criterion | Kind | Evidence |
|---|---|---|
| `install_refresh(Codex)` writes expected hook entry | VT | Unit test: parse the written `.codex/hooks.json`, verify `hooks.SessionStart[0]` shape |
| Idempotent re-run is no-op | VT | Unit test: second `install_refresh` → `RefreshOutcome::None` |
| Foreign codex hook survives merge | VT | Unit test: pre-existing non-doctrine `SessionStart` entry preserved |
| Malformed `.codex/hooks.json` → `PrintedFallback` | VT | Unit test: write bad JSON, run `install_refresh`, assert no clobber |
| Stale exec path refreshed | VT | Unit test: write hook with old path, run with new path, assert updated |
| `boot --emit` regenerates + prints, same bytes to file and stdout | VT | Unit test: capture stdout, compare byte-for-byte against on-disk `boot.md` after `write_if_changed` |
| Poison tolerance (`(deleted)` suffix) | VT | Inherits from existing `is_doctrine_program` tests |
| Emit ownership predicate — full command matrix | VT | Test every row in D4.3 matrix: positive (bare exec, spaced exec, poisoned exec) and negative (boot, boot --emit extra, shell wrapper, sync, stamp, other-tool) |
| Live codex SessionStart injects snapshot | VH | User-run: start codex in this project after `boot install --agent codex`, confirm boot context appears in prompt |
| Codex malformed-hook fallback prints `.codex/hooks.json` + `boot --emit` snippet | VT | Unit test: write bad JSON in `.codex/hooks.json`, run `install_refresh(Codex)`, assert `PrintedFallback` carries `CODE_HOOKS_REL` + emit snippet |
| Trust instructions print on Wired/Refreshed, not on None | VT | Unit test: capture `wire()` output for each outcome, assert trust lines present/absent |
| Spike hook coexistence warning prints after first install | VT | Unit test: install hook into `.codex/hooks.json` with a pre-existing same-matcher foreign entry, assert warning in output |

## D8. Open questions / risks

- **Existing spike hook must be removed manually.** The current `.codex/hooks.json` carries a
  `sh -c 'doctrine boot ...; cat ...'` spike entry. It won't match `is_doctrine_emit_command`
  (wrong suffix), so `plan_hook` treats it as foreign and preserves it. After `boot install --agent
  codex` wires the canonical hook, **both `SessionStart` entries fire** — the snapshot injects
  twice. The design now emits a concurrency warning (D5.2) but does not auto-remove the spike
  entry — the user deletes it manually after seeing the warning.
- **`[features] hooks = true`:** Codex requires this in `config.toml`. Not automated — `boot
  install` documents it. The spike confirmed it was already set.
- **Coexistence with plugins:** Codex loads hooks from all sources additively. A plugin's
  `SessionStart` hook coexists with doctrine's — no conflict.
- **`--emit` output racing with codex hook delivery:** Codex runs hooks synchronously at
  session start and captures stdout. `boot --emit` must stay fast (content-diffed regenerate
  keeps it cheap — sub-1ms when unchanged).
- **Root resolution:** `boot --emit` resolves root from `cwd` (codex runs hooks with the
  session `cwd`). This is the existing `root::find` path — no `git rev-parse` subshell needed.
