# DEC-307: Inquiry tree wraps, never truncates

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->


## Amendment — SL-266 PHASE-02 VH-1 (2026-09-26)

Decided by the user at the PHASE-02 human check (VH-1), after reading the
width-40 and width-16 goldens: "agree to a) - that's a pathological case, and
making it legible *at all* requires compromises."

Design sec-3 rule 2 as locked said dropped text starts at a fixed 8-column
indent with rails not drawn. At narrow widths that detached the text from its
node. Amended rule:

- Below `TREE_MIN_TEXT_COLS` (24) free columns beside the prefix, node text
  drops to the next line **one level in, under the node's own rails**, with a
  child rail when the node has children.
- Only when fewer than `TREE_MIN_DROP_COLS` (16) columns would remain beside
  those rails does the text fall back to the bare `TREE_DROP_INDENT` (8), rails
  not drawn — the deep-and-narrow case, where the rails alone fill the line.

Also from VH-1: the legend paints its marks and provenance letters as the
nodes do.


Also from VH-1 (user: "let's give the cursor and pinned signifiers a colour
too. cursor = same as the open sigil; pinned = .. magenta"): the cursor suffix
takes the open mark's colour (bold); `pinned` is magenta. Legibility verdict:
"legible".
