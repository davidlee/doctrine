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

1. **The seam.** ~91 `stdout()`/`print!` call sites across `src/commands/*`. A
   `-G` needs ONE interception point on the markdown-emitting show surface, not
   91 call-site edits. Candidates: a shared markdown-emit helper the `show`
   route delegates through, or interception at the `main.rs` dispatch boundary.
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
