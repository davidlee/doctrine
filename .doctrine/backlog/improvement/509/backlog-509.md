# IMP-509: Backlog has no body-edit verb

## What

`doctrine backlog edit` only transitions status/resolution. There is no verb to replace
or append to a backlog item's prose body, unlike `knowledge edit --body`. Agents
hand-edit `backlog-NNN.md`, and IMP-343's inconsistent filename makes that error-prone.

## Fix sketch

Add `--body` / `--append` (file or stdin, so prose avoids shell expansion — cf. ISS-486)
to `backlog edit`, mirroring `knowledge edit`.

## Evidence

`019ff160` (2026-08-11), `019ff631` (08-12), `01a0e868` (09-28), `01a0e9f9` (09-28).
