## The footgun

`doctrine review dispose --response "<text>"` takes free-text prose. Review
responses are exactly the place you write code-quoted spans — command names,
identifiers, discharge outcomes. In a **double-quoted** shell argument, every
`` `…` `` span is **command substitution**: the shell runs it and splices the
output in.

The failure is silent in the way that matters. `dispose` succeeds, the
receipt says `Disposed F-9 on RV-325 (answered)`, and the stored text has the
quoted spans replaced by empty strings:

```text
… since  and  already exist to reach it …
… will report .
```

Only stderr betrays it, and it scrolls past under a `| tail`:

```text
(eval):1: command not found: slice
```

## Why it is expensive here and not elsewhere

The review ledger is **turn-based**. Once disposed, a finding moves `open →
answered` and `dispose` refuses to rewrite it — *"out of turn on F-9: current
status answered != required open"*. There is **no amend verb**: the responder
verbs are `dispose` and `contest`, and `review unlock` only clears a stale
lock file left by a hard kill. So a mangled response is **permanent** until
the raiser contests the finding, which costs a whole external review round.

Same trap as leaving a stale response behind — see
[[mem.pattern.doctrine.review-dispose-settle-remedy-before-disposing]].

## What to do instead

Never build a `--response` (or `--detail`, or `--text`) argument in a
double-quoted shell string. Write the prose with the `Write` tool, then invoke
without a shell:

```python
python3 -c "
import subprocess
t = open('/path/to/scratch/response.txt').read()
r = subprocess.run(['./target/debug/doctrine','review','dispose','RV-325',
                    '--finding','F-10','--disposition','fixed','--response',t],
                   capture_output=True, text=True)
print(r.returncode, r.stdout.strip(), r.stderr.strip())
"
```

Single-quoted heredocs (`<<'EOF'`) are safe for `git commit -F -` and are the
right tool there, but they do not help for an argument that must be passed
inline.

**If it has already happened:** do not try to overwrite. Disclose the damage
in the next finding's response, quoting the correct text — the reviewer reads
the whole ledger. Say which spans were eaten and that the artefacts are
unaffected.

## Verify before moving on

`review show <RV> --json` and read the stored `response` back. A `Disposed …`
receipt is not evidence about what was stored.