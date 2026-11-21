{
  description = "sandhi - reach every network from every machine (PRJ-0001)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    # The yajna CLI for the notebook under notes/ (github:leoank/yajna).
    yajna.url = "github:leoank/yajna";
    yajna.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" "aarch64-linux" ];

      perSystem = { pkgs, system, lib, ... }:
        let
          # libtailscale as a static archive for the tailnet crate (ADR-0002), as saarthi
          # builds it: `ac7ed43` marks the descriptors it hands out close-on-exec
          # ([[saarthi/EXP-20261112-rq8kgc]]). Bump the Go attribute with the rev when its
          # go.mod moves.
          libtailscale = pkgs.buildGo127Module {
            pname = "libtailscale";
            version = "0-unstable-2026-10-30";
            src = pkgs.fetchFromGitHub {
              owner = "tailscale";
              repo = "libtailscale";
              rev = "ac7ed43af14e19e1c1810ee9f8319872c6beb412";
              hash = "sha256-K8QGeCyKBcYqBt9O192QpQ4EFRjgiPPE5djD+jyoM3M=";
            };
            vendorHash = "sha256-1Y21dNgCNMfVQvZN6k66GHvgdZo8lle5J8B5UES2Ryg=";
            env.CGO_ENABLED = "1";
            # A `package main` with cgo exports, built as a c-archive.
            buildPhase = ''
              runHook preBuild
              go build -buildvcs=false -buildmode=c-archive -o libtailscale.a .
              runHook postBuild
            '';
            installPhase = ''
              runHook preInstall
              install -Dm644 libtailscale.a $out/lib/libtailscale.a
              install -Dm644 tailscale.h $out/include/tailscale.h
              runHook postInstall
            '';
            doCheck = false;
          };

          # The cargo workspace; docs/ and notes/ stay out of it.
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [ ./Cargo.toml ./Cargo.lock ./crates ];
          };
        in
        {
          packages = rec {
            inherit libtailscale;
            # The `sandhi` binary; tests run in checks.workspace.
            sandhi = pkgs.rustPlatform.buildRustPackage {
              pname = "sandhi";
              version = "0.1.0";
              inherit src;
              cargoLock.lockFile = ./Cargo.lock;
              LIBTAILSCALE_LIB_DIR = "${libtailscale}/lib";
              cargoBuildFlags = [ "--bin" "sandhi" ];
              doCheck = false;
              meta.mainProgram = "sandhi";
            };
            default = sandhi;
          };

          devShells.default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.rustfmt
              pkgs.rust-analyzer
              inputs.yajna.packages.${system}.yajna
            ];
            LIBTAILSCALE_LIB_DIR = "${libtailscale}/lib";
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };

          # The cargo workspace, built and tested.
          checks.workspace = pkgs.rustPlatform.buildRustPackage {
            pname = "sandhi";
            version = "0.1.0";
            inherit src;
            cargoLock.lockFile = ./Cargo.lock;
            LIBTAILSCALE_LIB_DIR = "${libtailscale}/lib";
          };

          # The notebook passes yajna validate.
          checks.notebook = pkgs.runCommand "sandhi-notebook" {
            nativeBuildInputs = [ inputs.yajna.packages.${system}.yajna ];
          } ''
            cd ${lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [ ./yajna.toml ./notes ];
            }}
            yajna --root . validate
            touch $out
          '';
        };
    };
}
