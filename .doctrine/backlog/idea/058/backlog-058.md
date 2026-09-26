# IDE-058: Semi-global -G flag: pipe markdown output through glow --pager

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

A recurring manual habit: `doctrine show ID | glow --pager`. A semi-global `-G`
flag would fold that into one invocation (`doctrine show ID -G`), rendering an
emitted markdown body through the user's own markdown pager.

Origin: user proposal, this session.

## What already exists

- **`doctrine graph <FOCUS> -X/--render`** — the nearest precedent, shipped by
  SL-226 and refined by SL-245 PHASE-02/03. It pipes graphviz `dot -Tpng` into
  the kitty graphics protocol behind a support probe, and refuses cleanly off a
  tty (`--render needs stdout to be a terminal; drop -X to emit DOT`).
- **The clap global-arg seam** — `--color` is a root-level global
  (`src/main.rs`, `#[arg(global = true)]`), and the custom help renderer already
  walks `is_global_set()` to re-attach ancestor globals into cloned subcommand
  help (`src/commands/cli.rs`). A `-G` global rides this machinery; no parallel
  invention is needed.
- **The subprocess shell precedent** — `src/graphviz.rs`: pure `classify` plus a
  thin impure `rasterise_png`, with a `ToolUnavailable` outcome and no deadline
  (the bounded render is deferred to IMP-452).

## Design questions (unresolved — this is why it is an idea, not scoped)

1. **The seam.** ~91 `stdout()`/`print!` call sites across `src/commands/*`
   (59 of them the document shape `let out = <render>; write!(io::stdout(),
   "{out}")?`). A `-G` needs ONE interception point on the markdown-emitting
   show surface, not 91 call-site edits. See *Feasibility finding* below — this
   one looks answered, and the answer is not a render-seam helper.
2. **Fallback when `glow` is absent** (it is not installed in the dev jail).
   Pass through the raw markdown, or refuse with a named-tool message, mirroring
   `RasterOutcome::ToolUnavailable`?
3. **Which commands.** "Semi-global" — every markdown-emitting verb, or an
   explicit allowlist? Table output (the `list` surfaces) is not markdown.
4. **Interaction with `--color`.** `glow` does its own rendering, so doctrine's
   colour decision is irrelevant for that command's output: `-G` must hand over
   plain markdown.
5. **Paging semantics off a tty.** A piped or redirected `-G` should presumably
   be a no-op passthrough, consistent with `--color`'s tty-gated auto.

## Feasibility finding (design probe, this session)

`main()` is already a single funnel — `Cli::try_parse()` → `resolve_color` →
`worker_guard` → `dispatch` (`src/main.rs`). Everything downstream reaches for
`io::stdout()` on its own. So the interception belongs **above** dispatch, where
zero call sites have to change:

**Preferred — re-exec self with stdout on the pager's stdin.** In `main()`, before
dispatch: if `-G` is set and stdout is a terminal and `glow` is on PATH, strip
`-G` from argv and re-exec `current_exe()` with its stdout wired to `glow --pager`'s
stdin. The child runs the command completely unmodified — every one of the 59
`write!` sites lands in glow's pipe without knowing it exists. Streaming and
Ctrl-C are preserved, and there is no fd surgery. Precedent: `boot.rs` resolves
`current_exe()`, and `src/worktree/claim_lock.rs` already spawns a child *from*
`current_exe()`. Costs one extra process, on `-G` invocations only. The
`glow`-absent fallback is decided in the parent, before the re-exec.

**Rejected — fd capture.** `dup2` stdout onto a pipe/temp file, run dispatch,
restore, feed the buffer to glow. Also zero edits, but it buffers: no output until
the command finishes, and the pager gets a lump rather than a stream. Dominated by
re-exec.

**The predicate is the real question, not the interception.** "Semi-global" is the
right word, and clap can carry it: declare the flag in a `#[derive(Args)]` bundle
flattened into the markdown-emitting verbs — a sibling of the existing
`CommonListArgs` (`src/main.rs`), the repo's own pattern for "the mandatory spine
of a read surface, which a kind cannot quietly grow bespoke flags around". Then a
plain argv scan for `-G` at the root is *exact*, because clap has already rejected
the flag on any verb that does not declare it: the enumeration lives in the type
system rather than in a match arm someone forgets to extend. The alternative — a
root global riding `--color` plus a pure `pages_markdown(&Command)` predicate —
keeps the existing global-arg seam but re-introduces a hand-maintained list that
can drift. `-G` is free as a short flag (`-g` is `install --global` only).

**Named but deferred — sink-as-data.** Injecting an output target the way colour
and width already are (`src/tty.rs`: capabilities read in the shell, injected as
plain values, render layer never touching ambient tty) is the architecturally
correct end state, and would remove ambient `io::stdout()` from the whole command
layer. It buys this feature nothing that re-exec does not get for free, so it is
the refactor to do when a *second* output policy appears.

## Relationship to prior art

- The graphviz `-X` render is the shape to mirror: capability probe → guarded
  spawn → graceful, named refusal.
- IMP-452 (bounded subprocess helper plus render timeout) is the natural shared
  home for the spawn — `-X` today has no deadline. Riding that helper avoids a
  second bespoke spawn site.
- IMP-133 (CLI usability shortfall UX review) is the parent UX sweep this
  convenience flag serves.
- IDE-016 (agent UX research: which read surfaces benefit) holds the evidence
  question underneath it — is a human-facing pager worth the seam?

## Out of scope

- Choosing or bundling a pager. `glow` is the user's tool; doctrine only spawns
  a named program when it is present.
