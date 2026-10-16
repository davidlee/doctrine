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

Decisions settled in design: DEC-378 to DEC-385 (`doctrine show DEC-NNN`).

1. **Plain-English overview, one prompt** (DEC-381). Install prints an
   overview of what it will do and why, then `Proceed? [y/N/d]`; `d` shows the
   existing step printers. That prompt covers every required step; the
   memory-sync, boot-wire, hymn-starter and per-harness prompts go, with
   `prompt_step` and the `a` answer. Real choices: harnesses via `--agent`
   (asked once only when none is detected or given); skills via flags.
2. **One harness resolver** (DEC-381). Boot wiring and convergence consume the
   same set; boot detection's `AGENTS.md` rule moves into it.
3. **Commit hint** (DEC-382). Print `git add <paths> && git commit -m "…"` for
   exactly the tracked paths this run created, changed or removed.
4. **Hymn starters on demand** (DEC-384). Remove the install starter leg and
   `[hymns].expose`; add `doctrine prompt customise <slot>`; install removes
   (and reports) legacy starters byte-identical to their framework body with
   no sidecar or only the default self-replace. Resolves ISS-210, ISS-521.
5. **Version currency** (DEC-378, DEC-379, DEC-380).
   - Tracked `installed-version` in `doctrine.toml` `[install]`: the repo's
     version, an advisory team pin.
   - Gitignored `.doctrine/state/install.toml`: this checkout's install.
   - Pure leaf `install_state` classifies and owns the advisory text; the
     session hook (`prompt resolve`), `doctrine boot` / `boot --check` and
     `status` print it when not current, never inside snapshot bytes.
   - Install run by a binary older than the stamp warns and asks; under `-y`
     it refuses without an override flag.
6. **Not-installed clause** (DEC-383). One sentence added to the boot guard;
   prior guard texts upgrade in place.
7. **Shared install plan** (DEC-379). `InstallPlan` composes the existing
   base plan, forward steps, ownership actions, gitignore change and stamp
   writes; overview, detail view and commit hint render from it.
8. **One Revision** (DEC-385) across SPEC-009, SPEC-011 REQ-185 and SPEC-023
   REQ-322/329, landed at reconcile.

## Non-Goals

- Automatic reinstall or reseat on staleness: this slice reports, it does not
  act.
- A hard version pin: currency stays advisory (POL-002).
- Refreshing stale reference docs in already-installed clients (IDE-030,
  IMP-315, IMP-378, SL-242).
- Hymn model coverage, currency and model-key vocabulary (IMP-489 incl.
  concern 5, ISS-491).
- Codex agent defs and the bake format (SL-283).

## Affected surface

- `src/install.rs` — `run`, `run_forward_steps`, the printers, `prompt_step`
  (removed), the starter leg (removed), `project_starters` (reused by the verb),
  `detect_agents` / `install_harnesses` (one resolver), stamp writes,
  downgrade gate, legacy-starter removal, `InstallPlan`.
- `src/install_state.rs` (new leaf) — `Currency`, `classify`, `advisory`;
  registered in `.doctrine/adr/001/layering.toml`.
- `src/boot.rs` — `BOOT_REF_GUARD`, `plan_boot_import` prior-text upgrade,
  `boot` / `boot --check` advisory, harness detection rule.
- `src/status.rs` — currency field and line.
- `src/commands/prompt.rs` — `customise` verb; advisory after unbanded resolve.
- `src/dtoml.rs`, `src/install_config.rs` — `installed-version` key;
  comment-preserving write.
- `install/manifest.toml` — `[hymns].expose` removed.
- `Cargo.toml` — `semver` direct dependency.
- `.doctrine/hymns/` (this repo) — identical starters removed by install.
- SPEC-009, SPEC-011, SPEC-023 via one REV.
- `tests/` — binary-driven install e2e.

## Risks, assumptions, open questions

- **Bake ordering.** Legacy-starter removal runs where the starter leg ran,
  before convergence bakes worker defs from the disk corpus. SL-283 also
  changes the bake; land SL-282 first.
- **First write into user-edited `doctrine.toml`.** Must preserve comments and
  formatting exactly; test against this repo's heavily commented file.
- **Install deletes tracked files** (identical starters). Identity check must be
  exact; edited files and non-default sidecars are never touched.
- **Rollout noise.** Whoever upgrades first sees the newer-than-repo line until
  they commit the stamp.
- **Same-version builds** are indistinguishable; behaviour-changing slices get
  at least a patch release (release discipline).
- **Message tests.** `wire()` writes stdout directly; assert messages in
  binary-driven e2e tests.

## Verification / closure intent

- Fresh repo, interactive: overview, one `Proceed?`, commit hint; no hymn
  files written; tracked stamp and local marker written.
- `-y` prints the overview and asks nothing; `--dry-run` prints overview and
  details and writes nothing.
- AGENTS.md-only repo: the codex boot import and codex skills come from one
  harness set.
- `prompt resolve --role worker` on a fresh install yields the framework worker
  contract (ISS-210).
- `prompt customise <slot>` writes a starter + self-replaces sidecar; refuses a
  sealed slot; leaves an existing file alone.
- Install in this repo removes the five identical starters, `preamble/core`
  and `adherence/low`; `adherence/low` emits once (ISS-521).
- Currency matrix: local marker absent / older, tracked stamp older / newer,
  unreadable, current — each prints the DEC-380 line (or nothing) via session
  hook, `boot`, `status`; snapshot bytes unchanged.
- Downgrade: interactive warns and asks; `-y` refuses without the override.
- Existing guard block upgrades to the new text; a current block is untouched.
- `doctrine.toml` comments survive the stamp write byte-for-byte outside the
  key.
- One REV reconciled; IMP-516, IMP-513, IMP-514, ISS-210, ISS-521 resolved.

## Summary

## Follow-Ups

- ISS-216-style reseat on stamp mismatch — the stamp gives a trigger; acting
  on it is out of scope.
- IMP-378 — the tracked stamp can key once-per-repo migrations.
- IMP-489 concern 5 — model-key vocabulary for `prompt resolve --model`.
