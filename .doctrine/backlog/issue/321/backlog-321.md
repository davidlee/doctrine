# ISS-321: Plan reader discards authored per-phase spec/requirement links

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`plan.toml` carries a top-level `[specs]` / `[requirements]` block and per-phase
`specs` / `requirements` arrays. Authors populate them. The reader drops them.

`Plan` deserializes only `phases` (`src/plan.rs:20-23`), and `PlanPhase` carries
no `specs` / `requirements` fields (`src/plan.rs:31-43`). Every authored link is
parsed past and discarded — no gate, projection, or read surface consumes it.

## The comment is stale, and it misled a research agent

`src/plan.rs:14-18` asserts the link tables *"exist in the file but are empty (no
registry yet) and are not modelled"*.

The second clause is true. **The first is false** — ten plan files populate them:

```
.doctrine/slice/020-backlog-entity-v1/plan.toml:68   requirements = ["REQ-053", "REQ-058"]
.doctrine/slice/057-formal-vt-verification/plan.toml:53-54  specs = ["SPEC-002"], requirements = ["REQ-254"]
.doctrine/slice/043/plan.toml:72-73                  specs = ["SPEC-001"], requirements = ["REQ-078", "REQ-092"]
```

Top-level `[requirements].targets` is populated in SL-043, SL-057 and SL-167.

A pi-research agent surveying the plan model for the RFC-027 obligation study
read the comment and reported the tables as scaffold-only across the whole
corpus. The claim survived to a written brief and was caught only on
verification against the corpus. A stale comment on a public struct is
load-bearing for agent research, not just for humans.

## Why it matters now

`REQ-439` AC-2 (PRD-001, `pending`, landed by REV-045 under IMP-382) requires
that every phase state *"any applicable canonical spec or requirement links"*.
The corpus already carries that data for ten slices; the model cannot read it.
So this is not a greenfield feature — it is a pending product requirement with
authored data already waiting, and a reader that throws it away.

`REQ-447` AC-1 bears on the shape of the fix: a plan should cite canonical
requirement and specification identities rather than restate their content. The
existing arrays are already citations. Modelling them is cheap conformance.

## Scope

Two separable pieces:

1. **Cheap, now** — correct the `src/plan.rs` comment so it stops asserting a
   false corpus fact.
2. **Needs a home** — model the link tables and give them a consumer. This
   belongs in the "Phase plan surface" component spec that IMP-382's `/spec-tech`
   half will author, not in an ad-hoc patch. Do not model fields without a named
   consumer (RFC-003 derivability; IMP-382's own boundary).

## Source

RFC-027 obligation-graph study, thread A/B verification —
`.doctrine/rfc/027/obligation-study/thread-ab-verification.md`.
