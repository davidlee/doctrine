# IMP-377: Review dispose accepts response from file or stdin

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The problem

`doctrine review dispose --response "<prose>"` takes a long free-text argument
inline. Review responses routinely contain code-quoted spans — command names,
identifiers, discharge outcomes. Passed through a double-quoted shell string,
every `` `…` `` span becomes **command substitution**: the shell evaluates it
and splices the (usually empty) output into the stored text.

The failure is silent where it counts. `dispose` exits 0 and prints
`Disposed F-9 on RV-325 (answered)`. Only stderr carries `command not found`,
and that scrolls past under a `| tail`. The corrupted response reads:

```text
… since  and  already exist to reach it …
… will report .
```

## Why this is worth fixing rather than documenting

The ledger is turn-based. Once disposed, a finding is `answered`, and `dispose`
refuses out-of-turn rewrites. There is **no amend verb** — the responder verbs
are `dispose` and `contest`, and `review unlock` only clears a stale lock file.
So a mangled response is permanent short of asking the raiser to contest the
finding, which costs an entire external review round. A quoting slip should
not be able to permanently damage an authored record.

Observed live on RV-325 F-9 (SL-233 PHASE-08), 2026-08-01. Recovery was to
disclose the mangling inside the *next* finding's response — a ledger that
now needs a caveat to read correctly.

## Proposed shape

Follow `git commit -F -`, and the precedent already set by **IMP-332** (the
record verb's stdin/file request):

- `--response-file <PATH>`, and `-F -` for stdin.
- Mutually exclusive with `--response`.

Worth applying to the sibling free-text arguments at the same time rather than
one verb at a time — `review raise --detail`, `observation record --detail`,
and any other verb taking prose long enough that an agent will reach for a
shell string.

## Cheaper partial, if the flag is not wanted

Make the receipt honest: if the process wrote to stderr, or if the response
argument is empty after substitution where a non-empty one was intended, say
so. Today a corrupted write is indistinguishable from a clean one at the
call site, so verifying requires a separate `review show --json` read-back
after every dispose.

Related: `mem.pattern.doctrine.review-response-shell-backtick-mangling`
carries the workaround (build the prose in a file, invoke via `subprocess`
with no shell).
