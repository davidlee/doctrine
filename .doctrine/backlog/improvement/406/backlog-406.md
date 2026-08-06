# IMP-406: Serve non-Claude harnesses from the embed via .agents/skills

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Extend doctrine's binary-sourced skills channel beyond Claude to the
harness-neutral `.agents/skills/<id>/` location, keeping `npx skills add
davidlee/doctrine` only as the fallback for harnesses that need a bespoke layout.

## Why

SPEC-010 currently delegates **every** non-Claude agent to the external
installer, described there as "the universal external installer doctrine does
not reimplement". Measured during SL-250's research round, that delegate costs
more than it looked:

- a full `git clone` of the repo from GitHub **on every install** — Node, `npx`,
  network and GitHub reachability become hard runtime requirements;
- it **discards the embed**, against SPEC-010's own premise that the binary
  carries every skill "with no network fetch and no sidecar bundle";
- installs track published `HEAD`, not the embedded snapshot in the running
  binary — the version skew SPEC-010 already books as an accepted concern;
- it writes a root-level `skills-lock.json` into every client project.

`.agents/skills/` is not a doctrine invention: `npx skills add … --agent
universal` lands there (probe, 2026-08-06), so it is the ecosystem's own neutral
target. Doctrine already maintains its own `.agents/skills/` tree in this repo.

## What SL-250 leaves in place

SL-250 settles its `OQ-2` as `OQ-2b` — rebuilding the binary-sourced canonical
tree plus proven-ownership relative symlinks for Claude, the channel SPEC-010
still specifies but whose code was deleted at `347197e8`. Its scope requires
that mechanism to be **parameterised over the target directory** rather than
hard-coded to `.claude/skills/`.

So the mechanism should already exist when this item is picked up. The work here
is a second *target*, not a second mechanism.

**The content is paid for once.** The canonical `.doctrine/skills/<id>` tree
holds the files; every agent directory holds only *relative symlinks* into it.
So N harnesses cost one copy on disk and one refresh point for currency — adding
`.agents/skills/` adds links, not kilobytes, and cannot drift from
`.claude/skills/`. Verified against the pre-deletion source
(`git show 347197e8^:src/skills.rs`): `canonical_dir` is already agent-neutral
(`:305`) and `claude_links` already takes `agent_dir` as a parameter (`:439`) —
it is misnamed, not Claude-bound.

The steps:

1. Point the same materialise-and-reconcile at `.agents/skills/<id>/`.
2. Decide which harnesses stop delegating — anything that reads the neutral
   location — and which keep `npx` for a bespoke layout. **This needs
   verification per harness: that `--agent universal` writes to
   `.agents/skills/` does not prove any given harness reads it.**
3. Amend SPEC-010 `D2` / `FR-003` (`REQ-175`), which currently reads "Claude
   direct, all others delegated". A REV, not an edit.

## Open questions

- **`OQ-1` — which harnesses actually read `.agents/skills/`?** The load-bearing
  unknown. Establish empirically per harness before dropping its delegate.
- **`OQ-2` — does the `npx` fallback stay for all agents, or only named ones?**
  Keeping it universally available is cheaper than curating a list.
- **`OQ-3` — `skills-lock.json`.** Delegated installs write one; direct-written
  installs would not. Is that divergence acceptable, or does the direct path owe
  an equivalent record?

## Related

- SL-250 — retires the Claude plugin delivery channel; settles the Claude side
  and is expected to leave the mechanism target-parameterised.
- SPEC-010 / PRD-003 — the governing spec pair; `D2` is the clause this changes.
- IMP-245 — Cursor as a doctrine harness; a likely first beneficiary.
- IMP-400 — the parent intent SL-250 carries.
