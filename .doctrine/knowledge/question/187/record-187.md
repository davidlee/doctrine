# QUE-187: Runtime section drafts and alignment authority

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Where do interactively presented design-section bodies, alignment receipts, and
staleness live before `design.md` is materialised? Should they be a separate
runtime collection with stable section IDs and content fingerprints, be written
incrementally into authored `design.md`, or remain transcript-only? How does the
drafting gate prove alignment without folding section lifecycle into run stage?

DEC-072 settles the storage and materialisation part: sections are stable,
content-fingerprinted records in the gitignored run snapshot and collectively
materialise `design.md`. The remaining alignment-authority question is refined
as QUE-188 because human and adversarial-agent review need distinct,
content-bound evidence.
