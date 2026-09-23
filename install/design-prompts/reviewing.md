# Obligation: reviewing

Get the design attacked before it locks.

## The attack surfaces

Once the design feels coherent, run a hostile pass over it before treating it as
done. Attack:

- vague sections, and places where a short sample would remove the ambiguity
- hidden assumptions
- weak verification
- missing code-impact detail
- missing, misread, or weakly applied ADR, policy and standard constraints
- prose that makes a human reconstruct the design from identifiers, review
  history, or locally invented terminology
- relationships, sequences, state changes, or ownership boundaries that need a
  diagram to be understood reliably
- diagrams that disagree with the prose, omit load-bearing edges, or merely
  decorate an inventory of boxes

Governance is not optional background reading when the design makes
architectural or workflow choices. Apply the constraints you loaded to the
choices the draft actually made — the same lens as drafting, aimed at a finished
artefact rather than a forming one.

Entering `reviewing` already opened this pass's RV ledger: `doctrine design
show <slice> --format prompt` names it as `review_pass RV-NNN`. Raise the pass's
findings there, whoever conducts it, and do not `review new` another:
`review-disposed` accepts only that RV.

Record findings and dispositions on that ledger. Do not copy review
chronology or finding-by-finding responses into `design.md` or slice notes. The
design holds current governing meaning; the ledger holds the review history;
durable rulings are promoted to the knowledge or governance record that owns
them.

## After the pass

- When findings arrive, invoke `/feedback` before changing the design. Its
  correction-safety pass governs adjudication and integration; do not treat a
  reviewer's observation and proposed repair as one claim.
- Integrate the feedback before offering next steps. Occasionally that means
  revisiting an earlier stage; the run will re-face every guard on the way back.
- Reconcile the owning slice — `slice-nnn.md` so scope, risks, acceptance
  criteria, open questions and follow-up direction still match the revised
  design, and `slice-nnn.toml` for relations and metadata. Relations move via
  `doctrine link` and lifecycle status via `doctrine slice status`, never by
  hand-editing.
- Offer the user the choice explicitly: a formal hostile pass via
  `/inquisition` or a printed prompt for an external adversarial reviewer, or
  moving on to the implementation plan.
- If meaningful tradeoffs or uncertainty remain unresolved, stop and `/consult`.

## Recording the lock

Once the user has settled the pass and accepted the design, recording it takes
two submissions. First the disposition, alone, because a submission holds only
one checkpoint act:

```json
{"run_uid": "<uid>", "known_revision": <n>, "submission_id": "dispose",
 "checkpoint_act": {"act": "review-disposed",
   "acceptance": {"basis": "user: \"no further pass needed\""},
   "disposition": {"waived": {"reason": "<the reason they accepted>"}}}}
```

Then everything else their final reply granted, together:

```json
{"run_uid": "<uid>", "known_revision": <n+1>, "submission_id": "lock",
 "checkpoint_act": {"act": "design-accepted",
   "acceptance": {"basis": "user: \"agreed, lock it\""}},
 "declare": [{"subject": "att-1", "attests": "sec-1"},
             {"subject": "att-2", "attests": "sec-2"}],
 "acceptance": {"basis": "user: \"agreed, lock it\""},
 "stage": {"to": "locked"}}
