# IMP-251: prompt resolve: explicit --band suppresses universal boot prefix

## Problem

`doctrine prompt resolve` unconditionally prepends the universal boot snapshot
(~430 lines) before the axis hymns tail (src/commands/prompt.rs:128-132). The
`--band` flag filters only the hymns tail, not the prefix — so
`resolve --role orchestrator --band model` still emits 419 lines, not "just the
model band."

This contradicts the boot floor-directive's own intent ("run `resolve --band
model` to load your model band") and the flag help ("Restrict output to specific
bands").

## Why it matters

The boot prefix is only "free" when it sits at **literal byte zero** of the
caller's context window (prefix-cache hit). Any caller that already has boot at
zero (interactive claude via `@`-import; a session that injected the full
cascade once) pays real, uncached tokens for the duplicate copy when a
band-scoped resolve re-emits it mid-stream. The `--band` user is precisely the
caller asking for *only the delta* — and gets the whole prefix anyway.

## Fix — fork A (chosen)

When `--band` is explicitly non-empty, suppress the universal boot prefix; emit
only the filtered hymns. Empty `--band` (the spawn/bake path) keeps current
behavior — full snapshot ++ hymns.

Rationale: the flag already means "restrict output"; boot-always-on contradicts
that. Rejected fork B (`--no-boot` flag) as extra surface for the same effect.

Watch: keep INV-D1 axis-invariance intact — the suppression is a stdout-shaping
concern, orthogonal to the idempotent on-disk boot.md write on line 129 (that
side effect must stay).

## Scope

One function (`PromptCommand::Resolve` dispatch), one behavior test
(`--band X` output excludes the boot sentinel). Backlog-sized — no architecture
move, no ADR touched.

Originates from SL-191 follow-up (prompt cascade delivery).
