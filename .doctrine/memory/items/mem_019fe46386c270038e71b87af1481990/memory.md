Measured 2026-08-09 inside `/workspace` (SL-248 PHASE-08, `F-36`):

| pid | sid | comm |
|---|---|---|
| 1 | **0** | `bwrap` — the sandbox itself |
| 2 | **0** | the agent process |
| 54 | **0** | `doctrine` |
| *n* | *n* | the shell a test suite runs under |

The harness does **not** share the session of the code it runs. Any instrument
that kills by session and protects only "its own session" therefore leaves
session 0 — the sandbox and the agent driving it — fully exposed. Killing it
presents as a **cold exit with no error**: the operator simply lands back in
their shell. It is indistinguishable from OOM at a glance, and there is no
`dmesg` in the jail to tell them apart. The tell for "the sandbox was rebuilt"
rather than "a process was killed" is `ps -o pid,etime,comm -p 1`.

The guard that is actually required has three parts, not one:

1. a **floor** (`session < 2` refused) — this is what catches session 0, and it
   is the part an `own_session`-style guard misses entirely;
2. the refusal applied at **record** time as well as at **kill** time, so a
   machine session never enters the swept set in the first place;
3. a **per-pid** floor as well, so pid 1 is never signalled whatever session it
   reports.

Plus: fail **closed** when the own-session is unknown (`None` → signal nothing).

The generalisable process rule, which cost two sandbox teardowns to learn: **a
destructive test instrument is floored before it is aimed, never after.** The
floor was scheduled as the phase's last task so as not to change code under test
while a 21-row mutation battery ran. The battery passed; the teardown happened
during the task carrying the guard, before it was committed. If a phase builds
something that signals, deletes or unmounts, its first commit carries the
refusal that bounds it.

Related: [[mem.fact.linux.session-leader-identifies-the-sandbox-subject]].
