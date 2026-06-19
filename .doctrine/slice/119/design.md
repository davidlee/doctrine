# SL-119 Design: Wire boot snapshot to pi via APPEND_SYSTEM.md

## Current behaviour

`doctrine boot install` delivers the governance snapshot (`boot.md`) via:

| Harness | Mechanism |
|---|---|
| Claude | `SessionStart` hook → `doctrine boot` runs each session |
| Pi | `@.doctrine/state/boot.md` prepended to `AGENTS.md` — bare text reference |

Pi does not resolve `@` references within context files. The line arrives as
literal text. The LLM may not read it; when it does, it costs a round-trip and
duplicate context if `APPEND_SYSTEM.md` later delivers the same content.

## Target behaviour

| Harness | Primary delivery | Fallback |
|---|---|---|
| Claude | `SessionStart` hook → `doctrine boot` (unchanged) | `@` ref in `CLAUDE.md` |
| Pi | `.pi/APPEND_SYSTEM.md` symlink → boot.md | `@` ref in `AGENTS.md` with sentinel guard |
| Pi | `.pi/extensions/doctrine/index.ts` runs `doctrine boot` on `session_start` | — |

Five concrete changes, all in `src/boot.rs`:

1. Boot sentinel in `render_boot`
2. Sentinel guard in `plan_boot_import`
3. APPEND_SYSTEM.md symlink in `install_refresh`
4. Pi extension generation in `install_refresh`
5. `RefreshReport` extended with new legs

---

## 1. Boot sentinel

### Mechanics

New module-level const:

```rust
const BOOT_SENTINEL: &str = "BOOT-SENTINEL: doctrine-governance-snapshot";
```

In `render_boot`, inserted as an HTML comment immediately after the title line:

```rust
out.push_str("# Doctrine Boot Context\n");
out.push_str("<!-- ");
out.push_str(BOOT_SENTINEL);
out.push_str(" -->\n");
```

The sentinel is stable — it never changes across boots. The AGENTS.md guard
references it and never rots.

### Verification

Extend `render_boot_is_byte_deterministic_and_structured` to assert the sentinel
string appears exactly once in the output.

---

## 2. AGENTS.md sentinel guard

### Mechanics

`plan_boot_import` currently writes `@.doctrine/state/boot.md\n` as the
reference line. The reference block expands to a multi-line conditional:

```
@.doctrine/state/boot.md

If you have NOT seen `BOOT-SENTINEL: doctrine-governance-snapshot` anywhere
in your context (system prompt or preceding messages), you MUST read the file
referenced above now. If you HAVE seen it, you MUST NOT — the content is
already in context.
```

The idempotency check (`content.lines().any(|l| l.trim() == reference)`)
matches on the `@` line alone — unchanged semantics. A file with old-style bare
`@` gets rewritten to the new block on next install. A file with the new block
is already present and gets no rewrite.

### Constants

```rust
const BOOT_REF_LINE: &str = "@.doctrine/state/boot.md";

const BOOT_REF_GUARD: &str = "\
If you have NOT seen `BOOT-SENTINEL: doctrine-governance-snapshot` anywhere
in your context (system prompt or preceding messages), you MUST read the file
referenced above now. If you HAVE seen it, you MUST NOT — the content is
already in context.\
";
```

The assembled reference becomes `format!("{BOOT_REF_LINE}\n\n{BOOT_REF_GUARD}\n")`.

### Verification

- Unit: old-style `@` line → `RefAction::Add` with new block
- Unit: new block present → `RefAction::Present`
- Unit: no content → `RefAction::Create` with new block

---

## 3. APPEND_SYSTEM.md symlink

### Mechanics

Pure decision function:

```rust
/// The planned action for .pi/APPEND_SYSTEM.md.
enum SymlinkAction {
    /// Create .pi/ dir then create the symlink.
    CreateDirAndLink,
    /// Create the symlink.
    CreateLink,
    /// Replace existing file/symlink with correct symlink.
    ReplaceLink,
    /// Already correct — no work.
    NoOp,
}
```

