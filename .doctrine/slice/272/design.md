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
key is already refused `InertAtState`. The defect is confined to creation,
and it reaches creation by two routes:

```text
direct apply ─────────────────────────┐
                                      ├─▶ Batch::validate ─▶ declare_finding
delegation.propose ─▶ rehearse_proposal ┘   (every declaration)   (reads finding_blocks:
   (stores the declarations;                                        Null and Omitted → false)
    Sparse::Null serialises as none,
    so a stored null reads back Omitted)
```

A fix inside `declare_finding` alone misses the proposal route: the rehearsal
runs `declare_node` for inquiry subjects only, and `nulled_keys` deliberately
excludes `blocking` on the premise — now overturned — that a finding's `null`
reads as absent (`RV-389` F-18). The stored proposal would lose the `null`
before `accept` ever applies it.

Change: the rule lives on the one seam both routes cross. `Batch::validate`
already runs `inert_key` and `inert_at_state` on every declaration; it gains a
third pure check, `Declaration::finding_blocking_null()`, returning
`Refusal::FindingBlockingNull { id }` when the subject is a finding and
`blocking` is `Sparse::Null`. It runs after `inert_at_state`, so a held
finding's `null` keeps its `InertAtState` refusal. `rehearse_proposal` calls
`validate` over the whole proposal, so a proposal carrying the `null` is
refused before it is stored; a direct apply is refused before any declaration
lands.

`finding_blocks` stays as the finding home's reader, its doc corrected: `null`
cannot reach it, because `validate` refused it, so its fold is omission →
non-blocking only. `nulled_keys`' doc drops the F-18 premise and points here.

A **new** variant, not `BlockingJudgementWithdrawn`: that refusal's remedy
("omit the key to leave the held judgement as it is") is false at a finding's
creation, where nothing is held and omission means non-blocking. Wording:

> `{id}` sends `blocking: null` — a finding's blocking judgement is set when it
> is raised: send `true` or `false`, or omit the key for a non-blocking finding

**The published contract says so.** The finding-home `blocking` row is
`Presence::Optional` today, whose meaning is only "absent means absent", and
`install/design-payload-contract.md` publishes it as plain `optional`. A fifth
presence state, `Presence::OptionalNullRefused` — absent means absent, `null`
is refused rather than read as absence — carries the new rule. It renders as
`optional` with the annotation `(omit → non-blocking · null refused)`, beside
the inquiry row's existing annotation. None of the four existing states says
it: `Optional` accepts `null` as absence, `Sparse` reads it as clear, and
`RequiredAtCreation` demands the key at creation. The shipped document is
regenerated from the renderer, to which it is pinned byte-for-byte.

### `SubmissionExpired` names its remedy (`IMP-499`)

The refusal fires when a submission id is unknown and its asserted revision is
below the receipt floor. The change log's floor advances on the same window,
and its rows carry no submission id, so neither Doctrine nor the caller can
learn from the run's history whether the original landed. A plain "resubmit"
could apply it twice. Appended wording:

> … — whether the original landed cannot be read from the run's history; check
> the current run state (`design show`: the map and sections, not the change
> log) for the effect you intended, and resubmit only what is absent, at the
> current revision, under a fresh `submission_id`

The check is against current state — does the node, section body, finding or
act the submission meant to write exist as intended — which is always
available, rather than against the bounded log, which is not. Resubmitting
only what is verified absent is what keeps a retry from applying twice.

Settled at inquiry (`inq-4`, `inq-5`, non-durable); the wording was corrected
at review.

<!-- doctrine:section sec-3 -->
## Change rows

### A displaced act reports only a death not yet reported (`ISS-454`, `DEC-336`)

An act dies two ways. Coverage death is derived by `invalidation_rows` from
`live_acts(before) \ live_acts(after)`. Displacement — a same-kind recording
replacing the slot — is invisible to that difference (the slot key derives from
the kind) and is emitted at the record seam, `admit_and_record`. The act stores
**retain** coverage-dead acts (`live_acts` filters, it does not prune), so a
later displacement of an already-reported corpse emits a second
`act_invalidated`.

The predicate: a displacement emits `act_invalidated` exactly when the
displaced act's death has not already been reported — it was live at `prior`,
or it was recorded earlier in this same apply.

