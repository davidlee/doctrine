# DEC-210: Bind the harness config dir wholesale, defer narrowing

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->


## Correction at reconcile (2026-08-14, `RV-356` `F-9`)

This decision binds `$HOME/.claude` wholesale specifically to carry the
subscription credential into the jail, and design §5.2.1/§5.5 inherit its framing
as though that bind is the whole of the authorisation story. `PHASE-09`'s live
fire falsified it.

**The bind carries a credential; it does not create one.** In the capsule there is
no `~/.claude/.credentials.json` at all — the session authenticates as a child
session over a unix socket — so a headless `claude -p` reports `Not logged in`,
`apiKeySource: "none"`, `duration_api_ms: 0`. That was run #2 of five, and it was
verified **not** to be a confinement defect: `claude -p` fails identically
unconfined on the same host.

**The unstated host precondition**, therefore, is one of:

- a materialised `~/.claude/.credentials.json` on the host, or
- `CLAUDE_CODE_OAUTH_TOKEN` in the environment the spawn inherits.

The live fire was bridged with the second — the subscription OAuth token, *not* an
API key, so this decision's billing posture and `EVD-023`'s authorisation are both
preserved. Deliberately **not** written into the shipped script: over-fitting a
shipped spawn path to one capsule's auth shape is worse than recording the
dependency. Recording it here is that.

Without this, the next operator on a host with no materialised credential meets a
`Not logged in` from inside a jail and has no reason to suspect the host rather
than the confinement — which is exactly the diagnosis cost `PHASE-09` paid.
