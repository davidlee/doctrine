# bwrap 0.11.2, measured (SL-248 PHASE-08, NixOS, kernel via /proc)

Payload in every case: `sh -c 'echo LIVE; (sleep N </dev/null >/dev/null 2>&1 &); sleep 0.2'`
— a **detached** grandchild. The redirections matter: without them the
grandchild holds the harness's pipe open and the measurement reads as
"no survivor" for the wrong reason. That cost two probe cycles.

| profile | capsule sid (host view) | detached grandchild survives? |
|---|---|---|
| `--unshare-all --die-with-parent --new-session` (the confining profile) | 1 | **no** |
| `--unshare-all --new-session` (the `Teardown` control) | 1 | **yes**, in a foreign session |
| enumerated non-pid unshare set `+ --die-with-parent` (the `ProcessVisibility` control) | host-range sid | **yes** |
| enumerated, no `--die-with-parent` | host-range sid | **yes** |

## The counter-intuitive result

A pid namespace does **not** guarantee teardown. Row 2 has `--unshare-pid`
(via `--unshare-all`) and the grandchild still outlives the run. So
`--die-with-parent` is the load-bearing flag for containment, and a harness
that reasons "the pid namespace will clean up after me" leaks a process per
run — silently, found by the developer whose machine fills up.

Consequence: the reap must be **by session**, and the session is discoverable
from the host because `/proc/<pid>/stat` field 6 reports the sid in the
*reader's* namespace. A survivor inside a capsule pid namespace still shows a
host-range pid and a host-range sid to a host-side reader.

## Two more measured facts from the same session

- `--cap-add ALL` is accepted and effective: `CapEff` goes from
  `0000000000000000` (baseline, `--unshare-all`) to `000001ffffffffff`.
- The enumerated non-pid set — `--unshare-user-try --unshare-ipc
  --unshare-net --unshare-uts --unshare-cgroup-try` — is accepted, and with
  `--proc /proc` the capsule sees host pids (20 entries vs 4 under
  `--unshare-all`). Bare `--unshare-user` also works on this host, but the
  `-try` forms are what `--unshare-all` itself uses.
- `--share-net` **does** combine with the enumerated set on 0.11.2 (net
  interface count 4, same as the host, vs 1 under `--unshare-net`). The
  documented "only with `--unshare-all`" restriction is not enforced by this
  version. Omitting `--unshare-net` is the equivalent and does not rely on
  that leniency.

## Absent capability on this host

`/proc/<pid>/task/<tid>/children` does **not** exist (`CONFIG_PROC_CHILDREN`
off). Descendant discovery must walk `/proc/*/stat` field 4 (ppid) instead of
reading the children file.


## Refinement measured at execution (SL-248 PHASE-08, `F-30`)

The table above stands: the detached grandchild *does* outlive its parents
under a `Teardown` control. What the probe never asked is whether a harness
can **observe** that while it happens. It cannot.

The escapee outlives its parents **inside the pid namespace**, and that
namespace's init process holds the harness's captured stdout/stderr
descriptors until the namespace empties. So the trusted-side arm — which
reads the capture to end-of-file — cannot return while the escapee is alive:

| control removed | end-of-file on the capture | escapee at that moment |
|---|---|---|
| `Teardown` (pid namespace still present) | waits out the escapee; a long `sleep` payload takes the arm to its wall bound | killed with the tree by `timeout` |
| `ProcessVisibility` (no pid namespace) | 25ms | alive, in a session of its own |

**Consequence for anyone testing containment:** the escape you can watch is
the *visibility* control's, not the teardown control's. A test that tries to
catch a teardown escapee trusted-side either hangs to the wall bound or reads
"no survivor" for a reason that has nothing to do with teardown.

Two adjacent traps, both paid for in the same session:

- The descent to a capsule's session **leader** needs a live leader. A payload
  that echoes and exits is gone before the discovery poll runs; its orphan
  keeps the session id but leads nothing, and the descent correctly answers
  `None`. Make the payload linger a second after spawning the escapee.
- `SIGKILL` is asynchronous and a `/proc` entry is not a process. Re-reading
  the process table immediately after a sweep finds the survivor it just
  killed. Poll for the session to drain rather than asserting once.

## Re-measured against the session-escaping payload (SL-248 PHASE-09 `T2`)

`EVD-013`'s 2×2 was run against a payload that merely orphans a descendant.
`RV-346` `F-27` strengthened it to one that calls `setsid` first. Re-run, the
table is unchanged: **both** the pid namespace and `--die-with-parent` are
required, and `--die-with-parent` is what reaps. A descendant that leaves the
session is no harder to reap than a plain orphan, because `setsid` changes a
session and the reap works through the namespace.

`setsid` is still absent from this jail's PATH. `python3` is present and
`os.setsid()` is the same syscall. A second trap sits next to that one:
coreutils ships `sleep` as a **multi-call symlink that dispatches on argv[0]**,
so an `execv` that puts a search marker in argv[0] makes the escapee exit
instantly with "unknown program" — every cell then reports a false negative,
which is `EVD-013`'s failure mode wearing a different hat. Let the payload sleep
in python and put the marker in the source.

### What actually hides the escapee is the capture pipe, not the namespace

The `F-30` refinement above can be sharpened, and the sharpening matters. Run
the `Teardown`-removed cell twice, changing only where the arm's stdout goes:

| arm's stdout | bwrap exits after | escapee visible trusted-side afterwards |
|---|---|---|
| a **pipe** the parent reads to EOF | the escapee's whole lifetime | no — waited out |
| a **file** | ~0s | **yes** — the namespace init and the escapee, in a session of their own |

So `bwrap` itself returns promptly either way. What waits is end-of-file on the
capture, because the namespace's init inherits that descriptor and holds it
until the namespace empties. A harness that captures output through a pipe
therefore cannot see a teardown escapee **whatever** it asserts afterwards, and
a harness that does not capture output can see it immediately.

That is worth knowing before designing around it: the constraint is the
capture, which is a choice, not the namespace, which is not.
