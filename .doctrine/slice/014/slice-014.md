# codex SessionStart-emit boot wiring

## Context

SL-011 shipped the cache-friendly governance snapshot and its **Claude** wiring
(`@`-import into `CLAUDE.md` + a `SessionStart` hook that regenerates `boot.md`).
The `boot install` **`Harness::Codex`** arm was authored as *import-only*
(`@`-import into `AGENTS.md`, no hook — design D8 / SL-011 §5.2), on the open
assumption (SL-011 §6) that codex inlines `@`-imports the way Claude does.

The SL-011 live codex run **disproved that assumption** (see
`slice/011/notes.md` §"Codex closure finding"):

- Verified with `codex debug prompt-input` (renders the exact model-visible
  prompt, no model call): codex reads `AGENTS.md` verbatim but does **not**
  expand the `@`-import — the snapshot *body* never reaches the model. The codex
  arm is therefore **dead as wired**.
- BUT codex-cli 0.133.0 has a hook system with a **`SessionStart`** event whose
  **stdout is injected as developer context** (`developers.openai.com/codex/
  hooks`). A spike — `.codex/hooks.json` SessionStart →
  `sh -c 'doctrine boot >/dev/null 2>&1; cat <git-root>/.doctrine/state/boot.md'`
  — was **confirmed live**: codex reported the full Doctrine boot context in its
  prompt. This path is *better* than Claude's: stdout injects into the **current**
  session → **zero lag** (vs Claude's ≤2-session `@`-import lag), and it folds
  refresh + injection into one hook.

This slice replaces the dead codex `@`-import path with the proven
SessionStart-emit hook, wired by `boot install`'s codex arm.

## Scope & Objectives

1. **A regenerate-and-emit entry point.** Add `doctrine boot --emit` (sibling to
   SL-011's `boot` / `boot --check`): regenerate the snapshot via the existing
   `regenerate` path, then print the snapshot to stdout — one clean call for the
   hook command instead of a shell `boot; cat`. DETERMINISTIC, no clock (the
   SL-011 cache-key rules carry over). Ride the existing render path; no fork.

2. **`boot install` codex arm writes the SessionStart hook.** Replace the codex
   `import_targets`/refresh with a writer that merges a `SessionStart` entry
   (matcher `startup|resume|clear|compact`, `type: "command"`, command =
   `doctrine boot --emit`) into the codex hook config. Mirror the Claude
   `settings.local.json` hook writer (SL-011 PHASE-04): idempotent, ownership-
   matched, foreign-hook-preserving, fail-soft on malformed config.

3. **Surface the project-layer trust step.** GOTCHA (spike): project-local
   `.codex/hooks.json` loads only when the `.codex/` **layer** is trusted — a
   trust axis SEPARATE from per-hook trust; `--dangerously-bypass-hook-trust`
   bypasses the latter, not the former. The wiring cannot auto-trust the project
   layer (codex security model). `boot install` must print the one-time trust
   instruction (and/or document the user/managed-scope alternative).

## Non-Goals

- Re-opening or changing SL-011's **Claude** path (import + refresh hook) — it
  ships as-is. `AGENTS.md` keeps its `@`-import line: Claude uses it; codex simply
  ignores it (no AGENTS.md change needed — the two harnesses are non-conflicting).
- The SL-011 §5.3 `boot install` **bare-emit** branch (on-PATH → bare `doctrine`)
  — separate low-priority follow-up; the jail ro-bind already stabilises it.
- **Auto-trusting** codex project layers — impossible by design; documented, not
  automated.
- The `Harness` <-> `skills::Agent` **identity unification** — now UNBLOCKED
  (SL-012 is done, the `skills.rs` §3 gate is lifted), but it is its own refactor;
  keep this slice focused. Carried to Follow-Ups.

## Affected surface

- `src/boot.rs` — new `--emit` path (regenerate + print); the `Harness::Codex`
  arm of `install_refresh` rewritten from import-only to the hook writer; a codex
  hook-config merge/ownership helper (sibling to the Claude one).
- `src/main.rs` — `--emit` flag on the bare `boot` verb (alongside `--check`).
- Tests — `install_refresh(Codex)` writes the expected entry; idempotent; merge
  preserves foreign codex hooks; `boot --emit` regenerates + prints.

## Summary

Wire codex through its native `SessionStart` hook (stdout -> developer context),
not the `@`-import codex never expands. Add `doctrine boot --emit` (regenerate +
print), have `boot install`'s codex arm write the hook entry, and surface the
one-time project-layer trust step. The Claude path is untouched; the change is a
focused, spike-validated replacement of the one dead arm.

## Risks, assumptions, open questions

- **Config target:** project `.codex/hooks.json` (separate file, mirrors Claude's
  `settings.local.json`) vs inline `[hooks]` in `config.toml`. Lean hooks.json;
  decide in `/design`.
- **Ownership match:** the SL-011 `rsplit`-on-last-whitespace match keys on
  `<program> boot`. `doctrine boot --emit` ends in `--emit`, not `boot`, and JSON
  hook entries are structured (not a flat command string) — so the codex arm needs
  its OWN ownership rule (match the structured entry / a managed marker), not the
  Claude string match. Design must define it.
- **Sync only:** codex `async` hooks are parsed-but-skipped; the hook runs
  synchronously at session start, so `boot --emit` must stay fast (content-diffed
  regenerate keeps it cheap).
- **Trust friction:** the one-time project-layer trust is unavoidable; assess
  whether a user/managed-scope hook is a better default than project-local.
- **Root resolution:** `boot --emit` resolves its own root from `cwd` (codex runs
  hooks with the session `cwd`), removing the spike's `git rev-parse` subshell.

## Verification / closure intent

- Unit (no disk/model — house rule): `install_refresh(Codex)` produces the
  expected hook entry; re-run is idempotent; a foreign codex hook survives the
  merge; malformed config -> never clobber, print fallback. `boot --emit`
  regenerates then emits the snapshot bytes.
- Live (user-run closure, already spiked): a real codex `SessionStart` injects the
  snapshot as developer context; the project-layer trust step is documented and
  reproducible.

## Follow-Ups

- `Harness` <-> `skills::Agent` unification (SL-012 freed `skills.rs`).
- SL-011 §5.3 `boot install` bare-emit branch (dev-build invocation only).
- Revise SL-011 design §5.4/§6: the "codex has no SessionStart equivalent" /
  "does codex inline `@`-imports" open questions are RESOLVED by this slice.
