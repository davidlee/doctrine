# web/map buildHash is already '#'-prefixed; do not prepend '#'

web/map: `buildHash(view, id, depth)` (`router.ts`) returns a string that already
starts with `#/` (e.g. `#/focus/SL-003`). `setFocus` uses it directly
(`window.location.hash = buildHash(...)`) and `parseHash` expects exactly that
shape (`hash.slice(1)` then `^/focus|^/edge`).

**Trap:** prepending `'#'` to it — `href = '#' + buildHash(...)` — yields
`##/focus/…`. The browser stores the double hash; `parseHash` then sees
`#/focus/…` after `slice(1)`, matches nothing, and returns `{id:null}`, silently
clearing focus. Symptom: clicking a link does nothing useful / the relationship
table empties, while graph/entity-list clicks (which go through `setFocus`) work
— masking the bug.

**Rule:** assign `buildHash(...)` directly to `href` / `location.hash`; never
prepend `'#'`. RV-098 F-6 fixed five such sites in `render.ts`; the latent bug had
shipped since SL-091. A link→`parseHash` round-trip test (`render.test.ts`) now
guards it.
