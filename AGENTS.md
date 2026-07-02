@.doctrine/state/boot.md
If you have NOT seen `BOOT-SENTINEL: doctrine-governance-snapshot` anywhere in your context (system prompt or preceding messages), you MUST read the file referenced above now. If you HAVE seen it, you MUST NOT — the content is already in context.
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

# git

Do not stash. *never* checkout or otherwise discard uncommitted work.
Assume multiple agents are working in the same repository, and use `/worktree` 
or `/dispatch` accordingly.

commit as soon as work is coherent; git add specifies paths, don't use -A unless asked.

the main worktree stays on edge. DO NOT checkout the primary working tree
to another branch or i WILL END YOU. If auditing / closing a feature, land it on 
a worktree and push to main. 

DO NOT USE `git checkout <ref> --`

The thing to watch with the edge/main split: dispatch setup forks from trunk
(ladder → main). If main hasn't been promoted from edge before dispatch starts,
the worktree won't include the latest authored content. So the pre-dispatch
ritual becomes:

 ```bash
   git fetch . edge:main   # promote edge
 → main (bring dispatch landing zone
 current)
   dispatch setup --slice N
   dispatch sync --prepare-review
   # ... phases ...
   dispatch sync --integrate --trunk refs/heads/main  # land on main
 ```

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
- **lint as you go** — `doctrine check quick|commit|gate`. Don't use raw `cargo` 
  commands unless you know why you need them instead, or you'll hurt yourself.
- **no parallel implementation** — ride existing seams; find duplication before writing.

## environment

### worktrees

run corpus-inspecting/searching verbs from the coord tree's ./target/debug/doctrine, 
not ~/.cargo/bin/doctrine.

### jail

if `/workspace` exists, you're in a nixos bubblewrap jail, defined in flake.nix,
including some additional readonly repos mounted ro at `/workspace` plus READONLY 
`~/.cargo/bin/doctrine` - if you need a rw doctrine use the build target.

If you need something else that's missing, STOP and ask the User.

Each worktree builds into its own gitignored in-tree `target/` (cargo's default —
no shared `CARGO_TARGET_DIR` redirect). So `./target/debug/doctrine` is the live
binary after `cargo build`, and no two worktrees thrash a shared cache.

- Always use READ tool *before* writing any substantial edit (e.g.
  filling a template, writing `handover.md`) to avoid expensive write
  failure. `cot`, etc do NOT count!
