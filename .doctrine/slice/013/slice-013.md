# memory skills install ergonomics + off-script skill-port record

## Context

This session ported the spec-driver skill set into the `doctrine` plugin
**off-script** — no governing slice — and added a marketplace subset plugin. This
slice gives that work a governing artefact retroactively and scopes the one
deferred code change: a `--only-memory` convenience flag for
`doctrine skills install`.

What already landed on `main` this session (skill authoring + plugin manifests +
one CLI fix):

- **18 process skills ported** spec-driver → doctrine: route, slice,
  spec-product, spec-tech, design (+ `SKILL.compact.md` experimental variant),
  plan, phase-plan, preflight, execute, audit, canon, inquisition, record-memory,
  retrieve-memory, next, consult, notes, close — plus a `dispatch` placeholder.
  Vocabulary translated (DE/DR/IP/delta → slice / `design.md` / `plan.toml` /
  EN-EX-VT criteria), CLI verbs verified against the binary, the skill-reference
  graph closed, storage rule honoured. superpowers skills used as an authoring
  reference only (no cross-links — they are not installed alongside).
- **plugin.json + marketplace.json** descriptions synced to the full set.
- **doctrine-memory**: a marketplace-only subset plugin exposing just
  record-memory + retrieve-memory, to install the memory layer without all of
  doctrine. Its skill dirs are relative **symlinks** into the canonical
  `doctrine` domain (DRY, no copy drift; git stores them as real symlinks). The
  CLI embed scans all of `plugins/` and rejects duplicate skill ids, so discovery
  skips `MARKETPLACE_ONLY_DOMAINS` (`doctrine-memory`); a regression test guards it.

Durable decisions taken this session:

- Compression is case-by-case; this port was **translate-not-compress** (keep the
  source's words and FSMs). `design` carries a compact variant for later A/B.
- Description style: **triggers-first** (superpowers CSO), keeping "MUST" / gate
  teeth only where warranted (`route`).
- `/slice` derives from spec-driver `scope-delta`; `/canon` from its `doctrine`
  skill; `/inquisition` from the user's gist (persona kept verbatim in spirit).
- Heavy retargets — `audit` and `close` — drop machinery doctrine lacks (no AUD
  entity, spec registry, `complete`, or sync/validate surface): hand-authored
  `audit.md`, hand-edited lifecycle status. Each carries an explicit tooling-gap
  note so the limitation is visible, not silent.

## Scope & Objectives

1. **Record** the off-script skill port as durable doctrine history (this scope
   doc + `notes.md`).
2. **`--only-memory` flag** for `doctrine skills install` — convenience sugar
   selecting record-memory + retrieve-memory. The capability already exists via
   `--skill record-memory --skill retrieve-memory`; this is ergonomics only.
   **Design outcome (design.md D1):** implement it; derive the id set from the
   `doctrine-memory` subset plugin (mechanism B), not a hardcoded CLI list.
   Mutually exclusive with `--skill`/`--domain`; empty-derivation bails.
3. **Resolve doctrine-memory marketplace viability**: confirm Claude Code
   dereferences within-marketplace symlinks at install.
   **Design outcome (design.md §6, D4): RESOLVED — keep.** Claude Code clones the
   whole marketplace repo locally, so the relative symlinks resolve at install;
   the subset plugin and `MARKETPLACE_ONLY_DOMAINS` guard stand. No deletion.
   Residual confidence is soft → a manual install-smoke (VT-05) backs it.

## Non-Goals

- `dispatch` / parallel execution (tracked as a placeholder skill).
- Skill pressure-testing (the writing-skills "Iron Law") — a separate validation
  pass, deferred.
- The user-customizable governance surface (parked; folded into SL-011 per its
  memory note).
- A general skill group/tag taxonomy beyond what `--only-memory` needs.

## Summary

Mostly a record plus a small ergonomics change. The skill port already shipped on
`main`; this slice governs it after the fact and scopes the deferred CLI flag and
the marketplace-symlink decision. Low risk — the flag rides existing
`--skill`/`--domain` filtering; the marketplace question is a test-then-decide.

## Follow-Ups

- Skill validation pass: route a real task through the graph; `/inquisition` per
  skill.
- Decide the skill-grouping mechanism if `--only-memory` should not hardcode ids.
- Customizable governance surface (SL-011).
- `dispatch` skill (parallel execution).
