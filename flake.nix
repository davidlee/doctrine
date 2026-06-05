{
  description = "satan-attrd — SATAN attribute layer daemon";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    pub.url = "path:/home/david/flakes/pub";
    llm-agents.url = "github:numtide/llm-agents.nix";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs @ {
    flake-parts,
    rust-overlay,
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

          stdenv.cc # cc/ld on PATH (linker for cargo build)
          stdenv.cc.cc.lib
          codex
          nodejs_latest
          sqlite
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
            exposePostgres = true;
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
          };
          # jailed-codex = jailLib.makeJailedCodex {
          #   profile = "specDev";
          #   extraPkgs = projectPkgs;
          #   extraOptions = jailEnvOptions;
          #   inherit workspaceDeps;
          # };
          bubblewrap = pkgs.bubblewrap;
        };

        # Rust binary
        doctrined = pkgs.rustPlatform.buildRustPackage {
          pname = "doctrined";
          version = "0.1.0";
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type: let
              baseName = builtins.baseNameOf path;
              relPath = lib.removePrefix (toString ./. + "/") (toString path);
            in
              # Include single-crate sources + migrations baked into the binary
              # by sqlx::migrate! at build time.
              (
                type
                == "directory"
                && builtins.elem baseName [
                  "src"
                  "tests"
                  "migrations"
                ]
              )
              || baseName == "Cargo.toml"
              || baseName == "Cargo.lock"
              || lib.hasPrefix "src/" relPath
              || lib.hasPrefix "tests/" relPath
              || lib.hasPrefix "migrations/" relPath;
          };
          cargoLock.lockFile = ./Cargo.lock;
          doCheck = false; # tests require a live Postgres
          meta = {
            mainProgram = "doctrined";
            description = "doctrinetable daemon";
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
            inherit doctrined;
            default = doctrined;
          };

        devshells.default = {
          packages = projectPkgs ++ lib.optionals isLinux (lib.attrValues jailPkgs);

          env = [
            {
              name = "LD_LIBRARY_PATH";
              value = lib.makeLibraryPath [pkgs.stdenv.cc.cc.lib];
            }
          ];

          commands = [
            {
              name = "jpi";
              help = "op -> jailed-pi";
              command = "op run -- jailed-pi $@";
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
