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

## layout

```
.doctrine/slice/nnn/
  slice-nnn.toml  metadata, relations, lifecycle status
  slice-nnn.md    scope document
  design.md       canonical technical design
  plan.toml       authored phase plan (objectives, EN/EX/VT criteria, links)
  plan.md         plan prose — rationale & sequencing (no queried data: storage rule)
  notes.md        durable implementation notes (on-demand)
  handover.md     disposable agent context — GITIGNORED
  audit.md        verification / code-review / drift findings (hand-made; no scaffold yet)
  phases ->       symlink into the runtime state tree (GITIGNORED)

.doctrine/adr/nnn/   adr-nnn.{toml,md} + nnn-slug symlink — project-global ADRs (authored)
.doctrine/state/slice/nnn/phases/   runtime phase tracking (phase-NN.{toml,md}) — GITIGNORED
.doctrine/governance.md   user-owned governance pointer layer — projected into the boot snapshot
doc/*       evergreen, authoritative specs. not yet structurally supported by doctrine
.doctrine/memory/items/nnn/  memory store — memory.toml + memory.md per item;
            mem.<key> symlink alias. Query via `doctrine memory find|retrieve`.
install/    sources copied to .doctrine by the installer; plugins/skills handled special
src/        rust code (incl. src/git.rs — the impure born-frame capture seam,
            the external decision register's forget.remote.v1/forget.checkout.v1 reproduced byte-for-byte;
            src/boot.rs — the cache-friendly governance snapshot, SL-011)
```

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
- **lint as you go** — `cargo clippy` zero warnings; `just check` before every commit.
  (The gate runs plain `cargo clippy` — bins/lib only; do NOT use `--all-targets`,
  which lights up `unwrap_used`/`expect_used` denials in test code.)
- **no parallel implementation** — ride existing seams; find duplication before writing.

## known CLI gaps (todo as the tooling surface expands)

- **no `slice audit` scaffold** — every other artifact has one; `audit.md` is hand-made.
- **slice status rollup — SHIPPED (SL-009).** `slice list` now derives `X/Y complete`
  per slice from the phase state tree (`!N` blocked, `?N` anomalous, `—` untracked)
  and flags `⚠` when the hand-edited status and the rollup diverge. Read-only — it
  *reveals* divergence; reconciling it is the lifecycle-transition gap below.
- **no slice lifecycle transition** — `slice-nnn.toml` `status` is hand-edited; no
  command moves a slice proposed→…→done or links it to phase state. (SL-009 surfaces
  the divergence this would resolve; the terminal-status set lives in
  `slice::is_terminal_status` for that verb to reuse.)
- **no standalone plan validation** — a malformed `plan.toml` only surfaces when
  `slice phases` parses it.
- **memory retrieval — SHIPPED.** `record/show/list` (SL-005); SL-007 producer
  (`record` scope+git-anchor capture, `verify`, `src/git.rs` seam); SL-008 reader
  (scope-aware `find`/`retrieve`, 9-key ranking, git staleness, the `retrieve`
  trust holdback). No retrieval gap remains. Producer follow-up still open: `record`
  has no `--trust`/`--severity` flag (audit A-3) — the risk axes are TOML-only.
- **boot snapshot — SHIPPED (SL-011).** `doctrine boot` regenerates the cache-friendly
  governance snapshot; `boot install` wires the `@`-import + SessionStart hook;
  `boot --check` is the disk sentry (stale / unpopulated). In-session edits lag ≤2
  sessions — `/canon` carries the regenerate-THEN-`/clear` freshen-now ritual.

## environment

nixos; bubblewrap jails (mounted into /workspace/*).

Always READ before WRITE of any substantial edit (e.g. filling a
template) to avoid expensive write failure.

default reviewer: codex mcp - use default (GPT-5.5) for external adversarial reviews. Opus sub-agent is also useful for variety on subsequent passes.
