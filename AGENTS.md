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
            forgettable's forget.remote.v1/forget.checkout.v1 reproduced byte-for-byte;
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

- **review/audit ledger — SHIPPED (SL-040).** The RV kind (`RV-NNN`, ADR-007) is the
  structured audit substrate `audit.md` lacked: `doctrine review new --facet
  reconciliation --target SL-NNN`, then append-only `raise`/`dispose`/`verify`/
  `contest`/`withdraw` under a turn-based baton + per-review lock (D-C4a), with a
  warm-cache `prime`. `/audit` is rewired onto it — `audit.md` retired for new audits
  (existing files stay valid; no migration). Remaining review skills (`/inquisition`,
  `/code-review`, reconciliation) not yet rewired — IMP-023.
- **slice status rollup — SHIPPED (SL-009).** `slice list` now derives `X/Y complete`
  per slice from the phase state tree (`!N` blocked, `?N` anomalous, `—` untracked)
  and flags `⚠` when the hand-edited status and the rollup diverge. Read-only — it
  *reveals* divergence; the lifecycle-transition verb below reconciles it.
- **slice lifecycle transition — SHIPPED (SL-040 / ADR-009).** `doctrine slice status
  <id> <state>` classifies and writes the move (advance / back-edge / skip / abandon)
  across `proposed→design→plan→ready→started→audit→reconcile→done`; refuses the closure
  seam out of order (`→reconcile` only from `audit`, `→done` only from `reconcile`) and
  refuses leaving a terminal status (`done`/`abandoned`). The closure seam enforces the
  D-C9b close-gate — refuses `→reconcile`/`→done` while an RV targeting the slice carries
  an unresolved `blocker`. Resolves the SL-009 rollup divergence. (`<id>` is the bare
  number, e.g. `40`, not `SL-040`.)
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

- Always use READ tool *before* writing any substantial edit (e.g.
  filling a template, writing `handover.md`) to avoid expensive write
  failure. `cot`, etc do NOT count!
- default reviewer: codex mcp - use default (GPT-5.5) for external
  adversarial reviews. Opus sub-agent is also useful for variety on
  subsequent passes.
