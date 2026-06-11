# IMP-021: Filter and sort backlog by risk facet axes (likelihood/impact/exposure)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced during SL-039 design. SL-039 derives a per-risk **exposure** signal
(`likelihood`×`impact`) as an ordering tiebreak in `backlog order`. The same
derived axes would be a useful **sanity-check filter/sort on `backlog list`**
("show risks by exposure / likelihood / impact"). Distinct from SL-039 (which only
consumes exposure as an order fallback); this is a list-surface affordance.

Note the vocabulary: `likelihood`×`impact` is **risk exposure**, an *input* to a
future priority model — not priority itself.
