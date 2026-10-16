# Install explains itself

## Context

Interactive `doctrine install` (no `-y`) is hard to follow (IMP-516, observed
2026-10-16 on a fresh project):

- The first screen is a raw step list (`create dir` repeated per path, one
  `gitignore +` line per entry), then a bare `Proceed? [y/N]`.
- Forward steps are asked one at a time *after* the base install has run, in
  internal vocabulary: "Materialize shipped memory corpus?", "Wire @-import +
  session hooks…?", "Project exposed hymn starters?", then one question per
  harness.
- Most of those are not real choices: answering N leaves doctrine half-working.

One of those prompts should not exist at all. Install writes the
`[hymns].expose` starter set (`install/manifest.toml`) into `.doctrine/hymns/`
every time (IMP-513). SPEC-009 D8 says customised hymns appear only when first
customised, so install contradicts its own spec; the list is stale (IMP-489) and
ignores the installed harnesses; and the `role/worker` starter replaces the
framework worker contract in every new project (ISS-210).

The complement is a checkout that hasn't run install, or ran an older one
(IMP-514). SL-281 gitignores framework-owned paths, so a fresh clone lacks them
until someone installs, and nothing tells an agent which of three states it is
in: no binary; binary but checkout not installed; install older than the
binary; install *newer* than the binary (an older doctrine on `PATH` than the
one that last installed — e.g. a stale `~/.cargo/bin` shadowing a newer one).

Both are the same concern seen from two moments: install's account of what it
does and what to do next, and the rest of doctrine's account of what install
hasn't done yet. The rendering differs; the model of "what install owns and
whether it's in place" should be shared.

## Scope & Objectives

1. **Plain-English overview first.** Before anything is written, say what will
   happen and why, in user terms, e.g.:
   - a managed block in `.gitignore` keeps generated files out of git;
   - `.doctrine/` is created; whatever in it isn't meant to be customised is
     ignored;
   - a short section at the top of `CLAUDE.md` / `AGENTS.md` makes the agent
     load doctrine's context.
   The raw step list stays available on request ("more detail").
2. **One decision for required work.** A single proceed/abort covers every
   required step (base plan, memory sync, boot wiring, framework convergence).
   Separate prompts remain only for genuine choices: which harnesses, a skills
   subset.
3. **Hand over a commit.** On success, print the next step:
   `git add <the paths install touched> && git commit -m "…"`, naming exactly
   the tracked paths this run created or changed.
4. **Drop the hymn-starter leg from install** (IMP-513). Install writes no
   hymn files. Provide the replacement path: a `prompt` verb that materialises
   the framework body of a named slot as a user-provenance starter on demand.
   Resolves ISS-210 at the root.
5. **Detect a missing or stale install and say how to fix it** (IMP-514).
   - Install records the binary version that last ran it, in a derived,
     gitignored stamp.
   - Doctrine surfaces (boot, `status`, session hook — design picks which)
     compare the stamp to their own version and print one line:
     - absent → run `doctrine install`;
     - stamp older than binary → re-run `doctrine install`;
     - stamp newer than binary → this binary is older than the one that last
       installed the checkout; upgrade it (or fix `PATH`). Re-installing would
       downgrade framework-owned files, so the advice is *not* "re-run install".
   - The no-binary state can only be served by a tracked surface: one short
     line in the managed `CLAUDE.md` / `AGENTS.md` section pointing at the
     install docs when `.doctrine/state/boot.md` is absent. One line at most;
     it costs tokens every session.
   - A library entry (install / recovery) carries the detail.
6. **Shared model, separate renderings.** The install overview (2), the
   commit hint (3) and the staleness line (5) draw on one description of what
   install owns and whether it's present, rather than three ad-hoc lists.
   Design decides its shape; ride the SL-281 harness registry and ownership
   plan (`print_ownership_plan`) before adding anything new.
7. `-y` and `--dry-run` behaviour unchanged except for 4 (no starters written)
   and 5 (stamp written).

## Non-Goals

- Automatic reinstall or reseat on staleness: this slice reports, it does not
  act.
- Refreshing stale reference docs in already-installed clients (IDE-030,
  IMP-315, IMP-378).
- Hymn model coverage and currency (IMP-489).
- Removing already-projected starters from existing clients (needs the
  migration primitive, IMP-378). Note the leftover; don't build it.
- Codex agent defs and the bake format (SL-283).

## Affected surface

- `src/install.rs` — `run` (plan print, confirm, forward steps), `print_plan`,
  `print_forward_summary`, `print_ownership_plan`, `prompt_step`, the
  hymn-starter leg and `project_starters` / `embedded_expose_set`; stamp write.
- `src/boot.rs` and the session hook / `status` path — stamp check and the
  one-line recommendation.
- `install/` — the managed `CLAUDE.md` / `AGENTS.md` section text; a library
  entry for install recovery.
- `install/manifest.toml` — `[hymns].expose`.
- `src/hymns.rs`, `src/commands/prompt.rs` — on-demand starter verb.
- `.doctrine/spec/tech/009` (SPEC-009, Install & distribution) — D8 and the
  interactive-flow requirements; changes land via a Revision (REV).
- `tests/` — install integration tests.

## Risks, assumptions, open questions

- **Bake ordering.** The starter leg runs early on purpose: the worker-def bake
  reads the *disk* hymn corpus, and existing sidecars must be present before it
  renders (the comment above the leg in `run`; ISS-206 double-emit). Removing
  the leg must keep the bake idempotent. SL-283 also changes the bake; sequence
  the two deliberately.
- **Existing clients** keep their projected starters, including the suppressing
  `role/worker` twin, until a migration exists. Decide whether install warns
  about it.
- **Which prompts are real choices?** Assumption: memory sync and boot wiring
  are required; harness selection and skills subset are choices. Confirm in
  design.
- **Stamp location.** It must be derived and gitignored, yet survive long
  enough to be compared; runtime state is `rm -rf`-able by contract, and a
  wiped stamp reads as "not installed". Decide whether that false positive is
  acceptable (it is at least correct advice: re-run install).
- **Install by an older binary.** When install itself finds a newer stamp,
  should it warn, require confirmation (and refuse under `-y`?), or proceed and
  downgrade? Design decides; the downgrade must at least not be silent.
- **Where the staleness check runs.** Every invocation costs latency; boot or
  the session hook may be enough. Each surface that prints it costs tokens.
- **Testing interactive output.** `prompt_step` already has stdin-driven tests;
  the overview needs golden-style assertions on wording that won't be brittle.

## Verification / closure intent

- Interactive install on a fresh repo: overview, one proceed prompt, harness
  choice, commit hint; no hymn files written.
- `-y` install produces the same tree minus `.doctrine/hymns/` starters.
- `prompt resolve --role worker` on a fresh install yields the framework worker
  contract (ISS-210 closed).
- On-demand starter verb writes a user-provenance starter for a named slot.
- Stamp absent → boot/status prints the install line; stamp older than the
  binary → prints the reinstall line; stamp newer than the binary → prints the
  upgrade-binary line; current → silent.
- Install run by a binary older than the stamp does not downgrade silently.
- With no `.doctrine/state/boot.md`, the tracked managed section tells an agent
  doctrine isn't installed for this checkout.
- SPEC-009 reconciled via REV; IMP-516, IMP-513, ISS-210, IMP-514 resolved.

## Summary

## Follow-Ups

- ISS-216-style reseat on stamp mismatch — the stamp gives a trigger; acting
  on it is out of scope.
- IMP-378 — migration to remove legacy projected starters from existing clients.
