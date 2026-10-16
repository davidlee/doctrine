# ISS-521: Repo hymn twin model/adherence/low double-emits

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed (2026-10-16)

This repo tracks `.doctrine/hymns/model/adherence/low.md`, byte-identical to the
framework body `install/hymns/model/adherence/low.md`, with **no** sidecar. A
same-slot user twin without `replaces` appends instead of replacing (the
ISS-206 shape), so the snippet emits twice:

```
$ doctrine prompt explain --role worker --model adherence/low
model/adherence/low  prov=Framework spec=([adherence:2],0) rank=1
model/adherence/low  prov=User      spec=([adherence:2],0) rank=2
$ doctrine prompt resolve --role worker --band model --model adherence/low | grep -c "Self-guidance for models with low"
2
```

Not an install leg (`adherence/low` is not in `[hymns].expose`); likely a hand
copy. Any low-adherence model run in this repo gets the guidance twice.

## Fix

Delete the repo twin (it carries no customisation). Same family as the
byte-identical starters SL-282 inq-7 handles; the identical-twin recognition
there could also report this case (identical, no sidecar → double-emit).

Found during SL-282 design.
