# EVD-017: A two-uid shared mirror re-opens the escalation the uid split closed

Observed 2026-08-11 while diagnosing why the spike guest's `origin` was dead.
Recorded because it falsifies a claim already banked in the spike's `NOTES.md`
item 11, and because it is `DEC-135`'s rule arriving from the **host** end rather
than the guest end.

## The immediate symptom

`capsule-gitd` is active and listening on `10.99.0.1:9418`; the guest connects
fine; the daemon refuses inside:

```
Request upload-pack for '/doctrine.git'
fatal: detected dubious ownership in repository at '/var/lib/capsule/doctrine.git'
```

```
/var/lib/capsule               capsule-git:capsule-git  2775
/var/lib/capsule/doctrine.git  david:capsule-git        2775
git version 2.55.0
```

The daemon runs as uid 972 `capsule-git`; the mirror is owned by `david`, because
`capsule-sync` creates it as the human. git 2.55 refuses to serve a repository
owned by another uid. **The unit path's git channel is broken on this host right
now** — item 11's "deployed and exercised" is stale. A one-line
`GIT_CONFIG_COUNT`/`safe.directory` `Environment=` on the unit restores service,
if service is wanted.

## The finding under the symptom

`safe.directory` exists because a repository you do not own can carry config and
hooks that execute **as you**. The shared mirror is precisely that repository:

```
hooks/   david:capsule-git  2775   <- daemon uid can create post-receive
config   david:capsule-git   664   <- daemon uid can set core.hooksPath, an alias, core.pager
```

`capsule-sync` and `just fetch` both run git as `david` in that repo. So a
compromised `receive-pack` — the exact precondition the uid split was built for —
writes `hooks/post-receive` or rewrites `config`, and the human's next sync
executes it as `david`.

Item 11's claim, *"the uid serving the mirror has no path to the tree the mirror
came from"*, is false. The path is the human's next `capsule-sync`.

## Why it generalises past this spike

The escalation is not a misconfiguration to be tightened; it is **forced by the
topology**. Guest-push requires two uids to share one repository, which requires
`core.sharedRepository=group` and 2775, which makes `config` and `hooks/`
writable by the untrusted side. Confining the daemon uid closes the forward
direction and opens the reverse one. Neither `safe.directory` nor tighter modes
dissolve that; they relocate it.

This is `DEC-135` in miniature — trusted code running git in a repository the
capsule's side can author — and it is the strongest argument available for
`DEC-192`. The inversion does not defend the precondition, it **deletes** it:
under host-initiated transport no repository is written by two uids anywhere, so
there is no `sharedRepository`, no setgid bit, no `capsule-git` group, no
`safe.directory` exception, and nothing for a compromised daemon to leave behind
— because there is no daemon.

## Limits

n = 1, one host (Sleipnir). This is **inspection plus one live daemon error**,
not an exploited path: nobody has demonstrated a compromised `receive-pack`
writing a hook and a subsequent sync executing it. The argument runs from the
permission bits and the two code paths that run git as `david` in that repo, both
of which were read rather than exercised. git 2.55.0's ownership check is what
surfaced it; an older git would have served the fetch and left the escalation
unremarked — so the *symptom* is version-dependent and the *finding* is not.

## Related

`DEC-192` (the decision this supports), `EVD-016` (the same session's transport
measurement), `DEC-135` (the execution-context rule this instantiates from the
host end), `RSK-231` (deletion over mitigation), `IMP-426` (the round).
