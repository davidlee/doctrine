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
