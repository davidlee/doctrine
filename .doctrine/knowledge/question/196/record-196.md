# QUE-196: Question sources admitted during existing-design import

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

DEC-084 permits an explicit conservative import of an existing `design.md`.
That raises two possible sources for a reconstructed inquiry frontier:

1. non-terminal QUE records that directly shape the slice; and
2. doc-local `OQ-*` entries in authored design prose.

## Options

1. **Tiered admission (recommended).** Bootstrap every non-terminal `QUE` with
   a direct `shapes SL-NNN` edge as a durable open inquiry. Also recognise
   conventional `OQ-*` entries only within the design's explicit Open Questions
   section, importing them as unverified `imported-prose` inquiry proposals
   with source fingerprint/location. They are not knowledge records, accepted
   truth, or gate evidence. If an OQ explicitly cites a canonical QUE ID, one
   node carries both references; otherwise Doctrine never deduplicates by text
   similarity and instead flags possible overlap for inspection.
2. **Durable QUE records only.** Bootstrap direct, non-terminal shaping QUEs and
   ignore all prose OQs. This has the strongest semantics but can strand
   intentionally documented questions that predate or escaped knowledge
   capture.
3. **All question-like prose.** Parse OQ labels and natural-language questions
   throughout the design. This maximises recall but introduces heuristic
   interpretation, unstable identity, and false inquiry nodes.

The recommendation is option 1. It uses the relation graph where authoritative
durable identity exists, admits a narrow conventional prose bridge without
laundering it into truth, and refuses speculative text-similarity merging.

## Answer

Option 1 was accepted and is recorded by DEC-085.
