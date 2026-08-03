# EVD-007: Every scored refusal token is computed trusted-side, downstream of the ingestion mechanism

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Datum

Across the SL-241 C3 matrix — sixteen hazard rows run against both mechanisms
and both fixtures — **every refusal token in the scored set is produced by the
mechanism-independent downstream pipeline**, not by the mechanism:

| stage | tokens observed | which pipeline leg |
|---|---|---|
| `conform` | `ancestry-not-descendant`, `ancestry-merge-commit`, `undeclared-path`, `forbidden-path`, `gitlink`, `gitmodules` | ancestry validation; actual-path computation; selector conformance |
| `harvest` | `resource-cap` | resource bound |
| `verify` | (no token — suite outcome) | verification against the exact tree |
| `advance` | `stale-base` | normalisation / CAS |

Each token's basis is visibly trusted-side computed, from object identity rather
than worker assertion — the scored `planted` column carries the computation, not
a claim about it: `S=cb5611aa… is NOT a descendant of B=7f43b875…`,
`merge commit(s) in B..S: c9392831…`, `docs/h4-undeclared.md`.

**The consequence for QUE-200.** The mechanism choice moves almost no
trust-bearing line. Everything the boundary actually decides happens *after* the
objects are in quarantine, in the pipeline QUE-200 already describes as
mechanism-independent. What the mechanism choice moves is only what happens
*before* that — how the objects get there — and the two rows that exercise that
half are H6/H7 (EVD-006, mechanism-neutral) and H13 (EVD-010, M-B only).

## The one mechanism-specific trust-bearing line is defective — state it, do not smooth it

`harvest/fsck-failed` is M-B's, and it is **emitted at two sites** in
`harvest-bundle.sh` — the `git fetch` failing (`:100`) and the final
whole-quarantine fsck (`:106`) — with git's stderr **discarded at both**. One
token therefore stands for two causes that are not alike:

- *the ingested objects are bad* — security-relevant; and
- *this quarantine's derived cache is stale* — operational.

I5 says refusals report trusted-side-computed tokens. It does not say one token
may stand for two causes, and the ambiguity cost a session to diagnose
(F-P05-28).

Compounding it: **the harvester fscks the whole quarantine, not the range it just
ingested**, so pre-existing or derived-cache damage from any source lands as a
refusal attributed to the capsule.

Both are recorded as operator findings, unfixed — SL-241 PHASE-05 holds `src/`
untouched (S4).

## What it bears on

Supports the reading that QUE-200's mechanism decision is *narrow*: it decides
the transfer, not the admission boundary. It also locates the one place the
mechanism decision does carry trust weight, and shows M-B's line there needs
splitting before it ships.

## Related

- [[safe-capsule-ingestion-mechanism]] — QUE-200, the question this informs.
- EVD-006, EVD-010 — the two rows that exercise the pre-quarantine half.
- SL-241 PHASE-05 T4a–T4e; `~/capsules/probes/c3/results.tsv`.
- Finding F-P05-28 (the two-site `fsck-failed` token; whole-quarantine fsck).
