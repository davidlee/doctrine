# ISS-526: The darwin devshell does not evaluate: jailLib.agentsByName read unguarded

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`nix eval --raw .#devShells.aarch64-darwin.default.drvPath` fails with
`error: attribute 'agentsByName' missing`. Same result at `12a1944e1` (before
`0d53c29bb`), so it predates the jail shell-pin cleanup. Observed by eval from
x86_64-linux; not reproduced on a mac.

## Cause (inferred from source)

`flake.nix`, perSystem let block: `jailLib` is `{}` off Linux, but

```nix
claude = jailLib.agentsByName.claude;
codex = jailLib.agentsByName.codex;
```

are read unconditionally, and `projectPkgs` (consumed by the devshell on every
system) lists `(jailLib.unjailed.codex or codex)` and `claude`. Forcing the
darwin devshell forces those attributes on an empty set.

## Likely fix

Guard both reads (`lib.optionals isLinux`, or `or` fallbacks to the unjailed
packages from `inputs.agents.packages.${system}`, if it publishes darwin), and
check with the eval above. ISS-345 is the related gap: the darwin guard can't
run in the jail, which is how this went unnoticed.

## Resolution

Fixed in `195e869c0`. Cause as inferred, plus a second blocker: `iproute2` has
no darwin build.

- `jailLib` now evaluates on every system. The agents flake's
  `mkJailedAgents {}` exposes `agentsByName`/`unjailed` (claude, codex, pi) on
  darwin, though not under `packages`; only the jail builders are Linux-only,
  and they were already behind `isLinux`.
- The conformance-suite deps split out as `conformancePkgs`, Linux-only.

Evidence: darwin devshell drv evaluated from x86_64-linux; the Linux devshell
drv stayed byte-identical to the prior HEAD; v0.46.8 built on darwin in GitHub
Actions (user report).
