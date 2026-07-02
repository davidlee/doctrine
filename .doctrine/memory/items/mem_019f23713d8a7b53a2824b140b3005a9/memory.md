# Claude Code trust: three independent layers

Claude Code gates execution through **three independent layers**, each of which
**fails silently** (no error) when it blocks. Decoded from the 2.1.198 native
binary + confirmed against live `~/.claude.json` / settings state this session.
A plugin/hook can go inert for a reason in ANY layer — diagnose all three.

## Layer 1 — folder trust ("execute anything here at all?")

- Keys: `hasTrustDialogAccepted`, `hasCompletedProjectOnboarding`, stored
  **per-project in `~/.claude.json`** (NOT settings.json).
- Set by the trust dialog on first entry. **Reset:** edit that project's entry
  in `~/.claude.json` (flip/remove `hasTrustDialogAccepted`) → dialog re-fires.

## Layer 2 — plugin enablement ("is this plugin active?")

- Key: `enabledPlugins` map in settings.json, `"name@marketplace": true|false`.
- Project-scope installs are enabled implicitly. Directory-source registration
  itself lives in `~/.claude/plugins/known_marketplaces.json`, not here — see
  [[mem.system.claude.plugin-load-model]] for the full resolution chain and the
  silent "orphaned entry" trap.

## Layer 3 — restricted-mode hook gate (bites hooks specifically)

Decoded (2.1.198 symbol names; unstable across builds):

```js
B_()  = Rl() || Xce()                              // "restricted?"
Rl()  = env CLAUDE_CODE_SAFE_MODE  ||  --safe-mode
Xce() = policySettings.allowManagedHooksOnly===true
      || (settings.disableAllHooks===true && policy doesn't override)
oX()  = Set of policySettings.enabledPlugins where value===true   // MANAGED only
```

When `B_()` is true (restricted), **plugin** hooks fire only if the plugin id is
in **`policySettings.enabledPlugins`** — the *managed/enterprise* settings, NOT
user/project `enabledPlugins`. Miss it and the hook vanishes, no error.

Managed policy file locations probed: `/etc/claude-code/managed-settings.json`,
`~/.config/claude-code/managed-settings.json`, macOS `/Library/Application
Support/ClaudeCode/managed-settings.json`, `/run/claude-code/…`. In the doctrine
jail all four are **absent**, safe-mode unset, `disableAllHooks` unset ⇒
`B_()`=false ⇒ this gate is NOT engaged; every layer sits open.

## Marketplace blocklist (the literal "blackmark")

`~/.claude/plugins/blocklist.json` — a fetched list keyed `plugin@marketplace`
with `reason`/`text`. **github/marketplace plugins only**; a local
`directory`-source plugin never appears here, so it is NOT the cause of a
silent-inert local-fs plugin (that's Layer 2 / registry — see
[[mem.system.claude.plugin-load-model]]).

## Diagnosis order when a plugin/hook is silently inert

1. Layer 1: `~/.claude.json` project entry trusted?
2. Layer 2: marketplace in `known_marketplaces.json` + `enabledPlugins` true?
3. Layer 3: safe-mode / managed `allowManagedHooksOnly` / `disableAllHooks`?
4. Blocklist (github plugins only).
