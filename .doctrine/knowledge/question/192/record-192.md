# QUE-192: Prompt fragment repetition and cache receipts

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

How should repeated `doctrine design show --format prompt` calls avoid
reinjecting stable guidance already present in the caller's context?

The active skill definition is delivered by the harness and is never part of
the per-turn prompt. The repeatable units are the invariant `stage/design`
hymn and the selected coarse process fragment.

Options:

1. Content-addressed receipts. Every emitted fragment carries a stable name and
   content digest; the caller repeats
   `--known-fragment <name>@<digest>`. Doctrine omits it only on an exact digest
   match and otherwise emits the current body. Repeat the flag for multiple
   fragments.
2. Blind omission through `--omit-prompt <name>`. Simpler, but can silently
   suppress changed guidance after an upgrade or asset edit.
3. Always repeat both fragments in v1 and defer caching.

Option 1 is recommended. It adds a small stateless projection input, works
across compaction and session boundaries, and avoids building a server-side
prompt-delivery ledger. The TurnEnvelope remains present every turn because it
contains volatile state.

## Answer

DEC-078 chooses option 1: stable prompt fragments use content-addressed caller
receipts, while the dynamic TurnEnvelope is emitted on every projection.
