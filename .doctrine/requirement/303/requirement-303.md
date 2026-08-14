# REQ-303: Harness-agnosticism: the isolation+funnel guarantee holds across orchestrating harnesses with no harness-specific command required as a mandatory element, and each harness's enforcement altitude is stated honestly rather than silently assumed uniform.

## Statement

> **AMENDED — FALSIFIED (SL-254, 2026-08-14).** Both clauses of the title line
> are now false, in opposite directions. (a) A harness-specific command IS now a
> required element of the shipped path: `scripts/spawn-confined.sh` execs
> `claude -p --output-format stream-json …` for the claude harness. (b) Each
> harness's enforcement altitude is no longer "stated honestly rather than
> assumed uniform" — it is genuinely uniform, because enforcement moved to the
> kernel and spawn fails closed without it. The title is retained unchanged; the
> statement below is what must hold.

Harness-agnosticism holds at the level of the **guarantee**, not at the level of
the **invocation**.

1. **The guarantee is uniform.** The isolation+funnel guarantee — a private
   worktree per unit of work, the coordination/runtime tier absent by
   construction, and every delta landing through the orchestrator's funnel —
   holds identically for every orchestrated **harness**, and is enforced
   identically across harnesses: a kernel-level bwrap/`sandbox-exec` jail no
   harness can decline (write-fenced by `--ro-bind / /` on Linux and by an SBPL
   `(deny file-write*)` floor on macOS, its writable set being the fork worktree
   **plus the harness config dir**), plus a fail-closed refusal when that jail
   cannot be established — *named* on Linux (`bwrap-unavailable`), unnamed on
   macOS, which runs no backend-presence probe. No harness enjoys a weaker
   altitude, and none is trusted to cooperate. **Uniformity is a claim about
   harnesses, not platforms** (corrected SL-254): the Linux and Darwin argv differ
   on network and on process lifetime — see REQ-291 clause 5.
2. **The invocation is harness-specific, and that is the accepted cost.** One
   spawn script, `scripts/spawn-confined.sh <harness>`, owns the per-harness
   argv (headless flags, output format) behind a single uniform interface. Adding
   a harness means adding one exec line there and nothing else: no new arm, no
   new skill, no new routing predicate, no change to the guarantee. Agnosticism
   is therefore a property of the CONTRACT and the surrounding machinery, not a
   claim that the platform never names a harness's binary.
3. **No harness-specific SPAWN MECHANISM is admitted.** The prohibition that
   survives is against a harness's own framework primitive — an in-session agent
   tool, a bespoke hook wall, a harness-native worktree isolation — standing in
   for the confined subprocess path. Those were what made altitude non-uniform,
   and they are deleted.

## Rationale

The original clause conflated two things: needing a harness's *name* in one argv
table, and needing a harness's *framework* to supply the isolation. The first is
unavoidable — a headless spawn must know what to exec — and cheap, being one line
in one script. The second was expensive, because it made the guarantee's strength
a function of what each harness would honour, which is exactly the non-uniformity
the honesty clause existed to disclose. SL-254 pays the first cost to abolish the
second: naming `claude -p` in a script buys a guarantee that no longer has to be
qualified per harness.
