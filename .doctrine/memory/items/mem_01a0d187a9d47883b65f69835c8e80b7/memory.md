Node's asynchronous `child_process.execFile` and `spawn` accept **no `input`
option** — only the `*Sync` variants honour it — and passing one is silently
ignored (no error). The child then reads nothing, and if its stdin pipe is left
open it blocks until the caller's timeout.

Verified in this repo's node (26.9.0): `execFile("cat", [], {input: "HELLO"}, cb)`
never delivers stdin and never fires the callback; the same call with
`child.stdin.end("HELLO")` returns `HELLO`.

Consequence for a generated subprocess adapter (e.g. `templates/surface.ts`):
write the payload with `child.stdin.end(json)` (or `spawn` with
`stdio: ["pipe", ...]`), and keep the timeout as the backstop — a strictly
fail-open wrapper otherwise swallows the silent-empty-stdin failure, so the
adapter "works" and surfaces nothing.
