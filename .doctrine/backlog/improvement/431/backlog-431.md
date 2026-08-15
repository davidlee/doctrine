# IMP-431: slice notes has no read form

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`doctrine slice notes <ID>` reads as a getter and is a **create** verb. There is no
read form at all, so the bare-noun invocation — the natural first guess — is a
writer that then refuses to overwrite.

## Why it matters

The failure is quiet and mis-shaped: an agent asking to *see* the notes either
creates something it did not intend or receives a refusal about overwriting, which
describes an act it never asked for. Neither response tells it how to read.

This is a known family. `IMP-191` (*slice status: no read-only query form —
setter-only overload*) was the same defect on `slice status`, and it is **closed**.
The convention it settled — bare noun reads, explicit verb writes — was not carried
across to `notes`.

## Fix

Follow `IMP-191`'s resolution: bare `slice notes <ID>` reads; a `--set` / `new`
form writes. Worth a sweep of the other `slice` subcommands for the same overload
while the convention is in hand, so this is settled once rather than per-verb.

## Evidence

- `019fdfcc-ce76` — `slice notes <n>` reads as getter but is a create verb
- `019fd6a4-41ca` — `doctrine slice notes <id>` is a writer, not a reader

## References

- `IMP-191` — the same defect on `slice status`, closed; its resolution is the pattern
- `SL-086` — agent-facing CLI discoverability & output-format hardening
