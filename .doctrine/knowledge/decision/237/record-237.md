## Decision

Recording an act emits exactly one `ActRecorded` change row: subject is the
record's own `DesignId`, payload is one term, `act=<kind>`, via
`PayloadTerm::token(PayloadKey::Act, ...)`. It is the exact mirror of
`ActInvalidated`, which already reports the death of a recorded act by subject
with the act kind as its one term (`run.rs:1850-1854`).

`ActRecorded` and `ReviewAttested` are **not** unified. They are two lifecycle
families, and the split is by replacement key, not by taste:

| family | key | recorded | invalidated |
|---|---|---|---|
| act records (`CheckpointAct`, `AgentDeclaration`) | **act kind** — one latest answer per kind | `ActRecorded` | `ActInvalidated` |
| section attestations | **attestation id** — reviewer lanes coexist | `ReviewAttested` | `ReviewInvalidated` |

`CheckpointActGroup::record` (`snapshot.rs:361`) retains-then-pushes keyed on
`ActKind`; `AgentDeclarationGroup::record` (`:387`) on the act discriminant;
attestations key on id, and that module's own doc calls the difference "the
whole design". Their inverse events already encode it — `ActInvalidated` carries
`act`, `ReviewInvalidated` carries `section` + `attestation`.

## Why one term and not more

`Act` (Token) and `Fingerprint` (Digest) exist in `PayloadKey`; `Basis` and any
coverage key do not. Carrying the basis or the covered set would answer a
question nobody asked — `ISS-355` is "did what I sent land?", which the subject
id and act kind answer completely — and would require minting keys, with prose
`ValueKind` for an unbounded basis.

One term also stays clear of `R5`: `WIDEST_PAYLOAD_EVENT` (`render/mod.rs:198`)
hardcodes `StageMoved` as the widest-payload exemplar at three terms, and
**neither** compile-time assert would catch a wider member — `WIDEST_ROW_BYTES`
sums `ENVELOPE_PAYLOAD_BYTES`, the budget, not the derived widest. A row over
three terms would silently falsify that constant.

## Naming bound

`REQ-437` (SPEC-029 NF-002) is discharged by the compile-time
`assert!(widest(ChangeEvent::ALL) <= DESIGN_EVENT_NAME_BYTES)`
(`change_log.rs:46`). `DESIGN_EVENT_NAME_BYTES` is 32 against a current widest of
27 (`section_fingerprint_changed`), so `act_recorded` at 12 bytes is
comfortable — but the headroom is five bytes and any future member's name is a
conformance question, not a taste question.

## Provenance

Reached in the `SL-256` design run with GPT-5.5 (codex) as peer advisor on type
design, not as reviewer. Independently agreed by both parties.