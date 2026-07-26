# DEC-018: Observation correction is append-only

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Ordinary observation correction is append-only:

- supersession records point from an existing observation to a replacement
  observation; and
- retraction records remove an observation from the resolved active view without
  deleting its history.

Original observation files are not edited. Control records have their own UUIDs
and typed payloads while using the same four-field observation core.

Normal readers resolve supersession and retraction. Raw or history inspection may
show the complete chain.

## Rationale

Cheap permissive capture needs a correction path, but in-place mutation would
damage auditability and reintroduce merge contention. Append-only control records
preserve both the reported history and a clean current view.

## Hard-redaction boundary

Retraction is not redaction: it leaves the original bytes in the corpus and Git
history. Removal of genuinely sensitive material remains a manual operational
exercise. SL-231 does not provide hard-redaction or history-rewrite machinery.

That capability should be reconsidered only if operational experience demonstrates
a recurring need; the possibility alone does not justify product surface now.
