# ISS-299: Inquiry map never reaches the user

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

SL-233 §2 commits:

> Make the active path, nearby frontier, blockers, resolved/open counts, and
> material map changes **visible to the user** without injecting the full map on
> every turn.

The **render** discharges this. `src/design_run/render/envelope.rs` carries
`frontier()` — ranked by kinship and posture, capped at
`ENVELOPE_FRONTIER_NODES` with an `omitted` count — plus `pinned()`, blockers,
and the totals line. The bounded surface exists and is roughly the right shape.

**Nothing obliges an agent to surface it.** Across every shipped design-prompt
asset there are zero references to `frontier`, `map`, `design show`, or a
decision tree:

| asset | hits |
|---|---|
| `exploring.toml` | 0 |
| `inquiring.toml` | 0 |
| `drafting.toml` / `drafting.md` | 0 / 0 |
| `reviewing.toml` / `reviewing.md` | 0 / 0 |
| `inquiry.md` — *delivered every turn of exploring and inquiring* | **0** |
| `delegation.md` | 1, and it concerns export refusal, not display |

So no runbook step requires it and the every-turn craft fragment never mentions
it. Whether the user sees the map depends entirely on the agent spontaneously
calling `doctrine design show` and relaying the output.

## Observed consequence

In the CHR-049 subject run `dr-019fc13a`, the map has **never been rendered with
content**:

- three `design show` calls, all between 06:47:23Z and 06:47:29Z, all at
  `nodes = 0`;
- nine nodes created at 07:04Z (3 `user-directed`, 6 `agent-proposed`);
- no `show` call since.

The user moderating the run reported the map as missing and could not tell
whether it had been built.

## Why this compounds with ISS-298

The only path that renders the map is the one the subject abandoned after four
calls. Its single attempt to widen that view — `design show --full` — returned
byte-identical output to the plain form (ISS-298). A read path that teaches an
agent it is not worth calling will not deliver a surface that only renders there.

## What this is not

Not a claim that a full decision tree per turn was promised — SL-233 explicitly
excluded that, and the exclusion looks right. The defect is narrower: a committed
user-facing surface with no delivery mechanism, so its visibility is left to
agent initiative that nothing prompts.

## Provenance

Found while moderating CHR-049. Not a defect in the exercise instrument.
