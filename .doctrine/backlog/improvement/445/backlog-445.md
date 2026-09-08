# IMP-445: Name the accepted tokens when a change event is refused

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What changed

`SL-256` `PHASE-03` replaced `ChangeEvent`'s `#[serde(rename_all)]` with
`#[serde(try_from = "String", into = "String")]` so the wire token has one
source (`STD-001`, `DEC-239`). Strictness was preserved exactly — an unknown
token is still a hard parse failure — but the **message** got shorter.

Before, serde's derive printed the whole accepted vocabulary:

```
unknown variant `integrated_review_recorded`, expected one of `node_created`,
`node_lifecycle`, …, `step_discharged`
```

After, `Refusal::UnknownChangeEvent` prints only the offending token:

```
unknown change event: `integrated_review_recorded`
```

## Why it matters

The accepted list is the useful half for the one reader who ever sees this
error: someone hitting `ISS-315`, whose snapshot carries a token retired out
from under it. The list is how they tell *retired* from *misspelt*, and how they
find the member that replaced it.

Reproduced live while verifying `SL-256` `PHASE-03`: `doctrine design show
SL-244` fails on `integrated_review_recorded`, and the pre-slice binary's
message named the candidates while the new one does not. That run is
`ISS-315`'s own reproduction case and is still unreadable.

Disclosure itself is intact — the read fails loudly and names the token, so
`STD-003` is satisfied. This is about remedy, not about silence.

## Shape

`Refusal::UnknownChangeEvent`'s `Display` arm in `src/design_run/refusal.rs` can
render `ChangeEvent::READABLE`, which is already the resolution set `TryFrom`
walks — so there is no second list to keep in step (`STD-001`).

Weigh it against the byte budget: the refusal crosses `Display` into
`anyhow!("{refused}")`, and 23 tokens is a long line. Naming the roster rather
than expanding it, or eliding to near-matches, may be the better trade.
