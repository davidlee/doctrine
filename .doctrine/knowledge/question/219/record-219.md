# QUE-219: Submission strictness against snapshot forward-compatibility

## The question

A design-run submission that carries a key the schema does not hold is **absorbed
in silence**: admission passes, the revision bumps, a receipt is written, no
change row prints, and the command exits 0. The caller is told
`revision N stage <stage>` — indistinguishable from a successful mutation.

Should the engine refuse it, and how does refusing reconcile with stored
declarations outliving the binary that wrote them?

## The four surfaces

One class, four places, each with a different mechanism:

| item | surface | why it is not a one-line derive |
|---|---|---|
| `ISS-333` | `ApplyRequest`, the outermost type a caller hand-authors | `#[serde(flatten)] envelope` — serde cannot reconcile `flatten` with `deny_unknown_fields`; the flattened keys would themselves be refused |
| `ISS-328` | nested `CreateRecord` | the attribute *is* available here; the reason it was not applied is this question |
| `ISS-327` | keys inert at the **subject's state** — `provenance` on an existing node, `lifecycle` on a new one, `blocking` on a held finding | not a schema problem at all; four arms in `run.rs` read a key only on one branch |
| `ISS-290` | `ChangeEvent::payload_terms()` declares `(PayloadKey, ValueKind)` and **nothing checks the construction against the declaration** | a declared contract with no enforcement; already produced one unclearable runbook edge, fixed at the instance in `3516a8b3`, class still unguarded |

`SL-249` closed the sibling axis (`ISS-318`: `Declaration::WIRE_KEYS` crossed with
`IdKind::declarable` is now total, and a key inert at its subject's *kind* is
refused by name at `Batch::validate`) and scoped this residue out by name under
`DEC-183` — *"same class, different mechanism, separate change."*

## Why it matters — the tradeoff that blocks all four

`ISS-328` states the constraint, and it binds every one of them:

> Stored proposal declarations ride the run snapshot and so outlive the binary
> that wrote them (`mem.fact.design-run.snapshot-outlives-the-binary`). A
> snapshot holding a `Declaration` written by an older binary is re-read by a
> newer one, so tightening a nested type converts previously-readable stored
> state into a parse failure **at exactly the moment someone is trying to
> resume**.

`ISS-315` is that failure already live and for a different reason: `ChangeEvent`
is a closed enum deserialised strictly, so one retired variant
(`integrated_review_recorded`) makes `doctrine design show 244` fail outright —
not just the change log, not just that row, the whole snapshot. It is the
strongest available evidence for the read-path horn, and it is deliberately
**not** gated on this question: it is a live breakage worth fixing on its own
merits, and its fix is half the answer.

## The candidate answers

Not costed; each is repo-wide, not per-item.

1. **Strict on the write path, tolerant on the read path.** Refuse unknown keys
   at submission; accept and preserve unknown rows when re-reading a stored
   snapshot. Splits the two directions the constraint conflates.
2. **Migrate.** Version the snapshot and convert on read. Highest cost, cleanest
   end state, and the only answer that lets both paths stay strict.
3. **Accept the limit; close the discoverability half instead.** `ISS-333`'s
   third candidate — a schema dump so the caller never has to guess a key. It
   addresses the observed cost rather than the observed mechanism, and `SL-251`
   is already taking that route on the neighbouring axis (the payload contract
   becoming fetchable rather than exemplified).
4. **Refuse the no-op submission.** `ISS-346`'s second half: a payload that
   admits, changes nothing and emits no event should arguably not advance the
   revision at all. Cheaper than (1) and would have caught the observed probes
   without touching serde. Care needed around `Admission::Resumed`, a legitimate
   no-advance path.

(3) and (4) are complementary to each other and to (1); (2) forecloses nothing.
The point of asking once is that four items answered independently would produce
four inconsistent rulings on the same tradeoff.

## What turns on it

`ISS-290`, `ISS-327`, `ISS-328`, `ISS-333` and `ISS-346` all carry
`needs QUE-219`. Each is blocked on this ruling rather than on effort.

## Evidence of cost

- `SL-248` design run, 2026-08-06: three successive probes
  (`{"sections": []}`, `{"sections":[{"foo":"bar"}]}`, `{"zzz_nonsense":[…]}`)
  all accepted at revisions 47–49, all establishing nothing. The answer was
  findable only by reading `src/design_run/submission.rs`.
- `SL-253`: two probe payloads absorbed; the run sat two revisions ahead of its
  own decisions with `resolved=0`. The agent stopped because `/design`'s
  degradation rule said so, **not because anything refused**. Observation
  `019ff42b-2655-7aa3-b042-0a83272a328c`.
- `SL-254` design run: `ISS-328`'s second sighting, the first that cost anything.
- `RFC-026` E8.7 — 15 of 33 source reads over `CHR-049`'s run were payload-shape
  lookups, the single largest category.

## References

- `ISS-290`, `ISS-327`, `ISS-328`, `ISS-333`, `ISS-346` — the gated batch
- `ISS-315` — the read-path failure, live and ungated
- `ISS-318` (resolved), `DEC-183` — the sibling axis and the ruling that scoped this out
- `SL-251` — takes the discoverability route on the payload-contract axis
- `mem.fact.design-run.snapshot-outlives-the-binary` — the binding constraint
- `SPEC-029` — Design run engine
