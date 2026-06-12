# jpi double op-run strips pi's UI; pi 0.79 trust gate then exits silently on a repo with .pi/.agents

## Symptom
`jpi` (jailed-pi) exits 0 immediately, no output — but only in this repo. `op
run -- jailed-pi` also returns 0. Other repos launch fine.

## Two stacked causes

1. **No UI (the exit).** The `jpi` devshell alias was `op run -- jailed-pi`, but
   `jailed-pi` already self-wraps `op run` internally — so it nested two `op run`
   layers. The outer `op run` (esp. with a locked 1Password session) denies pi a
   terminal → pi's `uiContext === noOpUIContext` → `hasUI() === false`.

2. **Trust gate (the silence).** pi 0.79.x added a project-trust gate
   (`core/project-trust.js`). `resolveProjectTrusted`:
   - returns `true` early only if `!hasProjectTrustInputs(cwd)` — i.e. no `.pi/`
     dir and no `.agents/skills/` anywhere up-tree.
   - when inputs exist and no decision recorded, `case "ask"` then
     `if (!hasUI) return false` → **silently untrusted, NO prompt**.
   With empty `$@` (no message) and no UI, pi has nothing to do → exit 0.

"This repo only" = doctrine's `flake.lock` bump pulled pi 0.79.x (the gate), AND
doctrine has both `.pi/` and `.agents/skills/` (trips `hasProjectTrustInputs`).
Other repos pin older pi and/or lack trust inputs. Trust store is
`~/.pi/agent/trust.json` (per-path map, in-jail key e.g. `/workspace/doctrine`).

## Fix
- Drop the redundant outer `op run` in the `jpi` alias (`flake.nix` ~L199) →
  single op restores the UI. The prompt then renders; answer "Trust" once
  (persists to `trust.json`).
- Optionally `defaultProjectTrust: "always"` in `~/.pi/agent/settings.json` to
  skip the prompt entirely for an all-yours `/workspace` jail.

**Why:** nested `op run` is the real footgun — `jailed-*` launchers already
op-wrap; wrapping again strips the tty and degrades pi to a no-UI mode whose new
failure surface (the trust gate) exits silently instead of erroring.

**How to apply:** never double-wrap a `jailed-*` agent in `op run`. When a jailed
pi/agent exits 0 with no output, check (a) op session unlocked, (b) single op
layer, (c) `trust.json` / `defaultProjectTrust`. See
[[mem.pattern.build.jail-target-redirect]].