```

A concluded pass names the run's own pass RV (the envelope's `review_pass`)
instead: `"disposition": {"conducted": {"review": "RV-NNN"}}`.

## What the machine will reject

- Locking needs current section attestations and an integrated review. A stale
  attestation is not a current one, and re-reading it does not refresh it.
- A section attestation binds that section's content fingerprint and its
  reviewer lane: edit the section and only its attestation goes stale. A user
  acceptance binds more: the payload fingerprint, the disposition, the node
  and the revision. Change any of those and it is stale by construction; that
  is the point of binding it.
- Human section review is the v1 default. Configurable reviewer postures are
  deferred; do not invent one.
- Carry every finding to a disposition. An undispositioned finding blocks the
  lock and does not expire on its own.
- A review that found nothing is a result worth stating plainly, not a gap to
  fill with invented findings.
- The current design must stand alone. A history ledger may explain how it
  changed, but it may not carry context required to understand or implement it.
  *One provisional exception, under the RFC-026 P10 trial:* a routed finding's
  criterion sketch and placement constraint stay on the RV ledger and are not
  repaired into the design, and `/plan` is instructed to read them there.
  Nothing else may lean on the ledger this way.

## Routing a severe finding (provisional — RFC-026 P10 trial)

Applies to `blocker` and `major` findings on a design-review ledger. `minor` and
`nit` dispositions are unchanged.

Every severe finding carries one route, written as the first token of the
disposition:

    --disposition "route:<route> <vocab>"     e.g.  route:probe fix-now

The closed set is exactly five:

| route | the question behind the finding | what settles it |
|---|---|---|
| `review` | should we accept this commitment and its consequences? | design judgement and adversarial review |
| `demonstrate` | can these parts connect as proposed? | a thin implementation exercising the disputed connection — "it compiles" is not the bar |
| `probe` | does the mechanism withstand the adversary? | a stated adversary, then a hostile probe |
| `control` | would the planned check notice failure? | a negative control: name the concrete incorrect candidate the check must reject, and observe it rejected |
| `owner-fix` | do two accounts of one fact disagree? | remove the duplicate, verify the surviving owner, sweep the affected class |

The route and the vocab are different axes: the vocab records what you did, the
route records what instrument can settle the finding. There is no default.

**When more than one route fits.** Route on the claim whose failure would make
the rest of the finding moot. If two still fit, prefer any route other than
`review`. If two non-`review` routes still fit, take the first of `owner-fix`,
`control`, `probe`, `demonstrate`. Where the finding carries a genuinely
separable second arm, name it in `--response` so the raiser can raise it as a
sibling — a finding is immutable and cannot be split in place. If you cannot
tell which question the finding is asking at all, that is the ambiguity the
anti-escape guardrails already send to `/consult`, not a reason to write
`review`.

`demonstrate`, `probe` and `control` are the **instrument routes**, and they are
NOT repaired in prose. (`review` and `owner-fix` are settled the way they always
were.) In `--response` you write, as plain prose — no backticks and no dollar
signs:

- `probe` — the adversary, as *must hold against X, need not hold against Y*.
- `control` — the concrete incorrect candidate the check must reject. A control
  establishes discrimination against a named fault, not completeness.
- `owner-fix` — which duplicate goes, which owner survives, and the class you
  will sweep. The sweep is the clause people drop.
- all three instrument routes — what the criterion must assert, and what the
  obligation needs of its host phase. You cannot name the phase: phases are
  devised at planning, after this review. Name the constraint, not the phase.

The form rule is not style. `--response` is one shell argument with no file or
stdin form, so a backtick span or a dollar sign is expanded away before doctrine
sees it and the receipt still reads clean. Read your response back with
`review show <RV> --json` before moving on. There is no amend verb: if what you
read back is wrong, the only repair is to ask the raiser to `contest` so you can
re-dispose.

**When a finding may stay open past the gate.** It may not, if the next step
adds external reliance, durable state, authority or exposure, dependency spread,
or governing meaning on top of the thing in doubt. Otherwise the bounded next
step may proceed with the finding named. Measure the cost to regain an accepted
state, not the cost to regenerate a diff.

**Accumulation.** Do not route a second finding against a mechanism that already
carries one — that changes the argument and reopens the design decision. Another
test is not a disposition. Raisers: where several routed findings attack one
mechanism, contest rather than verify.

**Raisers, on a routed finding.** `verify` asserts that the obligation was
correctly transcribed onto a phase criterion — not that the defect is repaired.
That is a narrower claim than `verify` usually carries, and the `route:` token is
what tells a later reader which claim it was. It happens after `slice phases`,
not during this review: conclude the pass with routed findings `answered`.

Nothing validates any of this. The slice close gate will not let a blocker be
closed over unverified, which forces the verify act to happen — but no gate
reads what you wrote, and none checks that a criterion exists. `CON-006`
enumerates every unenforced clause with the code site proving it.
