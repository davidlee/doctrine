# IMP-499: Expired-submission refusal names no remedy

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`Refusal::SubmissionExpired` (`src/design_run/refusal.rs`) says the submission
asserts a revision below the retained replay window and that Doctrine "refuses
rather than guess" — but not what the caller should do next. Its sibling
`SubmissionReplayed` gained a remedy under `ISS-361`; this one was left.

## Why the remedy is not obvious

The refusal fires because Doctrine can no longer tell a retry from a new
submission. So "re-read and resubmit at the current revision" is safe only if
the caller *knows* the original never landed; if it did, a blind resubmit under a
fresh `submission_id` applies it twice. The remedy text must route the caller to
check first — e.g. read the change log (`design show`) for the intended effect,
then resubmit only what is missing, asserting the current revision.

## Shape of a fix

Reword the refusal to name that check-then-resubmit path, with a test asserting
the remedy wording (as `reused_submission_id_with_changed_payload_is_refused`
does for the replay refusal).
