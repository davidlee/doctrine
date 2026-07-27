# QUE-197: Checkpoint identity across the authored creation crash boundary

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

DEC-083 requires `design apply` to create a DEC/QUE/ASM checkpoint without
silently duplicating authored truth after a crash. Inspection of
`knowledge::run_new` and `entity::materialise` shows that current creation
claims a fresh numeric directory, writes the scaffold, and only then returns
the canonical ID to its caller.

Therefore a process death after successful authored write but before the design
journal captures the return value can leave a record whose identity is unknown
to the checkpoint operation. The design currently says this condition blocks
for reconciliation but does not define a public way to resolve it.

## Options

1. **Journalled reservation before materialisation (recommended).** Split the
   existing entity-engine creation seam so the design shell can: journal the
   operation intent; reserve a fresh canonical ID; journal that ID; materialise
   the knowledge record at the held reservation; perform its legal relations
   and status transition; then commit the design snapshot. A crash before the
   ID is journalled can leave only an empty/partial reservation, not an authored
   record; a crash after it is journalled always has an exact canonical target
   to inspect and resume. This refactors existing reservation machinery but is
   not a generic cross-tier transaction framework.
2. **Caller-stable origin token.** Journal a submission token and add structured
   origin metadata plus idempotent lookup to knowledge creation. A retry finds
   the record by token. This avoids splitting reservation/materialisation but
   expands the knowledge schema and query contract for a design-specific need.
3. **Explicit human repair.** Keep the ambiguous window. Put the run into
   `checkpoint-recovery-required` and accept a repair declaration that either
   binds a discovered canonical record or explicitly authorises
   create-anyway. This minimises shared-engine work but can strand automated
   recovery and permits a knowingly possible duplicate.

The recommendation is option 1. Reliable checkpoint creation is SL-233's
primary adherence mechanism, and the current entity engine already owns
fresh-ID reservation. Exposing a recoverable reserved-materialisation seam
removes the ambiguity at its source with less semantic expansion than adding
design provenance to every knowledge record.

## Answer

Option 1 was accepted and is recorded by DEC-086.
