# DEC-086: Journal checkpoint record identity before materialisation

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Managed checkpoint creation splits the existing entity-engine fresh-creation
operation into a recoverable reserved-materialisation seam:

1. Persist a checkpoint intent keyed by the apply submission ID.
2. Claim a fresh canonical knowledge-record ID through the existing reservation
   backend.
3. Persist the claimed canonical ID in the checkpoint journal.
4. Materialise the DEC/QUE/ASM scaffold at that held reservation.
5. Apply its requested status and legal relation edges.
6. Persist the design snapshot with the canonical disposition and mark the
   journal complete.

A crash before step 3 may leave an empty or partial reservation but cannot leave
an unidentified authored record. Existing reseat/cleanup mechanics handle that
reservation and retry may claim another ID. From step 3 onward the journal names
the exact canonical target: recovery inspects that ID and resumes the first
incomplete effect idempotently.

The shared entity engine owns reservation and materialisation mechanics;
`knowledge` owns record semantics; the design command owns the cross-tier
operation journal. This is a narrow decomposition of existing creation
responsibilities, not a generic transaction framework.
