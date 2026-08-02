# DEC-115: No diagram rendering in this slice; DOT over d2 when one is taken

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

`doctrine spec anchors` renders `json` and `markdown`. No diagram format ships
in SL-243. When one is taken it is DOT, not d2.

This is a **scope change**: `O1` said "a neutral JSON core plus markdown and d2
renderings". The slice scope is edited to match, so the delivered surface and
the scope agree at audit rather than the gap reading as an omission.

## Why the diagram waits

The slice's own Follow-Ups defer the focus-scoped altitude view because the
whole-corpus graph "does not fit on a page". A diagram of the whole corpus is
48 specs against 84 dark files — a hairball in any renderer. Shipping the
unreadable rendering first is work that produces nothing anyone reads, and the
readable form is precisely what is already deferred. So the diagram is the same
piece of work as the focus-scoped view, or arrives just after it: **IMP-385**.

Deferring costs almost nothing later. The report core is presentation-neutral
(DEC-112, and the SPEC-027 shape the scope commits to mirroring), so a renderer
is a format-enum variant plus an emitter, with no rework of the core.

## Why DOT rather than d2, when it is taken

No d2 emitter and no mermaid emitter exists for the graph projection; the only
mermaid emitter in the tree is in `concept_map.rs`, over a different type. DOT
exists in `catalog/dot.rs`, is already deterministic — sorted on (source,
display label, target) with the original list index as final tiebreak — and
declares itself the single source for `dot_escape` going forward.

"Do not fork `dot_escape` a third time" turns out not to be an argument for d2:
d2's escaping rules are its own, so it would share nothing either way.
AGENTS.md's stated diagram preference is mermaid, so d2 is not a house default
either — it was in `O1` without a reason behind it.

The reason that does favour DOT is downstream. DOT has a mature rasteriser, so
doctrine could render and write the image straight to the terminal via the
kitty graphics protocol rather than leaving the user to run
`doctrine graph … | dot -T png | viu -` by hand. That is **IDE-046**, and it is
what makes DOT the more valuable of the two emitters rather than merely the one
that already exists.

## What this corrects

DEC-112's body states the format enum covers `json | markdown | dot`. That
prejudged this fork and is amended: the enum ships with `json | markdown`, and
gains its diagram variant with IMP-385.

## Provenance

Settled at the `inq-9` fork of design run `dr-019fc13a` (SL-243). The absence of
a d2 or mermaid emitter for the graph projection was verified in the slice's
research round.

## Related

- [[DEC-112]] — the JSON shape and the format enum this narrows.
