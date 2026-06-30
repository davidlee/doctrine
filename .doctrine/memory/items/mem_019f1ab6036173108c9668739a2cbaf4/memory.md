# Seatbelt write-floor: SBPL rule ordering + device/temp surface (RSK-014 H2)

The macOS Seatbelt (`sandbox-exec`) write-floor arm (SL-183 / IMP-045), proven on
macOS 26.4.1. Sibling of the bwrap arm [[mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement]].
Apparatus + evidence: `.doctrine/backlog/risk/014/probe-h2-seatbelt/`.

## The model (inverse of bwrap)
bwrap hides via a mount namespace; Seatbelt fences *operations* over an unchanged
fs. Parity = **allow-default-deny-write-except**, NOT SBPL default-deny:
```scheme
(version 1)(allow default)(deny file-write*)
(allow file-write* (subpath (param "WT")))   ; realpath'd -D params
```
Invoke opaquely (base64 body), realpath every `-D` param — `subpath` matches the
RESOLVED path and macOS aliases `/tmp`→`/private/tmp`, `/var`→`/private/var`.

## F-A — SBPL is LAST-MATCH-WINS (the load-bearing footgun)
macOS temp worktrees live UNDER `/private/tmp`. If you emit `(deny file-write*
(subpath PTMP))` *after* the WT allow, the deny is the last match for any path
under `/private/tmp` — **including the worktree itself** → the floor denies in-wt
writes. **Rule: emit deny-coarse-FIRST, allow-specific-LAST** so the narrower
worktree/extra_rw allow wins. Verified: same profile flips in-wt write
ALLOWED↔BLOCKED purely on rule order.

## F-B — the floor denies the device write surface → breaks tooling
`(deny file-write*)` denies `/dev/null`, `/dev/stdout`, `/dev/stderr`,
`/dev/tty*`, `/dev/fd`, `/dev/dtracehelper` → broke `python3` and any `>/dev/null`.
Re-allow the device sinks as a constant allow-set (after the deny, before/among
the specific allows — they're literals/regex, order-independent of WT).

## F-E — `/var/folders/$USER/T` is a SECOND temp surface
macOS per-user temp (`DARWIN_USER_TEMP_DIR`, the `$TMPDIR` default) is **distinct
from `/tmp`**. `xcrun`/Xcode-shim `python3` hardcodes an `xcrun_db` cache there;
the `TMPDIR=<wt>/.tmp` redirect (D-mac3) does NOT cover it → denied, noisy.
Cosmetic for python but would break cache-dependent tools. Decide in design:
redirect/allow `/var/folders/$USER/T` too.

## Containment results (orchestrator context, pass 1)
Every escape vector BLOCKED: absolute, `../` traversal, symlink-deref, **hardlink**
(`ln` to outside target denied — Seatbelt resolves the link target), shared-`.git`,
`/tmp` alias, `$HOME`, python child, `nohup &`/`setsid` detached. **`launchctl
submit` IPC residual is DENIED by Seatbelt default** (rc=1, no launchd job) — the
brief §5 feared this as open; it's empirically contained (control: rc=0 works
unsandboxed). `at` denied too.

## Discipline + scope caveat
- **Never trust a vector's self-report** — an in-wt echo after a denied `ln` reads
  as "WROTE" but didn't touch the canary. The independent checksum verifier is
  truth (RSK-014 idiom).
- Permission-mode (auto/ask) is NOT a write confound in the orchestrator context:
  a bare unsandboxed write succeeds under the gate (gate = tool-invocation, not
  syscall). **Subagent context untested** — M1-sub (nesting inside a real
  `isolation:worktree` subagent, where Claude's own Seatbelt is active) is pass 2,
  to run under both permission modes. Orchestrator composition ✓ is the weaker claim.
