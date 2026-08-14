# REQ-335: A confined-orchestrator altitude exists as a harness-neutral tier: an orchestrator running inside the coordination worktree under a cwd-confining jail with the shared object store read-only lands every worker delta and coordination write through a gated write-funnel of mediated tools (import -> conclude -> reap), never a direct coordination .git write; it reads authored/runtime state raw but performs all mutation through the funnel (reads-raw/writes-mediated), forks and confines nested workers depth-agnostically, and — because trunk-facing verbs (refresh-base/candidate/integrate) write outside the jail — reports-and-halts them to the delegating parent rather than performing them; the funnel tools route by their declared args alone, so the tier is a property of the mediated-write contract, not of any single harness.

## Statement

> **AMENDED — FALSIFIED IN WHOLE (SL-254, 2026-08-14).** The tier this
> requirement specifies is **Mode B** — the confined in-session orchestrator role
> — and it is retired entirely (`DEC-217`). Its entry point is gone, not merely
> one of its tools: `install/hymns/role/orchestrator.md`,
> `install/workflows/drive-slice.js`,
> `install/agents/claude/dispatch-orchestrator.md` and
> `install/agents/claude/dispatch-probe.md` were deleted whole (PHASE-10, `F-7`),
> and the `worker_commit` MCP tool that its write-funnel depended on was deleted
> with the claude arm (`DEC-204`).
>
> **This requirement is NOT retired, and stays `pending`.** It never reached
> `active` — it is forward intent, and `SL-254`'s locked design (`§6`, `OQ-2`)
> states explicitly that this tier *"stays `pending` as a contract"*. `DEC-217`
> retires Mode B's **shipped entry point**, which is a statement about what runs
> today, not a decision that the tier will never be built. Retiring the
> requirement would assert the latter, and that is a governance call beyond the
> `DEC-2xx` set `SL-254` minted. So what this amendment records is that nothing
> implements the tier today and the mechanisms it names are currently
> unreachable — not that the contract is withdrawn. `REV-052` therefore carries
> `REQ-335` as a `modify`, not a `status` row.

What replaced it: the orchestrator is UNCONFINED and runs on the coordination
worktree; confinement applies to WORKERS, one kernel-level jail per spawned
subprocess (REQ-291 as amended). The mediated-write funnel that this requirement
proposed as a *confinement affordance* survives on its own merits as the
orchestrator's ordinary landing path — `import → verify → conclude → reap` over
`funnel_machine` (REQ-384, REQ-387) — but it mediates the ORCHESTRATOR's writes
for auditability, not because a jail forbids it a direct `.git` write.

Consequently the specific mechanisms named above are not merely unimplemented but
unreachable as stated: there is no jail around the orchestrator, no read-only
coordination object store, no depth-agnostic nested worker forking from inside a
confined orchestrator, and no report-and-halt boundary for trunk-facing verbs
(`refresh-base` / `candidate` / `integrate`) — the orchestrator simply performs
them.

## Rationale

The confined-orchestrator altitude was designed for a world in which the
orchestrator itself might be an untrusted in-session agent, and the write-funnel
was the price of letting it act at all. SL-254's answer is different in kind:
confine the WORKER at the kernel and leave the orchestrator — a trusted,
human-supervised session — outside the jail. The tier is not deferred pending
implementation; the premise that motivated it has been decided against.
