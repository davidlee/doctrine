A generated pi extension is TypeScript (`templates/*.ts`, installed to
`.pi/extensions/doctrine/`). To drive it BEHAVIOURALLY in a Rust test (no
grepping the source), run it under `node` directly:

- Node 26 strips TS types natively, so `import("./surface.ts")` works with no
  loader — but module type follows the nearest `package.json`. A temp dir with
  no `package.json` makes a `.ts` default to CommonJS, and the ESM
  `import`/`export` in the generated file then fails. Write
  `{"type":"module"}` at the temp root first.
- Keep the template's pi import as `import type { ExtensionAPI } from
  "@earendil-works/pi-coding-agent"`: type-only imports are ERASED, so node
  never resolves the package (which is not installed in the test env).
- The driver (a `.mjs`) fakes the host: `const handlers = {}; await
  mod.default({ on: (n, f) => { handlers[n] = f; } });` then invoke
  `handlers["tool_result"](event, ctx)` and print `JSON.stringify(result ??
  null)`.
- Stub the child binary as an executable `/bin/sh` script that captures its
  stdin (`cat > "$0.stdin"`) — proving the envelope reached the child — and
  echoes a block. A second stub that `exit 0`s immediately exercises the EPIPE
  path: the handler must return `undefined` and node must exit 0 (a missing
  no-op `child.stdin.on("error")` would crash it).

SL-263 PHASE-04 uses this for `surface_extension_delivers_to_doctrine` /
`surface_extension_survives_early_exit` in `src/boot.rs`.
