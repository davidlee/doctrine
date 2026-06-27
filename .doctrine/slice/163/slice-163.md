# check command proxy verb

## Context

The shipped skill corpus hardcodes this repo's own conventions and local state —
a POL-002 platform-independence gap. `plugins/` is embedded via RustEmbed
(`src/skills.rs` `#[folder = "plugins/"]`) and materialised into client projects
by `doctrine claude install`; anything baked into those skills must rest on a
contract doctrine owns, not on what this repo happens to have.

Two concrete couplings:

1. **Task-runner commands.** Five authored skills (`execute`, `close`, `audit`,
   `worktree`, `notes`) tell agents to run `just check` at phase / commit
   boundaries (six sites; **no** `just gate` site exists in shipped skills —
   `/design` correction). A client project has no `justfile` and no `check`
   recipe — the instruction is load-bearing on a host convention this repo owns
   (POL-002 facet 1). POL-002's Scope clause names this exact case: the doctrine
   repo may keep `just gate`, but the *product* must not depend on it.
2. **A non-portable memory uid.** `plugins/doctrine/skills/dispatch/SKILL.md:19`
   cites `mem_019ec65ecbc7` — a doctrine-repo-local memory uid that does not
   exist in a client corpus, so the citation dangles on install. (The `[[mem.…]]`
   wikilinks in other skills all resolve to the shipped `memory/` corpus and the
   per-client install seed `mem.signpost.project.orientation`; those are fine —
   verified. This bare uid is the lone offender.)

Doctrine already owns a config contract for "how this project runs checks": the
`doctrine.toml` `[verification]` table (`src/verify.rs`, parsed through the
shared `src/dtoml.rs` reader), whose `command` field is the project-default base
argv for `VT` coverage evidence. This slice adds a CLI verb that proxy-executes
project-declared check commands sourced from that owned contract, rewrites the
shipped skills off `just` and onto the verb, and scrubs the dangling uid. The
`just check` / `just gate` strings survive only as informing *defaults*, never as
carried correctness.

## Scope & Objectives

What changes, and why:

1. **`doctrine check` verb** — three cadence subcommands that proxy-execute a
   project-configured command and forward its exit status (D1/D2):
   - `doctrine check quick` — per-edit. Default when unconfigured: informative
     no-op `echo` (D4 — never fails a per-edit hook).
   - `doctrine check commit` — per-commit. Default when unconfigured: `just check`.
   - `doctrine check gate` — end-of-phase. Default when unconfigured: `just gate`.
   ("Proxy-execute" = resolve argv from config, spawn it, **inherit** stdio,
   exit with its code — D5.) Middle altitude named `commit` (not `check`) to
   avoid the `doctrine check check` token collision.
2. **Config surface** — three explicit keys (`quick`/`commit`/`gate`) under the
   existing `[verification]` table (`.doctrine/doctrine.toml`); the VT `command`
   key is frozen (D1, INV-1). Rides the single shared `dtoml` reader (no parallel
   parser).
3. **Skill sweep (`just`)** — D6's two-treatment split across the six `just check`
   occurrences in five authored skills (`plugins/doctrine/skills/{execute,close,
   audit,worktree,notes}/SKILL.md`):
   - **4 instruction rewrites** (`execute`/`close`/`audit`/`notes`) → `doctrine
     check gate` — phase/close-boundary gate sites.
   - **2 worktree illustrative-example token updates** — token-only edits that
     preserve project-provided / orchestrator-supplied caller-control semantics
     (CR-F4); not rewritten to a fixed gate call.
4. **Skill scrub (uid)** — remove/replace the dangling `mem_019ec65ecbc7`
   citation in `dispatch/SKILL.md` with portable prose (no repo-local uid).
5. **Re-embed + reinstall** — `cargo build` re-bakes the RustEmbed asset;
   `doctrine claude install` regenerates the installed skills (`.agents/`,
   gitignored) to match source.

Closure intent ("done" judged by):
- `doctrine check {quick,commit,gate}` run the configured command, forward exit
  status, and fall back to documented defaults when the `[verification]` keys are
  absent.
- No `just check` / `just gate` string and no repo-local memory uid remains in
  any authored shipped skill (`plugins/**`); a client install issues only
  `doctrine check …` and carries no dangling memory reference.
- Unit coverage on argv resolution (config-present and absent/default paths) and
  E2E exit-status forwarding; the shipped-surface guard is green; `just gate`
  green on this repo.

## Non-Goals

- **Not** removing this repo's own `justfile` / `just check` / `just gate`
  recipes — sanctioned client habits (POL-002 Scope); the verb's defaults still
  call them here.
- **Not** changing the existing `VT` coverage-verification semantics of
  `[verification].command` (`coverage record`/`verify` through
  `src/coverage_store.rs` + `src/verify.rs::resolve`). If the design reuses that
  field, existing-consumer behaviour is preserved (behaviour-preservation gate).
- **Not** a general user task runner (`doctrine run <name>`); scope is the three
  fixed check altitudes (`quick`/`commit`/`gate`).
- **Not** auto-invoking the gate from other verbs; the verb is called explicitly
  by skills/agents.
- **Not** touching the resolving `[[mem.…]]` wikilinks (they ship/seed fine), nor
  `just` mentions in this repo's own `AGENTS.md` / `CLAUDE.md` / justfile (not
  shipped — the product, not the repo, is the constraint).

## Affected surface

- `src/` — new `commands/check.rs` (verb) + wiring in `src/commands/cli.rs`;
  config read via `src/dtoml.rs` / `src/verify.rs` (possibly a new field).
- `plugins/doctrine/skills/{execute,close,audit,worktree,notes}/SKILL.md` — the
  `just` sweep.
- `plugins/doctrine/skills/dispatch/SKILL.md` — the uid scrub.
- Defaults / docs as the design dictates.

## Risks / Assumptions / Open Questions

- **OQ-1 — config key shape.** RESOLVED (D1): three explicit keys
  `quick`/`commit`/`gate` under `[verification]`; `command` (VT base) frozen.
- **OQ-2 — informing defaults.** RESOLVED: yes — defaults are baked argv literals
  (POL-002 *inform*, never *carry*). A client overrides via config.
- **OQ-3 — absent-command behaviour.** RESOLVED (D3): the baked default spawns;
  spawn `ENOENT` → actionable error naming the owned `[verification].<kind>` key.
  No host-marker sniff (a sniff would itself be the POL-002 facet-1 coupling).
- **A-1.** `plugins/` is the sole authored skill source; `.agents/` is generated
  by `doctrine claude install` (untracked) — verified via `git ls-files`.
- **A-2.** The 10 `[[mem.…]]` wikilinks in shipped skills all resolve to the
  shipped `memory/` corpus or the per-client seed — verified by comm against
  `memory_key`s. Only the bare uid dangles.

## Summary

Add `doctrine check quick|commit|gate` proxying project-declared check commands
from the owned `[verification]` contract, rewrite the shipped skills off `just`
and onto the verb, and scrub a dangling repo-local memory uid — closing a
POL-002 platform-independence gap while leaving this repo's own `just` habits
intact.

## Follow-Ups

- Periodic shipped-surface lint for repo-local couplings (`just …`, bare
  `mem_…` uids, non-portable paths) could be worth automating later.
