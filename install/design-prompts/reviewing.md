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

Record findings and dispositions on the design's RV ledger. Do not copy review
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

## What the machine will reject

- Locking needs current section attestations and an integrated review. A stale
  attestation is not a current one, and re-reading it does not refresh it.
- An attestation binds the payload fingerprint, the disposition, the node and the
  revision. Change any of those and it is stale by construction — that is the
  point of binding it.
- Human section review is the v1 default. Configurable reviewer postures are
  deferred; do not invent one.
- Carry every finding to a disposition. An undispositioned finding blocks the
  lock and does not expire on its own.
- A review that found nothing is a result worth stating plainly, not a gap to
  fill with invented findings.
- The current design must stand alone. A history ledger may explain how it
  changed, but it may not carry context required to understand or implement it.
