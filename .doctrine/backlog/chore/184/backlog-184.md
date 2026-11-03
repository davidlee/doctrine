# CHR-184: Doctrine as a dev-envs producer (flakes:SL-023)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Doctrine's side of flakes:SL-023 (layered dev-env consumer, carried to
Doctrine; recorded here per flakes:DEC-119). Every Doctrine change that slice
makes cites this chore.

- PHASE-04 (additive, flakes:DEC-111/DEC-112/DEC-114): `dev-tools` gains
  `passthru.devEnv` (`LD_LIBRARY_PATH`; `hostEnv = ["DOCTRINE_BIN"]`) so the
  `~/flakes/dev-envs` consumer can apply it; `bubblewrap` joins
  `conformancePkgs` (capsule-verify and the live-bwrap rows need it); new
  `capsule-tools` = `dev-tools` + the crane `doctrine`, for the fleet guest.
  The fleet's doctrine target switches to `capsule-tools`; then `dev-tools`
  drops the crane `doctrine` (a consumer would otherwise rebuild the product per
  checkout snapshot).
- Later (flakes:SL-023 PHASE-06): the flake's own jails, forms, `agents` and
  `quotient` inputs and personal devshell commands go; `.envrc` uses
  `use_dev_envs`; `justfile` loses `quotient-forms`.
