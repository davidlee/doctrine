# Jev relevance trial

## Context

Originates from IDE-061, whose body records the design-conversation decisions
this scope rests on (`doctrine show IDE-061`).

TypeSafe's Jev is a hosted "System One" classifier: it returns typed, calibrated
judgments (Choice / Score / Noul) over caller-defined answer spaces, and never
generates prose. It is cheap (documented $0.042 per million input tokens, output
free), so it may improve useful-source coverage for the `/research` round
without spending a reasoning agent's context on every comparison.

We expect to adopt Jev in some form, but not necessarily as a BM25 reranker.
This slice builds the evidence to decide where, and records it; it does not
integrate Jev into any workflow.

## Scope & Objectives

Build a replayable relevance evaluator, as a Rust workspace crate outside
`default-members`, with one code path and two drivers:

- **rank** — for one slice, produce a ranked research packet with per-item
  provenance (source entity, section, snapshot commit, score, arm).
- **eval** — replay many completed slices and report recall at a fixed token
  budget, per arm.

Three arms, each over the corpus as of the slice's pre-research commit
(leakage guard):

| Arm | Candidates | Ranking |
|---|---|---|
| A | `doctrine search` BM25 top-N | BM25 (baseline) |
| B | same pool | Jev per-section score |
| C | whole snapshot | Jev per-section score (recall ceiling) |

Components, kept small and reusable, since later trials such as repair
propagation should inherit them:

- **Jev client**: thin `reqwest` + `serde` over `POST /v1/systemone`. It owns the
  error typing: rate-limit, infrastructure failure, and semantic abstention are
  distinct, and none of them becomes a zero score. Raw probabilities are
  retained. `typesafe-sdk-rust` is a wire-shape reference only.
- **Sectioner**: heading-bounded sections of entity bodies. An entity's rank is
  its best section's score.
- **Snapshot builder**: the corpus at a given commit (`git archive`), queried
  with `doctrine search -p`.
- **Labels**: silver labels, meaning the entity ids cited in each slice's
  `research.md` and `design.md`.
- **Cache/replay**: keyed on the full semantic input (model id, ordered state,
  questions, rubric, preprocessing version). Raw output is stored apart from
  threshold decisions, and a failure is never cached as a negative.
- **Budget**: dry-run by default (planned requests, token and dollar estimate).
  A live run takes an explicit spend cap and aborts before exceeding it, counting
  retries. It backs off on rate-limit responses, since the user is on the free
  tier.
- **Egress**: an explicit corpus path allow-list (default `.doctrine/` entity
  bodies), plus a local log of what each run sent. The credential is read from
  `JEV_API_KEY` and never written to records, cache, or debug output.
- **Report**: per-arm candidate recall, and recall at a fixed token budget, with
  sample counts. Jev-surfaced entities that no one cited are listed for human
  judgment rather than scored wrong.

Run sequence: a live probe of about 10 calls, then one slice across all three
arms, then about 25 slices. The user spot-checks the silver labels of about 5
slices before any rubric tuning. The verdict is recorded as an EVD record.

## Non-Goals

- Any change to the shipped `doctrine` binary, the `/research` skill, CLI/MCP
  surfaces, or confined-worker network access. Adoption is a later slice.
- Trials 2–4 of the brief (repair propagation, citation support, timely
  guidance).
- A graph-neighbour candidate arm. Add it only if arms A–C leave the question
  open.
- Human adjudication of every example.
- A universal confidence threshold.

## Summary

Affected surface:

- New crate `crates/<name>/` (name settled in design).
- `Cargo.toml` workspace `members`: added, but not `default-members`.
- `justfile`: a dedicated recipe. New members are not auto-gated (see memory
  "just gate runs a named-package test gate").
- Trial outputs: runtime state is gitignored; the EVD verdict is authored.

Risks and assumptions:

- **Silver labels measure what research found and used, not what was
  relevant.** That biases arm C's precision downwards, which is why uncited hits
  are listed for review.
- **Pre-research commit detection.** Mapping each slice to a clean corpus
  snapshot may be ambiguous, since research and scope can land in one commit.
  Design must pick a rule.
- **Free-tier rate limits are unknown.** Arm C's volume (about 8M tokens per
  query) may be slow under them.
- **Jev's documented weaknesses** include irrelevant context and sensitivity to
  question phrasing. Rubric wording matters, and the live probe checks
  repeatability on a small sample.
- **Workspace lint groups.** A new workspace member trips the workspace's
  pedantic and doc lint groups (see memory "New workspace member trips the cargo
  lint group").

Open questions for design:

- The crate name and the CLI shape of rank and eval.
- Rubric wording: separate applicability and contribution signals (Noul
  versus Score).
- The token budget used for recall-at-budget.
- Section size limits against Jev's 32k state-plus-question limit.
- Where cache and run logs live under `.doctrine/state/`, and whether they fall
  within POL-002's reach at all, given the crate does not ship.

Verification and closure: offline tests cover the request/result contract,
missing answers, the failure-not-zero rule, cache-key invalidation,
budget/abort, and label extraction, with no API spend. The live probe
establishes connectivity and response semantics. The slice closes when a
developer can reproduce the eval from a clean checkout plus `JEV_API_KEY`,
inspect why each result surfaced, and an EVD record states the verdict with
sample counts.

## Follow-Ups

- An adoption slice, whose shape depends on the verdict (B wins: reranker; C
  wins: retriever).
- Trial 2 (repair propagation), reusing the client, sectioner, and provenance.
