# SL-094: stale-closure pattern with CSS transform wrappers

When wiring event handlers that read/write CSS transforms on a DOM wrapper element that gets recreated on re-render, avoid capturing viewport state in handler closures. Read the current transform from the element's style.transform on each event via parseTransform() instead.
