# Notes SL-245: Inline terminal diagram rendering

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage
<!-- runbook step explore.triage, design run dr-019fd12e -->

### Constraining governance

| authority | how it binds |
|---|---|
| **SPEC-027** (resp. 4) | The DOT emitter must carry "no filesystem or external-renderer dependency". The renderer is therefore sited outside it — settled before scoping, not open. |
| **ADR-001** | leaf ← engine ← command, downward only. `map_server = "command"` (`layering.toml:102`), so a leaf renderer **cannot** reuse the existing `DotRenderer`. This is the slice's central structural constraint. |
| **POL-002** facet (3) | New this week (REV-047). A feature-scoped host capability must be opt-in and must fail naming what was missing and what would satisfy it. Forbids a default-path `dot` dependency and forbids a silent fallback. |
| **STD-001** | The protocol is nothing but meaningful literals — `\x1b_G`, `a=T`, `f=100`, `m=0/1`, the 4096 chunk bound. Every one is a named constant, single-sourced. |
| **POL-001** | Prose in errors stays plain. The absence path is user-facing text. |

Checked and **not** applicable: ADR-013 (no governance dependency remains — REV-047
already landed); ADR-019 (nothing embedded or published); ADR-006/008/011/012/020
(no dispatch or worktree surface).

### Shaping decisions already taken

- **D-A. Renderer sited outside SPEC-027** — user, 2026-08-05, pre-scoping. DOT
  text in, terminal bytes out.
- **D-B. No crate.** `base64 = "0.22"` is already a direct dependency annotated
  "Leaf-legal external" (`Cargo.toml:137`); the encoder is ~30 lines; `kitty_image`
  is v0.1.0 and pins `base64 ^0.21`, which would put two base64 majors in the lock.
- **D-C. Sizing in cells, not pixels.** The protocol's `c=`/`r=` are columns/rows
  and `tty.rs::stdout_terminal_width` already reports cells. The preflight plan of
  chasing the ioctl's pixel fields is dropped.
- **D-D. No capability handshake.** Opt-in is the capability assertion. Env
  sniffing would be actively wrong — in-jail `TERM=xterm-256color` with no
  `KITTY_WINDOW_ID` against a terminal that speaks the protocol.

### Assumptions carried

- **A-1.** The user's terminal (ghostty) implements the kitty protocol's
  transmit-and-display path for PNG. Not probed; opt-in makes it the caller's
  assertion.
- **A-2.** `dot -Tpng` output is not byte-stable across graphviz versions, so no
  golden may contain real rasteriser output. Asserted from general knowledge, not
  measured — cheap to check if a golden is ever proposed.
- **A-3.** `t=d` (direct transmission) is the protocol default; the docs' minimal
  example omits it. Flagged unverified in `research/raw/protocol.md`; verify at
  point of use.

### Open questions (the inquiry map)

Carried into the run's inquiry, not resolved here. The live forks:

1. Extract the `dot` spawn down to a leaf seam and refactor `map_server` onto it,
   or write a second sync spawn and knowingly hold two. Bears on "no parallel
   implementation"; complicated by tokio/async vs sync.
2. CLI shape — separate verb, a `--render` axis beside `--format`, or a `--kitty`
   shorthand. Note `--color` is **global** (`src/main.rs:110`), so it is not the
   straightforward precedent preflight took it for; and facet (3) forces the
   default off, unlike `--color`'s `auto`.
3. Flag set but stdout is not a terminal — hard error, or DOT with a note on
   stderr.
4. Which surface is wired first, and whether SPEC-027 wants a sentence noting a
   peer renders its output.

### Risks

- **R-1.** No raster viewer in the jail and an agent cannot see an image, so
  "the picture is right" is VH-only. Test surface must be framing/encoding with a
  stubbed rasteriser — the `FakeDotRenderer` pattern (`map_server/shell.rs:73`)
  is the in-repo model.
- **R-2.** The escape sequence reaching a pipe or file corrupts every redirected
  invocation. The guard is load-bearing, not cosmetic.
- **R-3.** Scope creep toward the web/TypeScript emitters (`web/map/src/dot.ts`),
  which a Rust leaf cannot serve. Out of reach; say so rather than drift.

### Load-bearing prior art

- `src/map_server/shell.rs:33-70` — the existing `dot` spawn: `NotFound` →
  `ToolUnavailable { tool }`, non-zero → `CommandFailed { status, stderr }`, both
  legs bounded by a 10s timeout. The discrimination facet (3) wants.
- `src/tty.rs` — thin `stdout_*` impure wrapper + pure both-injected decision fn.
- `mem.pattern.design.capability-as-data-seam` (verified) — probe in the shell,
  hand the pure core a *multi-valued descriptor*; never collapse absent /
  unsupported / degraded into a bare `Option` or `bool`.

## Inquiry state — parked 2026-08-05

Design run `dr-019fd12e-03c3-7c92-9b87-bb48b4845e13`, revision 9, stage
`exploring`. Re-enter with `doctrine design resume SL-245` — the run carries the
map, the blocking set, the discharged runbook and the change log. This section
carries only what the run does not.

**Resolved.** `inq-1` → **DEC-143**: two `dot` spawns held knowingly. Pure
protocol encoder in leaf; a small sync spawn beside it; `map_server`'s async
`DotRenderer` untouched. The `dot` program name and the tool-unavailable /
command-failed taxonomy are single-sourced under STD-001; each spawn site
carries a comment naming the other, so the duplication is discoverable from
either end (the user's addition).

**Open at the cursor — `inq-2`, the CLI shape.** Three shapes were put to the
user, who parked before answering:

- **(a) A pipe-consuming verb** — `doctrine graph SL-245 | doctrine render`.
- **(b) A `--render` flag on each emitting verb** — `doctrine graph SL-245 --render`.
- **(c) A `--format kitty` variant** — reuses the existing flag.

The agent recommended **(a)**, reversing the user's earlier lean toward a flag,
on a boundary argument: under (b) `src/commands/graph.rs` imports the renderer,
so the `graph` verb acquires the external-renderer dependency SPEC-027 resp. 4
forbids its component — siting the renderer outside SPEC-027 buys nothing if the
verb inside it reaches out and calls it. (a) additionally serves every future DOT
producer with no per-verb wiring (IMP-385's anchor report included), is opt-in by
construction so POL-002 facet (3) needs no defaulted-off flag, and answers
`inq-6` with *no SPEC-027 change needed*. Its cost is that it is a pipe, against
IDE-046's "one command, picture appears" — the rebuttal being that the card's
complaint was a three-stage pipe through two external tools. (c) is weakest: it
conflates representation with delivery, needs two separate additions because the
verbs' format enums differ in kind (`FromStr` vs `ValueEnum`), and creates
`--format mermaid --render` as a nonsense combination to guard.

**Outstanding user acts**, both gating advance to drafting and neither
performable by the agent: `governance-confirmed` (the user confirms the
governing-context claim) and `graph-reviewed` (the user reviews the six-node map
and its declared blocking set `inq-1..inq-5`). The user was asked and parked
without answering; do not infer either from engagement.

**Still blocked or unopened:** `inq-3` (not-a-terminal fork, child of `inq-2`),
`inq-4` (capability descriptor, now unblocked by DEC-143), `inq-5` (VT/VH split,
now unblocked), `inq-6` (first consumer + SPEC-027 note, non-blocking, needs
`inq-2`).

**Note on `research/`** — `.gitignore:49` excludes `.doctrine/slice/*/research/`,
so `research.md` and its `raw/` threads live on disk in this worktree only. They
are runtime tier by design; if a fresh clone needs them, re-run the round.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
