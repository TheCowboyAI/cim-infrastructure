{
  description = "CIM Infrastructure - Event-sourced infrastructure management for CIM";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Source for the entire workspace
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [
            # No special dependencies needed for domain logic
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
          ];
        };

        # Build *just* the cargo dependencies
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the workspace
        cim-infrastructure = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          doCheck = false; # We'll run tests separately
        });

        # Run tests
        cim-infrastructure-tests = craneLib.cargoNextest (commonArgs // {
          inherit cargoArtifacts;
          partitions = 1;
          partitionType = "count";
        });
      in
      {
        checks = {
          inherit cim-infrastructure cim-infrastructure-tests;

          # Run clippy
          cim-infrastructure-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets --workspace -- --deny warnings";
          });

          # Check formatting
          cim-infrastructure-fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        packages = {
          default = cim-infrastructure;
          inherit cim-infrastructure;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks.${system};

          # Extra inputs for development
          nativeBuildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-nextest
            cargo-edit
            cargo-outdated
            cargo-audit
            cargo-license
            cargo-tarpaulin

            # NATS server for integration testing
            nats-server
            nats-top

            # General development tools
            just
            bacon
            mold
            sccache

            # Documentation
            mdbook
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = 1;
          RUST_LOG = "debug";

          # NATS configuration for testing
          NATS_URL = "nats://localhost:4222";

          shellHook = ''
            echo "CIM Infrastructure Development Shell"
            echo "===================================="
            echo ""
            echo "Workspace modules:"
            echo "  - cim-domain-infrastructure (event-sourced domain)"
            echo ""
            echo "Available commands:"
            echo "  cargo build --workspace     - Build all modules"
            echo "  cargo test --workspace      - Run all tests"
            echo "  cargo watch                 - Watch for changes and rebuild"
            echo "  cargo nextest run           - Run tests with nextest"
            echo "  nix flake check             - Check the flake"
            echo ""
            echo "NATS commands (for integration testing):"
            echo "  nats-server                 - Start NATS server"
            echo "  nats-top                    - Monitor NATS server"
            echo ""
            echo "NATS URL: $NATS_URL"
          '';
        };
      });
}
