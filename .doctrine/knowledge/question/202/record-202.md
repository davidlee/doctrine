# QUE-202: Capsule-model conflict admission path

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

**Question.** Under the RFC-025 capsule model, how is the **second of two
results from one base** admitted?

Refusal is settled and cheap: both capsules contract the same base `B`, the
first advances the accepted ref, and the second's stage-4 CAS finds the ref no
longer at `B` and refuses (`advance/stale-base`). Nothing auto-resolves and
nothing lands. That is a *safety* property and SL-241's rig proves it.

What is not designed is what happens **next**. The incumbent answers with
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

## Candidate shapes (not evaluated)

1. **Decouple the candidate verbs from their journal read** — re-ground
   `check_provenance` (REQ-316) on the capsule model's own proof: pinned OID +
   verify-capsule attestation + ancestry from a contracted base. Named as
   post-spike REV work in D2.
2. **Rebase-in-capsule** — the refused worker is still alive with its work
   intact (§ 5.4's retry loop); hand back the new base and let it redo the
   merge inside the capsule, where execution is already confined. Moves conflict
   resolution to the untrusted side entirely.
3. **A trusted-side 3-way over quarantine objects** — `git merge-tree` is
   object-only and needs no worktree, but re-deriving `Conflicted` semantics in
   the rig is exactly what DQ-1 disqualifies as evidence.

## What settles it

Not this slice. SL-241 runs H10/H16 on both harnesses and records the split;
the design work belongs to the post-spike REV that inherits
`mechanism-census.md`'s DELETE rows.

## Related

- SL-241 — the spike; design § 5.1 (the split), § 5.6 (the re-derivation),
  § 9 Closure (the scoping sentence).
- RV-340 F-9 — the finding that minted this.
- [[safe-capsule-ingestion-mechanism]] (QUE-200) — the ingestion-mechanism
  question; does *not* cover this.
- DEC-110 — reuse binds conflict semantics, not transport.
- RFC-025 `red-team.md` RT-2; `mechanism-census.md`.
