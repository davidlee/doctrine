Bash's `printf` builtin **flushes stdout after every call** (measured 2026-08-02,
bash 5.3, stdout a FIFO: a line written before a 5s `sleep` arrived at the reader
immediately, not at process exit). C-stdio full-buffering intuition does not
apply to it.

**What that buys.** Any long-running bash process that already emits a
progress line per phase can be interrupted at a chosen phase *without adding a
fault-injection flag to it*:

```bash
mkfifo -- "$fifo"
case "$-" in *m*) had_monitor=1 ;; esac
set -m                      # its OWN process group, or you kill your own shell
long_running >"$fifo" &
pid=$!
[ "$had_monitor" -eq 1 ] || set +m
while IFS= read -r line; do
  case "$line" in "phase=three done"*) kill -KILL -- -"$pid"; break ;; esac
done <"$fifo"
wait "$pid" 2>/dev/null || true
```

**Three details that are load-bearing, not style:**

- **`set -m` before the `&`.** Without job control a background job shares the
  script's process group, so `kill -- -$pid` kills the script. The pgid is fixed
  at fork, so restoring the flag straight after the launch is safe.
- **Signal the GROUP, not the leader.** Killing only the leader orphans its
  children — a sandbox, a compiler, anything still writing to disk. (A real
  parent crash *would* orphan them; the group kill is the stronger, tidier
  variant, and it changes nothing about what the interrupted work left behind.)
- **The writer blocks on opening the FIFO until a reader opens it.** That is a
  synchronisation point you can use — opening the read end is what releases the
  run — but it also means a reader that never opens hangs the writer forever.

**Why it matters beyond convenience.** The alternative is a timed kill, which is
a race, and a race is not a probe: the outcome varies run to run and a test that
sometimes interrupts before a commit point and sometimes after cannot assert
either. Synchronising on the subject's *own* emitted output makes the
interruption point deterministic and keeps the subject unmodified — you are not
testing a build with a test-only branch compiled into it.

Related: [[mem.pattern.doctrine.tdd-loop]] — a deliberately-failing path needs a
control proving it ran, and "the process was killed" is exactly the kind of
claim that passes vacuously if the process had already finished. Assert *where*
it died from evidence it emitted, not from the fact that you sent a signal.
