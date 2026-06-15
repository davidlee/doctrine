# Doctrine CLI is source of truth

The `doctrine` CLI is the source of truth for command shapes — ids, flags,
subcommand names, argument order. Don't guess them from memory; ask the binary:
`doctrine --help`, then `doctrine <verb> --help` for any verb. Command surfaces
move faster than recall, so a guessed flag is a stale flag.

If you carry guidance that hardcodes a command shape and the CLI disagrees, the
CLI wins. See [[signpost.doctrine.cli-command-map]] for the verb tour,
[[signpost.doctrine.lifecycle-start]] for where each verb sits in the flow, and
[[signpost.doctrine.install]] for the installation path. For the reading
consequence — read entities via `doctrine <kind> show`, not raw files — see
[[concept.doctrine.reading-entities]].

Durable project knowledge lives in doctrine's own memory store, not the model's
recall. Before acting on a non-trivial assumption, query it:
`doctrine memory find` / `doctrine memory retrieve` (the `/retrieve-memory`
skill wraps these). Capture durable facts back with `doctrine memory record`.
The store, not this conversation, is the system of record — see
[[concept.doctrine.memory-model]] and [[signpost.doctrine.recording-memories]].
What the store writes where is governed by
[[fact.doctrine.storage-tiers]].
