# Install artefact ownership

## Context

`doctrine install` writes two kinds of file into a client repo, but treats them
identically:

- **framework-owned** — generated from the binary (skills, agent defs, their
  harness symlinks). The client is not meant to edit them.
- **user-owned** — committed customisation (`doctrine.toml`,
  `project-orientation.md`, exposed hymn starters, authored entities).

Every write is skip-if-exists (SPEC-009 D2), nothing is ever removed, and the
gitignore contract is a hand-kept list. Four open backlog items are symptoms of
that one missing distinction:

| item | symptom |
|---|---|
| IMP-468 | Install projects exactly one agent def per harness (`install_agents_for` matches a single asset). The capsule defs (`capsule-orchestrator`, `capsule-phase-planner`, `capsule-worker`) are embedded but never installed, so `/capsule-driver` ships without the agents it spawns; spawns fall back to `general-purpose`. |
| ISS-216 | A changed agent def is never refreshed (skip-if-exists), and a stale pre-subdir-split symlink can point the harness at an old path. |
| ISS-353 | Install never prunes a skill or agent def whose source was deleted; retired instructions stay invocable. |
| CHR-043 | The default `.gitignore` misses framework-generated paths (`.claude/agents/`, `.agents/skills/`, `.pi/skills`, …); clients hand-patch it. Its listed paths predate ADR-019 and are partly stale. |

Plus two small riders on the same code: CHR-040 (pi symlink points at the old
`codex/` dir) and IMP-181 (`_ → pi` wildcard arms in agent-def install).

ADR-019 already names the axis — **owned** is one of its seven independent asset
properties. This slice makes install act on it.

## Scope & Objectives

1. **Classify** every artefact install writes as framework-owned or user-owned,
   declared once (manifest or equivalent single source) — not inferred per call
   site.
2. **Framework-owned artefacts** are:
   - installed in full per harness — every agent def in the harness's def set,
     not one hardcoded asset (IMP-468, IMP-181);
   - refreshed on re-install when they differ from the embedded source (ISS-216);
   - pruned when their source no longer exists (ISS-353);
   - gitignored, with the gitignore entries derived from the framework-owned set
     rather than hand-listed (CHR-043).
3. **User-owned artefacts** keep today's contract: write-if-absent, never
   overwritten, never pruned. In particular exposed hymn starters under
   `.doctrine/hymns/` are user-owned and must not be blanket-ignored.
4. Fix symlink layout drift: harness symlinks resolve to the current per-harness
   subdir (ISS-216 stale symlink, CHR-040).
5. Revise SPEC-009 D2 (via a Revision) so "never overwrites" applies to
   user-owned artefacts only.

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
- `.doctrine/spec/tech/009/` (SPEC-009) — D2 revision.
- worktree derived-tier classification, if it enumerates installed paths.

## Risks & open questions

- **Overwriting user edits.** A client may have edited an installed agent def.
  Refresh must not silently clobber something the user owns — needs a rule
  (detect divergence? treat edits as user-owned by moving them to a hymn
  overlay?). Design decision.
- **Prune safety.** Prune must only remove paths install provably wrote
  (a ledger, or a strictly framework-owned namespace), never an unknown file in
  a shared dir like `.claude/agents/`.
- **Gitignore of shared harness dirs.** `.claude/agents/` may hold a client's
  own agents. Derived entries must name doctrine's files, or doctrine must own a
  namespace — not ignore the whole dir.
- **Agent defs are template-expanded** (`{{ prompt resolve }}`), so "differs from
  source" means "differs from the expansion", not from raw bytes.

## Verification / closure intent

- Fresh install into a temp project installs all claude agent defs including the
  three capsule defs (VT).
- Re-install after an embedded def changes refreshes the installed copy; a
  user-owned file is untouched (VT).
- Removing a def/skill from the embedded set prunes its installed copy and
  nothing else (VT).
- Every framework-owned path install writes is covered by a shipped gitignore
  entry; no user-owned path (hymn starters, `doctrine.toml`, authored entities)
  is (VT — a by-construction cross-check).
- SPEC-009 D2 revised.

## Summary

Give install an ownership axis (ADR-019 "owned"): framework-owned artefacts are
installed in full, refreshed, pruned and gitignored; user-owned stay
write-if-absent.

## Follow-Ups

- IMP-378 — client migration primitive, which can build on the ownership
  classification to untrack previously committed framework artefacts.
