# IMP-045: macOS sandboxing via Seatbelt/sandbox-exec with bwrap fallback (cross-platform jail seam)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced from SL-056 (dispatch worker sandboxing). Today's jail seam assumes
Linux bubblewrap (`bwrap`) — no isolation story on macOS, where `bwrap` does not
exist.

Approach: a `safe_exec`-style sandbox seam that selects a platform backend —
macOS Seatbelt (`sandbox-exec` + a generated `.sb` profile) with `bwrap` as the
Linux backend / fallback. Profile generation (allow/deny filesystem + network
scopes) is the shared abstraction; the per-OS launcher is the thin shell.

Reference prior art: Anthropic's sandbox-runtime macOS Seatbelt utilities —
https://github.com/anthropic-experimental/sandbox-runtime/blob/main/src/sandbox/macos-sandbox-utils.ts
(profile-string assembly, path allowlisting, sandbox-exec invocation).

Related: IMP-004 (jail dispatch isolation spike — per-worktree target + bwrap
confinement) is the Linux-side sibling; this item generalises the seam to a
cross-platform backend split rather than assuming bwrap.
