# REQ-476: `boot install` selects the Claude settings file it writes from the `[install] claude-settings-scope` key — an absent key, table or `doctrine.toml` all yielding project `.claude/settings.json` — and renders hook commands in the form that file requires on `baked ⟺ gitignored`.

## Statement

The two Claude scopes and what each fixes:

| scope | file | tracked? | command form |
|---|---|---|---|
| `project` (default) | `.claude/settings.json` | yes | portable `${DOCTRINE_BIN:-doctrine}` |
| `local` | `.claude/settings.local.json` | no | baked exec abspath |

The key is the **only** selector — there is no `--scope` flag (SL-250 `DEC-163`),
so the choice is a sticky project fact rather than a per-invocation one, and every
caller inherits it without electing to. The installer announces its target file and
names the key that changes it, before any hook line.

Absence resolves to `project` at three levels independently: an absent key, an
absent `[install]` table, and an absent `doctrine.toml` all yield the same default.

## Rationale

Scope and command form are one decision, not two. The file's tracked-ness dictates
the form on SL-195's `baked ⟺ gitignored` invariant: a committed settings file must
carry no per-machine absolute path (POL-002), and a gitignored one may. Governing
them separately would admit an implementation that honours the key and writes a
host abspath into a tracked file — precisely the breach the invariant exists to
prevent.

Resolution happens *inside* the installer rather than at its call sites. The second
caller is `memory sync install` — the documented ritual after any shipped-memory
edit, and so the highest-frequency install path — and a scope it could forget to
pass is the defect. The cost is one small TOML read per spec; that is not worth a
parameter whose omission is silent.

Delivered by SL-250. Governed by REV-049 (`FR-008`, introduced at reconciliation
from RV-350 `F-1` axes 2 and 3).
