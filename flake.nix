{
  description = "doctrine: dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    pub.url = "path:/home/david/flakes/pub";
    llm-agents.url = "github:numtide/llm-agents.nix";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs @ {
    flake-parts,
    rust-overlay,
    crane,
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

      perSystem = {
        pkgs,
        system,
        ...
      }: let
        inherit (pkgs) lib stdenv;
        isLinux = stdenv.isLinux;

        jailLib =
          if isLinux
          then inputs.pub.lib.${system}.mkJailedAgents {inherit (inputs) llm-agents;}
          else {};

        projectPkgs = with pkgs; [
          jujutsu
          jjui
          just
          rust-bin.beta.latest.default
          rust-analyzer
          cargo-edit # `cargo set-version` for the release recipe

          stdenv.cc # cc/ld on PATH (linker for cargo build)
          stdenv.cc.cc.lib
          codex
          nodejs_latest
          eslint
          bun
          typescript
          typescript-language-server

          graphviz
        ];

        jailEnvOptions = with jailLib.combinators; [
          (try-fwd-env "OPENROUTER_API_KEY")
          (set-env "LD_LIBRARY_PATH" "${lib.makeLibraryPath [pkgs.stdenv.cc.cc.lib]}")
          # Jail builds into its own target dir so it never clobbers the host's.
          # The repo binds rw into the jail at a different absolute path, but
          # cargo bakes CARGO_BIN_EXE (the e2e-test spawn path) at compile time —
          # a shared target/ leaves a jail-built test binary pointing at the jail
          # mount path, which spawn-fails when run on the host (and vice versa).
          # Park it under the persisted, out-of-tree ~/.cargo (in-jail HOME
          # appears as /home/david, backed by host /home/agent): survives
          # launches (warm cache) and keeps the bound working tree clean. Host
          # stays on default target/.
          (set-env "CARGO_TARGET_DIR" "/home/david/.cargo/doctrine-target-jail")
          # Share the HOST doctrine binary into the jail. persist-home already
          # mounts an isolated, writable ~/.cargo; this ro-binds the host's real
          # install on top (extraOptions applies after persist-home, so it wins)
          # so host + jail execute ONE physical binary at one path string. Kills
          # the boot-snapshot version-skew thrash — single source of truth is the
          # host `cargo install --path .`. ro-bind-try: jail still launches if the
          # binary isn't installed yet. Tilde expands in the host launcher shell;
          # in-jail $HOME is also /home/david, so src == dst.
          (try-readonly (noescape "~/.cargo/bin/doctrine"))
          # Put cargo-bin on the jail PATH so the SessionStart hook's bare
          # `doctrine boot` resolves to the shared binary above.
          (add-path "/home/david/.cargo/bin")
        ];

        # workspaceDeps now sourced from the JAIL_WORKSPACE_DEPS env var
        # (set in the gitignored .envrc; requires `use flake --impure`).
        # makeJailedAgent reads + merges it, so nothing portable lives here.

        jailPkgs = lib.optionalAttrs isLinux {
          jailed-pi = jailLib.makeJailedPi {
            profile = "specDev";
            # exposePostgres = true;
            allowSelfAsSubagent = true;
            maxSubagentDepth = 2;
            extraPkgs = projectPkgs;
            extraOptions = jailEnvOptions;
          };
          # jailed-pi-research = jailLib.makeJailedPi {
          #   name = "pi-research";
          #   profile = "research";
          #   extraPkgs = projectPkgs;
          #   extraOptions = jailEnvOptions;
          #   inherit workspaceDeps;
          # };
          jailed-claude = jailLib.makeJailedClaude {
            profile = "specDev";
            extraPkgs = projectPkgs;
            extraOptions = jailEnvOptions;
            allowSelfAsSubagent = true;
            # claude can spawn pi/dirge inside its own jail (no re-jail).
            subagents = ["pi" "dirge"];
            maxSubagentDepth = 2;
          };
          # jailed-codex = jailLib.makeJailedCodex {
          #   profile = "specDev";
          #   extraPkgs = projectPkgs;
          #   extraOptions = jailEnvOptions;
          #   inherit workspaceDeps;
          # };
          jailed-dirge = jailLib.makeJailedDirge {
            profile = "specDev";
            # exposePostgres = true;
            allowSelfAsSubagent = true;
            # maxSubagentDepth = 2;
            extraPkgs = projectPkgs;
            extraOptions = jailEnvOptions;
          };

          bubblewrap = pkgs.bubblewrap;
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
          nativeBuildInputs = [pkgs.bun];
          dontConfigure = true;
          buildPhase = ''
            export HOME=$TMPDIR
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
          nativeBuildInputs = [pkgs.bun pkgs.nodejs_latest];
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
        # Build with the SAME toolchain the devshell + `just lint` use
        # (rust-bin.beta.latest); crane defaults to nixpkgs-stable rustc, and
        # the version skew flips lint verdicts (e.g. unfulfilled_lint_expectations
        # on consts referenced only by a dead fn → spurious -D warnings failure).
        craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.rust-bin.beta.latest.default;
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
          rm -rf $out/plugins $out/install $out/memory $out/.pi/extensions/doctrine
          mkdir -p $out/plugins $out/install $out/memory $out/.pi/extensions $out/web/map/dist
          cp -R ${./plugins}/. $out/plugins/
          cp -R ${./install}/. $out/install/
          cp -R ${./memory}/. $out/memory/
          cp -R ${./.pi/extensions/doctrine} $out/.pi/extensions/doctrine
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
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };

        packages =
          jailPkgs
          // {
            inherit doctrine;
            # Unjailed dirge pulled straight from the pub flake — same pkgs +
            # callPackage as the jailed-dirge wrapper bundles, so it's the
            # identical derivation (one drv hash, store path reused, no rebuild).
            # Must come from pub's eval; doctrine's own pkgs (different nixpkgs +
            # rust-overlay pin) would fork the drv and rebuild from scratch.
            dirge = inputs.pub.packages.${system}.dirge;
            default = doctrine;
          };

        devshells.default = {
          packages =
            projectPkgs
            # Bare (unjailed) agents on the host PATH, mirroring the jailed
            # set. From pub's eval (jailLib.unjailed) so they're the identical
            # drvs the jails bundle — dirge here == packages.dirge below.
            ++ lib.optionals isLinux (with jailLib.unjailed; [pi dirge claude])
            ++ lib.optionals isLinux (lib.attrValues jailPkgs);

          # darwin + nix: rustc's link line emits `-liconv` with `-nodefaultlibs`,
          # which strips the Nix clang wrapper's auto-injected NIX_LDFLAGS — so
          # libiconv is never on the search path and the link dies with
          # `library not found for -liconv`. Hand rustc an explicit `-L`, the one
          # flag it passes through `-nodefaultlibs`. Append so a caller's own
          # RUSTFLAGS survive. No-op off darwin (glibc provides iconv) and off
          # nix (Apple's /usr/bin/cc finds the SDK's libiconv.tbd natively).
          devshell.startup.iconv-rustflags.text = lib.optionalString stdenv.isDarwin ''
            export RUSTFLAGS="''${RUSTFLAGS:+$RUSTFLAGS }-L ${pkgs.libiconv}/lib"
          '';

          env = [
            {
              name = "LD_LIBRARY_PATH";
              value = lib.makeLibraryPath [pkgs.stdenv.cc.cc.lib];
            }
          ];

          commands = [
            {
              name = "drn";
              help = "short for doctrine";
              command = "doctrine $@";
            }
            {
              name = "jdi";
              help = "jailed-dirge --yolo";
              command = "jailed-dirge $@";
            }
            {
              name = "jpi";
              help = "jailed-pi";
              command = "jailed-pi $@";
            }
            {
              name = "jcl";
              help = "jailed-claude --dangerously-skip-permissions";
              command = "jailed-claude --dangerously-skip-permissions $@";
            }
          ];
        };
      };
    };
}
