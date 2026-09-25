This boot sector is **universal** — model-agnostic by construction. It rides the
prefix cache and survives a model switch (`/model`) untouched. No per-model,
per-role, or per-harness content is baked into this snapshot.

Those supplements live in the **hymns** cascade and are pulled on demand, never
written here:

    doctrine prompt resolve --role <role> --band model --model <id> [--model <id> …]

`--role` is required — pass `worker` or `orchestrator`. `--model` is repeatable —
each occurrence adds a key to your context trait set, and a band selector matches
only when its whole pinned set is present (a conjunction). Pass every key that
describes you, not just one; `doctrine prompt model-keys` lists the keys the
corpus defines, and `doctrine prompt explain` shows why each snippet matched.

Resolve the cascade with `doctrine prompt resolve` rather than reading the
corpus by hand: the selector match, the replacement graph, and the assembly
order are the verb's contract.

**Floor directive — act on this now:** self-identify your model, run the resolve
above to load your model band from the hymns corpus, and re-resolve whenever your
model changes. Best-effort: it degrades gracefully; no correctness invariant
depends on it.
