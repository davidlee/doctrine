# SL-182 per-arming policy: network is a bool, not an enum

SL-182's per-arming jail policy (`<main>/.doctrine/state/dispatch/jail/<worktree-name>.toml`,
keyed by worktree name) carries exactly two fields: `extra_rw` (paths) and
`network` — a **bool**. `network = true` is the default (open); `network = false`
means deny (Linux/bwrap: `--unshare-net`; macOS/Seatbelt SL-183: emit `(deny
network*)`). `extra_ro` and a `strict/loose` mode were dropped (SL-182 D6).

**Why:** SL-183's design (D-mac4 / RV-203 F-6) briefly over-specified this as a
"closed enum" and required enum-based ambiguity handling. That was wrong — a bool is
already a closed 2-value domain, and widening it to an enum would be a schema
refactor of SL-182, which OQ-mac3 forbade (SL-183 slots into SL-182's seam as-is, no
SL-182 refactor).

**How to apply:** any SL-183 (or later cross-platform-arm) work reuses `network`
as-is. Fail-closed-on-malformed-policy is a policy-LOAD concern (the arm denies the
whole Bash on an unreadable/malformed policy), NOT a reason to change the field
shape. Don't reach for an enum. See [[mem_019f1a5ceef674e3aa8e48012e8b216f]]
(per-arming single-slot granularity).
