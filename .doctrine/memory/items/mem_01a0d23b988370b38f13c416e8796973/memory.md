A comparator that decides "is the on-disk entry exactly what we would write?"
has two failure directions, and the under-check is the one people guard against
(a stale value never heals). The over-check is worse and easy to introduce:

    fn field_matches(handler, key, expected: Option<u32>) -> bool {
        match expected {
            Some(n) => handler.get(key).and_then(Value::as_u64) == Some(u64::from(n)),
            None    => handler.get(key).is_none(),   // <-- over-reach
        }
    }

`None` here means "this spec does not set the field". Encoding that as "the key
MUST be absent" makes the writer claim ownership of a field it never writes.
When the caller then heals a non-canonical entry by drop-and-reinsert (see
`plan_hook`), any operator-added key with that name is silently deleted — and,
because the entry is judged non-canonical forever, every install rewrites the
file.

SL-263 hit this: `additionalContextLimit` / `timeout` are codex handler fields
set only by the codex spec, but the `None ⇒ absent` rule applied them to the four
Claude specs and codex `boot_emit` too. `timeout` is a documented Claude Code
per-hook field, so a hand-set `"timeout": 60` was reverted on the next
`doctrine boot install` (reproduced live). The fix: `None` means "do not
compare" (leave a field the spec does not own alone); only `Some(n)` asserts a
value. That still heals a stale/missing owned field.

Rule: **a comparator compares what the writer emits, and nothing more.** Symmetric
to the other lesson (mem_019f286c92fe77a391635ba1d0743d5f — the comparator must
track the emitted value); together they bound it from both sides.

Corollary for review: if the codebase's own tests never seed an *extra* field on
an entry, the over-check is invisible and the suites stay green.