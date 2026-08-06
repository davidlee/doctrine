# IMP-407: Doctor leg: name the layer blocking Claude hook activation

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Carved out of SL-250 (*Retire the Claude plugin delivery channel*) by user
decision, 2026-08-06, during that slice's design run. SL-250 ships the direct
hook write and the skills channel; the diagnostic leg that explains an inert
install lands here instead.

Two legs, both `doctrine doctor` work, both reading hook activation state:

### Leg 1 — the activation walk

A check that walks the diagnosis order the trust memory
(`mem_019f23713d8a7b53a2824b140b3005a9`, *Claude Code trust: three independent
layers*) establishes, and names the **blocking layer** rather than reporting
"hooks not working":

1. `~/.claude.json` project entry — is the folder trusted?
2. Marketplace registered in `known_marketplaces.json` + `enabledPlugins` true?
   (Only while a plugin channel survives — SL-250 keeps the manifest published.)
3. Safe-mode env / `--safe-mode`; managed `allowManagedHooksOnly` /
   `disableAllHooks` / `strictPluginOnlyCustomization` — probe
   `/etc/claude-code/managed-settings.json`,
   `~/.config/claude-code/managed-settings.json`, the macOS
   `/Library/Application Support/ClaudeCode/` path, `/run/claude-code/`.
4. `~/.claude/plugins/blocklist.json` — github/marketplace plugins only; a
   `directory`-source plugin never appears there.
5. Doctrine's own hook entries present, canonical, and sole in the settings file
   it owns.

### Leg 2 — keep the published manifest honest against the registry

SL-250 creates a second source of hook truth. After it lands, the `HookSpec`
registry in the binary is what direct-write emits, while
`plugins/doctrine/hooks/hooks.json` remains published as the
`strictPluginOnlyCustomization` escape hatch. Nothing keeps the two in step.

**They have already drifted once, before the slice even starts:** the plugin
ships `${DOCTRINE_BIN:-doctrine} boot --emit` on matcher `*`, while the live
constants are `prompt resolve --role orchestrator` (`src/boot.rs` `RESOLVE_EMIT_ARGS`)
on `startup|clear` (`SESSION_MATCHER`). A user who takes the documented escape
hatch gets the stale form silently.

A conformance check comparing the manifest against the compiled registry closes
that. It is the same shape as the existing `SpawnSeamSymmetry` check, which
already parses that file.

## Constraints carried over from SL-250

- **`R7` — the doctor can verify plausibility, not activation.** `/hooks` is the
  only surface reporting which hooks are live and which file each came from, and
  it is interactive-only; there is no programmatic query. This leg can confirm
  that the settings file doctrine wrote is present, canonical and sole — it
  **cannot** confirm the harness loaded it. Acceptance criteria must not promise
  otherwise.
- **`R2` / POL-002 — feature-scoped capability declaration.** Layers 1–4 read
  per-user files outside the project entirely. POL-002 facet (3) permits this
  when the dependency is *declared*: the check must be opt-in, no default path
  may acquire it, and an absent file must yield a descriptive finding naming what
  was missing — never a crash. **No engine code reads `~/.claude*` today**, so
  this sets the precedent.
- **`ADR-011` `D3`** — a Claude-only doctor leg is an honest per-harness
  capability-altitude row, not a violation.

## Why it was safe to carve out

SL-250's other legs do not depend on it, and the one place they touch is
smaller than the slice assumed:

**SL-250's `R6` is not forced.** That risk reads "retiring the plugin blinds
`SpawnSeamSymmetry` and reds its live-config regression test", because the check
parses `plugins/doctrine/hooks/hooks.json` (`src/doctor_checks.rs:622`)
*precisely because* it is the authored shipped source. But SL-250's Non-Goals
keep the `plugins/` tree and the published marketplace manifest, and `R9`'s
mitigation actively **requires** the plugin to keep working as the
managed-policy escape hatch — which requires its hooks. So the file survives,
the check keeps its input, and nothing reds. What SL-250 leaves behind is not a
blinded check but the un-policed drift Leg 2 addresses.

## Prior art / references

- SL-250 — the slice this was carved from; ships the activation this diagnoses.
- IMP-400 — the parent intent. Its Doctor-leg section is the origin of Leg 1;
  IMP-400 stays open on its own migration question regardless.
- `src/doctor_checks.rs` — the check contract: a free
  `pub(crate) fn <topic>_findings(root: &Path) -> Vec<Finding>` plus a `Category`
  variant with `severity`/`ordinal`/`display_name` and a `CATEGORIES_BY_ORDINAL`
  row. Smallest precedent: `raw_label_findings` (`:38-58`). Tested three ways —
  pure-core unit tests over fixtures, live-config regression locks against the
  real repo tree, and `tests/e2e_doctor_golden.rs`.
- Naming precedent (research thread 2): `hook_settings_findings` +
  `Category::HookSettings` + `CATEGORY_NAME_HOOK_SETTINGS`.
- The per-user layers are **fixture-driven** in test — they cannot be mutated in
  a test run.
- RFC-018 — Claude harness field notes; the home for empirical findings.
- `doctor` has no governing spec (research thread 2, open governance question 2):
  SL-168 built the verb and check legs accreted without one. A pre-existing gap
  this item inherits rather than creates.
