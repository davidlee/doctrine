# Obligation: inquiry

Shape the question space before drafting anything.

Delivered every turn of **exploring and inquiring** both — the two stages are
one kind of work, and none of the craft below is specific to either.

## The loop

A light loop that clarifies intent, surfaces known and unknown unknowns, and
drives towards enough clarity to lock a design. Apply it to the slice scope
itself first where the scope is the unclear thing, then to the technical design.

Each turn:

- Summarise what is already understood, the assumptions you are carrying, and
  the open questions, risks, concerns and dependencies still live.
- Take unresolved questions **one at a time**, choosing the most impactful or
  the one most naturally related to what was just settled. Consider it properly
  — implications, adjacent questions — and explore related context only as far
  as the question needs.
- Offer two or three options with their tradeoffs, and recommend one with your
  reasoning. A question with no options asks the user to do the design.

Continue until there is enough clarity to begin the design proper and the user
has accepted your summary.

## Craft

- Prefer multiple-choice questions where they fit; open-ended is fine too. A
  project may legitimately disagree — some want prose forks in design loops —
  and that disagreement is what makes this craft rather than doctrine.
- One question per message. A topic needing more exploration is more than one
  question, not one longer message.
- Focus on purpose, constraints, success criteria, and verification strategy.
  Those are what every later stage argues from.

## What the machine will reject

- Add inquiry nodes for the questions that actually shape the design. A node is
  a question worth a decision, not a task to tick off.
- Disposition blocking inquiries explicitly. Advancing to drafting needs every
  blocking node dispositioned and the user's acceptance — not your judgement
  that they are unimportant.
- To merge two inquiries, cite the canonical record by id. Text similarity never
  merges: if two nodes are the same question, say so with a citation.
- Imported prose enters unverified, carrying its source location and
  fingerprint. Leave it unverified until it is actually verified.
- Record a traversal change with its reason. A redirected cursor with no reason
  reads as drift, and the envelope will show it as exactly that.
