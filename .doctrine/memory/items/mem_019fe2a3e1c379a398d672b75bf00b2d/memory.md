## The situation

You need to observe process B *while* process A is still running — a liveness
window. The obvious design is a thread: spawn A on its own thread, publish
something (a pid) back to the main thread, and have the main thread run B
before A exits.

## The pattern

If the seam that spawns A already exposes a callback meaning *"A's top-level
process exists and I have not waited on it yet"*, run B **inside that
callback**. The window is then guaranteed by construction: the callback cannot
return before B does, and the spawner cannot reap A before the callback
returns.

```rust
fn execute_observed(
  &self, placement: &CapsulePlacement, execution: &Execution,
  observer: &dyn Fn(HostPid),
) -> Result<Observation, BackendError>;
```

## Why it beats the thread

- **No race.** The thread version has to *win* the window; this one owns it.
- **No `Send` / `Sync`.** The trait stays object-safe with borrowed closures,
  and the test doubles stay `Cell`/`RefCell` rather than `Arc<Mutex<_>>`.
- **No join / timeout / poison handling** on either side.
- The absence of a callback becomes a *typed* outcome — nothing was observed —
  rather than a timeout you have to distinguish from slowness.

## The costs, both real

1. **A's stdout is unread for the duration of the callback** if the spawner
   collects output after the callback (`wait_with_output`). A payload that
   fills the OS pipe buffer (64 KiB on Linux) blocks there. It blocks *alive*,
   which widens the window rather than closing it — but a design that needs A
   to make progress during the window must drain the pipe first.
2. **Fixtures are consumed in execute order, which is not the order the
   caller thinks in.** B runs before A completes, so a scripted double hands
   out B's answer first. Script subject-first and you get a green test
   measuring the wrong participant.

## Sample the window's end, not its start

Whatever records "was A still live for this observation" must be sampled
**after** B returns. Sampling before credits an observation of a process that
was there at launch and gone by the time B looked. To make that testable,
inject the liveness predicate rather than hard-wiring it — only a predicate
whose answer changes across B's run can tell the two orderings apart.
