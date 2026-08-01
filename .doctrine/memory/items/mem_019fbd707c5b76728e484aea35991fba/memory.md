# A word-split command string loses its quoting, and the failure is silent

Found in SL-241 PHASE-03, wiring a verify capsule to run the `verify:` command
from an interpretation-surface declaration.

A command read from a config file and expanded unquoted is split on IFS with
**no quote processing** — shell quoting is done by the *parser*, and the parser
already ran. So the quote characters survive into the argument vector as
ordinary bytes:

```sh
cmd='node -e "process.exit(1)"'
exec $cmd            # node receives THREE words:
                     #   node · -e · "process.exit(1)"   <- quotes are literal
```

Node then evaluates `"process.exit(1)"` — a string literal expression — does
nothing, and **exits 0**. A verify step that should have refused instead
attested a passing run.

## Why it survives review

The failure needs a config value that contains quoting. The obvious examples
usually do not: `npm test`, `cargo test`, `make check` all word-split
correctly, so the naive expansion looks right for as long as anyone tests it
with them. It first bites on a real client's declaration, far from the code.

It was caught here only because a deliberately-failing scenario built to go RED
came back green — by a negative control, not by reading.

## The fix

Pass the string to a shell, which is the thing that knows how to parse it:

```sh
exec sh -c "$cmd"
```

`sh` execs a single simple command, so the exit status is still the command's
own — an important property when that status IS the verdict. If a shell layer
is unacceptable, the config must carry an **array**, not a string; there is no
third option, because "split a string into words correctly" is exactly the job
of a shell parser.

## Generalises to

Any place a command, path list, or flag set crosses from data into an argument
vector: CI step strings, `Exec=` lines, JSON `"command"` fields, task-runner
config. The tell is `$var` unquoted in command position with a comment
explaining that word splitting is intended.

Same family as [[mem.pattern.shell.shebang-interpreter-is-a-mount-dependency]]
and [[mem.pattern.shell.guard-exit-swallowed-by-command-substitution]]: in
shell, the *invocation form* silently changes the semantics of a mechanism that
reads as correct.