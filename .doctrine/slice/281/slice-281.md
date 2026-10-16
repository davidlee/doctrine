# Install artefact ownership

## Context

`doctrine install` writes two kinds of file into a client repo, but treats them
identically:

- **framework-owned** — generated from the binary (skills, agent defs, their
  harness symlinks). The client is not meant to edit them.
- **user-owned** — committed customisation (`doctrine.toml`,
  `project-orientation.md`, exposed hymn starters, authored entities).

The base-plan writes are skip-if-exists (SPEC-009 D2); the forward harness legs
already refresh their canon copies and ownership-classify their links, but
undeclared and inconsistently. Nothing is ever removed, and the gitignore
contract is a hand-kept list. Four open backlog items are symptoms of that one
missing distinction:

| item | symptom |
|---|---|
| IMP-468 | Install projects exactly one agent def per harness (`install_agents_for` matches a single asset). The capsule defs (`capsule-orchestrator`, `capsule-phase-planner`, `capsule-worker`) are embedded but never installed, so `/capsule-driver` ships without the agents it spawns; spawns fall back to `general-purpose`. |
| ISS-216 | A stale pre-subdir-split symlink points the harness at an old path and is kept as foreign, never healed. (The "changed def never refreshed" half is already fixed — the canon copy is rewritten every install.) |
| ISS-353 | Install never prunes a skill or agent def whose source was deleted; retired instructions stay invocable. |
| CHR-043 | The default `.gitignore` misses framework-generated paths (`.claude/agents/`, `.agents/skills/`, `.pi/skills`, …); clients hand-patch it. Its listed paths predate ADR-019 and are partly stale. |

Plus two small riders on the same code: CHR-040 (pi symlink points at the old
`codex/` dir) and IMP-181 (`_ → pi` wildcard arms in agent-def install).

ADR-019 already names the axis — **owned** is one of its seven independent asset
properties. This slice makes install act on it.

## Scope & Objectives

1. **Classify** every artefact install writes as framework-owned or user-owned,
   declared once in `install/manifest.toml`: a `[framework] roots` list and a
   `[harness.<name>]` registry (DEC-373). Each harness's def set is enumerated
   from the embed dir, never listed. User-owned is the default.
2. **Framework-owned artefacts** (DEC-372, DEC-374):
   - installed in full per harness — every agent def in the harness's def set,
     not one hardcoded asset (IMP-468, IMP-181); an explicit `--agent` leg is
     integration, not default projection (DEC-372);
   - canon roots (`.doctrine/agents`, `.doctrine/skills`, `.doctrine/workflows`)
     converge exactly to the embedded set on a project install: refresh members,
     remove everything else, report each removal (ISS-353). A global install
     writes and relinks but only reports orphans (DEC-377);
   - canon layout uniform: `.doctrine/agents/<harness>/<def>.md` for every
     harness, claude included;
   - shared link dirs (`.claude/agents`, `.claude/skills`, `.pi/agents`, and
     global equivalents): a symlink resolving into a canon root is doctrine's —
     relinked or removed; a regular file is foreign — kept, reported, flagged when
     it shadows a def (ISS-216 stale symlink, CHR-040);
   - gitignored via a managed `.gitignore` block, rewritten each install, derived
     from the ownership declaration (CHR-043, DEC-375).
3. **User-owned artefacts** keep today's contract: write-if-absent, never
   overwritten, never pruned. Hymn starters under `.doctrine/hymns/` are
   user-owned and never ignored.
4. **Harness vocabulary.** The registry (with `detect` and `boot` per harness)
   replaces both `_ => pi` wildcards, `detect_agents` and `boot_harness_names`;
   an unregistered harness gets no defs or links but keeps npx skills
   delegation, disclosed (DEC-377). Pi skills go to one root —
   the npx `universal` target (`.agents/skills`), never `pi` as well — ending the
   duplicate-skill warnings on pi boot (DEC-373).
5. **Revision** (DEC-376): one REV over SPEC-009 (D2 split by ownership, D4, D5,
   D7 clarification, collateral never-overwrite sentences) and REQ-170 — drafted
   at `/plan`, applied at `/reconcile`.
6. **Dogfood cleanup:** `git rm .doctrine/agents/AGENTS.md` (a
   pre-minimal-projection relic) and its gitignore negation; this repo's
   `.gitignore` carries the generated block; delete the hand-copied capsule defs
   in `.claude/agents/` so install links them.

## Non-Goals

- **Migrating existing clients** — untracking files a client already committed,
  or refreshing stale projected reference docs: IMP-378, IMP-315, IDE-030. This
  slice is additive to the gitignore contract; it does not reach back.
- **Retiring the projected reference-doc model** — SL-242 owns that (it
  `concerns` CHR-043; boundary: SL-242 cleans corpus and orphans, this slice owns
  the gitignore contract and install behaviour).
- **ISS-210** (exposed `role/worker` starter self-replaces) — only constrains
  this slice (starters are user-owned); its fix is separate.
- Capsule lifecycle gaps (ISS-470, ISS-471) — a separate slice.
- Non-Claude skills channel (IMP-406), plugin-channel retirement (IMP-400).

## Affected surface

- `src/install.rs` — plan building (`build_plan`), `install_agents_for`,
  agent-def / symlink materialisation, gitignore steps.
- `install/manifest.toml` — ownership declaration, `[gitignore]` entries.
- `install/agents/**` — the per-harness def sets.
- `.doctrine/spec/tech/009/` (SPEC-009), REQ-170 — the Revision.
- `src/worktree/allowlist.rs` + `.gitignore` — derived-tier classification and
  its parity gate.
- `tests/e2e_claude_install.rs` — closure VT base.
- `src/hymns.rs` — `Provenance { Framework, User }` vocabulary source.

## Risks & open questions

Settled in design:

- **Overwriting user edits** — canon roots are framework-owned and always
  converge; a user's own file in a shared link dir is foreign and kept (DEC-374).
- **Prune safety** — exact convergence inside declared canon roots; recognition
  by link target in shared dirs; no ledger (DEC-374).
- **Gitignore of shared harness dirs** — per-link lines inside the managed block,
  never a whole-dir ignore (DEC-375).
- **Agent defs are template-expanded** — refresh always writes the per-project
  bake, so no divergence comparison is needed.

Remaining:

- Skills written by `npx skills` (`.agents/skills/<id>`) are real copies; they get
  derived gitignore lines but are not pruned (npx owns their lifecycle).

## Verification / closure intent

- Fresh install into a temp project installs all claude agent defs including the
  three capsule defs (VT).
- Re-install after an embedded def changes refreshes the installed copy; a
  user-owned file is untouched (VT).
- Removing a def/skill from the embedded set prunes its installed copy and
  link, and nothing else; a regular file in a shared link dir survives (VT).
- A stale-layout symlink into a canon root is relinked (VT).
- Every framework-owned path install writes is covered by the managed gitignore
  block; no user-owned path (hymn starters, `doctrine.toml`, authored entities)
  is (VT — a by-construction cross-check). This repo's `.gitignore` contains
  every derived entry (VT — parity test).
- Pi install requests one skills root only (VT over the delegate argv).
- SPEC-009 + REQ-170 revised (DEC-376).

## Summary

Give install an ownership axis (ADR-019 "owned"): framework-owned artefacts are
installed in full, refreshed, pruned and gitignored; user-owned stay
write-if-absent.

## Follow-Ups

- IMP-378 — client migration primitive, which can build on the ownership
  classification to untrack previously committed framework artefacts.
- IMP-513 — hymn starters materialise on customise intent, not install.
- IMP-514 — detect a missing or stale install and say how to fix it.
