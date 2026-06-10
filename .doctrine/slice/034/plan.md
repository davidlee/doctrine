# Implementation Plan SL-034: doctrine-partner skill subset and route comprehension/posture provision

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases along the design's natural fault line. PHASE-01 is *structure +
catalog integrity* — relocate the skills to the canonical domain and ship the
`doctrine-partner` subset, with the discovery-exclusion test as the proof of
no skill-id collision. PHASE-02 is *route provision* — the comprehension row,
the conduct-posture line, the two README accuracy fixes, and the boot
regeneration. Each phase ends green and is its own reviewable commit.

The split is principled, not cosmetic: PHASE-02's route additions name `/pair`
and `/walkthrough` as installed skills, which is only true once PHASE-01 has
moved them into the `doctrine` domain (design D1). Planning them apart keeps
the hard reference honest — the dependency is a real entrance criterion
(PHASE-02 EN-1), not an ordering convenience.

## Sequencing & Rationale

**PHASE-01 before PHASE-02 — the reference must resolve.** Route may
hard-reference `/pair`/`/walkthrough` only because they live in core; relocating
them is the precondition. Doing the route prose first would reference skills not
yet at their canonical home.

**TDD lives in PHASE-01.** The one behaviour-bearing change is the discovery
exclusion (a const extension). Its proof is `discover_excludes_marketplace_only_domains`:
extend the assertions RED-first (doctrine-partner excluded *and* pair/walkthrough
present under `doctrine`), watch it fail, then add `PARTNER_SUBSET_DOMAIN` to make
it pass. Everything else in the phase is file moves and new files that the same
test transitively guards (INV-1 one-entry-per-id).

**The re-embed footgun gates both builds.** A lone edit under `plugins/` or
`install/` does not re-embed on `cargo build` — the embedding crate must
recompile (`mem.pattern.embed.rustembed-recompile-and-symlinks`,
`mem.pattern.distribution.skill-refresh-command`). So each phase's verification
runs `touch src/skills.rs` → `cargo build` *before* it trusts the binary: PHASE-01
before the discovery test reads the moved tree, PHASE-02 before `doctrine boot`
reads the edited `routing-process.md`. Skipping the touch ships stale bytes (R1).

**Boot is golden-safe (design §2, §10).** The boot suite checks the routing
section by presence only (`contains("Route before you act")`), not a verbatim
table sentinel, so the new row updates the snapshot without breaking goldens.
The regeneration is committed as a clean, reviewable diff — the pre-existing boot
drift (`Active Policies` unpopulated) is **not** folded in (R4; a follow-up).

**OQ-1 (b)+ — fix the precedent, don't copy its lie.** Both READMEs are corrected
in PHASE-02 to accurate source-vs-distribution wording. doctrine-memory's current
"duplicated / byte-identical copies … update both copies" describes a copy model
the source does not use (symlinks); mirroring it into doctrine-partner would
propagate a known-false statement. Correcting both is one extra small edit and
retires the reconcile-the-READMEs follow-up outright.

## Notes

- Build target in the jail: `~/.cargo/doctrine-target-jail/debug/doctrine` is the
  fresh binary; the PATH copy and `./target/debug` are stale
  (`mem.pattern.build.jail-target-redirect`).
- Clippy gate is **plain** `cargo clippy` (bins/lib) — `--all-targets` lights up
  `unwrap_used`/`expect_used` denials in test code.
- Symlink form to copy verbatim: `record-memory -> ../../doctrine/skills/record-memory`
  (`ls -la plugins/doctrine-memory/skills/`).
- Net-diff sanity at PHASE-01 close: `git status` must show no `plugins/partner/`.
