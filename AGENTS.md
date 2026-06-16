@.doctrine/state/boot.md
# jail

if `/workspace` exists, you're in a bubblewrap jail with the system defined in flake.nix,
including some additional readonly repos mounted ro at `/workspace` plus my ro 
`~/.cargo/bin/doctrine` - if you need a rw doctrine use the build target.

If you need something else that's missing, STOP and ask the User.

# bootstrap doctrine

**Start EVERY substantive task with `/route`** — it chooses the governing skill
before you inspect files, run commands, or write code. The routing table, core
process, and guardrails ride the boot snapshot (`@.doctrine/state/boot.md`,
inlined above), so they are not recited here.

The CLI is the source of truth for command shapes — `doctrine --help` (dev:
`./target/debug/doctrine --help`, off-PATH after `cargo build`). Don't guess
ids or flags; ask the CLI. Durable knowledge lives in doctrine's own memory
(`doctrine memory record|find|retrieve`), not Claude's — the index is in the
snapshot's Memory section; `/record-memory` and `/retrieve-memory` wrap it.

---

## storage model (the storage rule)

Three tiers — know which one you're writing:
- **authored** (`*.toml` + `*.md` under `slice/nnn/` and `adr/nnn/`): committed,
  diffable, reviewed. Structured data in TOML; prose in MD; **never queried/derived
  data in prose.** ADRs are authored entities too — status lives in `adr-nnn.toml`.
- **runtime state** (`.doctrine/state/`, the `phases` symlink, `handover.md`,
  `boot.md`): GITIGNORED, disposable, `rm -rf`-able. Progress lives here, never in
  authored files.
- **derived**: regenerable indexes/caches — gitignored.

## conventions

(`/route`'s digest already carries: no code without an approved plan; use the
CLI, don't guess; immutable `PHASE-NN` / `EN-/EX-/VT-` ids; TDD red/green/refactor.
These are the project-specific additions.)

- **frequent conventional commits**; scope with the slice id, e.g.
  `fix(SL-004): …`, `doc(SL-005): …`, `plan(SL-005): …`. Commit on `main`.
- **reference form** — cite entities by their prefixed canonical id everywhere
  (prose, commits, comments): `SL-020`, `ADR-004`, `PRD-010`, `REQ-060`,
  `RSK-004`, `ASM-001`. The id is identity; the slug is never authoritative. Cite
  the **durable** id, never a mobile membership label (`FR-`/`NF-` move per spec —
  use the `REQ-NNN` they label).
- **ask, don't infer.** correctness comes first and last.
- **pure/imperative split** (slices-spec § Architecture): no clock, rng, git, or disk
  in the pure layer — pass them in as inputs (the date/uid pattern). Impurity lives in
  the thin shell.
- **behaviour-preservation gate**: when changing shared machinery (the entity engine),
  the existing suites are the proof — they must stay green unchanged.
- **lint as you go** — `cargo clippy` or `eslint` zero warnings; `just gate` before every commit
  (`just check` is the fast inner-loop variant — root package only, skips the cordage
  workspace crate; `gate` runs `--workspace`). (The gate runs plain `cargo clippy` —
  bins/lib only; do NOT use `--all-targets`, which lights up
  `unwrap_used`/`expect_used` denials in test code.)
- **no parallel implementation** — ride existing seams; find duplication before writing.

## environment

nixos; bubblewrap jails (mounted into /workspace/*).

- Always use READ tool *before* writing any substantial edit (e.g.
  filling a template, writing `handover.md`) to avoid expensive write
  failure. `cot`, etc do NOT count!
- default reviewer: codex mcp - use default (GPT-5.5) for external
  adversarial reviews. Opus sub-agent is also useful for variety on
  subsequent passes.