Decision table for `plan_append_system(root: &Path) -> SymlinkAction`:

| Condition | Action |
|---|---|
| `.pi/` absent | `CreateDirAndLink` |
| `.pi/APPEND_SYSTEM.md` absent | `CreateLink` |
| Symlink target == `../../.doctrine/state/boot.md` | `NoOp` |
| Anything else at that path | `ReplaceLink` |

Imperative apply `install_append_system(root, dry_run) -> AppendSystemOutcome`:

```rust
enum AppendSystemOutcome {
    CreatedDirAndLink,
    CreatedLink,
    ReplacedLink,
    NoOp,
}
```

Symlink target is a relative path `../../.doctrine/state/boot.md` — portable
within the project tree.

### Integration into `install_refresh`

The pi arm calls `install_append_system` before the extension step. The `.pi/`
directory is created if absent — same pattern as Claude's
`.claude/settings.local.json` which `boot::install_claude_hook` creates
unconditionally.

### Verification

- Unit: all four `plan_append_system` states produce correct actions
- Integration: `doctrine boot install --agent pi` creates symlink
- Integration: idempotent re-run is no-op
- Integration: stale regular file replaced

---

## 4. Pi session_start extension

### Mechanics

Pure decision function — generates the candidate content and compares
byte-for-byte against what's on disk:

```rust
enum ExtAction {
    /// No file — generate.
    Generate,
    /// First line matches our header but content differs — regenerate.
    Regenerate,
    /// Byte-identical to what we would generate — no-op.
    NoOp,
    /// File exists but first line is not our header — foreign, skip.
    SkipForeign,
}
```

Decision table for `plan_pi_extension(root: &Path, exec: &Path) -> ExtAction`:

| Condition | Action |
|---|---|
| No file at `.pi/extensions/doctrine/index.ts` | `Generate` |
| File present, first line matches generated header, content identical | `NoOp` |
| File present, first line matches generated header, content differs | `Regenerate` |
| File present, first line doesn't match header | `SkipForeign` |

Generated file:

```ts
// Generated by `doctrine boot install` — do not edit.
const { execSync } = require("node:child_process");

module.exports = function (pi) {
  pi.on("session_start", () => {
    execSync("/path/to/doctrine boot", { stdio: "inherit", timeout: 5000 });
  });
};
```

`/path/to/doctrine` is the resolved `current_exe()` baked at install time.
`timeout: 5000` (5s) prevents a hung boot from blocking pi startup forever.

Ownership marker (first line of the generated file):
`// Generated by 'doctrine boot install' — do not edit.`

### Integration

`install_pi_extension(root, exec, dry_run) -> ExtOutcome` — creates
`.pi/extensions/doctrine/` directory as needed, writes the file.

### Verification

- Unit: all four `plan_pi_extension` states produce correct actions
- Unit: generated TS contains the baked exec path
- Unit: user-modified file not overwritten
- Integration: `doctrine boot install --agent pi` creates extension
- Integration: idempotent re-run is no-op
- Integration: path change triggers regenerate

---

## 5. RefreshReport extension

### Mechanics

Two new fields on `RefreshReport`:

```rust
struct RefreshReport {
    hook: RefreshOutcome,
    baseref: BaseRefOutcome,
    mcp: RefreshOutcome,
    // New:
    append_system: AppendSystemOutcome,
    extension: ExtOutcome,
}
```

With `NoOp`/`NotApplicable` defaults for the Claude arm. The reporting in
`wire()` prints outcomes for each new leg.

```rust
Harness::Pi => {
    let append_system = install_append_system(root, dry_run)?;
    let extension = install_pi_extension(root, exec, dry_run)?;
    Ok(RefreshReport {
        hook: RefreshOutcome::None,
        baseref: BaseRefOutcome::NotApplicable,
        mcp: RefreshOutcome::None,
        append_system,
        extension,
    })
}
```

### Reporting

Wire output gains two new lines per harness:

```
  pi: created symlink: .pi/APPEND_SYSTEM.md -> ../../.doctrine/state/boot.md
  pi: generated extension: .pi/extensions/doctrine/index.ts
```

