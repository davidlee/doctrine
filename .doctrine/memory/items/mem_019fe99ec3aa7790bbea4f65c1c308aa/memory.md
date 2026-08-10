Measured (SL-248 PHASE-09 `T9`, and again after the coordinator's correction).

Under `--unshare-pid`, an arm whose teardown is removed returns only when the
pid namespace empties: bwrap's namespace init holds the harness's captured
descriptors until then. So a detached descendant that closes its **own** copies
of stdout and stderr (`exec 1>&- 2>&-`) releases nothing, and the arm's wall
clock equals that descendant's whole lifetime plus ~130 ms.

Delay sweep through the real arms, escapee lifetime 23 s: control arm 23.13 s at
delay 0, then 24.11 / 25.13 / 26.13 / 27.13 s at delays 1–4. Exactly
`escapee lifetime + delay`.

**The trap this closes.** The same payload shape measured in a bare `sh -c`
returns in ~1 s, because there is no namespace and the descendant's own close
*is* the last write end. A bare shell is therefore the wrong instrument for any
claim about capture-pipe cost inside a capsule: it measures the **weakened** arm
and says nothing about the confining one. A measurement handed down as
authoritative was taken that way, and it inverted the cost conclusion.

**Consequence for payload design.** With a bound removed, whatever the payload —
or anything it spawned — sleeps is what the control arm costs. That is `EX-10`'s
rule generalised past row 8's wall payload. Size a detached descendant's linger
to the trusted side's *observation window*, not to "well over the arm's own
runtime": inside the namespace it cannot outlive the arm at all, so survival
beyond that window is unobservable and is pure cost.
