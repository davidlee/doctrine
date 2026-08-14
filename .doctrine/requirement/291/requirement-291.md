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

Enforcement is a **uniform contract with uniform reach across harnesses**. There
is one spawn path, `scripts/spawn-confined.sh <harness>`, and every harness —
claude included — takes it. Across **platforms** the claim is narrower and is
stated as such in clause 5.

1. **Enforcement altitude is the KERNEL, not the CLI.** The worker's exec is
   wrapped in a bubblewrap jail (Linux) or `sandbox-exec` (macOS). On Linux
   `--ro-bind / /` makes the entire filesystem read-only, re-opened by explicit
   binds; on macOS there is no `--ro-bind` — the profile is `(allow default)` plus
   `(deny file-write*)`, so reads and exec are unrestricted and only writes are
   fenced. **The writable set is the fork's worktree AND the harness config dir**
   (`~/.claude` / `~/.pi`), plus a private `/tmp` and the device sinks — not the
   worktree alone (corrected SL-254). This is not cooperative — a worker cannot
   decline it, and it does not depend on the harness honouring a hook, a skill
   instruction, or a tool allowlist.
2. **Fail closed at spawn, with no rungs below.** On Linux a missing `bwrap` is a
   NAMED refusal (`jail.rs::REASON_NO_BWRAP`) and spawn does not proceed. On macOS
   **there is no backend-presence probe at all** — `have_bwrap` is
   `#[cfg(not(target_os = "macos"))]` with no Darwin sibling and
   `spawn-confined.sh:61` skips the probe — so a missing `sandbox-exec` fails
   closed but **unnamed**, at exec. There is no
   unconfined fallback and no "degraded harness" / reduced-enforcement-altitude
   rung — that tier is abolished (`DEC-208`). Because enforcement cannot be
   partially attained, there is no per-harness altitude left to state honestly:
   confinement is either established or spawn refuses. The macOS naming gap is a
   defect in the refusal's legibility, not a rung below the floor.
3. **Identity rides the same argv as the write floor.** The confinement prefix
   sets `DOCTRINE_WORKER=1` (Linux: `--setenv` in the spawn script's inline bwrap
   array; macOS: the trailing `env` token in `jail.rs::sandbox_exec_argv`); the
   worker-mode guard keys on `DOCTRINE_WORKER == "1"` and
   nothing else (`DEC-207`, REQ-192) — a value compare, so a bare-set non-`1`
   value is not worker mode. Identity therefore cannot be present
   without the confinement that carries it, which is what makes the two uniform
   together rather than separately.
4. **Base is pinned explicitly, on every harness.** Base-by-placement
   (`worktree.baseRef='head'` with the orchestrator's cwd in the coordination
   tree) was the in-session base story and is gone; the subprocess path forks
   explicitly from a named base.
5. **The floor is NOT byte-uniform across platforms** (added on correction,
   SL-254). It is uniform in intent and in write-fencing; the Linux and Darwin
   argv diverge on three axes, all verified in shipped code:
   - **Network.** `JailPolicy::network` defaults false and the Darwin arm calls
     `jail-prefix` without `--network`, so the Seatbelt profile emits
     `(deny network*)`. The **Linux inline bwrap array carries no
     `--unshare-net`** (`spawn-confined.sh:176-183`), so a Linux worker has full
     network. Consequence, unresolved and sharp: a macOS `claude -p` worker is
     network-denied and could not reach the API at all, so the Darwin claude arm
     as wired appears non-functional — a live question for SL-254 `PHASE-09`/`VA-2`'s
     Darwin census.
   - **Process lifetime.** `--die-with-parent` has no Seatbelt analog, so reap
     semantics differ.
   - **Read/exec surface.** Linux fences reads too (`--ro-bind / /`); macOS does
     not.
6. **One writable carve-out is disclosed, not hidden.** The harness config dir is
   bound read-write deliberately — claude needs it for the subscription credential
   (`DEC-210`), pi writes it at runtime — so a confined claude worker **can write
   the orchestrator's own `~/.claude` `settings.json`, hooks and agent
   definitions**. The narrowed mount set is a known, untaken refinement.

## Rationale

The original requirement's honesty clause — state each harness's reach rather
than silently assume uniformity — was the right discipline for a world with two
arms of genuinely different capability. SL-254 removes the need for it by
removing the difference: enforcement moved from a cooperative CLI/hook guard,
whose reach depended on what a given harness would honour, down to a kernel
boundary that no harness can decline. Honesty about non-uniform reach *across
harnesses* is superseded by uniformity, and the residual cooperative guard is
retained only to turn an opaque read-only-filesystem error into a named refusal.
The honesty discipline itself is **not** discharged: it now applies to the
platform axis (clause 5) and to the config-dir carve-out (clause 6), which is why
both are stated in the requirement rather than left to be discovered in the
spawn script.
