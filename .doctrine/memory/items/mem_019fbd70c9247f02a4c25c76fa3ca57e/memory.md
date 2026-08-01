# An absence assertion needs its subject reachable from the asserting vantage

Found in SL-241 PHASE-03, auditing PHASE-02's confinement probe.

A confinement profile's central claims are absence-shaped — *the canonical repo
is ABSENT, not read-only*; *no git credentials cross*. The natural assertion is

```sh
in_sandbox test ! -e "$HOME/.ssh"
```

and it is worthless unless `$HOME/.ssh` **resolves outside the sandbox**. Two
separate ways it silently does not:

1. **The subject never existed.** Nothing to hide, so nothing is proven.
2. **An OUTER layer already hid it.** The probe ran inside a bubblewrap jail
   that does not mount `~/.ssh`. The path exists on the host; the *jail* hides
   it; the capsule under test is not doing any work. The assertion passes with
   **no inner sandbox at all** — which is the check that finds it.

Both were live and green. A third route on the same section: a probe shaped
`[ -z "$(git config --get credential.helper)" ]` also passes when `git` is not
in the sandbox's allowlist, because the substitution just comes back empty.

## The test that finds it

**Run the assertion with the thing under test removed.** If it still passes,
it is decoration. This is the positive-control discipline pointed at absence
rather than at a grep — see
[[mem.pattern.harness.grep-negative-needs-positive-control]].

## The fix, and a bonus property

Gate on reachability from the prober's own vantage, and record `n/a` **with its
reason** when the subject is unreachable — never a silent green:

```sh
absent_inside() {              # <desc> <path>
  [ -e "$2" ] || { printf '  n/a   %s — subject not visible from here\n' "$1"; return 0; }
  assert "$1" in_sandbox test '!' -e "$2"
}
```

This makes the leg **environment-conditional by construction rather than by
verdict**: in a jail the subject is `n/a`; on a host run the same leg becomes
load-bearing with no edit. Prefer a subject that certainly exists and carries
the general claim (the host home ROOT) over a specific one that may not
(`~/.ssh`).

## Where it bites hardest

Nested confinement — a sandbox tested from inside another sandbox — because the
outer layer's hiding is indistinguishable from the inner layer's, and the
result reads as the profile working. Any "defence in depth" test run from
inside one of the layers has this shape.