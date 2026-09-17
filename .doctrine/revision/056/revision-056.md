# REV REV-056 — SPEC-027 describes the graph shell's render output mode

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-245 added `--render` (`-X`) to `doctrine graph`: the shell rasterises its DOT
through graphviz and writes a kitty-protocol image inline instead of the DOT
text. `DEC-258` queued this Revision at design time and `RV-368` `F-2` restated
it; it is the last governance item SL-245 owes, carried forward unchanged by the
`RV-369` implementation audit.

Both rows **describe the new composition**; neither relaxes the emitter
boundary. `catalog::dot::render` is untouched, and responsibility 4's
"no filesystem or external-renderer dependency" clause stays literally true —
the renderer sits entirely outside the emitter, in the `terminal_image` leaf,
and the shell passes the emitted DOT string through unread.

Both rows are `modify` and therefore surfaced for manual landing.

## Reconcile narrative (SL-245)

- [`RV-369`, carried from `DEC-258` / `RV-368` `F-2`] — **SPEC-027
  responsibility 5** describes `doctrine graph` as a thin ADR-001 command shell
  that applies the projection pipeline and writes to stdout. It gains the render
  output mode.

  **Before**

  > Serve `doctrine graph [FOCUS]` as a thin ADR-001 command shell: validate
  > `--kind` against the kind registry, resolve FOCUS to a `CatalogKey`, apply
  > the projection pipeline in fixed order, and write to stdout.

  **After**

  > Serve `doctrine graph [FOCUS]` as a thin ADR-001 command shell: validate
  > `--kind` against the kind registry, resolve FOCUS to a `CatalogKey`, apply
  > the projection pipeline in fixed order, and write to stdout — or, under
  > `--render`, hand the emitted DOT to the terminal-image renderer and write
  > its image bytes in that text's place.

- [`RV-369`, same provenance] — **REQ-396**'s third acceptance criterion pins
  `run_graph` to "nothing else", which the render branch would falsify as
  written.

  **Before**

  > `run_graph` resolves the project root, delegates to `build_graph_output`,
  > and writes the result to stdout — nothing else.

  **After**

  > `run_graph` resolves the project root, delegates to `build_graph_output`,
  > and writes the result to stdout — nothing else. Under `--render` it first
  > prepares the render request, ahead of the root lookup so a `-X` that cannot
  > be honoured is refused as such, and writes `terminal_image::render_dot`'s
  > bytes in place of the DOT text.

  REQ-396's second criterion ("the command module neither builds nor
  post-processes DOT") stays true: the shell hands the DOT string on unread.

## Not in scope

- `ISS-242` carries the note that `concept-map export --render` joins the
  ungoverned concept-map surface. The 2026-09-15 scope cut (`IMP-451`) means
  `concept-map export` was **not** wired in SL-245, so there is nothing yet for
  a concept-map spec to pick up beyond the flag's existence on `graph`.