Or `no-op` / `replaced` / `skipped (foreign file)` as appropriate.

### Test impact

- ~20 existing test constructions of `RefreshReport` need
  `AppendSystemOutcome::NotApplicable` and `ExtOutcome::NotApplicable` added —
  mechanical, pervasive, but each is a single-line field addition
- New integration tests cover the pi arm

---

## Code impact summary

| Function | Change |
|---|---|
| `render_boot` | +1 line (sentinel after title) |
| `plan_boot_import` | Reference expands from `@` line to multi-line guard block |
| `ensure_boot_import` | No change (pure consumer of `plan_boot_import`) |
| `install_refresh` | Pi arm expanded from 3-line no-op to two new calls |
| `RefreshReport` | +2 fields, defaults for `NotApplicable` |
| `wire` | +2 report lines for pi outcomes |
| New: `plan_append_system` | Pure decision function |
| New: `install_append_system` | Imperative apply |
| New: `plan_pi_extension` | Pure decision function |
| New: `install_pi_extension` | Imperative apply |
| New: `SymlinkAction`, `ExtAction` | Outcome enums |
| New: `AppendSystemOutcome`, `ExtOutcome` | Report enums |

All pure functions have no clock/rng/disk. All imperative functions reach disk
through `fsutil::write_atomic` where applicable (file writes) and `std::os::unix::fs::symlink`
for the symlink.

---

## Verification alignment

| What | How |
|---|---|
| Sentinel in boot.md | Extend `render_boot` unit test |
| AGENTS.md guard idempotency | Extend `plan_boot_import` unit tests |
| Symlink creation + idempotency | New unit + integration tests |
| Extension generation + idempotency | New unit + integration tests |
| Foreign extension not overwritten | Unit test |
| Stale regular file replaced | Unit test |
| Claude arm unchanged | All existing boot tests still green |
| Behaviour-preservation gate | Full `just check` pass |
| `doctrine boot install --agent pi` e2e | Integration test |

---

## Design decisions

| Decision | Rationale |
|---|---|
| Sentinel as stable const | AGENTS.md guard never rots — no per-boot regeneration of the guard line |
| Guard match on `@` line only | Simpler than dual-scan; old-style refs rewrite once |
| Symlink over copy | `doctrine boot` regenerates boot.md; symlink follows live — no stale copy problem |
| Extension over manual step | Pi-native equivalent of Claude's `SessionStart` hook; zero manual steps |
| `.pi/` created if absent | `doctrine install` is the authoritative setup path; symlink+extension are part of that setup |
| Byte-compare for extension idempotency | Generate candidate content, compare against disk — no path parsing needed |
| `execSync` timeout: 5s | Prevents a hung `doctrine boot` from blocking pi startup forever |
| `stdio: "inherit"` on execSync | Boot output goes to pi's terminal; harmless, debuggable |
| First-line header as ownership marker | Simple heuristic for "did we generate this?" without checksums or metadata files |
| `Harness::Codex` renamed to `Harness::Pi` | User-facing naming; mechanical rename across `boot.rs` |

## Risks

- **Dangling symlink window** — between `install` creating the symlink and the
  first `doctrine boot` run, `APPEND_SYSTEM.md` points at nothing. Pi reads it,
  gets empty content, sentinel absent → LLM falls back to reading `@` ref from
  AGENTS.md. The extension runs `doctrine boot` on `session_start` so the
  window closes on first session start.
- **`session_start` fires on every session** — `doctrine boot` is cheap (pure
  projection, write-if-changed), but it does touch disk. Acceptable; can
  throttle later if needed.
- **Symlink target assumes flat project layout** — `../../.doctrine/state/boot.md`
  from `.pi/` is correct for standard pi projects. Nested pi configs would need
  a different relative path — edge case, not addressed.
- **Extension `stdio: "inherit"`** — if pi captures stdout during
  `session_start`, `doctrine boot` output is invisible. Harmless (boot is
  write-if-changed, no useful stdout).
