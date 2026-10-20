# IMP-519: Export webModules so a sealed room runs web-build without fetching

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised 2026-10-19 from oubliette. In oubliette's `assistant` room,
`session doctrine -- just web-build` fails with `DNSResolveFailed`: `web-install`
runs `bun install`, and a room has no egress. That is by design. Oubliette's
policy `POL-005` ("A room never fetches", in oubliette's corpus, not this one)
says a room repo's language-package dependencies are its flake's output, fixed
by its lockfile, and the repo puts them into its own tree.

Most of it exists. `flake.nix`'s `webModules` is a fixed-output derivation over
`web/map/bun.lock`, and `just web-hash` regenerates its hash. What's missing:

1. **Export it** where the room reaches it: in `packages.dev-tools`, or as a
   small script there (e.g. `doctrine-web-deps`) that places the store path's
   `node_modules` into `web/map`.
2. **`web-install` uses it when present,** and falls back to `bun install`
   otherwise (host devshell, slot). It must not run `bun install` after placing
   the tree, or it fetches again. Whether `bun install --frozen-lockfile` stays
   offline over a complete tree is unchecked.
3. **Refuse on drift.** Embed a fingerprint of every input that determines the
   tree (`package.json` and `bun.lock` at least, plus any `bunfig.toml` or
   patches) in the output. Refuse when the working tree's inputs differ,
   rather than build against a stale tree. A lockfile hash alone misses a
   `package.json` edit made without relocking, and no install runs in the room
   to catch it. Verify that negative case.
4. **Link or copy.** Vite writes `node_modules/.vite`, so a symlink into the
   store may break it. `webDist` copies (`cp -R`, `chmod -R u+w`); set vite's
   `cacheDir` elsewhere, or copy. Unverified.
5. **Native binaries.** `webModules` sets `dontFixup`. esbuild is static and
   `.node` addons load through nix's node, but a standalone dynamically linked
   binary needs `autoPatchelfHook` or a loader the room does not have yet
   (oubliette `IMP-021`).
