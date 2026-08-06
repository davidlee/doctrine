# DEC-171: REV targets SPEC-011 only; SPEC-010 is verified

## Decision

SL-250's REV amends **SPEC-011 / REQ-186 alone**. SPEC-010 does not enter the
REV; its pre-existing divergence closes by **conformance**, verified at close.

## Why SPEC-011 is the target

`REQ-186` (SPEC-011 `FR-006`) reads:

> `boot install` merges a `<exec> boot` SessionStart hook into Claude
> settings.local.json, refreshing a stale owned copy and preserving every foreign
> hook and key.

This slice invalidates it on three axes:

- **Not one hook.** Six `HookSpec`s across five events ([[DEC-162]]).
- **Not `settings.local.json`.** A scope-selected default of project
  `.claude/settings.json`, remembered in `doctrine.toml` ([[DEC-163]]).
- **Plus a new obligation.** The abandoned-scope sweep ([[DEC-164]]), which
  REQ-186 says nothing about.

Research `X2` had already relocated the governance target here; reading the
requirement text confirms it.

## Why SPEC-010 stays out

`OQ-2b` restores exactly what SPEC-010's responsibilities 3–6 already describe —
the derived canonical `.doctrine/skills/<id>` tree, the relative agent symlink
reconciled by proven ownership, `.tmp-<id>` staging with remove-then-rename, and
the self-enforced `.doctrine/skills/*` gitignore. The spec becomes **true again**
rather than needing new text.

[[DEC-166]]'s parameterisation sits below spec altitude: one target driven still
"reconciles a relative agent symlink into it". And `OQ-2` left the `npx` delegate
unchanged for non-Claude agents, so SPEC-010's dual-path `D2` survives intact.

A REV entry that changes no text is ceremony. Standing up a separate REV or slice
to close a divergence *this* slice closes as a side effect is worse — and the
divergence is already captured durably as observation `019fd685`, which recorded
that it was found only by reading git history.

## Consequence for the slice card

SL-250's closure criteria currently read *"SPEC-010 amended through a REV to
describe the surviving channel set"*. That is wrong on its face under this
decision and is rewritten as a **conformance claim**: SPEC-010 responsibilities
3–6 verified true of the restored code.

This is the stronger criterion of the two. An amendment edits the text to fit
whatever was built; a conformance claim requires the built thing to be checked
against text written before it.

Recorded from design run `dr-019fd692` checkpoint `cp-8` disposing `inq-10`.
