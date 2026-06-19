---
seq: 0005
scope: spec
target: SPEC-018 (Cross-corpus relation contract)
confidence: high
reversible: yes (proposal only; no spec/requirement authored — fence holds)
---
## What
SPEC-018, "Cross-corpus relation contract" — the authored-edge substrate beneath
the *entire* graph topology — is the **only tech spec with zero requirement
members**. Every peer carries 5–13 (`SPEC-001`=8, `SPEC-007`=13, even the thin
`SPEC-003`=4); SPEC-018 = 0. Verified at the source, not just the count:
`.doctrine/spec/tech/018/members.toml` holds only its seed comment ("Seeded empty;
`spec req add` … appends rows"), and the `## Requirements` section of
`spec-018.md` is empty.

This is not a stub — the prose is rich (10KB body + an 8KB `relation-vocabulary.md`
companion, 5 sharp responsibilities). That is exactly what makes the gap notable:
the contract is *described* in full but has **no verifiable requirement spine**.
Consequences:
- nothing downstream can cite a `REQ-NNN` for relation-contract behaviour — slices
  touching relations (SL-046/047/097, IMP-053 record↔record, IMP-095 supersede
  migration) have no requirement to trace to;
- reconciliation & coverage (PRD-013 / SPEC-002 / SPEC-013 coverage) have no
  requirement anchor for the relation layer — the contract is unverifiable by the
  machinery doctrine ships for exactly this;
- the contract's invariants live only as prose `responsibilities:`, so a regression
  (e.g. an inbound edge leaking to storage, a read-strict validator) violates no
  recorded requirement.

The responsibilities already read as requirement seeds — five crisp, testable
invariants (the relation model & outbound-only storage; the tier partition;
vocabulary single-source-of-truth; write-strict/read-tolerant validation; the
uniform `link`/`unlink` write seam). The extraction is mostly transcription of
intent that already exists.

## Options
1. **Author the requirement spine now** — `spec req add` the 5 responsibilities as
   FR/NF requirements, weaving SPEC-018's members.toml. Tradeoff: closes the
   verifiability gap and gives relation slices a trace target; cost is one focused
   authoring pass + deciding FR vs NF for each invariant.
2. **Author only the NF invariants** (outbound-only, write-strict/read-tolerant —
   the safety properties), defer the structural FRs. Tradeoff: captures the
   highest-risk guarantees cheaply; leaves the spec partially-membered (still an
   anomaly, just smaller).
3. **Leave prose-only; rely on `RELATION_RULES` as the contract.** Tradeoff: zero
   cost, and the code table *is* authoritative for the vocabulary — but the spec's
   own job is the requirement basis for reconciliation, and "the code is the spec"
   is precisely the coupling SPEC-018 was written to avoid.

## Recommendation
Option 1: author the full spine. SPEC-018 is the foundation the graph-topology
story (proposals 0003/0004, PRD-011) rests on; a foundation with no requirements
is invisible to reconciliation and coverage, the two systems that would otherwise
keep it honest. The responsibilities are already decomposed — this is low-novelty,
high-leverage. If time-boxed, do Option 2 first (the two safety invariants) and
backfill the FRs, but the end state is the full spine.

Decisions deferred to YOU:
- (a) **FR vs NF split** per invariant — e.g. is "outbound-only, inbound derived"
  a functional requirement or a quality/architecture constraint? (outbound-only +
  write-strict/read-tolerant read as NF; model/tiers/vocab-source/link-seam as FR.)
- (b) **granularity** — one REQ per responsibility (5), or split the model
  responsibility into model-vs-storage (6–7)?
- (c) **timing** — now, or after IMP-053 (record↔record relation class) lands, since
  it will add labels and may add a requirement; doing it now risks one revision
  later, doing it after risks the gap persisting through more relation work.

## Next doctrine move
```
# inspect both tiers + the responsibilities to harvest as requirements (read-only):
doctrine spec show SPEC-018
cat .doctrine/spec/tech/018/spec-018.md          # responsibilities = req seeds

# author the spine (the actual move — NOT executed; fence forbids authored edits):
/route                          # → /spec-tech (revise SPEC-018) which drives:
#   doctrine spec req add SPEC-018 --requirement REQ-NNN --label NF-001 ...
#   (exact verb shape per `doctrine spec --help`; new REQ minting via requirement verb)
```
(Verbs described, NOT executed.)

## Illustration (optional) — ILLUSTRATIVE, not applied
Candidate requirement seeds, hand-derived from SPEC-018's `responsibilities:`
(labels indicative only — the FR/NF call is deferred per decision (a)):
```
NF-? relations are stored outbound-only; inbound is ALWAYS derived from in_edges,
     never persisted (sole carve-out: typed superseded_by). [ADR-004]
NF-? validation is write-strict (link refuses dangling/illegal-kind numbered edges)
     and read-tolerant (reader/validate report, never rewrite).
FR-? the legal vocabulary is the single code-authoritative RELATION_RULES table
     keyed by (source∈sources, label); writer/validator/reader/overlay all read it.
FR-? edges partition into tier-1 uniform [[relation]], tier-2 constrained/payload,
     tier-3 free-text-typed; typed guarantees preserved, not flattened. [ADR-010]
FR-? the cross-kind write seam is the uniform link/unlink verb over append_edge/
     remove_edge, gated by the table's LinkPolicy (Writable/LifecycleOnly/TypedVerbOnly).
```
