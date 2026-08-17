# ISS-443: A peer-intended relation target silently resolves to an unrelated local record

`doctrine link` writes a **false edge without a diagnostic** when the target was
meant to name a record in a peer corpus. Not a missing feature — a wrong write
that every validation layer passes.

## The chain, run end to end

Authoring `ADR-022`'s own cross-boundary citation, on this corpus:

```
$ doctrine link IMP-441 references --role concerns oubliette:EVD-002
Error: unknown kind prefix `oubliette:EVD` in `oubliette:EVD-002`

$ doctrine link IMP-441 references --role concerns EVD-002
linked: IMP-441 references EVD-002
```

The second command succeeded. `EVD-002` in **this** corpus is *`claude -p`
exposes token usage metrics without usage billing*; the intended target was
oubliette's *Time-to-interactive is about two minutes, not 8.31 s*. Two unrelated
records, same number. The edge was written into `backlog-441.toml` and removed by
hand afterwards.

**Every layer passed, correctly.** Write-strict validation refuses a dangling
target, an illegal kind, and an illegal role — this target dangles at nothing, its
kind is legal for the label, and the role is well-formed. `doctrine check`'s
prose-citation scan reports *unresolved* ids, and this one resolves. There is no
bug in any of them: the id is simply ambiguous, and nothing in the system knows
there is a second namespace.

**The first error is what makes it likely.** `unknown kind prefix` reads as *you
typed the ref wrong*, so the obvious next move is to drop the qualifier — which
is the move that silently corrupts the graph. The refusal points at the footgun.

## Fix, and what it deliberately is not

Recognise the `<name>:<ID>` shape at the reference parser and refuse it **as a
peer-corpus target**, naming the limitation and the sanctioned alternative:
prose plus the record's free-text `[evidence]` block (`ADR-022` clauses 3 and 4).

That is the whole change. It resolves nothing, reaches no second corpus, and
needs no peer declaration — a shape check and an error string. What it buys is
that the qualified form becomes **known and rejected** instead of unrecognised,
so the bare fallback stops looking like the working option.

Whether a peer target should ever *resolve* is `IMP-441`, and is speculative.
**This item must not wait on it.** Coupling a live wrong-write to a feature nobody
has committed to is how the footgun stays loaded.

## Not covered

A bare local id typed in **prose** while meaning a peer's record — `ADR-022`
clause 3 records four such collisions among the ids needed to describe its own
subject. Nothing can detect that from the text; the qualified prose form is the
only defence, and it is a convention, not a check.
