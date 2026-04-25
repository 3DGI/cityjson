{
  description = "3DGI CityJSON Rust workspace — dev shell + container image";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Stable toolchain. We mirror the channel from rust-toolchain.toml
        # so the flake stays in lockstep with the workspace pin, then build
        # a `minimal` profile + only the components the workspace actually
        # uses — rust-docs and several preview components weigh in at >1 GB
        # combined and we never reference them.
        rustToolchainPin =
          (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain;
        rustStable = pkgs.rust-bin.${rustToolchainPin.channel}.latest.minimal.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Nightly is needed for `just doc` (docsrs cfg) and `just miri` only.
        # We pin the date so rebuilds are reproducible — bump as needed.
        rustNightly = pkgs.rust-bin.nightly."2026-01-15".minimal.override {
          extensions = [ "rust-src" "miri" "rustfmt" "clippy" "llvm-tools-preview" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # `cargo +nightly ...` is rustup syntax. Without rustup installed we
        # shim it with a tiny wrapper that dispatches between the two
        # toolchains. Everything else (rustc, clippy, rustfmt) resolves to the
        # stable toolchain via PATH.
        cargoShim = pkgs.writeShellScriptBin "cargo" ''
          case "''${1:-}" in
            +nightly)
              shift
              exec ${rustNightly}/bin/cargo "$@"
              ;;
            +stable)
              shift
              exec ${rustStable}/bin/cargo "$@"
              ;;
            +*)
              echo "cargo shim: unknown toolchain override '$1' (only +stable/+nightly supported)" >&2
              exit 1
              ;;
            *)
              exec ${rustStable}/bin/cargo "$@"
              ;;
          esac
        '';

        # Mirror the shim for rustc, in case anything calls `rustc +nightly`.
        rustcShim = pkgs.writeShellScriptBin "rustc" ''
          case "''${1:-}" in
            +nightly) shift; exec ${rustNightly}/bin/rustc "$@" ;;
            +stable)  shift; exec ${rustStable}/bin/rustc  "$@" ;;
            +*)
              echo "rustc shim: unknown toolchain override '$1'" >&2
              exit 1
              ;;
            *) exec ${rustStable}/bin/rustc "$@" ;;
          esac
        '';

        # Tooling shared by the dev shell and the container image.
        commonTools = with pkgs; [
          cargoShim
          rustcShim
          rustStable
          rustNightly

          # Build orchestration
          just

          # Native build deps for crates that compile C/C++ (rusqlite bundled,
          # cc-rs, the cityjson-lib/ffi/cpp consumer).
          gcc
          binutils
          gnumake
          cmake
          pkg-config

          # FFI / wasm tooling
          rust-cbindgen
          wasm-bindgen-cli
          wasm-pack
          binaryen

          # Python toolchain — uv manages venvs, the interpreters are pinned
          # via nix so all three CI versions are available offline.
          uv
          python311
          python312
          python313

          # SQLite headers/CLI for poking at cityjson-index sidecars (the
          # crate itself uses bundled sqlite, this is for human use).
          sqlite

          # General dev convenience
          git
          openssh
          curl
          ripgrep
          fd
          jq
          bash
          coreutils
          findutils
          gnused
          gnugrep
          gnutar
          gzip
          xz
          which
          less
          cacert
        ];

        shellEnv = {
          # Make `cargo +nightly` behave even if PATH ordering is ever wrong.
          RUST_BACKTRACE = "1";
          # Avoid uv writing into ~/.cache when run inside an ephemeral
          # container; default is fine on a workstation.
          UV_LINK_MODE = "copy";
        };

        # A single union profile of every dev tool. Used by the container
        # image so /bin contains all the binaries side-by-side.
        devEnv = pkgs.buildEnv {
          name = "cityjson-dev-env";
          paths = commonTools;
          # Several rust toolchains expose the same binary names (cargo,
          # rustc, …); the cargo/rustc shims must win, so they appear first
          # in `commonTools` — `priority` is left at the default and any
          # collision falls back to the order in `paths`.
          ignoreCollisions = true;
        };
      in {
        devShells.default = pkgs.mkShell ({
          name = "cityjson-dev";
          packages = commonTools;
          shellHook = ''
            # Ensure the +nightly shims always shadow the toolchain binaries,
            # regardless of how mkShell orders nativeBuildInputs.
            export PATH="${cargoShim}/bin:${rustcShim}/bin:$PATH"
            echo "cityjson dev shell — rust $(${rustStable}/bin/rustc --version | awk '{print $2}'), nightly $(${rustNightly}/bin/rustc --version | awk '{print $2}')"
            echo "available recipes: just --list"
          '';
        } // shellEnv);

        # OCI image consumable by docker / podman / nerdctl. Built reproducibly
        # from the same package set that drives `nix develop`.
        #
        #   nix build .#devcontainer
        #   ./result | podman load           # tag: cityjson-dev:latest
        #
        packages.devcontainer = pkgs.dockerTools.streamLayeredImage {
          name = "cityjson-dev";
          tag = "latest";

          contents = [ devEnv ] ++ (with pkgs; [
            dockerTools.binSh
            dockerTools.usrBinEnv
            dockerTools.caCertificates
            dockerTools.fakeNss
          ]);

          # Keep the per-user mutable state on a writable path inside the
          # container. /tmp is always writable; /workspaces is the bind mount.
          extraCommands = ''
            mkdir -p tmp workspaces/cityjson home/dev
            chmod 1777 tmp
          '';

          enableFakechroot = true;
          fakeRootCommands = ''
            ${pkgs.dockerTools.shadowSetup}
          '';

          config = {
            Cmd = [ "/bin/bash" ];
            WorkingDir = "/workspaces/cityjson";
            Env = [
              "PATH=${devEnv}/bin:/bin:/usr/bin"
              "HOME=/home/dev"
              "RUST_BACKTRACE=1"
              "UV_LINK_MODE=copy"
              "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
              "LANG=C.UTF-8"
            ];
            Labels = {
              "org.opencontainers.image.source" = "https://github.com/3DGI/cityjson";
              "org.opencontainers.image.description" = "CityJSON workspace dev environment built from flake.nix";
            };
          };
        };

        # `nix flake check` wires this in automatically.
        checks.devShell = self.devShells.${system}.default;

        formatter = pkgs.nixpkgs-fmt;
      });
}
