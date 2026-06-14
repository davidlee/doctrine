# ISS-011: SL-056 SubagentStart hook merge keys identity on command only — stale matcher never healed on reinstall (fail-open unstamped worker)

Source: RV-016 finding F-13 (reconciliation review of SL-056), severity minor / follow-up.

## Detail

`src/boot.rs:658-696` — the SubagentStart hook merge keys ownership on the hook
**command** only (the Current-decision merge compares/owns on the command; `set_command`
rewrites only the command). If a `.claude/settings.local.json` already carries a
SubagentStart hook with the right command but a **stale/wrong matcher** (e.g. an old
agent-type literal), a `doctrine claude install` reinstall does NOT heal the matcher → the
stamp hook silently never fires for the dispatch-worker → **fail-open: an unstamped worker
writes freely** on the one harness with no env leg and no bwrap.

`mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher` notes the merge is
generalized over event+matcher; the ownership key should include the matcher.

## Fix

Key the merge identity on `(event, matcher, command)`, or reconcile the matcher on
reinstall so a stale matcher is healed.
