# A `${x:-DEFAULT}` is only inert *in the command that consumes it*

Writing `${PID:-0}` to keep `set -u` quiet looks like a null default. It is not
one for `kill`:

```bash
kill -0 "${PID:-0}"      # PID unset → kill -0 0 → signals the CALLER'S OWN
                         # PROCESS GROUP → exit 0 → the guard PASSES
```

So a liveness guard written this way passes for exactly the state it exists to
catch. Found when a mutation-testing run no-op'd the thing being guarded and the
mutant **survived** (SL-241 PHASE-05, F-P05-34).

## Do this instead

Test emptiness separately, then use the value:

```bash
[ -n "${PID}" ] || return 1
kill -0 "${PID}" 2>/dev/null || return 1
```

## The general rule

A fallback is inert only if the fallback **value** is inert in the **operation**
below it. `0` is a fine identity for arithmetic and a live process-group id for
`kill`. Empty string is inert for `grep -F` and a match-everything pattern for
`grep -E`. `.` is inert as a path join and matches any character in a regex.

Every `:-` deserves the question: *what does this value MEAN to the command that
receives it?*

## Same family

- [[mem.pattern.testing.count-the-repetition-not-the-sameness]] — a clause that
  passes for a reason unrelated to its subject; also found by a surviving mutant.
- [[mem.pattern.shell.ifs-tab-read-collapses-empty-fields]] — the other shell
  default that silently answers a different question.
