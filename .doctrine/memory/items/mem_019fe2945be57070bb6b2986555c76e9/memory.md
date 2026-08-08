## The problem

A trusted parent spawns a sandbox and needs the **subject's** host pid — for a
liveness probe, or to name a session to reap. What `Command::spawn` hands back is
the *immediate child*, which is a wrapper: under a wall bound that is
`timeout(1)`, and beneath it `bwrap` forks again. Counting wrappers is wrong the
first time a profile drops one.

## The rule

The subject is the **nearest descendant of the immediate child whose session id
equals its own pid** — i.e. the nearest session *leader* below the child.

- `bwrap --new-session` calls `setsid()` in exactly the top-level sandbox
  process. Nothing between the trusted side and it is a session leader.
- The kernel reports that session to a host-namespace reader as the **leader's
  host pid**, so the sid you read is also the pid you want. Reading
  `/proc/<pid>/stat` from outside the pid namespace is enough; you never need to
  enter it.
- Because it names a property rather than a depth, the same rule holds when the
  wall bound is removed (no `timeout`) and when the pid namespace is removed
  (`--unshare-all` swapped for an enumerated set).

Measured on bwrap 0.11.2:

```
reader sid 1310932
1310936 ppid=1310932 pgrp=1310936 sid=1310932   timeout -k 2 10 bwrap …
1310938 ppid=1310936 pgrp=1310936 sid=1310932   bwrap …
1310939 ppid=1310938 pgrp=1310939 sid=1310939   bwrap …   ← the subject
```

## Three ways to get it wrong

1. **First match instead of nearest.** A payload that `setsid`s a descendant
   creates a *second* leader below the same child. `/proc`'s directory order is
   neither numeric nor stable, so first-match names the escapee about half the
   time. Take the minimum by depth, and break ties on the lower pid so the answer
   is a function of the process table rather than of readdir order.
2. **Member instead of leader.** After the top-level process exits, its children
   keep its sid but none of them *is* the leader. `session == pid` answers
   `None` there, which is the honest result; a `session != mine` filter answers
   with an orphan whose liveness means nothing.
3. **Field index off by one.** In `/proc/<pid>/stat`, `comm` is the payload's own
   `argv[0]` basename, unescaped, and may contain spaces and parentheses. Split
   on the **last** `)` in the line, never on whitespace from the left. After that
   split, field 1 is ppid, field 2 is **pgrp**, field 3 is sid. Confusing 2 and 3
   gives you a process-group kill wearing a session kill's name, which passes
   every test whose payload never actually detaches.

## Corollary

Excluding your *own* session needs no code: a process leading your session
predates the child you spawned, so it can never be that child's descendant. Keep
the "what is my sid" call for the teardown sweep, where it is the thing that
makes a foreign session identifiable — not for the descent, where it is a guard
that can never fire and therefore can never be tested.
