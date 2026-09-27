<!-- doctrine:section sec-1 -->
## The invariant

`SL-259` (*Truthful apply*) set the design run's exit-signal contract: an error
means nothing landed; a success reports rows that tell the truth; input the
engine will not honour is refused by name, and the refusal names the fix. This
slice closes four places that still break it (`RFC-031` triage, clump 1):

| item | what the run says today | what it should say |
|---|---|---|
| `ISS-482` | a new finding sent `blocking: null` lands non-blocking, silently | a refusal naming the two values it accepts |
| `IMP-499` | `SubmissionExpired` refuses with no remedy | the refusal names a safe way forward |
| `ISS-454` | one act's death reported twice across revisions | each death reported once |
| `ISS-488` | a question re-word lands with no row | one `node_question_changed` row |

The first two are refusals (sec-2); the last two are change rows (sec-3).
`IDE-057` (traversal-only applies) is the same requirement's next case and is
out of scope.

<!-- doctrine:section sec-2 -->
## Refusals

### A finding's `blocking: null` (`ISS-482`)

The finding home admits `blocking` only when the finding is created
(`submission.rs:748-753`, `KeyWhen::Only(Absent)`); on a held finding the
key is already refused `InertAtState`. The defect is confined to creation:
`declare_finding` reads `Declaration::finding_blocks`, which folds `Null` and
`Omitted` into `false`.

Change: `declare_finding` matches `blocking_declaration()` itself —

```text
Value(b) → b    Omitted → false    Null → Refusal::FindingBlockingNull { id }
```

— and `finding_blocks` is deleted (its only caller is this site). One accessor
then serves both homes, and the distinction between absence and `null` survives
to every reader.

A **new** variant, not `BlockingJudgementWithdrawn`: that refusal's remedy
("omit the key to leave the held judgement as it is") is false at a finding's
creation, where nothing is held and omission means non-blocking. Wording:

> `{id}` sends `blocking: null` — a finding's blocking judgement is set when it
> is raised: send `true` or `false`, or omit the key for a non-blocking finding

### `SubmissionExpired` names its remedy (`IMP-499`)

The refusal fires because Doctrine can no longer tell a retry from a new
submission, so "resubmit" is safe only if the original did not land. The
remedy routes through that check. Appended wording:

> … — check `design show` for whether the original landed, then resubmit only
> what is missing, at the current revision, under a fresh `submission_id`

Settled at inquiry (`inq-4`, `inq-5`, non-durable).

<!-- doctrine:section sec-3 -->
## Change rows

### A displaced act reports only a live death (`ISS-454`, `DEC-336`)

An act dies two ways. Coverage death is derived by `invalidation_rows` from
`live_acts(before) \ live_acts(after)`. Displacement — a same-kind recording
replacing the slot — is invisible to that difference (the slot key derives from
the kind) and is emitted at the record seam, `admit_and_record`. The act stores
**retain** coverage-dead acts (`live_acts` filters, it does not prune), so a
later displacement of an already-reported corpse emits a second
`act_invalidated`.

Change: `apply` already holds `live_acts_before` (`run.rs:354`). It is passed
to `record_act` and `record_declaration`, and on to `admit_and_record`, which
emits the displacement row only when
`live_acts_before.contains(&(kind, id))`.

Within one apply this stays exact: an act live at `prior`, killed by a declare
earlier in the same apply and then re-recorded, is absent from the coverage
difference (the replacement fills the slot) and reported once, by displacement.

`invalidation_rows`' doc keeps its disjointness claim and gains the reason it
now holds across revisions: the displacement row is gated on the same
liveness the coverage difference reads.

### A question re-word emits `node_question_changed` (`ISS-488`, `DEC-335`)

`REQ-478` requires a row per recorded mutation; a re-word is recorded and
emits none, and under `DEC-301` it re-faces the user's acts, so the log shows
the act voided without its cause.

