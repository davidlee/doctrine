# REQ-291: Per-harness enforcement altitude is a uniform contract with honest non-uniform reach: codex/pi pin base explicitly, catch worker-on-main via the DOCTRINE_WORKER env leg, and baseline-verify pre-dispatch; claude attains base==B by placement (cwd==coordination tree, baseRef=head) plus the post-spawn verify-worker ancestor check, with marker-only identity, no pre-dispatch baseline-verify, and a fail-open SubagentStart stamp.

## Statement

> **AMENDED — FALSIFIED (SL-254, 2026-08-14).** The title line above states the
> SL-056/SL-228 posture and is no longer true in any of its clauses. Enforcement
> is no longer *per-harness* and its reach is no longer *non-uniform*: it is
> KERNEL-level and identical on every harness. The claude leg it describes —
> base-by-placement, marker-only identity, the `SubagentStart` stamp, the
> post-spawn `verify-worker` ancestor check — described the in-session claude arm,
> which is deleted. The title is retained unchanged; the statement below is what
> must hold.

Enforcement is a **uniform contract with uniform reach**. There is one spawn
path, `scripts/spawn-confined.sh <harness>`, and every harness — claude included
— takes it.

1. **Enforcement altitude is the KERNEL, not the CLI.** The worker's exec is
   wrapped in a bubblewrap jail (Linux) or `sandbox-exec` (macOS). `--ro-bind / /`
   makes the entire filesystem read-only inside the jail; the fork's worktree is
   the write floor. This is not cooperative — a worker cannot decline it, and it
   does not depend on the harness honouring a hook, a skill instruction, or a
   tool allowlist.
2. **Fail closed at spawn, with no rungs below.** A missing `bwrap` is a NAMED
   refusal (`jail.rs::REASON_NO_BWRAP`) and spawn does not proceed. There is no
   unconfined fallback and no "degraded harness" / reduced-enforcement-altitude
   rung — that tier is abolished (`DEC-208`). Because enforcement cannot be
   partially attained, there is no per-harness altitude left to state honestly:
   confinement is either established or spawn refuses.
3. **Identity rides the same argv as the write floor.** The confinement prefix
   sets `DOCTRINE_WORKER=1`; the worker-mode guard keys on that variable and
   nothing else (`DEC-207`, REQ-192). Identity therefore cannot be present
   without the confinement that carries it, which is what makes the two uniform
   together rather than separately.
4. **Base is pinned explicitly, on every harness.** Base-by-placement
   (`worktree.baseRef='head'` with the orchestrator's cwd in the coordination
   tree) was the in-session base story and is gone; the subprocess path forks
   explicitly from a named base.

## Rationale

The original requirement's honesty clause — state each harness's reach rather
than silently assume uniformity — was the right discipline for a world with two
arms of genuinely different capability. SL-254 removes the need for it by
removing the difference: enforcement moved from a cooperative CLI/hook guard,
whose reach depended on what a given harness would honour, down to a kernel
boundary that no harness can decline. Honesty about non-uniform reach is
superseded by uniformity, and the residual cooperative guard is retained only to
turn an opaque read-only-filesystem error into a named refusal.
