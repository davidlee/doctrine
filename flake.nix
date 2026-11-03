{
  description = "doctrine: dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
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
        # `cargo build`. mold arrives via CARGO_BUILD_RUSTFLAGS from the host's
        # env (~/flakes/agents/dev-env.nix: host shells, jails and forms) instead.
        # It also throws outright on darwin ("Mold can't be used to emit Mach-O"),
        # and this stdenv is forced on every darwin eval path.
        inherit (pkgs) stdenv;
        isLinux = stdenv.hostPlatform.isLinux;

        # What a doctrine working environment needs, independent of how (or
        # whether) it is confined. Exported as `packages.dev-tools`; consumers
        # build the environment around it (agents, jails, forms live there).
        devToolPkgs = commonToolPkgs ++ conformancePkgs;

        commonToolPkgs = with pkgs; [
          jujutsu
          jjui
          just
          rust # the single toolchain, incl. rust-analyzer
          mold # must be on PATH for the host env's -C link-arg=-fuse-ld=mold
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
          # capsule-verify (justfile) and the live-bwrap rows run it (CHR-184).
          bubblewrap
        ]);

        # What the tools need beyond packages, as plain data, for a consumer
        # that builds the environment around dev-tools (flakes:SL-023's
        # dev-envs, CHR-184): `env` is set as given; each `hostEnv` name is
        # forwarded from the launch environment when set.
        devEnv = {
          env.LD_LIBRARY_PATH = lib.makeLibraryPath [pkgs.stdenv.cc.cc.lib];
          hostEnv = ["DOCTRINE_BIN"]; # a per-checkout override
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
        # from, which forks drvs and rebuilds from scratch.
        _module.args.pkgs = pkgs;

        packages = {
          inherit doctrine;
          # Frontend layers exposed individually so they are buildable and
          # debuggable without the full rust binary — `nix build .#web-modules -L`
          # streams the bun install, `.#web-dist` the vite build, without
          # reaching them only through `doctrine`.
          web-modules = webModules;
          web-dist = webDist;
          # The gate's tools as one closure, for out-of-tree consumers that
          # build the environment around them (SL-022's producer contract;
          # flakes:SL-023's dev-envs). No product: a consumer snapshots the
          # checkout per reload, and a crane doctrine here would rebuild with
          # every snapshot (CHR-184). The host and jails take `just install`.
          dev-tools = pkgs.buildEnv {
            name = "doctrine-dev-tools";
            paths = devToolPkgs;
            ignoreCollisions = true;
            passthru = {inherit devEnv;};
          };
          # The fleet guest's tools: the same set plus the product, which the
          # guest cannot reach from the host's `just install` (CHR-184). The
          # microvm capsule puts this straight into the guest's
          # systemPackages, so the VM and the devshell can't drift.
          capsule-tools = pkgs.buildEnv {
            name = "doctrine-capsule-tools";
            paths = devToolPkgs ++ [doctrine];
            ignoreCollisions = true;
          };
          default = doctrine;
        };

        # The fallback shell, where no consumer builds one (the Mac, or a
        # checkout without ~/flakes): the same tools and env as dev-tools.
        devshells.default = {
          packages = devToolPkgs;

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

          env = lib.mapAttrsToList (name: value: {inherit name value;}) devEnv.env;

          commands = [
            {
              name = "jd";
              help = "just doctrine";
              command = "just doctrine $@";
            }
          ];
        };
      };
    };
}
