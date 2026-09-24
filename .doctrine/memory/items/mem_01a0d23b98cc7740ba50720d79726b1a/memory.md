`@earendil-works/pi-coding-agent`'s `ExtensionRunner.emit*` methods wrap each
handler invocation in `try { await handler(event, ctx) } catch (err) {
this.emitError(...) }` (see `dist/core/extensions/runner.js`,
`emitToolResult`). Consequences for an extension author:

- A handler that **throws or returns a rejected promise** does not fail the
  turn and does not change the tool result — pi logs an extension error and
  leaves the result as produced. So "fail-open" does not require guarding every
  intermediate call (`ctx.sessionManager`, `[...event.content]`, etc.) inside the
  handler body; the host absorbs a post-await throw.
- What the host does **not** catch is a Node process-level fault such as an
  `'error'` event on a stream with no listener. An early-exiting child makes
  `child.stdin.end(...)` raise unhandled EPIPE, which can kill the pi process —
  attach a no-op `child.stdin?.on('error', () => {})` before writing.

So: handler rejections are the host's problem; unhandled stream errors are yours.
`SL-263`'s generated `templates/surface.ts` gets this right (no-op listener), and
the concern that the un-guarded body might break fail-open is unfounded.

Decorated source-map line numbers: read `dist/**/*.js`, not the `.map` files.