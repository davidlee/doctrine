# QUE-202: Capsule-model conflict admission path

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Question

Under the RFC-025 capsule model, how is the **second of two results from one
base** admitted?

Refusal is settled and cheap: both capsules contract the same base `B`, the
first advances the accepted ref, and the second's stage-4 CAS finds the ref no
longer at `B` and refuses (`advance/stale-base`). Nothing auto-resolves and
nothing lands. That is a *safety* property and SL-241's rig proves it.

The missing design was what happens **next**. The incumbent answers with
`candidate create`'s 3-way merge, its `Conflicted` classification, `admit`'s OID
pin, and supersede-on-moved-trunk guidance. Those are exactly the verbs
SL-241 § 2.1 F5 shows are welded to coordination staging: `candidate_create`
reads `journal.toml` out of `refs/heads/dispatch/<N>`, a branch the capsule model
does not have. So "reuse the conflict semantics" (DEC-110 / RT-2) and "no
coordination branch" cannot both hold as stated.

## Why it matters

This is the largest gap the SL-241 spike surfaces, and it is a *capability* gap
rather than a matrix row. Without a record owning it, it exists only as a
sentence inside D8's rejected-alternative discussion: invisible to any survey,
blocking no REV, and lost to whoever inherits the census after SL-241 closes
(RV-340 F-9).

It also bounds the spike's own claim. SL-241's go/no-go must read *sixteen rows
with a capsule-model boundary or a recorded dissolution, plus two incumbent
regression legs* — never "16/16" — precisely because H10/H16's resolution leg is
this question, not evidence.

## Answer

`DEC-137` settles the question by separating the existing candidate **engine**
from its coordination-branch **shell**.

After stale-base refusal, the trusted side explicitly supplies the current
accepted commit, pinned source commit, contracted base, and verify-capsule
attestation to the incumbent object-only three-way classifier. A clean
composition creates a normalized `Created` candidate. A real content conflict
records `Conflicted` and halts. The exact resulting candidate is verified in a
fresh verification capsule, admitted by immutable Git commit identifier, and
integrated by a journalled expected-tip compare-and-swap. Another accepted-ref
movement creates an explicitly superseding candidate; nothing silently rebases
or auto-resolves.

V0 retains the original source capsule frozen as live work. Formal repair runs
as a new transaction in a fresh capsule based on the current accepted commit;
the original environment is never resumed or granted more authority. Neither a
stale nor superseded result is automatically deleted. It becomes manually
cleanup-eligible only after an incorporating result is integrated and formally
closed, or after explicit operator abandonment. Full versus flattened canonical
history remains policy because the frozen capsule preserves the source until
that boundary.

This is a design answer, not new spike evidence. SL-241 still proves refusal
only; `REV-046` carries the answer into the target technical contract and its
implementation acceptance tests.

## Related

- SL-241 — the spike; design § 5.1 (the split), § 5.6 (the re-derivation),
  § 9 Closure (the scoping sentence).
- RV-340 F-9 — the finding that minted this.
- [[safe-capsule-ingestion-mechanism]] (QUE-200) — the ingestion-mechanism
  question; does *not* cover this.
- DEC-110 — reuse binds conflict semantics, not transport.
- DEC-137 — candidate admission plus frozen-source/fresh-repair lifecycle.
- RFC-025 `red-team.md` RT-2; `mechanism-census.md`.
