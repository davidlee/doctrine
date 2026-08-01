## The footgun

Any git operation that writes a reflog needs a committer identity. With none
configured, git **guesses** `user@hostname` — which means resolving the
hostname. Inside a `bwrap --unshare-all --share-net` sandbox the UTS namespace
is unshared but the network is not, so that lookup is a DNS query for a name no
resolver can answer, and it **blocks until the resolver gives up**.

Measured on the SL-241 capsule rig, one `git clone` of a 24-file fixture:

| identity | network | wall clock |
|---|---|---|
| unset | shared | **3905 ms** |
| set | shared | 40 ms |
| unset | `--no-net` | 35 ms |

Every capsule paid it twice (worker + verify). The visible symptom was
"provisioning takes 7.7 s", which reads exactly like *sandboxes are slow* — the
cost is per-git-op, scales with the number of operations rather than the size of
the repo, and is invisible to any profiler that only times the whole run.

## The fix

Pin the identity **on the clone**, not after it:

```
git clone -c user.name=… -c user.email=… --no-hardlinks -- <src> <dst>
```

`-c` is effective for the clone's own fetch and reflog writes and persists into
the new repo's config. A post-clone `git config` is too late — the clone has
already paid. Assert it persisted; a silent revert to the post-clone form
restores the whole tax.

## Generalises

Two things: (1) a sandbox that shares the network but not the UTS namespace
turns every unresolvable-name lookup into a timeout, not an error — suspect it
whenever a confined operation is slow in multiples of the resolver timeout;
(2) an append-only results file written **across** a fix like this must be
banner-stamped with the rig's state, or the pre-fix rows get quoted later as if
they measured the same thing.
