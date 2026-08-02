## The trap

`.doctrine/state/slice/NNN/design.toml` is **runtime tier** and is not a faithful
picture of what the engine believes. It retains the last *declared* value of a
field even when the engine no longer honours it.

Concretely, from SL-243's run `dr-019fc13a` at revision 9:

| read via | cursor |
|---|---|
| raw TOML — `[map.cursor]` | `at = "inq-4"`, `authority = "user-pinned"` |
| `doctrine design show 243` | `cursor unset STALE` |

Both describe the same revision. `inq-4` had been disposed, which clears the
cursor; the TOML block simply survives it. The engine is right and the file is
stale.

## Why it bites

Copying the file (`cp design.toml somewhere/`) looks like a faithful snapshot and
is not. A CHR-049 moderator snapshotted run state before an induced context break
in order to score whether the agent resumed "at the exact cursor and posture it
left", and captured a cursor the engine already considered unset. Scored as
captured, the agent's post-break `agent-proposed` cursor would have read as a
**lost user pin** — when the pin had been consumed with the node it pointed at.
The instrument would have manufactured a finding.

## The rule

Read design-run state with `doctrine design show <slice>` — `--format json` for a
machine-readable form, `--format status` for a human summary. Snapshot the
*rendered* output, not the file, whenever the snapshot will be compared against
anything.

The file is still the right thing to `grep` for facts the engine does not
derive — edge counts, receipt history, raw node lifecycles. Know which question
you are asking.

## Related

- [[mem.concept.doctrine.reading-entities]] — the authored-tier form of this rule.
  This is its runtime-tier counterpart, and less obvious because runtime state
  carries no `show`-only prose tier to warn you off the file.
- [[mem.fact.doctrine.storage-tiers]] — why this file is disposable in the first
  place.
- `mem_019fc1308b8e7b73acf9644547351373` — the same shape in another subsystem:
  grepping spec TOML for `[[source]]` anchors lies twice; read via
  `spec show --json`.
