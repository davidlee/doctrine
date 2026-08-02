# DEC-111: Anchor report reads the corpus in-process, not through doctrine's own JSON contract

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

`doctrine spec anchors` (SL-243) enumerates specs and their `[[source]]` anchors
**in-process**, through `spec::build_registry` / `entity::scan_ids` — the same
walk `spec validate` already uses. It does not shell out to
`doctrine spec show <ID> --json` per spec.

The published JSON contract remains the read path for **external** consumers and
for an agent doing a census by hand. The two must agree, and proving that is a
requirement of the slice, not a nicety.

## Why this is not a breach of read-via-`show`

The guardrail exists so that *agents* stop hand-parsing `.doctrine/` TOML and get
both tiers synthesised. The shipped verb **is** the engine: there is nothing
above it to defer to, and routing it through its own CLI would mean 49 process
spawns and 48 whole-corpus re-parses to produce a report one scan already
supports.

The specific hazards `mem.pattern.spec.read-anchors-via-json-not-grep` warns
about are absent on this path rather than merely avoided by care:

- **Slug symlinks cannot double-count.** `entity::scan_ids` keeps a directory
  entry only when its name parses as `u32`, so
  `002-requirement-reconciliation-engine` fails the parse and is skipped.
- **The scaffold's commented example cannot be read as an anchor.** The walk
  parses TOML; a `#`-prefixed `identifier = "doctrine/cli"` is a comment to a
  parser and a match to a grep.

Both hazards are properties of *grepping*, and the in-process walk does not grep.

## What this obliges

The report's correctness now rests on a walk that nothing otherwise ties to the
published contract. If they diverge, the report and `spec show --json` disagree
silently. So the slice owes a test asserting the two produce identical anchor
sets over the real corpus, checked against the recorded baseline — 48 specs, 81
anchors, 0 non-resolving. That test is the whole defence, which is why it is a
gate and not decoration.

## Provenance

Settled at the `inq-4` fork of design run `dr-019fc13a` (SL-243), on evidence
from the slice's research round: verified against the running binary that
`spec list --json` emits only `{id, members, slug, status, subtype, tags}` and
carries no anchors, while `spec show --json` carries the whole set. The slice
scope's prose implied both emitted anchors; that phrasing is what this record
corrects.

## Related

- [[mem.pattern.spec.read-anchors-via-json-not-grep]] — the census read path and
  the two ways a raw grep inflates the count. Correct as written; it addresses an
  agent, not the engine.
- [[mem.pattern.spec.source-anchor-liveness-unchecked]] — anchors rot silently.
