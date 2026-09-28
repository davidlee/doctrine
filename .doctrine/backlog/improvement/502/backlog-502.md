# IMP-502: Check shipped doctrine command spellings against the CLI

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

Skills and reference docs keep worked `doctrine …` invocations at the step that
runs them (SL-273 OQ-1: permitted under ADR-005's restate line — a single worked
command is use, not a flag table). The cost is silent drift: a renamed verb,
removed flag, or changed enum leaves stale spellings in the shipped corpus.
CHR-038 was a manual sweep for exactly this.

## Proposal

An always-on test, same shape as SL-273 PHASE-01's `lib:` shipped-roots test:

1. Walk the shipped corpus (skills, `install/*.md`, templates, shipped memories)
   and extract `doctrine …` invocations from fenced code and inline backticks.
2. Validate each against the clap tree (`<Cli as CommandFactory>::command()`):
   subcommand path exists; each `--flag` exists on that leaf; enum values are in
   the value set.
3. Tolerate placeholders (`…`, `<ref>`, `SL-NNN`, `REV-NNN`).
4. Fail on mismatch, naming file:line.

Catches spelling drift only, not semantic misuse — that stays with review.

Origin: user decision 2026-09-28 during SL-273 (option (a): backlog, not a
PHASE-05).
