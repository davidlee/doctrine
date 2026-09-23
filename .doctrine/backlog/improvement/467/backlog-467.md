# IMP-467: Design-run gate prose: actor, batching, present-then-ask

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Rewrite the prose that design-run delivers to agents so that the gates require
the friction you'd expect — "I've laid it out; say agreed or proceed to X" —
and no more. Abbreviated process, no slice. Parts may warrant an independent
review.

## Evidence (preflight, 2026-09-23)

- The discharge lines (rendered from `gate.rs` into the `design resume`
  envelope and `install/design-run-stages.md`) read "the user performs `<act>`".
  Agents read that as "the user must run the CLI". DEC-088 says the agent
  submits the act, and asserts the user's explicit acceptance.
- `hymns/stage/design.md`: "You propose; the user accepts. A payload cannot
  declare itself accepted." This reads as a ban on the agent recording the
  acceptance.
- All nine `install/design-prompts/conditions/*.md` fragments warn only
  against giving the act lightly. None says what a sufficient discharge looks
  like or what to show the user, so agents over-comply inconsistently.
- `basis` (AcceptanceDeclaration / AgentActDeclaration) is shown only as
  `text required`, so agents ask the user to author it. DEC-088's meaning is
  "concisely identify what the user accepted". It can cite or quote the
  user's reply. DEC-088 itself suspects the field is paperwork tax (CHR-049).
- Batching is undocumented. Section attestations are `att-` Declarations in
  the `declare` array (several per submission), and `checkpoint_act` rides the
  same request. IMP-464 assumes one submission per section.
- The inquiry-map UI does not exist (ISS-299), so the agent has to make each
  gate readable to the user, and nothing tells it how.

## Scope

1. **Explicit actor.** For every user act, say that the user assents in
   conversation and the agent records it. Change the rendered discharge
   wording in `gate.rs`; the pinned docs and tests move with it.
2. **Basis semantics.** One line: what `basis` is, and that a quote or
   paraphrase of the user's reply plus the harness turn is sufficient.
3. **A worked batched example.** One submission covering N `section-reviewed`
   attestations + `review-disposed` + `design-accepted`. **First, probe** that
   the engine admits it in a single apply. If it doesn't, the rest of IMP-464
   stays engine work.
4. **A present-then-ask script per user gate** (`governance-confirmed`,
   `graph-reviewed`, `sufficiency-accepted`, section review, `review-disposed`,
   `design-accepted`): what to lay out, which wording satisfies the condition,
   and a single-word reply that grants it.
5. **Rebalance the condition fragments.** Keep the *why*, and add what an
   adequate light-touch discharge looks like.
6. Fold in **ISS-287**: `reviewing.md` misdescribes what a section
   attestation binds.
7. Retire the "a payload cannot declare itself accepted" phrasing in favour of
   a citation of IMP-466's authority ADR.

## Out of scope

- Engine changes beyond the discharge-line wording, apart from the batching
  result from item 3.
- The envelope-affordance redesign (IMP-390) and the map UI (ISS-299).

## Done when

Every user-gate discharge names who records the act and how; a batched lock
example exists and has been verified against the binary; the generated docs
are regenerated and their pin tests pass.
