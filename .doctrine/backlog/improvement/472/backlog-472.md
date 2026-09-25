# IMP-472: Live-repainting design watch view

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine design watch [SL-NNN]` — a long-running sidecar that repaints the
SL-266 inquiry-map tree in place whenever the design run's revision changes.
Split from SL-266 because repaint-in-place (terminal control, change detection,
resize, exit handling) is distinct enough from the one-shot `design tree`
render to scope separately. Serves the `map_view = "sidecar"` delivery mode.
