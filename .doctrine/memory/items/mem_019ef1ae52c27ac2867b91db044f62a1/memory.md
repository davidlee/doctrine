# Project Orientation

> **This is a seed memory created during `doctrine install`.**
> Edit it to suit your project's needs, then run `doctrine memory verify <key>`
> to attest it.

## Project Purpose

Doctrine is a CLI toolkit that governs intentional change in software repos.
It provides a slice lifecycle (scope → design → plan → execute → audit →
close), adversarial review with turn-based ledgers, a durable agent memory
corpus, and an entity engine over TOML+MD. The audience is AI coding agents
(orchestrated by a human) working in a Doctrine-instrumented repo.

## Guiding Principles

- **DRY.** Find duplication before writing new code. Ride existing seams.
- **Small, composable, single-responsibility.** Pure functions where possible;
  impurity in the thin shell.
- **TDD.** Red, green, REFACTOR. Test behaviour, not trivial implementation.
- **CLI is the source of truth.** Don't guess ids, command shapes, or flags —
  `doctrine --help` (dev: `./target/debug/doctrine --help`).
- **No code without an approved plan.** The routing gate (`/route`) chooses the
  governing skill before any inspection or implementation.
- **Read entities via `show`, not raw files.** Structured data in TOML, prose
  in MD — `show` synthesizes both. Never judge an entity from one tier.
- **Cite durable ids** (e.g. `SL-023`, `ADR-005`, `REQ-059`), never mobile
  membership labels (`FR-`/`NF-`).

## Architecture

Four pillars over a thin Rust shell (`src/`):

- **Slice lifecycle** — every change scoped, designed, planned, phased,
  audited, closed.
- **Governance** — rules of the road in `.doctrine/state/boot.md` (the boot
  snapshot, `@`-imported into agent context). Carries the routing table, core
  process, guardrails.
- **Memory** — durable, scoped knowledge agents retrieve instead of
  rediscovering. Shipped corpus in `memory/` (global orientation); local items
  in `.doctrine/memory/items/`.
- **Entity engine** — slices, ADRs, specs, reviews, backlog items, memories
  are all authored entities over one engine (TOML metadata + MD body). Storage
  is tiered: authored (committed, diffable), runtime (gitignored, disposable),
  derived (regenerable).

Module layering (ADR-001): leaf ← engine ← command, no cycles.

## Structure

- `src/` — Rust CLI + engine. Entry: `main.rs`.
- `memory/` — shipped (global-orientation) memory corpus, embedded via
  RustEmbed.
- `.agents/skills/` — agent skill definitions (Markdown).
- `install/` — templates and reference docs seeded by `doctrine install`
  (glossary, using-doctrine, governance, routing-process, seed templates).
- `.doctrine/spec/` — product and tech specs (authored).
- `.doctrine/adr/` — architectural decision records (authored).
- `.doctrine/slice/` — active slices (authored scope + design + plan + phases).
- `.doctrine/backlog/` — backlog items (improvements, issues, chores, risks,
  ideas).
- `.doctrine/state/` — runtime state (gitignored): boot snapshot, dispatch
  state, phase sheets.
- `.doctrine/memory/items/` — local (client-project) memories.
- `justfile` — task runner (`just gate`, `just check`, `just build`).

## Conventions

- **2-space indent.** Rust: `rustfmt` (standard). No tabs in Markdown.
- **Conventional commits** scoped by slice id: `fix(SL-143): …`, `doc(SL-005):
  …`. Commit on `main`.
- **Edge/main split.** Primary worktree stays on `edge`. Promote to `main` via
  `git fetch . edge:main` before dispatch. Never checkout the primary worktree
  to another branch.
- **Lint as you go.** `just gate` before every commit (clippy zero warnings).
  `just check` for fast inner loop (root package only). `cargo fmt` and
  `cargo clippy` — no `--all-targets` (it enables `unwrap_used`/`expect_used`
  denials in test code).
- **No stashing.** Multiple agents work in the same repo. Use worktrees
  (`/worktree` or `/dispatch`) for isolation.
- **Phase ids and criteria ids are immutable** — edits append, never renumber.
- **Ask, don't infer.** Correctness first and last.

## Tooling & Development Workflow

- **Build:** `cargo build` (dev binary at `./target/debug/doctrine`).
- **Test:** `cargo test` (unit + integration). `just check` for fast pre-commit
  (root package only, skips cordage workspace crate).
- **Gate:** `just gate` — runs `cargo clippy --workspace` (zero warnings
  required). Do this before every commit.
- **Format:** `cargo fmt`.
- **Environment:** NixOS. Development happens inside a bubblewrap jail mounted
  at `/workspace`. Each worktree builds into its own gitignored in-tree
  `target/` (cargo's default — no shared `CARGO_TARGET_DIR` redirect; SL-156,
  ADR-008 D-B1).
- **Branch strategy:** `edge` (primary) → `main` (landing zone). Worktrees for
  parallel work. Promote edge to main before dispatch: `git fetch . edge:main`.
- **Boot:** run `doctrine boot` after governance edits to regenerate the boot
  snapshot. Check with `doctrine boot --check`.
- **Memory sync:** after editing shipped memories (`memory/`), run `cargo
  build` (to re-embed via RustEmbed), then `doctrine memory sync` to
  materialize, then `doctrine claude install` to refresh installed skills.

## Further Reading

- `.doctrine/spec/product/004/` — PRD: core product specification
- `.doctrine/spec/tech/007/` — Tech spec: memory system
- `.doctrine/spec/product/008/` — PRD: dispatch system
- `install/using-doctrine.md` — which verb for which intent, storage tiers,
  reading via `show`
- `install/glossary.md` — entity kinds, ids, reference forms, verification
  taxonomy
- `install/routing-process.md` — the routing table and core process (also
  inlined in boot snapshot)
- `install/governance.md` — project-local governance pointers
- `install/review-ledger.md` — adversarial review protocol
- `.agents/skills/` — agent skill definitions (route, execute, audit, etc.)
- `README.md` — project README