The second arm is not hypothetical. One request may carry both a
`checkpoint_act` of `design-accepted` and the run-level `acceptance`, and the
shipped lock recipe sends exactly that; `apply` records the explicit act, then
the acceptance through the same `record_act` seam, so the second displaces an
act recorded moments earlier. A gate on `prior` alone would suppress that
death when no acceptance was live at `prior`.

Change: `apply` already holds `live_acts_before` (`run.rs:354`). It becomes a
mutable `reportable` set, passed to `record_act` and `record_declaration` and
on to `admit_and_record`, which:

1. emits the displacement row only when `reportable.contains(&(kind, id))`;
2. then inserts `(kind, id)`, since the act it just recorded is live and its
   own later displacement in this apply is a death to report.

The four cases, per slot:

| state at `prior` | recordings in this apply | `act_invalidated` rows |
|---|---|---|
| live | 1 | 1 (the prior act) |
| live | 2 | 2 (prior act; first recording) |
| dead, already reported | 1 | 0 |
| dead, already reported | 2 | 1 (first recording) |

In every row each death is reported once, and between two deaths of a slot
there is an `act_recorded` — `DEC-336`'s invariant. An act live at `prior`,
killed by a declare earlier in the same apply and then re-recorded, is absent
from the coverage difference (the replacement fills the slot) and reported
once, by displacement.

`DEC-336`'s predicate sentence ("live immediately before the apply") is
amended to this one; its invariant is unchanged.

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
| `src/design_run/submission.rs` | `Declaration::finding_blocking_null()`; `Batch::validate` calls it after `inert_at_state`; docs of `finding_blocks` and `nulled_keys` corrected |
| `src/design_run/run.rs` | `reportable` set threaded through `record_act` / `record_declaration` → `admit_and_record`, gating and extending on each recording; `invalidation_rows` doc; `rehearse_proposal` doc; `declare_node` emits `NodeQuestionChanged` on changed text; the stale "state, not delta" comment |
| `src/design_run/change_log.rs` | `ChangeEvent::NodeQuestionChanged` — `READABLE`, `EMITTABLE`, `as_str`, `payload_terms` (empty) |
| `src/design_run/payload_contract.rs` | `Presence::OptionalNullRefused` and its rendering; the finding-home `blocking` row uses it |
| `install/design-payload-contract.md` | regenerated from the renderer |
| `src/design_run/tests.rs` | unit criteria for the displacement gate and the re-word row |
| `tests/e2e_design_state.rs` | payload-path criteria for both refusals; `every_event_fixture` gains a re-word apply |
| `tests/e2e_design_delegation.rs` | `a_proposal_may_send_a_findings_blocking_null` inverted: the proposal is refused and not stored |

Design-target selectors: the nine paths above.

<!-- doctrine:section sec-5 -->
## Verification

Each criterion fails before its fix (red first).

- **VT-1** (`ISS-482`, e2e) — on both routes a finding created with
  `blocking: null` is refused with the `FindingBlockingNull` text naming
  `true`/`false`/omit: a direct apply leaves the snapshot byte-identical; a
  `delegation.propose` stores no proposal. `blocking: true` and an omitted key
  still land blocking and non-blocking respectively.
- **VT-2** (`IMP-499`, e2e) — a submission below the replay window is refused and
  the text states that the history cannot say whether the original landed and
  names the check against current state before any resubmission.
- **VT-3** (`ISS-454`, unit) — over `run_with_a_map()`, the four cases of the
  sec-3 table, counting `act_invalidated` and `act_recorded` rows:
  a declare that coverage-kills `cpa-sufficiency-accepted` emits one
  `act_invalidated`, and a later single re-recording emits none; one request
  carrying both `checkpoint_act` `design-accepted` and run-level `acceptance`
  emits one `act_invalidated` with no acceptance live at `prior` and two with
  one live. The positive control stays green: displacing a **live** act by a
  single recording emits exactly one `act_invalidated` (the `ISS-367` case).
- **VT-4** (`ISS-488`, unit) — re-wording a held node emits exactly one
  `node_question_changed` naming it; re-declaring the same text emits none;
  creating a node emits none.
- **VT-5** (`REQ-478` roster, e2e) — `every_material_event_kind_persists_a_change_row`
  is red once the variant is added and green once `every_event_fixture`
  re-words a node.
- **VT-6** (`ISS-482` contract, e2e) — `doctrine design contract --format prompt`
  renders the finding-home `blocking` row with `null refused`, and the existing
  byte-for-byte pin of `install/design-payload-contract.md` to the renderer
  stays green after regeneration.
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