- **Vocabulary.** `ChangeEvent::NodeQuestionChanged`, token
  `node_question_changed`, a member of both `READABLE` and `EMITTABLE`
  (`DEC-239`). `payload_terms` is empty: the subject is the node, and subject
  plus kind is complete (`DEC-237`). No term means no change to the widest
  payload, so the projection-bounds arithmetic does not move.
- **Emission.** In the node-update arm of `declare_node`, after the sparse
  question resolves: one row when the resolved text differs from
  `existing.question()`. None for a same-text re-declaration; none at creation
  (`node_created` covers it). A `null` question clears the text, which is a
  change and emits the row.
- **The stale comment** at `run.rs:1467-1470` ("state, not delta") is replaced
  with a pointer to `DEC-335`.

Compatibility (`DEC-249`): an older binary reading a snapshot that carries the
new token degrades that one row to a disclosed unreadable row; the run still
parses. No stored state type changes.

<!-- doctrine:section sec-4 -->
## Code impact

| path | what changes |
|---|---|
| `src/design_run/refusal.rs` | `Refusal::FindingBlockingNull { id }` and its text; `SubmissionExpired`'s remedy clause |
| `src/design_run/submission.rs` | `Declaration::finding_blocks` removed |
| `src/design_run/run.rs` | `declare_finding` matches `blocking_declaration()`; `live_acts_before` threaded through `record_act` / `record_declaration` → `admit_and_record`, gating the displacement row; `invalidation_rows` doc; `declare_node` emits `NodeQuestionChanged` on changed text; the stale "state, not delta" comment |
| `src/design_run/change_log.rs` | `ChangeEvent::NodeQuestionChanged` — `READABLE`, `EMITTABLE`, `as_str`, `payload_terms` (empty) |
| `src/design_run/tests.rs` | unit criteria for the displacement gate and the re-word row |
| `tests/e2e_design_state.rs` | payload-path criteria for both refusals; `every_event_fixture` gains a re-word apply |

Design-target selectors: the six paths above.

<!-- doctrine:section sec-5 -->
## Verification

Each criterion fails before its fix (red first).

- **VT-1** (`ISS-482`, e2e) — creating a finding with `blocking: null` is refused
  with the `FindingBlockingNull` text naming `true`/`false`/omit, and the
  snapshot is byte-identical; `blocking: true` and an omitted key still land
  blocking and non-blocking respectively.
- **VT-2** (`IMP-499`, e2e) — a submission below the replay window is refused and
  the text names the check-then-resubmit remedy.
- **VT-3** (`ISS-454`, unit) — over `run_with_a_map()`: a declare that
  coverage-kills `cpa-sufficiency-accepted` emits one `act_invalidated`; a later
  re-recording of the same kind emits **no** `act_invalidated` and one
  `act_recorded`. The positive control stays green: displacing a **live** act
  emits exactly one `act_invalidated` (the `ISS-367` case).
- **VT-4** (`ISS-488`, unit) — re-wording a held node emits exactly one
  `node_question_changed` naming it; re-declaring the same text emits none;
  creating a node emits none.
- **VT-5** (`REQ-478` roster, e2e) — `every_material_event_kind_persists_a_change_row`
  is red once the variant is added and green once `every_event_fixture`
  re-words a node.
- **Gate** — `doctrine check gate` green.

<!-- doctrine:section sec-6 -->
## Risks and residuals

- **Older binaries** show a `node_question_changed` row as unreadable
  (`DEC-249`). Accepted: the disclosed degradation path exists for exactly this.
- **Signature widening.** `record_act` / `record_declaration` gain a parameter.
  Private to `run.rs`; no public surface moves.
- **A revived act.** An act dead at `prior` whose coverage is later restored
  (content edited back) becomes live again with no row. Pre-existing, outside
  this slice; noted, not fixed.
- **Residual in the cluster.** `IDE-057` (traversal rows) remains the open case
  of `REQ-478`; batch fold order (`ISS-356`/`ISS-360`) is `RFC-031` clump 2.

