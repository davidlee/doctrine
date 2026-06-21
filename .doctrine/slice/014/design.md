# SL-014 Design: Codex SessionStart-emit boot wiring

## D1. Current vs Target

**Current:** `boot install --agent codex` writes the `@`-import into `AGENTS.md` (dead — codex
doesn't expand `@`-imports, confirmed live in SL-011 closure) plus pi extensions. No hook
is written.

**Target:** `boot install --agent codex` writes a `SessionStart` hook into `.codex/hooks.json`
with command `<exec> boot --emit`. The hook's stdout (the boot snapshot body) is injected by
codex as developer context per the official docs — zero lag, the proven spike mechanism.

- The `@`-import line in `AGENTS.md` is **untouched** — harmless (codex ignores it; Claude
  uses it). No `AGENTS.md` change.
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

- **`src/boot.rs`: `run_emit(root, exec)`** — calls `regenerate()` then `println!("{}", render_boot(...))`
- **`src/commands/cli.rs`:** `Boot` variant gains `--emit` flag (`conflicts_with = "check"`)

Rides `build_sections` + `render_boot` unchanged. No new section assembly path.

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

### D4.4 Merge behaviour (inherited from `plan_hook`)

- **No existing hook** → append at tail (`Wired`)
- **Owned hook exists, stale exec path** → normalize to one canonical entry (`Refreshed`)
- **Owned hook exists, already canonical** → no-op (`None`)
- **Multiple owned hooks** → drop all, insert one canonical (`Refreshed`)
- **Foreign `SessionStart` hooks** → preserved (entry survives the drop)
- **Malformed JSON / wrong-typed `hooks`/`SessionStart`** → `PrintedFallback`, no clobber
- **Poison tolerance** → `(deleted)` suffix is stripped by `is_doctrine_program` (shared)

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

## D5. Trust surfacing (D2.3)

Codex requires the project `.codex/` layer to be trusted before project-local hooks load.
`boot install` cannot automate this (codex security model). After writing the hook, it prints:

```
  codex: wired hook: <exec> boot --emit
  codex: .codex/hooks.json written. To activate:
    1. Start codex in this project and accept the project trust prompt.
    2. Run /hooks in codex to trust the doctrine hook.
```

User-scope (`~/.codex/hooks.json`) is a documented alternative but NOT automated by this
slice — it's simple enough for users to copy the entry manually if they prefer no per-project
trust step.

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
| `boot --emit` regenerates + prints | VT | Unit test: temp root with governance body, assert stdout == `render_boot(build_sections(...))` |
| Poison tolerance (`(deleted)` suffix) | VT | Inherits from existing `is_doctrine_program` tests |
| Disjoint ownership predicates | VT | Test: emit predicate doesn't match boot/sync/stamp commands |
| Live codex SessionStart injects snapshot | VH | User-run: start codex in this project after `boot install --agent codex`, confirm boot context appears in prompt |

## D8. Open questions / risks

- **Existing spike hook must be removed manually.** The current `.codex/hooks.json` carries a
  `sh -c 'doctrine boot ...; cat ...'` spike entry. It won't match `is_doctrine_emit_command`
  (wrong suffix), so `plan_hook` treats it as foreign and preserves it. After `boot install --agent
  codex` wires the canonical hook, **both `SessionStart` entries fire** — the snapshot injects
  twice. The user removes the spike entry manually or deletes the file before running
  `boot install`.
- **`[features] hooks = true`:** Codex requires this in `config.toml`. Not automated — `boot
  install` documents it. The spike confirmed it was already set.
- **Coexistence with plugins:** Codex loads hooks from all sources additively. A plugin's
  `SessionStart` hook coexists with doctrine's — no conflict.
- **`--emit` output racing with codex hook delivery:** Codex runs hooks synchronously at
  session start and captures stdout. `boot --emit` must stay fast (content-diffed regenerate
  keeps it cheap — sub-1ms when unchanged).
- **Root resolution:** `boot --emit` resolves root from `cwd` (codex runs hooks with the
  session `cwd`). This is the existing `root::find` path — no `git rev-parse` subshell needed.
