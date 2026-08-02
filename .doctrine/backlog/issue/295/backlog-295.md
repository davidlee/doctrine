# ISS-295: Checkpoint crash window between record write and journal is unrecoverable

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised as `RV-342` F-4 (major) during the SL-233 audit campaign and deferred
here rather than repaired in-slice: the repair amends a criterion of a
completed phase and contains a design fork. See "Why this is not a drive-by fix".

## The window

`execute_checkpoint`'s first-attempt `Create` branch (`src/commands/design.rs`):

1. `crate::knowledge::create_record` claims the id, journals `Reserved` from
   inside the `on_reserved` callback, and then materialises the record to disk.
2. Only after `create_record` **returns** does the next line journal the
   `Materialised` transition.

An interruption between those two events leaves the record **fully written on
disk** while the journal still says `Reserved`.

## Why retry cannot recover it

On retry the intent state is `Reserved`, so control takes the resume branch
(state below `Materialised`). That branch calls `knowledge::materialise_record_at`
→ `entity::materialise` with `MaterialiseRequest::InExisting` → `refuse_clobber`,
which bails with *"Refusing to overwrite existing …"* whenever any artifact
target already exists. Every subsequent retry takes the same branch and fails
identically. Recovery requires hand-repair of the journal or the tree.

This is not only a power-loss window. The two writes are separated by ordinary
user-interruptible work and the binary installs no signal handler, so a Ctrl-C
at that instant bricks the checkpoint.

## Why no test can currently reach it

`EX-8` supplies six named fault points (`CheckpointStep`) and **every one fires
before the effect it names** — verified across all eight call sites. There is no
fault point between record-on-disk and journal-says-`Materialised`, so the
unrecoverable window is unreachable through the fault seam by construction.

`post_journal_recovery_resumes_the_exact_reserved_id` appears to cover this and
does not: it crashes at record-materialise, which is *before* the write, so it
exercises retry with nothing on disk — the half where materialise succeeds. The
hard half is the one that cannot be reached and is not asserted.

## Scope

This does not violate `EX-2` or `EX-3` as literally worded — the id *is*
journalled before the record exists, and recovery *does* resume against the
reserved id rather than claiming a fresh one. It defeats the stated objective of
[[DEC-083]] and [[DEC-086]] (a recoverable checkpoint) in the one window where
recovery is impossible.

## Why this is not a drive-by fix

Three things, and only one is size:

1. **It needs a seventh fault point.** Closing the window means a fault between
   record-written and journal-says-`Materialised`. Only one test consumes a step
   token, so the code ripple is small — but `EX-8` *names* six, so this is a
   criterion amendment on a completed phase, which the audit campaign does not do.
2. **It touches shared machinery.** The resume path spans `commands/design.rs` →
   `knowledge.rs` → `entity.rs`, and `entity.rs` is under the
   behaviour-preservation gate.
3. **There is a live design fork inside it.** On resume with the reserved record
   already on disk: adopt it, refuse, or overwrite? Overwrite is exactly what
   `refuse_clobber` (D7, "no silent clobber") exists to prevent — a retry run
   days later would eat hand edits. Adopt-if-complete / refuse-if-partial needs a
   fileset-vs-disk classification that does not exist yet, and the **partial
   write** case (interruption *during* `write_fileset`) bricks identically but
   must not be adopted.

Note `write_fileset` is transactional on *failure* — it unwinds what it created.
It is not transactional against process death, so the partial case is real.

## Sketch of a fix

Make the resume branch idempotent by classifying the reserved record's scaffold
fileset against disk: all artifacts present → adopt (journal `Materialised` and
continue, matching the existing `CheckpointEffect::Adopt` semantics); none
present → materialise as today; **some** present → refuse with a diagnostic that
names the partial state and the hand-repair, which is strictly better than
today's opaque clobber refusal in the fully-recoverable case. Then add the
post-write fault point so the hard half becomes testable, and assert it.

Estimated ~40–60 source lines across three files plus two tests, on top of the
fork ruling.
