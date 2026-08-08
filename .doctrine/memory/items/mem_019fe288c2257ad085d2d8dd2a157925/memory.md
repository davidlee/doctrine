# A socket pair reports EOF only when every write end is closed

`read_to_end` on one end of a `UnixStream::pair` returns when the **peer** is
fully closed — not when the child that inherited it exits. Every live copy of
the peer descriptor keeps the reader blocked, including copies the reading
process itself is still holding.

## The shape that bites

Hand a child descriptors you own, wait for it, then drain your end:

```rust
let (capture, output) = UnixStream::pair()?;
let profile = Profile::with_stdio(output);   // profile owns the write end
let result = run(&profile)?;                 // child exits here
capture.read_to_end(&mut buffer)?;           // blocks forever
```

The child is dead and its inherited copies are gone, but `profile` still owns
the original. `Command`'s `Stdio::from` duplicates rather than consumes, so the
parent's copy outlives the run by construction.

## The fix, and why it is written not implied

```rust
let result = run(&profile)?;
drop(profile);                               // the last write end
capture.read_to_end(&mut buffer)?;
```

Scope order would eventually do this, but only if nothing later in the function
touches the profile — a refactor away from a hang with no compiler signal.
Write the `drop` explicitly with the reason at the site.

## Where it is easy to miss

The same run under a *different* configuration may pipe the child's output
through `Command`'s own capture, where the standard-library plumbing closes the
parent's ends for you. Only the branch that supplies caller-owned descriptors
has this obligation, so a test exercising the common branch proves nothing about
it. SL-248 PHASE-08 `T4` — row 12's control arm.

## Related

- `mem.fact.rust.cloexec-sweep-races-on-ebadf` — the other descriptor-lifetime
  trap in the same file.
