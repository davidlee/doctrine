# EVD-003: Compaction stranded aligned design sections before materialisation

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

During the SL-233 design interview, nine design sections were presented and
aligned with the user under the current `/design` contract, which delays
writing `design.md` until all sections have been reviewed. The sections
therefore existed only in conversational history when the agent context was
compacted.

The compaction summary preserved the decision graph and a useful semantic
outline, but not the complete section prose. Exact recovery required locating
and parsing the raw Codex JSONL session:

```text
/home/david/.codex/sessions/2026/07/27/
  rollout-2026-07-27T12-56-29-019fa180-a04b-7da1-b5a8-ae8326ae995d.jsonl
```

The assistant-authored section messages were then reconciled into
`.doctrine/slice/233/design.md` and committed as `6835ba7f`.

This incident supports DEC-072's choice to store each aligned draft section as
a stable, fingerprinted runtime record before final authored materialisation.
Had that mechanism existed, ordinary slice-ID resume could have recovered the
exact section bodies and alignment state without transcript archaeology.

The evidence is one dogfooding incident rather than a comparative evaluation.
It nevertheless demonstrates the concrete failure mode: durable semantic
records preserved the design's decisions, while the absence of resumable
section state stranded substantial reviewed prose.
