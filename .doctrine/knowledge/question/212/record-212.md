# QUE-212: How does a result leave a shareless capsule?

## What the rule actually says

`DEC-135` chose Git bundle ingestion under a structural rule: *trusted
control-plane code never runs Git **in a capsule-authored repository**.* That is
about the repository used as **execution context** — git reads config, hooks and
`.gitattributes` filters from the repo it runs in, so using a capsule-written
repo as context is arbitrary code execution. It is **not** a claim that a bundle
is the only safe carrier.

`REQ-451`'s six bounds — path, symlink, quiescence, time, byte, object — are the
tamed form of a specific hazard: under bubblewrap the natural channel was **a
file appearing in a shared location**, which is racy and path-traversable. The
VM has no shares, so most of that hazard class is absent rather than defended.

## The actual defect in the spike's transport: direction, not format

The spike has the guest **push** to a host git-daemon. Consequences:

- the host runs `receive-pack` — a hostile-input-parsing daemon — as a live
  service on a port the guest can reach;
- the mirror is a guest-written repository, which the owner then fetches from;
- the ref-restriction update hook, the `capsule-git` group and a mirror-sync uid
  all exist to confine that arrangement.

Inverting the direction dissolves it. **Host-initiated `git fetch
ssh://guest/…` into a fresh quarantine repository** keeps hostile code
(`upload-pack`) guest-side, runs the host's git in a **host-authored** quarantine
repo, and satisfies `DEC-135`'s rule without a bundle. The spike already has
host→guest ssh, so the channel exists.

Owner's observation, 2026-08-11, and it corrects this record's first draft, which
had wrongly treated "bundle" as the essential thing rather than "do not use a
capsule repo as execution context".

## What survives the correction

1. **`index-pack` still parses hostile bytes.** True of every option including
   bundles, so it is not a new risk — but it is not zero. `transfer.fsckObjects`
   and a size ceiling are the answer. Note `fetch` has no `receive.maxInputSize`
   analogue, so the ceiling must come from elsewhere (VM disk size, ref
   restriction, `--depth`); price it rather than assume it.
2. **A fetch leaves no artifact.** `DEC-133` separates the durable admission
   journal from short-horizon forensic exhibits, and a live fetch produces
   nothing to hash, retain, or deterministically re-ingest. Host-side
   `git bundle create` after the fetch, or retaining the quarantine repo, both
   answer it. This is a **retention** question, not a security one, and it is the
   only remaining reason to want a file.
3. **`REQ-451`/`REQ-452` still need rewording**, because their subject changes.
   Not because the discipline was wrong — because its hazard model was written
   for a shared-filesystem channel.

## The consequence worth more than the fix

If the result leaves by host-initiated fetch, and provisioning enters by
host-initiated push over the same ssh channel, then **the guest never initiates a
connection to the host at all** and git-daemon leaves the perimeter entirely.
`README.md`'s *"the only two ports the guest may reach"* becomes one — the proxy.
`receive-pack`, the `refs/heads/capsule/*` update hook, the `capsule-git` group
and the mirror-sync uid stop being confined and start being **absent**.

That is deletion rather than mitigation, which is the shape `RSK-231` asked for
and the thing this round should confirm first.

## Open, and what to price

- Does host-initiated fetch actually work against a guest under the netns design,
  and what does it cost?
- What bounds the fetch, given no `receive.maxInputSize`?
- Does anything require a snapshottable artifact — `DEC-133` retention,
  deterministic replay, `FR-006` conformance re-derivation — or does a retained
  quarantine repo serve?
- Can provisioning invert too, and does git-daemon then actually go?

## Related

`DEC-135`, `DEC-133`, `REQ-451`, `REQ-452`, `IMP-426`, `RFC-025`,
`red-team.md` `RT-1`, `ADR-020` (*"no trusting capsule-controlled Git
configuration"* — the same rule stated at architecture altitude), `CPT-002`.
