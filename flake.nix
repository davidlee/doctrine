{
  description = "doctrine: dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    agents.url = "github:davidlee/nix-config?dir=flakes/agents";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs @ {
    flake-parts,
    rust-overlay,
    crane,
    nixpkgs,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devshell.flakeModule
      ];

      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      flake = {
        homeManagerModules.default = import ./nix/module.nix;
        homeManagerModules.satan-attrd = import ./nix/module.nix;
      };

      perSystem = {system, ...}: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };

        inherit (pkgs) lib;

        # THE toolchain. Every consumer takes this one binding: the devshell,
        # `just lint`/`gate`, and the crane build below. Beta, not nightly — and
        # release.yml's `dtolnay/rust-toolchain@beta` is a fourth consumer that
        # must track this channel by hand (see its comment; a channel split broke
        # v0.11.0..v0.17.1). Extensions are devshell conveniences and do not
        # affect codegen, so crane sharing this derivation is free.
        rust = pkgs.rust-bin.beta.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "rust-docs"
          ];
        };

        # NOT stdenvAdapters.useMoldLinker: it injects -fuse-ld=mold through
        # mkDerivationFromStdenv, i.e. only into derivations built with it — here
        # just webModules/webDist (bun/vite, nothing native to link). It never
        # reaches crane (craneLib is built over plain pkgs) nor a devshell
        # `cargo build`. mold arrives via CARGO_BUILD_RUSTFLAGS in .envrc instead.
        # It also throws outright on darwin ("Mold can't be used to emit Mach-O"),
        # and this stdenv is forced on every darwin eval path.
        inherit (pkgs) stdenv;
        isLinux = stdenv.hostPlatform.isLinux;

        # Evaluates on every system: agentsByName/unjailed (the bare agent
        # CLIs) exist on darwin too; only the jail builders are Linux-only,
        # and those are read solely under isLinux guards (ISS-526).
        jailLib = inputs.agents.lib.${system}.mkJailedAgents {};

        # The tool set minus the jailed agents: what a doctrine working
        # environment needs, independent of how it is confined. Exported as
        # `packages.dev-tools` for consumers that jail differently — the
        # microvm capsule runs the agent inside a VM, where a bwrap wrapper
        # binding *host* paths is meaningless.
        devToolPkgs = commonToolPkgs ++ conformancePkgs;

        commonToolPkgs = with pkgs; [
          jujutsu
          jjui
          just
          rust # the single toolchain, incl. rust-analyzer
          mold # must be on PATH for .envrc's -C link-arg=-fuse-ld=mold
          helix
          cargo-edit # `cargo set-version` for the release recipe
          # tokei

          stdenv.cc # cc/ld on PATH (linker for cargo build)
          stdenv.cc.cc.lib

          nodejs_latest
          eslint
          bun
          typescript
          typescript-language-server
          sccache

          graphviz
          d2
          mermaid-cli
          uv
          python314

          shellcheck
          procps # pgrep
        ];

        # Conformance-suite payload dependencies, declared rather than
        # inherited: an absent binary makes a row's escape silently fail to
        # launch, and the arm then reads as a hold. That is `EVD-013`'s
        # false-negative-through-absent-binary, and row 7 was sitting in it.
        # Linux only, like the live-bwrap rows they serve (iproute2 has no
        # darwin build; ISS-526).
        conformancePkgs = lib.optionals isLinux (with pkgs; [
          util-linux # setsid — row 7's session escape
          socat # row 5 — present transitively before this, undeclared
          tinyproxy
          iproute2
        ]);

        projectPkgs =
          devToolPkgs
          ++ (with jailLib.unjailed; [
            codex
            claude
          ]);

        # No provider key enters these jails: mkJail defaults to the LLM
        # gateway (`gatewayClient "pi"`), and the jail gets only the gateway
        # key, on FD 22. A jail with `gateway = null` falls back to
        # jailed-agents.nix's own key forwarding; don't duplicate it here.
        jailEnvOptions = with jailLib.combinators; [
          (try-fwd-env "DOCTRINE_BIN")
          (set-env "LD_LIBRARY_PATH" "${lib.makeLibraryPath [pkgs.stdenv.cc.cc.lib]}")
          # No CLAUDE_CODE_SHELL or bashInteractive here: the agents flake pins
          # both in every jail.
          # No CARGO_TARGET_DIR redirect: each worktree builds into its own
          # in-tree, gitignored `target/` (cargo's default). Per-worktree build
          # isolation is then correct by construction — both dispatch arms, `just`
          # and raw `cargo`, with no shared target to thrash and no env channel
          # required (SL-156; ADR-008 D-B1/D-B5 via REV-011). The original
          # cross-mount concern (a jail-built test binary bakes CARGO_BIN_EXE at
          # the jail mount path and spawn-fails on the host, and vice versa) is
          # satisfied *by* in-tree targets: jail `/workspace/doctrine/target` and
          # host `/home/.../doctrine/target` are distinct physical dirs, so no
          # binary is shared across mounts. Persistent trees (main/edge) keep a
          # warm cache in their bound `target/`; ephemeral forks cold-build (the
          # deliberate trade — sccache is the warm-fork lever, ADR-008 D-B4). Host
          # was always on default `target/` and is unchanged. Do not re-add a
          # shared redirect: it reintroduces the cross-worktree fingerprint thrash
          # this removal fixes.
          #(ro-bind "${pkgs.coreutils}/bin/env" "/usr/bin/env")
          #(add-pkg-deps [pkgs.coreutils])
          # No doctrine bind here: the agents flake's jail default ro-binds the
          # host's doctrine install dir (~/.local/share/doctrine/bin, from
          # `just install`) at /run/doctrine-bin, ahead on PATH. A directory
          # bind, not a file bind, so a reinstall's atomic rename reaches a
          # running jail (IMP-249).
        ];

        # workspaceDeps now sourced from the JAIL_WORKSPACE_DEPS env var
        # (set in the gitignored .envrc; requires `use flake --impure`).
        # makeJailedAgent reads + merges it, so nothing portable lives here.

        # Every jail in this flake shares the same confinement posture; only
        # the agent and its subagent graph differ. Holding the shared half here
        # keeps a jail added later from silently missing the project's tools
        # and env.
        mkJail = maker: args:
          maker (
            {
              profile = "specDev";
              extraPkgs = projectPkgs;
              extraOptions = jailEnvOptions;
              gateway = jailLib.gatewayClient "pi";
            }
            // args
          );

        jailPkgs = lib.optionalAttrs isLinux {
          jailed-pi = mkJail jailLib.makeJailedPi {
            allowSelfAsSubagent = true;
            maxSubagentDepth = 2;
          };
          jailed-pi-research = mkJail jailLib.makeJailedPi {
            name = "pi-research";
            profile = "research";
            allowSelfAsSubagent = true;
          };
          jailed-claude = mkJail jailLib.makeJailedClaude {
            allowSelfAsSubagent = true;
            # claude can spawn pi inside its own jail (no re-jail).
            subagents = [
              "pi"
            ];
            maxSubagentDepth = 2;
          };
          jailed-codex = mkJail jailLib.makeJailedCodex {
            subagents = [
              "claude"
              "pi"
              "codex"
            ];
          };
          jailed-shell = mkJail jailLib.makeJailedAgent {
            name = "shell";
            agent = pkgs.zsh;
            subagents = [
              "pi"
              "claude"
            ];
          };

          bubblewrap = pkgs.bubblewrap;
        };

        # quotient `form-1` forms (`quotient help form`): the jails quotient's
        # `pi_agent` runs a delegate in, carrying this project's toolchain and
        # env. Interim copy of satan's `quotientFormFamily` (satan flake.nix);
        # retire for quotient IMP-003's shared builder (CHR-183).
        #
        # The root is the server-supplied cwd, bound at its own absolute path:
        # read-only or writable is the capability's one word. The form's one
        # positional is the session directory; pi's argv follows verbatim.
        quotientForms = let
          sessionVar = "QUOTIENT_ACTIVE_SESSION_DIR";
          rawArg = jailLib.combinators.unsafe-add-raw-args;
          mkForm = capability: let
            name = "doctrine-quotient-${capability}";
            formJail = mkJail jailLib.makeJailedPi {
              inherit name;
              # No /workspace launch-dir mount and no env-sourced deps: the
              # root and session dir below are the form's whole addition.
              mountLaunchDir = false;
              workspaceDepsFromEnv = false;
              extraOptions =
                jailEnvOptions
                ++ [
                  # The root is bound at its host path, so host and delegate
                  # share `target/`: differing rustflags (.envrc's mold flag)
                  # would make each rebuild over the other. Not RUSTC_WRAPPER:
                  # sccache's host cache dir is unwritable here, and the
                  # wrapper does not enter cargo's fingerprint.
                  (jailLib.combinators.try-fwd-env "CARGO_BUILD_RUSTFLAGS")
                  (rawArg (
                    if capability == "read"
                    then ''--ro-bind "$PWD" "$PWD"''
                    else ''--bind "$PWD" "$PWD"''
                  ))
                  (rawArg ("--bind \"$" + sessionVar + "\" \"$" + sessionVar + "\""))
                  (rawArg "--tmpfs /tmp")
                  # After the generator's own --chdir "$HOME": the later wins.
                  (rawArg ''--chdir "$PWD"'')
                  (rawArg "--die-with-parent")
                ];
            };
          in
            pkgs.writeShellScriptBin name ''
              set -euo pipefail
              if [ "$#" -lt 1 ]; then
                echo "${name}: missing the session directory (the one positional)" >&2
                exit 64
              fi
              session_dir=$1
              shift
              case "$session_dir" in
                /*) ;;
                *) echo "${name}: session directory must be absolute: $session_dir" >&2; exit 64 ;;
              esac
              if [ ! -d "$session_dir" ]; then
                echo "${name}: no session directory at $session_dir" >&2
                exit 64
              fi
              export ${sessionVar}="$session_dir"
              exec ${formJail}/bin/jailed-${name} "$@"
            '';
        in
          lib.optionalAttrs isLinux {
            doctrine-quotient-read = mkForm "read";
            doctrine-quotient-write = mkForm "write";
          };

        # Frontend: hermetic bun build → web/map/dist, embedded into the binary
        # via rust-embed (release profile reads web/map/dist/, debug reads
        # web/map/). dist is gitignored, so crane's git-based cleanCargoSource
        # drops it; we build it here and graft it into the rust source tree.
        #
        # Source for the bun build, sans the local node_modules/dist (a plain
        # nix path import copies everything — gitignore is not consulted).
        webSrc = lib.cleanSourceWith {
          src = ./web/map;
          filter = path: _type: let
            b = baseNameOf path;
          in
            b != "node_modules" && b != "dist";
        };

        # node_modules via a fixed-output derivation keyed on bun.lock.
        # REGENERATE webModules.outputHash whenever web/map/bun.lock changes —
        # `nix build` prints the correct `got: sha256-…` on mismatch.
        webModules = stdenv.mkDerivation {
          name = "doctrine-web-node-modules";
          src = webSrc;
          nativeBuildInputs = [
            pkgs.bun
            pkgs.cacert
          ];
          dontConfigure = true;
          buildPhase = ''
            export HOME=$TMPDIR
            # Nix gives fixed-output builds host network but binds NO CA bundle
            # into the sandbox — a derivation must bring its own. Without it
            # bun's registry TLS never completes and it retries with backoff,
            # which presents as a HANG (version banner printed, then nothing —
            # no package lines, no error). Diagnosed by the same build passing
            # under `--option sandbox false`, which exposes the host /etc.
            export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
            export NIX_SSL_CERT_FILE=$SSL_CERT_FILE
            bun install --frozen-lockfile --no-progress
          '';
          installPhase = ''
            mkdir -p $out
            cp -R node_modules $out/node_modules
          '';
          dontFixup = true;
          outputHashMode = "recursive";
          outputHashAlgo = "sha256";
          outputHash = "sha256-Fn1c5nzfclWXvney5hCVNUviKz3oeyYkl45Ry0M/w8c=";
        };

        webDist = stdenv.mkDerivation {
          name = "doctrine-web-dist";
          src = webSrc;
          nativeBuildInputs = [
            pkgs.bun
            pkgs.nodejs_latest
          ];
          configurePhase = ''
            export HOME=$TMPDIR
            cp -R ${webModules}/node_modules ./node_modules
            chmod -R u+w ./node_modules
          '';
          buildPhase = ''
            node node_modules/vite/bin/vite.js build
          '';
          installPhase = ''
            mkdir -p $out
            cp -R dist/. $out/
          '';
        };

        # Rust binary — crane for workspace-aware builds.
        # cleanCargoSource uses git ls-files + Cargo.toml exclude list, so
        # plugins/ and install/ (git-tracked, embedded via rust-embed) and
        # crates/cordage (workspace member) are included automatically. The
        # built web dist is grafted on top (it is gitignored, hence absent).
        # Build with the SAME toolchain the devshell + `just lint` use — the
        # `rust` binding above, not a second spelling of it. Crane defaults to
        # nixpkgs-stable rustc, and the version skew flips lint verdicts (e.g.
        # unfulfilled_lint_expectations on consts referenced only by a dead fn →
        # spurious -D warnings failure).
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;
        # cleanCargoSource keeps ONLY .rs/.toml/.lock, silently stripping every
        # non-rust embedded asset (RustEmbed folders + include_str! targets) —
        # the folders survive via their .toml siblings so it still compiles, but
        # the binary ships asset-incomplete. Graft the complete git-tracked
        # asset roots back so the embed matches what `cargo install` sees on
        # disk; web/map/dist is the freshly built frontend (gitignored, absent
        # even from a full tree).
        cleanedSrc = craneLib.cleanCargoSource ./.;
        srcWithDist = pkgs.runCommandLocal "doctrine-src" {} ''
          cp -R ${cleanedSrc} $out
          chmod -R u+w $out
          rm -rf $out/plugins $out/install $out/memory $out/templates $out/publication
          mkdir -p $out/plugins $out/install $out/memory $out/web/map/dist $out/templates $out/publication
          cp -R ${./plugins}/.     $out/plugins/
          cp -R ${./install}/.     $out/install/
          cp -R ${./memory}/.      $out/memory/
          cp -R ${./templates}/.   $out/templates/
          cp -R ${./publication}/. $out/publication/
          cp -R ${webDist}/. $out/web/map/dist/
          chmod -R u+w $out
        '';
        doctrine = craneLib.buildPackage {
          pname = "doctrine";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = srcWithDist;
          # Deps layer never needs the assets — keep it on the lean source.
          cargoArtifacts = craneLib.buildDepsOnly {
            pname = "doctrine-deps";
            src = cleanedSrc;
            cargoExtraArgs = "--workspace";
          };
          cargoExtraArgs = "--workspace";
          doCheck = false; # tests need a live Postgres
          meta = {
            mainProgram = "doctrine";
            description = "Project governance and task-management CLI";
          };
        };
      in {
        # The SAME instance the let block above uses — not a second
        # `import nixpkgs`. Two instantiations per system double the eval and
        # hand the devshell module a different pkgs than `packages` is built
        # from, which forks drvs and rebuilds from scratch (see the `dirge`
        # note below for that failure mode).
        _module.args.pkgs = pkgs;

        packages =
          jailPkgs
          // quotientForms
          // {
            inherit doctrine;
            # Unjailed dirge pulled straight from the agents flake — same pkgs +
            # callPackage as agents builds it, so it's the
            # identical derivation (one drv hash, store path reused, no rebuild).
            # Must come from agents's eval; doctrine's own pkgs (different nixpkgs +
            # rust-overlay pin) would fork the drv and rebuild from scratch.
            dirge = inputs.agents.packages.${system}.dirge;
            # Frontend layers exposed individually so they are buildable and
            # debuggable without the full rust binary — `nix build .#web-modules -L`
            # streams the bun install, `.#web-dist` the vite build, without
            # reaching them only through `doctrine`.
            web-modules = webModules;
            web-dist = webDist;
            # The devshell's tool set as one closure, for out-of-tree consumers
            # that need the same tools without the devshell (the microvm
            # capsule puts this straight into the guest's systemPackages, so
            # the VM and this devshell can't drift).
            dev-tools = pkgs.buildEnv {
              name = "doctrine-dev-tools";
              # doctrine rides here, not in devToolPkgs: the host devshell and
              # jails take the cargo install (`just install`); the VM guest
              # can't reach the host's, so it keeps the crane build.
              paths = devToolPkgs ++ [doctrine];
              ignoreCollisions = true;
            };
            default = doctrine;
          };

        devshells.default = {
          packages =
            projectPkgs
            # Bare (unjailed) agents on the host PATH, from agents's eval
            # (jailLib.unjailed): pi and claude are the drvs the jails bundle;
            # dirge has no jail, and here == packages.dirge below.
            ++ (with jailLib.unjailed; [
              pi
              dirge
              claude
            ])
            ++ lib.optionals isLinux (lib.attrValues jailPkgs);

          # darwin + nix: rustc's link line emits `-liconv` with `-nodefaultlibs`,
          # which strips the Nix clang wrapper's auto-injected NIX_LDFLAGS — so
          # libiconv is never on the search path and the link dies with
          # `library not found for -liconv`. Hand rustc an explicit `-L`, the one
          # flag it passes through `-nodefaultlibs`. Append so a caller's own
          # RUSTFLAGS survive. No-op off darwin (glibc provides iconv) and off
          # nix (Apple's /usr/bin/cc finds the SDK's libiconv.tbd natively).
          devshell.startup.iconv-rustflags.text = lib.optionalString stdenv.hostPlatform.isDarwin ''
            export RUSTFLAGS="''${RUSTFLAGS:+$RUSTFLAGS }-L ${pkgs.libiconv}/lib"
          '';

          env = [
            {
              name = "LD_LIBRARY_PATH";
              value = lib.makeLibraryPath [pkgs.stdenv.cc.cc.lib];
            }
            {
              name = "UV_PYTHON_PREFERENCE";
              value = "only-system";
            }
          ];

          commands = [
            {
              name = "d";
              help = "short for doctrine";
              command = "doctrine $@";
            }
            {
              name = "jd";
              help = "just doctrine";
              command = "just doctrine $@";
            }
            {
              name = "drn";
              help = "short for doctrine";
              command = "doctrine $@";
            }
            {
              name = "jpi";
              help = "jailed-pi";
              command = "jailed-pi $@";
            }
            {
              name = "jcx";
              help = "jailed-codex";
              command = "jailed-codex $@";
            }
            {
              name = "jcl";
              help = "jailed-claude --dangerously-skip-permissions";
              command = "jailed-claude --dangerously-skip-permissions $@";
            }
            {
              name = "jail-zsh";
              help = "jailed shell (zsh) in pi's context";
              command = "jailed-shell $@";
            }
          ];
        };
      };
    };
}
