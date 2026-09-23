# IDE-056: Refuse design-run regression after slice audit

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised by the user in `SL-261` design (inq-7, 2026-09-24): refuse regressing a
design run once its slice is audited, so reconcile's direct edits to
`design.md` are final.

Assessed as hygiene, not safety: no loss path needs it. `materialise` refuses
on a diverged watermark (`commands/design.rs:2073`), and regress → `adopt`
takes the reconciled document as truth (`DEC-279`). Costs a new coupling: the
design run would read slice lifecycle status, which it does not today.
