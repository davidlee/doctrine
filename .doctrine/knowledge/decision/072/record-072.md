# DEC-072: Design sections are first-class runtime records

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 will hold interactively developed design sections in the gitignored run
snapshot rather than writing an authored `design.md` piecemeal or relying on
the conversation transcript.

Each section has a stable run-local ID, order, title, Markdown body, and content
fingerprint. Section lifecycle is orthogonal to the run's coarse workflow
stage. Editing a section changes its fingerprint and therefore makes any
content-bound alignment or review evidence stale.

The ordered runtime collection is the source from which Doctrine materialises
the authored `design.md` at the drafting-to-reviewing boundary. This keeps
partially developed prose disposable while making progress inspectable,
addressable, and recoverable within the exact-resume guarantee of DEC-057.

Human and adversarial-agent section review must be first-class, distinct
evidence rather than one undifferentiated `aligned` flag. DEC-073 settles the
v1 review policy and evidence model: human section review is the default,
adversarial section review is first-class and opt-in, and an integrated
adversarial review remains mandatory.
