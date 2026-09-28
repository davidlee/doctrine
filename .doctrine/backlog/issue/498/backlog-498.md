# ISS-498: slice show omits outbound fulfils edges under its relationships header

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine show SL-269` printed `relationships: governed_by: POL-002, ADR-007`
while `slice-269.toml` carried eight `fulfils` edges (ISS-279, ISS-277, ISS-483,
ISS-494, IMP-240, IMP-190, ISS-496, ISS-292 partial). The inbound side renders
(`show IMP-240` lists `fulfilled by: SL-269`). An agent reading `show` concludes
the edges are missing; RV-409 F-3 did exactly that.

Same class as ISS-461 (backlog show omits `related`) and ISS-363 (rfc show omits
`references`): each kind's show renders a hand-picked subset of labels. Fix
direction: render every authored outbound relation through one shared renderer.
