# ISS-222: Jail lint-js broken: eslint shebang needs /usr/bin/env, absent in bubblewrap jail

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`just gate` / `doctrine check gate` fail at lint-js inside the NixOS
bubblewrap jail: `web/map/node_modules/.bin/eslint` has a `/usr/bin/env`
shebang and `env` is absent in the jail — "bad interpreter", exit 126.
Pre-existing (surfaced during the SL-216 audit, RV-268 F-3); web/map
untouched by that slice. Memory: `mem.fact.env.jail-lint-js-bad-interpreter`.
Fix routes: provide `env`/node in the jail (flake.nix), or make lint-js
degrade gracefully when the node env is absent. Until fixed, the gate's js
leg needs a host-side run.
