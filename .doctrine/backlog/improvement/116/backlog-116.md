# IMP-116: Deliver boot snapshot to pi via APPEND_SYSTEM.md

## Problem

`doctrine boot install` prepends `@.doctrine/state/boot.md` to `AGENTS.md` as
the sole delivery mechanism for the codex (pi) harness. Pi does not resolve `@`
references within context files — the line arrives as literal text. The LLM must
choose to read the file (fragile, costs a round-trip), and may not.

The Claude harness gets a `SessionStart` hook that runs `doctrine boot`;
codex is deliberately import-only (no hook). The `@` reference alone is
insufficient.

## Design

Pi natively supports `.pi/APPEND_SYSTEM.md` — content is auto-appended to every
session's system prompt with zero round-trips. `doctrine boot install` should
generate this file (full boot snapshot content) as the primary pi delivery
vehicle, under the existing `Harness::Codex` arm in `boot::install_refresh`.

Per-harness delivery after:

| Harness | Primary | Fallback |
|---|---|---|
| codex (pi) | `.pi/APPEND_SYSTEM.md` — full boot content | `@` ref in `AGENTS.md` |
| claude | `SessionStart` hook → `doctrine boot` | `@` ref in `CLAUDE.md` |

The existing `@` import in context files stays as the portable fallback and
coordination trigger.

## Refinements

### 1. Double-token-tax guard

When APPEND_SYSTEM.md works (boot content already in system prompt), the `@`
ref in AGENTS.md is redundant — the LLM wastes tokens reading a file it already
has. The guard: a boot sentinel + AGENTS.md conditional.

Boot snapshot carries an unmistakable sentinel (e.g. a version-stamped UUID
or a distinguished marker like `BOOT-SENTINEL: <hash>`). AGENTS.md carries a
line near the `@` ref:

> If you have not seen `BOOT-SENTINEL: <hash>` in your system prompt, you MUST
> read `@.doctrine/state/boot.md` now. If you have, you MUST NOT.

This way both paths work: pi sessions (APPEND_SYSTEM.md delivers the sentinel,
no read needed) and bare-codex / other harnesses (saw nothing → reads the file).

### 2. APPEND_SYSTEM.md as a symlink

Rather than copying boot content into `.pi/APPEND_SYSTEM.md` (which would
stale), point it at the live snapshot:

    .pi/APPEND_SYSTEM.md → ../../.doctrine/state/boot.md

Pros:
- `doctrine boot` regenerates boot.md → APPEND_SYSTEM.md is always current
- No separate file-copy refresh hook needed
- `doctrine boot install` creates the symlink once; `doctrine boot` does the rest

Cons:
- Symlink dangles before first `doctrine boot` (acceptable; install creates the
  link, boot regenerates before the next session)
- Pi must resolve symlinks when loading APPEND_SYSTEM.md (it does — standard
  filesystem semantics)

### 3. Regeneration trigger: pi extension on session_start

The symlink ensures delivery freshness, but boot.md itself must be regenerated
when governance changes. The Claude harness gets `hooks.SessionStart: "doctrine
boot"` — automatic per-session regeneration. Pi has no native shell-hook
setting, but its extension system supports `session_start` events.

`doctrine boot install` should generate a small pi extension at
`.pi/extensions/doctrine-boot-refresh.ts`:

```ts
import { execSync } from "node:child_process";

export default function () {
  // Resolve doctrine relative to cwd — `doctrine boot install` writes the
  // resolved path into the generated extension file.
  const bin = "./target/debug/doctrine";
  execSync(`${bin} boot`, { stdio: "inherit" });
}
```

Registered on `session_start` via pi's standard extension loading. This is the
pi-native equivalent of Claude's `SessionStart` hook — zero manual steps, same
guarantee: boot.md is fresh at every session start.

Alternative (simpler, but manual): no extension — the AGENTS.md sentinel guard
says "if boot.md is stale, run `doctrine boot` and re-read." This costs an
extra round-trip but avoids shipping an extension. Worth comparing.

### 4. Conditional installation

Only create `.pi/APPEND_SYSTEM.md` when pi is the detected target:
- `.pi/` directory exists in the project root, OR
- `--agent codex` explicitly given, OR
- `.codex/` directory exists

Otherwise the file is dead weight. This reuses the existing harness-detection
logic in `boot::resolve_harnesses`.
