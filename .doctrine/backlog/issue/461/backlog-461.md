# ISS-461: backlog show omits related edges under its relationships header

`doctrine backlog show IMP-464` and `doctrine backlog inspect IMP-464` both print

```
relationships:
  references(concerns): PRD-019, RFC-011
```

and stop. The item also carries two `[[relation]] label = "related"` rows
(`IMP-430`, `ISS-310`), authored through `doctrine link` and present in
`backlog-464.toml`. `doctrine inspect IMP-464` renders them correctly:

```
outbound:
  references(concerns): PRD-019 — "…", RFC-011 — "…"
  related: IMP-430, ISS-310
```

So the edges are stored and one read path shows them while the kind-local read
path silently drops them, under a header that claims to be listing relationships.
A reader with no reason to reach for the generic `inspect` concludes the item has
no `related` edges.

This is the shape `STD-001`'s sibling `STD-003` ("no silent skip — a degraded
read is disclosed") exists to forbid: either render the rows or say what was
withheld and why.

Found while authoring `IMP-464` on 2026-09-19. Whether the same omission affects
other kinds' `show` renderers is unchecked — worth a sweep, since `SL-048` put
every kind on the same uniform `[[relation]]` rows.
